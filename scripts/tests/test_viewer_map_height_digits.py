"""Verify the joined EUR map-height digit bank and its interpreted previews."""
import copy
import hashlib
import json
import shutil
import struct
import tempfile
import unittest
from pathlib import Path

from view_rom_data import generate, project_map_height_digits, viewer_section_updates
from viewer_browser_harness import Chrome


def glyph(index):
    data = bytearray(256)
    for tile in range(8):
        data[tile * 32] = ((tile + 2) << 4) | (index + 1)
    return bytes(data)


def source(offset, role, raw):
    return {'offset': offset, 'role': role, 'length': len(raw), 'sha256': hashlib.sha256(raw).hexdigest()}


def pieces():
    all_glyphs = [glyph(index) for index in range(10)]
    new = [(0x0ac9a4, 'queue_literals', struct.pack('<I', 0x06013600)),
           (0x40e344, 'glyph_tiles', b''.join(all_glyphs[:5])),
           (0x40e944, 'glyph_tiles', b''.join(all_glyphs[6:]))]
    previous = {'source_base': 0x0840e344, 'destination': 0x06013700, 'selected_index': 5,
                'halfwords': list(struct.unpack('<128H', all_glyphs[5]))}
    old = [(0x0ac9a0, 'firaga-queue-source-base', struct.pack('<I', previous['source_base'])),
           (0x0ac9a8, 'firaga-queue-destination', struct.pack('<I', previous['destination'])),
           (0x40e844, 'firaga-queue-halfwords', all_glyphs[5])]
    return new, previous, old


def project(new, previous=None, old=None):
    return project_map_height_digits(b''.join(p[2] for p in new), [source(*p) for p in new],
                                     json.dumps(previous).encode() if previous else None,
                                     [source(*p) for p in old] if old else None)


def fixture(work, prior='valid', invalid_new=False, split=True, profile='europe'):
    dataset = work / 'dataset'; dataset.mkdir(); new, previous, old = pieces()
    items = [('graphics.map-height-digits', 'height.bin', 'map-height-digits-v1-bin', b''.join(p[2] for p in new), [source(*p) for p in new])]
    if prior != 'missing':
        if prior == 'wrong-value': previous['halfwords'][0] ^= 1
        items.append(('engine.firaga-queued-transfer', 'previous.json', 'firaga-queued-transfer-v1-json',
                      json.dumps(previous).encode(), [source(*p) for p in old]))
    manifest = f'profile="{profile}"\nextractor_revision=368\ndecoder_revision=367\n'
    for identity, path, fmt, content, entries in items:
        (dataset / path).write_bytes(content)
        manifest += f'[[resources]]\nid="{identity}"\npath="{path}"\nformat="{fmt}"\nsha256="{hashlib.sha256(content).hexdigest()}"\n'
        for index, entry in enumerate(entries):
            digest = '0' * 64 if invalid_new and identity == 'graphics.map-height-digits' and index == 2 else entry['sha256']
            manifest += (f'[[resources.sources]]\noffset={entry["offset"]}\nlength={entry["length"]}\n'
                         f'role="{entry["role"]}"\nsha256="{digest}"\n')
    (dataset / 'manifest.toml').write_text(manifest)
    return generate(dataset, work / 'view', split=split)


def payload(path):
    return json.JSONDecoder().raw_decode(path.read_text().split('const data = ', 1)[1])[0]


