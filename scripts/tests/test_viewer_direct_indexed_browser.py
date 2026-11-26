"""Check independent direct indexed archives in the scoped graphics viewer."""
import copy
import hashlib
import json
import shutil
import tempfile
import unittest
from pathlib import Path

from test_viewer_indexed_graphics_browser import archive, catalog, payload
from view_rom_data import generate, project_direct_indexed_graphics
from viewer_browser_harness import Chrome


SPECS = {
    'graphics.direct-indexed-archive': ('direct-indexed-archive', 0x023d08, 0x3ae484, 4, 0),
    'graphics.direct-indexed-dictionary-archive': ('direct-indexed-dictionary-archive', 0x023d10, 0x3ae5a4, 41, 96),
}


def direct(resource_id):
    _, literal, source, count, dictionary = SPECS[resource_id]
    value = archive(source, count, tiles=2)
    value['dictionary_bytes'] = list(range(dictionary))
    for index, program in enumerate(value['programs']):
        program['decoded_words'] = [((index % 15) + 1) * 0x11111111] * 16
        if not dictionary:
            program['packets'] = [{'kind': 'literal', 'bytes': [0]}]
    return {'caller_literal_offset': literal, 'archive': value}


def fixture(work, *, main=True, malformed=None, order=None):
    dataset = work/'dataset'; dataset.mkdir()
    resources = []
    if main:
        main_value = catalog()
        if malformed == 'main': main_value['archives'][0]['programs'][0]['decoded_words'].pop()
        resources.append(('graphics.indexed-archives', 'indexed-graphics', 2, main_value))
    for resource_id, (name, _, _, _, _) in SPECS.items():
        value = direct(resource_id)
        if malformed == resource_id: value['archive']['programs'][0]['decoded_words'].pop()
        resources.append((resource_id, name, 1, value))
    if order == 'reverse': resources.reverse()
    manifest = 'profile="test"\nextractor_revision=1\ndecoder_revision=1\n'
    for rid, name, version, value in resources:
        content = json.dumps(value).encode(); (dataset/(name+'.json')).write_bytes(content)
        manifest += (f'[[resources]]\nid="{rid}"\npath="{name}.json"\nformat="{name}-v{version}-json"\n'
                     f'sha256="{hashlib.sha256(content).hexdigest()}"\n')
    (dataset/'manifest.toml').write_text(manifest)
    index = generate(dataset, work/'view', split=True)
    return index, index.parent/(index.name+'.parts/indexed-graphics.html')


