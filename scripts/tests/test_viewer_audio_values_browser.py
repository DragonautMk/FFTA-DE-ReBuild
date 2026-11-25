"""Check bounded inspection of stored PCM and CGB waveform values."""
import json
import shutil
import tempfile
import unittest
from pathlib import Path

from test_viewer_instruments_browser import fixture
from viewer_browser_harness import Chrome


def page_with_samples(work):
    page = fixture(work)
    text = page.read_text()
    start = text.index('const data = ') + len('const data = ')
    data, length = json.JSONDecoder().raw_decode(text[start:])
    wave = data['audio']['waves'][0]
    wave.update({'header_type': 0x12, 'flags': 0x4000, 'frequency_word': 22050,
                 'loop_start': 70, 'declared_length': 150, 'interpolation_endpoint': -3,
                 'samples': [-128, 127] + [index % 256 - 128 for index in range(148)]})
    data['audio']['waves'].append({'id': 12, 'source_offset': 0x6000, 'tone_ids': [],
                                   'frequency_word': 22050, 'loop_start': 0, 'flags': 0,
                                   'samples': []})
    page.write_text(text[:start] + json.dumps(data) + text[start + length:])
    return page


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class ViewerAudioValuesTest(unittest.TestCase):
    def test_pcm_headers_pages_canvas_and_cgb_amplitudes(self):
        with tempfile.TemporaryDirectory(prefix='viewer-audio-values-') as directory:
            work = Path(directory)
            browser = Chrome(work, page_with_samples(work))
            try:
                result = browser.evaluate('''(async()=>{
                  const q=s=>document.querySelector(s), change=e=>e.dispatchEvent(new Event('change'));
                  const rows=()=>[...q('#audio-sample-rows').rows];
                  const wait=async f=>{const end=Date.now()+3000;while(!f()){if(Date.now()>end)throw Error('audio timeout');await new Promise(r=>setTimeout(r,10))}};
                  if(q('#audio-values').open || q('#cgb-values').open || rows().length!==64 ||
                     rows()[0].cells[1].textContent!=='-128' || rows()[0].cells[2].textContent!=='0x80' ||
                     rows()[1].cells[1].textContent!=='127' || rows()[1].cells[2].textContent!=='0x7F')throw Error('initial PCM values');
                  const headers=Object.fromEntries([...q('#audio-header-rows').rows].map(r=>[r.cells[0].textContent,r.cells[1].textContent]));
                  if(headers['Header type']!=='0x12' || headers.Flags!=='0x4000' || headers['Loop start']!=='70' ||
                     headers['Declared length']!=='150' || headers['Interpolation endpoint']!=='-3 (0xFD)')throw Error('PCM header');
                  q('#audio-sample-next').click();if(rows()[0].cells[0].textContent!=='64')throw Error('second page');
                  q('#audio-sample-next').click();if(rows().length!==22 || !q('#audio-sample-next').disabled)throw Error('last page');
                  q('#audio-sample-prev').click();if(rows()[0].cells[0].textContent!=='64')throw Error('previous page');
                  q('#audio-sample-index').value='99';q('#audio-sample-go').click();
                  if(rows()[0].cells[0].textContent!=='64')throw Error('index jump');
                  q('#audio-sample-index').value='150';q('#audio-sample-go').click();
                  if(!q('#audio-sample-status').textContent.includes('0 to 149'))throw Error('invalid index');
                  q('#audio-sample-index').value='';q('#audio-sample-go').click();
                  if(!q('#audio-sample-status').textContent.includes('0 to 149'))throw Error('empty index');
                  const cgb=[...q('#cgb-sample-rows').rows];
                  if(cgb.length!==32 || cgb[0].cells[1].textContent!=='0' || cgb[1].cells[2].textContent!=='0xF' ||
                     !q('#cgb-source').textContent.includes('0x3000'))throw Error('CGB values');
                  q('#audio>details').open=true;q('#audio-loop').checked=true;q('#audio-play').click();await wait(()=>!q('#audio-stop').disabled);
                  const canvas=q('#waveform'),rect=canvas.getBoundingClientRect();
                  canvas.dispatchEvent(new MouseEvent('click',{clientX:rect.left+rect.width*(75.5/150)}));
                  if(!q('#audio-values').open || q('#audio-sample-index').value!=='75' || rows()[0].cells[0].textContent!=='64' ||
                     q('#audio-stop').disabled)throw Error('canvas selection or playback');
                  q('#audio-stop').click();q('#audio-id').value='2';change(q('#audio-id'));
                  if(rows().length || !q('#audio-sample-go').disabled || !q('#audio-sample-next').disabled ||
                     !q('#audio-sample-status').textContent.includes('No stored samples'))throw Error('empty wave');
                  return 'PASS';
                })()''')
                self.assertEqual(result, 'PASS')
            finally:
                browser.close()
