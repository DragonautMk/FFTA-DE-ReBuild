"""Exercise physical UI choice lists and their cursor positions in Chrome."""
import hashlib
import json
import shutil
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from view_rom_data import generate
from viewer_browser_harness import Chrome


def fixture(work):
    dataset = work / 'dataset'
    dataset.mkdir()
    first = {'identifier': 1, 'cursor_x': 0, 'cursor_y': 0, 'opaque': 0,
             'navigation': {'left': 0, 'right': 1, 'up': 2, 'down': 3}}
    second = {'identifier': 255, 'cursor_x': 128, 'cursor_y': 255, 'opaque': 254,
              'navigation': {'left': 128, 'right': 255, 'up': 7, 'down': 8}}
    same_position = {**second, 'identifier': 10}
    value = {'lists': [{'id': 'choice-a', 'source_offset': 100, 'kind': 'direct-list', 'entries': [first, second, same_position]},
                       {'id': 'choice-b', 'source_offset': 128, 'kind': 'filtered-template', 'entries': [second]}]}
    content = json.dumps(value).encode()
    (dataset / 'choices.json').write_bytes(content)
    (dataset / 'manifest.toml').write_text(
        'profile="test"\nextractor_revision=1\ndecoder_revision=1\n'
        '[[resources]]\nid="engine.ui-choice-lists"\npath="choices.json"\n'
        'format="ui-choice-lists-v1-json"\nsha256="' + hashlib.sha256(content).hexdigest() + '"\n')
    return generate(dataset, work / 'view')


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class ViewerUiChoicesTest(unittest.TestCase):
    def test_choice_positions_raw_fields_and_physical_lists(self):
        with tempfile.TemporaryDirectory(prefix='viewer-ui-choices-') as directory:
            work = Path(directory)
            browser = Chrome(work, fixture(work))
            try:
                self.assertEqual(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change'));
                  if(q('#ui-choices').hidden || q('#ui-choices>details').open)throw Error('disclosure');
                  const row=[...document.querySelectorAll('#rows tr')].find(r=>r.cells[0].textContent.includes('engine.ui-choice-lists'));
                  if(!row || !row.querySelector('a'))throw Error('resource');row.querySelector('a').click();
                  if(!q('#ui-choices>details').open || q('#choice-list').options.length!==2 || q('#choice-rows').rows.length!==3)throw Error('list');
                  q('#choice-rows').rows[1].querySelector('button').click();
                  if(q('#choice-entry').value!=='1' || !q('#choice-selected').textContent.includes('cursor (128, 255)') ||
                     !q('#choice-selected').textContent.includes('left 0x80') || !q('#choice-selected').textContent.includes('right 0xFF') ||
                     q('#choice-rows').rows[1].cells[1].textContent!=='0x6C' || q('#choice-rows').rows[1].cells[5].textContent!=='0xFE')throw Error('raw entry');
                  const canvas=q('#choice-canvas'),ctx=canvas.getContext('2d');
                  if([...ctx.getImageData(193,360,1,1).data].join(',')!=='224,86,43,255')throw Error('selected pixel');
                  const rect=canvas.getBoundingClientRect();
                  canvas.dispatchEvent(new MouseEvent('click',{clientX:rect.left+24/384*rect.width,clientY:rect.top+24/384*rect.height}));
                  if(q('#choice-entry').value!=='0')throw Error('canvas selection');
                  q('#choice-list').value='1';change(q('#choice-list'));
                  if(!q('#choice-source').textContent.includes('choice-b') || !q('#choice-source').textContent.includes('filtered-template') ||
                     q('#choice-entry').options.length!==1 || q('#choice-rows').rows[0].cells[1].textContent!=='0x80')throw Error('physical list');
                  return 'PASS';
                })()'''), 'PASS')
            finally:
                browser.close()
