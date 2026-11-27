"""Check conditional record browsing and fixed-state diagnostic semantics."""
import copy
import hashlib
import json
import random
import shutil
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import view_rom_data
from test_viewer_scoped_browser import parse_page
from test_viewer_special_units_browser import wait_for
from viewer_browser_harness import Chrome


def fixture(work, *, verified=True, profile='europe'):
    dataset=work/'dataset';dataset.mkdir();records=[]
    for i in range(127):
        records.append({'index':i,'field00':1 if i<20 else 0,'field01':0,'key02':1,'operand04':1,'key05':2,'operand07':0})
    records[2].update(field01=7,key02=0x603,operand04=2,key05=0x603,operand07=5)
    records[3].update(key02=0,operand04=0,key05=0,operand07=255)
    records[4].update(key02=1,operand04=2,key05=2,operand07=3)
    catalog={'source_offset':0x713744,'records':records};content=json.dumps(catalog).encode();digest=hashlib.sha256(content).hexdigest()
    (dataset/'conditions.json').write_bytes(content)
    (dataset/'manifest.toml').write_text(f'profile="{profile}"\nextractor_revision=1\ndecoder_revision=1\n'
        '[[resources]]\nid="gameplay.conditional-records8"\npath="conditions.json"\n'
        f'format="conditional-records8-v1-json"\nsha256="{digest}"\n')
    with patch.object(view_rom_data,'EUROPE_CONDITIONAL_RECORDS8_SHA256',digest if verified else 'wrong'):
        page=view_rom_data.generate(dataset,work/'view',split=True)
    return page,catalog


def expected(record,context,first,second):
    if not record['field00']:return False
    if record['field01'] and record['field01']!=context:return False
    if record['key02']>=0x600:
        if first<record['operand04']:return False
    elif (first==1 and record['operand04']==0) or (first==0 and record['operand04']==1):return False
    if record['key05']==0:return True
    if record['key05']==record['key02']:second=first
    if record['key05']>=0x600:return second<record['operand07']
    return not((second==1 and record['operand07']==1) or (second==0 and record['operand07']==0))


class ConditionalRecordsProjectionTest(unittest.TestCase):
    def test_shape_identity_and_scope_guards(self):
        for options in ({},{'verified':False},{'profile':'usa'}):
            with self.subTest(options=options),tempfile.TemporaryDirectory(prefix='viewer-cond8-project-') as directory:
                page,catalog=fixture(Path(directory),**options);parts=page.parent/'index.html.parts'
                home,world,audio=map(parse_page,(page,parts/'world.html',parts/'audio.html'))
                self.assertNotIn('conditional_records8',home);self.assertNotIn('conditional_records8',audio)
                if options.get('profile')=='usa':self.assertIsNone(world['conditional_records8']);continue
                self.assertEqual(world['conditional_records8']['records'],catalog['records'])
                self.assertEqual(world['conditional_records8']['reader_verified'],options.get('verified',True))
                self.assertIn('conditional-records8',[s['id'] for p in home['wiki']['pages'] if p['id']=='world' for s in p['sections']])
                for change in (lambda c:c.update(source_offset=True),lambda c:c['records'].pop(),
                               lambda c:c['records'][0].update(index=True),lambda c:c['records'][0].update(key02=65536),
                               lambda c:c['records'][0].update(operand07=-1)):
                    bad=copy.deepcopy(catalog);change(bad)
                    with self.assertRaises(ValueError):view_rom_data.project_conditional_records8(bad)


