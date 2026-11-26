"""Check bounded signed map offsets and their independent offline view."""
import copy
import hashlib
import json
import shutil
import sys
import tempfile
import time
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from view_rom_data import generate, project_signed_map_deltas
from test_viewer_ui_rectangles_browser import catalog as rectangle_catalog
from viewer_browser_harness import Chrome


ENTRIES = ([-128, -1, 0, 1, 127] * 10)[:48]


def fixture(work, value, *, profile='europe', split=True, bad_hash=False):
    dataset = work / 'dataset'; dataset.mkdir()
    content = json.dumps(value).encode(); (dataset / 'ui-raw-transfers.json').write_bytes(content)
    digest = '0' * 64 if bad_hash else hashlib.sha256(content).hexdigest()
    manifest = (f'profile="{profile}"\nextractor_revision=338\ndecoder_revision=337\n'
                '[[resources]]\nid="graphics.ui-raw-transfers"\npath="ui-raw-transfers.json"\n'
                f'format="ui-raw-transfers-v1-json"\nsha256="{digest}"\n')
    (dataset / 'manifest.toml').write_text(manifest)
    return generate(dataset, work / 'view', split=split)


def catalog():
    result = rectangle_catalog()
    result['signed_map_deltas'] = {'caller_literal': 0x083954a4, 'entries': list(ENTRIES)}
    return result


def payload(path):
    return json.JSONDecoder().raw_decode(path.read_text().split('const data = ', 1)[1])[0]


