"""Verify structural progression and pricing catalogs and their wiki views."""
import copy
import hashlib
import json
import shutil
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from view_rom_data import generate, project_progression_catalog
from viewer_browser_harness import Chrome


NAMES = {'rewards': 'progression-rewards', 'gates': 'progression-gates',
         'selection': 'progression-selection-gates', 'trade': 'trade-parameters'}


def catalogs():
    return {
        'rewards': {'source_offset': 0x6e79da, 'records': [
            {'id': i, 'source_offset': 0x6e79da + 10 * i, 'deltas00_07': [(i + j) % 256 for j in range(8)],
             'discount08': (i * 3) % 256, 'byte09': 255 - i} for i in range(100)]},
        'gates': {'source_offset': 0x6e7dce, 'records': [
            {'id': i, 'source_offset': 0x6e7dce + 12 * i, 'byte00': 255 - i,
             'halfword01': 60000 + i, 'byte03': i + 10, 'requirements04_0b': [(i + j) % 256 for j in range(8)]}
            for i in range(64)]},
        'selection': {'source_offset': 0x6e80de, 'records': [
            {'id': i + 1, 'source_offset': 0x6e80de + 16 * i, 'selector00': 255 - i,
             'minimum01': i, 'minimum02': 1000 + i, 'minimum04': 2000 + i,
             'any_minima06_0d': [(i + j) % 256 for j in range(8)], 'flag0e': 50000 + i}
            for i in range(64)]},
        'trade': {'source_offset': 0x6e85bc, 'records': [
            {'id': i + 2, 'source_offset': 0x6e85bc + 96 * i,
             'bytes00_1f': [(i + j) % 256 for j in range(32)],
             'bytes20_3f': [(i + 32 + j) % 256 for j in range(32)],
             'markup40_41': 0xabcd + i, 'discounts42_5f': [(200 + i + j) % 256 for j in range(30)]}
            for i in range(5)]},
    }


def fixture(work, *, include=None, profile='europe', alter=None):
    dataset = work / 'dataset'; dataset.mkdir()
    values = catalogs()
    if alter:
        alter(values)
    manifest = f'profile="{profile}"\nextractor_revision=1\ndecoder_revision=1\n'
    for kind, name in NAMES.items():
        if include is not None and kind not in include:
            continue
        content = json.dumps(values[kind]).encode(); (dataset / (name + '.json')).write_bytes(content)
        manifest += (f'[[resources]]\nid="gameplay.{name}"\npath="{name}.json"\n'
                     f'format="{name}-v1-json"\nsha256="{hashlib.sha256(content).hexdigest()}"\n')
    (dataset / 'manifest.toml').write_text(manifest)
    return dataset, values


def payload(path):
    return json.JSONDecoder().raw_decode(path.read_text().split('const data = ', 1)[1])[0]


