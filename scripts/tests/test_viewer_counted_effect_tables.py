"""Validate counted effect tables and their offline visual playback."""
import copy
import hashlib
import json
import shutil
import struct
import tempfile
import unittest
from pathlib import Path

from view_rom_data import generate, project_counted_effect_tables, viewer_section_updates
from viewer_browser_harness import Chrome


COUNTS = (41, 126, 128, 128, 128, 128, 128, 64, 64, 240, 96, 80, 104)


def parts():
    literals = [(0x0e9ee0, 4), (0x0ea004, 4), (0x0ef930, 4), (0x0efa10, 4),
                (0x0efc18, 4), (0x0efe08, 4), (0x0f0024, 4), (0x0f6664, 4),
                (0x0fd1cc, 4), (0x113258, 4), (0x113398, 4), (0x114600, 4), (0x11db34, 8)]
    segments = [(offset, 'reader_literals', bytes(length)) for offset, length in literals]
    chunks, roots, root = [], [], 0xb4b0f4
    for table, count in enumerate(COUNTS):
        roots.append(root)
        pairs = bytes(value for sample in range(count) for value in ((table * 19 + sample * 3) & 255,
                                                                      (255 - table * 7 - sample * 5) & 255))
        raw = struct.pack('<H', count) + pairs; end = root + len(raw)
        if chunks and chunks[-1][0] + len(chunks[-1][1]) == root:
            chunks[-1] = (chunks[-1][0], chunks[-1][1] + raw)
        else:
            chunks.append((root, raw))
        root = (end + 3) & ~3
    assert root == 0xb4bc84
    segments.append((0x3a7db4, 'table_directory', struct.pack('<6I', *(0x08000000 + root for root in roots[1:7]))))
    segments.extend((offset, 'counted_effect_tables', raw) for offset, raw in chunks)
    return segments


def sources(chunks):
    return [{'offset': offset, 'role': role, 'length': len(raw), 'sha256': hashlib.sha256(raw).hexdigest()}
            for offset, role, raw in chunks]


def fixture(work, invalid=False):
    dataset = work / 'dataset'; dataset.mkdir(); chunks = parts(); content = b''.join(raw for _, _, raw in chunks)
    (dataset / 'counted-effect-tables.bin').write_bytes(content)
    manifest = ('profile="europe"\nextractor_revision=362\ndecoder_revision=361\n'
                '[[resources]]\nid="graphics.counted-effect-tables"\npath="counted-effect-tables.bin"\n'
                f'format="counted-effect-tables-v1-bin"\nsha256="{hashlib.sha256(content).hexdigest()}"\n')
    for index, item in enumerate(sources(chunks)):
        digest = '0' * 64 if invalid and index == 14 else item['sha256']
        manifest += (f'[[resources.sources]]\noffset={item["offset"]}\nlength={item["length"]}\n'
                     f'role="{item["role"]}"\nsha256="{digest}"\n')
    (dataset / 'manifest.toml').write_text(manifest)
    return generate(dataset, work / 'view', split=True)


def payload(path):
    return json.JSONDecoder().raw_decode(path.read_text().split('const data = ', 1)[1])[0]


