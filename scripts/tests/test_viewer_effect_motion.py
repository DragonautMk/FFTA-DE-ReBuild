"""Validate source-bound screen-effect schedules and illustrative playback."""
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
import viewer_effect_motion as effect
from test_viewer_navbar_split_browser import wait_for
from test_viewer_scoped_browser import parse_page
from viewer_browser_harness import Chrome


def binding(offset, role, raw):
    return {'offset': offset, 'role': role, 'length': len(raw), 'sha256': hashlib.sha256(raw).hexdigest()}


def synthetic():
    pieces = [(offset, 'reader_literals', struct.pack('<' + 'I' * len(values), *(0x08000000 + value for value in values)))
              for offset, values in effect.LITERALS]
    motions = lambda count: b''.join(struct.pack('<BBhhH', i + 2, 200 + i, -i, i + 3, 0x1200 + i) for i in range(count))
    first = bytes(i % 3 for i in range(22)) + bytes(i % 4 for i in range(22)) + bytes(range(22)) + bytes(i + 1 for i in range(22))
    second = bytes(i % 8 for i in range(64)) + bytes(range(64)) + bytes(i + 1 for i in range(64))
    timed = b''.join(struct.pack('<HBB6hHH', i * 10, i, 240 + i, i, i + 10, -i, 3 * i,
                                 2 * i, -2 * i, i + 6, 0xAA00 + i) for i in range(7))
    starts = b''.join(struct.pack('<hh', i, -i) for i in range(5))
    ends = b''.join(struct.pack('<hh', i + 20, 10 - i) for i in range(5))
    raw = [struct.pack('<22H', *(i * 20 for i in range(22))), first, motions(4), second, motions(8),
           struct.pack('<20h', *(i - 10 for i in range(20))), timed, struct.pack('<84h', *(42 - i for i in range(84))),
           bytes(range(160)), starts + ends, bytes(range(5)) + bytes(range(10, 15)) + bytes(range(20, 25)),
           struct.pack('<5h', -2, -1, 0, 1, 2)]
    pieces += [(offset, role, content) for (offset, role, length), content in zip(effect.TABLES, raw) if len(content) == length]
    assert len(pieces) == len(effect.LITERALS) + len(effect.TABLES)
    return pieces


def fixture(work, *, bad_source=False, trusted=True, profile='europe', split=True, omit=False):
    pieces = synthetic(); raw = b''.join(piece[2] for piece in pieces); sources = [binding(*piece) for piece in pieces]
    if bad_source: sources[-1]['sha256'] = '0' * 64
    dataset = work/'dataset'; dataset.mkdir()
    manifest = f'profile="{profile}"\nextractor_revision=376\ndecoder_revision=375\n'
    if omit: manifest += 'resources=[]\n'
    if not omit:
        (dataset/'effect.bin').write_bytes(raw)
        manifest += ('[[resources]]\nid="graphics.screen-effect-motion"\npath="effect.bin"\n'
                     'format="screen-effect-motion-v1-bin"\nsha256="' + hashlib.sha256(raw).hexdigest() + '"\n')
        for item in sources:
            manifest += (f'[[resources.sources]]\noffset={item["offset"]}\nlength={item["length"]}\n'
                         f'role="{item["role"]}"\nsha256="{item["sha256"]}"\n')
    (dataset/'manifest.toml').write_text(manifest)
    with patch.object(effect, 'EFFECT_SHA256', hashlib.sha256(raw).hexdigest() if trusted else '0' * 64):
        return viewer.generate(dataset, work/'view', split=split)


