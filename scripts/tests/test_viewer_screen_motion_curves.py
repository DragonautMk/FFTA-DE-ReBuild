"""Verify source-owned motion curves, aliases, positions and browser playback."""
import copy
import hashlib
import json
import shutil
import struct
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import view_rom_data as viewer
import viewer_screen_curve_bank as curves
from test_viewer_navbar_split_browser import wait_for
from test_viewer_scoped_browser import parse_page
from viewer_browser_harness import Chrome


def entry(offset, role, data):
    return {'offset': offset, 'length': len(data), 'role': role, 'sha256': hashlib.sha256(data).hexdigest()}


def synthetic():
    literals = [(0x114200, 0x3a7fb0), (0x114208, 0x3a80c8), (0x1142c0, 0x3a7fb0),
                (0x1152c0, 0x3a823c), (0x1152cc, 0x3a8304), (0x1154ac, 0x3a823c), (0x1193d8, 0x3a8cf8)]
    pieces = [(offset, 'reader_literals', struct.pack('<I', 0x08000000 + target)) for offset, target in literals]
    def records(count):
        return b''.join(bytes([i] * 8) + struct.pack('<II', 0 if i % 3 == 0 else 0x083a89b0,
                          0 if i == 3 else 0x083a8c80 if i % 2 == 0 else 0x083a8a28) + bytes([i] * 4) for i in range(count))
    pieces.append((0x3a7fb0, 'curve_records', records(11)))
    pieces.extend((0x3a80cb + i * 8, 'position_records', bytes([0x80 + i])) for i in range(14))
    pieces.extend([(0x3a823c, 'curve_records', records(10)), (0x3a8304, 'position_records', bytes(range(128))),
                   (0x3a89b0, 'curve_samples', b''.join(struct.pack('<60H', *(bank * 100 + i for i in range(60))) for bank in range(7))),
                   (0x3a8cf8, 'terminal_copy', struct.pack('<H', 0x1234) + bytes(18))])
    rows = [{'selector': i, 'x': i + 1, 'y': i + 2, 'target_x': -i, 'target_y': i * 3} for i in range(14)]
    data = b''.join(bytes([r['selector'], r['x'], r['y']]) + struct.pack('<hh', r['target_x'], r['target_y']) for r in rows)
    specs = [(0x3a80c8, 3)] + [(0x3a80cc + i * 8, 7) for i in range(13)] + [(0x3a8134, 4)]
    prior_sources, cursor = [], 0
    for offset, length in specs:
        prior_sources.append(entry(offset, 'conditional-compact-obj-catalog', data[cursor:cursor + length])); cursor += length
    return pieces, {'selector_tables': {'motion': {'source_offset': 0x3a80c8, 'rows': rows}}}, prior_sources


def fixture(work, *, omit_prior=False, bad_source=False, profile='europe', split=True):
    pieces, prior, prior_sources = synthetic(); dataset = work / 'dataset'; dataset.mkdir()
    content = b''.join(raw for _, _, raw in pieces); owner = json.dumps(prior).encode()
    sources = [entry(offset, role, raw) for offset, role, raw in pieces]
    if bad_source: sources[7]['sha256'] = '0' * 64
    items = [('graphics.screen-curve-bank', 'curves.bin', 'screen-curve-bank-v1-bin', content, sources),
             ('graphics.conditional-compact-obj-catalog', 'owner.json', 'compact-obj-catalog-v15-json', owner, prior_sources)]
    manifest = f'profile="{profile}"\nextractor_revision=1\ndecoder_revision=1\n'
    for identity, name, fmt, raw, bindings in items:
        if omit_prior and name == 'owner.json': continue
        (dataset / name).write_bytes(raw)
        manifest += f'[[resources]]\nid="{identity}"\npath="{name}"\nformat="{fmt}"\nsha256="{hashlib.sha256(raw).hexdigest()}"\n'
        for item in bindings:
            manifest += (f'[[resources.sources]]\noffset={item["offset"]}\nlength={item["length"]}\n'
                         f'role="{item["role"]}"\nsha256="{item["sha256"]}"\n')
    (dataset / 'manifest.toml').write_text(manifest)
    with patch.object(curves, 'CURVES_SHA256', hashlib.sha256(content).hexdigest()), \
         patch.object(curves, 'OWNER_SHA256', hashlib.sha256(owner).hexdigest()):
        return viewer.generate(dataset, work / 'view', split=split)