@unittest.skipUnless(shutil.which('google-chrome'),'Google Chrome is unavailable')
class ConditionalRecordsBrowserTest(unittest.TestCase):
    def test_browse_filters_diagnostic_and_independent_cases(self):
        with tempfile.TemporaryDirectory(prefix='viewer-cond8-browser-') as directory:
            work=Path(directory);page,_=fixture(work);browser=Chrome(work,page)
            try:
                browser.click('#rows [data-resource-id="gameplay.conditional-records8"] a')
                wait_for(browser,"data.wiki.current==='world' && !document.querySelector('#conditional-records8').hidden")
                browser.click('#rows [data-resource-id="gameplay.conditional-records8"] a[href="#conditional-records8"]')
                self.assertTrue(browser.evaluate('''(()=>{const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change')),
                  input=e=>e.dispatchEvent(new Event('input'));
                  if(!q('#conditional-records8>details').open || q('#conditional8-rows').rows.length!==25 ||
                     !q('#conditional8-page').textContent.includes('127 slots'))throw Error('initial');
                  q('#conditional8-next').click();if(!q('#conditional8-page').textContent.includes('page 2'))throw Error('page');
                  q('#conditional8-gate').value='enabled';change(q('#conditional8-gate'));
                  if(q('#conditional8-rows').rows.length!==20)throw Error('gate');
                  q('#conditional8-filter').value='0x0603';input(q('#conditional8-filter'));
                  if(q('#conditional8-rows').rows.length!==1 || q('#conditional8-rows tr').dataset.recordId!=='2')throw Error('key filter');
                  q('#conditional8-rows a').click();
                  if(!q('#conditional8-diagnostic').open || q('#conditional8-id').value!=='2' || !q('#conditional8-second-control').hidden)throw Error('shared counter');
                  q('#conditional8-context').value='7';input(q('#conditional8-context'));
                  q('#conditional8-first').value='3';input(q('#conditional8-first'));
                  if(!q('#conditional8-result').textContent.startsWith('Matches'))throw Error('match');
                  q('#conditional8-first').value='5';input(q('#conditional8-first'));
                  if(!q('#conditional8-result').textContent.includes('second predicate rejects'))throw Error('second');
                  q('#conditional8-context').value='8';input(q('#conditional8-context'));
                  if(!q('#conditional8-result').textContent.includes('Context does not match'))throw Error('context');
                  q('#conditional8-id').value='3';change(q('#conditional8-id'));
                  if(!q('#conditional8-second-control').hidden || !q('#conditional8-detail').textContent.includes('No second predicate'))throw Error('bypass');
                  q('#conditional8-first').value='2';input(q('#conditional8-first'));
                  return q('#conditional8-result').textContent.includes('valid context and state');})()'''))
                rng=random.Random(713744);keys=[0,1,2,0x5ff,0x600,0x603,0xffff];cases=[]
                for _ in range(1500):
                    a,b=rng.choice(keys),rng.choice(keys);record={'field00':rng.randrange(2),'field01':rng.choice([0,1,7,255]),
                        'key02':a,'operand04':rng.choice([0,1,2,255]),'key05':b,'operand07':rng.choice([0,1,2,255])}
                    cases.append({'record':record,'context':rng.choice([0,1,7,255,0xffffffff]),
                                  'first':rng.randrange(256 if a>=0x600 else 2),'second':rng.randrange(256 if b>=0x600 else 2)})
                got=browser.evaluate('''(()=>{const cases=CASES;return cases.map(c=>conditional8Diagnostic(c.record,c.context,c.first,c.second).matched);})()'''.replace('CASES',json.dumps(cases)))
                self.assertEqual(got,[expected(c['record'],c['context'],c['first'],c['second']) for c in cases])
            finally:browser.close()

    def test_unverified_data_remains_raw(self):
        with tempfile.TemporaryDirectory(prefix='viewer-cond8-guard-') as directory:
            work=Path(directory);page,_=fixture(work,verified=False);browser=Chrome(work,page.parent/'index.html.parts/world.html')
            try:self.assertTrue(browser.evaluate('''(()=>{const q=s=>document.querySelector(s);return !q('#conditional-records8').hidden &&
                q('#conditional8-diagnostic').hidden && !q('#conditional8-unverified').hidden &&
                q('#conditional8-rows tr').cells[3].textContent==='0' && q('#conditional8-rows tr').cells[4].textContent.includes('key 0x0001') &&
                !q('#conditional8-rows tr').cells[4].textContent.includes('must be');})()'''))
            finally:browser.close()
