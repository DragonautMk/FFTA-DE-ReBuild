"""Verify offline, per-wave PCM loading and unchanged playback controls."""
import hashlib
import json
import shutil
import tempfile
import time
import unittest
from pathlib import Path

from test_viewer_scoped_browser import REPO, parse_page, prepared
from viewer_browser_harness import Chrome
from view_rom_data import write_split_viewer


def fixture(work):
    data, index = prepared(work)
    waves = []
    tones = []
    for i, samples in enumerate(([0, 64, -64, 0], [10, 20, 30, 40], [-90, 90, -90, 90])):
        waves.append({'id': i, 'source_offset': 0x2000 + 16 * i, 'tone_ids': [i], 'frequency_word': 22050,
                      'declared_length': len(samples), 'loop_start': 0, 'flags': 0x4000, 'samples': list(samples)})
        tones.append({'id': i, 'ctype': 0, 'word04': 0x08002000 + 16 * i, 'word08': 0xffffffff})
    data['audio'] = {'path': 'audio.json', 'waves': waves, 'cgb_waves': [], 'instruments': {
        'banks': [{'source_address': 0x08001000, 'programs': [{'selector': i, 'tone': i} for i in range(3)]}],
        'tones': tones, 'map_links': []}}
    data['sequence_directory'] = {'songs': [{'id': 5, 'header_id': 0}, {'id': 6, 'header_id': 0}],
                                  'headers': [{'id': 0, 'tone_bank_address': 0x08001000}], 'path': 'directory.json'}
    instructions, tracks = [], []
    for i in range(3):
        entry = len(instructions)
        codes = [['parameter', 0xbd, i], ['note', 1, 60, 100, None], ['wait', 1], ['end']]
        instructions.extend({'source_offset': entry + n, 'next': None if code[0] == 'end' else entry + n + 1,
                             'code': code} for n, code in enumerate(codes))
        tracks.append({'directory_header': 0, 'track_index': i, 'entry': entry})
    data['sequences'] = {'path': 'sequence.json', 'clocks': [0, 4] + [1] * 47,
                         'tracks': tracks, 'instructions': instructions}
    data['timed_sound_sources'] = {'path': 'timed.json', 'groups': [{
        'kind': 'rows', 'origin': 'fixture', 'source_offset': 256,
        'values': [{'tick': 0, 'channel': 0, 'opaque03': 0, 'sound_words': [5, 0]}]}]}
    write_split_viewer(data, index.parent, index.name, (REPO / 'view_rom_data.html').read_text())
    return index, data


def wait_for(browser, expression, timeout=10):
    end = time.monotonic() + timeout
    while time.monotonic() < end:
        try:
            if browser.evaluate(expression):
                return
        except (RuntimeError, OSError):
            pass
        time.sleep(.03)
    raise AssertionError('Timed out waiting for: ' + expression)


