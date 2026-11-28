"""Validate the scoped localized layout and transition previews."""
import copy
import hashlib
import json
import shutil
import struct
import tempfile
import unittest
from pathlib import Path

from view_rom_data import generate, project_binary_segments, viewer_section_updates
from viewer_localized_layouts import FAMILIES, LITERALS, project_localized_layouts
from viewer_browser_harness import Chrome


def parts():
    targets = [root for _, _, roots in FAMILIES for root in roots for _ in range(4)]
    targets += [0x3980d4, 0x3980ee, 0x3980c0, 0x398d90, 0x398d90, 0x398d90, 0x398d90, 0x398da4]
    words = b''.join(struct.pack('<I', 0x08000000 + target) for target in targets); cursor = 0; result = []
    for offset, length in LITERALS:
        result.append((offset, 'reader_literals', words[cursor:cursor+length])); cursor += length
    assert cursor == len(words)
    objects = b''
    for count in (4,3,1):
        objects += struct.pack('<H',count)
        for index in range(count):objects += struct.pack('<3H', 0x4000 | ((-2 + index) & 255), 0x4000 | ((-3 + index) & 511), 0x1007 + index)
    blocks = []
    for root,count in sorted((root,count) for _,counts,roots in FAMILIES for root,count in zip(roots,counts)):
        block = b''.join(struct.pack('<I4H', 0x08398102, index, 6, (index * 3 - 10) & 0xffff, 0) for index in range(count))
        blocks.append((root,block))
    result.extend([(0x3980c0,'allocation_sizes',struct.pack('<5I',*[1792]*5)), (0x3980d4,'object_lists',objects),
                   (0x39810c,'layout_rows',b''.join(block for _,block in blocks)),
                   (0x398d90,'layout_parameters',bytes(range(20))),
                   (0x398da4,'transition_rows',b''.join(struct.pack('<4B',16-i,i,0,0) for i in range(17)))])
    return result


def sources(items):
    return [{'offset':offset,'role':role,'length':len(raw),'sha256':hashlib.sha256(raw).hexdigest()} for offset,role,raw in items]


def fixture(work, invalid=False):
    dataset=work/'dataset';dataset.mkdir();items=parts();raw=b''.join(x[2] for x in items);(dataset/'layouts.bin').write_bytes(raw)
    manifest=('profile="europe"\nextractor_revision=365\ndecoder_revision=364\n'
              '[[resources]]\nid="graphics.localized-layout-tables"\npath="layouts.bin"\nformat="localized-layout-tables-v1-bin"\n'
              f'sha256="{hashlib.sha256(raw).hexdigest()}"\n')
    for index,entry in enumerate(sources(items)):
        digest='0'*64 if invalid and index==len(items)-3 else entry['sha256']
        manifest+=(f'[[resources.sources]]\noffset={entry["offset"]}\nlength={entry["length"]}\n'
                   f'role="{entry["role"]}"\nsha256="{digest}"\n')
    (dataset/'manifest.toml').write_text(manifest)
    return generate(dataset,work/'view',split=True)


def payload(page):return json.JSONDecoder().raw_decode(page.read_text().split('const data = ',1)[1])[0]


class LocalizedLayoutProjectionTest(unittest.TestCase):
    def test_twenty_tables_objects_parameters_and_transitions(self):
        data=parts();result=project_localized_layouts(project_binary_segments(b''.join(x[2] for x in data),sources(data)))
        self.assertEqual([len(f['layouts']) for f in result['families']],[5]*4)
        self.assertEqual(sum(len(layout['rows']) for family in result['families'] for layout in family['layouts']),267)
        self.assertEqual([len(item['objects']) for item in result['object_lists']],[4,3,1])
        self.assertEqual(result['families'][0]['layouts'][0]['rows'][0],[0x08398102,0,6,65526,0])
        self.assertEqual(result['transitions'][0],[16,0,0,0]);self.assertEqual(result['transitions'][-1],[0,16,0,0])

    def test_bad_source_hash_literal_row_and_transition_rejected(self):
        original=parts();raw=b''.join(x[2] for x in original);bad=sources(original);bad[-3]['sha256']='0'*64
        with self.assertRaises(ValueError):project_binary_segments(raw,bad)
        for index,byte in ((0,0),(-3,0),(-1,0)):
            changed=copy.deepcopy(original);v=bytearray(changed[index][2]);v[byte]^=1;changed[index]=(changed[index][0],changed[index][1],bytes(v))
            with self.assertRaises(ValueError):project_localized_layouts(project_binary_segments(b''.join(x[2] for x in changed),sources(changed)))

    def test_scope_and_invalid_resource(self):
        for invalid in (False,True):
            with self.subTest(invalid=invalid),tempfile.TemporaryDirectory(prefix='viewer-layout-scope-') as directory:
                index=fixture(Path(directory),invalid);folder=index.parent/'index.html.parts';home,scope,other=map(payload,(index,folder/'interface-layouts.html',folder/'interface.html'))
                self.assertNotIn('localized_layout_tables',home);self.assertNotIn('localized_layout_tables',other)
                self.assertEqual(scope['localized_layout_tables'] is None,invalid)
                sections=[x['id'] for page in home['wiki']['pages'] if page['id']=='interface-layouts' for x in page['sections']]
                self.assertEqual(len(sections),0 if invalid else 3)


