"""Check isolated PCM/CGB instrument audition and bank/key-map navigation."""
import json
import shutil
import tempfile
import unittest
from pathlib import Path

from viewer_browser_harness import Chrome


def fixture(work):
    repo = Path(__file__).resolve().parents[2]
    tones = [{'id': 0, 'ctype': 0, 'word04': 0x08002000, 'word08': 0x80408080},
             {'id': 1, 'ctype': 8, 'word04': 0x08002000},
             {'id': 2, 'ctype': 1, 'word04': 2},
             {'id': 3, 'ctype': 3, 'word04': 0x08003000},
             {'id': 4, 'ctype': 0x40, 'word04': 0},
             {'id': 5, 'ctype': 4, 'word04': 0},
             {'id': 6, 'ctype': 0, 'word04': 0x08004000},
             {'id': 7, 'ctype': 8, 'word04': 0x08005000}]
    waves = [{'id': 10, 'source_offset': 0x2000, 'tone_ids': [0, 1], 'frequency_word': 22050,
              'loop_start': 1, 'flags': 0x4000, 'samples': [0, 32, 0, -32]},
             {'id': 11, 'source_offset': 0x5000, 'tone_ids': [7], 'frequency_word': 22050,
              'loop_start': 0, 'flags': 0, 'samples': [0] * 2205}]
    cgb = [{'id': 20, 'source_offset': 0x3000, 'tone_ids': [3], 'amplitudes': [0, 15] * 16}]
    data = {'profile': 'synthetic', 'extractor': 1, 'decoder': 1,
            'resources': [{'id': 'audio.m4a-tones', 'path': 'audio', 'format': 'json',
                           'description': 'Synthetic audio', 'status': 'Available'}],
            'audio': {'path': 'audio', 'waves': waves, 'cgb_waves': cgb,
                      'instruments': {'tones': tones,
                          'banks': [{'source_address': 0x08001000,
                                     'programs': [{'selector': i, 'tone': i} for i in range(8)]},
                                    {'source_address': 0x08001100, 'programs': []}],
                          'map_links': [{'parent_tone': 4, 'key': 60, 'tone': 2},
                                        {'parent_tone': 4, 'key': 61, 'tone': 0}]}},
            'pcm_pitch': {'keys': [{'base': 0, 'shift': 0}] * 180, 'bases': [0x80000000] * 12},
            'sequence_directory': {'songs': [{'id': 0, 'header_id': 0}],
                                   'headers': [{'id': 0, 'tone_bank_address': 0x08001000}]},
            'sequences': {'path': 'sequence', 'clocks': [0, 4] + [1] * 47,
                          'tracks': [{'directory_header': 0, 'track_index': 0, 'entry': 0}],
                          'instructions': [{'source_offset': 0, 'next': 1, 'code': ['parameter', 0xbd, 0]},
                                           {'source_offset': 1, 'next': 2, 'code': ['note', 1, 60, 100, None]},
                                           {'source_offset': 2, 'next': 3, 'code': ['wait', 1]},
                                           {'source_offset': 3, 'next': None, 'code': ['end']}]}}
    page = work / 'instruments.html'
    page.write_text((repo / 'view_rom_data.html').read_text().replace('/* DATA */', json.dumps(data)))
    return page


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class ViewerInstrumentsTest(unittest.TestCase):
    def test_pcm_cgb_keymaps_mute_and_controls(self):
        with tempfile.TemporaryDirectory(prefix='viewer-instruments-') as directory:
            work = Path(directory)
            browser = Chrome(work, fixture(work))
            try:
                self.assertEqual(browser.evaluate('''(async()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change'));
                  const wait=async f=>{const end=Date.now()+3000;while(!f()){if(Date.now()>end)throw Error('timeout');await new Promise(r=>setTimeout(r,10))}};
                  window.instrumentSources=[];window.instrumentGains=[];window.instrumentContexts=[];
                  class Param {constructor(v=0){this.value=v;this.events=[]} setValueAtTime(v,t){this.value=v;this.events.push([v,t])} linearRampToValueAtTime(v,t){this.value=v;this.events.push([v,t])}}
                  class Node {connect(target){this.output=target} disconnect(){this.disconnected=true}}
                  class FakeContext {
                    constructor(){this.currentTime=0;this.destination={};instrumentContexts.push(this)} resume(){return Promise.resolve()}
                    createBuffer(c,n,r){const samples=new Float32Array(n);return {samples,rate:r,getChannelData(){return samples}}}
                    createGain(){const n=new Node();n.gain=new Param(1);instrumentGains.push(n);return n}
                    createBufferSource(){const n=new Node();n.playbackRate=new Param(1);n.start=t=>n.started=t;n.stop=t=>n.stopped=t;instrumentSources.push(n);return n}
                    createOscillator(){throw Error('unexpected sine fallback')}
                  }
                  window.AudioContext=FakeContext;
                  const links=[...q('#rows tr').querySelectorAll('a')];
                  if(q('#instruments').hidden || q('#instruments>details').open || !links.some(a=>a.textContent==='Open instrument banks') ||
                     !q('#instrument-status').textContent.includes('PCM sample 10'))throw Error('initial view');
                  q('#instruments>details').open=true;q('#instrument-play').click();await wait(()=>instrumentSources.length===1);
                  let n=instrumentSources.at(-1);
                  if(n.buffer.rate!==22050 || n.buffer.samples[1]!==.25 || Math.abs(n.playbackRate.value-.5)>1e-10 ||
                     !n.loop || Math.abs(n.loopStart-1/22050)>1e-12 || q('#instrument-stop').disabled)throw Error('PCM audition');
                  instrumentContexts[0].currentTime=.52;await new Promise(r=>setTimeout(r,130));
                  if(Number(q('#instrument-progress').value)<.45)throw Error('progress');
                  q('#instrument-stop').click();if(!q('#instrument-stop').disabled)throw Error('stop');
                  q('#instrument-program').value='1';change(q('#instrument-program'));
                  if(q('#instrument-fixed-rate-control').hidden || q('#instrument-fixed-rate').value!=='15768')throw Error('fixed rate control');
                  q('#instrument-fixed-rate').value='999';q('#instrument-fixed-rate').dispatchEvent(new Event('input'));
                  if(!q('#instrument-play').disabled)throw Error('fixed rate guard');
                  q('#instrument-program').value='0';change(q('#instrument-program'));if(q('#instrument-play').disabled)throw Error('unrelated rate guard');
                  q('#instrument-program').value='1';change(q('#instrument-program'));q('#instrument-fixed-rate').value='22050';q('#instrument-fixed-rate').dispatchEvent(new Event('input'));
                  q('#instrument-play').click();await wait(()=>instrumentSources.length===2);n=instrumentSources.at(-1);
                  if(n.playbackRate.value!==1)throw Error('fixed PCM rate');q('#instrument-stop').click();
                  q('#sequence-mute-cgb').checked=true;change(q('#sequence-mute-cgb'));
                  q('#instrument-program').value='2';change(q('#instrument-program'));
                  if(!q('#instrument-mute-cgb').checked || !q('#instrument-status').textContent.includes('duty 50%'))throw Error('mute sync or square');
                  q('#instrument-play').click();await wait(()=>instrumentSources.length===3);n=instrumentSources.at(-1);
                  if(!n.loop || n.buffer.rate!==32000 || n.buffer.samples.length!==256 || n.output.output.gain.value!==0 ||
                     !q('#instrument-status').textContent.includes('muted'))throw Error('muted square');
                  q('#instrument-mute-cgb').checked=false;change(q('#instrument-mute-cgb'));
                  if(q('#sequence-mute-cgb').checked || n.output.output.gain.value!==1)throw Error('live unmute');
                  q('#instrument-program').value='3';change(q('#instrument-program'));
                  if(!q('#instrument-stop').disabled || q('#instrument-wave-navigation').hidden)throw Error('change stops or CGB link');
                  q('#instrument-wave-open').click();
                  if(!q('#audio>details').open || q('#cgb-id').value!=='0')throw Error('CGB waveform link');
                  q('#instrument-play').click();await wait(()=>instrumentSources.length===4);n=instrumentSources.at(-1);
                  if(!n.loop || n.buffer.samples[0]!==-1 || n.buffer.samples[1]!==1)throw Error('CGB waveform');q('#instrument-stop').click();
                  q('#instrument-program').value='4';change(q('#instrument-program'));
                  if(q('#instrument-keymap').hidden || q('#instrument-keymap-rows').rows.length!==2 ||
                     !q('#instrument-status').textContent.includes('square'))throw Error('split mapping');
                  q('#instrument-keymap-rows').rows[1].querySelector('button').click();
                  if(q('#instrument-key').value!=='61' || !q('#instrument-status').textContent.includes('PCM sample'))throw Error('mapped key');
                  q('#instrument-key').value='40';change(q('#instrument-key'));if(!q('#instrument-play').disabled)throw Error('unmapped key');
                  q('#instrument-program').value='5';change(q('#instrument-program'));if(!q('#instrument-play').disabled)throw Error('unsupported voice');
                  q('#instrument-program').value='6';change(q('#instrument-program'));if(!q('#instrument-play').disabled)throw Error('missing waveform');
                  q('#instrument-bank').value='1';change(q('#instrument-bank'));
                  if(!q('#instrument-program').disabled || !q('#instrument-play').disabled || q('#instrument-key').value!=='40')throw Error('empty bank');
                  q('#instrument-bank').value='0';change(q('#instrument-bank'));q('#instrument-program').value='7';change(q('#instrument-program'));
                  q('#instrument-play').click();await wait(()=>instrumentSources.length===5);n=instrumentSources.at(-1);
                  if(n.loop || Math.abs(n.stopped-n.started-.1)>1e-10 || Math.abs(Number(q('#instrument-progress').max)-.1)>1e-10 ||
                     !q('#instrument-status').textContent.includes('sample length'))throw Error('one-shot progress');
                  q('#instrument-stop').click();
                  return 'PASS';
                })()'''), 'PASS')
                browser.evaluate("document.querySelector('#instrument-bank').value='0';document.querySelector('#instrument-bank').dispatchEvent(new Event('change'));document.querySelector('#instrument-key').focus()")
                browser.click('#instrument-play')
                self.assertEqual(browser.evaluate('document.activeElement.id'), 'instrument-key')
            finally:
                browser.close()
