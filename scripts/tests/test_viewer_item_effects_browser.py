"""Check item-effect source validation, bounded calculation, and wiki links."""
import copy
import hashlib
import json
import shutil
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from view_rom_data import generate, project_item_effect_parameters
from viewer_browser_harness import Chrome


def fixture(work, *, include_items=True, profile='europe'):
    dataset = work / 'dataset'; dataset.mkdir()
    effects = {'source_offset': 0x6e879c, 'records': [
        {'id': index, 'item_id': 362 + index, 'source_offset': 0x6e879c + index * 7,
         'coefficients': [3 + (index + coefficient) % 7 for coefficient in range(9)],
         'cap45_52': 30, 'opaque53_55': index % 8}
        for index in range(14)]}
    item = {'halfwords_00_06': [0] * 4, 'bytes_08_0f': [0] * 8,
            'signed_10_17': [0] * 8, 'bytes_18_1f': [0] * 8}
    resources = [('item-effect-parameters', effects)]
    if include_items:
        resources.append(('item-records', {'records': [item] * 376}))
    manifest = f'profile="{profile}"\nextractor_revision=1\ndecoder_revision=1\n'
    for name, value in resources:
        content = json.dumps(value).encode(); (dataset / (name + '.json')).write_bytes(content)
        manifest += (f'[[resources]]\nid="gameplay.{name}"\npath="{name}.json"\n'
                     f'format="{name}-v1-json"\nsha256="{hashlib.sha256(content).hexdigest()}"\n')
    (dataset / 'manifest.toml').write_text(manifest)
    return dataset, effects


def payload(page):
    return json.JSONDecoder().raw_decode(page.read_text().split('const data = ', 1)[1])[0]


class ItemEffectsProjectionTest(unittest.TestCase):
    def test_physical_record_validation_and_scope(self):
        with tempfile.TemporaryDirectory(prefix='viewer-item-effects-projection-') as directory:
            work = Path(directory); dataset, effects = fixture(work)
            index = generate(dataset, work / 'view', split=True)
            world = payload(index.parent / 'index.html.parts/world.html')
            self.assertEqual(world['item_effects']['records'], effects['records'])
            self.assertNotIn('item_effects', payload(index))
            self.assertNotIn('item_effects', payload(index.parent / 'index.html.parts/audio.html'))
            self.assertEqual(next(row['status'] for row in world['resources']
                                  if row['id'] == 'gameplay.item-effect-parameters'), 'Available: item effect parameters')
            mutations = [lambda c: c.update(source_offset=0), lambda c: c['records'].pop(),
                         lambda c: c['records'][0].update(item_id=363),
                         lambda c: c['records'][0].update(source_offset=0),
                         lambda c: c['records'][0].update(coefficients=[32] * 9),
                         lambda c: c['records'][0].update(cap45_52=True),
                         lambda c: c['records'][0].update(opaque53_55=8)]
            for mutate in mutations:
                bad = copy.deepcopy(effects); mutate(bad)
                with self.assertRaises(ValueError): project_item_effect_parameters(bad)

    def test_other_profiles_do_not_infer_european_item_mapping(self):
        with tempfile.TemporaryDirectory(prefix='viewer-item-effects-profile-') as directory:
            work = Path(directory); dataset, _ = fixture(work, profile='usa')
            index = generate(dataset, work / 'view', split=True)
            world = payload(index.parent / 'index.html.parts/world.html')
            self.assertIsNone(world['item_effects'])
            self.assertEqual(next(row['status'] for row in world['resources']
                                  if row['id'] == 'gameplay.item-effect-parameters'), 'Preview unavailable')


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class ItemEffectsBrowserTest(unittest.TestCase):
    def test_calculation_and_item_navigation(self):
        with tempfile.TemporaryDirectory(prefix='viewer-item-effects-browser-') as directory:
            work = Path(directory); dataset, _ = fixture(work)
            index = generate(dataset, work / 'view', split=True)
            browser = Chrome(work, index.parent / 'index.html.parts/world.html')
            try:
                self.assertEqual(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change')),
                    input=e=>e.dispatchEvent(new Event('input'));
                  if(q('#item-effects').hidden || q('#item-effects>details').open || q('#item-effect-coefficients').rows.length!==9 ||
                     !q('#item-effect-source').textContent.includes('cap 30'))throw Error('record');
                  q('#item-effect-current').value='10';input(q('#item-effect-current'));
                  q('#item-effect-quantity').value='2';input(q('#item-effect-quantity'));
                  if(!q('#item-effect-result').textContent.includes('increase 6') || !q('#item-effect-result').textContent.includes('value 16'))throw Error('ordinary increase');
                  q('#item-effect-quantity').value='10';input(q('#item-effect-quantity'));
                  if(!q('#item-effect-result').textContent.includes('increase 20'))throw Error('capped increase');
                  q('#item-effect-current').value='32';input(q('#item-effect-current'));
                  if(!q('#item-effect-result').textContent.includes('increase 0'))throw Error('already capped');
                  q('#item-effect-current').value='1';input(q('#item-effect-current'));
                  const expected=[3,3,4,4,4,5,5,6,6,6,7,7,8,8,9,9,3,3,4,4];
                  for(let id=0;id<expected.length;id++){
                    q('#item-effect-property').value=String(id);input(q('#item-effect-property'));
                    if(!q('#item-effect-result').textContent.includes('Coefficient '+expected[id]+' ·'))throw Error('property '+id);
                  }
                  q('#item-effect-property').value='20';input(q('#item-effect-property'));
                  if(!q('#item-effect-result').textContent.includes('Coefficient 0') || !q('#item-effect-result').textContent.includes('increase 0'))throw Error('unknown property');
                  q('#item-effect-quantity').value='256';input(q('#item-effect-quantity'));
                  if(!q('#item-effect-result').textContent.includes('0 to 255'))throw Error('input bounds');
                  q('#item-effect-quantity').value='2';input(q('#item-effect-quantity'));
                  q('#item-effect-property').value='1';input(q('#item-effect-property'));
                  q('#item-id').value='363';change(q('#item-id'));q('#item-effect-open').click();
                  if(q('#item-effect-id').value!=='1' || !q('#item-effects>details').open ||
                     q('#item-effect-property').value!=='1' || q('#item-effect-quantity').value!=='2' ||
                     !q('#item-effect-result').textContent.includes('increase 8'))throw Error('item to effect');
                  q('#item-effect-id').value='2';change(q('#item-effect-id'));q('#effect-item-open').click();
                  if(q('#item-id').value!=='364' || !q('#items>details').open)throw Error('effect to item');
                  q('#item-id').value='0';change(q('#item-id'));
                  if(!q('#item-effect-navigation').hidden)throw Error('unrelated item');
                  return 'PASS';
                })()'''), 'PASS')
            finally:
                browser.close()

    def test_effects_work_without_item_owner(self):
        with tempfile.TemporaryDirectory(prefix='viewer-item-effects-only-') as directory:
            work = Path(directory); dataset, _ = fixture(work, include_items=False)
            browser = Chrome(work, generate(dataset, work / 'view'))
            try:
                self.assertTrue(browser.evaluate("!document.querySelector('#item-effects').hidden && document.querySelector('#effect-item-navigation').hidden && document.querySelector('#item-effect-coefficients').rows.length===9"))
            finally:
                browser.close()