class ProgressionProjectionTest(unittest.TestCase):
    def test_scope_and_full_structural_values(self):
        with tempfile.TemporaryDirectory(prefix='viewer-progression-projection-') as directory:
            work = Path(directory); dataset, values = fixture(work)
            index = generate(dataset, work / 'view', split=True)
            world = payload(index.parent / 'index.html.parts/world.html')
            for kind, expected in values.items():
                self.assertEqual(world['progression_catalogs'][kind]['records'], expected['records'])
                row = next(row for row in world['resources'] if row['id'] == 'gameplay.' + NAMES[kind])
                self.assertTrue(row['status'].startswith('Available'))
            self.assertNotIn('progression_catalogs', payload(index))
            self.assertNotIn('progression_catalogs', payload(index.parent / 'index.html.parts/audio.html'))

    def test_bad_fields_and_independent_catalogs(self):
        values = catalogs()
        mutations = [
            ('rewards', lambda c: c.update(source_offset=0)),
            ('rewards', lambda c: c['records'].pop()),
            ('rewards', lambda c: c['records'][0].update(discount08=True)),
            ('gates', lambda c: c['records'][0].update(id=1)),
            ('gates', lambda c: c['records'][0].update(halfword01=65536)),
            ('selection', lambda c: c['records'][0].update(source_offset=0)),
            ('selection', lambda c: c['records'][0].update(any_minima06_0d=[0] * 7)),
            ('selection', lambda c: c['records'][0].update(flag0e=-1)),
            ('trade', lambda c: c['records'][0].update(markup40_41=65536)),
            ('trade', lambda c: c['records'][0].update(discounts42_5f=[256] * 30)),
            ('trade', lambda c: c['records'][0].update(bytes00_1f=[True] * 32)),
        ]
        for kind, mutate in mutations:
            bad = copy.deepcopy(values[kind]); mutate(bad)
            with self.subTest(kind=kind), self.assertRaises(ValueError):
                project_progression_catalog(bad, kind)
        with tempfile.TemporaryDirectory(prefix='viewer-progression-partial-') as directory:
            work = Path(directory)
            dataset, _ = fixture(work, include={'rewards', 'trade'},
                                 alter=lambda c: c['trade']['records'][0].update(id=9))
            index = generate(dataset, work / 'view', split=True)
            world = payload(index.parent / 'index.html.parts/world.html')
            self.assertEqual(set(world['progression_catalogs']), {'rewards'})
            self.assertIn('Invalid progression/pricing trade data',
                          next(row['status'] for row in world['resources'] if row['id'] == 'gameplay.trade-parameters'))

    def test_other_profile_does_not_use_european_layout(self):
        with tempfile.TemporaryDirectory(prefix='viewer-progression-profile-') as directory:
            work = Path(directory); dataset, _ = fixture(work, profile='usa')
            index = generate(dataset, work / 'view', split=True)
            world = payload(index.parent / 'index.html.parts/world.html')
            self.assertEqual(world['progression_catalogs'], {})
            self.assertTrue(all(row['status'] == 'Preview unavailable' for row in world['resources']))


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class ProgressionBrowserTest(unittest.TestCase):
    def test_lazy_panels_filters_pagination_and_stored_alias(self):
        with tempfile.TemporaryDirectory(prefix='viewer-progression-browser-') as directory:
            work = Path(directory); dataset, _ = fixture(work)
            index = generate(dataset, work / 'view', split=True)
            browser = Chrome(work, index.parent / 'index.html.parts/world.html')
            try:
                self.assertEqual(browser.evaluate('''(async()=>{
                  const q=s=>document.querySelector(s), wait=()=>new Promise(resolve=>setTimeout(resolve,50));
                  if(q('#progression-records').hidden || q('#progression-records>details').open ||
                     q('#progression-rewards-view').children.length || q('#progression-gates-view').children.length)throw Error('lazy panels');
                  const resource=[...q('#rows').rows].find(row=>row.dataset.resourceId==='gameplay.progression-gates');
                  const link=[...resource.querySelectorAll('a')].find(a=>a.getAttribute('href')==='#progression-records');
                  if(!link)throw Error('resource link');link.click();await wait();
                  if(!q('#progression-gates-panel').open || !q('#progression-records>details').open ||
                     q('#progression-gates-rows').rows.length!==25 || q('#progression-gates-rows').rows[0].cells[3].textContent!=='60000')throw Error('gate');
                  q('#progression-gates-next').click();
                  if(q('#progression-gates-rows').rows[0].cells[0].textContent!=='25' ||
                     q('#progression-gates-page').textContent!=='26–50 of 64')throw Error('pagination');
                  q('#progression-gates-filter').value='63';q('#progression-gates-filter').dispatchEvent(new Event('input'));
                  if(q('#progression-gates-rows').rows.length!==1 || q('#progression-gates-rows').rows[0].cells[0].textContent!=='63')throw Error('filter id');
                  q('#progression-rewards-panel').open=true;await wait();
                  if(q('#progression-rewards-rows').rows[0].cells[2].textContent!=='0' ||
                     q('#progression-rewards-rows').rows[0].cells[9].textContent!=='7' ||
                     q('#progression-rewards-page').textContent!=='1–25 of 100')throw Error('rewards');
                  q('#progression-rewards-filter').value='0x6e79da';q('#progression-rewards-filter').dispatchEvent(new Event('input'));
                  if(q('#progression-rewards-rows').rows.length!==1 || q('#progression-rewards-rows').rows[0].cells[0].textContent!=='0')throw Error('filter offset');
                  q('#progression-selection-panel').open=true;await wait();
                  const selection=q('#progression-selection-rows').rows[0];
                  if(selection.cells[0].textContent!=='1' || selection.cells[2].textContent!=='255' ||
                     selection.cells[14].textContent!=='50000 (0xC350)')throw Error('selection');
                  q('#progression-trade-panel').open=true;await wait();
                  const trade=q('#progression-trade-rows').rows[0];
                  if(trade.cells[0].textContent!=='2' || trade.cells[2].textContent!=='43981' || trade.cells[3].textContent!=='171' ||
                     trade.cells[4].querySelector('pre').textContent.split('\\n').length!==30 ||
                     !trade.cells[5].querySelector('pre').textContent.includes('0x30: 0x30') ||
                     trade.cells[4].querySelector('details').open)throw Error('trade alias');
                  q('#progression-trade-panel').open=false;await wait();q('#progression-trade-panel').open=true;await wait();
                  if(q('#progression-trade-rows').rows.length!==5 || q('#progression-gates-filter').value!=='63')throw Error('preserve state');
                  return 'PASS';
                })()'''), 'PASS')
            finally:
                browser.close()

    def test_partial_catalog_renders_alone(self):
        with tempfile.TemporaryDirectory(prefix='viewer-progression-partial-browser-') as directory:
            work = Path(directory); dataset, _ = fixture(work, include={'trade'})
            browser = Chrome(work, generate(dataset, work / 'view'))
            try:
                self.assertTrue(browser.evaluate('''(async()=>{
                  const q=s=>document.querySelector(s);q('#progression-trade-panel').open=true;
                  await new Promise(resolve=>setTimeout(resolve,50));
                  return !q('#progression-records').hidden && !q('#progression-trade-panel').hidden &&
                    q('#progression-gates-panel').hidden && q('#progression-trade-rows').rows.length===5;
                })()'''))
            finally:
                browser.close()