class DirectIndexedProjectionTest(unittest.TestCase):
    def test_direct_records_and_source_guards(self):
        for rid in SPECS:
            value = direct(rid); family = project_direct_indexed_graphics(value, rid)
            self.assertEqual(family['resource_id'], rid)
            self.assertEqual(family['archives'][0]['programs'][0]['words'], value['archive']['programs'][0]['decoded_words'])
            self.assertEqual(value, direct(rid))
            for mutate in (lambda c: c.__setitem__('caller_literal_offset', 0),
                           lambda c: c['archive'].__setitem__('source_offset', 0),
                           lambda c: c['archive']['programs'].pop(),
                           lambda c: c['archive']['dictionary_bytes'].append(0),
                           lambda c: c['archive']['programs'][0]['decoded_words'].__setitem__(0, True)):
                bad = copy.deepcopy(value); mutate(bad)
                with self.subTest(resource=rid, mutation=mutate), self.assertRaises(ValueError):
                    project_direct_indexed_graphics(bad, rid)

    def test_sources_remain_independent_in_any_manifest_order(self):
        cases = [(True, None, None, 5), (True, None, 'reverse', 5),
                 (False, None, None, 2), (True, 'main', None, 2),
                 (True, 'graphics.direct-indexed-archive', None, 4),
                 (True, 'graphics.direct-indexed-dictionary-archive', None, 4)]
        for main, malformed, order, expected in cases:
            with self.subTest(main=main, malformed=malformed, order=order), tempfile.TemporaryDirectory(prefix='viewer-direct-indexed-') as directory:
                index, page = fixture(Path(directory), main=main, malformed=malformed, order=order)
                data = payload(page); home = payload(index)
                self.assertEqual(len(data['indexed_archives']['families']), expected)
                self.assertNotIn('indexed_archives', home)
                self.assertTrue(all(row['viewer_scope'] == 'indexed-graphics' for row in home['resources']))
                for row in data['resources']:
                    failed_id = 'graphics.indexed-archives' if malformed == 'main' else malformed
                    self.assertEqual(row['status'].startswith('Invalid'), row['id'] == failed_id)


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class DirectIndexedBrowserTest(unittest.TestCase):
    def test_resource_links_select_independent_family_and_preserve_settings(self):
        with tempfile.TemporaryDirectory(prefix='viewer-direct-indexed-browser-') as directory:
            work = Path(directory); _, page = fixture(work, order='reverse'); browser = Chrome(work, page)
            try:
                self.assertEqual(browser.evaluate('''(async()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change'));
                  const open=id=>q('[data-resource-id="'+id+'"] a[href="#indexed-graphics"]').click();
                  const selected=()=>data.indexed_archives.families[Number(q('#indexed-family').value)];
                  const pixel=()=>[...q('#indexed-sheet').getContext('2d').getImageData(0,0,1,1).data].join(',');
                  open('graphics.direct-indexed-archive');await new Promise(r=>setTimeout(r,0));
                  if(!q('#indexed-graphics>details').open||selected().id!=='direct_literal'||q('#indexed-program').options.length!==4||
                     pixel()!=='17,17,17,255'||!q('#indexed-detail').textContent.includes('direct-indexed-archive.json')||
                     !q('#indexed-source-fields').textContent.includes('caller literal 0x23D08'))throw Error('literal link');
                  q('#indexed-loop').checked=true;q('#indexed-rate').value='12';q('#indexed-depth').value='8';change(q('#indexed-depth'));
                  open('graphics.direct-indexed-dictionary-archive');
                  if(selected().id!=='direct_dictionary'||q('#indexed-program').options.length!==41||q('#indexed-depth').value!=='8'||
                     !q('#indexed-loop').checked||q('#indexed-rate').value!=='12'||
                     !q('#indexed-detail').textContent.includes('direct-indexed-dictionary-archive.json')||
                     !q('#indexed-source-fields').textContent.includes('caller literal 0x23D10'))throw Error('dictionary link');
                  q('#indexed-program').value='40';change(q('#indexed-program'));
                  if(!q('#indexed-position').textContent.includes('41 of 41'))throw Error('final program');
                  open('graphics.indexed-archives');if(selected().resource_id!=='graphics.indexed-archives')throw Error('main link');
                  return 'PASS';
                })()'''), 'PASS')
            finally:
                browser.close()

    def test_invalid_or_missing_main_does_not_hide_direct_archives(self):
        for main, malformed in ((False, None), (True, 'main'), (True, 'graphics.direct-indexed-archive')):
            with self.subTest(main=main, malformed=malformed), tempfile.TemporaryDirectory(prefix='viewer-direct-indexed-isolation-') as directory:
                work = Path(directory); _, page = fixture(work, main=main, malformed=malformed); browser = Chrome(work, page)
                try:
                    result = browser.evaluate('''(()=>{
                      const q=s=>document.querySelector(s), rows=[...q('#rows').rows];
                      return {visible:!q('#indexed-graphics').hidden,links:rows.map(row=>[row.dataset.resourceId,!!row.querySelector('a[href="#indexed-graphics"]')])};
                    })()''')
                    self.assertTrue(result['visible'])
                    for rid, linked in result['links']:
                        self.assertEqual(linked, rid != malformed and not (rid == 'graphics.indexed-archives' and malformed == 'main'))
                finally:
                    browser.close()
