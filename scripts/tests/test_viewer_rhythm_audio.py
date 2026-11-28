"""Test bounded rhythm-tone lookup, pitch and audio playback paths."""
import copy
import hashlib
import json
import shutil
import struct
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import view_rom_data as viewer
from test_viewer_instruments_browser import fixture as instrument_fixture
from test_viewer_physical_audio import fixture_data
from test_viewer_scoped_browser import parse_page
from viewer_browser_harness import Chrome
from viewer_physical_audio import BANK_STARTS, ROM_BASE, project_rhythm_links


def tiny_catalog():
    def tone(identity, offset, ctype, key, target=0):
        return {'id': identity, 'source_offset': offset, 'ctype': ctype, 'key': key,
                'length': 0, 'pan_or_sweep': 0, 'word04': target, 'word08': 0}
    root = tone(0, BANK_STARTS[0], 0x80, 0, ROM_BASE + BANK_STARTS[0] + 12)
    root['physical_only'] = True
    a = tone(1, BANK_STARTS[0] + 12, 0, 36, 0x08002000)
    b = tone(2, BANK_STARTS[0] + 24, 0x40, 41)
    projected = [{key: value for key, value in item.items() if key in ('id', 'ctype', 'word04', 'word08')} for item in (a, b)]
    return {'instruments': {'tones': [root, *projected]}}, {'tones': [a, b]}


def generated(work, *, trusted=True, bad_key=False):
    audio, catalog, segments = fixture_data()
    for i, tone in enumerate(catalog['tones']):
        tone.update(key=(i * 3) % 128, length=0, pan_or_sweep=0)
    if bad_key: catalog['tones'][0]['key'] = True
    offset, role, raw = segments[0]; raw = bytearray(raw)
    struct.pack_into('<I', raw, 69 * 12 + 4, ROM_BASE + BANK_STARTS[0]); segments[0] = (offset, role, bytes(raw))
    dataset = work / 'dataset'; dataset.mkdir(); src = json.dumps(catalog).encode(); packed = b''.join(data for _, _, data in segments)
    (dataset/'tones.json').write_bytes(src); (dataset/'physical.bin').write_bytes(packed)
    manifest = 'profile="europe"\nextractor_revision=1\ndecoder_revision=1\n'
    for identity, path, fmt, raw in (('audio.m4a-tones', 'tones.json', 'audio-tones-v1-json', src),
                                      ('audio.physical-tone-banks', 'physical.bin', 'physical-tone-banks-v1-bin', packed)):
        manifest += f'[[resources]]\nid="{identity}"\npath="{path}"\nformat="{fmt}"\nsha256="{hashlib.sha256(raw).hexdigest()}"\n'
        if identity == 'audio.physical-tone-banks':
            for offset, role, data in segments:
                manifest += (f'[[resources.sources]]\noffset={offset}\nlength={len(data)}\nrole="{role}"\n'
                             f'sha256="{hashlib.sha256(data).hexdigest()}"\n')
    (dataset/'manifest.toml').write_text(manifest)
    with patch.object(viewer, 'EUR_TONES_SHA256', hashlib.sha256(src).hexdigest() if trusted else '0' * 64), \
         patch.object(viewer, 'EUR_PHYSICAL_SHA256', hashlib.sha256(packed).hexdigest()):
        return viewer.generate(dataset, work/'view', split=True)


