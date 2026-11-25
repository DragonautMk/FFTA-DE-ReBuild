"""Check live per-track mute alongside the sequence CGB bus."""
import json
import shutil
import tempfile
import unittest
from pathlib import Path

from test_viewer_instruments_browser import fixture
from viewer_browser_harness import Chrome


def page_with_tracks(work):
    page = fixture(work)
    text = page.read_text()
    start = text.index('const data = ') + len('const data = ')
    data, length = json.JSONDecoder().raw_decode(text[start:])
    codes = [['parameter', 0xbd, 0], ['note', 2, 60, 100, None],
             ['parameter', 0xbd, 3], ['note', 2, 60, 100, None], ['wait', 1], ['end'],
             ['parameter', 0xbd, 0], ['note', 2, 60, 100, None], ['wait', 1], ['end']]
    data['sequences']['instructions'] = [
        {'source_offset': i, 'next': None if code[0] == 'end' else i + 1, 'code': code}
        for i, code in enumerate(codes)]
    data['sequences']['tracks'] = [{'directory_header': 0, 'track_index': 0, 'entry': 0},
                                   {'directory_header': 0, 'track_index': 1, 'entry': 6},
                                   {'directory_header': 1, 'track_index': 0, 'entry': 6}]
    data['sequence_directory'] = {'songs': [{'id': i, 'header_id': i} for i in range(3)],
                                  'headers': [{'id': i, 'tone_bank_address': 0x08001000} for i in range(3)]}
    page.write_text(text[:start] + json.dumps(data) + text[start + length:])
    return page


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class ViewerTrackMuteTest(unittest.TestCase):
    def test_live_mute_tracks_and_cgb_independently(self):
        with tempfile.TemporaryDirectory(prefix='viewer-track-mute-') as directory:
            work = Path(directory)
            browser = Chrome(work, page_with_tracks(work))
            try:
                result = browser.evaluate('''(async()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change'));
                  const mute=i=>q('#sequence-track-mutes input[data-track-index="'+i+'"]');
                  const wait=async f=>{const end=Date.now()+3000;while(!f()){if(Date.now()>end)throw Error('timeout');await new Promise(r=>setTimeout(r,10))}};
                  window.sources=[];window.gains=[];window.contexts=[];
                  class Param {constructor(v=0){this.value=v} setValueAtTime(v){this.value=v} linearRampToValueAtTime(v){this.value=v}}
                  class Node {connect(target){this.output=target} disconnect(){this.disconnected=true}}
                  class FakeContext {
                    constructor(){this.currentTime=0;this.destination={};contexts.push(this)} resume(){return Promise.resolve()}
                    createBuffer(c,n,r){return {getChannelData(){return new Float32Array(n)}}}
                    createGain(){const n=new Node();n.gain=new Param(1);gains.push(n);return n}
                    createBufferSource(){const n=new Node();n.playbackRate=new Param(1);n.start=()=>{};n.stop=()=>{};sources.push(n);return n}
                    createOscillator(){const n=new Node();n.frequency=new Param(0);n.start=()=>{};n.stop=()=>{};sources.push(n);return n}
                  }
                  window.AudioContext=FakeContext;
                  q('#sequence-use-tempo').checked=false;change(q('#sequence-use-tempo'));
                  q('#sequence-rate').value='10';q('#sequence-rate').dispatchEvent(new Event('input'));
                  if(q('#sequence-track-mixer').hidden || q('#sequence-track-mixer').open || !mute(0) || !mute(1))throw Error('mixer controls');
                  mute(0).checked=true;change(mute(0));
                  if(!q('#sequence-note-rows').textContent.includes('track muted'))throw Error('muted note label');
                  q('#sequence-play').click();await wait(()=>sources.length===3);
                  const pcm=sources[0].output.output,cgb=sources[1].output.output,other=sources[2].output.output,bus=gains[0];
                  if(pcm.gain.value!==0 || cgb.gain.value!==0 || other.gain.value!==1 ||
                     pcm.output!==contexts[0].destination || cgb.output!==bus || bus.gain.value!==1)throw Error('initial routes');
                  mute(0).checked=false;change(mute(0));
                  if(pcm.gain.value!==1 || cgb.gain.value!==1 || other.gain.value!==1 || q('#sequence-stop').disabled)throw Error('live unmute');
                  q('#sequence-mute-cgb').checked=true;change(q('#sequence-mute-cgb'));
                  mute(0).checked=true;change(mute(0));mute(0).checked=false;change(mute(0));
                  if(bus.gain.value!==0 || pcm.gain.value!==1 || cgb.gain.value!==1)throw Error('independent CGB mute');
                  q('#sequence-track-mute-all').click();if(pcm.gain.value!==0 || cgb.gain.value!==0 || other.gain.value!==0)throw Error('mute all');
                  q('#sequence-track-unmute-all').click();if(pcm.gain.value!==1 || cgb.gain.value!==1 || other.gain.value!==1)throw Error('unmute all');
                  mute(1).checked=true;change(mute(1));q('#sequence-track').value='1';change(q('#sequence-track'));
                  if(!mute(1).checked || !q('#sequence-mute-cgb').checked || !pcm.disconnected)throw Error('track selection persistence');
                  q('#sequence-play').click();await wait(()=>sources.length===4);
                  if(sources[3].output.output.gain.value!==0)throw Error('muted selected track');
                  mute(0).checked=true;change(mute(0));q('#sequence-song').value='1';change(q('#sequence-song'));
                  if(!mute(0).checked || !q('#sequence-mute-cgb').checked)throw Error('song persistence');
                  q('#sequence-song').value='2';change(q('#sequence-song'));
                  if(!q('#sequence-track-mixer').hidden)throw Error('empty song');
                  q('#sequence-song').value='0';change(q('#sequence-song'));
                  if(!mute(0).checked || !mute(1).checked)throw Error('restored slot mute');
                  q('#sequence-stop').click();return 'PASS';
                })()''')
                self.assertEqual(result, 'PASS')
            finally:
                browser.close()
