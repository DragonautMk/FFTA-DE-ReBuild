"""Check text spacing, normal-font rendering and bitmap fallbacks in Chrome."""
import json
import shutil
import tempfile
import unittest
from pathlib import Path

from viewer_browser_harness import Chrome


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class ViewerTextReadabilityTest(unittest.TestCase):
    def test_hidden_commands_and_normal_font(self):
        with tempfile.TemporaryDirectory(prefix='viewer-text-') as directory:
            work = Path(directory)
            repo = Path(__file__).resolve().parents[2]
            glyphs = [{'index': i, 'width': 8, 'tile_words': [1] + [0] * 31} for i in (176, 203, 205, 1000)]
            program_units = [[0x63], 176, 203, [0x40, 0x73], [0x60], 205, 1000, [0x70]]
            short_units = [{'kind': 'control_byte', 'value': 0x63}, {'kind': 'glyph', 'index': 176},
                           {'kind': 'glyph', 'index': 203}, {'kind': 'control_byte', 'value': 0x40},
                           {'kind': 'control_byte', 'value': 0x73}, {'kind': 'glyph', 'index': 205},
                           {'kind': 'glyph', 'index': 1000}, {'kind': 'control_byte', 'value': 0x70}]
            program = {'source_offset': 10, 'header': 0, 'units': program_units}
            data = {'profile': 'europe', 'extractor': 1, 'decoder': 1, 'resources': [],
                    'short_texts': {'path': 'short', 'glyphs': glyphs,
                                    'languages': [{'language_id': 0, 'entries': [0]}],
                                    'strings': [{'encoding': 'compact', 'units': short_units}]},
                    'pointer_texts': {'path': 'pointer', 'glyphs': glyphs,
                                      'catalogs': [{'reader': 1, 'languages': [{'source_offset': 5, 'entries': [0]}]}],
                                      'strings': [{'encoding': 'compact', 'source_offset': 6, 'units': program_units}]},
                    'relative_texts': {'path': 'relative', 'glyphs': glyphs, 'catalogs': [
                        {'reader': 1, 'languages': [{'source_offset': 5, 'programs': [program]}]}]},
                    'script_texts': {'path': 'script', 'glyphs': glyphs, 'languages': [
                        {'id': 0, 'banks': [{'source_offset': 5, 'programs': [program]}]}]}}
            template = (repo / 'view_rom_data.html').read_text()
            page = work / 'text.html'
            page.write_text(template.replace('/* DATA */', json.dumps(data)))
            browser = Chrome(work, page)
            try:
                result = browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s), change=e=>e.dispatchEvent(new Event('change'));
                  for(const id of ['texts','pointer-texts','relative-texts','script-texts'])q('#'+id+'>details').open=true;
                  q('#relative-hide-commands').checked=true;change(q('#relative-hide-commands'));
                  q('#script-normal-font').checked=true;change(q('#script-normal-font'));
                  for(const id of ['text','pointer','relative','script']){
                    const p=q('#'+id+'-preview'), gap=p.querySelector('.text-gap');
                    if(!q('#'+id+'-normal-font').checked || !q('#'+id+'-hide-commands').checked)throw Error('settings '+id);
                    if(p.textContent!=='Ab d' || !gap || gap.textContent!==' ' || gap.getBoundingClientRect().width<=0 ||
                       p.querySelectorAll('.text-gap').length!==1 || p.querySelectorAll('canvas').length!==1)throw Error('render '+id);
                    if(!getComputedStyle(p.querySelector('.text-normal')).fontFamily.includes('Verdana'))throw Error('font '+id);
                  }
                  data.profile='synthetic';q('#script-normal-font').checked=false;change(q('#script-normal-font'));
                  q('#script-normal-font').checked=true;change(q('#script-normal-font'));
                  if(q('#script-preview').querySelectorAll('canvas').length!==4 || q('#script-preview').querySelector('.text-normal'))throw Error('other profile fallback');
                  data.profile='europe';q('#script-hide-commands').checked=false;change(q('#script-hide-commands'));
                  if(!q('#script-preview').textContent.includes('[command 0x40 0x73]'))throw Error('show commands');
                  return 'PASS';
                })()''')
                self.assertEqual(result, 'PASS')
            finally:
                browser.close()
