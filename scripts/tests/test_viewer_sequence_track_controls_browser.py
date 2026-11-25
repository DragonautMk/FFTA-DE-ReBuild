"""Check decoded track controls and live note automation in Chrome."""
import json
import shutil
import tempfile
import unittest
from pathlib import Path

from viewer_browser_harness import Chrome


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class ViewerSequenceTrackControlsTest(unittest.TestCase):
    def test_volume_pan_pitch_automation_and_cgb_mute(self):
        with tempfile.TemporaryDirectory(prefix='viewer-track-controls-') as directory:
            work = Path(directory)
            repo = Path(__file__).resolve().parents[2]
            codes = [
                ['parameter', 0xbd, 0], ['parameter', 0xbe, 64], ['parameter', 0xbf, 32],
                ['parameter', 0xbc, 12], ['parameter', 0xc1, 2], ['parameter', 0xc0, 64],
                ['parameter', 0xc8, 64], ['note', 2, 60, 100, None], ['wait', 1],
                ['parameter', 0xbe, 32], ['parameter', 0xbf, 96], ['parameter', 0xc0, 96],
                ['wait', 1], ['parameter', 0xc0, 64], ['end'],
                ['note', 2, 60, 100, None], ['end'],
            ]
            instructions = [{'source_offset': i, 'next': None if i in (14, 16) else i + 1, 'code': code}
                            for i, code in enumerate(codes)]
            tones = [{'id': 0, 'ctype': 0, 'word04': 0x08002000},
                     {'id': 1, 'ctype': 3, 'word04': 0x08003000}]
            banks = [{'source_address': 0x08001000 + i * 0x100, 'programs': [{'selector': 0, 'tone': i}]}
                     for i in range(2)]
            data = {'profile': 'synthetic', 'extractor': 1, 'decoder': 1, 'resources': [],
                    'audio': {'path': 'audio', 'waves': [{'id': 0, 'source_offset': 0x2000, 'tone_ids': [0],
                        'frequency_word': 22050, 'loop_start': 0, 'flags': 0x4000, 'samples': [0, 32, 0, -32]}],
                        'cgb_waves': [{'id': 0, 'source_offset': 0x3000, 'tone_ids': [1], 'amplitudes': list(range(16)) * 2}],
                        'instruments': {'banks': banks, 'tones': tones, 'map_links': []}},
                    'pcm_pitch': {'keys': [{'base': 0, 'shift': 0}] * 180, 'bases': [0x80000000] * 12},
                    'sequence_directory': {'songs': [{'id': i, 'header_id': i} for i in range(2)],
                        'headers': [{'id': i, 'tone_bank_address': banks[i]['source_address']} for i in range(2)]},
                    'sequences': {'path': 'sequence', 'clocks': [0, 4, 10] + [1] * 46,
                        'tracks': [{'directory_header': h, 'track_index': t, 'entry': entry}
                                   for h in range(2) for t, entry in ((0, 0), (1, 15))],
                        'instructions': instructions}}
            template = (repo / 'view_rom_data.html').read_text()
            page = work / 'controls.html'
            page.write_text(template.replace('/* DATA */', json.dumps(data)))
            browser = Chrome(work, page)
            try:
                result = browser.evaluate('''(async()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change'));
                  const wait=async fn=>{let end=Date.now()+2000;while(!fn()){if(Date.now()>end)throw Error('timeout');await new Promise(r=>setTimeout(r,10))}};
                  window.sources=[];window.gains=[];window.panners=[];window.contexts=[];
                  class Param {constructor(v=0){this.value=v;this.events=[]} setValueAtTime(v,t){this.value=v;this.events.push([v,t])} linearRampToValueAtTime(v,t){this.value=v;this.events.push([v,t])}}
                  class Node {connect(target){this.output=target} disconnect(){this.disconnected=true}}
                  class FakeContext {
                    constructor(){this.currentTime=0;this.destination={};contexts.push(this)} resume(){return Promise.resolve()}
                    createBuffer(c,n,r){return {getChannelData(){return new Float32Array(n)}}}
                    createGain(){const n=new Node();n.gain=new Param(1);gains.push(n);return n}
                    createStereoPanner(){const n=new Node();n.pan=new Param(0);panners.push(n);return n}
                    createBufferSource(){const n=new Node();n.kind='sample';n.playbackRate=new Param(1);n.start=t=>n.started=t;n.stop=t=>n.stopped=t;sources.push(n);return n}
                    createOscillator(){const n=new Node();n.kind='sine';n.frequency=new Param(0);n.start=t=>n.started=t;n.stop=t=>n.stopped=t;sources.push(n);return n}
                  }
                  window.AudioContext=FakeContext;q('#sequence-audio>details').open=true;
                  q('#sequence-use-tempo').checked=false;change(q('#sequence-use-tempo'));q('#sequence-rate').value='10';q('#sequence-rate').dispatchEvent(new Event('input'));
                  if(!q('#sequence-track-controls').checked || q('#sequence-control-list').open || q('#sequence-control-rows').rows.length!==10)throw Error('control list');
                  if(!q('#sequence-control-rows').textContent.includes('Key shift') || !q('#sequence-control-rows').textContent.includes('Pitch bend'))throw Error('control labels');
                  q('#sequence-play').click();await wait(()=>sources.length===2);
                  const pcm=sources[0],plain=sources[1],pitch=pcm.playbackRate.events;
                  if(pcm.kind!=='sample' || plain.kind!=='sine' || Math.abs(pitch[0][0]-1)>1e-9 ||
                     !pitch.some(([v,t])=>Math.abs(v-Math.pow(2,1/12))<1e-9 && Math.abs(t-.45)<1e-9) ||
                     !pitch.some(([v,t])=>Math.abs(v-1)<1e-9 && Math.abs(t-.85)<1e-9))throw Error('live pitch');
                  const volume=pcm.output.output,pan=volume.output;
                  if(!volume.gain.events.some(([v,t])=>Math.abs(v-64/127)<1e-9 && Math.abs(t-.05)<1e-9) ||
                     !volume.gain.events.some(([v,t])=>Math.abs(v-32/127)<1e-9 && Math.abs(t-.45)<1e-9) ||
                     !pan.pan.events.some(([v,t])=>v===-.5 && Math.abs(t-.05)<1e-9) ||
                     !pan.pan.events.some(([v,t])=>v===.5 && Math.abs(t-.45)<1e-9))throw Error('volume or pan');
                  if(plain.output.output.output!==contexts[0].destination || plain.output.output.gain.value!==1)throw Error('separate track routing');
                  q('#sequence-track-controls').checked=false;change(q('#sequence-track-controls'));
                  const oldSources=sources.length;q('#sequence-play').click();await wait(()=>sources.length>=oldSources+2);
                  if(Math.abs(sources[oldSources].playbackRate.value-.5)>1e-9 || sources[oldSources].output.output.output!==contexts[0].destination)throw Error('manual control bypass');
                  q('#sequence-track-controls').checked=true;change(q('#sequence-track-controls'));
                  q('#sequence-mute-cgb').checked=true;change(q('#sequence-mute-cgb'));q('#sequence-song').value='1';change(q('#sequence-song'));
                  if(!q('#sequence-mute-cgb').checked || !q('#sequence-track-controls').checked)throw Error('settings persisted');
                  const before=sources.length;q('#sequence-play').click();await wait(()=>sources.length>=before+2);
                  const cgb=sources[before],bus=cgb.output.output.output.output.output;
                  if(bus.output!==contexts[0].destination || bus.gain.value!==0)throw Error('muted CGB route');
                  q('#sequence-mute-cgb').checked=false;change(q('#sequence-mute-cgb'));if(bus.gain.value!==1)throw Error('live CGB unmute');
                  contexts[0].createStereoPanner=undefined;q('#sequence-play').click();await wait(()=>q('#sequence-status').textContent.includes('pan unavailable'));
                  q('#sequence-stop').click();return 'PASS';
                })()''')
                self.assertEqual(result, 'PASS')
            finally:
                browser.close()
