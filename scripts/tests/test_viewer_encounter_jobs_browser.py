"""Verify authenticated encounter-template job references and reverse navigation."""
import hashlib
import json
import shutil
import sys
import tempfile
import unittest
from contextlib import ExitStack
from pathlib import Path
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import view_rom_data
from test_viewer_encounters_browser import fixture as encounters_fixture
from test_viewer_job_names_browser import fixture as jobs_fixture, payload
from test_viewer_special_units_browser import special_catalog
from viewer_browser_harness import Chrome


def fixture(work, *, wrong_encounter=False, wrong_metadata=False, wrong_special=False, wrong_text=False):
    encounter_work = work / 'enc'; encounter_work.mkdir()
    encounters_fixture(encounter_work)
    dataset = encounter_work / 'dataset'; manifest = dataset / 'manifest.toml'; text = manifest.read_text()
    jobs_work = work / 'jobs'; jobs_work.mkdir(); jobs_dataset, _, _ = jobs_fixture(jobs_work)
    jobs_manifest = (jobs_dataset / 'manifest.toml').read_text().split('[[resources]]\n', 1)[1]
    for path in jobs_dataset.glob('*.json'):
        shutil.copy2(path, dataset / path.name)
    text = text.replace('profile="test"', 'profile="europe"') + '[[resources]]\n' + jobs_manifest
    unit_path = dataset / 'encounter-unit-templates.json'; original = unit_path.read_bytes()
    catalog = json.loads(original); records = catalog['records']
    for record in records:
        record['byte01'] = 1
    records[1]['byte00'], records[1]['byte01'] = 2, 0
    records[2]['byte00'], records[2]['byte01'] = 1, 0
    records[3]['byte00'], records[3]['byte01'] = 100, 0
    content = json.dumps(catalog).encode(); unit_path.write_bytes(content)
    text = text.replace(hashlib.sha256(original).hexdigest(), hashlib.sha256(content).hexdigest())
    special = json.dumps(special_catalog()).encode(); (dataset / 'special.json').write_bytes(special)
    digest = hashlib.sha256(special).hexdigest()
    text += (f'[[resources]]\nid="gameplay.linked-metadata14"\npath="special.json"\n'
             f'format="linked-metadata14-v1-json"\nsha256="{digest}"\n')
    manifest.write_text(text)
    with ExitStack() as stack:
        for name, value in {
            'EUROPE_ENCOUNTER_UNITS_SHA256': 'wrong' if wrong_encounter else hashlib.sha256(content).hexdigest(),
            'EUROPE_METADATA52_SHA256': 'wrong' if wrong_metadata else hashlib.sha256((dataset / 'linked-metadata52.json').read_bytes()).hexdigest(),
            'EUROPE_METADATA14_SHA256': 'wrong' if wrong_special else digest,
            'EUROPE_TEXT_CATALOG_SHA256': 'wrong' if wrong_text else hashlib.sha256((dataset / 'text-catalogs.json').read_bytes()).hexdigest(),
        }.items():
            stack.enter_context(patch.object(view_rom_data, name, value))
        page = view_rom_data.generate(dataset, work / 'view', split=True)
    return page


class EncounterJobProjectionTest(unittest.TestCase):
    def test_direct_descriptor_fallback_and_source_guards(self):
        for options in ({}, {'wrong_encounter': True}, {'wrong_metadata': True}, {'wrong_special': True}, {'wrong_text': True}):
            with self.subTest(options=options), tempfile.TemporaryDirectory(prefix='viewer-encounter-jobs-project-') as directory:
                page = fixture(Path(directory), **options); parts = page.parent / 'index.html.parts'
                world, battle = map(payload, (parts / 'world.html', parts / 'battle.html'))
                links = world['encounter_job_links']; self.assertNotIn('encounter_job_links', battle)
                if options.get('wrong_encounter') or options.get('wrong_metadata'):
                    self.assertIsNone(links); continue
                self.assertEqual(links['jobs'][:4], [1, None if options.get('wrong_special') else 0, 0, None])
                self.assertEqual(links['names_verified'], not options.get('wrong_text', False))
                self.assertEqual(links['jobs'].count(1), 21)


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class EncounterJobBrowserTest(unittest.TestCase):
    def test_name_language_reverse_pagination_and_fallbacks(self):
        with tempfile.TemporaryDirectory(prefix='viewer-encounter-jobs-browser-') as directory:
            work = Path(directory); page = fixture(work); browser = Chrome(work, page.parent / 'index.html.parts/world.html')
            try:
                self.assertTrue(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change',{bubbles:true}));
                  if(q('#encounter-unit-job').hidden || !q('#encounter-unit-job a') ||
                     q('#encounter-unit-job a').dataset.metadataId!=='1' || !q('#encounter-unit-job a').textContent.includes('d'))
                    throw Error('direct name');
                  q('#metadata-name-language').value='1';change(q('#metadata-name-language'));
                  if(!q('#encounter-unit-job a').textContent.includes('Ab'))throw Error('language');
                  q('#encounter-unit-job a').click();
                  const row=q('#metadata-name-rows tr[data-metadata-id="1"]'),button=row&&row.querySelector('button[data-metadata-id="1"]');
                  if(!button || button.textContent!=='21 templates' || !q('#metadata-names>details').open)throw Error('forward link');
                  button.click();
                  if(q('#metadata-encounters').hidden || !q('#metadata-encounters').open ||
                     q('#metadata-encounters-rows').rows.length!==20 || !q('#metadata-encounters-page').textContent.includes('21 references'))
                    throw Error('reverse page');
                  q('#metadata-encounters-next').click();
                  if(q('#metadata-encounters-rows').rows.length!==1 || !q('#metadata-encounters-page').textContent.includes('page 2'))
                    throw Error('reverse pagination');
                  q('#metadata-encounters-rows a').click();
                  if(q('#encounter-id').value!=='2' || q('#encounter-unit').value!=='23' || !q('#encounter-unit-details').open)
                    throw Error('reverse link');
                  q('#encounter-unit').value='1';change(q('#encounter-unit'));
                  if(!q('#encounter-unit-job').textContent.includes('descriptor fallback') ||
                     q('#encounter-unit-job a').dataset.metadataId!=='0')throw Error('descriptor fallback');
                  q('#encounter-unit').value='2';change(q('#encounter-unit'));
                  if(!q('#encounter-unit-job').textContent.includes('descriptor 1 default') ||
                     q('#encounter-unit-job a').dataset.metadataId!=='0')throw Error('descriptor 1');
                  q('#encounter-unit').value='3';change(q('#encounter-unit'));
                  return !q('#encounter-unit-job a') && q('#encounter-unit-job').textContent.includes('cannot be resolved');
                })()'''))
            finally:
                browser.close()
