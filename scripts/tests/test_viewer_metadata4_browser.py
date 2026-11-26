"""Check two-pair records and authenticated metadata cross references."""
import copy
import hashlib
import json
import shutil
import sys
import tempfile
import time
import unittest
from pathlib import Path
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import view_rom_data
from test_viewer_job_names_browser import fixture as job_fixture, payload
from viewer_browser_harness import Chrome


def pair_catalog():
    pairs = [{'slots':[{'selector':0,'value':0}, {'selector':0,'value':0}]} for _ in range(20)]
    pairs[0]['slots'] = [{'selector':0,'value':7}, {'selector':0,'value':8}]
    pairs[1]['slots'] = [{'selector':1,'value':4}, {'selector':2,'value':9}]
    pairs[2]['slots'] = [{'selector':0,'value':22}, {'selector':1,'value':11}]
    pairs[3]['slots'] = [{'selector':2,'value':1}, {'selector':1,'value':2}]
    return {'source_offset':0x6e6bf4, 'records':pairs}


def fixture(work, *, with_metadata=True, with_text=True, wrong_identity=False, split=True):
    dataset, _, metadata = job_fixture(work)
    manifest = dataset/'manifest.toml'; text=manifest.read_text()
    old_hash=hashlib.sha256((dataset/'linked-metadata52.json').read_bytes()).hexdigest()
    for index, row in enumerate(metadata['records']): row['bytes_20_33'][16]=index%20
    content=json.dumps(metadata).encode();(dataset/'linked-metadata52.json').write_bytes(content)
    metadata_sha=hashlib.sha256(content).hexdigest();text=text.replace(old_hash,metadata_sha)
    if not with_text or not with_metadata:
        chunks=text.split('[[resources]]\n')
        excluded=({'text.catalogs'} if not with_text else set())|({'gameplay.linked-metadata52'} if not with_metadata else set())
        text=chunks[0]+''.join('[[resources]]\n'+chunk for chunk in chunks[1:]
                             if not any(chunk.startswith('id="'+identity+'"\n') for identity in excluded))
    pair=pair_catalog();content=json.dumps(pair).encode();(dataset/'metadata4.json').write_bytes(content)
    pair_sha=hashlib.sha256(content).hexdigest();text_sha=hashlib.sha256((dataset/'text-catalogs.json').read_bytes()).hexdigest()
    text+=(f'[[resources]]\nid="gameplay.linked-metadata4"\npath="metadata4.json"\n'
           f'format="linked-metadata4-v1-json"\nsha256="{pair_sha}"\n')
    manifest.write_text(text)
    with patch.object(view_rom_data,'EUROPE_METADATA4_SHA256','wrong' if wrong_identity else pair_sha), \
         patch.object(view_rom_data,'EUROPE_METADATA52_SHA256',metadata_sha), \
         patch.object(view_rom_data,'EUROPE_TEXT_CATALOG_SHA256',text_sha):
        page=view_rom_data.generate(dataset,work/'view',split=split)
    return page,(pair_sha,metadata_sha,text_sha),metadata


