"""Exercise frame-quantized sequence timing and its browser consumers."""
import json
import shutil
import tempfile
import unittest
from pathlib import Path

from viewer_browser_harness import Chrome


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class ViewerSequenceFrameTempoTest(unittest.TestCase):
    def test_frame_timing_playback_export_and_schedule(self):
        with tempfile.TemporaryDirectory(prefix='viewer-frame-tempo-') as directory:
            work = Path(directory)
            repo = Path(__file__).resolve().parents[2]
            codes = [
                ['parameter', 0xbb, 40], ['wait', 1], ['parameter', 0xbb, 120], ['wait', 1], ['end'],
                ['note', 2, 60, 100, None], ['wait', 1], ['note', 1, 64, 100, None], ['wait', 1], ['end'],
            ]
            data = {
                'profile': 'synthetic', 'extractor': 1, 'decoder': 1, 'resources': [],
                'sequence_directory': {
                    'songs': [{'id': 5, 'header_id': 0}, {'id': 6, 'header_id': 0}],
                    'headers': [{'id': 0}],
                },
                'sequences': {
                    'path': 'synthetic sequence', 'clocks': [0, 4, 8] + [1] * 46,
                    'tracks': [{'directory_header': 0, 'track_index': 0, 'entry': 0},
                               {'directory_header': 0, 'track_index': 1, 'entry': 5}],
                    'instructions': [{'source_offset': i, 'next': None if code[0] == 'end' else i + 1,
                                      'code': code} for i, code in enumerate(codes)],
                },
                'timed_sound_sources': {'path': 'synthetic schedule', 'groups': [{
                    'kind': 'rows', 'origin': 'fixture', 'source_offset': 256,
                    'values': [{'tick': 0, 'channel': 0, 'opaque03': 0, 'sound_words': [5, 0]}],
                }]},
            }
            page = work / 'tempo.html'
            page.write_text((repo / 'view_rom_data.html').read_text().replace('/* DATA */', json.dumps(data)))
            browser = Chrome(work, page)
            try:
                result = browser.evaluate('''(async()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change'));
                  const input=e=>e.dispatchEvent(new Event('input'));
                  const near=(a,b)=>Math.abs(a-b)<1e-8, frame=280896/16777216;
                  const wait=async f=>{const end=Date.now()+5000;while(!f()){if(Date.now()>end)throw Error('timeout');await new Promise(r=>setTimeout(r,10))}};
                  const initial=frameTempoTimeline([],1);
                  if(initial.endTick!==60 || !near(initial.timeAt(0),0) || !near(initial.timeAt(59),59*frame) ||
                     Number.isFinite(initial.timeAt(60)))throw Error('initial frame cadence');
                  const slow=frameTempoTimeline([{tick:0,track:0,order:0,value:40}],1);
                  for(const [tick,expectedFrame] of [[1,2],[2,4],[3,6],[4,8],[8,15]])
                    if(!near(slow.timeAt(tick),expectedFrame*frame))throw Error('slow accumulator '+tick);
                  const fast=frameTempoTimeline([{tick:0,track:0,order:0,value:250},
                    {tick:1,track:0,order:1,value:0}],1);
                  if(!fast.paused || fast.endTick!==4 || !near(fast.timeAt(1),frame) ||
                     !near(fast.timeAt(2),frame) || !near(fast.timeAt(3),frame) ||
                     Number.isFinite(fast.timeAt(4)))throw Error('zero tempo and accumulated ticks');
                  const resumed=frameTempoTimeline([{tick:0,track:0,order:0,value:250},
                    {tick:1,track:0,order:1,value:0},{tick:2,track:0,order:2,value:80}],1);
                  if(resumed.paused || !near(resumed.timeAt(3),frame) || !Number.isFinite(resumed.timeAt(4)))
                    throw Error('same-frame tempo resumes');
                  const order=frameTempoTimeline([{tick:0,track:1,order:0,value:0},
                    {tick:0,track:0,order:0,value:250}],1);
                  if(!order.paused || order.endTick!==1)throw Error('same-tick track ordering');

                  window.live=[];window.rendered=[];
                  class Param {constructor(v=0){this.value=v}setValueAtTime(v){this.value=v}linearRampToValueAtTime(v){this.value=v}}
                  class Node {connect(target){this.output=target}disconnect(){}}
                  class FakeContext {
                    constructor(){this.currentTime=0;this.destination={};this.output=live}
                    resume(){return Promise.resolve()}
                    createGain(){const n=new Node();n.gain=new Param(1);return n}
                    createOscillator(){const n=new Node();n.frequency=new Param();n.start=t=>n.started=t;n.stop=t=>n.stopped=t;this.output.push(n);return n}
                  }
                  class FakeOffline extends FakeContext {
                    constructor(channels,length,rate){super();this.output=rendered;this.length=length;this.rate=rate}
                    startRendering(){const n=this.length;return Promise.resolve({numberOfChannels:2,length:n,sampleRate:this.rate,
                      getChannelData(){return new Float32Array(n)}})}
                  }
                  window.AudioContext=FakeContext;window.OfflineAudioContext=FakeOffline;
                  q('#sequence-audio>details').open=true;q('#sequence-track').value='1';change(q('#sequence-track'));
                  if(!q('#sequence-frame-tempo').checked || !q('#sequence-status').textContent.includes('frame tempo: 2 changes'))
                    throw Error('default frame mode');
                  q('#sequence-play').click();await wait(()=>live.length===2);
                  if(!near(live[0].stopped-live[0].started,11*frame) || !near(live[1].started-live[0].started,8*frame) ||
                     !near(live[1].stopped-live[0].started,11*frame))throw Error('live frame timing');
                  q('#sequence-stop').click();q('#sequence-download').click();await wait(()=>rendered.length===2);
                  if(!near(rendered[0].stopped,11*frame) || !near(rendered[1].started,8*frame) ||
                     !near(rendered[1].stopped,11*frame))throw Error('WAV frame timing');
                  q('#timed-sounds>details').open=true;q('#timed-sound-audition').open=true;
                  q('#timed-sound-duration').value='1';input(q('#timed-sound-duration'));
                  q('#timed-sound-play').click();await wait(()=>live.length===4);
                  if(!near(live[3].started-live[2].started,8*frame))throw Error('schedule frame timing');
                  q('#timed-sound-stop').click();
                  q('#sequence-song').value='1';change(q('#sequence-song'));
                  if(!q('#sequence-frame-tempo').checked)throw Error('mode did not persist across songs');
                  q('#sequence-frame-tempo').checked=false;change(q('#sequence-frame-tempo'));
                  q('#sequence-track').value='1';change(q('#sequence-track'));q('#sequence-play').click();await wait(()=>live.length===6);
                  if(!near(live[5].started-live[4].started,4/estimatedTempoRate(40)))throw Error('continuous alternative');
                  q('#sequence-stop').click();q('#sequence-use-tempo').checked=false;change(q('#sequence-use-tempo'));
                  if(!q('#sequence-frame-tempo').disabled || !q('#sequence-status').textContent.includes('manual timing'))
                    throw Error('manual mode');
                  q('#sequence-use-tempo').checked=true;change(q('#sequence-use-tempo'));
                  if(q('#sequence-frame-tempo').disabled || q('#sequence-frame-tempo').checked)
                    throw Error('frame choice not preserved');
                  return 'PASS';
                })()''')
                self.assertEqual(result, 'PASS')
            finally:
                browser.close()
