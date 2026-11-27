"""Browse conditional descriptors, linked packed fields and masks."""
import shutil
import tempfile
import unittest
from pathlib import Path

from test_viewer_conditional_lookups import fixture as unverified_fixture, fixture_catalogs
from test_viewer_scoped_browser import REPO, prepared
from view_rom_data import write_split_viewer
from viewer_browser_harness import Chrome


def verified_fixture(work):
    data, index = prepared(work)
    catalogs = fixture_catalogs()
    for key, value in catalogs.items():
        value['path'] = key + '.json'; value['reader_verified'] = True
    data['conditional_lookups'] = catalogs
    data['resources'].append({'id': 'gameplay.conditional-descriptor-banks', 'path': 'banks.json',
                              'format': 'json', 'description': 'Descriptor banks', 'status': 'Available'})
    index = write_split_viewer(data, index.parent, index.name, (REPO / 'view_rom_data.html').read_text())
    return index.parent / (index.name + '.parts') / 'world.html'


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class ConditionalLookupBrowserTest(unittest.TestCase):
    def test_tables_navigation_filtering_and_source_links(self):
        with tempfile.TemporaryDirectory(prefix='viewer-conditional-lookup-browser-') as directory:
            work = Path(directory); browser = Chrome(work, verified_fixture(work))
            try:
                self.assertTrue(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change'));
                  if(q('#conditional-lookups').hidden || q('#conditional-lookups>details').open ||
                     !q('[data-resource-id="gameplay.conditional-descriptor-banks"] a[href="#conditional-lookups"]'))throw Error('section or link');
                  q('#conditional-lookups>details').open=true;q('#conditional-lookup-descriptors').open=true;
                  if(q('#conditional-lookup-bank').options.length!==24 || q('#conditional-lookup-rows').rows.length!==15 ||
                     q('#conditional-lookup-rows').rows[0].cells[0].textContent!=='1' ||
                     q('#conditional-lookup-rows').rows[0].cells[1].textContent!=='0x6DF53C')throw Error('bank zero');
                  q('#conditional-lookup-bank').value='1';change(q('#conditional-lookup-bank'));
                  if(q('#conditional-lookup-rows').rows.length!==25 || !q('#conditional-lookup-page').textContent.includes('page 1 of 2'))throw Error('pagination');
                  q('#conditional-lookup-next').click();if(q('#conditional-lookup-rows').rows[0].cells[0].textContent!=='25')throw Error('next page');
                  q('#conditional-lookup-filter').value='0x6df5bc';q('#conditional-lookup-filter').dispatchEvent(new Event('input'));
                  if(q('#conditional-lookup-rows').rows.length!==1 || q('#conditional-lookup-rows').rows[0].cells[0].textContent!=='1')throw Error('filter');
                  q('#conditional-lookup-rows').rows[0].querySelector('a').click();
                  const cells=q('#conditional-packed-rows').rows[0].cells;
                  if(!q('#conditional-packed-details').open || q('#conditional-packed-index').value!=='2' ||
                     cells[3].textContent!=='0x61' || cells[6].textContent!=='0x46')throw Error('packed link or bit reconstruction');
                  q('#conditional-four-index').value='73';change(q('#conditional-four-index'));
                  const pair=q('#conditional-four-rows').rows[0].cells;
                  if(pair[0].textContent!=='73 (0x0049)' || pair[1].textContent!=='65462 (0xFFB6)')throw Error('four byte fields');
                  const mask=q('#conditional-mask-rows').rows[0].cells;
                  if(mask[2].textContent!=='0x80000001' || mask[3].textContent!=='0, 31' || q('#conditional-mask-rows').rows.length!==35)throw Error('mask bits');
                  return true;
                })()'''))
            finally:
                browser.close()

    def test_unverified_sources_do_not_offer_inferred_links(self):
        with tempfile.TemporaryDirectory(prefix='viewer-conditional-lookup-unverified-') as directory:
            work = Path(directory); index, _ = unverified_fixture(work)
            browser = Chrome(work, index.parent / (index.name + '.parts') / 'world.html')
            try:
                self.assertTrue(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s);q('#conditional-lookup-bank').value='1';
                  q('#conditional-lookup-bank').dispatchEvent(new Event('change'));
                  return !q('#conditional-lookup-rows a') && q('#conditional-lookup-rows').rows[0].cells[7].textContent==='Not linked' &&
                    q('#conditional-lookup-source').textContent.includes('not authenticated');
                })()'''))
            finally:
                browser.close()


if __name__ == '__main__':
    unittest.main()