class ScreenMotionProjectionTest(unittest.TestCase):
    def test_sources_aliases_and_missing_prior(self):
        pieces, owner, sources = synthetic(); prior = curves.prior_positions(owner, sources)
        segments = viewer.project_binary_segments(b''.join(raw for _, _, raw in pieces), [entry(*item) for item in pieces])
        value = curves.project_bank(segments, prior)
        self.assertEqual([len(c['records']) for c in value['controllers']], [11, 10])
        self.assertEqual([len(c['positions']) for c in value['controllers']], [14, 16])
        self.assertEqual([c['values'][60] for c in value['curves']], [100, 200, 300, 400, 500, 600, 0x1234])
        self.assertEqual(value['controllers'][0]['positions'][0]['bytes'], [0, 1, 2, 0x80, 0, 0, 0, 0])
        partial = curves.project_bank(segments); self.assertTrue(partial['prior_missing'])
        self.assertEqual(partial['controllers'][0]['positions'][0]['bytes'], [None, None, None, 0x80, None, None, None, None])
        bad = copy.deepcopy(owner); bad['selector_tables']['motion']['rows'][0]['x'] = 255
        with self.assertRaises(ValueError): curves.prior_positions(bad, sources)
        bad_segments = list(segments); bad_segments[0] = (segments[0][0], segments[0][1], bytes(4))
        with self.assertRaises(ValueError): curves.project_bank(bad_segments, prior)
        bad_segments = list(segments); raw = bytearray(segments[7][2]); raw[8:12] = b'\x12\x34\x56\x78'
        bad_segments[7] = (segments[7][0], segments[7][1], bytes(raw))
        with self.assertRaises(ValueError): curves.project_bank(bad_segments, prior)

    def test_scope_and_partial_data(self):
        for options in ({}, {'omit_prior': True}, {'bad_source': True}, {'profile': 'usa'}):
            with self.subTest(options=options), tempfile.TemporaryDirectory(prefix='viewer-motion-') as directory:
                page = fixture(Path(directory), **options); parts = page.parent / 'index.html.parts'
                home, interface, scoped = map(parse_page, (page, parts/'interface.html', parts/'interface-motion-curves.html'))
                self.assertNotIn('screen_curve_bank', home); self.assertNotIn('screen_curve_bank', interface)
                row = next(item for item in home['resources'] if item['id'] == 'graphics.screen-curve-bank')
                self.assertEqual(row['viewer_scope'], 'interface-motion-curves')
                if options.get('bad_source') or options.get('profile'):
                    self.assertIsNone(scoped['screen_curve_bank'])
                else:
                    self.assertEqual(scoped['screen_curve_bank']['prior_missing'], bool(options.get('omit_prior')))
                    self.assertEqual(len(scoped['screen_curve_bank']['curves']), 7)


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class ScreenMotionBrowserTest(unittest.TestCase):
    def test_playback_aliases_scoped_navigation_and_bookmark(self):
        with tempfile.TemporaryDirectory(prefix='viewer-motion-browser-') as directory:
            work = Path(directory); page = fixture(work); browser = Chrome(work, page)
            try:
                target = browser.evaluate("document.querySelector('[data-resource-id=\"graphics.screen-curve-bank\"] a').href")
                browser.command('Page.navigate', {'url': target})
                wait_for(browser, "data.wiki.current==='interface-motion-curves' && !document.querySelector('#screen-motion-curves').hidden")
                self.assertTrue(browser.evaluate('''(()=>{const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change',{bubbles:true}));
                  if(q('#screen-motion-curves>details>summary time').dateTime!=='2026-11-27T23:05:18Z'||q('#screen-motion-record').options.length!==11||
                     q('#screen-motion-value-rows').rows.length!==61||q('#screen-motion-position-rows').rows.length!==14||
                     !q('#screen-motion-current').textContent.includes('X 256'))throw Error('initial');
                  q('#screen-motion-step').value='60';q('#screen-motion-step').dispatchEvent(new Event('input'));
                  if(!q('#screen-motion-current').textContent.includes('Y 4660')||!q('#screen-motion-current').textContent.includes('alias'))throw Error('alias');
                  q('#screen-motion-record').value='3';change(q('#screen-motion-record'));
                  if(q('#screen-motion-current').textContent.includes('alias')||!q('#screen-motion-current').textContent.includes('Y 256'))throw Error('constant');
                  q('#screen-motion-record').value='0';change(q('#screen-motion-record'));
                  window.motionTimers=new Map();let serial=0;window.setTimeout=(fn,delay)=>{const id=++serial;motionTimers.set(id,{fn,delay});return id};
                  window.clearTimeout=id=>motionTimers.delete(id);window.fireMotion=()=>{const [id,item]=motionTimers.entries().next().value;motionTimers.delete(id);item.fn();return item.delay};
                  q('#screen-motion-loop').checked=true;q('#screen-motion-rate').value='20';q('#screen-motion-play').click();
                  if(fireMotion()!==50||q('#screen-motion-step').value!=='0')throw Error('loop');
                  q('#screen-motion-controller').value='1';change(q('#screen-motion-controller'));
                  if(!q('#screen-motion-loop').checked||q('#screen-motion-rate').value!=='20'||q('#screen-motion-stop').disabled||
                     q('#screen-motion-position-rows').rows.length!==16||q('#screen-motion-record').options.length!==10)throw Error('retained settings');
                  q('#screen-motion-stop').click();q('#screen-motion-record').value='9';change(q('#screen-motion-record'));return true;})()'''))
                saved = browser.evaluate("document.querySelector('#screen-motion-curves>details a').href")
                browser.command('Page.navigate', {'url': saved})
                wait_for(browser, "data.wiki.current==='interface-motion-curves' && document.querySelector('#screen-motion-controller').value==='1' && document.querySelector('#screen-motion-record').value==='9'")
                browser.evaluate("document.querySelector('#screen-motion-record').focus()")
                browser.click('#screen-motion-play')
                self.assertEqual(browser.evaluate('document.activeElement.id'), 'screen-motion-record')
            finally:
                browser.close()


if __name__ == '__main__':
    unittest.main()
