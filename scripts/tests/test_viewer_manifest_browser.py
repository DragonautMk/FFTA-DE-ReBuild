"""Check bounded, collapsible manifest-source browsing in Chrome."""
import json
import shutil
import tempfile
import unittest
from pathlib import Path

from viewer_browser_harness import Chrome


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class ViewerManifestBrowserTest(unittest.TestCase):
    def test_metadata_paging_filtering_and_safe_rendering(self):
        with tempfile.TemporaryDirectory(prefix='viewer-manifest-') as directory:
            work = Path(directory)
            repo = Path(__file__).resolve().parents[2]
            sources = [{'offset': 0x1000 + i, 'length': i + 1, 'role': 'range-' + str(i),
                        'sha256': f'{i:064x}'} for i in range(83)]
            sources[30]['role'] = '<svg onload=alert(1)>'
            sources[30]['extra'] = {'unknown': 'extra-value'}
            data = {'profile': 'test', 'extractor': 1, 'decoder': 1, 'resources': [
                {'id': 'test.many', 'path': 'many.json', 'format': 'json', 'description': 'Many',
                 'status': 'Available', 'sha256': 'file-digest', 'dependencies': ['test.other'],
                 'sources': sources, 'custom': {'keep': 3}},
                {'id': 'test.empty', 'path': 'empty.json', 'format': 'json', 'description': 'Empty',
                 'status': 'Available', 'sha256': 'empty-digest', 'dependencies': [], 'sources': []},
                {'id': 'test.slim', 'path': 'slim.json', 'format': 'json', 'description': 'Slim',
                 'status': 'Preview unavailable'}]}
            page = work / 'manifest.html'
            page.write_text((repo / 'view_rom_data.html').read_text().replace('/* DATA */', json.dumps(data)))
            browser = Chrome(work, page)
            try:
                self.assertEqual(browser.evaluate('''(async()=>{
                  const q=s=>document.querySelector(s), row=id=>q('[data-resource-id="'+id+'"]');
                  const many=row('test.many').querySelector('details');
                  if(!many || many.open || many.querySelector('table') || row('test.slim').querySelector('details'))throw Error('lazy details');
                  many.open=true;await new Promise(resolve=>setTimeout(resolve,0));
                  const button=label=>[...many.querySelectorAll('button')].find(b=>b.textContent===label);
                  const status=()=>many.querySelector('[role=status]').textContent, body=()=>many.querySelector('tbody');
                  const filter=many.querySelector('[aria-label="Filter source ranges"]'), page=many.querySelector('[aria-label="Source page"]');
                  if(!many.textContent.includes('file-digest') || !many.textContent.includes('test.other') ||
                     !many.textContent.includes('"keep":3') || body().rows.length!==25 ||
                     body().rows[0].cells[0].textContent!=='0x1000 (4096)' || !status().includes('Page 1 of 4'))throw Error('metadata or first page');
                  button('Next').click();if(body().rows[0].cells[0].textContent!=='0x1019 (4121)' || !status().includes('Page 2 of 4'))throw Error('next');
                  page.value='4';button('Go to page').click();
                  if(body().rows.length!==8 || !button('Next').disabled || !status().includes('Page 4 of 4'))throw Error('last page');
                  page.value='999';button('Go to page').click();if(!status().includes('Page 4 of 4'))throw Error('clamp');
                  filter.value='0X101E';button('Filter sources').click();
                  if(body().rows.length!==1 || !status().includes('1 of 83') ||
                     body().rows[0].cells[2].textContent!=='<svg onload=alert(1)>' || body().querySelector('svg') ||
                     !body().rows[0].cells[4].textContent.includes('extra-value'))throw Error('hex filter or escaping');
                  filter.value='range-82';filter.dispatchEvent(new KeyboardEvent('keydown',{key:'Enter',bubbles:true}));
                  if(body().rows.length!==1 || body().rows[0].cells[0].textContent!=='0x1052 (4178)')throw Error('role filter');
                  filter.value='not-present';button('Filter sources').click();
                  if(body().rows.length || !status().includes('No matching pages') || !button('Go to page').disabled)throw Error('empty filter');
                  filter.value='';button('Filter sources').click();if(body().rows.length!==25)throw Error('clear filter');
                  many.open=false;if(many.open)throw Error('collapse');
                  const empty=row('test.empty').querySelector('details');empty.open=true;await new Promise(resolve=>setTimeout(resolve,0));
                  if(!empty.textContent.includes('None') || !empty.textContent.includes('0 of 0 source ranges') || empty.querySelector('tbody').rows.length)throw Error('empty sources');
                  return 'PASS';
                })()'''), 'PASS')
            finally:
                browser.close()

    def test_overview_keeps_metadata_deferred(self):
        with tempfile.TemporaryDirectory(prefix='viewer-overview-') as directory:
            work = Path(directory)
            repo = Path(__file__).resolve().parents[2]
            data = {'profile': 'test', 'extractor': 1, 'decoder': 1,
                    'resources': [{'id': 'test.slim', 'path': 'slim.json', 'format': 'json',
                                   'description': 'Slim', 'status': 'Available', 'viewer_scope': 'other'}],
                    'wiki': {'current': 'index', 'pages': [{'id': 'index', 'label': 'All resources', 'href': 'index.html'},
                                                           {'id': 'other', 'label': 'Other resources', 'href': 'other.html'}]}}
            page = work / 'overview.html'
            page.write_text((repo / 'view_rom_data.html').read_text().replace('/* DATA */', json.dumps(data)))
            browser = Chrome(work, page)
            try:
                self.assertTrue(browser.evaluate('''(()=>{
                  const row=document.querySelector('[data-resource-id="test.slim"]');
                  return !row.querySelector('details') && row.querySelector('a').textContent==='Browse Other resources';
                })()'''))
            finally:
                browser.close()
