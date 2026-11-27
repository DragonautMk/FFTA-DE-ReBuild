"""Check frame-stepped PCM envelopes in live, timed and exported audio."""
import json
import shutil
import tempfile
import unittest
from pathlib import Path

from viewer_browser_harness import Chrome


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class ViewerPcmFrameEnvelopeTest(unittest.TestCase):
    def test_frame_envelope_and_smooth_alternative(self):
        with tempfile.TemporaryDirectory(prefix='viewer-pcm-frame-envelope-') as directory:
            work = Path(directory)
            repo = Path(__file__).resolve().parents[2]
            codes = [['parameter', 0xbd, 0], ['note', 1, 60, 100, None], ['wait', 1], ['end']]
            data = {
                'profile': 'synthetic', 'extractor': 1, 'decoder': 1, 'resources': [],
                'audio': {'path': 'audio', 'waves': [{'id': 0, 'source_offset': 0x2000,
                    'tone_ids': [0], 'frequency_word': 22050, 'loop_start': 0,
                    'flags': 0x4000, 'samples': [0, 32, 0, -32]}], 'cgb_waves': [],
                    'instruments': {'banks': [{'source_address': 0x08001000,
                        'programs': [{'selector': 0, 'tone': 0}]}],
                        'tones': [{'id': 0, 'ctype': 0, 'word04': 0x08002000, 'word08': 0x80408080}],
                        'map_links': []}},
                'sequence_directory': {'songs': [{'id': 5, 'header_id': 0}, {'id': 6, 'header_id': 0}],
                    'headers': [{'id': 0, 'tone_bank_address': 0x08001000}]},
                'sequences': {'path': 'sequence', 'clocks': [0, 3] + [1] * 47,
                    'tracks': [{'directory_header': 0, 'track_index': 0, 'entry': 0}],
                    'instructions': [{'source_offset': i, 'next': None if code[0] == 'end' else i + 1,
                                      'code': code} for i, code in enumerate(codes)]},
                'timed_sound_sources': {'path': 'schedule', 'groups': [{
                    'kind': 'rows', 'origin': 'fixture', 'source_offset': 256,
                    'values': [{'tick': 0, 'channel': 0, 'opaque03': 0, 'sound_words': [5, 0]}]}]},
            }
            page = work / 'audio.html'
            page.write_text((repo / 'view_rom_data.html').read_text().replace('/* DATA */', json.dumps(data)))
            browser = Chrome(work, page)
            try:
                result = browser.evaluate('''(async()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change')),
                    input=e=>e.dispatchEvent(new Event('input'));
                  const step=280896/16777216,near=(a,b)=>Math.abs(a-b)<1e-8;
                  const wait=async f=>{const end=Date.now()+5000;while(!f()){if(Date.now()>end)throw Error('timeout');await new Promise(r=>setTimeout(r,10))}};
                  const expected=[[0,128],[1,255],[2,127],[3,63],[4,31],[5,15],[6,7],[7,3],[8,1],[9,0]];
                  const plan=framePcmEnvelope(0x80408080,3*step,2);
                  if(!plan.stepped || !near(plan.duration,9*step) || plan.points.length!==expected.length)throw Error('frame plan length');
                  expected.forEach(([frame,value],i)=>{if(!near(plan.points[i][0],frame*step)||
                    !near(plan.points[i][1],value/255))throw Error('frame plan '+frame)});
                  const between=framePcmEnvelope(0x80408080,2.5*step,2);
                  if(!near(between.duration,9*step) || between.points.some((p,i)=>!near(p[0],plan.points[i][0])||!near(p[1],plan.points[i][1])))
                    throw Error('note-off waits for mixer frame');
                  const clipped=framePcmEnvelope(0x80408080,10*step,2.5*step);
                  if(!near(clipped.duration,2.5*step)||clipped.points.length!==3)throw Error('preview cutoff');
                  if(framePcmEnvelope(0,.5,3).duration!==0 || framePcmEnvelope(0x80408080,0,2).duration!==0 ||
                     !near(framePcmEnvelope(0x000000ff,10*step,2).duration,step))throw Error('silent envelopes');
                  const late=[];schedulePcmEnvelope({setValueAtTime:(v,t)=>late.push([v,t])},plan,1,0,2.5*step,9*step);
                  if(!near(late[0][0],127/255)||!near(late[1][0],63/255)||!near(late[1][1],3*step))
                    throw Error('late start uses held level');

                  window.live=[];window.offline=[];
                  class Param {constructor(events){this.events=events}setValueAtTime(v,t){this.events.push(['set',v,t])}
                    linearRampToValueAtTime(v,t){this.events.push(['ramp',v,t])}}
                  class Node {connect(target){this.output=target}disconnect(){}}
                  class Context {constructor(){this.currentTime=0;this.destination={};this.sink=live}resume(){return Promise.resolve()}
                    createBuffer(c,n,r){return {getChannelData(){return new Float32Array(n)}}}
                    createGain(){const n=new Node();n.events=[];n.gain=new Param(n.events);return n}
                    createBufferSource(){const n=new Node();n.playbackRate={value:1};n.start=t=>n.started=t;n.stop=t=>n.stopped=t;this.sink.push(n);return n}}
                  class Offline extends Context {constructor(c,n,r){super();this.sink=offline;this.length=n;this.rate=r}
                    startRendering(){const n=this.length;return Promise.resolve({numberOfChannels:2,length:n,sampleRate:this.rate,
                      getChannelData(){return new Float32Array(n)}})}}
                  window.AudioContext=Context;window.OfflineAudioContext=Offline;
                  q('#sequence-audio>details').open=true;
                  if(!q('#sequence-frame-envelope').checked || q('#sequence-frame-envelope').disabled)throw Error('default frame envelope');
                  q('#sequence-play').click();await wait(()=>live.length===1);
                  const amp=.08*100/127,events=live[0].output.events;
                  if(!near(live[0].stopped-live[0].started,9*step) || events.length!==expected.length || events.some(e=>e[0]!=='set'))
                    throw Error('live stepped envelope');
                  expected.forEach(([frame,value],i)=>{if(!near(events[i][1],amp*value/255)||!near(events[i][2],.05+frame*step))
                    throw Error('live level '+frame)});
                  q('#sequence-download').click();await wait(()=>offline.length===1);
                  const exported=offline[0].output.events;
                  if(exported.length!==events.length || exported.some((e,i)=>e[0]!=='set'||!near(e[1],events[i][1])||!near(e[2],events[i][2]-.05)))
                    throw Error('WAV envelope differs from live');
                  q('#sequence-stop').click();q('#timed-sounds>details').open=true;q('#timed-sound-audition').open=true;
                  q('#timed-sound-duration').value='1';input(q('#timed-sound-duration'));
                  q('#timed-sound-play').click();await wait(()=>live.length===2);
                  if(live[1].output.events.some(e=>e[0]!=='set')||!near(live[1].stopped-live[1].started,9*step))
                    throw Error('timed source envelope');
                  q('#timed-sound-stop').click();q('#sequence-frame-envelope').checked=false;change(q('#sequence-frame-envelope'));
                  q('#sequence-song').value='1';change(q('#sequence-song'));
                  if(q('#sequence-frame-envelope').checked)throw Error('choice lost on song change');
                  q('#sequence-play').click();await wait(()=>live.length===3);
                  if(!live[2].output.events.some(e=>e[0]==='ramp'))throw Error('smooth alternative');
                  q('#sequence-stop').click();q('#sequence-envelope').checked=false;change(q('#sequence-envelope'));
                  if(!q('#sequence-frame-envelope').disabled)throw Error('disabled envelope mode');
                  q('#sequence-envelope').checked=true;change(q('#sequence-envelope'));
                  if(q('#sequence-frame-envelope').disabled || q('#sequence-frame-envelope').checked)throw Error('retained envelope mode');
                  return 'PASS';
                })()''')
                self.assertEqual(result, 'PASS')
            finally:
                browser.close()