class CountedEffectProjectionTest(unittest.TestCase):
    def test_all_counts_directory_and_reader_contracts(self):
        chunks = parts(); result = project_counted_effect_tables(b''.join(raw for _, _, raw in chunks), sources(chunks))
        self.assertEqual([len(table['samples']) for table in result['tables']], list(COUNTS))
        self.assertEqual(sum(len(table['samples']) for table in result['tables']), 1455)
        self.assertEqual([table['directory_slot'] for table in result['tables'][1:7]], list(range(6)))
        self.assertEqual([result['tables'][index]['reader'] for index in (0, 1, 7, 8, 9)],
                         ['halfword', 'reversed', 'bias88', 'bias120', 'halfword'])
        self.assertEqual(result['tables'][1]['samples'][0], [19, 248])

    def test_bad_hash_directory_count_and_extent_are_rejected(self):
        chunks = parts(); content = b''.join(raw for _, _, raw in chunks); invalid = sources(chunks); invalid[14]['sha256'] = '0' * 64
        with self.assertRaisesRegex(ValueError, 'SHA-256'): project_counted_effect_tables(content, invalid)
        for index, relative, match in ((13, 0, 'directory'), (14, 0, 'count')):
            bad = copy.deepcopy(chunks); raw = bytearray(bad[index][2]); raw[relative] ^= 1; bad[index] = (bad[index][0], bad[index][1], bytes(raw))
            with self.assertRaisesRegex(ValueError, match): project_counted_effect_tables(b''.join(raw for _, _, raw in bad), sources(bad))
        bad = copy.deepcopy(chunks); bad[14] = (bad[14][0] + 2, bad[14][1], bad[14][2])
        with self.assertRaises(ValueError): project_counted_effect_tables(b''.join(raw for _, _, raw in bad), sources(bad))

    def test_page_is_scoped_and_invalid_sources_remain_unavailable(self):
        for invalid in (False, True):
            with self.subTest(invalid=invalid), tempfile.TemporaryDirectory(prefix='viewer-counted-effect-scope-') as directory:
                index = fixture(Path(directory), invalid); folder = index.parent / (index.name + '.parts')
                home, page, other = map(payload, (index, folder / 'interface-effects.html', folder / 'interface.html'))
                self.assertNotIn('counted_effect_tables', home); self.assertNotIn('counted_effect_tables', other)
                self.assertEqual(page['counted_effect_tables'] is None, invalid)
                self.assertEqual(any(s['id'] == 'counted-effect-tables' for p in home['wiki']['pages']
                                     if p['id'] == 'interface-effects' for s in p.get('sections', [])), not invalid)


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class CountedEffectBrowserTest(unittest.TestCase):
    def test_graph_values_reader_modes_playback_and_settings(self):
        with tempfile.TemporaryDirectory(prefix='viewer-counted-effect-browser-') as directory:
            work = Path(directory); index = fixture(work); page = index.parent / (index.name + '.parts') / 'interface-effects.html'
            browser = Chrome(work, page)
            try:
                expected = viewer_section_updates()['interface-effects']['counted-effect-tables']
                self.assertTrue(browser.evaluate('''(()=>{const expected=EXPECTED,q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change'));
                  window.pending=new Map();let serial=0;window.setTimeout=(f,ms)=>{const id=++serial;pending.set(id,{f,ms});return id};
                  window.clearTimeout=id=>pending.delete(id);window.fire=()=>{const [id,t]=pending.entries().next().value;pending.delete(id);t.f();return t.ms};
                  const badge=q('#counted-effect-tables>details>summary time');
                  if(!badge||badge.dateTime!==expected.at||!badge.textContent.startsWith('Added ')||q('#counted-effect-table').options.length!==13||
                    !q('[data-resource-id="graphics.counted-effect-tables"] a[href="#counted-effect-tables"]'))throw Error('section');
                  q('#counted-effect-tables').closest('.section-group').open=true;q('#counted-effect-tables>details').open=true;
                  const canvas=q('#counted-effect-map'),pixel=canvas.getContext('2d').getImageData(100,0,1,1).data;
                  if(pixel[0]!==21||pixel[1]!==93||pixel[2]!==122||!q('#counted-effect-current').textContent.includes('0xFF00'))throw Error('source image');
                  q('#counted-effect-table').value='1';change(q('#counted-effect-table'));
                  if(!q('#counted-effect-current').textContent.includes('reversed pair copy: 0x13F8'))throw Error('reversed');
                  q('#counted-effect-loop').checked=true;q('#counted-effect-rate').value='10';q('#counted-effect-play').click();
                  if(fire()!==100||q('#counted-effect-sample').value!=='1')throw Error('play');
                  q('#counted-effect-table').value='8';change(q('#counted-effect-table'));
                  if(!q('#counted-effect-loop').checked||q('#counted-effect-rate').value!=='10'||q('#counted-effect-stop').disabled)throw Error('settings');
                  q('#counted-effect-sample').value='0';change(q('#counted-effect-sample'));
                  if(!q('#counted-effect-current').textContent.includes('0x103F'))throw Error('bias');
                  q('#counted-effect-values-panel').open=true;q('#counted-effect-values-panel').dispatchEvent(new Event('toggle'));
                  if(q('#counted-effect-values').rows.length!==64||q('#counted-effect-values').rows[0].cells[5].textContent!=='0x103F')throw Error('table');
                  q('#counted-effect-stop').click();return true;})()'''.replace('EXPECTED', json.dumps(expected))))
            finally:
                browser.close()


if __name__ == '__main__':
    unittest.main()