@unittest.skipUnless(shutil.which('google-chrome'),'Google Chrome is unavailable')
class LocalizedLayoutBrowserTest(unittest.TestCase):
    def test_positions_objects_transition_playback_and_badges(self):
        with tempfile.TemporaryDirectory(prefix='viewer-localized-layout-browser-') as directory:
            work=Path(directory);index=fixture(work);page=index.parent/'index.html.parts/interface-layouts.html';browser=Chrome(work,page)
            try:
                expected=viewer_section_updates()['interface-layouts']
                self.assertTrue(browser.evaluate('''(()=>{const expected=EXPECTED,q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change')),
                  input=e=>e.dispatchEvent(new Event('input'));window.pending=new Map();let serial=0;
                  window.setTimeout=(f,ms)=>{const id=++serial;pending.set(id,{f,ms});return id};window.clearTimeout=id=>pending.delete(id);
                  window.fire=()=>{const [id,t]=pending.entries().next().value;pending.delete(id);t.f();return t.ms};
                  for(const id of Object.keys(expected)){const badge=q('#'+id+'>details>summary time');
                    if(!badge||badge.dateTime!==expected[id].at||!badge.textContent.startsWith('Added ')||!q('[data-resource-id="graphics.localized-layout-tables"] a[href="#'+id+'"]'))throw Error('badge or link');}
                  if(q('#localized-layout-rows').rows.length!==14||!q('#localized-layout-rows').rows[0].cells[7].textContent.startsWith('-10'))throw Error('initial positions');
                  q('#localized-layout-origin-x').value='3';input(q('#localized-layout-origin-x'));q('#localized-layout-origin-y').value='4';input(q('#localized-layout-origin-y'));
                  if(!q('#localized-layout-rows').rows[0].cells[7].textContent.startsWith('14')||!q('#localized-layout-rows').rows[0].cells[8].textContent.startsWith('32'))throw Error('origin');
                  q('#localized-layout-builder').value='2';change(q('#localized-layout-builder'));q('#localized-layout-language').value='1';change(q('#localized-layout-language'));
                  if(q('#localized-layout-rows').rows.length!==17||q('#localized-layout-origin-x').value!=='3')throw Error('table selection');
                  let row=q('#localized-layout-object-rows').rows[0];if(row.cells[3].textContent!=='-3'||row.cells[4].textContent!=='-2'||!row.cells[5].textContent.includes('32×8'))throw Error('object decode');
                  q('#localized-layout-object-list').value='2';change(q('#localized-layout-object-list'));if(q('#localized-layout-object-rows').rows.length!==1)throw Error('object list');
                  q('#localized-layout-transitions>details').open=true;q('#localized-transition-entry').value='15';change(q('#localized-transition-entry'));
                  q('#localized-transition-loop').checked=true;q('#localized-transition-rate').value='10';q('#localized-transition-play').click();
                  if(fire()!==100||q('#localized-transition-entry').value!=='16'||q('#localized-transition-b').value!==16)throw Error('play');
                  if(fire()!==100||q('#localized-transition-entry').value!=='0'||!q('#localized-transition-loop').checked)throw Error('loop');
                  q('#localized-transition-stop').click();return q('#localized-transition-stop').disabled&&pending.size===0;})()'''.replace('EXPECTED',json.dumps(expected))))
            finally:browser.close()


if __name__=='__main__':unittest.main()
