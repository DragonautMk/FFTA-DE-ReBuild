"""Exercise stored square-wave envelopes in live, timed and exported previews."""
import json
import shutil
import tempfile
import time
import unittest
from pathlib import Path

from viewer_browser_harness import Chrome


def fixture(work):
    codes = [['parameter', 0xbd, 0], ['note', 1, 60, 100, None], ['wait', 1], ['end']]
    data = {
        'profile': 'synthetic', 'extractor': 1, 'decoder': 1, 'resources': [],
        'audio': {'path': 'audio', 'waves': [{'id': 0, 'source_offset': 0x2000,
                    'tone_ids': [], 'samples': [0], 'frequency_word': 22050,
                    'flags': 0, 'loop_start': 0}], 'cgb_waves': [], 'instruments': {
            'banks': [{'source_address': 0x08001000, 'programs': [{'selector': 0, 'tone': 0}]}],
            'tones': [{'id': 0, 'ctype': 1, 'word04': 2, 'word08': 0x03010700}], 'map_links': []}},
        'sequence_directory': {'songs': [{'id': 5, 'header_id': 0}, {'id': 6, 'header_id': 0}],
                               'headers': [{'id': 0, 'tone_bank_address': 0x08001000}]},
        'sequences': {'path': 'sequence', 'clocks': [0, 32] + [1] * 47,
            'tracks': [{'directory_header': 0, 'track_index': 0, 'entry': 0}],
            'instructions': [{'source_offset': i, 'next': None if code[0] == 'end' else i + 1,
                              'code': code} for i, code in enumerate(codes)]},
        'timed_sound_sources': {'path': 'schedule', 'groups': [{
            'kind': 'rows', 'origin': 'fixture', 'source_offset': 256,
            'values': [{'tick': 0, 'channel': 0, 'opaque03': 0, 'sound_words': [5, 0]}]}]},
    }
    template = (Path(__file__).resolve().parents[2] / 'view_rom_data.html').read_text()
    page = work / 'audio.html'
    page.write_text(template.replace('/* DATA */', json.dumps(data)))
    return page


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class ViewerCgbSquareEnvelopeTest(unittest.TestCase):
    def test_envelope_plans_live_timed_and_settings(self):
        with tempfile.TemporaryDirectory(prefix='viewer-cgb-envelope-') as directory:
            work = Path(directory)
            browser = Chrome(work, fixture(work))
            try:
                result = browser.evaluate('''(async()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change')),
                    input=e=>e.dispatchEvent(new Event('input')),near=(a,b)=>Math.abs(a-b)<1e-9;
                  const wait=async f=>{const end=Date.now()+5000;while(!f()){
                    if(Date.now()>end)throw Error('timeout');await new Promise(r=>setTimeout(r,10))}};
                  const plan=approximateCgbSquareEnvelope(0x03010700,.5,2);
                  const expected=[[0,15],[7,14],[14,13],[21,12],[28,11],
                    [35,10],[38,9],[41,8],[44,7],[47,6],[50,5],[53,4],
                    [56,3],[59,2],[62,1],[65,0]];
                  if(!plan.stepped || !near(plan.duration,65/64) || plan.points.length!==expected.length)
                    throw Error('square envelope duration');
                  expected.forEach(([tick,value],i)=>{if(!near(plan.points[i][0],tick/64)||
                    !near(plan.points[i][1],value/15))throw Error('square level '+tick)});
                  const attack=approximateCgbSquareEnvelope(0x01000002,.5,2);
                  if(!near(attack.points[1][0],2/64) || !near(attack.points[1][1],1/15) ||
                     !near(attack.duration,30/64))throw Error('attack and zero sustain');
                  if(approximateCgbSquareEnvelope(0,1,2).duration!==0 ||
                     approximateCgbSquareEnvelope(0x020f0000,0,2).duration!==0 ||
                     !near(approximateCgbSquareEnvelope(0x020f0000,1,.25).duration,.25))throw Error('bounds');
                  const late=[];schedulePcmEnvelope({setValueAtTime:(v,t)=>late.push([v,t])},plan,1,0,15/64,2);
                  if(!near(late[0][0],13/15)||!near(late[1][1],21/64))throw Error('late start');
                  const old={startSeconds:0,endSeconds:.5,sample:{kind:'cgb-square'},envelope:null};
                  if(sequencePlaybackEvents([old],2,true,true,true).events[0].envelope!==null)
                    throw Error('legacy tone fallback');
                  window.live=[];window.offline=[];
                  class Param{constructor(events){this.events=events;this.value=1}setValueAtTime(v,t){this.value=v;this.events.push(['set',v,t])}
                    linearRampToValueAtTime(v,t){this.value=v;this.events.push(['ramp',v,t])}}
                  class Node{connect(target){this.output=target}disconnect(){}}
                  class Context{constructor(){this.currentTime=0;this.destination={};this.sink=live}resume(){return Promise.resolve()}
                    createBuffer(c,n,r){return {getChannelData(){return new Float32Array(n)}}}
                    createGain(){const n=new Node();n.events=[];n.gain=new Param(n.events);return n}
                    createBufferSource(){const n=new Node();n.playbackRate=new Param([]);n.start=t=>n.started=t;n.stop=t=>n.stopped=t;this.sink.push(n);return n}}
                  class Offline extends Context{constructor(c,n,r){super();this.sink=offline;this.length=n;this.rate=r}
                    startRendering(){const n=this.length;return Promise.resolve({numberOfChannels:2,length:n,sampleRate:this.rate,
                      getChannelData(){return new Float32Array(n)}})}}
                  window.AudioContext=Context;window.OfflineAudioContext=Offline;
                  q('#sequence-use-tempo').checked=false;change(q('#sequence-use-tempo'));
                  q('#sequence-rate').value='64';input(q('#sequence-rate'));
                  q('#sequence-duration').value='2';input(q('#sequence-duration'));
                  q('#sequence-envelope').checked=false;change(q('#sequence-envelope'));
                  if(!q('#sequence-cgb-envelope').checked)throw Error('default setting');
                  q('#sequence-play').click();await wait(()=>live.length===1);
                  const events=live[0].output.events,amp=.08*100/127;
                  if(!near(live[0].stopped-live[0].started,65/64) || events.length!==expected.length ||
                     events.some(e=>e[0]!=='set'))throw Error('live schedule');
                  expected.forEach(([tick,value],i)=>{if(!near(events[i][1],amp*value/15)||!near(events[i][2],.05+tick/64))
                    throw Error('live gain '+tick)});
                  q('#sequence-download').click();await wait(()=>offline.length===1);
                  if(offline[0].output.events.some((e,i)=>e[0]!=='set'||!near(e[1],events[i][1])||!near(e[2],events[i][2]-.05)))
                    throw Error('offline schedule');
                  q('#sequence-stop').click();q('#timed-sound-duration').value='2';input(q('#timed-sound-duration'));
                  q('#timed-sound-play').click();await wait(()=>live.length===2);
                  if(!near(live[1].stopped-live[1].started,65/64)||live[1].output.events.length!==expected.length)
                    throw Error('timed schedule');
                  q('#timed-sound-stop').click();q('#sequence-cgb-envelope').checked=false;change(q('#sequence-cgb-envelope'));
                  q('#sequence-song').value='1';change(q('#sequence-song'));
                  if(q('#sequence-cgb-envelope').checked)throw Error('song setting lost');
                  q('#sequence-play').click();await wait(()=>live.length===3);
                  if(!near(live[2].stopped-live[2].started,.5)||!live[2].output.events.some(e=>e[0]==='ramp'))
                    throw Error('unchecked fallback');
                  q('#sequence-stop').click();return 'PASS';
                })()''')
                self.assertEqual(result, 'PASS')
                browser.command('Page.reload')
                deadline = time.monotonic() + 10
                while time.monotonic() < deadline:
                    if browser.evaluate("document.readyState==='complete' && document.querySelector('#sequence-cgb-envelope') && !document.querySelector('#sequence-cgb-envelope').checked"):
                        break
                    time.sleep(.05)
                else:
                    self.fail('CGB envelope choice did not survive reload')
            finally:
                browser.close()

    def test_actual_wav_changes_and_mute(self):
        with tempfile.TemporaryDirectory(prefix='viewer-cgb-envelope-wav-') as directory:
            work = Path(directory)
            browser = Chrome(work, fixture(work))
            try:
                result = browser.evaluate('''(async()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change')),
                    input=e=>e.dispatchEvent(new Event('input'));
                  const wait=async f=>{const end=Date.now()+10000;while(!f()){
                    if(Date.now()>end)throw Error(q('#sequence-export-status').textContent);await new Promise(r=>setTimeout(r,10))}};
                  window.blobs=[];const original=URL.createObjectURL.bind(URL);
                  URL.createObjectURL=b=>{blobs.push(b);return original(b)};
                  HTMLAnchorElement.prototype.click=function(){};
                  q('#sequence-use-tempo').checked=false;change(q('#sequence-use-tempo'));
                  q('#sequence-rate').value='64';input(q('#sequence-rate'));
                  q('#sequence-duration').value='2';input(q('#sequence-duration'));
                  async function save(){const count=blobs.length;q('#sequence-download').click();await wait(()=>blobs.length>count);
                    return new DataView(await blobs[count].arrayBuffer())}
                  const shaped=await save();
                  function rms(view,start,end){let sum=0,n=0;for(let i=Math.floor(start*32000);i<Math.floor(end*32000);i++){
                    const x=view.getInt16(44+i*4,true);sum+=x*x;n++}return Math.sqrt(sum/n)}
                  const early=rms(shaped,.03,.09),late=rms(shaped,.4,.46);
                  if(shaped.getUint32(24,true)!==32000 || shaped.getUint16(22,true)!==2 ||
                     shaped.getUint32(40,true)/4/32000<1 || !early || late/early<.65 || late/early>.9)
                    throw Error('WAV envelope did not decay');
                  q('#sequence-cgb-envelope').checked=false;change(q('#sequence-cgb-envelope'));
                  const plain=await save();if(Math.abs(plain.getUint32(40,true)/4/32000-.5)>.001)throw Error('WAV fallback');
                  q('#sequence-mute-cgb').checked=true;change(q('#sequence-mute-cgb'));
                  const muted=await save();if(rms(muted,.1,.2)!==0)throw Error('WAV CGB mute');
                  return 'PASS';
                })()''')
                self.assertEqual(result, 'PASS')
            finally:
                browser.close()
