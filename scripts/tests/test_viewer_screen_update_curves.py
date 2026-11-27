"""Check source-backed screen curves, aliases and scoped browser playback."""
import copy
import hashlib
import json
import shutil
import struct
import tempfile
import unittest
from pathlib import Path

from view_rom_data import generate, project_screen_update_curves, viewer_section_updates
from viewer_browser_harness import Chrome


def parts():
    result = [(offset, 'reader_literals', bytes(length)) for offset, length in (
        (0x11a17c, 4), (0x11a1cc, 4), (0x11a204, 4), (0x11a39c, 8),
        (0x11b874, 4), (0x11b8c4, 4), (0x11b8fc, 4), (0x11bab0, 4))]
    for root in (0x3a8e3c, 0x3aa1c4):
        raw = bytearray(3199)
        for index in range(4):
            struct.pack_into('<150H', raw, index * 300, *(index * 200 + i for i in range(150)))
        for index, slot in enumerate((1, 2, 7, 8)):
            struct.pack_into('<I', raw, 1200 + slot * 4, 0x08000000 + root + index * 300)
        for index in range(13):
            raw[1248 + index * 150:1248 + (index + 1) * 150] = bytes((index + i) % 256 for i in range(150))
        result.append((root, 'curves_and_directories', bytes(raw)))
    return result


def sources(chunks):
    return [{'offset': offset, 'role': role, 'length': len(raw), 'sha256': hashlib.sha256(raw).hexdigest()}
            for offset, role, raw in chunks]


def fixture(work, invalid=False):
    dataset = work / 'dataset'; dataset.mkdir(); chunks = parts(); content = b''.join(chunk for _, _, chunk in chunks)
    (dataset / 'screen-update-tables.bin').write_bytes(content)
    manifest = ('profile="europe"\nextractor_revision=361\ndecoder_revision=360\n'
                '[[resources]]\nid="graphics.screen-update-tables"\npath="screen-update-tables.bin"\n'
                f'format="screen-update-tables-v1-bin"\nsha256="{hashlib.sha256(content).hexdigest()}"\n')
    for index, item in enumerate(sources(chunks)):
        digest = '0' * 64 if invalid and index == 8 else item['sha256']
        manifest += (f'[[resources.sources]]\noffset={item["offset"]}\nlength={item["length"]}\n'
                     f'role="{item["role"]}"\nsha256="{digest}"\n')
    (dataset / 'manifest.toml').write_text(manifest)
    return generate(dataset, work / 'view', split=True)


def payload(path):
    return json.JSONDecoder().raw_decode(path.read_text().split('const data = ', 1)[1])[0]


