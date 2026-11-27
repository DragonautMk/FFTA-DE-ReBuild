"""Check independent packed-pair sources, reader guards and browser controls."""
import copy
import hashlib
import json
import shutil
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import view_rom_data
from test_viewer_scoped_browser import parse_page
from test_viewer_special_units_browser import wait_for
from viewer_browser_harness import Chrome


def catalogs():
    banks = []
    for bank_id, start, count in [('primary', 0x6e7484, 92), ('secondary', 0x6e78d4, 16)]:
        records = []
        for index in range(count):
            groups = [[(index + byte + pair) % 4 for pair in range(4)] for byte in range(12)]
            records.append({'id': index, 'source_offset': start + index * 12, 'groups': groups})
        banks.append({'id': bank_id, 'source_offset': start, 'records': records})
    return {'banks': banks}, {'source_offset': 0x6e7994, 'groups': [[(i + j) % 4 for j in range(4)] for i in range(11)]}


def fixture(work, *, verified=True, profile='europe', only=None):
    indexed, global_pairs = catalogs()
    dataset = work / 'dataset'
    dataset.mkdir()
    resources = [('gameplay.linked-metadata12', 'linked-metadata12-v1-json', 'indexed.json', indexed),
                 ('gameplay.global-pair-flags', 'global-pair-flags-v1-json', 'global.json', global_pairs)]
    manifest = f'profile="{profile}"\nextractor_revision=1\ndecoder_revision=1\n'
    digests = []
    for identity, fmt, name, payload in resources:
        content = json.dumps(payload).encode()
        digest = hashlib.sha256(content).hexdigest()
        digests.append(digest)
        if only and only != name:
            continue
        (dataset / name).write_bytes(content)
        manifest += f'[[resources]]\nid="{identity}"\npath="{name}"\nformat="{fmt}"\nsha256="{digest}"\n'
    (dataset / 'manifest.toml').write_text(manifest)
    with patch.object(view_rom_data, 'EUROPE_METADATA12_SHA256', digests[0] if verified else 'wrong'), \
         patch.object(view_rom_data, 'EUROPE_GLOBAL_PAIR_FLAGS_SHA256', digests[1] if verified else 'wrong'):
        page = view_rom_data.generate(dataset, work / 'view', split=True)
    return page, indexed, global_pairs


