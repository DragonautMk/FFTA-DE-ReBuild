"""Check source-table CGB pitch and older dataset fallback."""
import copy
import json
import shutil
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from view_rom_data import project_audio_pcm_pitch
from test_viewer_instruments_browser import fixture
from viewer_browser_harness import Chrome


def pitch_catalog():
    return {'pcm_keys': {'entries': [{'base': 0, 'shift': 0}] * 180},
            'pcm_bases': {'entries': [0x80000000] * 12},
            'cgb_keys': {'entries': [{'base': index % 12, 'shift': (index // 12) % 8} for index in range(132)]},
            'cgb_bases': {'entries': [-2048 + index * 87 for index in range(12)]}}


def reference_register(catalog, key, fraction):
    key &= 255; fraction &= 255
    if key <= 35: key, fraction = 0, 0
    else:
        key -= 36
        if key > 130: key, fraction = 130, 255
    keys = catalog['cgb_keys']['entries']; bases = catalog['cgb_bases']['entries']
    a = bases[keys[key]['base']] >> keys[key]['shift']
    b = bases[keys[key + 1]['base']] >> keys[key + 1]['shift']
    return (a + ((fraction * (b - a)) >> 8) + 0x800) & 0xffffffff


class CgbPitchProjectionTest(unittest.TestCase):
    def test_cgb_tables_validation_and_legacy_fallback(self):
        catalog = pitch_catalog(); projected = project_audio_pcm_pitch(catalog)
        self.assertEqual(projected['cgb_keys'], catalog['cgb_keys']['entries'])
        self.assertEqual(projected['cgb_bases'], catalog['cgb_bases']['entries'])
        legacy = copy.deepcopy(catalog); del legacy['cgb_keys']; del legacy['cgb_bases']
        self.assertNotIn('cgb_keys', project_audio_pcm_pitch(legacy))
        mutations = [lambda c: c.pop('cgb_keys'), lambda c: c['cgb_keys']['entries'].pop(),
                     lambda c: c['cgb_keys']['entries'][0].__setitem__('base', 12),
                     lambda c: c['cgb_keys']['entries'][0].__setitem__('shift', True),
                     lambda c: c['cgb_bases']['entries'].__setitem__(0, -32769)]
        for mutate in mutations:
            bad = copy.deepcopy(catalog); mutate(bad)
            with self.subTest(mutation=mutate), self.assertRaises(ValueError):
                project_audio_pcm_pitch(bad)


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class CgbPitchBrowserTest(unittest.TestCase):
    def test_register_arithmetic_audition_wave_and_sequence(self):
        with tempfile.TemporaryDirectory(prefix='viewer-cgb-pitch-') as temp:
            work = Path(temp); page = fixture(work); text = page.read_text()
            prefix = 'const data = '; start = text.index(prefix) + len(prefix)
            data, length = json.JSONDecoder().raw_decode(text[start:]); catalog = pitch_catalog()
            data['pcm_pitch'] = project_audio_pcm_pitch(catalog)
            codes = [['parameter', 0xbd, 2], ['note', 1, 60, 100, None],
                     ['parameter', 0xbd, 3], ['note', 1, 60, 100, None], ['wait', 1], ['end']]
            data['sequences']['instructions'] = [{'source_offset': index,
                'next': index + 1 if index + 1 < len(codes) else None, 'code': code} for index, code in enumerate(codes)]
            page.write_text(text[:start] + json.dumps(data) + text[start + length:])
            expected = [reference_register(catalog, key, fraction) for key in range(256) for fraction in (0, 1, 127, 255)]
            browser = Chrome(work, page)
            try:
                self.assertEqual(browser.evaluate('''(async()=>{
                  const expected=EXPECTED, q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change'));
                  for(let key=0;key<256;key++)for(let j=0;j<4;j++){
                    const fraction=[0,1,127,255][j],word=sourceCgbRegister(data.pcm_pitch,key,fraction);
                    if(word!==expected[key*4+j])throw Error('register '+key+'/'+fraction);
                    const square=sourceCgbPitch(data.pcm_pitch,1,key,fraction),wave=sourceCgbPitch(data.pcm_pitch,3,key,fraction);
                    if(square.register!==(word&2047)||square.frequency!==131072/(2048-(word&2047))||
                       wave.frequency!==square.frequency/2)throw Error('hardware period');
                  }
                  if(sourceCgbRegister(null,60)!==null||sourceCgbPitch(data.pcm_pitch,4,60)!==null)throw Error('fallback');
                  window.sources=[];window.gains=[];
                  class Param {constructor(v=0){this.value=v} setValueAtTime(v){this.value=v} linearRampToValueAtTime(v){this.value=v}}
                  class Node {connect(target){this.output=target} disconnect(){}}
                  class FakeContext {
                    constructor(){this.currentTime=0;this.destination={}} resume(){return Promise.resolve()}
                    createBuffer(c,n,r){return {rate:r,getChannelData(){return new Float32Array(n)}}}
                    createGain(){const n=new Node();n.gain=new Param(1);gains.push(n);return n}
                    createBufferSource(){const n=new Node();n.playbackRate=new Param(1);n.start=()=>{};n.stop=()=>{};sources.push(n);return n}
                    createOscillator(){throw Error('unexpected sine fallback')}
                  }
                  window.AudioContext=FakeContext;
                  const wait=async count=>{const end=Date.now()+3000;while(sources.length<count){if(Date.now()>end)throw Error('timeout');await new Promise(r=>setTimeout(r,10))}};
                  q('#instrument-program').value='2';change(q('#instrument-program'));
                  const square=sourceCgbPitch(data.pcm_pitch,1,60);
                  if(!q('#instrument-status').textContent.includes('Driver-table pitch'))throw Error('pitch label');
                  q('#instrument-play').click();await wait(1);
                  if(Math.abs(sources[0].playbackRate.value-square.frequency*256/32000)>1e-12)throw Error('square playback');
                  q('#instrument-mute-cgb').checked=true;change(q('#instrument-mute-cgb'));
                  if(sources[0].output.output.gain.value!==0)throw Error('mute');q('#instrument-stop').click();
                  q('#instrument-mute-cgb').checked=false;change(q('#instrument-mute-cgb'));
                  q('#instrument-program').value='3';change(q('#instrument-program'));q('#instrument-play').click();await wait(2);
                  if(Math.abs(sources[1].playbackRate.value-square.frequency/2*32/32000)>1e-12)throw Error('wave playback');
                  q('#instrument-stop').click();q('#sequence-use-tempo').checked=false;change(q('#sequence-use-tempo'));
                  if(!q('#sequence-note-rows').textContent.includes('base register'))throw Error('sequence label');
                  q('#sequence-play').click();await wait(4);
                  if(Math.abs(sources[2].playbackRate.value-square.frequency*256/32000)>1e-12||
                     Math.abs(sources[3].playbackRate.value-square.frequency/2*32/32000)>1e-12)throw Error('sequence pitch');
                  q('#sequence-stop').click();return 'PASS';
                })()'''.replace('EXPECTED', json.dumps(expected))), 'PASS')
            finally: browser.close()
