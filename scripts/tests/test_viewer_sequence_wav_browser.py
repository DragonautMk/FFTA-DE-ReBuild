"""Check offline sequence WAV export against the selected audio settings."""
import json
import shutil
import tempfile
import unittest
from pathlib import Path

from viewer_browser_harness import Chrome


def fixture(work):
    codes = [
        ['parameter', 0xbd, 0], ['parameter', 0xbe, 64], ['parameter', 0xbf, 32],
        ['parameter', 0xc1, 2], ['parameter', 0xc0, 64], ['note', 2, 60, 100, None],
        ['wait', 1], ['parameter', 0xbe, 32], ['parameter', 0xbf, 96], ['parameter', 0xc0, 96], ['end'],
        ['parameter', 0xbd, 1], ['note', 2, 60, 80, None], ['end'],
    ]
    instructions = [{'source_offset': i, 'next': None if i in (10, 13) else i + 1, 'code': code}
                    for i, code in enumerate(codes)]
    bank = {'source_address': 0x08001000, 'programs': [{'selector': 0, 'tone': 0}, {'selector': 1, 'tone': 1}]}
    data = {'profile': 'synthetic', 'extractor': 1, 'decoder': 1, 'resources': [],
            'audio': {'path': 'audio', 'waves': [{'id': 0, 'source_offset': 0x2000, 'tone_ids': [0],
                'frequency_word': 22050, 'loop_start': 0, 'flags': 0x4000, 'samples': [0, 127, 0, -128]}],
                'cgb_waves': [{'id': 0, 'source_offset': 0x3000, 'tone_ids': [1], 'amplitudes': list(range(16)) * 2}],
                'instruments': {'banks': [bank], 'tones': [
                    {'id': 0, 'ctype': 0, 'word04': 0x08002000, 'word08': 0x8080ffff},
                    {'id': 1, 'ctype': 3, 'word04': 0x08003000}], 'map_links': []}},
            'pcm_pitch': {'keys': [{'base': 0, 'shift': 0}] * 180, 'bases': [0x80000000] * 12},
            'sequence_directory': {'songs': [{'id': 7, 'header_id': 0}, {'id': 9, 'header_id': 1}],
                'headers': [{'id': 0, 'tone_bank_address': bank['source_address']}, {'id': 1}]},
            'sequences': {'path': 'sequence', 'clocks': [0, 4, 10] + [1] * 46,
                'tracks': [{'directory_header': 0, 'track_index': 0, 'entry': 0},
                           {'directory_header': 0, 'track_index': 1, 'entry': 11}], 'instructions': instructions}}
    template = (Path(__file__).resolve().parents[2] / 'view_rom_data.html').read_text()
    page = work / 'sequence-wav.html'; page.write_text(template.replace('/* DATA */', json.dumps(data)))
    return page


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class SequenceWavBrowserTest(unittest.TestCase):
    def test_unavailable_offline_renderer_recovers(self):
        with tempfile.TemporaryDirectory(prefix='viewer-sequence-wav-unavailable-') as directory:
            work = Path(directory); browser = Chrome(work, fixture(work))
            try:
                self.assertTrue(browser.evaluate('''(async()=>{
                  const q=s=>document.querySelector(s),input=e=>e.dispatchEvent(new Event('input'));
                  window.OfflineAudioContext=undefined;window.webkitOfflineAudioContext=undefined;
                  q('#sequence-rate').value='0';input(q('#sequence-rate'));
                  if(!q('#sequence-download').disabled)return false;
                  q('#sequence-rate').value='48';input(q('#sequence-rate'));
                  q('#sequence-download').click();
                  let end=Date.now()+3000;
                  while(!q('#sequence-export-status').textContent.includes('unavailable')){
                    if(Date.now()>end)throw Error('offline failure was not reported');await new Promise(r=>setTimeout(r,10));
                  }
                  return !q('#sequence-download').disabled;
                })()'''))
            finally:
                browser.close()

    def test_actual_stereo_export_pan_and_mutes(self):
        with tempfile.TemporaryDirectory(prefix='viewer-sequence-wav-real-') as directory:
            work = Path(directory); browser = Chrome(work, fixture(work))
            try:
                self.assertEqual(browser.evaluate('''(async()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change')),
                    input=e=>e.dispatchEvent(new Event('input'));
                  const wait=async fn=>{let end=Date.now()+10000;while(!fn()){if(Date.now()>end)throw Error(q('#sequence-export-status').textContent);await new Promise(r=>setTimeout(r,10))}};
                  const fake={numberOfChannels:2,length:3,sampleRate:32000,getChannelData:i=>
                    i===0?new Float32Array([-1,0,1]):new Float32Array([.5,NaN,-.5])};
                  const raw=stereoWaveBytes(fake),v=new DataView(raw.buffer);
                  if(raw.length!==56 || v.getUint32(4,true)!==48 || v.getUint16(22,true)!==2 || v.getUint16(34,true)!==16 ||
                     v.getInt16(44,true)!==-32768 || v.getInt16(46,true)!==16384 || v.getInt16(50,true)!==0 ||
                     v.getInt16(52,true)!==32767 || v.getInt16(54,true)!==-16384)throw Error('encoder');
                  let rejected=false;try{stereoWaveBytes({...fake,numberOfChannels:1})}catch(e){rejected=true}if(!rejected)throw Error('encoder guard');
                  window.blobs=[];window.names=[];const originalUrl=URL.createObjectURL.bind(URL);
                  URL.createObjectURL=blob=>{blobs.push(blob);return originalUrl(blob)};
                  HTMLAnchorElement.prototype.click=function(){names.push(this.download)};
                  q('#sequence-audio>details').open=true;
                  q('#sequence-use-tempo').checked=false;change(q('#sequence-use-tempo'));
                  q('#sequence-rate').value='10';input(q('#sequence-rate'));q('#sequence-duration').value='1';input(q('#sequence-duration'));
                  q('#sequence-envelope').checked=false;change(q('#sequence-envelope'));
                  async function save(){const count=blobs.length;q('#sequence-download').click();await wait(()=>blobs.length>count);
                    const bytes=new Uint8Array(await blobs[count].arrayBuffer()),header=new DataView(bytes.buffer);
                    if(header.getUint32(24,true)!==32000 || header.getUint16(22,true)!==2 || header.getUint16(34,true)!==16 ||
                       header.getUint32(40,true)!==bytes.length-44 || header.getUint32(4,true)!==bytes.length-8)throw Error('WAV header');
                    return header;
                  }
                  q('#sequence-track').value='0';change(q('#sequence-track'));
                  const pcm=await save();if(names[0]!=='song-7-track-0.wav')throw Error('track filename');
                  let l1=0,r1=0,l2=0,r2=0;
                  for(let i=1000;i<9000;i++){l1+=Math.abs(pcm.getInt16(44+i*4,true));r1+=Math.abs(pcm.getInt16(46+i*4,true));}
                  for(let i=17000;i<25000;i++){l2+=Math.abs(pcm.getInt16(44+i*4,true));r2+=Math.abs(pcm.getInt16(46+i*4,true));}
                  if(!(l1>r1*1.2 && r2>l2*1.2 && l1>0 && r2>0))throw Error('actual volume and pan');
                  q('#sequence-track').value='1';change(q('#sequence-track'));
                  const sounding=await save();let energy=0;for(let i=0;i<sounding.getUint32(40,true)/4;i++)energy+=Math.abs(sounding.getInt16(44+i*4,true));
                  if(!energy)throw Error('CGB output');
                  q('#sequence-mute-cgb').checked=true;change(q('#sequence-mute-cgb'));
                  const muted=await save();for(let i=44;i<muted.byteLength;i+=2)if(muted.getInt16(i,true)!==0)throw Error('CGB mute');
                  q('#sequence-mute-cgb').checked=false;change(q('#sequence-mute-cgb'));
                  q('#sequence-track').value='all';change(q('#sequence-track'));
                  q('#sequence-track-mute-all').click();const allMuted=await save();
                  for(let i=44;i<allMuted.byteLength;i+=2)if(allMuted.getInt16(i,true)!==0)throw Error('track mutes');
                  if(names.at(-1)!=='song-7-all-tracks.wav' || !q('#sequence-export-status').textContent.includes('prepared'))throw Error('mix filename');
                  q('#sequence-duration').value='181';input(q('#sequence-duration'));if(!q('#sequence-download').disabled)throw Error('duration bound');
                  q('#sequence-duration').value='1';input(q('#sequence-duration'));q('#sequence-song').value='1';change(q('#sequence-song'));
                  if(!q('#sequence-download').disabled)throw Error('empty song guard');
                  return 'PASS';
                })()'''), 'PASS')
            finally:
                browser.close()

    def test_offline_and_live_scheduling_match_and_snapshot(self):
        with tempfile.TemporaryDirectory(prefix='viewer-sequence-wav-schedule-') as directory:
            work = Path(directory); browser = Chrome(work, fixture(work))
            try:
                self.assertEqual(browser.evaluate('''(async()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change')),
                    input=e=>e.dispatchEvent(new Event('input'));
                  const wait=async fn=>{let end=Date.now()+5000;while(!fn()){if(Date.now()>end)throw Error('timeout');await new Promise(r=>setTimeout(r,10))}};
                  window.live=[];window.offlineNodes=[];window.offlineContexts=[];window.names=[];
                  class Param{constructor(v=0){this.value=v;this.events=[]}setValueAtTime(v,t){this.value=v;this.events.push([v,t])}linearRampToValueAtTime(v,t){this.value=v;this.events.push([v,t])}}
                  class Node{connect(target){this.output=target}disconnect(){}}
                  class Base{
                    constructor(list){this.currentTime=0;this.destination={};this.list=list}
                    createGain(){const n=new Node();n.gain=new Param(1);return n}
                    createStereoPanner(){const n=new Node();n.pan=new Param(0);return n}
                    createBuffer(c,n,r){return {getChannelData(){return new Float32Array(n)}}}
                    createBufferSource(){const n=new Node();n.kind='sample';n.playbackRate=new Param(1);n.start=t=>n.started=t;n.stop=t=>n.stopped=t;this.list.push(n);return n}
                    createOscillator(){const n=new Node();n.kind='sine';n.frequency=new Param(0);n.start=t=>n.started=t;n.stop=t=>n.stopped=t;this.list.push(n);return n}
                  }
                  window.AudioContext=class extends Base{constructor(){super(live)}resume(){return Promise.resolve()}};
                  window.OfflineAudioContext=class extends Base{
                    constructor(channels,frames,rate){super(offlineNodes);this.frames=frames;this.rate=rate;offlineContexts.push(this)}
                    startRendering(){return new Promise(resolve=>{this.finish=()=>resolve({numberOfChannels:2,length:this.frames,sampleRate:this.rate,
                      getChannelData:()=>new Float32Array(this.frames)})})}
                  };
                  URL.createObjectURL=()=> 'blob:fake';URL.revokeObjectURL=()=>{};HTMLAnchorElement.prototype.click=function(){names.push(this.download)};
                  q('#sequence-use-tempo').checked=false;change(q('#sequence-use-tempo'));q('#sequence-rate').value='10';input(q('#sequence-rate'));
                  q('#sequence-duration').value='1';input(q('#sequence-duration'));
                  q('#sequence-play').click();await wait(()=>live.length===2);
                  q('#sequence-download').click();await wait(()=>offlineContexts.length===1 && offlineContexts[0].finish);
                  if(!q('#sequence-download').disabled || offlineNodes.length!==2)throw Error('render state');
                  function comparable(a,b){return a.length===b.length && a.every((x,i)=>Math.abs(x[0]-b[i][0])<1e-8 && Math.abs((x[1]-.05)-b[i][1])<1e-8)}
                  for(let i=0;i<2;i++){
                    if(live[i].kind!==offlineNodes[i].kind || live[i].loop!==offlineNodes[i].loop ||
                       Math.abs(live[i].started-.05-offlineNodes[i].started)>1e-8 ||
                       Math.abs(live[i].stopped-.05-offlineNodes[i].stopped)>1e-8 ||
                       !comparable(live[i].output.gain.events,offlineNodes[i].output.gain.events))throw Error('node '+i);
                  }
                  if(!comparable(live[0].playbackRate.events,offlineNodes[0].playbackRate.events) ||
                     !comparable(live[0].output.output.gain.events,offlineNodes[0].output.output.gain.events) ||
                     !comparable(live[0].output.output.output.pan.events,offlineNodes[0].output.output.output.pan.events))throw Error('controls');
                  q('#sequence-song').value='1';change(q('#sequence-song'));
                  offlineContexts[0].finish();await wait(()=>names.length===1);
                  if(names[0]!=='song-7-all-tracks.wav' || !q('#sequence-download').disabled)throw Error('snapshot or new selection');
                  q('#sequence-stop').click();return 'PASS';
                })()'''), 'PASS')
            finally:
                browser.close()