class SignedMapDeltaProjectionTest(unittest.TestCase):
    def test_valid_and_invalid_values(self):
        value = catalog(); projection = project_signed_map_deltas(value)
        self.assertEqual(projection, {'source_offset': 0x3954a4, 'caller_literal': 0x083954a4, 'entries': ENTRIES})
        self.assertEqual(value, catalog())
        mutations = [lambda c: c['signed_map_deltas'].update(caller_literal=0),
                     lambda c: c['signed_map_deltas']['entries'].pop(),
                     lambda c: c['signed_map_deltas']['entries'].__setitem__(0, True),
                     lambda c: c['signed_map_deltas']['entries'].__setitem__(0, -129),
                     lambda c: c['signed_map_deltas']['entries'].__setitem__(0, 128),
                     lambda c: c.update(signed_map_deltas=[])]
        for mutate in mutations:
            bad = copy.deepcopy(value); mutate(bad)
            with self.subTest(mutation=mutate), self.assertRaises(ValueError):
                project_signed_map_deltas(bad)

    def test_scope_legacy_and_independent_bad_sources(self):
        for case in ('valid', 'legacy', 'invalid-deltas', 'invalid-rectangles', 'other-profile', 'bad-hash'):
            value = catalog(); profile = 'europe'
            if case == 'legacy': del value['signed_map_deltas']
            if case == 'invalid-deltas': value['signed_map_deltas']['entries'][0] = 128
            if case == 'invalid-rectangles': value['halfword_copies']['tilemap_rectangles']['rectangles'][0]['width'] = 1
            if case == 'other-profile': profile = 'usa'
            with self.subTest(case=case), tempfile.TemporaryDirectory(prefix='viewer-signed-map-') as directory:
                index = fixture(Path(directory), value, profile=profile, bad_hash=case == 'bad-hash')
                parts = index.parent / 'index.html.parts'; world = payload(parts / 'world.html')
                self.assertNotIn('signed_map_deltas', payload(index))
                self.assertNotIn('signed_map_deltas', payload(parts / 'audio.html'))
                expected = case in ('valid', 'invalid-rectangles')
                self.assertEqual(bool(world['signed_map_deltas']), expected)
                self.assertEqual('ui_rectangles' in world['extra_tilemaps'], case not in ('invalid-rectangles', 'bad-hash'))
                status = world['resources'][0]['status']
                if case == 'invalid-deltas': self.assertIn('invalid signed map delta', status)
                if case == 'invalid-rectangles': self.assertTrue(status.startswith('Available: signed map deltas; Invalid'))


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class SignedMapDeltaBrowserTest(unittest.TestCase):
    def test_preview_playback_loop_focus_and_bookmark(self):
        with tempfile.TemporaryDirectory(prefix='viewer-signed-map-preview-') as directory:
            work = Path(directory); index = fixture(work, catalog())
            browser = Chrome(work, index.parent / 'index.html.parts/world.html')
            try:
                bookmark = browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change',{bubbles:true}));
                  let serial=0;window.pending=new Map();window.setTimeout=(fn,delay)=>{pending.set(++serial,{fn,delay});return serial};
                  window.clearTimeout=id=>pending.delete(id);
                  const fire=()=>{const [id,item]=pending.entries().next().value;pending.delete(id);item.fn();return item.delay};
                  q('[data-resource-id="graphics.ui-raw-transfers"] a[href="#signed-map-deltas"]').click();
                  const select=q('#signed-map-delta-index'),loop=q('#signed-map-delta-loop'),rate=q('#signed-map-delta-rate');
                  select.value='1';change(select);
                  const y=Math.round(20+127*140/255),pixel=[...q('#signed-map-delta-canvas').getContext('2d').getImageData(80,y,1,1).data];
                  if(pixel.join(',')!=='189,61,61,255'||!q('#signed-map-delta-position').textContent.includes('screen Y -1'))throw Error('marker');
                  rate.value='10';select.value='46';change(select);select.focus();q('#signed-map-delta-play').click();
                  if(document.activeElement!==select||fire()!==100||select.value!=='47'||q('#signed-map-delta-progress').value!==48)throw Error('play from selection');
                  if(fire()!==100||!q('#signed-map-delta-stop').disabled||!q('#signed-map-delta-status').textContent.includes('finished'))throw Error('finish');
                  loop.checked=true;rate.value='5';q('#signed-map-delta-play').click();
                  if(fire()!==200||select.value!=='0')throw Error('loop');
                  q('#signed-map-deltas-rows').rows[3].querySelector('button').click();
                  if(select.value!=='3'||!loop.checked||rate.value!=='5'||pending.size!==1)throw Error('row or settings');
                  const nested=q('#signed-map-deltas>details>details');nested.open=false;nested.dispatchEvent(new Event('toggle',{bubbles:true}));
                  if(q('#signed-map-delta-stop').disabled||pending.size!==1)throw Error('nested collapse stopped playback');
                  q('#signed-map-deltas>details').open=false;q('#signed-map-deltas>details').dispatchEvent(new Event('toggle'));
                  if(!q('#signed-map-delta-stop').disabled||pending.size)throw Error('collapse stop');
                  select.value='4';change(select);const link=q('#signed-map-deltas>details>p>a');
                  link.dispatchEvent(new Event('focus'));return link.href;
                })()''')
                browser.command('Page.navigate', {'url': bookmark})
                deadline = time.monotonic() + 25
                while time.monotonic() < deadline:
                    try:
                        if browser.evaluate("document.readyState==='complete' && document.querySelector('#signed-map-delta-index').value==='4'"):
                            break
                    except (RuntimeError, OSError): pass
                    time.sleep(.05)
                else: self.fail('Offset bookmark did not restore')
                self.assertTrue(browser.evaluate("document.querySelector('#signed-map-deltas>details').open && document.querySelector('#signed-map-delta-stop').disabled && document.querySelector('#signed-map-delta-position').textContent.includes('screen Y +127')"))
            finally: browser.close()

    def test_zero_values_and_invalid_preview_rate(self):
        with tempfile.TemporaryDirectory(prefix='viewer-signed-map-zero-') as directory:
            work = Path(directory); value = catalog(); value['signed_map_deltas']['entries'] = [0] * 48
            index = fixture(work, value); browser = Chrome(work, index.parent / 'index.html.parts/world.html')
            try:
                self.assertTrue(browser.evaluate('''(()=>{const q=s=>document.querySelector(s);
                  const pixel=[...q('#signed-map-delta-canvas').getContext('2d').getImageData(80,90,1,1).data];
                  q('#signed-map-delta-rate').value='0';q('#signed-map-delta-play').click();
                  return pixel.join(',')==='189,61,61,255' && q('#signed-map-delta-stop').disabled &&
                    q('#signed-map-delta-status').textContent.includes('1 to 60');})()'''))
            finally: browser.close()

    def test_links_values_and_collapsible_section(self):
        with tempfile.TemporaryDirectory(prefix='viewer-signed-map-browser-') as directory:
            index = fixture(Path(directory), catalog()); browser = Chrome(Path(directory), index.parent / 'index.html.parts/world.html')
            try:
                result = browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s),row=q('[data-resource-id="graphics.ui-raw-transfers"]');
                  const link=row.querySelector('a[href="#signed-map-deltas"]');
                  if(!link||!row.querySelector('a[href="#tilemaps"]'))throw Error('resource links');
                  link.click();
                  const section=q('#signed-map-deltas'),details=section.querySelector('details');
                  const rows=[...q('#signed-map-deltas-rows').rows].map(row=>[...row.cells].map(cell=>cell.textContent));
                  if(section.hidden||!details.open||rows.length!==48||
                     rows[0].join(',')!=='0,0x3954A4,-128,0x80'||
                     rows[1].join(',')!=='1,0x3954A5,-1,0xFF'||
                     rows[3].join(',')!=='3,0x3954A7,+1,0x01'||
                     rows[4].join(',')!=='4,0x3954A8,+127,0x7F')throw Error('table values');
                  if(!q('#signed-map-deltas-source').textContent.includes('0x083954A4')||
                     !q('#contents a[href="#signed-map-deltas"]'))throw Error('source or contents');
                  details.open=false;if(details.open)throw Error('collapse');
                  q('#expand-all').click();if(!details.open)throw Error('expand all');
                  q('#collapse-all').click();if(details.open)throw Error('collapse all');
                  return 'PASS';
                })()''')
                self.assertEqual(result, 'PASS')
            finally:
                browser.close()

    def test_link_with_invalid_rectangles_and_legacy_single_page(self):
        with tempfile.TemporaryDirectory(prefix='viewer-signed-map-partial-') as directory:
            value = catalog(); value['halfword_copies']['tilemap_rectangles']['rectangles'][0]['width'] = 1
            index = fixture(Path(directory), value); browser = Chrome(Path(directory), index.parent / 'index.html.parts/world.html')
            try:
                self.assertTrue(browser.evaluate('''(()=>{const row=document.querySelector('[data-resource-id="graphics.ui-raw-transfers"]');
                  return !!row.querySelector('a[href="#signed-map-deltas"]')&&!row.querySelector('a[href="#tilemaps"]');})()'''))
            finally: browser.close()
        with tempfile.TemporaryDirectory(prefix='viewer-signed-map-single-') as directory:
            index = fixture(Path(directory), catalog(), split=False); browser = Chrome(Path(directory), index)
            try: self.assertEqual(browser.evaluate("document.querySelector('#signed-map-deltas-rows').rows.length"), 48)
            finally: browser.close()