class MapHeightProjectionTest(unittest.TestCase):
    def test_join_all_digits_and_keep_nine_if_previous_is_missing_or_invalid(self):
        new, previous, old = pieces(); result = project(new, previous, old)
        self.assertTrue(result['digit_five_available'])
        self.assertEqual([d['source_offset'] for d in result['digits']], [0x40e344 + 256 * index for index in range(10)])
        self.assertEqual([bytes(d['bytes']) for d in result['digits']], [glyph(index) for index in range(10)])
        for prior, old_sources in ((None, None), ({**previous, 'selected_index': 4}, old),
                                   ({**previous, 'halfwords': [0] * 128}, old)):
            incomplete = project(new, prior, old_sources)
            self.assertFalse(incomplete['digit_five_available']); self.assertIsNone(incomplete['digits'][5]['bytes'])
            self.assertEqual(bytes(incomplete['digits'][6]['bytes']), glyph(6))

    def test_reject_new_hash_destination_and_extents(self):
        new, previous, old = pieces(); data = b''.join(p[2] for p in new); entries = [source(*p) for p in new]
        entries[1]['sha256'] = '0' * 64
        with self.assertRaisesRegex(ValueError, 'SHA-256'): project_map_height_digits(data, entries)
        bad = copy.deepcopy(new); bad[0] = (*bad[0][:2], struct.pack('<I', 0x06013700))
        with self.assertRaisesRegex(ValueError, 'destination'): project(bad, previous, old)
        bad = copy.deepcopy(new); bad[1] = (bad[1][0] + 2, *bad[1][1:])
        with self.assertRaisesRegex(ValueError, 'sources'): project(bad, previous, old)

    def test_scope_partial_dependency_and_wrong_region(self):
        for prior, invalid, profile in (('valid', False, 'europe'), ('missing', False, 'europe'),
                                        ('wrong-value', False, 'europe'), ('valid', True, 'europe'), ('valid', False, 'usa')):
            with self.subTest(prior=prior, invalid=invalid, profile=profile), tempfile.TemporaryDirectory(prefix='viewer-height-scope-') as directory:
                index = fixture(Path(directory), prior, invalid, profile=profile); folder = index.parent / (index.name + '.parts')
                home, page, world = map(payload, (index, folder / 'interface-height.html', folder / 'world.html'))
                self.assertNotIn('map_height_digits', home); self.assertNotIn('map_height_digits', world)
                valid = not invalid and profile == 'europe'
                self.assertEqual(page['map_height_digits'] is not None, valid)
                if valid:
                    self.assertEqual(page['map_height_digits']['digit_five_available'], prior == 'valid')
                sections = [s['id'] for p in home['wiki']['pages'] if p['id'] == 'interface-height' for s in p['sections']]
                self.assertEqual(sections, ['map-height-digits'] if valid else [])


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class MapHeightBrowserTest(unittest.TestCase):
    def test_single_page_retains_joined_digit_preview(self):
        with tempfile.TemporaryDirectory(prefix='viewer-height-single-') as directory:
            work = Path(directory); page = fixture(work, split=False); browser = Chrome(work, page)
            try:
                self.assertTrue(browser.evaluate('''(()=>{const q=s=>document.querySelector(s);
                  q('#map-height-value').value='50';q('#map-height-value').dispatchEvent(new Event('input'));
                  return !q('#map-height-digits').hidden&&q('#map-height-description').textContent.includes('digit 5 at 0x40E844')&&
                    q('#map-height-description').textContent.includes('digit 0 at 0x40E344');})()'''))
            finally:
                browser.close()

    def test_digits_selection_colors_and_collapse(self):
        with tempfile.TemporaryDirectory(prefix='viewer-height-browser-') as directory:
            work = Path(directory); index = fixture(work); page = index.parent / (index.name + '.parts') / 'interface-height.html'
            browser = Chrome(work, page)
            try:
                expected = viewer_section_updates()['interface-height']['map-height-digits']
                self.assertTrue(browser.evaluate('''(()=>{const expected=EXPECTED,q=s=>document.querySelector(s),event=(e,n)=>e.dispatchEvent(new Event(n));
                  const badge=q('#map-height-digits>details>summary time'),preview=q('#map-height-preview'),pixel=(canvas,x,y)=>[...canvas.getContext('2d').getImageData(x,y,1,1).data].join(',');
                  if(!badge||badge.dateTime!==expected.at||!badge.textContent.startsWith('Added ')||q('#map-height-gallery').children.length!==10)throw Error('section');
                  if(pixel(preview,0,0)!=='0,0,0,0'||pixel(preview,16,0)!=='0,0,0,255'||!q('#map-height-description').textContent.includes('Tens: cleared'))throw Error('zero');
                  q('#map-height-value').value='55';event(q('#map-height-value'),'input');
                  if(!q('#map-height-description').textContent.includes('digit 5 at 0x40E844')||pixel(preview,0,0)!=='0,0,0,255')throw Error('fives');
                  q('#map-height-colors').value='indices';event(q('#map-height-colors'),'change');
                  if(pixel(preview,0,0)!=='102,102,102,255'||pixel(preview,1,0)!=='34,34,34,255'||pixel(preview,8,0)!=='102,102,102,255')throw Error('tiles');
                  q('#map-height-gallery').children[5].click();
                  if(q('#map-height-inspect').value!=='5'||!q('#map-height-inspect-source').textContent.includes('previous.json')||
                    q('#map-height-halfwords').textContent.split('\\n').length!==16)throw Error('source');
                  q('#map-height-value').value='100';event(q('#map-height-value'),'input');
                  if(!q('#map-height-description').textContent.includes('0 to 99'))throw Error('bounds');
                  q('#map-height-digits>details').open=true;q('#collapse-all').click();return !q('#map-height-digits>details').open;})()'''.replace('EXPECTED', json.dumps(expected))))
            finally:
                browser.close()

    def test_missing_five_keeps_other_digits_usable(self):
        with tempfile.TemporaryDirectory(prefix='viewer-height-missing-') as directory:
            work = Path(directory); index = fixture(work, prior='missing'); page = index.parent / (index.name + '.parts') / 'interface-height.html'
            browser = Chrome(work, page)
            try:
                self.assertTrue(browser.evaluate('''(()=>{const q=s=>document.querySelector(s);q('#map-height-value').value='15';q('#map-height-value').dispatchEvent(new Event('input'));
                  q('#map-height-inspect').value='5';q('#map-height-inspect').dispatchEvent(new Event('change'));
                  return q('#map-height-description').textContent.includes('digit 5 unavailable')&&
                    q('#map-height-inspect-source').textContent.includes('unavailable')&&q('#map-height-halfwords').textContent==='Unavailable.'&&
                    q('#map-height-gallery').children.length===10;})()'''))
            finally:
                browser.close()


if __name__ == '__main__':
    unittest.main()
