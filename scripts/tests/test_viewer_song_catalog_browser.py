"""Check the bounded, searchable extracted-song directory."""
import shutil
import tempfile
import unittest
from pathlib import Path

from test_viewer_instrument_song_links_browser import page_with_directory
from viewer_browser_harness import Chrome


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class ViewerSongCatalogTest(unittest.TestCase):
    def test_paging_filtering_and_song_selection(self):
        with tempfile.TemporaryDirectory(prefix='viewer-song-catalog-') as directory:
            work = Path(directory)
            browser = Chrome(work, page_with_directory(work))
            try:
                result = browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s), rows=()=>[...q('#sequence-catalog-rows').rows];
                  const input=value=>{q('#sequence-catalog-filter').value=value;q('#sequence-catalog-filter').dispatchEvent(new Event('input'));};
                  if(q('#sequence-catalog').open || rows().length!==25 ||
                     !q('#sequence-catalog-status').textContent.includes('1–25 of 57') ||
                     !q('#sequence-catalog-prev').disabled || q('#sequence-catalog-next').disabled)throw Error('initial page');
                  if(rows()[0].cells[0].textContent!=='100' || rows()[0].cells[1].textContent!=='0' ||
                     rows()[0].cells[2].textContent!=='1' || rows()[0].cells[3].textContent!=='0x8001000')throw Error('stored row');
                  q('#sequence-catalog-next').click();
                  if(rows()[0].cells[0].textContent!=='125' || rows().length!==25)throw Error('second page');
                  q('#sequence-catalog-next').click();
                  if(rows().length!==7 || !q('#sequence-catalog-next').disabled)throw Error('last page');
                  q('#sequence-catalog-prev').click();if(rows()[0].cells[0].textContent!=='125')throw Error('previous page');
                  q('#sequence-mute-cgb').checked=true;q('#sequence-mute-cgb').dispatchEvent(new Event('change'));
                  input('100');
                  if(rows().length!==2 || rows()[0].cells[1].textContent!=='0' || rows()[1].cells[1].textContent!=='1')throw Error('duplicate IDs');
                  rows()[1].querySelector('button').click();
                  if(q('#sequence-song').value!=='53' || q('#sequence-instrument-navigation').hidden ||
                     !q('#sequence-mute-cgb').checked || !q('#sequence-stop').disabled)throw Error('selection or mute');
                  input('999');
                  if(rows().length!==1 || rows()[0].cells[0].textContent!=='202' ||
                     rows()[0].cells[2].textContent!=='0' || rows()[0].cells[3].textContent!=='Unavailable')throw Error('missing header');
                  input('not a song');if(rows().length || !q('#sequence-catalog-status').textContent.includes('No matching'))throw Error('empty search');
                  input('');if(rows().length!==25 || !q('#sequence-catalog-status').textContent.includes('1–25 of 57'))throw Error('clear filter');
                  return 'PASS';
                })()''')
                self.assertEqual(result, 'PASS')
            finally:
                browser.close()
