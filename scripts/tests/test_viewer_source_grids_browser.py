"""Exercise documented static source grids and their tilemap links."""
import copy
import hashlib
import json
import shutil
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from view_rom_data import generate, project_source_grids
from test_viewer_firaga_cells_browser import catalog as compact_catalog
from test_viewer_observed_lz_browser import catalog as lz_catalog
from viewer_browser_harness import Chrome


def catalogs():
    firaga = {'literals': [0x08370eb4, 0x083b9f7c, 0x08370eb4, 0x083b9e80], 'matrices': [
        {'source_offset': 0x3b9e80, 'width': 18, 'height': 7, 'cells': list(range(126))},
        {'source_offset': 0x3b9f7c, 'width': 17, 'height': 6, 'cells': list(range(102))}]}
    soil = {'alternate_literal': 0x08ae7874, 'rectangles': [
        {'source_offset': 0xae7810, 'height': 7, 'width': 7, 'cells': list(range(49))},
        {'source_offset': 0xae7874, 'height': 1, 'width': 3, 'cells': [0, 1, 2]}]}
    return firaga, soil


def fixture(work, *, include_firaga=True, include_soil=True, invalid=None, with_other=True):
    dataset = work / 'dataset'; dataset.mkdir(); firaga, soil = catalogs()
    if invalid == 'firaga': firaga['matrices'][0]['width'] = 3
    if invalid == 'soil': soil['alternate_literal'] = 0
    items = []
    if include_firaga: items.append(('graphics.firaga-copied-matrices', 'firaga-copied-matrices', 1, firaga))
    if include_soil: items.append(('graphics.soil-remaining-rectangles', 'soil-remaining-rectangles', 1, soil))
    if with_other:
        items.extend([('graphics.firaga-compact-cells', 'firaga-compact-cells', 1, compact_catalog()),
                      ('graphics.observed-tilemap', 'observed-tilemap', 1,
                       {'source_offset': 2000, 'width': 1, 'height': 1, 'mode': 4,
                        'runs': [{'control': 0x4001, 'value': 0}]}),
                      ('graphics.observed-lz-block', 'observed-lz-block', 2, lz_catalog())])
    manifest = 'profile="test"\nextractor_revision=1\ndecoder_revision=1\n'
    for rid, name, version, value in items:
        content = json.dumps(value).encode(); (dataset / (name + '.json')).write_bytes(content)
        manifest += (f'[[resources]]\nid="{rid}"\npath="{name}.json"\nformat="{name}-v{version}-json"\n'
                     f'sha256="{hashlib.sha256(content).hexdigest()}"\n')
    (dataset / 'manifest.toml').write_text(manifest)
    return generate(dataset, work / 'view', split=True)


def payload(path):
    return json.JSONDecoder().raw_decode(path.read_text().split('const data = ', 1)[1])[0]


