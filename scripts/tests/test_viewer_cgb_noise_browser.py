"""Check source noise-code projection and the synthesized CGB reference."""
import copy
import hashlib
import json
import shutil
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from view_rom_data import generate, project_audio_noise
from test_viewer_cgb_pitch_browser import pitch_catalog
from test_viewer_instruments_browser import fixture as instruments_fixture
from viewer_browser_harness import Chrome


def fixture(work, catalog):
    dataset = work / 'dataset'; dataset.mkdir()
    content = json.dumps(catalog).encode(); (dataset / 'driver.json').write_bytes(content)
    digest = hashlib.sha256(content).hexdigest()
    (dataset / 'manifest.toml').write_text(
        'profile="europe"\nextractor_revision=1\ndecoder_revision=1\n'
        '[[resources]]\nid="audio.m4a-driver-tables"\npath="driver.json"\n'
        f'format="audio-driver-tables-v1-json"\nsha256="{digest}"\n')
    return generate(dataset, work / 'view', split=True)


def payload(page):
    return json.JSONDecoder().raw_decode(page.read_text().split('const data = ', 1)[1])[0]


def reference(register, count):
    # Traverse precomputed LFSR runs across each exact output-sample interval.
    divisor = 16 * (register & 7) if register & 7 else 8
    period = divisor * 4 * (1 << (register >> 4)) * 32000
    state, runs = 0x7fff, []
    for _ in range((count * 16777216 // period) + 2):
        runs.append(-1 if state & 1 else 1)
        bit = (state ^ (state >> 1)) & 1
        state = (state >> 1) | (bit << 14)
        if register & 8:
            state = (state & ~0x40) | (bit << 6)
    samples = []
    for index in range(count):
        start, end, total = index * 16777216, (index + 1) * 16777216, 0
        for run in range(start // period, (end - 1) // period + 1):
            total += runs[run] * (min(end, (run + 1) * period) - max(start, run * period))
        samples.append(total / 16777216)
    return samples


class NoiseProjectionTest(unittest.TestCase):
    def test_projection_bounds_legacy_and_independent_failures(self):
        table = {'source_offset': 0x1234, 'entries': list(range(60))}
        catalog = pitch_catalog(); catalog['noise_codes'] = table
        self.assertEqual(project_audio_noise(catalog), table)
        self.assertIsNone(project_audio_noise({}))
        with self.assertRaises(ValueError): project_audio_noise({'noise_codes': None})
        mutations = [lambda c: c['entries'].pop(), lambda c: c['entries'].__setitem__(0, True),
                     lambda c: c['entries'].__setitem__(2, 256), lambda c: c.__setitem__('source_offset', -1),
                     lambda c: c.__setitem__('source_offset', 0x1000000 - 59)]
        for mutate in mutations:
            bad = copy.deepcopy(table); mutate(bad)
            with self.subTest(bad=bad), self.assertRaises(ValueError):
                project_audio_noise({'noise_codes': bad})
        for mode in ('both', 'noise-only', 'bad-noise', 'legacy'):
            with self.subTest(mode=mode), tempfile.TemporaryDirectory(prefix='viewer-noise-projection-') as directory:
                value = copy.deepcopy(catalog)
                if mode == 'noise-only': del value['pcm_keys']
                if mode == 'bad-noise': value['noise_codes']['entries'].pop()
                if mode == 'legacy': del value['noise_codes']
                index = fixture(Path(directory), value); parts = index.parent / 'index.html.parts'
                audio, home = payload(parts / 'audio.html'), payload(index)
                self.assertNotIn('audio_noise', home)
                row = audio['resources'][0]
                if mode in ('both', 'noise-only'):
                    self.assertEqual(audio['audio_noise'], {**table, 'path': 'driver.json'})
                    self.assertIn('CGB noise codes', row['status'])
                else:
                    self.assertIsNone(audio['audio_noise'])
                self.assertEqual(audio['pcm_pitch'] is not None, mode != 'noise-only')
                if mode == 'noise-only': self.assertIn('invalid PCM pitch', row['status'])
                if mode == 'bad-noise': self.assertIn('invalid CGB noise', row['status'])


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class NoiseBrowserTest(unittest.TestCase):
    def test_waveform_reference_controls_mute_and_wav(self):
        with tempfile.TemporaryDirectory(prefix='viewer-noise-browser-') as directory:
            work = Path(directory); page = instruments_fixture(work)
            text = page.read_text(); begin = text.index('const data = ') + len('const data = ')
            data, length = json.JSONDecoder().raw_decode(text[begin:]); codes = [0, 8, 0x44, 0xd7] + [0x44] * 56
            data['audio_noise'] = {'path': 'driver.json', 'source_offset': 0x1234, 'entries': codes}
            data['resources'].append({'id': 'audio.m4a-driver-tables', 'path': 'driver.json', 'format': 'synthetic',
                                      'description': 'Driver', 'status': 'Available: CGB noise codes'})
            page.write_text(text[:begin] + json.dumps(data) + text[begin + length:])
            browser = Chrome(work, page)
            try:
                for code in (0, 8, 0x44, 0xd7):
                    result = browser.evaluate('Array.from(cgbNoiseSamples(' + str(code) + ', 128))')
                    expected = reference(code, 128)
                    for actual, want in zip(result, expected): self.assertAlmostEqual(actual, want, places=6)
                self.assertEqual(browser.evaluate('''(async()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change'));
                  const wait=async fn=>{const end=Date.now()+3000;while(!fn()){if(Date.now()>end)throw Error('timeout');await new Promise(r=>setTimeout(r,10))}};
                  window.noiseSources=[];window.noiseContexts=[];
                  class Param {constructor(v=0){this.value=v} setValueAtTime(v){this.value=v}}
                  class Node {connect(target){this.output=target} disconnect(){this.disconnected=true}}
                  class FakeContext {
                    constructor(){this.currentTime=0;this.destination={};noiseContexts.push(this)} resume(){return Promise.resolve()}
                    createBuffer(c,n,r){const samples=new Float32Array(n);return {samples,rate:r,getChannelData(){return samples}}}
                    createGain(){const n=new Node();n.gain=new Param(1);return n}
                    createBufferSource(){const n=new Node();n.start=()=>n.started=true;n.stop=()=>n.stopped=true;noiseSources.push(n);return n}
                  }
                  window.AudioContext=FakeContext;
                  if(q('#cgb-noise').hidden || q('#cgb-noise>details').open || q('#noise-code').options.length!==60 ||
                     !q('#noise-detail').textContent.includes('keys 60') ||
                     ![...document.querySelectorAll('#rows a')].some(a=>a.getAttribute('href')==='#cgb-noise'))throw Error('initial directory');
                  q('#cgb-noise>details').open=true;
                  q('#noise-loop').checked=true;change(q('#noise-loop'));
                  q('#noise-code').value='2';change(q('#noise-code'));
                  if(!q('#noise-loop').checked || !q('#noise-detail').textContent.includes('0x44') ||
                     !q('#noise-detail').textContent.includes('keys 23'))throw Error('code and repeat');
                  q('#sequence-mute-cgb').checked=true;change(q('#sequence-mute-cgb'));
                  if(!q('#noise-mute-cgb').checked)throw Error('shared mute');
                  q('#noise-play').click();await wait(()=>noiseSources.length===1);
                  let n=noiseSources[0];
                  if(!n.loop || n.buffer.rate!==32000 || n.buffer.samples.length!==32000 || n.output.gain.value!==0 ||
                     q('#noise-stop').disabled || !q('#noise-status').textContent.includes('muted'))throw Error('muted playback');
                  q('#noise-mute-cgb').checked=false;change(q('#noise-mute-cgb'));
                  if(q('#sequence-mute-cgb').checked || q('#instrument-mute-cgb').checked || n.output.gain.value!==.08)throw Error('live unmute');
                  noiseContexts[0].currentTime=.4;await new Promise(r=>setTimeout(r,130));
                  if(Number(q('#noise-progress').value)<.35)throw Error('playback progress');
                  q('#noise-code').value='1';change(q('#noise-code'));
                  if(!n.stopped || !q('#noise-stop').disabled || !q('#noise-loop').checked ||
                     !q('#noise-detail').textContent.includes('7-bit'))throw Error('stop or selected setting');
                  let savedBlob=null;URL.createObjectURL=blob=>{savedBlob=blob;return 'blob:test'};
                  const original=HTMLAnchorElement.prototype.click;HTMLAnchorElement.prototype.click=function(){};
                  q('#noise-download').click();HTMLAnchorElement.prototype.click=original;
                  if(!savedBlob || !q('#noise-status').textContent.includes('WAV'))throw Error('download');
                  const bytes=new Uint8Array(await savedBlob.arrayBuffer()),view=new DataView(bytes.buffer);
                  if(String.fromCharCode(...bytes.slice(0,4))!=='RIFF' || view.getUint32(24,true)!==32000 ||
                     view.getUint32(40,true)!==32000 || bytes.length!==32044)throw Error('WAV header');
                  q('#noise-play').click();await wait(()=>noiseSources.length===2);n=noiseSources[1];
                  q('#cgb-noise>details').open=false;await wait(()=>n.stopped);
                  if(!q('#noise-stop').disabled)throw Error('closing section');
                  return 'PASS';
                })()'''), 'PASS')
                browser.evaluate("document.querySelector('#cgb-noise').closest('.section-group').open=true;document.querySelector('#cgb-noise>details').open=true;document.querySelector('#noise-code').focus()")
                browser.click('#noise-play')
                self.assertEqual(browser.evaluate('document.activeElement.id'), 'noise-code')
            finally:
                browser.close()

    def test_absent_noise_is_hidden(self):
        with tempfile.TemporaryDirectory(prefix='viewer-noise-legacy-') as directory:
            work = Path(directory); browser = Chrome(work, instruments_fixture(work))
            try:
                self.assertTrue(browser.evaluate("document.querySelector('#cgb-noise').hidden"))
            finally:
                browser.close()


if __name__ == '__main__':
    unittest.main()
