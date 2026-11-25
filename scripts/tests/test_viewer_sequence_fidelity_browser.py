"""Check note lengths, source PCM pitch and envelope scheduling in Chrome."""
import json
import shutil
import tempfile
import unittest
from pathlib import Path

from viewer_browser_harness import Chrome


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class ViewerSequenceFidelityTest(unittest.TestCase):
    def test_lengths_pitch_and_pcm_envelope(self):
        with tempfile.TemporaryDirectory(prefix='viewer-audio-fidelity-') as directory:
            work = Path(directory)
            repo = Path(__file__).resolve().parents[2]
            tone = {'id': 0, 'ctype': 0, 'word04': 0x08002000, 'word08': 0x80408080}
            wave = {'id': 0, 'source_offset': 0x2000, 'tone_ids': [0], 'frequency_word': 22050,
                    'loop_start': 0, 'flags': 0x4000, 'samples': [0, 32, 0, -32]}
            codes = [['parameter', 0xbd, 0], ['note', 2, 60, 100, 3], ['wait', 1],
                     ['note', 3, 61, 100, 6], ['wait', 1], ['end_tie', 61], ['end']]
            instructions = [{'source_offset': i, 'next': i + 1 if i + 1 < len(codes) else None, 'code': code}
                            for i, code in enumerate(codes)]
            data = {'profile': 'synthetic', 'extractor': 1, 'decoder': 1, 'resources': [],
                    'audio': {'path': 'audio', 'waves': [wave], 'cgb_waves': [], 'instruments': {
                        'banks': [{'source_address': 0x08001000, 'programs': [{'selector': 0, 'tone': 0}]}],
                        'tones': [tone], 'map_links': []}},
                    'pcm_pitch': {'keys': [{'base': 0, 'shift': 0}] * 180, 'bases': [0x80000000] * 12},
                    'sequence_directory': {'songs': [{'id': 0, 'header_id': 0}],
                                           'headers': [{'id': 0, 'tone_bank_address': 0x08001000}]},
                    'sequences': {'path': 'sequence', 'clocks': [0, 1, 2, 250] + [1] * 45,
                                  'tracks': [{'directory_header': 0, 'track_index': 0, 'entry': 0}],
                                  'instructions': instructions}}
            template = (repo / 'view_rom_data.html').read_text()
            page = work / 'audio.html'
            page.write_text(template.replace('/* DATA */', json.dumps(data)))
            browser = Chrome(work, page)
            try:
                result = browser.evaluate('''(async()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change'));
                  const wait=async fn=>{let end=Date.now()+2000;while(!fn()){if(Date.now()>end)throw Error('timeout');await new Promise(r=>setTimeout(r,10))}};
                  const high=(a,b)=>Number((BigInt(a>>>0)*BigInt(b>>>0))>>32n)>>>0;
                  const pitch={keys:Array.from({length:180},(_,i)=>({base:i%12,shift:i%16})),
                    bases:Array.from({length:12},(_,i)=>(0xf1234567-i*0x1020304)>>>0)};
                  for(let i=0;i<600;i++){
                    const word=(0xabcdef01+i*1234567)>>>0, raw=(i*43)&255, f=(i*67)&255;
                    let key=raw, fraction=f;if(key>178){key=178;fraction=255}
                    const a=pitch.bases[pitch.keys[key].base]>>>pitch.keys[key].shift,
                      b=pitch.bases[pitch.keys[key+1].base]>>>pitch.keys[key+1].shift;
                    const expected=high(word,(a+high((b-a)>>>0,(fraction<<24)>>>0))>>>0);
                    if(sourcePcmFrequency(pitch,word,raw,f)!==expected)throw Error('multiply-high pitch '+i);
                  }
                  const step=280896/16777216, plan=approximatePcmEnvelope(0x80408080,.5,3);
                  if(Math.abs(plan.duration-(.5+7*step))>1e-9 || plan.points[0][1]!==128/255 || plan.points.at(-1)[1]!==0)throw Error('envelope plan');
                  if(approximatePcmEnvelope(0,.5,3).duration!==0 || approximatePcmEnvelope(0x800080ff,.5,3).duration>=.5)throw Error('silent envelopes');
                  window.sources=[];window.gains=[];
                  class FakeContext {
                    constructor(){this.currentTime=0;this.destination={}}
                    resume(){return Promise.resolve()}
                    createBuffer(c,n,r){return {getChannelData(){return new Float32Array(n)}}}
                    createGain(){const events=[];const node={events,gain:{setValueAtTime(v,t){events.push(['set',v,t])},
                      linearRampToValueAtTime(v,t){events.push(['ramp',v,t])}},connect(){},disconnect(){}};gains.push(node);return node}
                    createBufferSource(){const node={playbackRate:{value:1},connect(){},start(t){this.started=t},stop(t){this.stopped=t}};sources.push(node);return node}
                    createOscillator(){throw Error('expected PCM')}
                  }
                  window.AudioContext=FakeContext;q('#sequence-audio>details').open=true;
                  q('#sequence-rate').value='10';q('#sequence-rate').dispatchEvent(new Event('input'));
                  const rows=[...q('#sequence-note-rows').rows];
                  if(rows.length!==2 || rows[0].cells[1].textContent!=='0' || rows[0].cells[2].textContent!=='5' ||
                     rows[1].cells[1].textContent!=='1' || rows[1].cells[2].textContent!=='2')throw Error('extension and byte wrap');
                  if(q('#sequence-envelope').disabled || !q('#sequence-envelope').checked)throw Error('envelope control');
                  q('#sequence-play').click();await wait(()=>sources.length===2);
                  if(Math.abs(sources[0].playbackRate.value-.5)>1e-12 || Math.abs((sources[0].stopped-sources[0].started)-plan.duration)>1e-8)throw Error('pitch or release');
                  const first=gains[1].events.find(e=>e[0]==='ramp');
                  const expected=.08*100/127*(128/255+(127/255)*(.003/step));
                  if(!first || Math.abs(first[1]-expected)>1e-7 || gains[1].events.length<=5)throw Error('envelope automation');
                  q('#sequence-envelope').checked=false;change(q('#sequence-envelope'));q('#sequence-play').click();await wait(()=>sources.length===4);
                  if(Math.abs(sources[2].stopped-sources[2].started-.5)>1e-8 || gains[4].events.length!==4)throw Error('simple envelope fallback');
                  q('#sequence-stop').click();return 'PASS';
                })()''')
                self.assertEqual(result, 'PASS')
            finally:
                browser.close()
