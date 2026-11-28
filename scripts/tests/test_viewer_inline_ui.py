"""Validate the source-bound localized UI strings and language controls."""
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
import viewer_inline_ui as inline
from test_viewer_navbar_split_browser import wait_for
from test_viewer_scoped_browser import parse_page
from viewer_browser_harness import Chrome


def binding(offset, role, raw):
    return {'offset': offset, 'role': role, 'length': len(raw), 'sha256': hashlib.sha256(raw).hexdigest()}


def synthetic():
    boundaries = [[0x395187,0x3951b8,0x3951e1,0x395214,0x395241,0x395262],
                  [0x395280,0x395289,0x395294,0x39529d,0x3952aa,0x3952b3],
                  [0x3952c8,0x3952db,0x3952ea,0x395302,0x395319,0x39532e],
                  [0x395344,0x39534d,0x395356,0x39535f,0x39536c,0x395375],
                  [0x39538c,0x395395,0x39539e,0x3953a7,0x3953b2,0x3953c5],
                  [0x3953dc,0x3953e7,0x3953f4,0x395401,0x39540c,0x39541d],
                  [0x395434,0x39543d,0x395446,0x39544f,0x39545a,0x39546d]]
    pieces = [(offset,'reader_literals',struct.pack('<'+'I'*len(targets),*(0x08000000+x for x in targets)))
              for offset,targets in inline.LITERALS]
    blocks={}
    for group,ends in enumerate(boundaries):
        blocks[inline.TABLES[group]]=struct.pack('<5I',*(0x08000000+x for x in ends[:5]))
        chunks=[]
        for lang,(start,end) in enumerate(zip(ends,ends[1:])):
            length=end-start
            if group==0 and lang==0:
                chunks.append(bytes([0x80,0xb0,0x40,0x80,0xb1]+[0x40]*(length-6)+[0]))
            else:
                glyph=178 if (group==1 and lang==3) else 177+((group+lang)%2)
                chunks.append(bytes([1]+[glyph]*(length-2)+[0]))
        blocks[ends[0]]=b''.join(chunks)
    blocks[0x395184]=bytes([1,179,0]);blocks[0x395262]=bytes([1,179,179,179,179,179,0])
    blocks[0x395484]=bytes([1,179,0])*4
    blocks[0x395490]=b''.join(bytes([0x80,0xb0+i]) for i in range(6))
    blocks[0x3954a0]=struct.pack('<I',0x08395490)
    pieces += [(offset,role,blocks[offset]) for offset,role,length in inline.DATA_SOURCES if len(blocks[offset])==length]
    assert len(pieces)==len(inline.LITERALS)+len(inline.DATA_SOURCES)
    return pieces


def fixture(work, *, bad_source=False, trusted=True, profile='europe', omit_font=False, omit=False):
    pieces=synthetic();raw=b''.join(item[2] for item in pieces);sources=[binding(*item) for item in pieces]
    if bad_source:sources[-1]['sha256']='0'*64
    glyphs=[{'index':i,'width':8,'tile_words':[0x11111111]*32} for i in range(176,182)]
    short={'languages':[{'language_id':i,'entries':[0]} for i in range(5)],
           'strings':[{'encoding':'compact','units':[{'kind':'glyph','index':176}]}],'glyphs':glyphs}
    items=[('text.localized-inline-ui','inline.bin','localized-inline-ui-v1-bin',raw,sources),
           ('font.complete-bank','font.json','font-bank-v1-json',json.dumps({'glyphs':glyphs}).encode(),[]),
           ('text.short-strings','short.json','short-texts-v1-json',json.dumps(short).encode(),[])]
    dataset=work/'dataset';dataset.mkdir();manifest=f'profile="{profile}"\nextractor_revision=377\ndecoder_revision=376\n'
    for identity,name,fmt,content,bindings in items:
        if omit and identity=='text.localized-inline-ui' or omit_font and identity=='font.complete-bank':continue
        (dataset/name).write_bytes(content)
        manifest+=f'[[resources]]\nid="{identity}"\npath="{name}"\nformat="{fmt}"\nsha256="{hashlib.sha256(content).hexdigest()}"\n'
        for item in bindings:
            manifest+=(f'[[resources.sources]]\noffset={item["offset"]}\nlength={item["length"]}\n'
                       f'role="{item["role"]}"\nsha256="{item["sha256"]}"\n')
    (dataset/'manifest.toml').write_text(manifest)
    with patch.object(inline,'INLINE_SHA256',hashlib.sha256(raw).hexdigest() if trusted else '0'*64):
        return viewer.generate(dataset,work/'view',split=True)