class SourceGridsProjectionTest(unittest.TestCase):
    def test_preserves_cells_geometry_and_source_fields(self):
        firaga, soil = catalogs()
        for kind, raw, key in [('firaga_matrices', firaga, 'matrices'), ('soil_rectangles', soil, 'rectangles')]:
            result = project_source_grids(raw, kind)
            self.assertEqual([r['cells'] for r in result['maps']], [r['cells'] for r in raw[key]])
            self.assertEqual([(r['width'], r['height']) for r in result['maps']],
                             [(r['width'], r['height']) for r in raw[key]])
        self.assertEqual(project_source_grids(soil, 'soil_rectangles')['metadata']['alternate_literal'], 0x08ae7874)

    def test_rejects_bad_bindings_or_cells(self):
        for kind, mutate in [('firaga_matrices', lambda c: c['literals'].__setitem__(0, 0)),
                             ('firaga_matrices', lambda c: c['matrices'][0]['cells'].pop()),
                             ('soil_rectangles', lambda c: c['rectangles'][1]['cells'].__setitem__(0, True)),
                             ('soil_rectangles', lambda c: c.__setitem__('alternate_literal', 0))]:
            firaga, soil = catalogs(); value = firaga if kind == 'firaga_matrices' else soil
            mutate(value)
            with self.subTest(kind=kind, mutation=mutate), self.assertRaises(ValueError):
                project_source_grids(value, kind)

    def test_scope_and_invalid_group_isolation(self):
        for invalid in (None, 'firaga', 'soil'):
            with self.subTest(invalid=invalid), tempfile.TemporaryDirectory(prefix='viewer-source-grids-scope-') as directory:
                index = fixture(Path(directory), invalid=invalid); parts = index.parent / (index.name + '.parts')
                self.assertNotIn('extra_tilemaps', payload(index))
                self.assertNotIn('extra_tilemaps', payload(parts / 'objects.html'))
                groups = payload(parts / 'world.html')['extra_tilemaps']
                self.assertEqual('firaga_matrices' in groups, invalid != 'firaga')
                self.assertEqual('soil_rectangles' in groups, invalid != 'soil')


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class SourceGridsBrowserTest(unittest.TestCase):
    def test_resource_navigation_values_and_manual_lz_graphics(self):
        with tempfile.TemporaryDirectory(prefix='viewer-source-grids-') as directory:
            work = Path(directory); index = fixture(work); browser = Chrome(work, index.parent / (index.name + '.parts/world.html'))
            try:
                self.assertEqual(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change',{bubbles:true}));
                  const row=id=>[...q('#rows').rows].find(r=>r.dataset.resourceId===id);
                  row('graphics.firaga-copied-matrices').querySelector('a').click();
                  if(!q('#tilemaps>details').open||q('#tilemap-id').value!=='11'||q('#tilemap-canvas').width!==144||
                     q('#tilemap-canvas').height!==56||q('#firaga-cell-controls').hidden===false||
                     q('#tilemap-source-metadata').hidden||!q('#tilemap-source-fields').textContent.includes('literals'))throw Error('matrix link');
                  if(q('#tilemap-cells').querySelectorAll('button').length!==126)throw Error('matrix cells');
                  q('#tilemap-graphics').value='0';change(q('#tilemap-graphics'));
                  const pixel=[...q('#tilemap-canvas').getContext('2d').getImageData(0,0,1,1).data].join(',');
                  if(pixel!=='34,34,34,255')throw Error('manual LZ graphics');
                  q('#tilemap-id').value='12';change(q('#tilemap-id'));
                  if(q('#tilemap-canvas').width!==136||q('#tilemap-cells').querySelectorAll('button').length!==102)throw Error('second matrix');
                  row('graphics.soil-remaining-rectangles').querySelector('a').click();
                  if(q('#tilemap-id').value!=='13'||q('#tilemap-canvas').width!==56||
                     !q('#tilemap-source-fields').textContent.includes('alternate_literal'))throw Error('soil link');
                  q('#tilemap-id').value='14';change(q('#tilemap-id'));
                  if(q('#tilemap-canvas').width!==24||q('#tilemap-canvas').height!==8||
                     q('#tilemap-cells').querySelectorAll('button').length!==3)throw Error('small rectangle');
                  q('#tilemap-id').value='0';change(q('#tilemap-id'));if(!q('#tilemap-source-metadata').hidden)throw Error('metadata scope');
                  return 'PASS';
                })()'''), 'PASS')
            finally: browser.close()

    def test_standalone_and_invalid_resource_links(self):
        for invalid in (None, 'firaga', 'soil'):
            with self.subTest(invalid=invalid), tempfile.TemporaryDirectory(prefix='viewer-source-grids-alone-') as directory:
                work = Path(directory); index = fixture(work, invalid=invalid, with_other=False)
                browser = Chrome(work, index.parent / (index.name + '.parts/world.html'))
                try:
                    result = browser.evaluate('''(()=>{
                      const row=id=>[...document.querySelector('#rows').rows].find(r=>r.dataset.resourceId===id);
                      return ['graphics.firaga-copied-matrices','graphics.soil-remaining-rectangles']
                        .map(id=>!!row(id).querySelector('a[href="#tilemaps"]'));
                    })()''')
                    self.assertEqual(result, [invalid != 'firaga', invalid != 'soil'])
                finally: browser.close()
