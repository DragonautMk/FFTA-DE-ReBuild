"""Check source-gated action-to-four-byte-record links and callback catalogs."""
import copy
import hashlib
import json
import shutil
import sys
import tempfile
import unittest
from contextlib import ExitStack
from pathlib import Path
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import view_rom_data
from test_viewer_action_names_browser import fixture as action_fixture
from test_viewer_scoped_browser import parse_page
from viewer_browser_harness import Chrome


def fixture(work, *, wrong_actions=False, wrong_slots=False, wrong_text=False, callbacks=True, profile='europe'):
    action_fixture(work)
    dataset=work/'dataset'; manifest=dataset/'manifest.toml'; text=manifest.read_text().replace('profile="europe"',f'profile="{profile}"')
    action_path=dataset/'records.json'; old=action_path.read_bytes(); actions=json.loads(old)
    for index,row in enumerate(actions['records']):row['inline0c_0f']=[index%3,2,0,255]
    raw=json.dumps(actions).encode();action_path.write_bytes(raw);action_sha=hashlib.sha256(raw).hexdigest()
    text=text.replace(hashlib.sha256(old).hexdigest(),action_sha)
    records={'source_offset':0x7133f8,'records':[{'index':i,'field00':i%256,'field01':(i+1)%256,
                 'field02':(i+2)%256,'field03':(i+3)%256} for i in range(211)]}
    if callbacks:
        records['callback_tables']=[[0x08000001+2*i for i in range(count)] for count in (29,32,46)]
        records['action_callbacks']=[{'callback':0x08000001+2*i,'opaque_first':i,'opaque_second':0xffffffff-i} for i in range(93)]
    raw=json.dumps(records).encode();slots_sha=hashlib.sha256(raw).hexdigest();(dataset/'linked-records4.json').write_bytes(raw)
    text+=(f'[[resources]]\nid="gameplay.linked-records4"\npath="linked-records4.json"\n'
           f'format="linked-records4-v1-json"\nsha256="{slots_sha}"\n');manifest.write_text(text)
    text_sha=hashlib.sha256((dataset/'text.json').read_bytes()).hexdigest()
    with ExitStack() as stack:
        for name,value in {'EUROPE_ACTION_RECORDS_SHA256':'wrong' if wrong_actions else action_sha,
                           'EUROPE_LINKED_RECORDS4_SHA256':'wrong' if wrong_slots else slots_sha,
                           'EUROPE_TEXT_CATALOG_SHA256':'wrong' if wrong_text else text_sha}.items():
            stack.enter_context(patch.object(view_rom_data,name,value))
        page=view_rom_data.generate(dataset,work/'linked-view',split=True)
    return page,records,actions


class ActionSlotsProjectionTest(unittest.TestCase):
    def test_projection_and_identity_guards(self):
        for options in ({},{'wrong_actions':True},{'wrong_slots':True},{'wrong_text':True},{'callbacks':False},{'profile':'usa'}):
            with self.subTest(options=options),tempfile.TemporaryDirectory(prefix='viewer-action-slots-project-') as directory:
                page,records,actions=fixture(Path(directory),**options);parts=page.parent/'index.html.parts'
                world,home,audio=map(parse_page,(parts/'world.html',page,parts/'audio.html'))
                self.assertNotIn('linked_records4',home);self.assertNotIn('linked_records4',audio)
                if options.get('profile')=='usa':self.assertIsNone(world['linked_records4']);continue
                slot=world['linked_records4'];self.assertEqual(slot['records'],records['records'])
                self.assertEqual(bool(slot['callback_tables']),options.get('callbacks',True))
                status=next(row['status'] for row in world['resources'] if row['id']=='gameplay.linked-records4')
                self.assertEqual('callback sources' in status,options.get('callbacks',True))
                links=world['action_records4_links']
                if options.get('wrong_actions') or options.get('wrong_slots'):
                    self.assertIsNone(links);continue
                self.assertEqual(links['references'],[row['inline0c_0f'][:3] for row in actions['records']])
                self.assertEqual(links['names_verified'],not options.get('wrong_text',False))
                self.assertIn('linked-records4',[s['id'] for p in home['wiki']['pages'] if p['id']=='world' for s in p['sections']])
                for mutation in (lambda c:c.update(source_offset=True),lambda c:c['records'].pop(),
                                 lambda c:c['records'][0].update(index=True),lambda c:c['records'][0].update(field00=256)):
                    bad=copy.deepcopy(records);mutation(bad)
                    with self.assertRaises(ValueError):view_rom_data.project_linked_records4(bad)
                if records.get('callback_tables'):
                    bad=copy.deepcopy(records);bad['callback_tables'][0][0]=0x08000000
                    with self.assertRaises(ValueError):view_rom_data.project_linked_records4(bad)
                    bad=copy.deepcopy(records);bad['action_callbacks'][0]['opaque_first']=0x100000000
                    with self.assertRaises(ValueError):view_rom_data.project_linked_records4(bad)
                with patch.object(view_rom_data,'EUROPE_ACTION_RECORDS_SHA256','a'),patch.object(view_rom_data,'EUROPE_LINKED_RECORDS4_SHA256','b'):
                    altered=copy.deepcopy(world['action_records']);altered['records'][0]['inline0c_0f'][0]=211
                    self.assertIsNone(view_rom_data.link_action_records4('europe','a','b',altered,slot,None))
                    self.assertIsNone(view_rom_data.link_action_records4('usa','a','b',world['action_records'],slot,None))


