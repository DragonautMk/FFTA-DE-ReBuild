"""Check compact source-table projection and links to witnessed sequences."""
import copy
import hashlib
import json
import shutil
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
from view_rom_data import generate,project_compact_obj_catalog,project_compact_catalog_auxiliary
from test_viewer_compact_catalog_browser import catalog as base_catalog
from viewer_browser_harness import Chrome


def catalog():
    template=base_catalog()['banks'][0];banks=[]
    for root in (0x6c58b8,0x6ccae4,0x6ccdec,0x6cac40):
        bank=copy.deepcopy(template);shift=root-bank['root'];bank['root']=root
        for item in bank['lists']:item['list']['source_offset']+=shift
        for item in bank['sequences']:item['sequence']['source_offset']+=shift
        banks.append(bank)
    table=lambda rows:{'source_offset':100,'rows':rows}
    motion=[{'selector':2 if i%2==0 else 7,'x':i,'y':1,'target_x':-2,'target_y':3} for i in range(30)]
    return {'banks':banks,'selector_tables':{'motion':table(motion),
        'scene':table([{'first_selector':2,'second_selector':1,'x':-1,'y':2}]),
        'grid_first':table([{'base_selector':3,'x_mode':3,'y_mode':0,'x':1,'y':2,'value':9}]),
        'script_lookup':table([2]),'effects':table([{'kind':1,'trigger':3}])},
        'red_spring_records':[{'source_offset':200,'bank_pointer':0x08000000+0x6c58b8,'halfwords':[7]+[0]*9}],
        'star_cross_records':[],
        'soil_evidence_metadata':{'flags':12,'sequence_relative_offset':8,'list_offsets':[8,16],
          'selector0_relative_offset':2,'frames':[{'list':11,'duration':2},{'list':19,'duration':3}]}}


def fixture(work,value=None):
    dataset=work/'dataset';dataset.mkdir();body=json.dumps(catalog() if value is None else value).encode()
    (dataset/'compact-obj-catalog.json').write_bytes(body)
    (dataset/'manifest.toml').write_text('profile="test"\nextractor_revision=1\ndecoder_revision=1\n'
      '[[resources]]\nid="graphics.conditional-compact-obj-catalog"\npath="compact-obj-catalog.json"\n'
      'format="compact-obj-catalog-v15-json"\nsha256="'+hashlib.sha256(body).hexdigest()+'"\n')
    return generate(dataset,work/'view',split=True)


def payload(path):return json.JSONDecoder().raw_decode(path.read_text().split('const data = ',1)[1])[0]


class CompactTablesProjectionTest(unittest.TestCase):
    def test_documented_links_optional_records_and_invalid_fields(self):
        raw=catalog();banks=project_compact_obj_catalog(raw)['banks'];aux=project_compact_catalog_auxiliary(raw,banks)
        groups={item['id']:item for item in aux['groups']}
        self.assertEqual(groups['motion']['links'][0][0],{'field':'selector','bank':0,'sequence':0,'selector':2})
        self.assertEqual([link['selector'] for link in groups['scene']['links'][0]],[2,2])
        self.assertEqual(groups['grid_first']['links'][0][0]['selector'],7)
        self.assertEqual(groups['red_spring_records']['links'][0][0]['selector'],7)
        self.assertEqual(groups['script_lookup']['links'][0],[])
        self.assertEqual(groups['soil_evidence_frames']['rows'][1],{'index':1,'list':19,'duration':3})
        self.assertIsNone(project_compact_catalog_auxiliary({'banks':raw['banks']},banks))
        for mutate in [lambda c:c['selector_tables']['motion']['rows'][0].__setitem__('x','bad'),
                       lambda c:c['red_spring_records'][0]['halfwords'].pop(),
                       lambda c:c['soil_evidence_metadata']['frames'][0].__setitem__('duration',256)]:
            bad=copy.deepcopy(raw);mutate(bad)
            with self.subTest(mutation=mutate),self.assertRaises(ValueError):project_compact_catalog_auxiliary(bad,banks)

    def test_invalid_auxiliary_preserves_animation_banks(self):
        with tempfile.TemporaryDirectory(prefix='viewer-compact-table-invalid-') as directory:
            bad=catalog();bad['selector_tables']['motion']['rows'][0]['x']='invalid'
            page=fixture(Path(directory),bad);data=payload(page.parent/(page.name+'.parts/objects.html'))
            self.assertEqual(len(data['obj_animation_compact']['compact_catalog']['banks']),4)
            self.assertNotIn('auxiliary',data['obj_animation_compact']['compact_catalog'])
            self.assertIn('invalid source tables',data['resources'][0]['status'])


