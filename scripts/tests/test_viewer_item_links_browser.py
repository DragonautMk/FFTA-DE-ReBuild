"""Check the item-linked pair wiki and its two-way navigation in Chrome."""
import hashlib
import json
import shutil
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from view_rom_data import generate
from viewer_browser_harness import Chrome


def fixture(work, *, include_items):
    dataset = work / 'dataset'
    dataset.mkdir()
    slots = [{'selector': index, 'value': 100 + index} for index in range(9)]
    links = {'source_offset': 0x1234, 'records': [
        {'count': 2, 'opaque': 7, 'slots': slots},
        {'count': 1, 'opaque': 0, 'slots': slots},
        {'count': 0, 'opaque': 0, 'slots': slots},
    ]}
    items = {'records': [{'halfwords_00_06': [0] * 4, 'bytes_08_0f': [0] * 8,
                          'signed_10_17': [0] * 8, 'bytes_18_1f': [0, 0, 0, 0, 0, link, 0, 0]}
                         for link in (0, 1, 1, 9)]}
    manifest = 'profile="test"\nextractor_revision=1\ndecoder_revision=1\n'
    resources = [('item-links', links)]
    if include_items:
        resources.append(('item-records', items))
    for name, value in resources:
        content = json.dumps(value).encode()
        (dataset / (name + '.json')).write_bytes(content)
        manifest += (f'[[resources]]\nid="gameplay.{name}"\npath="{name}.json"\n'
                     f'format="{name}-v1-json"\nsha256="{hashlib.sha256(content).hexdigest()}"\n')
    (dataset / 'manifest.toml').write_text(manifest)
    return generate(dataset, work / 'view')


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class ViewerItemLinksTest(unittest.TestCase):
    def test_pair_records_and_bidirectional_navigation(self):
        with tempfile.TemporaryDirectory(prefix='viewer-item-links-') as directory:
            work = Path(directory)
            browser = Chrome(work, fixture(work, include_items=True))
            try:
                result = browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change'));
                  if(q('#item-links').hidden || q('#item-links>details').open)throw Error('disclosure');
                  const row=[...document.querySelectorAll('#rows tr')].find(r=>r.cells[0].textContent.includes('gameplay.item-links'));
                  row.querySelector('a').click();if(!q('#item-links>details').open)throw Error('resource link');
                  if(q('#item-link-slots').rows.length!==9 || !q('#item-link-source').textContent.includes('count 2') ||
                     !q('#item-link-source').textContent.includes('opaque byte 7'))throw Error('record');
                  let cells=[...q('#item-link-slots').rows[8].cells].map(c=>c.textContent);
                  if(cells.join('|')!=='8|Beyond count|8|108')throw Error('stored inactive value');
                  q('#item-link-show-inactive').checked=false;change(q('#item-link-show-inactive'));
                  if(q('#item-link-slots').rows.length!==2)throw Error('hide inactive');
                  q('#item-id').value='2';change(q('#item-id'));q('#item-link-open').click();
                  if(q('#item-link-record').value!=='1' || q('#item-link-slots').rows.length!==1 ||
                     q('#item-link-show-inactive').checked || !q('#item-link-source').textContent.includes('0x1248'))throw Error('item to link');
                  if(!q('#item-link-owners').textContent.includes('2 item records') || q('#item-link-owners').querySelectorAll('a').length!==2)throw Error('owners');
                  q('#item-link-owners a').click();
                  if(q('#item-id').value!=='1' || !q('#items>details').open)throw Error('link to item');
                  q('#item-link-record').value='2';change(q('#item-link-record'));
                  if(!q('#item-link-owners').textContent.includes('No extracted item') || q('#item-link-slots').rows.length)throw Error('unreferenced');
                  q('#item-id').value='3';change(q('#item-id'));
                  if(!q('#item-link-navigation').hidden)throw Error('missing pair record');
                  q('#item-link-show-inactive').checked=true;change(q('#item-link-show-inactive'));
                  if(q('#item-link-slots').rows.length!==9)throw Error('restore inactive');
                  return 'PASS';
                })()''')
                self.assertEqual(result, 'PASS')
            finally:
                browser.close()

    def test_pair_records_without_items(self):
        with tempfile.TemporaryDirectory(prefix='viewer-item-links-only-') as directory:
            work = Path(directory)
            browser = Chrome(work, fixture(work, include_items=False))
            try:
                self.assertEqual(browser.evaluate('''(()=>{
                  if(document.querySelector('#item-links').hidden || !document.querySelector('#items').hidden ||
                     !document.querySelector('#item-link-owners').textContent.includes('unavailable') ||
                     document.querySelector('#item-link-slots').rows.length!==9)throw Error('independent records');
                  return 'PASS';
                })()'''), 'PASS')
            finally:
                browser.close()
