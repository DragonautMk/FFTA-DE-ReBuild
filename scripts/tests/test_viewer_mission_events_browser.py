"""Exercise mission, event and sparse parameter navigation in Chrome."""
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


def fixture(work, *, parameters_only=False):
    dataset = work / 'dataset'
    dataset.mkdir()
    def mission(index):
        return {'index': index, 'word00': 0x1234, 'packed02': [1, 2, 3], 'key05': 4, 'operand07': 5,
                'key08': 6, 'operand0a': 7, 'key0b': 8, 'operand0d': 9, 'packed0e': [10, 11, 12, 13],
                'inline12': list(range(16)), 'inline22': list(range(8)), 'inline2a': list(range(9)),
                'scaled33_raw': 14, 'byte34': 15, 'byte35': 16, 'mapped36_raw': 17, 'mapped37_raw': 18,
                'packed38': [19, 20, 21, 22, 23], 'mapped3d_raw': 24, 'scaled3e_raw': 25, 'byte3f': 26,
                'byte40': 27, 'flags41_raw': 0x8000, 'word43': 28, 'event_row45': 1 if index < 25 else 0}
    missions = {'source_offset': 100, 'records': [mission(i) for i in range(26)]}
    events = {'source_offset': 2000, 'records': [
        {'index': i, 'header00': 0, 'event_key01': i, 'primary02': [0, 2, 200][i],
         'alternate04': 5 if i == 1 else 0, 'parameter06': [0, 115, 200][i], 'byte07': 0,
         'flags08_raw': 0x8000, 'opaque0a': [9, 10]} for i in range(3)]}
    def parameter(index):
        return {'index': index, 'header00': 0, 'packed01': 0, 'packed02': 0, 'packed03': 0,
                'packed04': 0, 'packed05': 0, 'packed06': 0, 'value07': 65535, 'byte09': 0,
                'value0a': 0, 'signed0c': -7, 'flags0d': 0, 'byte0e': 0, 'byte0f': 0,
                'byte10': 0, 'byte11': 0, 'value12': 0, 'opaque14': 0, 'signed15': -128,
                'opaque16': [1, 2, 3, 4, 5]}
    parameters = {'source_offset': 10000, 'records': [parameter(0), parameter(115)]}
    headers = {'source_offset': 3000, 'headers': [
        {'index': i, 'unit_count': 0, 'map_argument': 0, 'unit_templates_pointer': 0,
         'placement_parameter0': 0, 'placement_parameter2': 0, 'placement_count0': 0,
         'random_position_range': 0, 'placement_count2': 0, 'position_pointers': [0] * 4,
         'unit_count_comparison': 0, 'nonzero_gate': 0, 'switch_selector': 0, 'zero_gate': 0,
         'script_default': 0, 'opaque': [0] * 7} for i in (1, 2)]}
    resources = [('parameter-records27', parameters)]
    if not parameters_only:
        resources += [('mission-records70', missions), ('event-records12', events), ('encounter-headers', headers)]
    manifest = 'profile="test"\nextractor_revision=1\ndecoder_revision=1\n'
    for name, value in resources:
        content = json.dumps(value).encode()
        (dataset / (name + '.json')).write_bytes(content)
        manifest += (f'[[resources]]\nid="gameplay.{name}"\npath="{name}.json"\n'
                     f'format="{name}-v1-json"\nsha256="{hashlib.sha256(content).hexdigest()}"\n')
    (dataset / 'manifest.toml').write_text(manifest)
    return generate(dataset, work / 'view')


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class ViewerMissionEventsTest(unittest.TestCase):
    def test_navigation_sparse_parameters_and_conditional_encounter(self):
        with tempfile.TemporaryDirectory(prefix='viewer-mission-events-') as directory:
            work = Path(directory)
            browser = Chrome(work, fixture(work))
            try:
                self.assertEqual(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change'));
                  if(q('#mission-records').hidden || q('#mission-records>details').open)throw Error('disclosure');
                  for(const [name,panel] of [['mission-records70','mission-record-panel'],['event-records12','event-record-panel'],
                                              ['parameter-records27','parameter-record-panel']]){
                    const row=[...document.querySelectorAll('#rows tr')].find(r=>r.cells[0].textContent.includes('gameplay.'+name+')'));
                    if(!row || !row.querySelector('a'))throw Error('resource '+name);row.querySelector('a').click();
                    if(!q('#'+panel).open)throw Error('panel '+name);
                  }
                  if(!q('#mission-record-fields').textContent.includes('4660 (0x1234)') ||
                     !q('#mission-record-fields').textContent.includes('0x01 0x02 0x03'))throw Error('mission fields');
                  q('#mission-event-open').click();
                  if(q('#event-record-id').value!=='1' || !q('#event-record-panel').open ||
                     !q('#event-mission-page').textContent.includes('25 references') || q('#event-mission-rows').rows.length!==20)throw Error('mission to event');
                  q('#event-mission-next').click();if(q('#event-mission-rows').rows.length!==5)throw Error('mission page');
                  q('#event-mission-rows a').click();if(q('#mission-record-id').value!=='20')throw Error('event to mission');
                  q('#event-parameter-open').click();
                  if(q('#parameter-record-id').value!=='115' || !q('#parameter-record-panel').open ||
                     !q('#parameter-record-source').textContent.includes('0x'+(10000+115*27).toString(16).toUpperCase()) ||
                     !q('#parameter-record-fields').textContent.includes('-7 (0xF9)') ||
                     !q('#parameter-record-fields').textContent.includes('-128 (0x80)'))throw Error('sparse parameter');
                  if(q('#parameter-event-rows').rows.length!==1 || q('#parameter-event-rows a').textContent!=='1')throw Error('parameter reverse');
                  q('#parameter-event-rows a').click();if(q('#event-record-id').value!=='1')throw Error('parameter to event');
                  q('#event-encounter-open').click();
                  if(q('#encounter-id').value!=='2' || !q('#encounter-header-details').open ||
                     q('#encounter-event-references').hidden || q('#encounter-event-rows a').textContent!=='1')throw Error('conditional encounter');
                  q('#encounter-event-rows a').click();if(q('#event-record-id').value!=='1')throw Error('encounter to event');
                  q('#event-record-id').value='2';change(q('#event-record-id'));
                  if(!q('#event-parameter-navigation').hidden || !q('#event-encounter-navigation').hidden ||
                     !q('#event-record-fields').textContent.includes('200 (0xC8)'))throw Error('missing target');
                  return 'PASS';
                })()'''), 'PASS')
            finally:
                browser.close()

    def test_parameter_catalog_without_related_resources(self):
        with tempfile.TemporaryDirectory(prefix='viewer-parameter-only-') as directory:
            work = Path(directory)
            browser = Chrome(work, fixture(work, parameters_only=True))
            try:
                self.assertEqual(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s);
                  if(q('#mission-records').hidden || !q('#mission-record-panel').hidden || !q('#event-record-panel').hidden ||
                     q('#parameter-record-panel').hidden || !q('#parameter-event-page').textContent.includes('unavailable'))throw Error('partial dataset');
                  q('#parameter-record-id').value='115';q('#parameter-record-id').dispatchEvent(new Event('change'));
                  if(!q('#parameter-record-fields').textContent.includes('-7 (0xF9)'))throw Error('standalone parameter');
                  return 'PASS';
                })()'''), 'PASS')
            finally:
                browser.close()
