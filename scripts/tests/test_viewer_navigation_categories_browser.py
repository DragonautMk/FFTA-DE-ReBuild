"""Check compact offline navigation, categorized resources and deep links."""
import json
import shutil
import sys
import tempfile
import time
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from test_viewer_scoped_browser import prepared, parse_page
from viewer_browser_harness import Chrome


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class ViewerNavigationCategoriesTest(unittest.TestCase):
    def test_compact_index_categories_search_and_directory_inspector(self):
        with tempfile.TemporaryDirectory() as directory:
            work = Path(directory); data, page = prepared(work)
            self.assertLess(page.stat().st_size, 100000)
            self.assertNotIn('obj-animation-play', page.read_text())
            browser = Chrome(work, page)
            try:
                self.assertEqual(browser.evaluate('''(async()=>{
                  const q=s=>document.querySelector(s),qa=s=>[...document.querySelectorAll(s)];
                  const compare=(a,b)=>a.localeCompare(b,'en',{sensitivity:'base',numeric:true});
                  const categories=qa('#rows > details'), nav=qa('#wiki-page-links > details');
                  const labels=nodes=>nodes.map(d=>d.querySelector(':scope > summary').textContent.replace(/ · \\d+$/,''));
                  if(!categories.length || categories.some(d=>d.open) || nav.some(d=>d.open) ||
                     labels(categories).join('|')!==labels(categories).sort(compare).join('|') ||
                     labels(nav).join('|')!==labels(nav).sort(compare).join('|') ||
                     qa('#rows [data-resource-id]').length!==data.resources.length)throw Error('initial categories');
                  for(const category of categories){
                    const children=[...category.querySelectorAll(':scope > details')];
                    if(children.some(d=>d.open) || labels(children).join('|')!==labels(children).sort(compare).join('|'))throw Error('subcategories');
                  }
                  const search=q('#search');search.value='audio.m4a-tones';search.dispatchEvent(new Event('input'));
                  if(qa('#rows [data-resource-id]').length!==1 || !q('#rows > details').open ||
                     !q('#rows > details > details').open || !q('#index-count').textContent.includes('1 of'))throw Error('search');
                  const row=q('#rows [data-resource-id="audio.m4a-tones"]');
                  if(!row || !row.querySelector('a').textContent.includes('Browse Audio'))throw Error('topic link');
                  const file=new File(['{"ok":7}'],'wave.json',{type:'application/json'});
                  Object.defineProperty(file,'webkitRelativePath',{value:'dataset/wave.json'});
                  Object.defineProperty(q('#dataset-directory'),'files',{configurable:true,value:[file]});
                  q('#dataset-directory').dispatchEvent(new Event('change'));
                  const inspect=qa('#rows [data-resource-id="audio.m4a-tones"] a').find(a=>a.textContent==='Inspect selected file');
                  if(!inspect || !q('#dataset-status').textContent.includes('1 of'))throw Error('folder matching');
                  inspect.click();
                  for(let i=0;i<50 && !q('#inspect-content').textContent.includes('"ok"');i++)await new Promise(r=>setTimeout(r,10));
                  if(!q('#file-inspector>details').open || !q('#inspect-content').textContent.includes('"ok"'))throw Error('inspector');
                  search.value='';search.dispatchEvent(new Event('input'));
                  if(qa('#rows > details').some(d=>d.open) || qa('#rows [data-resource-id]').length!==data.resources.length)throw Error('clear search');
                  return 'PASS';
                })()'''), 'PASS')
            finally:
                browser.close()

    def test_topic_groups_navigation_and_bookmarked_selection(self):
        with tempfile.TemporaryDirectory() as directory:
            work = Path(directory); _, page = prepared(work)
            objects = page.parent / (page.name + '.parts/objects.html')
            browser = Chrome(work, objects)
            try:
                self.assertTrue(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s),groups=[...document.querySelectorAll('main > details.section-group')];
                  const names=groups.map(d=>d.dataset.category), compare=(a,b)=>a.localeCompare(b,'en',{sensitivity:'base',numeric:true});
                  return groups.length>0 && names.join('|')===names.slice().sort(compare).join('|') &&
                    groups.every(d=>!d.open) && !q('#resource-index>details').open &&
                    ![...document.querySelectorAll('#wiki-page-links > details')].some(d=>d.open) &&
                    !!q('#contents a[href="#obj-animations"]');
                })()'''))
                browser.click('#contents a[href="#obj-animations"]')
                self.assertTrue(browser.evaluate('''(()=>{
                  const section=document.querySelector('#obj-animations');
                  return section.querySelector(':scope > details').open && section.closest('.section-group').open;
                })()'''))
                browser.evaluate("document.querySelector('#collapse-all').click()")
                self.assertTrue(browser.evaluate("[...document.querySelectorAll('main > details.section-group')].every(d=>!d.open)"))
                browser.evaluate("document.querySelector('#expand-all').click()")
                self.assertTrue(browser.evaluate("[...document.querySelectorAll('main > details.section-group')].every(d=>d.open)"))
                target = objects.as_uri() + '?view=obj-animations&select=obj-animation-catalog%3A1#obj-animations'
                browser.command('Page.navigate', {'url': target})
                end = time.monotonic() + 30
                while time.monotonic() < end:
                    try:
                        if browser.evaluate("document.readyState==='complete' && location.search.includes('obj-animation-catalog')"):
                            break
                    except (RuntimeError, OSError):
                        pass
                    time.sleep(.05)
                else:
                    self.fail('Bookmarked view did not load')
                self.assertTrue(browser.evaluate('''(()=>{
                  const section=document.querySelector('#obj-animations');
                  return section.closest('.section-group').open && section.querySelector(':scope > details').open &&
                    document.querySelector('#obj-animation-catalog').selectedIndex===1;
                })()'''))
            finally:
                browser.close()
