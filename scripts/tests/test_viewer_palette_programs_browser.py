"""Exercise stored palette commands, color previews and producers in Chrome."""
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
    def header(width, count):
        return {'first': 0, 'flags': 0, 'width': width, 'command_count': count, 'initial_delay': 2}
    def command(opcode, parameter):
        return {'opcode': opcode, 'mode': 0, 'duration': 4, 'relative_parameter': 20, 'parameter': parameter}
    value = {'programs': [
        {'source_offset': 100, 'header': header(2, 3), 'commands': [
            command(12, {'kind': 'components', 'values': [-1, 2, 3]}),
            command(0, {'kind': 'color-words', 'colors': [0x001f, 0x83e0]}),
            command(11, {'kind': 'color-cycle', 'count': 3, 'direction': 1, 'colors': [0x001f, 0x83e0, 0x7c00]})]},
        {'source_offset': 200, 'header': header(1, 1), 'commands': [
            command(11, {'kind': 'color-cycle', 'count': 2, 'direction': 0, 'colors': [0, 0x7fff]})]},
        {'source_offset': 300, 'header': header(0, 0), 'commands': []}],
        'producers': [{'source_offset': 1000, 'data': {'kind': 'pointers', 'programs': [0, None, 1]}},
                      {'source_offset': 2000, 'data': {'kind': 'list', 'entries': [
                          {'program': 1, 'delay_override': 4, 'reserved': 5},
                          {'program': 0, 'delay_override': 7, 'reserved': 9}]}}]}
    content = json.dumps(value).encode()
    (dataset / 'programs.json').write_bytes(content)
    (dataset / 'manifest.toml').write_text(
        'profile="test"\nextractor_revision=1\ndecoder_revision=1\n'
        '[[resources]]\nid="graphics.palette-programs"\npath="programs.json"\n'
        'format="palette-programs-v1-json"\nsha256="' + hashlib.sha256(content).hexdigest() + '"\n')
    return generate(dataset, work / 'view')


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class ViewerPaletteProgramsTest(unittest.TestCase):
    def test_commands_cycles_producers_and_focus(self):
        with tempfile.TemporaryDirectory(prefix='viewer-palette-programs-') as directory:
            work = Path(directory)
            browser = Chrome(work, fixture(work))
            try:
                self.assertEqual(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change'));
                  window.pending=new Map();let serial=0;
                  window.setTimeout=(fn,delay)=>{const id=++serial;pending.set(id,{fn,delay});return id};
                  window.clearTimeout=id=>pending.delete(id);
                  window.firePaletteTimer=()=>{const [id,item]=pending.entries().next().value;pending.delete(id);item.fn();return item.delay};
                  if(q('#palette-programs').hidden || q('#palette-programs>details').open)throw Error('disclosure');
                  const row=[...document.querySelectorAll('#rows tr')].find(r=>r.cells[0].textContent.includes('graphics.palette-programs'));
                  if(!row || !row.querySelector('a'))throw Error('resource link');row.querySelector('a').click();
                  if(!q('#palette-programs>details').open || q('#palette-program-references').rows.length!==2 ||
                     q('#palette-program-producer-rows').rows[1].cells[1].textContent!=='null' ||
                     !q('#palette-program-detail').textContent.includes('[-1,2,3]'))throw Error('source and references');
                  q('#palette-program-command').value='1';change(q('#palette-program-command'));
                  if(!q('#palette-program-cycle').hidden || q('#palette-program-colors').children.length!==2 ||
                     !q('#palette-program-colors').children[1].textContent.includes('0x83E0'))throw Error('color words');
                  q('#palette-program-command').value='2';change(q('#palette-program-command'));
                  q('#palette-program-colors').children[1].click();
                  if(q('#palette-program-step').value!=='1' || q('#palette-program-progress').value!==2 ||
                     q('#palette-program-current').style.backgroundColor!=='rgb(0, 255, 0)')throw Error('selected color');
                  q('#palette-program-rate').value='20';q('#palette-program-loop').checked=true;q('#palette-program-play').click();
                  if(firePaletteTimer()!==50 || q('#palette-program-step').value!=='2')throw Error('play');
                  firePaletteTimer();if(q('#palette-program-step').value!=='0')throw Error('loop');
                  q('#palette-program-id').value='1';change(q('#palette-program-id'));
                  if(q('#palette-program-stop').disabled || !q('#palette-program-loop').checked || q('#palette-program-rate').value!=='20' ||
                     q('#palette-program-command').value!=='0' || q('#palette-program-references').rows.length!==2)throw Error('preserved settings');
                  firePaletteTimer();q('#palette-program-loop').checked=false;firePaletteTimer();
                  if(!q('#palette-program-stop').disabled || !q('#palette-program-status').textContent.includes('finished'))throw Error('finish');
                  q('#palette-program-producer').value='1';change(q('#palette-program-producer'));
                  if(q('#palette-program-producer-rows').rows[1].cells[2].textContent!=='0x0007' ||
                     q('#palette-program-producer-rows').rows[1].cells[3].textContent!=='0x0009')throw Error('producer metadata');
                  q('#palette-program-producer-rows').rows[1].querySelector('button').click();
                  if(q('#palette-program-id').value!=='0')throw Error('producer navigation');
                  q('#palette-program-id').value='2';change(q('#palette-program-id'));
                  if(!q('#palette-program-command').disabled || !q('#palette-program-play').disabled ||
                     !q('#palette-program-detail').textContent.includes('No stored commands'))throw Error('empty program');
                  q('#palette-program-id').value='0';change(q('#palette-program-id'));
                  q('#palette-program-command').value='2';change(q('#palette-program-command'));
                  return 'PASS';
                })()'''), 'PASS')
                self.assertTrue(browser.evaluate('''(()=>{const s=document.querySelector('#palette-program-command');s.focus();return document.activeElement===s})()'''))
                browser.click('#palette-program-play')
                self.assertEqual(browser.evaluate('document.activeElement.id'), 'palette-program-command')
            finally:
                browser.close()
