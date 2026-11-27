"""Check song-to-timed-source candidates and exact row navigation."""
import hashlib
import json
import shutil
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from test_viewer_navbar_split_browser import wait_for
from test_viewer_timed_sounds_browser import fixture as timed_fixture
from view_rom_data import generate
from viewer_browser_harness import Chrome


def fixture(work, *, include_sequences=True, split=True):
    dataset, directory = timed_fixture(work, include_sequences=include_sequences)
    path = dataset / 'audio-directory.json'; old = path.read_bytes()
    rows = []
    for index in range(105):
        words = [0x2005, 0] if index != 102 else [0, 0xa005]
        rows.append({'tick': index, 'channel': index % 4, 'opaque03': 0, 'sound_words': words})
    rows.append({'tick': 105, 'channel': 2, 'opaque03': 0, 'sound_words': [0x1000, 0]})
    directory['timed_sounds']['sources'][-1]['values']['values'] = rows
    directory['songs'].extend([{'id': 5, 'header_id': 5}, {'id': 7, 'header_id': 5}])
    raw = json.dumps(directory).encode(); path.write_bytes(raw)
    manifest = dataset / 'manifest.toml'; text = manifest.read_text()
    text = text.replace(hashlib.sha256(old).hexdigest(), hashlib.sha256(raw).hexdigest()); manifest.write_text(text)
    return generate(dataset, work / 'references-view', split=split)


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class AudioReferenceBrowserTest(unittest.TestCase):
    def test_candidates_pagination_navigation_and_bookmark(self):
        with tempfile.TemporaryDirectory(prefix='viewer-audio-references-') as directory:
            work = Path(directory); index = fixture(work); parts = index.parent / 'index.html.parts'
            browser = Chrome(work, parts / 'audio.html')
            try:
                self.assertTrue(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change',{bubbles:true}));
                  q('#sequence-song').value='1';change(q('#sequence-song'));
                  q('#sequence-mute-cgb').checked=true;change(q('#sequence-mute-cgb'));
                  q('#sequence-timed-references').open=true;return !q('#sequence-timed-references').hidden;})()'''))
                wait_for(browser, "document.querySelector('#sequence-timed-status').textContent.startsWith('106 candidate references')")
                self.assertTrue(browser.evaluate('''(()=>{const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change',{bubbles:true}));
                  if(q('#sequence-timed-rows').rows.length!==25 || q('#sequence-timed-prev').disabled===false ||
                     !q('#sequence-timed-rows').rows[0].textContent.includes('observed Red Spring'))throw Error('first page');
                  q('#sequence-song').value='2';change(q('#sequence-song'));
                  if(!q('#sequence-timed-status').textContent.startsWith('106 candidate references'))throw Error('duplicate ID');
                  q('#sequence-song').value='0';change(q('#sequence-song'));
                  if(!q('#sequence-timed-status').textContent.startsWith('1 candidate references') ||
                     !q('#sequence-timed-rows').textContent.includes('0x1000'))throw Error('zero word versus candidate zero');
                  q('#sequence-song').value='3';change(q('#sequence-song'));
                  if(!q('#sequence-timed-status').textContent.startsWith('0 candidate references'))throw Error('empty');
                  q('#sequence-song').value='1';change(q('#sequence-song'));
                  for(let i=0;i<4;i++)q('#sequence-timed-next').click();
                  const a=q('#sequence-timed-rows a[data-group="5"][data-row="102"][data-slot="1"]');
                  return !!a && a.textContent==='0x750' && a.href.includes('timed_row=102') &&
                    a.closest('tr').textContent.includes('0xA005');})()'''))
                selector = '#sequence-timed-rows a[data-group="5"][data-row="102"][data-slot="1"]'
                bookmark = browser.evaluate('document.querySelector(' + json.dumps(selector) + ').href')
                browser.evaluate('document.querySelector(' + json.dumps(selector) + ').click()')
                self.assertTrue(browser.evaluate('''(()=>{const q=s=>document.querySelector(s);
                  return q('#timed-sound-source').value==='5' && q('#timed-sound-page').textContent.includes('page 2 of 2') &&
                    q('#timed-sound-rows tr[data-timed-row="102"]').classList.contains('timed-reference-row') &&
                    q('#timed-sound-rows tr[data-timed-row="102"] td[data-timed-slot="1"]').classList.contains('timed-reference-word') &&
                    q('#timed-sounds>details').open && q('#sequence-mute-cgb').checked;})()'''))
                browser.command('Page.navigate', {'url': bookmark})
                wait_for(browser, "data.wiki.current==='audio' && document.querySelector('#timed-sound-source').value==='5' && !!document.querySelector('#timed-sound-rows tr[data-timed-row=\"102\"]')")
                self.assertTrue(browser.evaluate("document.querySelector('#timed-sound-rows tr[data-timed-row=\"102\"] td[data-timed-slot=\"1\"]').classList.contains('timed-reference-word') && document.querySelector('#timed-sounds>details').open"))
            finally:
                browser.close()

    def test_no_player_does_not_offer_reverse_references(self):
        with tempfile.TemporaryDirectory(prefix='viewer-audio-references-partial-') as directory:
            work = Path(directory); page = fixture(work, include_sequences=False, split=False); browser = Chrome(work, page)
            try:
                self.assertTrue(browser.evaluate("document.querySelector('#sequence-timed-references').hidden && !document.querySelector('#timed-sounds').hidden"))
            finally:
                browser.close()