class ScreenEffectProjectionTest(unittest.TestCase):
    def test_projection_and_source_guards(self):
        pieces = synthetic(); raw = b''.join(x[2] for x in pieces)
        segments = viewer.project_binary_segments(raw, [binding(*piece) for piece in pieces])
        value = effect.project(segments)
        self.assertEqual([len(x['rows']) for x in value['sprite_banks']], [22, 64])
        self.assertEqual([len(x['motions']) for x in value['sprite_banks']], [4, 8])
        self.assertEqual(value['sprite_banks'][0]['rows'][21]['tick'], 420)
        self.assertEqual(value['sprite_banks'][1]['rows'][63]['motion'], 7)
        self.assertEqual(value['sprite_banks'][0]['motions'][3]['delta'], [-3, 6])
        self.assertEqual(value['timed'][2]['pairs'], [[2, 12], [-2, 6], [4, -4]])
        self.assertEqual((value['timed'][2]['stored_duration'], value['timed'][2]['duration']), (8, 7))
        self.assertEqual((value['background'][4]['trigger_tick'], value['background'][4]['delay']), (236, 2))
        self.assertEqual([len(x['values']) for x in value['curves']], [20, 84, 160])
        for change in (lambda s: s.pop(), lambda s: s[0].__setitem__(1, 'wrong'),
                       lambda s: s[0].__setitem__(2, bytes(len(s[0][2]))),
                       lambda s: s[15].__setitem__(2, bytes([255]) + s[15][2][1:])):
            bad = [list(x) for x in copy.deepcopy(segments)]; change(bad)
            if bad and len(bad) > 15 and bad[15][2][0] == 255:
                bad[15][2] = bad[15][2][:22] + bytes([255]) + bad[15][2][23:]
            with self.assertRaises(ValueError): effect.project([tuple(x) for x in bad])
        invalid_sources = [binding(*piece) for piece in pieces]; invalid_sources[-1]['sha256'] = '0' * 64
        with self.assertRaises(ValueError): viewer.project_binary_segments(raw, invalid_sources)

    def test_scope_and_untrusted_sources(self):
        for options in ({}, {'bad_source': True}, {'trusted': False}, {'profile': 'usa'}, {'omit': True}):
            with self.subTest(options=options), tempfile.TemporaryDirectory(prefix='viewer-effect-') as directory:
                page = fixture(Path(directory), **options); parts = page.parent/'index.html.parts'
                home = parse_page(page); self.assertNotIn('screen_effect_motion', home)
                if options.get('omit'):
                    self.assertFalse((parts/'interface-effect-motion.html').exists()); continue
                scoped = parse_page(parts/'interface-effect-motion.html')
                self.assertEqual(next(x for x in home['resources'] if x['id']=='graphics.screen-effect-motion')['viewer_scope'], 'interface-effect-motion')
                if options: self.assertIsNone(scoped['screen_effect_motion'])
                else: self.assertEqual(len(scoped['screen_effect_motion']['curves'][2]['values']), 160)


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class ScreenEffectBrowserTest(unittest.TestCase):
    def test_scoped_playback_selection_curves_and_badges(self):
        with tempfile.TemporaryDirectory(prefix='viewer-effect-browser-') as directory:
            work = Path(directory); index = fixture(work); browser = Chrome(work, index)
            try:
                target = browser.evaluate("document.querySelector('[data-resource-id=\"graphics.screen-effect-motion\"] a').href")
                browser.command('Page.navigate', {'url':target})
                wait_for(browser, "data.wiki.current==='interface-effect-motion' && !document.querySelector('#screen-effect-motion').hidden")
                self.assertTrue(browser.evaluate('''(()=>{const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change',{bubbles:true}));
                  if(q('#screen-effect-motion>details>summary time').dateTime!=='2026-11-28T00:09:51Z'||
                     !['sprites','timed','backgrounds','values'].every(id=>q('#screen-effect-'+id+'>summary time').dateTime==='2026-11-28T00:09:51Z')||
                     q('#screen-effect-sprite-rows').rows.length!==86||q('#screen-effect-motion-rows').rows.length!==12||
                     q('#screen-effect-timed-rows').rows.length!==7||q('#screen-effect-background-rows').rows.length!==5||
                     q('#screen-effect-value-rows').rows.length!==264||q('#screen-effect-record').options.length!==22||
                     q('#screen-effect-step').max!=='2'||!q('#screen-effect-current').textContent.includes('spawn tick 0'))throw Error('initial');
                  window.effectTimers=new Map();let serial=0;window.setTimeout=(fn,delay)=>{const id=++serial;effectTimers.set(id,{fn,delay});return id};
                  window.clearTimeout=id=>effectTimers.delete(id);window.fireEffect=()=>{const [id,item]=effectTimers.entries().next().value;effectTimers.delete(id);item.fn();return item.delay};
                  q('#screen-effect-loop').checked=true;q('#screen-effect-rate').value='20';q('#screen-effect-step').value='2';
                  q('#screen-effect-play').click();if(fireEffect()!==50||q('#screen-effect-step').value!=='0')throw Error('loop');
                  q('#screen-effect-kind').value='second';change(q('#screen-effect-kind'));
                  q('#screen-effect-record').value='63';change(q('#screen-effect-record'));
                  if(!q('#screen-effect-loop').checked||q('#screen-effect-rate').value!=='20'||q('#screen-effect-stop').disabled||
                     q('#screen-effect-step').max!=='9'||!q('#screen-effect-current').textContent.includes('physical second-sprite'))throw Error('settings');
                  q('#screen-effect-stop').click();q('#screen-effect-kind').value='timed';change(q('#screen-effect-kind'));
                  q('#screen-effect-record').value='2';change(q('#screen-effect-record'));
                  if(q('#screen-effect-step').max!=='7'||!q('#screen-effect-current').textContent.includes('third value 4.00'))throw Error('timed');
                  q('#screen-effect-background-rows').rows[4].querySelector('button').click();
                  if(q('#screen-effect-kind').value!=='background'||q('#screen-effect-step').max!=='260'||
                     !q('#screen-effect-current').textContent.includes('trigger tick 236'))throw Error('background');
                  const last=q('#screen-effect-value-rows').rows[263];last.querySelector('button').click();
                  return q('#screen-effect-kind').value==='curve'&&q('#screen-effect-record').value==='2'&&q('#screen-effect-step').value==='159'&&
                    q('#screen-effect-current').textContent.includes('value 159')&&q('#screen-effect-current').textContent.includes('65377')&&
                    q('#screen-effect-current').textContent.includes('399');})()'''))
                browser.evaluate("(()=>{const e=document.querySelector('#screen-effect-record');for(let p=e.parentElement;p;p=p.parentElement)if(p.tagName==='DETAILS')p.open=true;e.focus()})()")
                browser.click('#screen-effect-play')
                self.assertEqual(browser.evaluate('document.activeElement.id'), 'screen-effect-record')
            finally:
                browser.close()


if __name__ == '__main__':
    unittest.main()
