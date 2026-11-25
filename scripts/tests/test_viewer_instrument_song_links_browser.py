"""Check the stored song-to-instrument-bank links in both directions."""
import json
import shutil
import tempfile
import unittest
from pathlib import Path

from test_viewer_instruments_browser import fixture
from viewer_browser_harness import Chrome


def page_with_directory(work, include_sequences=True, include_directory=True):
    page = fixture(work)
    text = page.read_text()
    start = text.index('const data = ') + len('const data = ')
    data, length = json.JSONDecoder().raw_decode(text[start:])
    data['sequence_directory'] = {
        'headers': [{'id': 0, 'tone_bank_address': 0x08001000},
                    {'id': 1, 'tone_bank_address': 0x08001100},
                    {'id': 2, 'tone_bank_address': 0x08009999},
                    {'id': 3, 'tone_bank_address': None}],
        'songs': ([{'id': 100 + i, 'header_id': 0} for i in range(53)] +
                  [{'id': 100, 'header_id': 1}, {'id': 200, 'header_id': 2},
                   {'id': 201, 'header_id': 3}, {'id': 202, 'header_id': 999}])}
    if not include_sequences:
        del data['sequences']
    if not include_directory:
        del data['sequence_directory']
        data.pop('sequences', None)
    page.write_text(text[:start] + json.dumps(data) + text[start + length:])
    return page


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class ViewerInstrumentSongLinksTest(unittest.TestCase):
    def test_bank_song_pagination_and_navigation(self):
        with tempfile.TemporaryDirectory(prefix='viewer-song-banks-') as directory:
            work = Path(directory)
            browser = Chrome(work, page_with_directory(work))
            try:
                result = browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change',{bubbles:true}));
                  const rows=()=>[...q('#instrument-songs-rows').rows];
                  if(q('#instrument-songs').hidden || rows().length!==25 ||
                     !q('#instrument-songs-status').textContent.includes('1–25 of 53') ||
                     !q('#instrument-songs-prev').disabled || q('#instrument-songs-next').disabled ||
                     q('#sequence-instrument-navigation').hidden)throw Error('initial associations');
                  if(!q('#instrument-songs').textContent.includes('does not establish'))throw Error('association qualification');
                  q('#instrument-key').value='61';change(q('#instrument-key'));
                  q('#sequence-mute-cgb').checked=true;change(q('#sequence-mute-cgb'));
                  q('#instrument-songs-next').click();
                  if(rows().length!==25 || rows()[0].cells[0].textContent!=='125' ||
                     !q('#instrument-songs-status').textContent.includes('26–50 of 53'))throw Error('second page');
                  rows()[1].cells[0].querySelector('a').click();
                  if(q('#sequence-song').value!=='26' || !q('#sequence-audio>details').open ||
                     q('#instrument-key').value!=='61' || !q('#sequence-mute-cgb').checked)throw Error('song navigation');
                  q('#instrument-songs-next').click();
                  if(rows().length!==3 || !q('#instrument-songs-next').disabled ||
                     rows()[2].cells[0].textContent!=='152')throw Error('last page');
                  q('#instrument-songs-prev').click();if(rows()[0].cells[0].textContent!=='125')throw Error('previous page');
                  q('#instrument-program').value='4';change(q('#instrument-program'));
                  q('#sequence-instrument-open').click();
                  if(q('#instrument-program').value!=='4' || !q('#instruments>details').open)throw Error('same bank reset');
                  q('#sequence-song').value='53';change(q('#sequence-song'));
                  if(q('#sequence-instrument-navigation').hidden)throw Error('second bank link');
                  q('#sequence-instrument-open').click();
                  if(q('#instrument-bank').value!=='1' || rows().length!==1 || rows()[0].cells[0].textContent!=='100' ||
                     rows()[0].cells[1].textContent!=='1' || q('#instrument-key').value!=='61' ||
                     !q('#instrument-mute-cgb').checked)throw Error('reverse navigation or duplicate ID');
                  for(const index of ['54','55','56']){
                    q('#sequence-song').value=index;change(q('#sequence-song'));
                    if(!q('#sequence-instrument-navigation').hidden)throw Error('unmapped bank '+index);
                  }
                  return 'PASS';
                })()''')
                self.assertEqual(result, 'PASS')
            finally:
                browser.close()

    def test_missing_directory_or_sequence_preview(self):
        with tempfile.TemporaryDirectory(prefix='viewer-song-banks-missing-') as directory:
            work = Path(directory) / 'missing-preview'
            work.mkdir()
            browser = Chrome(work, page_with_directory(work, include_sequences=False))
            try:
                self.assertTrue(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s);
                  return !q('#instrument-songs').hidden && q('#instrument-songs-rows').rows.length===25 &&
                    !q('#instrument-songs-rows a') && q('#sequence-instrument-navigation').hidden;
                })()'''))
            finally:
                browser.close()
            work = Path(directory) / 'missing-directory'
            work.mkdir()
            browser = Chrome(work, page_with_directory(work, include_directory=False))
            try:
                self.assertTrue(browser.evaluate("document.querySelector('#instrument-songs').hidden && !document.querySelector('#instruments').hidden"))
            finally:
                browser.close()
