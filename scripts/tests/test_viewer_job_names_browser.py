"""Verify direct selected-job name joins and offline cross-page links."""
import hashlib
import json
import shutil
import sys
import tempfile
import time
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from view_rom_data import generate, project_job_metadata, project_slot_job_lookup, project_slot_job_names
from viewer_browser_harness import Chrome


def fixture(work):
    dataset = work / 'dataset'
    dataset.mkdir()
    slots = {'source_offset': 0x399750, 'entries': [{'slot': i, 'job_id': i % 2} for i in range(20)]}
    records = [{'bytes_00_04': [0, 0, 0, 0, 0], 'routing': 0,
                'bytes_06_1f': [0] * 26, 'bytes_20_33': [0] * 20} for _ in range(116)]
    records[1]['bytes_00_04'][0] = 1
    metadata = {'source_offset': 0x6e5464, 'records': records}
    glyphs = [{'index': i, 'width': 8, 'tile_words': [1] + [0] * 31} for i in (176, 203, 205, 1000)]
    first = [{'kind': 'glyph', 'index': 176}, {'kind': 'glyph', 'index': 203},
             {'kind': 'control_byte', 'value': 0x40}, {'kind': 'glyph', 'index': 205},
             {'kind': 'glyph', 'index': 1000}]
    second = [{'kind': 'glyph', 'index': 205}]
    texts = {'glyphs': glyphs, 'strings': [{'source_offset': 10, 'encoding': 'compact', 'units': first},
                                          {'source_offset': 20, 'encoding': 'compact', 'units': second}],
             'pointer_catalogs': [{'reader': 0x80165e8, 'languages': [
                 {'source_offset': 100, 'entries': [0, 1]}, {'source_offset': 200, 'entries': [1, 0]}]}],
             'archive_catalogs': []}
    manifest = 'profile="europe"\nextractor_revision=1\ndecoder_revision=1\n'
    for resource_id, name, fmt, value in [
        ('gameplay.slot-job-lookup', 'slot-job-lookup.json', 'slot-job-lookup-v1-json', slots),
        ('gameplay.linked-metadata52', 'linked-metadata52.json', 'linked-metadata52-v1-json', metadata),
        ('text.catalogs', 'text-catalogs.json', 'text-catalogs-v1-json', texts),
    ]:
        content = json.dumps(value).encode()
        (dataset / name).write_bytes(content)
        manifest += (f'[[resources]]\nid="{resource_id}"\npath="{name}"\n'
                     f'format="{fmt}"\nsha256="{hashlib.sha256(content).hexdigest()}"\n')
    (dataset / 'manifest.toml').write_text(manifest)
    return dataset, slots, metadata


def payload(page):
    return json.JSONDecoder().raw_decode(page.read_text().split('const data = ', 1)[1])[0]