def browser_fixture(work):
    page = instrument_fixture(work)
    data = json.JSONDecoder().raw_decode(page.read_text().split('const data = ', 1)[1])[0]
    instruments = data['audio']['instruments']
    instruments['tones'].append({'id': 8, 'ctype': 0x80, 'word04': 0x08006000, 'word08': 0})
    instruments['banks'][0]['programs'].append({'selector': 8, 'tone': 8})
    instruments['rhythm_links'] = [
        {'parent_tone': 8, 'key': 60, 'tone': 0, 'pitch_key': 24, 'pan_or_sweep': 0, 'supported': True},
        {'parent_tone': 8, 'key': 61, 'tone': 2, 'pitch_key': 45, 'pan_or_sweep': 0, 'supported': True},
        {'parent_tone': 8, 'key': 62, 'tone': 4, 'pitch_key': 7, 'pan_or_sweep': 0, 'supported': False}]
    data['pcm_pitch']['keys'] = [{'base': 0, 'shift': i % 8} for i in range(180)]
    data['sequence_directory']['songs'][0]['id'] = 5
    codes = [['parameter', 0xbd, 8], ['note', 1, 60, 100, None], ['wait', 1],
             ['note', 1, 61, 100, None], ['wait', 1], ['note', 1, 62, 100, None], ['end']]
    data['sequences']['instructions'] = [{'source_offset': i, 'next': i + 1 if i + 1 < len(codes) else None, 'code': code}
                                          for i, code in enumerate(codes)]
    data['timed_sound_sources'] = {'path': 'synthetic', 'groups': [{'kind': 'rows', 'origin': 'synthetic',
        'source_offset': 256, 'values': [{'tick': 0, 'channel': 0, 'opaque03': 0, 'sound_words': [5, 0]}]}]}
    template = (Path(__file__).resolve().parents[2] / 'view_rom_data.html').read_text()
    page.write_text(template.replace('/* DATA */', json.dumps(data))); return page