class ScreenUpdateCurvesTest(unittest.TestCase):
    def test_values_and_final_aliases_match_the_source_layout(self):
        chunks = parts(); result = project_screen_update_curves(b''.join(c for _, _, c in chunks), sources(chunks))
        first, second = result['banks']; self.assertEqual(len(first['curves']), 17)
        self.assertEqual(first['curves'][0]['values'][:2], [0, 1])
        self.assertEqual(first['curves'][0]['values'][150], 200)
        self.assertEqual(first['curves'][3]['values'][150], 0)
        self.assertEqual(first['curves'][4]['values'][150], 1)
        self.assertEqual(first['curves'][16]['values'][150], 0)
        self.assertEqual([c['values'] for c in first['curves']], [c['values'] for c in second['curves']])

    def test_bad_source_hash_directory_and_copies_are_rejected(self):
        chunks = parts(); raw = b''.join(c for _, _, c in chunks); bad = sources(chunks); bad[8]['sha256'] = '0' * 64
        with self.assertRaisesRegex(ValueError, 'SHA-256'): project_screen_update_curves(raw, bad)
        bad = copy.deepcopy(chunks); value = bytearray(bad[8][2]); value[1204] ^= 1; bad[8] = (bad[8][0], bad[8][1], bytes(value))
        with self.assertRaisesRegex(ValueError, 'directory'): project_screen_update_curves(b''.join(c for _, _, c in bad), sources(bad))
        bad = copy.deepcopy(chunks); value = bytearray(bad[9][2]); value[20] ^= 1; bad[9] = (bad[9][0], bad[9][1], bytes(value))
        with self.assertRaisesRegex(ValueError, 'disagree'): project_screen_update_curves(b''.join(c for _, _, c in bad), sources(bad))

    def test_data_stays_inside_its_scope_and_invalid_sources_are_unavailable(self):
        for invalid in (False, True):
            with self.subTest(invalid=invalid), tempfile.TemporaryDirectory(prefix='viewer-curves-scope-') as directory:
                index = fixture(Path(directory), invalid); folder = index.parent / (index.name + '.parts')
                home, page, other = map(payload, (index, folder / 'interface-curves.html', folder / 'interface.html'))
                self.assertNotIn('screen_update_curves', home); self.assertNotIn('screen_update_curves', other)
                self.assertEqual(page['screen_update_curves'] is None, invalid)
                self.assertEqual(any(s['id'] == 'screen-update-curves' for p in home['wiki']['pages']
                                     if p['id'] == 'interface-curves' for s in p.get('sections', [])), not invalid)


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class ScreenUpdateCurvesBrowserTest(unittest.TestCase):
    def test_graph_playback_loop_table_badge_and_source_link(self):
        with tempfile.TemporaryDirectory(prefix='viewer-curves-browser-') as directory:
            work = Path(directory); index = fixture(work); page = index.parent / (index.name + '.parts') / 'interface-curves.html'
            browser = Chrome(work, page)
            try:
                expected = viewer_section_updates()['interface-curves']['screen-update-curves']
                self.assertTrue(browser.evaluate('''(()=>{const expected=EXPECTED,q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change'));
                  window.pending=new Map();let serial=0;window.setTimeout=(f,ms)=>{const id=++serial;pending.set(id,{f,ms});return id};
                  window.clearTimeout=id=>pending.delete(id);window.fire=()=>{const [id,t]=pending.entries().next().value;pending.delete(id);t.f();return t.ms};
                  const badge=q('#screen-update-curves>details>summary time');
                  if(!badge||badge.dateTime!==expected.at||!badge.textContent.startsWith(expected.kind==='added'?'Added ':'Updated ')||q('#screen-update-curves').hidden||
                    q('#screen-curve-id').options.length!==17||!q('[data-resource-id="graphics.screen-update-tables"] a[href="#screen-update-curves"]'))throw Error('section');
                  q('#screen-update-curves>details').open=true;q('#screen-curve-step').value='149';q('#screen-curve-step').dispatchEvent(new Event('input'));
                  q('#screen-curve-rate').value='10';q('#screen-curve-play').click();if(fire()!==100||q('#screen-curve-step').value!=='150'||
                    !q('#screen-curve-current').textContent.includes('aliases'))throw Error('final alias');fire();if(!q('#screen-curve-stop').disabled)throw Error('stop');
                  q('#screen-curve-loop').checked=true;q('#screen-curve-play').click();fire();if(q('#screen-curve-step').value!=='0')throw Error('loop');
                  q('#screen-curve-bank').value='1';change(q('#screen-curve-bank'));q('#screen-curve-id').value='16';change(q('#screen-curve-id'));
                  if(!q('#screen-curve-loop').checked||q('#screen-curve-rate').value!=='10')throw Error('settings');
                  q('#screen-curve-table').open=true;q('#screen-curve-table').dispatchEvent(new Event('toggle'));
                  if(q('#screen-curve-values').rows.length!==151||!q('#screen-curve-values').rows[150].textContent.includes('alias'))throw Error('table');
                  q('#screen-curve-stop').click();return true;})()'''.replace('EXPECTED', json.dumps(expected))))
            finally:
                browser.close()


if __name__ == '__main__':
    unittest.main()
