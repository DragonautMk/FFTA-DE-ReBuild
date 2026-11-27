"""Check estimated shared song tempo, timing and manual fallback in Chrome."""
import json
import shutil
import tempfile
import unittest
from pathlib import Path

from viewer_browser_harness import Chrome


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class ViewerSequenceTempoTest(unittest.TestCase):
    def test_shared_tempo_changes_pause_and_manual_fallback(self):
        with tempfile.TemporaryDirectory(prefix='viewer-sequence-tempo-') as directory:
            work = Path(directory)
            repo = Path(__file__).resolve().parents[2]
            codes = [
                ['parameter', 0xbb, 40], ['wait', 1], ['pattern', 5], ['wait', 1], ['end'],
                ['parameter', 0xbb, 120], ['return'],
                ['note', 2, 60, 100, None], ['wait', 1], ['note', 1, 64, 100, None], ['wait', 1], ['end'],
                ['parameter', 0xbb, 60], ['wait', 1], ['parameter', 0xbb, 0], ['end'],
                ['note', 2, 60, 100, None], ['wait', 1], ['note', 2, 64, 100, None], ['wait', 1],
                ['note', 1, 67, 100, None], ['end'],
            ]
            ends = {4, 6, 11, 15, 21}
            instructions = [{'source_offset': i, 'next': None if i in ends else i + 1, 'code': code}
                            for i, code in enumerate(codes)]
            data = {'profile': 'synthetic', 'extractor': 1, 'decoder': 1, 'resources': [],
                    'sequence_directory': {'songs': [{'id': 0, 'header_id': 0}, {'id': 1, 'header_id': 1}],
                                           'headers': [{'id': 0}, {'id': 1}]},
                    'sequences': {'path': 'sequence', 'clocks': [0, 4, 8] + [1] * 46,
                                  'tracks': [{'directory_header': 0, 'track_index': 0, 'entry': 0},
                                             {'directory_header': 0, 'track_index': 1, 'entry': 7},
                                             {'directory_header': 1, 'track_index': 0, 'entry': 12},
                                             {'directory_header': 1, 'track_index': 1, 'entry': 16}],
                                  'instructions': instructions}}
            template = (repo / 'view_rom_data.html').read_text()
            page = work / 'tempo.html'
            page.write_text(template.replace('/* DATA */', json.dumps(data)))
            browser = Chrome(work, page)
            try:
                result = browser.evaluate('''(async()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change'));
                  const wait=async fn=>{let end=Date.now()+3000;while(!fn()){if(Date.now()>end)throw Error('timeout');await new Promise(r=>setTimeout(r,10))}};
                  const slow=estimatedTempoRate(40),fast=estimatedTempoRate(120),boundary=4/slow;
                  const map=estimatedTempoTimeline([{tick:0,track:0,order:0,value:40},{tick:4,track:0,order:1,value:120}],48,12);
                  if(Math.abs(map.timeAt(4)-boundary)>1e-10 || Math.abs(map.timeAt(8)-(boundary+4/fast))>1e-10)throw Error('tempo map');
                  const same=estimatedTempoTimeline([{tick:4,track:1,order:0,value:60},{tick:4,track:0,order:0,value:0}],48,12);
                  if(same.paused || !Number.isFinite(same.timeAt(5)))throw Error('same tick ordering');
                  const paused=estimatedTempoTimeline([{tick:4,track:0,order:0,value:0}],48,12);
                  if(!paused.paused || !Number.isFinite(paused.timeAt(4)) || Number.isFinite(paused.timeAt(5)) || paused.endTick!==5)throw Error('zero tempo');
                  window.sources=[];window.contexts=[];
                  class FakeContext {
                    constructor(){this.currentTime=0;this.destination={};contexts.push(this)} resume(){return Promise.resolve()}
                    createGain(){return {gain:{setValueAtTime(){},linearRampToValueAtTime(){}},connect(){},disconnect(){}}}
                    createOscillator(){const n={frequency:{value:0},connect(){},start(t){this.started=t},stop(t){this.stopped=t}};sources.push(n);return n}
                  }
                  window.AudioContext=FakeContext;q('#sequence-audio>details').open=true;
                  q('#sequence-frame-tempo').checked=false;change(q('#sequence-frame-tempo'));
                  q('#sequence-track').value='1';change(q('#sequence-track'));
                  if(!q('#sequence-use-tempo').checked || !q('#sequence-status').textContent.includes('2 changes') ||
                     !q('#sequence-timeline-detail').textContent.includes('seconds'))throw Error('cross-track tempo');
                  q('#sequence-play').click();await wait(()=>sources.length===2);
                  if(Math.abs((sources[0].stopped-sources[0].started)-(boundary+4/fast))>1e-8 ||
                     Math.abs((sources[1].started-sources[0].started)-boundary)>1e-8)throw Error('scheduled tempo changes');
                  const canvas=q('#sequence-notes'),ctx=canvas.getContext('2d'),x=Math.floor(boundary/12*canvas.width)+1;
                  if(!ctx.getImageData(x,11,1,1).data[3] || ctx.getImageData(5,11,1,1).data[3])throw Error('timeline uses elapsed seconds');
                  contexts[0].currentTime=.05+boundary;await new Promise(r=>setTimeout(r,160));
                  if(Math.abs(parseFloat(q('#sequence-playhead').style.left)-boundary/12*100)>.15)throw Error('playhead matches timeline');
                  q('#sequence-use-tempo').checked=false;change(q('#sequence-use-tempo'));
                  q('#sequence-play').click();await wait(()=>sources.length===4);
                  if(Math.abs((sources[2].stopped-sources[2].started)-8/48)>1e-8 ||
                     Math.abs((sources[3].started-sources[2].started)-4/48)>1e-8)throw Error('manual fallback');
                  q('#sequence-use-tempo').checked=true;change(q('#sequence-use-tempo'));
                  q('#sequence-mute-cgb').checked=true;change(q('#sequence-mute-cgb'));
                  q('#sequence-song').value='1';change(q('#sequence-song'));q('#sequence-track').value='1';change(q('#sequence-track'));
                  if(!q('#sequence-use-tempo').checked || !q('#sequence-mute-cgb').checked ||
                     !q('#sequence-status').textContent.includes('stopped by zero tempo') || q('#sequence-note-rows').rows.length!==2)throw Error('pause and persistent controls');
                  q('#sequence-play').click();await wait(()=>sources.length===6);
                  if(Math.abs((sources[4].stopped-sources[4].started)-12)>1e-8 ||
                     Math.abs((sources[5].stopped-sources[4].started)-12)>1e-8)throw Error('held notes at zero tempo');
                  q('#sequence-stop').click();return 'PASS';
                })()''')
                self.assertEqual(result, 'PASS')
            finally:
                browser.close()
