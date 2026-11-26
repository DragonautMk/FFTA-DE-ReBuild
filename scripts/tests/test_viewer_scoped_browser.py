"""Check offline scope pages, partitioned payloads and safe output ownership."""
import json
import shutil
import sys
import tempfile
import time
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from view_rom_data import generate, viewer_resource_scope, write_split_viewer
from test_viewer_obj_animations_browser import fixture
from viewer_browser_harness import Chrome


REPO = Path(__file__).resolve().parents[2]


def parse_page(path):
    text = path.read_text()
    return json.JSONDecoder().raw_decode(text.split('const data = ', 1)[1])[0]


def prepared(work):
    single = fixture(work, extra=True)
    data = parse_page(single)
    data['profile'] = 'europe'
    data['audio'] = {'path': 'wave.json', 'waves': [{'id': 7, 'samples': [0, 64, -64], 'loop_start': 0}]}
    data['short_texts'] = {'path': 'text.json', 'glyphs': [{'index': 176, 'width': 8, 'tile_words': [1] + [0] * 31}],
                           'languages': [{'language_id': 0, 'entries': [0]}],
                           'strings': [{'encoding': 'compact', 'units': [{'kind': 'glyph', 'index': 176}]}]}
    for resource_id, path in [('audio.m4a-tones', 'wave.json'), ('text.short-strings', 'text.json'),
                              ('engine.opaque', 'unknown.json')]:
        data['resources'].append({'id': resource_id, 'path': path, 'format': 'synthetic',
                                  'description': 'Synthetic resource', 'status': 'Available',
                                  'sources': [{'sentinel': 'PRIVATE_TEST_MARKER'}]})
    template = (REPO / 'view_rom_data.html').read_text()
    page = write_split_viewer(data, work / 'split', 'my viewer#demo.html', template)
    return data, page


class ScopePartitionTest(unittest.TestCase):
    def test_partition_ownership_and_single_page_compatibility(self):
        with tempfile.TemporaryDirectory(prefix='viewer-scope-partition-') as directory:
            work = Path(directory)
            data, page = prepared(work)
            parts = page.parent / (page.name + '.parts')
            home = parse_page(page)
            self.assertNotIn('audio', home)
            self.assertNotIn('short_texts', home)
            self.assertNotIn('PRIVATE_TEST_MARKER', page.read_text())
            self.assertEqual(len(home['resources']), len(data['resources']))
            audio = parse_page(parts / 'audio.html')
            text = parse_page(parts / 'text.html')
            objects = parse_page(parts / 'objects.html')
            self.assertEqual(audio['audio'], data['audio'])
            self.assertNotIn('short_texts', audio)
            self.assertEqual(text['short_texts'], data['short_texts'])
            self.assertNotIn('audio', text)
            self.assertEqual(objects['obj_animation_banks'], data['obj_animation_banks'])
            self.assertEqual(objects['obj_animation_extra'], data['obj_animation_extra'])
            self.assertEqual([r['id'] for r in audio['resources']], ['audio.m4a-tones'])
            self.assertEqual([r['id'] for r in text['resources']], ['text.short-strings'])
            self.assertIn('PRIVATE_TEST_MARKER', (parts / 'audio.html').read_text())
            self.assertNotIn('tree-load-large', page.read_text())
            self.assertNotIn('tree-load-large', (parts / 'text.html').read_text())
            self.assertTrue((parts / '.gitignore').is_file())
            self.assertEqual(viewer_resource_scope('graphics.unit-obj-normal-battle'), 'battle')
            self.assertEqual(viewer_resource_scope('graphics.unit-obj-script-override'), 'script-actors')
            self.assertEqual(viewer_resource_scope('graphics.layered-obj-animations'), 'objects')
            self.assertEqual(viewer_resource_scope('graphics.obj-dimensions'), 'actors')
            self.assertEqual(viewer_resource_scope('engine.unknown'), 'other')
            rebuilt = generate(work / 'dataset', work / 'generated', split=True)
            self.assertEqual(parse_page(rebuilt)['wiki']['current'], 'index')
            self.assertTrue(parse_page(rebuilt.parent / 'index.html.parts/objects.html')['obj_animation_banks'])
            legacy = generate(work / 'dataset', work / 'legacy')
            self.assertNotIn('wiki', parse_page(legacy))
            self.assertIn('tree-load-large', legacy.read_text())
            unowned = work / 'unowned'
            (unowned / 'index.html.parts').mkdir(parents=True)
            sentinel = unowned / 'index.html.parts/user-file'
            sentinel.write_text('keep')
            with self.assertRaisesRegex(ValueError, 'not owned'):
                write_split_viewer(data, unowned, 'index.html', (REPO / 'view_rom_data.html').read_text())
            self.assertEqual(sentinel.read_text(), 'keep')
            self.assertFalse((unowned / 'index.html').exists())
            invalid = dict(data, newly_added_data={'lost': 'no'})
            with self.assertRaisesRegex(ValueError, 'Unassigned viewer data'):
                write_split_viewer(invalid, work / 'invalid', 'index.html', (REPO / 'view_rom_data.html').read_text())


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class ScopeBrowserTest(unittest.TestCase):
    def test_navigation_loads_only_the_selected_scope(self):
        with tempfile.TemporaryDirectory(prefix='viewer-scope-browser-') as directory:
            work = Path(directory)
            _, page = prepared(work)
            browser = Chrome(work, page)
            try:
                def wait_for(current):
                    end = time.monotonic() + 30
                    while time.monotonic() < end:
                        try:
                            if browser.evaluate('document.readyState === "complete" && typeof data !== "undefined" && data.wiki.current === ' + json.dumps(current)):
                                return
                        except (RuntimeError, OSError):
                            pass
                        time.sleep(.05)
                    self.fail('Scope did not load: ' + current)
                self.assertTrue(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s),row=q('#rows [data-resource-id="audio.m4a-tones"]');
                  return !q('#wiki-pages').hidden && data.wiki.pages.length===16 &&
                    typeof data.audio==='undefined' && typeof data.short_texts==='undefined' &&
                    row.querySelector('a').textContent.includes('Browse Audio') && !q('#tree-load-large');
                })()'''))
                browser.click('#rows [data-resource-id="audio.m4a-tones"] a')
                wait_for('audio')
                self.assertTrue(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s);
                  return !q('#audio').hidden && q('#texts').hidden && typeof data.short_texts==='undefined' &&
                    data.resources.length===1 && q('#audio-id').value==='0' && !q('#tree-load-large');
                })()'''))
                browser.click('#wiki-page-links a[href$="/text.html"]')
                wait_for('text')
                self.assertTrue(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s);
                  return !q('#texts').hidden && !q('#text-search').hidden && q('#audio').hidden &&
                    typeof data.audio==='undefined' && data.resources.length===1 && q('#text-entry').value==='0';
                })()'''))
                browser.click('#wiki-page-links a[href$="/objects.html"]')
                wait_for('objects')
                self.assertTrue(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s);
                  return !q('#obj-animations').hidden && q('#audio').hidden && q('#texts').hidden &&
                    typeof data.audio==='undefined' && typeof data.short_texts==='undefined' &&
                    q('#obj-animation-catalog').options.length===3;
                })()'''))
                browser.click('#wiki-page-links > a')
                wait_for('index')
            finally:
                browser.close()
