"""Check bounded, cancellable filtering of structured JSON arrays."""
import shutil
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from test_viewer_instruments_browser import fixture
from view_rom_data import split_viewer_template
from viewer_browser_harness import Chrome


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class ViewerJsonFilterTest(unittest.TestCase):
    def test_filter_results_navigation_limits_and_cancellation(self):
        with tempfile.TemporaryDirectory(prefix='viewer-json-filter-') as directory:
            for split in (False, True):
                with self.subTest(split=split):
                    work = Path(directory) / ('split' if split else 'legacy')
                    work.mkdir()
                    page = fixture(work)
                    if split:
                        page.write_text(split_viewer_template(page.read_text()))
                    browser = Chrome(work, page)
                    try:
                        result = browser.evaluate('''(async()=>{
                          const q=s=>document.querySelector(s),rows=()=>[...q('#tree-rows').rows];
                          const wait=async f=>{const end=Date.now()+8000;while(!f()){if(Date.now()>end)throw Error('wait timeout');await new Promise(r=>setTimeout(r,10))}};
                          const select=(content,name)=>{const files=new DataTransfer();files.items.add(new File([content],name));
                            q('#inspect-file').files=files.files;q('#inspect-file').dispatchEvent(new Event('change'));};
                          const apply=async value=>{q('#tree-filter').value=value;q('#tree-filter').dispatchEvent(new Event('input'));
                            q('#tree-filter-apply').click();await wait(()=>q('#tree-filter-cancel').disabled);};
                          const records=Array.from({length:151},(_,i)=>({id:i,label:'ordinary',nested:{hidden:'NestedNeedle'}}));
                          for(const i of [3,47,120,135])records[i].label='Unique_MARK';
                          records[2]=null;records[4]='Unique_MARK scalar';records[6].special_key='value';records[9]=['findME'];
                          select(JSON.stringify(records),'records.json');await wait(()=>!q('#tree').hidden);
                          q('#tree-table').checked=true;q('#tree-table').dispatchEvent(new Event('change'));
                          await apply('uNiQuE_mArK');
                          if(rows().map(r=>r.cells[0].textContent).join(',')!=='3,4,47,120,135' ||
                             !q('#tree-path').textContent.includes('5 matches of 151') ||
                             !q('#tree-index-label').textContent.includes('Result position'))throw Error('matching source indices');
                          rows()[0].cells[1].querySelector('button').click();
                          if(!q('#tree-path').textContent.includes('[3]') || !q('#tree-filter-control').hidden)throw Error('open match');
                          q('#tree-up').click();
                          if(rows().length!==5 || q('#tree-filter').value!=='uNiQuE_mArK')throw Error('filter preserved on return');
                          q('#tree-index').value='3';q('#tree-go').click();
                          if(!q('#tree-navigation-status').textContent.includes('source index 120'))throw Error('result position');
                          q('#tree-table').checked=false;q('#tree-table').dispatchEvent(new Event('change'));
                          if(rows().map(r=>r.cells[0].textContent).join(',')!=='3,4,47,120,135')throw Error('list mode');
                          await apply('SPECIAL_KEY');if(rows().length!==1 || rows()[0].cells[0].textContent!=='6')throw Error('field names');
                          await apply('NestedNeedle');
                          if(rows().length || !q('#tree-go').disabled || !q('#tree-filter-status').textContent.includes('0 matches'))throw Error('nested values');
                          await apply('findme');if(rows().length!==1 || rows()[0].cells[0].textContent!=='9')throw Error('direct array value');
                          await apply('null');if(rows().length!==1 || rows()[0].cells[0].textContent!=='2')throw Error('null value');
                          q('#tree-filter-clear').click();if(rows().length!==100 || q('#tree-filter').value!=='')throw Error('clear filter');
                          select(JSON.stringify(Array.from({length:12000},(_,i)=>({id:i,group:'same'}))),'many.json');
                          await wait(()=>!q('#tree').hidden && q('#tree-path').textContent.includes('12000 entries'));
                          q('#tree-table').checked=true;q('#tree-table').dispatchEvent(new Event('change'));await apply('same');
                          if(!q('#tree-filter-status').textContent.includes('10000 matches') ||
                             !q('#tree-filter-status').textContent.includes('stopped after 10000'))throw Error('bounded results');
                          q('#tree-index').value='9999';q('#tree-go').click();
                          if(rows().length!==25 || rows().at(-1).cells[0].textContent!=='9999')throw Error('last bounded result');
                          q('#tree-filter').value='no-such-value';q('#tree-filter').dispatchEvent(new Event('input'));
                          q('#tree-filter-apply').click();q('#tree-filter-cancel').click();
                          await new Promise(r=>setTimeout(r,50));
                          if(!q('#tree-filter-status').textContent.includes('cancelled') || rows().at(-1).cells[0].textContent!=='9999')throw Error('cancellation');
                          q('#tree-filter').value='another-search';q('#tree-filter-apply').click();
                          select('[{"id":99}]','replacement.json');
                          await wait(()=>!q('#tree').hidden && q('#tree-path').textContent.includes('1 entries'));
                          await new Promise(r=>setTimeout(r,50));
                          if(!q('#tree-filter-status').textContent.includes('No filter applied') || rows()[0].cells[0].textContent!=='0')throw Error('file replacement');
                          return 'PASS';
                        })()''')
                        self.assertEqual(result, 'PASS')
                    finally:
                        browser.close()