class JobNameProjectionTest(unittest.TestCase):
    def test_direct_only_and_missing_dependencies(self):
        with tempfile.TemporaryDirectory(prefix='viewer-job-projection-') as directory:
            work = Path(directory)
            dataset, slots, metadata = fixture(work)
            result = generate(dataset, work / 'view', split=True)
            world = payload(result.parent / 'index.html.parts/world.html')
            jobs = world['job_names']
            self.assertEqual(len(jobs['rows']), 20)
            self.assertEqual(jobs['rows'][0]['name_index'], 0)
            self.assertEqual(jobs['rows'][1]['name_index'], 1)
            self.assertEqual(jobs['rows'][0]['names'][0], [176, 203, [0x40], 205, 1000])
            self.assertEqual(jobs['rows'][0]['names'][1], [205])
            self.assertEqual([row['job_id'] for row in jobs['reference_rows']], [104, 105, 106])
            self.assertEqual(jobs['reference_rows'][0]['names'][0], [176, 203, [0x40], 205, 1000])
            self.assertNotIn('pointer_texts', world)
            slots = project_slot_job_lookup(slots); slots['path'] = 'slot.json'
            metadata = project_job_metadata(metadata); metadata['path'] = 'metadata.json'
            pointer = payload(result.parent / 'index.html.parts/text.html')['pointer_texts']
            metadata['records'][1]['routing'] = 255
            partial = project_slot_job_names(slots, metadata, pointer)
            self.assertIsNone(partial['rows'][1]['name_index'])
            self.assertEqual(partial['rows'][1]['names'], [None, None])
            metadata['records'][105]['routing'] = 255
            redirected = project_slot_job_names(slots, metadata, pointer)
            self.assertIsNone(redirected['reference_rows'][1]['name_index'])
            self.assertEqual(redirected['reference_rows'][1]['names'], [None, None])
            missing = project_slot_job_names(slots, None, pointer)
            self.assertEqual(missing['languages'], [])
            self.assertIsNone(missing['rows'][0]['name_index'])
            self.assertEqual(missing['reference_rows'], [])
            slots['entries'][0]['slot'] = True
            with self.assertRaisesRegex(ValueError, 'slot-job'):
                project_slot_job_lookup(slots)
            metadata['records'][0]['bytes_00_04'][0] = True
            with self.assertRaisesRegex(ValueError, 'metadata bytes'):
                project_job_metadata(metadata)


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class JobNameBrowserTest(unittest.TestCase):
    def test_name_display_language_and_cross_page_navigation(self):
        with tempfile.TemporaryDirectory(prefix='viewer-job-browser-') as directory:
            work = Path(directory)
            dataset, _, _ = fixture(work)
            result = generate(dataset, work / 'view', split=True)
            browser = Chrome(work, result.parent / 'index.html.parts/world.html')
            try:
                self.assertTrue(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change'));
                  if(q('#job-names').hidden || q('#job-names>details').open || q('#job-name-rows').rows.length!==20 ||
                     q('#job-name-rows').rows[0].cells[2].textContent!=='Ab d' ||
                     q('#job-name-rows').rows[0].cells[2].querySelectorAll('canvas').length!==1 ||
                     !q('#job-name-rows').rows[0].cells[2].querySelector('.text-gap') ||
                     q('#job-documented').hidden || q('#job-documented').open || q('#job-reference-rows').rows.length!==3 ||
                     q('#job-reference-rows').rows[0].cells[0].textContent!=='104' ||
                     q('#job-reference-rows').rows[0].cells[1].textContent!=='Ab d' ||
                     typeof data.pointer_texts!=='undefined')return false;
                  q('#job-name-language').value='1';change(q('#job-name-language'));
                  return q('#job-name-rows').rows[0].cells[2].textContent==='d' &&
                    q('#job-name-rows').rows[1].cells[2].textContent==='Ab d' &&
                    q('#job-reference-rows').rows[0].cells[1].textContent==='d';
                })()'''))
                browser.evaluate("document.querySelector('#job-names>details').open=true")
                href = browser.evaluate("document.querySelector('#job-name-rows tr:first-child a').href")
                hit = browser.evaluate('''(()=>{
                  const a=document.querySelector('#job-name-rows tr:first-child a');a.scrollIntoView({block:'center'});
                  const r=a.getClientRects()[0],x=r.left+r.width/2,y=r.top+r.height/2,e=document.elementFromPoint(x,y);
                  return {x,y,width:innerWidth,height:innerHeight,hit:e?e.tagName:null,match:e===a||a.contains(e)};
                })()''')
                self.assertTrue(hit['match'], 'Name link is not clickable: ' + json.dumps(hit))
                point = {'x': hit['x'], 'y': hit['y']}
                browser.command('Input.dispatchMouseEvent', {'type': 'mouseMoved', **point})
                browser.command('Input.dispatchMouseEvent', {'type': 'mousePressed', **point, 'button': 'left', 'clickCount': 1})
                browser.command('Input.dispatchMouseEvent', {'type': 'mouseReleased', **point, 'button': 'left', 'clickCount': 1})
                end = time.monotonic() + 30
                while time.monotonic() < end:
                    try:
                        if browser.evaluate("document.readyState==='complete' && typeof data!=='undefined' && data.wiki.current==='text'"):
                            break
                    except (RuntimeError, OSError):
                        pass
                    time.sleep(.05)
                else:
                    state = browser.evaluate("({url:location.href, ready:document.readyState, current:typeof data==='undefined'?null:data.wiki.current})")
                    self.fail('Name string page did not load: ' + json.dumps({'href': href, 'state': state}))
                self.assertTrue(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s);
                  return q('#pointer-texts>details').open && q('#pointer-reader').value==='0' &&
                    q('#pointer-language').value==='1' && q('#pointer-entry').value==='0';
                })()'''))
            finally:
                browser.close()
