"""Minimal Chrome DevTools harness for offline viewer interaction tests."""
import base64
import json
import os
from pathlib import Path
import socket
import struct
import subprocess
import tempfile
import time
import urllib.request


class Chrome:
    def __init__(self, work, page):
        profile = work / 'chrome'
        profile.mkdir()
        self.log = (work / 'chrome.log').open('w')
        # Chrome creates a Unix socket under TMPDIR. Point a short path to the
        # runner-owned directory so long validation-unit paths still work.
        self.temp_link = tempfile.TemporaryDirectory(prefix='ffta-viewer-', dir='/tmp')
        short_work = Path(self.temp_link.name) / 'work'
        short_work.symlink_to(work.resolve(), target_is_directory=True)
        env = dict(os.environ, TMPDIR=str(short_work))
        self.proc = subprocess.Popen([
            'google-chrome', '--headless', '--no-sandbox', '--disable-gpu',
            '--autoplay-policy=no-user-gesture-required', '--remote-debugging-port=0',
            '--remote-allow-origins=*', '--user-data-dir=' + str(profile), page.as_uri(),
        ], stdout=self.log, stderr=self.log, env=env)
        portfile = profile / 'DevToolsActivePort'
        deadline = time.monotonic() + 30
        while not portfile.exists():
            if self.proc.poll() is not None or time.monotonic() > deadline:
                self.log.flush()
                detail = (work / 'chrome.log').read_text(errors='replace')[-3000:]
                if self.proc.poll() is None:
                    self.proc.terminate()
                    self.proc.wait(timeout=15)
                self.log.close()
                self.temp_link.cleanup()
                raise RuntimeError('Chrome did not start: ' + detail)
            time.sleep(.05)
        self.sock = None
        try:
            port = int(portfile.read_text().splitlines()[0])
            pages = json.load(urllib.request.urlopen(f'http://127.0.0.1:{port}/json/list', timeout=10))
            endpoint = next(page['webSocketDebuggerUrl'] for page in pages if page['type'] == 'page')
            self.sock = socket.create_connection(('127.0.0.1', port), timeout=30)
            self.sock.settimeout(90)
            key = base64.b64encode(os.urandom(16)).decode()
            self.sock.sendall((
                f'GET /{endpoint.split("/", 3)[-1]} HTTP/1.1\r\n'
                f'Host: 127.0.0.1:{port}\r\nUpgrade: websocket\r\nConnection: Upgrade\r\n'
                f'Sec-WebSocket-Key: {key}\r\nSec-WebSocket-Version: 13\r\n\r\n'
            ).encode())
            response = b''
            while b'\r\n\r\n' not in response:
                response += self.sock.recv(4096)
            if b' 101 ' not in response:
                raise RuntimeError('WebSocket handshake failed')
            self.seq = 0
            for _ in range(400):
                try:
                    if self.evaluate('document.readyState === "complete" && Boolean(document.querySelector("#sprite-loop"))'):
                        break
                except RuntimeError:
                    pass
                time.sleep(.05)
            else:
                raise RuntimeError('Viewer did not finish loading')

        except BaseException:
            self.close()
            raise

    def exact(self, length):
        result = b''
        while len(result) < length:
            part = self.sock.recv(length - len(result))
            if not part:
                raise RuntimeError('DevTools connection closed')
            result += part
        return result

    def send(self, obj):
        payload = json.dumps(obj).encode()
        length = len(payload)
        mask = os.urandom(4)
        if length < 126:
            header = bytes([0x81, 0x80 | length])
        elif length < 65536:
            header = bytes([0x81, 0x80 | 126]) + struct.pack('!H', length)
        else:
            header = bytes([0x81, 0x80 | 127]) + struct.pack('!Q', length)
        self.sock.sendall(header + mask + bytes(byte ^ mask[i % 4] for i, byte in enumerate(payload)))

    def receive(self):
        first, second = self.exact(2)
        length = second & 127
        if length == 126:
            length = struct.unpack('!H', self.exact(2))[0]
        elif length == 127:
            length = struct.unpack('!Q', self.exact(8))[0]
        mask = self.exact(4) if second & 128 else None
        payload = self.exact(length)
        if mask:
            payload = bytes(byte ^ mask[i % 4] for i, byte in enumerate(payload))
        if first & 15 != 1:
            raise RuntimeError('Unexpected WebSocket frame')
        return json.loads(payload)

    def command(self, method, params=None):
        self.seq += 1
        ident = self.seq
        self.send({'id': ident, 'method': method, 'params': params or {}})
        while True:
            reply = self.receive()
            if reply.get('id') == ident:
                break
        if 'error' in reply:
            raise RuntimeError(str(reply))
        return reply['result']

    def evaluate(self, expression):
        result = self.command('Runtime.evaluate', {
            'expression': expression, 'awaitPromise': True, 'returnByValue': True,
        })
        if result.get('exceptionDetails'):
            raise RuntimeError(json.dumps(result['exceptionDetails'])[-2500:])
        return result.get('result', {}).get('value')

    def click(self, selector):
        box = self.evaluate('(()=>{const e=document.querySelector(' + json.dumps(selector) +
                            ');e.scrollIntoView({block:"center"});const r=e.getBoundingClientRect();'
                            'return {x:r.left+r.width/2,y:r.top+r.height/2}})()')
        self.command('Input.dispatchMouseEvent', {'type': 'mouseMoved', **box})
        self.command('Input.dispatchMouseEvent', {'type': 'mousePressed', **box, 'button': 'left', 'clickCount': 1})
        self.command('Input.dispatchMouseEvent', {'type': 'mouseReleased', **box, 'button': 'left', 'clickCount': 1})

    def close(self):
        if self.proc.poll() is None:
            if self.sock is None:
                self.proc.terminate()
            else:
                try:
                    self.send({'id': 999999, 'method': 'Browser.close'})
                except OSError:
                    pass
        if self.sock is not None:
            self.sock.close()
        try:
            self.proc.wait(timeout=15)
        except subprocess.TimeoutExpired:
            self.proc.terminate()
            try:
                self.proc.wait(timeout=15)
            except subprocess.TimeoutExpired:
                self.proc.kill()
                self.proc.wait()
        self.log.close()
        self.temp_link.cleanup()
