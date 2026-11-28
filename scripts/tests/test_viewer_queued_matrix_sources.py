"""Verify the three queued matrices and their language-dependent row links."""
import hashlib
import json
from pathlib import Path
import shutil
import struct
import tempfile
import unittest

from test_viewer_direct_rectangle_bank import pack_segments, payload
from view_rom_data import generate, project_queued_matrix_sources, viewer_section_updates
from viewer_browser_harness import Chrome


def chunks():
    pairs = (0x02f33c, 0x02f7c8, 0x02faa4, 0x02fc3c, 0x0300f8, 0x0303b4)
    return [(0x02df28, 'queue_literals', struct.pack('<I', 0x083ba048)),
            *((offset, 'queue_literals', struct.pack('<2I', 0x08437194, 0x08448278)) for offset in pairs),
            (0x3ba048, 'queue_cells', struct.pack('<72H', *range(72))),
            (0x437194, 'queue_cells', struct.pack('<182H', *range(182))),
            (0x448278, 'queue_cells', struct.pack('<182H', *range(1000, 1182)))]


def fixture(work, *, bad=False, legacy=False):
    dataset = work / 'dataset'; dataset.mkdir()
    content, sources = pack_segments(chunks())
    path = dataset / 'queued-matrix-sources.bin'; path.write_bytes(content)
    manifest = ('profile="europe"\nextractor_revision=379\ndecoder_revision=378\n[[resources]]\n'
                'id="graphics.queued-matrix-sources"\nformat="queued-matrix-sources-v1-bin"\n'
                f'path="{path.name}"\nsha256="{hashlib.sha256(content).hexdigest()}"\n')
    if bad:
        sources[-1]['sha256'] = '0' * 64
    for source in sources:
        manifest += '[[resources.sources]]\n' + ''.join(f'{key}={json.dumps(value)}\n' for key, value in source.items())
    (dataset / 'manifest.toml').write_text(manifest)
    return generate(dataset, work / 'view', split=not legacy)


class QueuedMatrixProjectionTest(unittest.TestCase):
    def test_exact_widths_rows_cells_and_staging(self):
        result = project_queued_matrix_sources(*pack_segments(chunks()))
        self.assertEqual([(row['source_offset'], row['width'], row['height']) for row in result['maps']],
                         [(0x3ba048, 18, 4), (0x437194, 26, 7), (0x448278, 26, 7)])
        self.assertEqual([item['cells'] for item in result['maps']],
                         [list(range(72)), list(range(182)), list(range(1000, 1182))])
        self.assertEqual([(path['first_row'], path['row_count']) for path in result['paths']],
                         [(0, 2), (3, 2), (2, 1), (None, None), (5, 2), (None, None)])
        self.assertEqual(result['initializer']['task_start'] - result['initializer']['queued_start'], 3 * 36)
        self.assertEqual(result['destination_stride'], 64)

    def test_refuses_bad_span_and_coherently_modified_literal(self):
        parts = chunks()
        parts[1] = (parts[1][0], parts[1][1], parts[1][2][4:] + parts[1][2][:4])
        with self.assertRaisesRegex(ValueError, 'pointers'):
            project_queued_matrix_sources(*pack_segments(parts))
        parts = chunks()
        parts[-1] = (parts[-1][0], parts[-1][1], parts[-1][2][:-2])
        with self.assertRaisesRegex(ValueError, 'sources'):
            project_queued_matrix_sources(*pack_segments(parts))

    def test_scope_and_failed_binding(self):
        for bad in (False, True):
            with self.subTest(bad=bad), tempfile.TemporaryDirectory() as directory:
                index = fixture(Path(directory), bad=bad)
                parts = index.parent / (index.name + '.parts')
                home = payload(index)
                self.assertNotIn('queued_matrix_sources', home)
                self.assertNotIn('queued_matrix_sources', payload(parts / 'interface.html'))
                matrix = payload(parts / 'interface-queued-matrices.html')
                self.assertNotIn('bg_graphics', matrix)
                self.assertNotIn('queued_ui_maps', matrix)
                self.assertEqual(matrix['queued_matrix_sources'] is None, bad)
                self.assertEqual(matrix['resources'][0]['status'].startswith('Invalid queued matrix'), bad)
                self.assertEqual(len([p for p in home['wiki']['pages'] if p['id'] == 'interface-queued-matrices']), 1)


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class QueuedMatrixBrowserTest(unittest.TestCase):
    def test_language_paths_highlight_verified_rows_and_preserve_cells(self):
        for legacy in (False, True):
            with self.subTest(legacy=legacy), tempfile.TemporaryDirectory() as directory:
                work = Path(directory); index = fixture(work, legacy=legacy)
                page = index if legacy else index.parent / (index.name + '.parts/interface-queued-matrices.html')
                browser = Chrome(work, page)
                try:
                    stamp = viewer_section_updates()['interface-queued-matrices']['tilemaps']['at']
                    result = browser.evaluate('''(()=>{
                      const q=s=>document.querySelector(s), picker=q('#queued-matrix-language'), grid=q('#tilemap-cells');
                      const rows=q('#queued-matrix-path-rows').rows, map=q('#tilemap-id');
                      const highlighted=()=>[...grid.querySelectorAll('.copy-source-row')].map(r=>r.rowIndex).join();
                      const select=id=>{picker.value=String(id);picker.dispatchEvent(new Event('change'))};
                      if(rows.length!==7||map.options.length!==3||!q('#queued-matrix-staging').textContent.includes('rows 3–6'))
                        throw Error('matrix count or staging');
                      if(!picker.options[1].textContent.includes('French'))throw Error('locale label');
                      for(let language=0;language<5;language++){
                        select(language);
                        const matrix=language===1?'0x448278':'0x437194';
                        if([...rows].slice(1).some(row=>row.cells[4].textContent!==matrix))throw Error('language '+language);
                        if(rows[1].cells[3].textContent!==(language===1?'0x2F340':'0x2F33C'))throw Error('literal');
                      }
                      select(1);rows[2].querySelector('a').click();
                      if(map.value!=='2'||highlighted()!=='3,4'||!q('#tilemaps>details').open||
                         !q('#tilemap-cell-detail').textContent.includes('Cell (0, 3) · raw 0x0436 · source 0x448314'))
                        throw Error('French row link');
                      rows[6].querySelector('a').click();
                      if(highlighted()!=='')throw Error('selection-only path');
                      rows[0].querySelector('a').click();
                      if(highlighted()!=='0,1,2,3'||grid.querySelectorAll('button').length!==72||map.value!=='0')throw Error('initializer row link');
                      for(const id of ['tilemaps','queued-matrix-paths']){
                        if(q('#'+id+'>details>summary time').dateTime!==STAMP)throw Error('badge '+id);
                      }
                      q('#collapse-all').click();
                      return q('#tilemaps>details').open?'open':'PASS';
                    })()'''.replace('STAMP', json.dumps(stamp)))
                    self.assertEqual(result, 'PASS')
                finally:
                    browser.close()
