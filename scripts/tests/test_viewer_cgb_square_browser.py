"""Check CGB square duty patterns, playback controls and mute in Chrome."""
import json
import shutil
import tempfile
import unittest
from pathlib import Path

from viewer_browser_harness import Chrome


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class ViewerCgbSquareTest(unittest.TestCase):
    def test_square_patterns_pitch_mute_and_sine_fallback(self):
        with tempfile.TemporaryDirectory(prefix='viewer-cgb-square-') as directory:
            work = Path(directory)
            repo = Path(__file__).resolve().parents[2]
            tones = [{'id': i, 'ctype': (1 if i % 2 == 0 else 2), 'word04': i} for i in range(4)]
            tones += [{'id': 4, 'ctype': 0x40, 'word04': 0}, {'id': 5, 'ctype': 1, 'word04': 3},
                      {'id': 6, 'ctype': 4, 'word04': 0}]
            instructions, tracks = [], []
            for index, program in enumerate(range(6)):
                entry = len(instructions)
                codes = [['parameter', 0xbd, program], ['parameter', 0xc1, 2],
                         ['note', 2, 60, 100, None], ['wait', 1],
                         ['parameter', 0xc0, 96], ['wait', 1], ['end']]
                instructions.extend({'source_offset': entry + i,
                                     'next': None if code[0] == 'end' else entry + i + 1, 'code': code}
                                    for i, code in enumerate(codes))
                tracks.append({'directory_header': 0, 'track_index': index, 'entry': entry})
            data = {'profile': 'synthetic', 'extractor': 1, 'decoder': 1, 'resources': [],
                    'audio': {'path': 'audio', 'waves': [{'id': 0, 'samples': [0], 'loop_start': 0,
                                                         'source_offset': 0x2000, 'tone_ids': []}],
                              'cgb_waves': [], 'instruments': {
                        'tones': tones, 'banks': [{'source_address': 0x08001000,
                        'programs': [{'selector': i, 'tone': 6 if i == 5 else i} for i in range(6)]}],
                        'map_links': [{'parent_tone': 4, 'key': 60, 'tone': 5}]}},
                    'sequence_directory': {'songs': [{'id': 0, 'header_id': 0}],
                                           'headers': [{'id': 0, 'tone_bank_address': 0x08001000}]},
                    'sequences': {'path': 'sequences', 'clocks': [0, 4, 10] + [1] * 46,
                                  'tracks': tracks, 'instructions': instructions}}
            template = (repo / 'view_rom_data.html').read_text()
            page = work / 'square.html'
            page.write_text(template.replace('/* DATA */', json.dumps(data)))
            browser = Chrome(work, page)
            try:
                self.assertEqual(browser.evaluate('''(async()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change'));
                  const wait=async fn=>{const end=Date.now()+3000;while(!fn()){if(Date.now()>end)throw Error('timeout');await new Promise(r=>setTimeout(r,10))}};
                  window.sources=[];window.gains=[];
                  class Param {constructor(v=0){this.value=v;this.events=[]} setValueAtTime(v,t){this.value=v;this.events.push([v,t])} linearRampToValueAtTime(v,t){this.value=v;this.events.push([v,t])}}
                  class Node {connect(target){this.output=target} disconnect(){this.disconnected=true}}
                  class FakeContext {
                    constructor(){this.currentTime=0;this.destination={}} resume(){return Promise.resolve()}
                    createBuffer(c,n,r){const samples=new Float32Array(n);return {samples,rate:r,getChannelData(){return samples}}}
                    createGain(){const n=new Node();n.gain=new Param(1);gains.push(n);return n}
                    createBufferSource(){const n=new Node();n.kind='sample';n.playbackRate=new Param(1);n.start=t=>n.started=t;n.stop=t=>n.stopped=t;sources.push(n);return n}
                    createOscillator(){const n=new Node();n.kind='sine';n.frequency=new Param(0);n.start=t=>n.started=t;n.stop=t=>n.stopped=t;sources.push(n);return n}
                  }
                  window.AudioContext=FakeContext;
                  q('#sequence-use-tempo').checked=false;change(q('#sequence-use-tempo'));
                  q('#sequence-rate').value='10';q('#sequence-rate').dispatchEvent(new Event('input'));
                  if(!q('#sequence-status').textContent.includes('5 CGB') || !q('#sequence-status').textContent.includes('1 sine fallback'))throw Error('voice counts');
                  if(!q('#sequence-note-rows').textContent.includes('duty 12.5%') || !q('#sequence-note-rows').textContent.includes('duty 75%'))throw Error('voice labels');
                  q('#sequence-play').click();await wait(()=>sources.length===6);
                  const patterns=[[-1,-1,-1,-1,-1,-1,-1,1],[1,-1,-1,-1,-1,-1,-1,1],
                                  [1,-1,-1,-1,-1,1,1,1],[-1,1,1,1,1,1,1,-1]];
                  for(let n=0;n<5;n++){
                    const source=sources[n],pattern=patterns[n===4?3:n];
                    if(source.kind!=='sample' || !source.loop || source.buffer.rate!==32000 || source.buffer.samples.length!==256)throw Error('square source');
                    for(let step=0;step<8;step++)for(let i=0;i<32;i++)
                      if(source.buffer.samples[step*32+i]!==pattern[step])throw Error('duty waveform');
                  }
                  if(sources[3].buffer!==sources[4].buffer || sources[5].kind!=='sine')throw Error('cache or fallback');
                  const base=440*Math.pow(2,(60-69)/12)*256/32000,events=sources[0].playbackRate.events;
                  if(!events.some(([v,t])=>Math.abs(v-base)<1e-8 && Math.abs(t-.05)<1e-8) ||
                     !events.some(([v,t])=>Math.abs(v-base*Math.pow(2,1/12))<1e-8 && Math.abs(t-.45)<1e-8))throw Error('live pitch');
                  const bus=gains[0];if(sources.some(source=>source.output.output.output!==bus || source.output.output.gain.value!==1))throw Error('CGB route');
                  q('#sequence-mute-cgb').checked=true;change(q('#sequence-mute-cgb'));
                  if(bus.gain.value!==0 || !q('#sequence-note-rows').textContent.includes('duty 50% · muted'))throw Error('live mute');
                  q('#sequence-track').value='2';change(q('#sequence-track'));
                  if(!q('#sequence-mute-cgb').checked || !bus.disconnected)throw Error('track persistence');
                  q('#sequence-voice').value='sine';change(q('#sequence-voice'));q('#sequence-play').click();
                  await wait(()=>sources.length===7);
                  if(sources[6].kind!=='sine' || gains.findLast(node=>node.output===bus.output).gain.value!==0)throw Error('sine mode mute');
                  q('#sequence-stop').click();return 'PASS';
                })()'''), 'PASS')
            finally:
                browser.close()