@unittest.skipUnless(shutil.which('google-chrome'),'Google Chrome is unavailable')
class ActionSlotsBrowserTest(unittest.TestCase):
    def test_bidirectional_names_paging_and_callback_sources(self):
        with tempfile.TemporaryDirectory(prefix='viewer-action-slots-browser-') as directory:
            work=Path(directory);page,_,_=fixture(work);browser=Chrome(work,page.parent/'index.html.parts/world.html')
            try:
                self.assertTrue(browser.evaluate('''(()=>{const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change',{bubbles:true})),
                  input=e=>e.dispatchEvent(new Event('input',{bubbles:true}));
                  if(q('#linked-records4').hidden || q('#linked-records4>details').open ||
                     q('#linked-records4-id').options.length!==211 || q('#action-record-linked-slots').hidden ||
                     q('#action-record-linked-rows').rows.length!==3)throw Error('initial');
                  const third=q('#action-record-linked-rows tr:nth-child(2) a');
                  if(third.dataset.recordId!=='2' || third.dataset.inlineOffset!=='13' || q('#action-record-linked-rows').textContent.includes('255'))throw Error('inline limits');
                  third.click();
                  if(q('#linked-records4-id').value!=='2' || !q('#linked-records4>details').open ||
                     !q('#linked-records4-page').textContent.includes('40 references') || q('#linked-records4-rows').rows.length!==25)throw Error('forward');
                  q('#linked-records4-next').click();if(q('#linked-records4-rows').rows.length!==15)throw Error('pagination');
                  q('#linked-records4-language').value='1';change(q('#linked-records4-language'));
                  if(q('#action-name-language').value!=='1')throw Error('language sync');
                  q('#linked-records4-filter').value='A b';input(q('#linked-records4-filter'));
                  if(q('#linked-records4-rows').rows.length!==10)throw Error('name filter');
                  q('#linked-records4-filter').value='';input(q('#linked-records4-filter'));
                  q('#linked-records4-rows tr[data-action-id="1"] a').click();
                  if(q('#action-record-id').value!=='1' || !q('#action-records>details').open)throw Error('reverse');
                  q('#linked-records4-table-id').value='2';change(q('#linked-records4-table-id'));
                  if(q('#linked-records4-tables').hidden || q('#linked-records4-table-rows').rows.length!==46 ||
                     !q('#linked-records4-table-rows').textContent.includes('0x08000001'))throw Error('callback table');
                  q('#linked-records4-callback-id').value='92';change(q('#linked-records4-callback-id'));
                  if(q('#linked-records4-callbacks').hidden || q('#linked-records4-callback-rows').rows.length!==1 ||
                     !q('#linked-records4-callback-rows').textContent.includes('0xFFFFFFA3'))throw Error('callback record');
                  return true;})()'''))
            finally:browser.close()

    def test_unverified_and_missing_sources(self):
        for options in ({'wrong_actions':True},{'wrong_slots':True},{'wrong_text':True},{'callbacks':False}):
            with self.subTest(options=options),tempfile.TemporaryDirectory(prefix='viewer-action-slots-guard-') as directory:
                work=Path(directory);page,_,_=fixture(work,**options);browser=Chrome(work,page.parent/'index.html.parts/world.html')
                try:
                    state=browser.evaluate('''(()=>{const q=s=>document.querySelector(s);return {visible:!q('#linked-records4').hidden,
                      links:!q('#action-record-linked-slots').hidden,reverse:!q('#linked-records4-references').hidden,
                      language:!q('#linked-records4-language-control').hidden,tables:!q('#linked-records4-tables').hidden,
                      callbacks:!q('#linked-records4-callbacks').hidden};})()''')
                    verified=not(options.get('wrong_actions') or options.get('wrong_slots'))
                    self.assertTrue(state['visible']);self.assertEqual(state['links'],verified);self.assertEqual(state['reverse'],verified)
                    self.assertEqual(state['language'],verified and not options.get('wrong_text',False))
                    self.assertEqual(state['tables'],options.get('callbacks',True));self.assertEqual(state['callbacks'],options.get('callbacks',True))
                finally:browser.close()