class RhythmProjectionTest(unittest.TestCase):
    def test_available_entries_and_source_guards(self):
        audio, catalog = tiny_catalog(); links = project_rhythm_links(audio, catalog)
        self.assertEqual([(item['key'], item['tone'], item['pitch_key'], item['supported']) for item in links],
                         [(0, 1, 36, True), (1, 2, 41, False)])
        for mutate in (lambda a, c: c['tones'][0].update(key=True),
                       lambda a, c: c['tones'][0].update(source_offset=0),
                       lambda a, c: c['tones'][0].update(word04=0),
                       lambda a, c: a['instruments']['tones'][0].update(word04=0)):
            bad_audio, bad_catalog = copy.deepcopy(audio), copy.deepcopy(catalog); mutate(bad_audio, bad_catalog)
            with self.assertRaises(ValueError): project_rhythm_links(bad_audio, bad_catalog)

    def test_hash_gate_partial_failure_and_scope(self):
        for options in ({}, {'trusted': False}, {'bad_key': True}):
            with self.subTest(options=options), tempfile.TemporaryDirectory(prefix='viewer-rhythm-') as directory:
                page = generated(Path(directory), **options); parts = page.parent/'index.html.parts'
                samples, world = parse_page(parts/'audio-samples.html'), parse_page(parts/'world.html')
                self.assertNotIn('audio', world)
                links = samples['audio']['instruments'].get('rhythm_links')
                if options: self.assertIsNone(links)
                else:
                    self.assertEqual(len(links), 128); self.assertTrue(all(link['supported'] for link in links))
                    self.assertEqual(links[1]['pitch_key'], 3)


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class RhythmBrowserTest(unittest.TestCase):
    def test_instrument_sequence_timed_and_wav_use_child_pitch(self):
        with tempfile.TemporaryDirectory(prefix='viewer-rhythm-browser-') as directory:
            work = Path(directory); browser = Chrome(work, browser_fixture(work))
            try:
                self.assertTrue(browser.evaluate('''(async()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change',{bubbles:true}));
                  const wait=async f=>{const until=Date.now()+5000;while(!f()){if(Date.now()>until)throw Error('timeout');await new Promise(r=>setTimeout(r,10))}};
                  window.live=[];window.offline=[];window.downloads=[];
                  class Param{constructor(v=0){this.value=v}setValueAtTime(v){this.value=v}linearRampToValueAtTime(v){this.value=v}}
                  class Node{connect(target){this.output=target}disconnect(){}}
                  class Base{constructor(list){this.currentTime=0;this.destination={};this.list=list}
                    createGain(){const n=new Node();n.gain=new Param(1);return n}
                    createStereoPanner(){const n=new Node();n.pan=new Param(0);return n}
                    createBuffer(c,n,r){const samples=new Float32Array(n);return {samples,rate:r,getChannelData(){return samples}}}
                    createBufferSource(){const n=new Node();n.kind='sample';n.playbackRate=new Param(1);n.start=t=>n.started=t;n.stop=t=>n.stopped=t;this.list.push(n);return n}
                    createOscillator(){const n=new Node();n.kind='sine';n.frequency=new Param(0);n.start=t=>n.started=t;n.stop=t=>n.stopped=t;this.list.push(n);return n}}
                  window.AudioContext=class extends Base{constructor(){super(live)}resume(){return Promise.resolve()}};
                  window.OfflineAudioContext=class extends Base{constructor(channels,frames,rate){super(offline);this.frames=frames;this.rate=rate}
                    startRendering(){return Promise.resolve({numberOfChannels:2,length:this.frames,sampleRate:this.rate,getChannelData:()=>new Float32Array(this.frames)})}};
                  URL.createObjectURL=()=> 'blob:synthetic';URL.revokeObjectURL=()=>{};HTMLAnchorElement.prototype.click=function(){downloads.push(this.download)};
                  q('#instruments>details').open=true;q('#instrument-program').value='8';change(q('#instrument-program'));
                  if(q('#instrument-keymap').hidden||q('#instrument-keymap-rows').rows.length!==3||q('#instrument-play').disabled||
                     !q('#instrument-source').textContent.includes('stored pitch key 24'))throw Error('rhythm PCM');
                  q('#instrument-play').click();await wait(()=>live.length===1);
                  const pcmRate=sourcePcmFrequency(data.pcm_pitch,22050,24)/22050;
                  if(Math.abs(live[0].playbackRate.value-pcmRate)>1e-12)throw Error('instrument PCM pitch');q('#instrument-stop').click();
                  q('#instrument-key').value='61';change(q('#instrument-key'));
                  if(!q('#instrument-source').textContent.includes('stored pitch key 45'))throw Error('rhythm CGB');
                  q('#instrument-mute-cgb').checked=true;change(q('#instrument-mute-cgb'));q('#instrument-play').click();await wait(()=>live.length===2);
                  const cgbRate=440*Math.pow(2,(45-69)/12)*256/32000;
                  if(Math.abs(live[1].playbackRate.value-cgbRate)>1e-12||live[1].output.output.gain.value!==0)throw Error('CGB pitch/mute');
                  q('#instrument-stop').click();q('#instrument-key').value='62';change(q('#instrument-key'));
                  if(!q('#instrument-play').disabled||!q('#instrument-keymap-rows').rows[2].textContent.includes('nested'))throw Error('nested guard');
                  q('#instrument-mute-cgb').checked=false;change(q('#instrument-mute-cgb'));
                  q('#sequence-audio>details').open=true;q('#sequence-use-tempo').checked=false;change(q('#sequence-use-tempo'));
                  q('#sequence-envelope').checked=false;change(q('#sequence-envelope'));
                  const before=live.length;q('#sequence-play').click();await wait(()=>live.length>=before+3);
                  if(live[before].kind!=='sample'||Math.abs(live[before].playbackRate.value-pcmRate)>1e-12||
                     live[before+1].kind!=='sample'||Math.abs(live[before+1].playbackRate.value-cgbRate)>1e-12||
                     live[before+2].kind!=='sine')throw Error('sequence mapping');q('#sequence-stop').click();
                  q('#sequence-download').click();await wait(()=>downloads.length>0);
                  if(offline.length!==3||Math.abs(offline[0].playbackRate.value-pcmRate)>1e-12||
                     Math.abs(offline[1].playbackRate.value-cgbRate)>1e-12||offline[2].kind!=='sine')throw Error('WAV mapping');
                  const timedBefore=live.length;q('#timed-sound-play').click();await wait(()=>live.length>=timedBefore+3);
                  if(Math.abs(live[timedBefore].playbackRate.value-pcmRate)>1e-12||Math.abs(live[timedBefore+1].playbackRate.value-cgbRate)>1e-12)
                    throw Error('timed mapping');q('#timed-sound-stop').click();return true;
                })()'''))
            finally:
                browser.close()


if __name__ == '__main__':
    unittest.main()
