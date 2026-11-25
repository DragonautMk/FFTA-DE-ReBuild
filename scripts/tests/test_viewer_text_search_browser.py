"""Exercise decoded text search, navigation and cooperative cancellation in Chrome."""
import json
import shutil
import tempfile
import unittest
from pathlib import Path

from viewer_browser_harness import Chrome


def fixture(work, profile='europe', all_archives=True):
    repo = Path(__file__).resolve().parents[2]
    glyphs = [{'index': i, 'width': 8, 'tile_words': [1] + [0] * 31} for i in (176, 203, 205, 1000)]
    units = [[0x63], 176, 203, [0x40], [0x60], 205, 1000, [0x70]]
    short_units = [{'kind': 'control_byte', 'value': 0x63}, {'kind': 'glyph', 'index': 176},
                   {'kind': 'glyph', 'index': 203}, {'kind': 'control_byte', 'value': 0x40},
                   {'kind': 'control_byte', 'value': 0x60}, {'kind': 'glyph', 'index': 205},
                   {'kind': 'glyph', 'index': 1000}, {'kind': 'control_byte', 'value': 0x70}]
    empty = {'source_offset': 11, 'header': 0, 'units': [176]}
    program = {'source_offset': 12, 'header': 0, 'units': units}
    data = {'profile': profile, 'extractor': 1, 'decoder': 1, 'resources': [],
            'short_texts': {'path': 'short', 'glyphs': glyphs,
                            'languages': [{'language_id': 0, 'entries': [0, 0]},
                                          {'language_id': 1, 'entries': [0, 1]}],
                            'strings': [{'encoding': 'compact', 'units': [{'kind': 'glyph', 'index': 176}]},
                                        {'encoding': 'compact', 'units': short_units}]}}
    if all_archives:
        pointer_languages = lambda hit: [{'source_offset': 5, 'entries': [0, 0]},
                                         {'source_offset': 6, 'entries': [0, 1 if hit else 0]}]
        relative_languages = lambda hit: [{'source_offset': 5, 'programs': [empty, empty]},
                                          {'source_offset': 6, 'programs': [empty, program if hit else empty]}]
        data['pointer_texts'] = {'path': 'pointer', 'glyphs': glyphs,
                                 'catalogs': [{'reader': 1, 'languages': pointer_languages(False)},
                                              {'reader': 2, 'languages': pointer_languages(True)}],
                                 'strings': [{'encoding': 'compact', 'source_offset': 6, 'units': [176]},
                                             {'encoding': 'compact', 'source_offset': 7, 'units': units}]}
        data['relative_texts'] = {'path': 'relative', 'glyphs': glyphs,
                                  'catalogs': [{'reader': 1, 'languages': relative_languages(False)},
                                               {'reader': 2, 'languages': relative_languages(True)}]}
        data['script_texts'] = {'path': 'script', 'glyphs': glyphs,
                                'languages': [{'id': 0, 'banks': [{'source_offset': 5, 'programs': [empty]}]},
                                              {'id': 1, 'banks': [{'source_offset': 6, 'programs': [empty]},
                                                                  {'source_offset': 7, 'programs': [empty, program]}]}]}
    page = work / 'text-search.html'
    page.write_text((repo / 'view_rom_data.html').read_text().replace('/* DATA */', json.dumps(data)))
    return page


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class ViewerTextSearchTest(unittest.TestCase):
    def test_search_navigation_paging_and_cancellation(self):
        with tempfile.TemporaryDirectory(prefix='viewer-text-search-') as directory:
            work = Path(directory)
            browser = Chrome(work, fixture(work))
            try:
                result = browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s), change=e=>e.dispatchEvent(new Event('change'));
                  window.pending=new Map();let serial=0;
                  window.setTimeout=(fn,delay)=>{const id=++serial;pending.set(id,fn);return id};
                  window.clearTimeout=id=>pending.delete(id);
                  const fire=()=>{const first=pending.entries().next();if(first.done)throw Error('no scheduled search');
                    const [id,fn]=first.value;pending.delete(id);fn()};
                  const drain=()=>{for(let i=0;pending.size&&i<10000;i++)fire();if(pending.size)throw Error('search did not finish')};
                  const search=(term,scope='',language='')=>{
                    q('#text-search-query').value=term;q('#text-search-scope').value=scope;
                    q('#text-search-language').value=language;q('#text-search-form').requestSubmit();};
                  q('#text-pin').checked=true;change(q('#text-pin'));
                  q('#text-hide-commands').checked=true;change(q('#text-hide-commands'));
                  q('#text-normal-font').checked=true;change(q('#text-normal-font'));
                  search('b D','','1');drain();
                  let links=[...q('#text-search-rows').querySelectorAll('a')];
                  if(links.length!==4 || !q('#text-search-status').textContent.includes('4 matches') ||
                     [...q('#text-search-rows').rows].some(r=>r.cells[1].textContent!=='Ab d�'))throw Error('recognized search');
                  links[0].click();
                  if(q('#text-language').value!=='1'||q('#text-entry').value!=='1'||!q('#texts>details').open)throw Error('short navigation');
                  links[1].click();
                  if(q('#pointer-reader').value!=='1'||q('#pointer-language').value!=='1'||q('#pointer-entry').value!=='1'||!q('#pointer-texts>details').open)
                    throw Error('pointer navigation '+[q('#pointer-reader').value,q('#pointer-language').value,q('#pointer-entry').value,q('#pointer-texts>details').open].join('/'));
                  links[2].click();
                  if(q('#relative-reader').value!=='1'||q('#relative-language').value!=='1'||q('#relative-entry').value!=='1'||!q('#relative-texts>details').open)throw Error('relative navigation');
                  links[3].click();
                  if(q('#script-language').value!=='1'||q('#script-bank').value!=='1'||q('#script-entry').value!=='1'||!q('#script-texts>details').open)throw Error('script navigation');
                  for(const prefix of ['text','pointer','relative','script']){
                    if(!q('#'+prefix+'-pin').checked || !q('#'+prefix+'-hide-commands').checked ||
                       !q('#'+prefix+'-normal-font').checked || q('#'+prefix+'-language').selectedOptions[0].textContent!=='1')throw Error('preserved settings '+prefix);
                  }
                  search('bd','','1');drain();if(!q('#text-search-status').textContent.includes('0 matches')||!q('#text-search-results').hidden)throw Error('gap');
                  search('d�','script','1');drain();if(q('#text-search-rows').rows.length!==1)throw Error('unknown glyph marker');
                  data.short_texts.languages[0].entries.push(...Array(1200).fill(0));
                  search('a','short','0');drain();
                  if(!q('#text-search-status').textContent.includes('first 1000')||q('#text-search-rows').rows.length!==20||
                     !q('#text-search-page').textContent.includes('Page 1 of 50'))throw Error('limit');
                  q('#text-search-next').click();if(!q('#text-search-page').textContent.includes('Page 2 of 50'))throw Error('next page');
                  q('#text-search-prev').click();if(!q('#text-search-page').textContent.includes('Page 1 of 50'))throw Error('previous page');
                  data.short_texts.strings.push({encoding:'compact',units:Array(20001).fill({kind:'glyph',index:176})});
                  data.short_texts.languages[0].entries=[2];
                  search('z','short','0');fire();
                  if(q('#text-search-cancel').disabled || !pending.size || !q('#text-search-status').textContent.includes('0 entries'))throw Error('bounded long record');
                  q('#text-search-cancel').click();
                  if(!q('#text-search-cancel').disabled || pending.size || !q('#text-search-status').textContent.includes('cancelled'))throw Error('cancellation');
                  search('a','short','1');drain();if(q('#text-search-rows').rows.length!==2)throw Error('restart');
                  return 'PASS';
                })()''')
                self.assertEqual(result, 'PASS')
            finally:
                browser.close()

    def test_unmapped_profile_is_explicit_and_disabled(self):
        with tempfile.TemporaryDirectory(prefix='viewer-text-profile-') as directory:
            work = Path(directory)
            browser = Chrome(work, fixture(work, profile='unknown', all_archives=False))
            try:
                self.assertEqual(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s);
                  return !q('#text-search').hidden && q('#text-search-query').disabled && q('#text-search-start').disabled &&
                    q('#text-search-status').textContent.includes('only mapped for the European profile') ? 'PASS' : 'FAIL';
                })()'''), 'PASS')
            finally:
                browser.close()
