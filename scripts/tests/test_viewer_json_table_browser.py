"""Check bounded generic JSON record tables in both offline viewer modes."""
import json
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
class ViewerJsonTableTest(unittest.TestCase):
    def test_array_tables_navigation_and_large_file_limit(self):
        records = [{'id': i, 'name': f'entry {i}', 'nested': {'code': i},
                    'list': [i, i + 1], 'nullable': None} for i in range(151)]
        records[0]['name'] = '<img src=x onerror=window.unsafeExecuted=1>'
        records[0]['__proto__'] = 'stored key'
        records[0]['huge'] = 9007199254740993
        records[0]['safe_number'] = 9007199254740991
        records[0]['overflow'] = '__POSITIVE_EXPONENT__'
        records[0]['negative_overflow'] = '__NEGATIVE_EXPONENT__'
        del records[1]['nullable']
        records[2], records[3], records[4] = None, 'scalar', [1, 2]
        records[5].update({f'extra_{i:02}': i for i in range(40)})
        raw = (json.dumps(records).replace('"__POSITIVE_EXPONENT__"', '1e400')
               .replace('"__NEGATIVE_EXPONENT__"', '-1e400'))
        with tempfile.TemporaryDirectory(prefix='viewer-json-table-') as directory:
            for split in (False, True):
                with self.subTest(split=split):
                    work = Path(directory) / ('split' if split else 'legacy')
                    work.mkdir()
                    page = fixture(work)
                    if split:
                        page.write_text(split_viewer_template(page.read_text()))
                    browser = Chrome(work, page)
                    try:
                        script = '''(async()=>{
                          const q=s=>document.querySelector(s),rows=()=>[...q('#tree-rows').rows];
                          const wait=async f=>{const end=Date.now()+5000;while(!f()){if(Date.now()>end)throw Error('load timeout');await new Promise(r=>setTimeout(r,10))}};
                          const select=(content,name)=>{const files=new DataTransfer();files.items.add(new File([content],name));
                            q('#inspect-file').files=files.files;q('#inspect-file').dispatchEvent(new Event('change'));};
                          select(PAYLOAD,'records.json');await wait(()=>!q('#tree').hidden);
                          if(rows().length!==100 || q('#tree-table-control').hidden)throw Error('initial list');
                          q('#tree-table').checked=true;q('#tree-table').dispatchEvent(new Event('change'));
                          const headings=()=>[...document.querySelectorAll('#tree-head th')].map(x=>x.textContent), col=name=>headings().indexOf(name);
                          if(rows().length!==25 || !q('#tree-table-note').textContent.includes('omitted') ||
                             col('id')<0 || col('__proto__')<0 || col('Value')<0)throw Error('bounded table');
                          if(rows()[0].cells[col('nullable')].textContent!=='null' || rows()[1].cells[col('nullable')].textContent!=='—' ||
                             rows()[2].cells[col('Value')].textContent!=='null' || rows()[3].cells[col('Value')].textContent!=='"scalar"' ||
                             rows()[4].cells[col('Value')].textContent!=='Array (2)' || rows()[0].cells[col('__proto__')].textContent!=='"stored key"' ||
                             window.unsafeExecuted!==undefined)throw Error('mixed, missing, null or literal data');
                          if(!rows()[0].cells[col('huge')].textContent.includes('precision may be lost') ||
                             rows()[0].cells[col('safe_number')].textContent!=='9007199254740991')throw Error('integer precision warning');
                          if(rows()[0].cells[col('overflow')].textContent!=='Infinity (outside browser range; see raw file)' ||
                             rows()[0].cells[col('negative_overflow')].textContent!=='-Infinity (outside browser range; see raw file)' ||
                             !q('#tree').textContent.includes('numbers use browser precision'))throw Error('overflow warning');
                          rows()[0].cells[col('nested')].querySelector('button').click();
                          if(!q('#tree-path').textContent.includes('[0]["nested"]') || !q('#tree-table-control').hidden ||
                             rows()[0].cells[0].textContent!=='code')throw Error('nested path');
                          q('#tree-up').click();q('#tree-up').click();
                          rows()[5].cells[1].querySelector('button').click();
                          if(!rows().some(r=>r.cells[0].textContent==='extra_39'))throw Error('omitted field access');
                          q('#tree-up').click();q('#tree-index').value='145';q('#tree-go').click();
                          if(rows().length!==25 || rows()[0].cells[0].textContent!=='125' ||
                             !q('#tree-navigation-status').textContent.includes('145'))throw Error('table jump');
                          q('#tree-next').click();if(rows().length!==1 || rows()[0].cells[0].textContent!=='150')throw Error('last page');
                          q('#tree-prev').click();if(rows()[0].cells[0].textContent!=='125')throw Error('previous page');
                          q('#tree-index').value='151';q('#tree-go').click();
                          if(!q('#tree-navigation-status').textContent.includes('0 to 150'))throw Error('invalid jump');
                          q('#tree-table').checked=false;q('#tree-table').dispatchEvent(new Event('change'));
                          q('#tree-index').value='145';q('#tree-go').click();
                          if(rows().length!==51 || rows()[0].cells[0].textContent!=='100' || headings().join(',')!=='Key or index,Value')throw Error('list jump');
                          select(' '.repeat(8*1024*1024)+'{}','large.json');
                          await wait(()=>q('#tree-status').textContent.includes('8 MiB'));
                          if(!q('#tree').hidden || (q('#tree-load-large') && !q('#tree-load-large').hidden))throw Error('large JSON pause');
                          return 'PASS';
                        })()'''.replace('PAYLOAD', json.dumps(raw))
                        self.assertEqual(browser.evaluate(script), 'PASS')
                    finally:
                        browser.close()
