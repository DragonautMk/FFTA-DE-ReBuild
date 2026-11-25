"""Check multi-bank compact OBJ catalog projections and animation navigation."""
import copy
import hashlib
import json
import shutil
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from view_rom_data import generate, project_compact_obj_catalog
from test_viewer_compact_obj_browser import compact
from viewer_browser_harness import Chrome


def catalog():
    def bank(shift):
        data=copy.deepcopy(compact())
        for item in data['lists']:item['list']['source_offset']+=shift
        for item in data['sequences']:item['sequence']['source_offset']+=shift
        return {'root':996+shift,'flags':12,'sequence_relative_offset':1004,
                'lists':data['lists'],'sequences':data['sequences']}
    return {'banks':[bank(0),bank(10000)],'selector_tables':{'unused':'not required for bank previews'}}


def fixture(work, value=None, *, split=True):
    dataset=work/'dataset';dataset.mkdir();items=[('graphics.conditional-compact-obj-catalog','compact-obj-catalog',15,catalog() if value is None else value),
     ('graphics.obj-custom-banks','obj-custom-banks',1,{'banks':[{'source_offset':8000,'words':[0x11111111]*8+[0x22222222]*8}]}),
     ('graphics.ui-raw-palettes','ui-raw-palettes',1,{'banks':[{'source_offset':9000,'colors':[0,0x001f,0x03e0]+[0]*13}]})]
    manifest='profile="test"\nextractor_revision=1\ndecoder_revision=1\n'
    for rid,name,version,content in items:
        body=json.dumps(content).encode();(dataset/(name+'.json')).write_bytes(body)
        manifest+=(f'[[resources]]\nid="{rid}"\npath="{name}.json"\nformat="{name}-v{version}-json"\n'
                   f'sha256="{hashlib.sha256(body).hexdigest()}"\n')
    (dataset/'manifest.toml').write_text(manifest)
    return generate(dataset,work/'view',split=split)


def payload(path):return json.JSONDecoder().raw_decode(path.read_text().split('const data = ',1)[1])[0]


class CompactCatalogProjectionTest(unittest.TestCase):
    def test_multiple_banks_keep_original_selectors_and_validate_roots(self):
        value=project_compact_obj_catalog(catalog());self.assertEqual(len(value['banks']),2)
        self.assertEqual([b['object_root'] for b in value['banks']],[1000,11000])
        self.assertEqual(value['banks'][0]['flags'],12)
        self.assertEqual(value['banks'][1]['sequences'][0]['frames'][1]['list_id'],19)
        mutations=[lambda c:c['banks'][1].__setitem__('root',996),
                   lambda c:c['banks'][0].__setitem__('sequence_relative_offset',1005),
                   lambda c:c['banks'][0].__setitem__('flags',True),
                   lambda c:c['banks'][0]['sequences'][0]['sequence']['frames'][0].__setitem__('list',88)]
        for mutate in mutations:
            value=copy.deepcopy(catalog());mutate(value)
            with self.subTest(mutation=mutate),self.assertRaises(ValueError):project_compact_obj_catalog(value)

    def test_catalog_scoped_and_invalid_catalog_is_not_rendered(self):
        with tempfile.TemporaryDirectory(prefix='viewer-compact-catalog-scope-') as directory:
            page=fixture(Path(directory));parts=page.parent/(page.name+'.parts')
            home,objects,world=(payload(p) for p in (page,parts/'objects.html',parts/'world.html'))
            self.assertNotIn('obj_animation_compact',home);self.assertNotIn('obj_animation_compact',world)
            self.assertEqual(len(objects['obj_animation_compact']['compact_catalog']['banks']),2)
            self.assertEqual(home['resources'][0]['viewer_scope'],'objects')
        with tempfile.TemporaryDirectory(prefix='viewer-compact-catalog-invalid-') as directory:
            bad=catalog();bad['banks'][0]['root']=0
            data=payload(fixture(Path(directory),bad,split=False))
            self.assertNotIn('compact_catalog',data['obj_animation_compact'])
            self.assertTrue(data['resources'][0]['status'].startswith('Invalid compact OBJ catalog:'))


@unittest.skipUnless(shutil.which('google-chrome'),'Google Chrome is unavailable')
class CompactCatalogBrowserTest(unittest.TestCase):
    def test_catalog_resource_links_banks_frames_and_preserved_settings(self):
        with tempfile.TemporaryDirectory(prefix='viewer-compact-catalog-') as directory:
            work=Path(directory);index=fixture(work);browser=Chrome(work,index.parent/(index.name+'.parts/objects.html'))
            try:
                self.assertEqual(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change'));
                  const row=[...q('#rows').rows].find(r=>r.dataset.resourceId==='graphics.conditional-compact-obj-catalog');
                  if(!row||!row.querySelector('a'))throw Error('resource link');row.querySelector('a').click();
                  if(q('#obj-animation-catalog').value!=='compact_catalog'||q('#obj-animation-bank').options.length!==2||
                     !q('#obj-animation-source').textContent.includes('bank flags 0x000C'))throw Error('catalog/flags');
                  q('#obj-animation-graphics').value='0';change(q('#obj-animation-graphics'));
                  q('#obj-animation-palette').value='0';change(q('#obj-animation-palette'));
                  q('#obj-animation-loop').checked=true;q('#obj-animation-rate').value='12';q('#obj-animation-play').click();
                  q('#obj-animation-bank').value='1';change(q('#obj-animation-bank'));
                  if(q('#obj-animation-stop').disabled||!q('#obj-animation-loop').checked||q('#obj-animation-rate').value!=='12'||
                     q('#obj-animation-graphics').value!=='0'||!q('#obj-animation-sequence').selectedOptions[0].textContent.startsWith('2 ·')||
                     !q('#obj-animation-frame').selectedOptions[0].textContent.includes('list 11'))throw Error('bank/settings');
                  q('#obj-animation-frame').value='1';change(q('#obj-animation-frame'));
                  if(!q('#obj-animation-detail').textContent.includes('Object list 19'))throw Error('sparse frame');
                  q('#obj-animation-stop').click();return 'PASS';
                })()'''),'PASS')
            finally:browser.close()