class Metadata4ProjectionTest(unittest.TestCase):
    def test_bytes_identity_missing_and_hole_validation(self):
        projected=view_rom_data.project_metadata4(pair_catalog())
        self.assertEqual(projected['records'][2]['slots'],[{'selector':0,'value':22},{'selector':1,'value':11}])
        for mutate in (lambda d:d['records'].pop(),lambda d:d['records'][0]['slots'].pop(),
                       lambda d:d['records'][0]['slots'][0].__setitem__('selector',True),
                       lambda d:d['records'][0]['slots'][0].__setitem__('value',256),
                       lambda d:d.__setitem__('source_offset',0)):
            bad=copy.deepcopy(pair_catalog());mutate(bad)
            with self.subTest(mutation=mutate),self.assertRaises(ValueError): view_rom_data.project_metadata4(bad)
        for kwargs in ({},{'with_text':False},{'with_metadata':False},{'wrong_identity':True}):
            with self.subTest(kwargs=kwargs),tempfile.TemporaryDirectory() as directory:
                page, hashes, _=fixture(Path(directory),**kwargs);world=payload(page.parent/'index.html.parts/world.html')
                self.assertEqual(len(world['metadata4']['records']),20)
                if kwargs.get('with_metadata',True) and not kwargs.get('wrong_identity'):
                    self.assertEqual(world['metadata4_links']['owners'],[i%20 for i in range(116)])
                    self.assertEqual(world['metadata4_links']['names_verified'],kwargs.get('with_text',True))
                else:self.assertIsNone(world['metadata4_links'])
                self.assertNotIn('metadata4',payload(page.parent/'index.html.parts/interface.html'))
        with tempfile.TemporaryDirectory() as directory:
            page,hashes,metadata=fixture(Path(directory));world=payload(page.parent/'index.html.parts/world.html')
            meta=view_rom_data.project_job_metadata(metadata);meta['path']='metadata.json'
            with patch.object(view_rom_data,'EUROPE_METADATA4_SHA256',hashes[0]), \
                 patch.object(view_rom_data,'EUROPE_METADATA52_SHA256',hashes[1]), \
                 patch.object(view_rom_data,'EUROPE_TEXT_CATALOG_SHA256',hashes[2]):
                link=view_rom_data.link_metadata4
                self.assertTrue(link('europe',*hashes,world['metadata4'],meta,world['metadata_names'])['names_verified'])
                self.assertFalse(link('europe',hashes[0],hashes[1],'wrong',world['metadata4'],meta,world['metadata_names'])['names_verified'])
                self.assertIsNone(link('usa',*hashes,world['metadata4'],meta,world['metadata_names']))
                bad=copy.deepcopy(meta);bad['records'][0]['bytes_20_33'][16]=20
                self.assertIsNone(link('europe',*hashes,world['metadata4'],bad,world['metadata_names']))
                bad=copy.deepcopy(world['metadata4']);bad['records'][0]['slots'][0]['selector']=116
                self.assertIsNone(link('europe',*hashes,bad,meta,world['metadata_names']))


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class Metadata4BrowserTest(unittest.TestCase):
    def test_links_holes_names_filters_and_bookmarks(self):
        with tempfile.TemporaryDirectory(prefix='viewer-metadata4-') as directory:
            work=Path(directory);page,_,_=fixture(work);browser=Chrome(work,page.parent/'index.html.parts/world.html')
            try:
                bookmark=browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change',{bubbles:true}));
                  const section=q('#metadata4'),select=q('#metadata4-record');
                  if(section.hidden || select.options.length!==20 || !q('[data-resource-id="gameplay.linked-metadata4"] a[href="#metadata4"]'))throw Error('view');
                  select.value='2';change(select);let rows=q('#metadata4-slots').rows;
                  if(rows[0].cells[1].querySelector('a') || rows[0].cells[2].textContent!=='22' ||
                     !rows[1].cells[1].querySelector('a') || rows[1].cells[3].textContent!=='Yes' ||
                     rows[1].cells[4].textContent!=='No' || !q('#metadata4-summary').textContent.includes('1 nonzero'))throw Error('hole');
                  rows[1].cells[1].querySelector('a').click();
                  if(q('#metadata-name-filter-mode').value!=='exact-id' || q('#metadata-name-rows tr').dataset.metadataId!=='1')throw Error('selector link');
                  const refs=q('#metadata-name-rows tr [data-metadata4-record="2"][data-metadata4-slot="1"]');
                  if(!refs)throw Error('reverse selector link');refs.click();if(select.value!=='2')throw Error('reverse target');
                  select.value='1';change(select);
                  if(q('#metadata4-owners').rows.length!==6 || !q('#metadata4-owner-status').textContent.includes('6 references'))throw Error('owner links');
                  q('#metadata4-owner-filter').value='21';q('#metadata4-owner-filter').dispatchEvent(new Event('input'));
                  if(q('#metadata4-owners').rows.length!==1 || q('#metadata4-owners tr').dataset.metadataId!=='21')throw Error('owner filter');
                  q('#metadata4-owners a').click();
                  if(q('#metadata-name-rows tr').dataset.metadataId!=='21')throw Error('owner navigation');
                  const owner=q('#metadata-name-rows tr [data-metadata4-owner="1"]');if(!owner)throw Error('owner reverse');owner.click();
                  if(select.value!=='1' || !section.querySelector(':scope > details').open)throw Error('owner reverse target');
                  const link=section.querySelector(':scope > details > p > a');link.dispatchEvent(new Event('focus'));return link.href;
                })()''')
                browser.command('Page.navigate',{'url':bookmark});deadline=time.monotonic()+30
                while time.monotonic()<deadline:
                    try:
                        if browser.evaluate("document.readyState==='complete' && document.querySelector('#metadata4-record').value==='1' && document.querySelector('#metadata4>details').open"):break
                    except (RuntimeError,OSError):pass
                    time.sleep(.05)
                else:self.fail('Bookmark did not restore selected pair')
            finally:browser.close()

    def test_numeric_fallback_and_standalone_records(self):
        for kwargs in ({'with_text':False},{'with_metadata':False},{'wrong_identity':True}):
            with self.subTest(kwargs=kwargs),tempfile.TemporaryDirectory() as directory:
                work=Path(directory);page,_,_=fixture(work,split=False,**kwargs);browser=Chrome(work,page)
                try:
                    result=browser.evaluate('''(()=>{const q=s=>document.querySelector(s),select=q('#metadata4-record');
                      select.value='1';select.dispatchEvent(new Event('change'));
                      return {hidden:q('#metadata4').hidden,link:!!q('#metadata4-slots a'),
                        label:q('#metadata4-slots tr:first-child td:nth-child(2)').textContent,
                        owners:q('#metadata4-owner-status').textContent};})()''')
                    self.assertFalse(result['hidden'])
                    self.assertEqual(result['link'],kwargs.get('with_text') is False)
                    if kwargs.get('with_text') is False:self.assertEqual(result['label'],'1')
                    else:self.assertIn('unavailable',result['owners'])
                finally:browser.close()