class InlineProjectionTest(unittest.TestCase):
    def test_formats_sources_directories_and_fallback(self):
        pieces=synthetic();raw=b''.join(x[2] for x in pieces);segments=viewer.project_binary_segments(raw,[binding(*piece) for piece in pieces])
        result=inline.project(segments)
        self.assertEqual([len(d['entries']) for d in result['directories']],[5]*7)
        self.assertEqual(result['directories'][0]['entries'][0]['units'][:3],
                         [{'kind':'glyph','index':176},{'kind':'control_byte','value':64},{'kind':'glyph','index':177}])
        self.assertEqual(result['directories'][1]['entries'][3]['source_offset'],0x39529d)
        self.assertEqual(result['fallback'],{'directory':1,'language_id':3,'shared_index':1,'reader':0x05a750})
        self.assertEqual([x['minimum_level'] for x in result['thresholds']],[0,3,6,9,12,15])
        self.assertEqual(result['thresholds'][-1]['maximum_level'],None)
        self.assertEqual(len(result['shared']),6)
        for mutate in (lambda s:s.pop(),lambda s:s[0].__setitem__(2,bytes(len(s[0][2]))),
                       lambda s:s[25].__setitem__(2,bytes(len(s[25][2]))),
                       lambda s:s[-2].__setitem__(2,bytes(len(s[-2][2])))):
            bad=[list(x) for x in copy.deepcopy(segments)];mutate(bad)
            with self.assertRaises(ValueError):inline.project([tuple(x) for x in bad])
        for raw in (b'',b'\x01\x02',b'\x80',b'\x80\x00',b'\x01\x02\x00\x00'):
            with self.assertRaises(ValueError):inline.decode(raw,0)
        self.assertEqual(inline.decode(b'\x80\x00\x00',10)['units'],[{'kind':'glyph','index':0}])
        bad=[binding(*piece) for piece in pieces];bad[-1]['sha256']='0'*64
        with self.assertRaises(ValueError):viewer.project_binary_segments(b''.join(piece[2] for piece in pieces),bad)

    def test_scoped_missing_or_untrusted_sources_and_font(self):
        for options in ({},{'bad_source':True},{'trusted':False},{'profile':'usa'},{'omit_font':True},{'omit':True}):
            with self.subTest(options=options),tempfile.TemporaryDirectory(prefix='viewer-inline-') as directory:
                page=fixture(Path(directory),**options);parts=page.parent/'index.html.parts';home=parse_page(page)
                self.assertNotIn('inline_ui',home)
                if options.get('omit'):
                    self.assertFalse((parts/'text-inline-ui.html').exists());continue
                scoped=parse_page(parts/'text-inline-ui.html');row=next(x for x in home['resources'] if x['id']=='text.localized-inline-ui')
                self.assertEqual(row['viewer_scope'],'text-inline-ui')
                if options.get('bad_source') or options.get('trusted') is False or options.get('profile'):
                    self.assertIsNone(scoped['inline_ui'])
                else:
                    self.assertEqual(len(scoped['inline_ui']['directories']),7)
                    self.assertEqual(bool(scoped['inline_ui']['glyphs']),not options.get('omit_font',False))


@unittest.skipUnless(shutil.which('google-chrome'),'Google Chrome is unavailable')
class InlineBrowserTest(unittest.TestCase):
    def test_fallback_search_language_and_shared_settings(self):
        with tempfile.TemporaryDirectory(prefix='viewer-inline-browser-') as directory:
            work=Path(directory);index=fixture(work);browser=Chrome(work,index)
            try:
                target=browser.evaluate("document.querySelector('[data-resource-id=\"text.localized-inline-ui\"] a').href")
                browser.command('Page.navigate',{'url':target})
                wait_for(browser,"data.wiki.current==='text-inline-ui' && !document.querySelector('#inline-ui-text').hidden")
                self.assertTrue(browser.evaluate('''(()=>{const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change',{bubbles:true}));
                  if(q('#inline-ui-text>details>summary time').dateTime!=='2026-11-28T00:23:43Z'||
                     !['search','languages','shared','thresholds'].every(id=>q('#inline-ui-'+id+'>summary time').dateTime==='2026-11-28T00:23:43Z')||
                     q('#inline-directory').options.length!==7||q('#inline-language').options.length!==5||
                     q('#inline-language-rows').rows.length!==5||q('#inline-shared-rows').rows.length!==6||q('#inline-threshold-rows').rows.length!==6)
                    throw Error('initial');
                  q('#inline-normal-font').checked=true;change(q('#inline-normal-font'));q('#inline-hide-commands').checked=true;change(q('#inline-hide-commands'));
                  if(q('#inline-preview').textContent!=='A B')throw Error('normal font and spacing');
                  q('#inline-pin').checked=true;change(q('#inline-pin'));q('#inline-language').value='3';change(q('#inline-language'));
                  q('#inline-directory').value='1';change(q('#inline-directory'));
                  if(!q('#inline-source').textContent.includes('physical 0x39529D · selected 0x395262')||
                     !q('#inline-source').textContent.includes('guarded language-3 fallback')||q('#inline-preview').textContent!=='CCCCC')throw Error('fallback');
                  q('#inline-fallback').checked=false;change(q('#inline-fallback'));
                  if(!q('#inline-source').textContent.includes('selected 0x39529D')||!q('#inline-preview').textContent.startsWith('B'))throw Error('physical');
                  q('#inline-fallback').checked=true;change(q('#inline-fallback'));
                  q('#inline-search-query').value='BBB';q('#inline-search-find').click();
                  const row=[...q('#inline-search-rows').rows].find(r=>r.cells[0].textContent.includes('0x39529D'));
                  if(!row||!q('#inline-search-status').textContent.includes('matches'))throw Error('search');
                  row.querySelector('button').click();
                  return !q('#inline-fallback').checked&&q('#inline-language').value==='3'&&q('#inline-directory').value==='1'&&
                    q('#inline-source').textContent.includes('selected 0x39529D')&&q('#inline-pin').checked;})()'''))
                target=(index.parent/'index.html.parts/text-short.html').as_uri();browser.command('Page.navigate',{'url':target})
                wait_for(browser,"data.wiki.current==='text-short' && document.querySelector('#text-language').value==='3'")
                self.assertTrue(browser.evaluate("document.querySelector('#text-pin').checked && document.querySelector('#text-hide-commands').checked && document.querySelector('#text-normal-font').checked"))
            finally:
                browser.close()


if __name__=='__main__':unittest.main()