class LazyPcmAssetTest(unittest.TestCase):
    def test_assets_preserve_samples_and_reject_changed_output(self):
        with tempfile.TemporaryDirectory(prefix='viewer-lazy-pcm-assets-') as directory:
            work = Path(directory)
            index, data = fixture(work)
            parts = index.parent / (index.name + '.parts')
            song = parse_page(parts / 'audio.html')
            instruments = parse_page(parts / 'audio-samples.html')
            self.assertEqual(song['audio'], instruments['audio'])
            for original, stored in zip(data['audio']['waves'], song['audio']['waves']):
                self.assertNotIn('samples', stored)
                encoded = json.dumps(original['samples'], separators=(',', ':'))
                self.assertEqual(stored['samples_asset']['key'], hashlib.sha256(encoded.encode()).hexdigest())
                asset = parts / stored['samples_asset']['href']
                content = json.JSONDecoder().raw_decode(asset.read_text().split(' = ', 2)[-1])[0]
                self.assertEqual(content['samples'], original['samples'])
                self.assertEqual(stored['sample_count'], len(original['samples']))
            asset = parts / song['audio']['waves'][0]['samples_asset']['href']
            previous = (parts / 'audio.html').read_bytes()
            asset.write_text('changed by another owner')
            with self.assertRaisesRegex(ValueError, 'PCM wave asset'):
                write_split_viewer(data, index.parent, index.name, (REPO / 'view_rom_data.html').read_text())
            self.assertEqual((parts / 'audio.html').read_bytes(), previous)
            self.assertEqual(asset.read_text(), 'changed by another owner')


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class LazyPcmBrowserTest(unittest.TestCase):
    def test_samples_load_individually_and_retry_offline(self):
        with tempfile.TemporaryDirectory(prefix='viewer-lazy-pcm-samples-') as directory:
            work = Path(directory)
            index, _ = fixture(work)
            parts = index.parent / (index.name + '.parts')
            waves = parse_page(parts / 'audio-samples.html')['audio']['waves']
            asset = parts / waves[0]['samples_asset']['href']
            saved = asset.with_suffix('.saved')
            browser = Chrome(work, parts / 'audio-samples.html')
            try:
                self.assertTrue(browser.evaluate('''(()=>{
                  return typeof globalThis.__FFTA_PCM_WAVES__==='undefined' &&
                    data.audio.waves.every(w=>!('samples' in w)) && document.querySelector('#audio-id').options.length===3;
                })()'''))
                browser.evaluate("document.querySelector('#audio>details').open=true")
                wait_for(browser, "Array.isArray(data.audio.waves[0].samples)")
                self.assertTrue(browser.evaluate('''(()=>{const q=s=>document.querySelector(s);
                  return Object.keys(globalThis.__FFTA_PCM_WAVES__).length===1 &&
                    q('#audio-sample-rows').rows.length===4 && q('#audio-sample-rows').rows[1].cells[1].textContent==='64' &&
                    !('samples' in data.audio.waves[1]);})()'''))
                browser.evaluate("document.querySelector('#audio-id').value='1';document.querySelector('#audio-id').dispatchEvent(new Event('change'))")
                wait_for(browser, "Array.isArray(data.audio.waves[1].samples)")
                self.assertTrue(browser.evaluate("Object.keys(globalThis.__FFTA_PCM_WAVES__).length===2 && !('samples' in data.audio.waves[2])"))
                asset.rename(saved)
                browser.command('Page.navigate', {'url': (parts / 'audio-samples.html').as_uri()})
                wait_for(browser, "document.readyState==='complete' && typeof data!=='undefined' && !('samples' in data.audio.waves[0])")
                browser.evaluate("document.querySelector('#audio>details').open=true")
                wait_for(browser, "document.querySelector('#audio-status').textContent.startsWith('Cannot load samples:')")
                self.assertTrue(browser.evaluate("!document.querySelector('#audio-load-samples').hidden && !('samples' in data.audio.waves[0])"))
                saved.rename(asset)
                browser.evaluate("document.querySelector('#audio-load-samples').click()")
                wait_for(browser, "Array.isArray(data.audio.waves[0].samples)")
                self.assertTrue(browser.evaluate("document.querySelector('#audio-sample-rows').rows[1].cells[1].textContent==='64'"))
            finally:
                if saved.exists():
                    saved.rename(asset)
                browser.close()

    def test_song_cancel_selected_waves_and_actual_wav(self):
        with tempfile.TemporaryDirectory(prefix='viewer-lazy-pcm-song-') as directory:
            work = Path(directory)
            index, _ = fixture(work)
            parts = index.parent / (index.name + '.parts')
            browser = Chrome(work, parts / 'audio.html')
            try:
                self.assertTrue(browser.evaluate('''(async()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change'));
                  const wait=async f=>{const end=Date.now()+5000;while(!f()){
                    if(Date.now()>end)throw Error('timeout');await new Promise(r=>setTimeout(r,10))}};
                  if(typeof globalThis.__FFTA_PCM_WAVES__!=='undefined'||data.audio.waves.some(w=>'samples' in w))
                    throw Error('initial eager samples');
                  window.sources=[];
                  class Param{constructor(v=0){this.value=v}setValueAtTime(v){this.value=v}linearRampToValueAtTime(v){this.value=v}}
                  class Node{connect(target){this.output=target}disconnect(){}}
                  window.AudioContext=class{constructor(){this.currentTime=0;this.destination={}}resume(){return Promise.resolve()}
                    createGain(){const n=new Node();n.gain=new Param(1);return n}
                    createBuffer(c,n,r){const values=new Float32Array(n);return {values,getChannelData(){return values}}}
                    createBufferSource(){const n=new Node();n.playbackRate=new Param(1);n.start=t=>n.started=t;n.stop=t=>n.stopped=t;sources.push(n);return n}};
                  const append=document.head.append.bind(document.head);window.queued=[];
                  document.head.append=function(node){if(node.src&&node.src.includes('audio-pcm-'))queued.push(node);else append(node)};
                  window.release=()=>queued.splice(0).forEach(node=>append(node));
                  q('#sequence-track').value='0';change(q('#sequence-track'));
                  q('#sequence-use-tempo').checked=false;change(q('#sequence-use-tempo'));
                  q('#sequence-play').click();await wait(()=>queued.length===1);
                  if(q('#sequence-stop').disabled)throw Error('cannot cancel loading');
                  q('#sequence-stop').click();release();await wait(()=>Array.isArray(data.audio.waves[0].samples));
                  await new Promise(r=>setTimeout(r,50));if(sources.length)throw Error('cancelled playback started');
                  q('#sequence-play').click();await wait(()=>sources.length===1);
                  if(sources[0].buffer.values[1]!==.5 || 'samples' in data.audio.waves[1] || queued.length)
                    throw Error('selected wave playback');
                  q('#sequence-stop').click();q('#sequence-track').value='1';change(q('#sequence-track'));
                  q('#sequence-play').click();await wait(()=>queued.length===1);release();await wait(()=>sources.length===2);
                  if(!('samples' in data.audio.waves[1])||'samples' in data.audio.waves[2])throw Error('second selected wave');
                  q('#sequence-stop').click();window.blobs=[];const original=URL.createObjectURL.bind(URL);
                  URL.createObjectURL=b=>{blobs.push(b);return original(b)};HTMLAnchorElement.prototype.click=function(){};
                  q('#sequence-track').value='2';change(q('#sequence-track'));q('#sequence-download').click();
                  await wait(()=>queued.length===1);release();await wait(()=>blobs.length===1);
                  const wav=new DataView(await blobs[0].arrayBuffer());
                  if(wav.getUint32(24,true)!==32000 || wav.getUint32(40,true)===0 || !('samples' in data.audio.waves[2]))
                    throw Error('WAV loaded selected wave');
                  return true;
                })()'''))
            finally:
                browser.close()

    def test_instrument_preview_and_timed_sounds_load_their_waves(self):
        with tempfile.TemporaryDirectory(prefix='viewer-lazy-pcm-instrument-') as directory:
            work = Path(directory)
            index, _ = fixture(work)
            parts = index.parent / (index.name + '.parts')
            browser = Chrome(work, parts / 'audio-samples.html')
            try:
                browser.evaluate("document.querySelector('#instruments>details').open=true")
                wait_for(browser, "Array.isArray(data.audio.waves[0].samples)")
                self.assertTrue(browser.evaluate("Object.keys(globalThis.__FFTA_PCM_WAVES__).length===1 && !('samples' in data.audio.waves[1])"))
                browser.evaluate('''(()=>{const q=s=>document.querySelector(s);
                  q('#instrument-program').value='1';q('#instrument-program').dispatchEvent(new Event('change'))})()''')
                wait_for(browser, "Array.isArray(data.audio.waves[1].samples)")
                self.assertTrue(browser.evaluate('''(()=>{const q=s=>document.querySelector(s);
                  return q('#instrument-status').textContent.includes('PCM sample 1') &&
                    !q('#instrument-play').disabled && !('samples' in data.audio.waves[2]);})()'''))
                browser.command('Page.navigate', {'url': (parts / 'audio.html').as_uri()})
                wait_for(browser, "document.readyState==='complete' && typeof data!=='undefined' && data.wiki.current==='audio'")
                self.assertTrue(browser.evaluate('''(async()=>{
                  const q=s=>document.querySelector(s),wait=async f=>{const end=Date.now()+5000;while(!f()){
                    if(Date.now()>end)throw Error('timeout');await new Promise(r=>setTimeout(r,10))}};
                  window.sources=[];
                  class Param{constructor(v=0){this.value=v}setValueAtTime(v){this.value=v}linearRampToValueAtTime(v){this.value=v}}
                  class Node{connect(target){this.output=target}disconnect(){}}
                  window.AudioContext=class{constructor(){this.currentTime=0;this.destination={}}resume(){return Promise.resolve()}
                    createGain(){const n=new Node();n.gain=new Param(1);return n}
                    createBuffer(c,n,r){const samples=new Float32Array(n);return {samples,getChannelData(){return samples}}}
                    createBufferSource(){const n=new Node();n.playbackRate=new Param(1);n.start=t=>n.started=t;n.stop=t=>n.stopped=t;sources.push(n);return n}};
                  if(typeof globalThis.__FFTA_PCM_WAVES__!=='undefined')throw Error('audio page loaded samples eagerly');
                  q('#timed-sound-play').click();await wait(()=>sources.length===3);
                  if(Object.keys(globalThis.__FFTA_PCM_WAVES__).length!==3 || data.audio.waves.some(w=>!Array.isArray(w.samples)))
                    throw Error('timed playback waves');
                  if(sources[0].buffer.samples[1]!==.5)throw Error('timed sample values');
                  q('#timed-sound-stop').click();return true;
                })()'''))
            finally:
                browser.close()