@unittest.skipUnless(shutil.which('google-chrome'),'Google Chrome is unavailable')
class CompactTablesBrowserTest(unittest.TestCase):
    def test_table_filter_paging_and_bidirectional_links(self):
        with tempfile.TemporaryDirectory(prefix='viewer-compact-tables-') as directory:
            work=Path(directory);index=fixture(work);browser=Chrome(work,index.parent/(index.name+'.parts/objects.html'))
            try:
                self.assertEqual(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change',{bubbles:true}));
                  const group=id=>{const n=data.obj_animation_compact.compact_catalog.auxiliary.groups.findIndex(g=>g.id===id);
                    q('#compact-table-group').value=String(n);change(q('#compact-table-group'));return n;};
                  if(q('#compact-catalog-tables').hidden||q('#compact-catalog-tables>details').open)throw Error('disclosure');
                  const row=[...q('#rows').rows].find(r=>r.dataset.resourceId==='graphics.conditional-compact-obj-catalog');
                  if(!row||row.querySelectorAll('a[href^="#"]').length!==2)throw Error('two resource views');
                  [...row.querySelectorAll('a')].find(a=>a.hash==='#compact-catalog-tables').click();
                  if(!q('#compact-catalog-tables>details').open||q('#compact-table-rows').rows.length!==25||
                     !q('#compact-table-page').textContent.includes('of 30'))throw Error('pagination');
                  q('#compact-table-next').click();if(q('#compact-table-rows').rows[0].cells[0].textContent!=='25')throw Error('next');
                  q('#compact-table-filter').value='0x1D';q('#compact-table-filter').dispatchEvent(new Event('input'));
                  if(q('#compact-table-rows').rows.length!==1||q('#compact-table-rows').rows[0].cells[0].textContent!=='29')throw Error('filter');
                  q('#compact-table-filter').value='';q('#compact-table-filter').dispatchEvent(new Event('input'));
                  group('scene');if(q('#compact-table-rows').rows[0].querySelectorAll('a').length!==2)throw Error('scene links');
                  q('#compact-table-rows').rows[0].querySelectorAll('a')[1].click();
                  if(!q('#obj-animations>details').open||q('#obj-animation-catalog').value!=='compact_catalog'||
                     q('#obj-animation-bank').value!=='2'||!q('#obj-animation-sequence').selectedOptions[0].textContent.startsWith('2 ·'))throw Error('sequence navigation');
                  const reverse=[...document.querySelectorAll('#obj-animation-catalog-references a')].find(a=>a.textContent.includes('scene #0'));
                  if(!reverse)throw Error('reverse link');reverse.click();
                  if(q('#compact-table-group').selectedOptions[0].textContent!=='Scene'||q('#compact-table-rows').rows[0].cells[0].textContent!=='0')throw Error('table navigation');
                  group('grid_first');if(!q('#compact-table-rows').rows[0].querySelector('a').textContent.includes('selector 7'))throw Error('grid computation');
                  group('red_spring_records');if(!q('#compact-table-rows').rows[0].textContent.includes('[7,0,0,0,0,0,0,0,0,0]')||
                    !q('#compact-table-rows').rows[0].querySelector('a'))throw Error('raw record');
                  group('soil_evidence_frames');if(q('#compact-table-rows').rows.length!==2||q('#compact-table-rows').rows[1].querySelector('a'))throw Error('unlinked metadata');
                  group('motion');q('#compact-table-next').click();q('#compact-table-rows').rows[4].querySelector('a').click();
                  const back=[...document.querySelectorAll('#obj-animation-catalog-references a')].find(a=>a.textContent.includes('motion #29'));
                  if(!back)throw Error('reverse later row');back.click();
                  if(q('#compact-table-rows').rows[4].cells[0].textContent!=='29')throw Error('reverse page');
                  return 'PASS';
                })()'''),'PASS')
            finally:browser.close()
