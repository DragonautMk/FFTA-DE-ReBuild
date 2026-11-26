"""Exercise the conditional region-name directory and its source links."""
import copy
import shutil
import sys
import tempfile
import time
import unittest
from pathlib import Path
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import view_rom_data
from test_viewer_world_names_browser import fixture, generate_fixture, payload
from viewer_browser_harness import Chrome


class WorldRegionProjectionTest(unittest.TestCase):
    def test_catalog_scope_identity_and_partial_languages(self):
        with tempfile.TemporaryDirectory(prefix='viewer-world-regions-projection-') as directory:
            index, digests = generate_fixture(Path(directory))
            parts = index.parent / 'index.html.parts'
            world, text = payload(parts / 'world.html'), payload(parts / 'text.html')
            regions = world['world_region_names']
            self.assertEqual([(row['region_index'], row['name_index']) for row in regions['regions']],
                             [(i, 718 + i) for i in range(31)])
            self.assertEqual(regions['regions'][16]['names'], [[205], None])
            self.assertEqual(regions['regions'][30]['names'][1], None)
            for page in (index, parts / 'text.html', parts / 'audio.html'):
                self.assertNotIn('world_region_names', payload(page))
            with patch.object(view_rom_data, 'EUROPE_TEXT_CATALOG_SHA256', digests[1]):
                project, texts = view_rom_data.project_world_region_names, text['pointer_texts']
                self.assertEqual(project('europe', digests[1], texts), regions)
                self.assertIsNone(project('usa', digests[1], texts))
                self.assertIsNone(project('europe', 'wrong', texts))
                self.assertIsNone(project('europe', digests[1], None))
                changed = copy.deepcopy(texts); changed['catalogs'].append(changed['catalogs'][0])
                self.assertIsNone(project('europe', digests[1], changed))
                changed = copy.deepcopy(texts); changed['catalogs'][0]['reader'] = 0
                self.assertIsNone(project('europe', digests[1], changed))
                changed = copy.deepcopy(texts)
                for language in changed['catalogs'][0]['languages']:
                    language['entries'] = [0]
                partial = project('europe', digests[1], changed)
                self.assertEqual(len(partial['regions']), 31)
                self.assertTrue(all(row['names'] == [None, None] for row in partial['regions']))
                self.assertEqual(partial['glyphs'], [])


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class WorldRegionBrowserTest(unittest.TestCase):
    def test_filter_placement_language_and_source_navigation(self):
        with tempfile.TemporaryDirectory(prefix='viewer-world-regions-browser-') as directory:
            work = Path(directory); index, _ = generate_fixture(work)
            browser = Chrome(work, index.parent / 'index.html.parts/world.html')
            try:
                self.assertTrue(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s), change=e=>e.dispatchEvent(new Event('change')),
                    input=e=>e.dispatchEvent(new Event('input'));
                  if(q('#world-regions').hidden || q('#world-regions>details').open ||
                     q('#world-regions-navigation').hidden || q('#world-region-rows').rows.length!==31 ||
                     q('#world-region-language').options.length!==2 ||
                     !q('#contents a[href="#world-regions"]')) return false;
                  const row=q('#world-region-rows').rows[16];
                  if(row.cells[0].textContent!=='16' || row.cells[1].textContent!=='d' ||
                     !row.cells[2].textContent.includes('Node 2') ||
                     !row.cells[3].querySelector('a').href.includes('job_entry=734') ||
                     [...q('#world-region-rows').rows].filter(item=>item.cells[2].querySelector('a')).length!==6)
                    return false;
                  row.cells[2].querySelector('a').click();
                  if(!q('#world>details').open || !q('#world-detail').textContent.includes('node 2') ||
                     q('#world-selected-name').textContent!=='Observed name: d') return false;
                  q('#world-region-filter').value='734';input(q('#world-region-filter'));
                  if(q('#world-region-rows').rows.length!==1 || q('#world-region-rows').rows[0].cells[0].textContent!=='16')return false;
                  q('#world-region-language').value='1';change(q('#world-region-language'));
                  if(!q('#world-region-rows').rows[0].cells[1].textContent.includes('unavailable') ||
                     q('#world-region-rows').rows[0].cells[3].querySelector('a'))return false;
                  q('#world-region-filter').value='Ab';input(q('#world-region-filter'));
                  if(q('#world-region-rows').rows.length!==11)return false;
                  q('#world-region-filter').value='nothing matches';input(q('#world-region-filter'));
                  if(q('#world-region-rows').rows.length!==0 || !q('#world-region-status').textContent.includes('0 of 31'))return false;
                  q('#world-region-filter').value='7';input(q('#world-region-filter'));
                  const match=[...q('#world-region-rows').rows].find(item=>item.cells[0].textContent==='7');
                  q('#world-regions>details').open=true;
                  const link=match.cells[3].querySelector('a'), target=new URL(link.href);
                  return target.searchParams.get('job_entry')==='725' && target.searchParams.get('job_language')==='1';
                })()'''))
                hit = browser.evaluate('''(()=>{
                  const row=[...document.querySelector('#world-region-rows').rows].find(item=>item.cells[0].textContent==='7');
                  const a=row.cells[3].querySelector('a');
                  for(let p=a.parentElement;p;p=p.parentElement)if(p.tagName==='DETAILS')p.open=true;
                  a.scrollIntoView({block:'center'});
                  const r=a.getClientRects()[0],x=r.left+r.width/2,y=r.top+r.height/2,e=document.elementFromPoint(x,y);
                  return {x,y,match:e===a||a.contains(e)};
                })()''')
                self.assertTrue(hit['match'])
                point = {'x': hit['x'], 'y': hit['y']}
                for event in ({'type': 'mouseMoved'}, {'type': 'mousePressed', 'button': 'left', 'clickCount': 1},
                              {'type': 'mouseReleased', 'button': 'left', 'clickCount': 1}):
                    browser.command('Input.dispatchMouseEvent', {**event, **point})
                deadline = time.monotonic() + 30
                while time.monotonic() < deadline:
                    try:
                        if browser.evaluate("document.readyState==='complete' && typeof data!=='undefined' && data.wiki.current==='text-pointers'"):
                            break
                    except (RuntimeError, OSError):
                        pass
                    time.sleep(.05)
                else:
                    self.fail('Region name source did not load')
                self.assertTrue(browser.evaluate("document.querySelector('#pointer-texts>details').open && "
                                                 "document.querySelector('#pointer-language').value==='1' && "
                                                 "document.querySelector('#pointer-entry').value==='725'"))
            finally:
                browser.close()

    def test_single_page_and_unrecognized_source(self):
        with tempfile.TemporaryDirectory(prefix='viewer-world-regions-single-') as directory:
            work = Path(directory); index, _ = generate_fixture(work, split=False)
            browser = Chrome(work, index)
            try:
                self.assertTrue(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s);q('#world-regions>details').open=true;
                  q('#world-region-rows').rows[16].cells[3].querySelector('a').click();
                  return q('#pointer-texts>details').open && q('#pointer-entry').value==='734';
                })()'''))
            finally:
                browser.close()
        with tempfile.TemporaryDirectory(prefix='viewer-world-regions-unknown-') as directory:
            work = Path(directory); dataset, _ = fixture(work)
            index = view_rom_data.generate(dataset, work / 'view', split=True)
            browser = Chrome(work, index.parent / 'index.html.parts/world.html')
            try:
                self.assertTrue(browser.evaluate("document.querySelector('#world-regions').hidden && "
                                                 "document.querySelector('#world-regions-navigation').hidden"))
            finally:
                browser.close()


if __name__ == '__main__':
    unittest.main()
