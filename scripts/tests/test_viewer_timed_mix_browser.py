"""Exercise simultaneous, bounded timed-sound auditions in the browser."""
import json
import shutil
import tempfile
import unittest
from pathlib import Path

from test_viewer_instruments_browser import fixture
from viewer_browser_harness import Chrome


def page_with_schedule(work):
    page = fixture(work)
    text = page.read_text()
    start = text.index('const data = ') + len('const data = ')
    data, length = json.JSONDecoder().raw_decode(text[start:])
    data['sequence_directory'] = {
        'songs': [{'id': 5, 'header_id': 5}, {'id': 6, 'header_id': 6},
                  {'id': 7, 'header_id': 5}, {'id': 7, 'header_id': 6}],
        'headers': [{'id': 5, 'tone_bank_address': 0x08001000},
                    {'id': 6, 'tone_bank_address': 0x08001000}],
    }
    codes = [['parameter', 0xbd, 0], ['note', 1, 60, 100, None], ['wait', 1], ['end'],
             ['parameter', 0xbd, 2], ['note', 1, 60, 100, None], ['wait', 1], ['end']]
    data['sequences']['instructions'] = [
        {'source_offset': i, 'next': None if code[0] == 'end' else i + 1, 'code': code}
        for i, code in enumerate(codes)
    ]
    data['sequences']['tracks'] = [
        {'directory_header': 5, 'track_index': 0, 'entry': 0},
        {'directory_header': 6, 'track_index': 0, 'entry': 4},
    ]
    def row(tick, first, second=0):
        return {'tick': tick, 'channel': 3, 'opaque03': 0, 'sound_words': [first, second]}
    data['timed_sound_sources'] = {'path': 'audio-directory.json', 'groups': [
        {'kind': 'rows', 'origin': 'test rows', 'source_offset': 0x100,
         'values': [row(30, 5), row(0, 0x2005, 6), row(50, 7, 0x0fff), row(12000, 5)]},
        {'kind': 'rows', 'origin': 'bounded rows', 'source_offset': 0x200,
         'values': [row(0, 5) for _ in range(66)]},
        {'kind': 'literal', 'origin': 'test literal', 'source_offset': 0x500, 'values': [0]},
    ]}
    page.write_text(text[:start] + json.dumps(data) + text[start + length:])
    return page


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class TimedMixBrowserTest(unittest.TestCase):
    def test_overlap_timing_limits_and_live_mutes(self):
        with tempfile.TemporaryDirectory(prefix='viewer-timed-mix-') as directory:
            work = Path(directory)
            browser = Chrome(work, page_with_schedule(work))
            try:
                result = browser.evaluate('''(async()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change')),
                    input=e=>e.dispatchEvent(new Event('input'));
                  const wait=async f=>{const end=Date.now()+3000;while(!f()){if(Date.now()>end)throw Error('timeout');await new Promise(r=>setTimeout(r,10))}};
                  window.sources=[];window.gains=[];window.contexts=[];
                  class Param {constructor(v=0){this.value=v} setValueAtTime(v){this.value=v} linearRampToValueAtTime(v){this.value=v}}
                  class Node {connect(target){this.output=target} disconnect(){this.disconnected=true}}
                  class FakeContext {
                    constructor(){this.currentTime=0;this.destination={};contexts.push(this)} resume(){return Promise.resolve()}
                    createBuffer(c,n,r){return {getChannelData(){return new Float32Array(n)}}}
                    createGain(){const n=new Node();n.gain=new Param(1);gains.push(n);return n}
                    createBufferSource(){const n=new Node();n.playbackRate=new Param(1);n.start=t=>n.started=t;n.stop=t=>n.stopped=t;sources.push(n);return n}
                    createOscillator(){const n=new Node();n.frequency=new Param(0);n.start=t=>n.started=t;n.stop=t=>n.stopped=t;sources.push(n);return n}
                  }
                  window.AudioContext=FakeContext;
                  q('#timed-sounds>details').open=true;q('#timed-sound-audition').open=true;
                  await new Promise(r=>setTimeout(r,20));
                  q('#sequence-use-tempo').checked=false;change(q('#sequence-use-tempo'));
                  q('#sequence-rate').value='10';input(q('#sequence-rate'));
                  q('#sequence-mute-cgb').checked=true;change(q('#sequence-mute-cgb'));
                  const mute=q('#sequence-track-mutes input[data-track-index="0"]');mute.checked=true;change(mute);
                  q('#timed-sound-duration').value='1';input(q('#timed-sound-duration'));
                  q('#timed-sound-play').click();await wait(()=>sources.length===2);
                  if(Math.abs(sources[0].started-.05)>1e-8 || Math.abs(sources[1].started-.05)>1e-8 ||
                     !q('#timed-sound-play-status').textContent.includes('2 unavailable or ambiguous') ||
                     !q('#timed-sound-play-status').textContent.includes('1 beyond 180 seconds'))throw Error('simultaneous starts or skipped candidates');
                  const pcm=sources[0].output.output,cgb=sources[1].output.output,bus=gains[0];
                  if(pcm.gain.value!==0 || cgb.gain.value!==0 || bus.gain.value!==0 || cgb.output!==bus)throw Error('initial mutes');
                  mute.checked=false;change(mute);q('#sequence-mute-cgb').checked=false;change(q('#sequence-mute-cgb'));
                  if(pcm.gain.value!==1 || cgb.gain.value!==1 || bus.gain.value!==1)throw Error('live mutes');
                  contexts[0].currentTime=.4;await wait(()=>sources.length===3);
                  if(Math.abs(sources[2].started-.55)>1e-8)throw Error('stored tick delay');
                  contexts[0].currentTime=.6;await new Promise(r=>setTimeout(r,130));
                  if(Number(q('#timed-sound-progress').value)<.5)throw Error('playback position');
                  sources.forEach(node=>node.onended());
                  if(!q('#timed-sound-stop').disabled || !q('#timed-sound-play-status').textContent.includes('finished'))throw Error('completion');
                  q('#timed-sound-rate').value='0';input(q('#timed-sound-rate'));q('#timed-sound-play').click();
                  if(!q('#timed-sound-play-status').textContent.includes('schedule rate'))throw Error('invalid rate');
                  q('#timed-sound-rate').value='60';input(q('#timed-sound-rate'));
                  q('#timed-sound-source').value='1';change(q('#timed-sound-source'));
                  q('#timed-sound-play').click();await wait(()=>sources.length===67);
                  if(!q('#timed-sound-play-status').textContent.includes('2 beyond the 64-start limit'))throw Error('start limit');
                  q('#timed-sound-source').value='2';change(q('#timed-sound-source'));
                  if(!q('#timed-sound-audition').hidden || !q('#timed-sound-stop').disabled || !sources[3].output.output.disconnected)throw Error('source change stops mix');
                  return 'PASS';
                })()''')
                self.assertEqual(result, 'PASS')
                browser.evaluate('''(()=>{const e=document.querySelector('#timed-sound-source');
                  for(let p=e.parentElement;p;p=p.parentElement)if(p.tagName==='DETAILS')p.open=true;
                  e.value='0';e.dispatchEvent(new Event('change'));e.focus()})()''')
                self.assertEqual(browser.evaluate('document.activeElement.id'), 'timed-sound-source')
                browser.click('#timed-sound-play')
                self.assertEqual(browser.evaluate('document.activeElement.id'), 'timed-sound-source')
            finally:
                browser.close()
