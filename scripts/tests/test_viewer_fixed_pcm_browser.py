"""Check fixed-rate PCM samples in the approximate sequence player."""
import json
import shutil
import tempfile
import unittest
from pathlib import Path

from viewer_browser_harness import Chrome


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class ViewerFixedPcmTest(unittest.TestCase):
    def test_fixed_sample_rate_loop_envelope_and_pitch_independence(self):
        with tempfile.TemporaryDirectory(prefix='viewer-fixed-pcm-') as directory:
            work = Path(directory)
            repo = Path(__file__).resolve().parents[2]
            tones = [{'id': 0, 'ctype': 8, 'word04': 0x08002000, 'word08': 0x80408080},
                     {'id': 1, 'ctype': 0, 'word04': 0x08002000, 'word08': 0x80408080},
                     {'id': 2, 'ctype': 8, 'word04': 0x08003000, 'word08': 0x80408080}]
            waves = [{'id': 0, 'source_offset': 0x2000, 'tone_ids': [0, 1], 'frequency_word': 22050,
                      'loop_start': 1, 'flags': 0x4000, 'samples': [0, 32, 0, -32]}]
            instructions, tracks = [], []
            for index, (program, key) in enumerate([(0, 60), (0, 72), (1, 60), (2, 60)]):
                start = len(instructions)
                codes = [['parameter', 0xbd, program], ['parameter', 0xc1, 2],
                         ['note', 2, key, 100, None], ['wait', 1],
                         ['parameter', 0xc0, 96], ['wait', 1], ['end']]
                instructions.extend({'source_offset': start + i,
                                     'next': None if code[0] == 'end' else start + i + 1, 'code': code}
                                    for i, code in enumerate(codes))
                tracks.append({'directory_header': 0, 'track_index': index, 'entry': start})
            data = {'profile': 'synthetic', 'extractor': 1, 'decoder': 1, 'resources': [],
                    'audio': {'path': 'audio', 'waves': waves, 'cgb_waves': [], 'instruments': {
                        'tones': tones, 'banks': [{'source_address': 0x08001000,
                        'programs': [{'selector': i, 'tone': i} for i in range(3)]}], 'map_links': []}},
                    'pcm_pitch': {'keys': [{'base': 0, 'shift': 0}] * 180, 'bases': [0x80000000] * 12},
                    'sequence_directory': {'songs': [{'id': 0, 'header_id': 0}],
                                           'headers': [{'id': 0, 'tone_bank_address': 0x08001000}]},
                    'sequences': {'path': 'sequence', 'clocks': [0, 4, 10] + [1] * 46,
                                  'tracks': tracks, 'instructions': instructions}}
            template = (repo / 'view_rom_data.html').read_text()
            page = work / 'fixed-pcm.html'
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
                  window.AudioContext=FakeContext;q('#sequence-audio>details').open=true;
                  q('#sequence-use-tempo').checked=false;change(q('#sequence-use-tempo'));
                  q('#sequence-rate').value='10';q('#sequence-rate').dispatchEvent(new Event('input'));
                  q('#sequence-mute-cgb').checked=true;change(q('#sequence-mute-cgb'));
                  if(q('#sequence-fixed-rate-control').hidden || q('#sequence-fixed-rate').value!=='15768' ||
                     !q('#sequence-note-rows').textContent.includes('Fixed-rate PCM · sample 0') ||
                     !q('#sequence-status').textContent.includes('3 PCM, 0 CGB, 1 sine fallback'))throw Error('mapping');
                  q('#sequence-play').click();await wait(()=>sources.length===4);
                  for(let i=0;i<2;i++){
                    const n=sources[i];
                    if(n.kind!=='sample' || !n.loop || Math.abs(n.playbackRate.value-15768/22050)>1e-10 || n.playbackRate.events.length ||
                       Math.abs(n.loopStart-1/22050)>1e-12 || n.buffer.samples[1]!==.25 || n.output.output===gains[0])throw Error('fixed source '+i);
                  }
                  if(sources[2].kind!=='sample' || !sources[2].playbackRate.events.length || sources[3].kind!=='sine' ||
                     gains[0].gain.value!==0 || gains[1].gain.events.length<=4)throw Error('normal pitch, fallback, mute or envelope');
                  q('#sequence-fixed-rate').value='22050';q('#sequence-fixed-rate').dispatchEvent(new Event('input'));
                  q('#sequence-track').value='1';change(q('#sequence-track'));
                  if(q('#sequence-fixed-rate').value!=='22050' || !q('#sequence-mute-cgb').checked)throw Error('settings');
                  q('#sequence-play').click();await wait(()=>sources.length===5);
                  if(sources[4].playbackRate.value!==1 || sources[4].playbackRate.events.length)throw Error('adjusted fixed rate');
                  q('#sequence-fixed-rate').value='999';q('#sequence-fixed-rate').dispatchEvent(new Event('input'));
                  if(!q('#sequence-play').disabled || !q('#sequence-status').textContent.includes('1000 to 96000'))throw Error('rate guard');
                  q('#sequence-voice').value='sine';change(q('#sequence-voice'));
                  if(q('#sequence-play').disabled)throw Error('sine mode blocked by fixed rate');
                  q('#sequence-voice').value='pcm';change(q('#sequence-voice'));
                  if(!q('#sequence-play').disabled)throw Error('fixed rate guard lost');
                  q('#sequence-track').value='2';change(q('#sequence-track'));
                  if(q('#sequence-play').disabled || !q('#sequence-note-rows').textContent.includes('PCM 0'))throw Error('unrelated PCM track blocked');
                  return 'PASS';
                })()'''), 'PASS')
            finally:
                browser.close()
