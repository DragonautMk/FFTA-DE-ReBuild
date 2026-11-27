"""Check lightweight audio topics and cross-page song navigation."""
import json
import shutil
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from test_viewer_instrument_song_links_browser import page_with_directory
from test_viewer_navbar_split_browser import wait_for
from test_viewer_scoped_browser import REPO, parse_page, prepared
from view_rom_data import write_split_viewer
from viewer_browser_harness import Chrome


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class ViewerAudioScopesTest(unittest.TestCase):
    def test_scoped_audio_navigation_and_shared_mute(self):
        with tempfile.TemporaryDirectory(prefix='viewer-audio-scopes-') as directory:
            work = Path(directory); data, index = prepared(work)
            instruments_page = page_with_directory(work)
            instrument_data = parse_page(instruments_page)
            for key in ('audio', 'sequence_directory', 'sequences', 'pcm_pitch'):
                data[key] = instrument_data[key]
            data['audio_noise'] = {'path': 'noise.json', 'source_offset': 100, 'entries': [0] * 60}
            write_split_viewer(data, index.parent, index.name, (REPO / 'view_rom_data.html').read_text())
            parts = index.parent / (index.name + '.parts')
            samples, song, noise = map(parse_page, (parts / 'audio-samples.html', parts / 'audio.html', parts / 'audio-noise.html'))
            self.assertEqual(samples['audio'], song['audio'])
            self.assertNotIn('sequences', samples)
            self.assertNotIn('audio', noise)
            self.assertNotIn('sequences', noise)
            self.assertEqual(noise['audio_noise'], data['audio_noise'])
            browser = Chrome(work, parts / 'audio-samples.html')
            try:
                self.assertTrue(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change',{bubbles:true}));
                  if(q('#audio').hidden || q('#instruments').hidden || !q('#sequence-audio').hidden ||
                     typeof data.sequences!=='undefined' || q('#instrument-songs-rows').rows.length!==25)throw Error('sample scope');
                  q('#instrument-mute-cgb').checked=true;change(q('#instrument-mute-cgb'));
                  q('#instrument-songs-next').click();
                  const a=q('#instrument-songs-rows').rows[1].cells[0].querySelector('a');
                  if(!a || !a.href.includes('/audio.html?') || !a.href.includes('sequence-song%3A26'))throw Error('cross-page link');
                  return true;
                })()'''))
                browser.click('#instrument-songs-rows tr:nth-child(2) a')
                wait_for(browser, "typeof data !== 'undefined' && data.wiki.current==='audio' && document.querySelector('#sequence-song').value==='26'")
                self.assertTrue(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s);
                  return q('#sequence-audio>details').open && q('#sequence-mute-cgb').checked &&
                    !!data.sequences && !!data.audio && typeof data.audio_noise==='undefined';
                })()'''))
                browser.command('Page.navigate', {'url': (parts / 'audio-noise.html').as_uri() + '#cgb-noise'})
                wait_for(browser, "typeof data !== 'undefined' && data.wiki.current==='audio-noise' && document.querySelector('#cgb-noise>details').open")
                self.assertTrue(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s);
                  return q('#noise-mute-cgb').checked && !q('#cgb-noise').hidden && q('#noise-code').options.length===60 &&
                    typeof data.audio==='undefined' && typeof data.sequences==='undefined';
                })()'''))
            finally:
                browser.close()
