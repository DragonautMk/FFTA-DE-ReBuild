"""Search nested extracted JSON while preserving scoped viewer behavior."""
import json
import shutil
import sys
import tempfile
import time
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from test_viewer_instruments_browser import fixture
from test_viewer_resource_pages_browser import fixture as resource_fixture, page_data
from view_rom_data import generate, split_viewer_template
from viewer_browser_harness import Chrome


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class ViewerDeepJsonSearchTest(unittest.TestCase):
    def test_nested_scope_navigation_limits_and_cancellation(self):
        with tempfile.TemporaryDirectory(prefix='viewer-deep-json-') as directory:
            work = Path(directory); page = fixture(work)
            page.write_text(split_viewer_template(page.read_text()))
            browser = Chrome(work, page)
            try:
                self.assertTrue(browser.evaluate('''(async()=>{
                  const q=s=>document.querySelector(s),rows=()=>[...q('#tree-search-rows').rows];
                  const wait=async f=>{const end=Date.now()+10000;while(!f()){
                    if(Date.now()>end)throw Error('wait timeout');await new Promise(r=>setTimeout(r,10))}};
                  const select=(content,name)=>{const f=new DataTransfer();f.items.add(new File([content],name));
                    q('#inspect-file').files=f.files;q('#inspect-file').dispatchEvent(new Event('change'));};
                  const search=async value=>{q('#tree-search-query').value=value;q('#tree-search-query').dispatchEvent(new Event('input'));
                    q('#tree-search-find').click();await wait(()=>q('#tree-search-cancel').disabled);};
                  const data={outer:{label:'ordinary',rows:[{name:'NeedleAlpha',inner:{hidden:'needleBeta'}}]},
                    other:{hidden:'needleOutside'},unicode:'MÙ',html:'<img src=x onerror=globalThis.bad=1>'};
                  select(JSON.stringify(data),'nested.json');await wait(()=>!q('#tree').hidden);
                  await search('NeEdLe');
                  if(rows().length!==3||!rows()[1].cells[0].textContent.includes('["inner"]["hidden"]')||
                     rows()[1].cells[1].textContent!=='Value')throw Error('nested values and paths');
                  rows()[1].querySelector('button').click();
                  if(!q('#tree-path').textContent.includes('["inner"]["hidden"]')||
                     !q('#tree-rows').textContent.includes('needleBeta'))throw Error('open scalar');
                  q('#tree-up').click();q('#tree-search-scope').value='current';await search('hidden');
                  if(rows().length!==1||rows()[0].cells[1].textContent!=='Field')throw Error('current subtree');
                  q('#tree-search-scope').value='file';await search('mù');
                  if(rows().length!==1||!rows()[0].cells[2].textContent.includes('MÙ'))throw Error('unicode search');
                  await search('onerror');if(rows().length!==1||globalThis.bad)throw Error('safe result text');
                  const special=JSON.parse('{"__proto__":{"needle_key":"hit"},"constructor":"safe"}');
                  select(JSON.stringify(special),'special.json');await wait(()=>!q('#tree').hidden);
                  await search('needle_key');if(rows().length!==1||!rows()[0].cells[0].textContent.includes('__proto__'))throw Error('special keys');
                  rows()[0].querySelector('button').click();if(!q('#tree-rows').textContent.includes('hit'))throw Error('special path');
                  select(JSON.stringify(Array.from({length:3000},(_,i)=>({id:i,text:'match'}))),'many.json');
                  await wait(()=>!q('#tree').hidden && q('#tree-path').textContent.includes('3000 entries'));
                  await search('match');
                  if(rows().length!==25||!q('#tree-search-status').textContent.includes('1,000-match limit')||
                     !q('#tree-search-position').textContent.includes('1–25 of 1000'))throw Error('bounded results');
                  q('#tree-search-next').click();if(!q('#tree-search-position').textContent.includes('26–50'))throw Error('next page');
                  q('#tree-search-prev').click();
                  q('#tree-search-query').value='missing';q('#tree-search-query').dispatchEvent(new Event('input'));
                  if(!q('#tree-search-status').textContent.includes('Previous results remain'))throw Error('stale results indication');
                  q('#tree-search-find').click();q('#tree-search-cancel').click();
                  await new Promise(r=>setTimeout(r,50));
                  if(!q('#tree-search-status').textContent.includes('cancelled')||rows().length!==25)throw Error('cancel retains results');
                  q('#tree-search-find').click();select('{"replacement":true}','new.json');
                  await wait(()=>!q('#tree').hidden && q('#tree-rows').textContent.includes('replacement'));
                  await new Promise(r=>setTimeout(r,50));
                  if(!q('#tree-search-results').hidden||q('#tree-search-query').value!==''||
                     !q('#tree-search-status').textContent.includes('Enter a field'))throw Error('file replacement');
                  let deep={leaf:'verydeep'};for(let i=0;i<258;i++)deep={n:deep};
                  select(JSON.stringify(deep),'deep.json');await wait(()=>!q('#tree').hidden);await search('verydeep');
                  if(rows().length!==0||!q('#tree-search-status').textContent.includes('beyond 256 levels'))throw Error('depth limit');
                  select('"RootNeedle"','scalar.json');await wait(()=>!q('#tree').hidden && q('#tree-rows').textContent.includes('RootNeedle'));
                  await search('rootneedle');if(rows().length!==1||rows()[0].cells[0].textContent!=='$')throw Error('root scalar');
                  return true;})()'''))
            finally:
                browser.close()

    def test_resource_page_uses_search_offline(self):
        with tempfile.TemporaryDirectory(prefix='viewer-deep-resource-') as directory:
            work = Path(directory); dataset, _ = resource_fixture(work)
            index = generate(dataset, work / 'out', 'wiki.html', split=True)
            resource = next(row for row in page_data(index)['resources'] if row['id'] == 'gameplay.example')
            page = index.parent / (index.name + '.parts') / 'files' / resource['embedded_view']
            browser = Chrome(work, index)
            try:
                browser.command('Page.navigate', {'url': page.as_uri()})
                deadline = time.monotonic() + 10
                while time.monotonic() < deadline:
                    if browser.evaluate("Boolean(document.querySelector('#tree')) && !document.querySelector('#tree').hidden"):
                        break
                    time.sleep(.03)
                else:
                    self.fail('Resource page did not parse JSON')
                self.assertTrue(browser.evaluate('''(async()=>{
                  const q=s=>document.querySelector(s);q('#tree-search-query').value='Mù';q('#tree-search-find').click();
                  const end=Date.now()+5000;while(!q('#tree-search-cancel').disabled){
                    if(Date.now()>end)throw Error('timeout');await new Promise(r=>setTimeout(r,10));}
                  const row=q('#tree-search-rows').rows[0];if(!row||!row.cells[0].textContent.includes('["records"][0]["name"]'))return false;
                  row.querySelector('button').click();return q('#tree-rows').textContent.includes('Mù')&&!globalThis.xss;})()'''))
            finally:
                browser.close()