class PackedPairsProjectionTest(unittest.TestCase):
    def test_projection_authentication_and_page_scope(self):
        for options in ({}, {'verified': False}, {'profile': 'usa'}, {'only': 'global.json'}, {'only': 'indexed.json'}):
            with self.subTest(options=options), tempfile.TemporaryDirectory(prefix='viewer-pairs-project-') as directory:
                page, indexed, global_pairs = fixture(Path(directory), **options)
                home = parse_page(page)
                world = parse_page(page.parent / 'index.html.parts/world.html')
                audio = parse_page(page.parent / 'index.html.parts/audio.html')
                for key in ('metadata12', 'global_pair_flags'):
                    self.assertNotIn(key, home)
                    self.assertNotIn(key, audio)
                indexed_present = options.get('profile') != 'usa' and options.get('only') != 'global.json'
                global_present = options.get('profile') != 'usa' and options.get('only') != 'indexed.json'
                self.assertEqual(bool(world['metadata12']), indexed_present)
                self.assertEqual(bool(world['global_pair_flags']), global_present)
                sections = [s['id'] for p in home['wiki']['pages'] if p['id'] == 'world' for s in p['sections']]
                self.assertEqual('metadata12' in sections, indexed_present)
                self.assertEqual('global-pair-flags' in sections, global_present)
                if indexed_present:
                    self.assertEqual(world['metadata12']['banks'], indexed['banks'])
                    self.assertEqual(world['metadata12']['reader_verified'], options.get('verified', True))
                if global_present:
                    self.assertEqual(world['global_pair_flags']['groups'], global_pairs['groups'])
                    self.assertEqual(world['global_pair_flags']['reader_verified'], options.get('verified', True))

    def test_bad_banks_offsets_and_pairs_are_rejected(self):
        indexed, global_pairs = catalogs()
        changes = [lambda c: c['banks'].pop(), lambda c: c['banks'].reverse(),
                   lambda c: c['banks'][0].update(source_offset=True),
                   lambda c: c['banks'][0]['records'].pop(),
                   lambda c: c['banks'][1]['records'][0].update(id=True),
                   lambda c: c['banks'][1]['records'][0].update(source_offset=0),
                   lambda c: c['banks'][0]['records'][0]['groups'].pop(),
                   lambda c: c['banks'][0]['records'][0]['groups'][0].pop(),
                   lambda c: c['banks'][0]['records'][0]['groups'][0].__setitem__(0, True),
                   lambda c: c['banks'][0]['records'][0]['groups'][0].__setitem__(0, 4)]
        for change in changes:
            bad = copy.deepcopy(indexed)
            change(bad)
            with self.assertRaises(ValueError):
                view_rom_data.project_metadata12(bad)
        for change in (lambda c: c.update(source_offset=0), lambda c: c['groups'].pop(),
                       lambda c: c['groups'][0].__setitem__(0, -1)):
            bad = copy.deepcopy(global_pairs)
            change(bad)
            with self.assertRaises(ValueError):
                view_rom_data.project_global_pair_flags(bad)


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class PackedPairsBrowserTest(unittest.TestCase):
    def test_navigation_modes_bank_changes_and_boundaries(self):
        with tempfile.TemporaryDirectory(prefix='viewer-pairs-browser-') as directory:
            work = Path(directory)
            page, _, _ = fixture(work)
            browser = Chrome(work, page)
            try:
                browser.click('#rows [data-resource-id="gameplay.linked-metadata12"] a')
                wait_for(browser, "data.wiki.current==='world' && !document.querySelector('#metadata12').hidden")
                browser.click('#rows [data-resource-id="gameplay.linked-metadata12"] a[href="#metadata12"]')
                self.assertTrue(browser.evaluate('''(()=>{const q=s=>document.querySelector(s), change=e=>e.dispatchEvent(new Event('change'));
                  if(!q('#metadata12>details').open || q('#metadata12-record').options.length!==92 ||
                     q('#metadata12-rows').rows.length!==12 || q('#metadata12-rows [data-slot="47"]').textContent!=='47: 2')throw Error('initial');
                  q('#metadata12-mode').value='high';change(q('#metadata12-mode'));
                  if(q('#metadata12-rows [data-slot="3"]').textContent!=='3: 1')throw Error('high bit');
                  q('#metadata12-record').value='90';change(q('#metadata12-record'));
                  q('#metadata12-bank').value='1';change(q('#metadata12-bank'));
                  if(q('#metadata12-record').options.length!==16 || q('#metadata12-record').value!=='0' ||
                     q('#metadata12-mode').value!=='high' || !q('#metadata12-source').textContent.includes('6E78D4'))throw Error('bank');
                  q('#metadata12-record').value='15';change(q('#metadata12-record'));
                  if(!q('#metadata12-source').textContent.includes('6E7988'))throw Error('last record');
                  if(q('#global-pair-flags').hidden || q('#global-pairs-rows').rows.length!==11 ||
                     q('#global-pairs-rows [data-slot="43"]').textContent!=='43: 1')throw Error('global');
                  q('#global-pairs-mode').value='low';change(q('#global-pairs-mode'));
                  if(q('#global-pairs-rows [data-slot="3"]').textContent!=='3: 1')throw Error('low bit');
                  return q('#metadata12-mode').value==='high';})()'''))
                browser.click('#rows [data-resource-id="gameplay.global-pair-flags"] a[href="#global-pair-flags"]')
                self.assertTrue(browser.evaluate("document.querySelector('#global-pair-flags>details').open"))
            finally:
                browser.close()

    def test_unverified_sources_expose_only_stored_pairs(self):
        with tempfile.TemporaryDirectory(prefix='viewer-pairs-unverified-') as directory:
            work = Path(directory)
            page, _, _ = fixture(work, verified=False)
            browser = Chrome(work, page.parent / 'index.html.parts/world.html')
            try:
                self.assertTrue(browser.evaluate('''(()=>{const q=s=>document.querySelector(s);return !q('#metadata12').hidden &&
                  !q('#global-pair-flags').hidden && q('#metadata12-mode-label').hidden && q('#global-pairs-mode-label').hidden &&
                  !q('#metadata12-unverified').hidden && !q('#global-pairs-unverified').hidden &&
                  q('#metadata12-rows [data-slot="3"]').textContent==='3: 3' && q('#global-pairs-rows [data-slot="3"]').textContent==='3: 3';})()'''))
            finally:
                browser.close()
