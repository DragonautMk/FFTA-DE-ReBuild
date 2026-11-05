"""Public synthetic export coverage; no ROM, game state or personal fixture."""
import argparse
import contextlib
import hashlib
import io
import json
import os
from pathlib import Path
import sys
import re
import shutil
import subprocess
import tempfile
import unittest
from unittest import mock
import zipfile

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import debug_asset_inventory as inv
import export_debug_assets as exporter


def snapshot(rom='a' * 64, payload=b'public synthetic payload'):
    return b'FFTA-ST\0' + (1).to_bytes(4, 'little') + bytes.fromhex(rom) + bytes(32) + hashlib.sha256(payload).digest() + payload


class DebugAssetExportTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory(prefix="ffta export ' à ")
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)
        patch = mock.patch.object(exporter, 'ROOT', self.root)
        patch.start(); self.addCleanup(patch.stop)
        self.assets = self.root/'debug_saves'
        self.ready = self.assets/'ready'
        self.state = self.write('debug_saves/ready/EUR/SaveStates/'+ 'a'*64 + '/slot-1.ffstate', snapshot())
        self.save = self.write('debug_saves/ready/EUR/ffta.sav', bytes(65536))
        self.loaded = self.write('debug_saves/ready/EUR/loaded/IN-07-file-1.ffstate', snapshot(payload=b'loaded checkpoint'))
        self.catalog = {'schema':1,'rom_sha256':{'EUR':'a'*64,'USA':'b'*64},
                        'cases':[{'id':'DS-01','name':'Public case','status':'parziale','checkpoints':[self.item(self.state,region='EUR')]}],
                        'loaded_checkpoints':[self.item(self.loaded,region='EUR',scope='Loaded map only')],
                        'eur_base':self.item(self.save)}
        self.write('debug_saves/ready/catalog.json',json.dumps(self.catalog).encode())
        self.write('saves/debug-cases/catalog.json',b'{"schema":1,"cases":[]}')
        pe = b'MZ'+bytes(58)+(64).to_bytes(4,'little')+b'PE\0\0\x64\x86'
        self.runtime = self.write('runtime.exe',pe)
        self.launcher = self.write('launcher.exe',pe)
        base_files={'FFTARecomp.exe':pe,'recomp.exe':pe,'LEGGIMI.txt':b'No personal cases or results are included.','portable.txt':b'portable'}
        manifest={'kind':'ffta-portable-package','platform':'windows','files':{n:{'sha256':hashlib.sha256(b).hexdigest()} for n,b in base_files.items()}}
        self.base=self.root/'base.zip'
        with zipfile.ZipFile(self.base,'w') as z:
            for n,b in base_files.items():z.writestr(exporter.BASE_PREFIX+n,b)
            z.writestr(exporter.BASE_PREFIX+'PACKAGE-MANIFEST.json',json.dumps(manifest))
        self.args=argparse.Namespace(base=self.base,runtime=self.runtime,launcher=self.launcher,assets=self.assets,
            legacy=self.root/'saves/debug-cases',source_manifest=self.root/'missing.json',recordings=self.root/'missing-recordings',
            replay=self.root/'missing-replay',output=self.root/'dist/out.zip',scan_root=[],exclude_scan=[])

    def write(self, name, data):
        p=self.root/name;p.parent.mkdir(parents=True,exist_ok=True);p.write_bytes(data);return p

    def item(self,p,**extra):
        return {'file':p.relative_to(self.ready).as_posix(),'sha256':exporter.digest(p),**extra}

    def export(self):
        with contextlib.redirect_stdout(io.StringIO()):exporter.export(self.args)
        with zipfile.ZipFile(self.args.output) as z:
            self.assertIsNone(z.testzip())
            return {n.removeprefix(exporter.PREFIX+'/'):z.read(n) for n in z.namelist()}

    def test_complete_export_exposes_loaded_states_later_saves_and_all_aliases(self):
        self.write('target/later/EUR/scene.ffstate',snapshot(payload=b'new scene'))
        self.write('target/later/EUR/alias.ffstate',snapshot(payload=b'new scene'))
        self.write('target/later/EUR/progress.sav',b'1'*65536)
        self.write('target/later/result.json',b'{"status":"partial"}')
        self.write('cheats/catalog.json',b'{"source":"synthetic"}')
        self.write('target/later/broken.ffstate',b'broken original')
        before={p:p.read_bytes() for p in self.assets.rglob('*') if p.is_file()}
        payloads=self.export();inventory=json.loads(payloads['DEBUG-ASSETS-INVENTORY.json'])
        rows={r['source']:r for r in inventory['files']}
        self.assertIn('target/later/result.json',rows);self.assertIn('cheats/catalog.json',rows)
        self.assertEqual(rows['target/later/EUR/scene.ffstate']['archive'],rows['target/later/EUR/alias.ffstate']['archive'])
        self.assertIn('issue',rows['target/later/broken.ffstate']);self.assertNotIn('catalog',rows['target/later/broken.ffstate'])
        main=json.loads(payloads['Data/config/debug-catalog/catalog.json'])
        self.assertEqual(main['variants'][0]['file'],self.loaded.relative_to(self.ready).as_posix())
        self.assertEqual(main['generated_standard_saves'][0]['id'],'BASE-EUR')
        self.assertEqual(payloads['DebugAssets/original-catalog.json'],before[self.ready/'catalog.json'])
        for row in rows.values():
            self.assertEqual(hashlib.sha256(payloads[row['archive']]).hexdigest(),row['sha256'])
            if 'catalog' in row:self.assertIn(row['catalog'],payloads)
        for name,b in payloads.items():
            if name.startswith('DEBUG-CATALOG-'):
                c=json.loads(b)
                for entry in c['variants']+c['generated_standard_saves']:
                    self.assertNotIn('..',Path(entry['file']).parts)
                    self.assertEqual(hashlib.sha256(payloads[entry['file']]).hexdigest(),entry['sha256'])
        self.assertIn(b'Choose / reload catalog',payloads['DEBUG-ASSETS.html'])
        self.assertTrue(all(p.read_bytes()==b for p,b in before.items()))
        manifest=json.loads(payloads['DEBUG-ASSETS-MANIFEST.json'])
        self.assertEqual(set(manifest['files']),set(payloads)-{'DEBUG-ASSETS-MANIFEST.json'})
        for n,m in manifest['files'].items():self.assertEqual(hashlib.sha256(payloads[n]).hexdigest(),m['sha256'])

    def test_invalid_loaded_hash_refuses_instead_of_silently_hiding_it(self):
        self.loaded.write_bytes(snapshot(payload=b'changed'))
        with self.assertRaisesRegex(ValueError,'Catalog checksum mismatch'):exporter.export(self.args)
        self.assertFalse(self.args.output.exists())

    def test_refuses_overwrite_and_changing_source(self):
        self.export();before=self.args.output.read_bytes()
        with self.assertRaisesRegex(ValueError,'already exists'):exporter.export(self.args)
        self.assertEqual(self.args.output.read_bytes(),before)
        self.args.output=self.root/'dist/changed.zip'
        orig=inv.index_html
        def change(*args):
            self.loaded.write_bytes(snapshot(payload=b'mutated during export'))
            return orig(*args)
        with mock.patch.object(inv,'index_html',side_effect=change):
            with self.assertRaisesRegex(ValueError,'Source changed before publication'):exporter.export(self.args)
        self.assertFalse(self.args.output.exists())

    def test_recordings_preserve_sibling_layout_and_incomplete_status(self):
        base='target/recording/EUR/case/'
        start=self.write(base+'start.ffstate',snapshot(payload=b'start'))
        end=self.write(base+'end.ffstate',snapshot(payload=b'end'))
        inputs=self.write(base+'input.input',b'0 A\n')
        self.write(base+'recording.json',json.dumps({'kind':'ffta-input-recording','status':'interrupted',
            'start_sha256':exporter.digest(start),'end_sha256':exporter.digest(end),'input_sha256':exporter.digest(inputs)}).encode())
        payloads=self.export();inventory=json.loads(payloads['DEBUG-ASSETS-INVENTORY.json'])
        group=inventory['recordings'][0];self.assertEqual(group['status'],'interrupted');self.assertEqual(group['identity_check'],'verified')
        for n in ('start.ffstate','end.ffstate','input.input','recording.json'):self.assertIn(group['directory']+'/'+n,payloads)

    def test_scan_exclusions_extra_roots_and_missing_inputs_are_explicit(self):
        self.write('target/later/scene.ffstate',snapshot())
        self.write('target/later/text-catalogs.json',b'extracted dataset')
        self.write('target/cache/hidden.ffstate',snapshot())
        self.write('target/later/skip/explicit.ffstate',snapshot())
        hidden=self.write('outside/private.ffstate',snapshot())
        (self.root/'target/link').symlink_to(hidden.parent,target_is_directory=True)
        (self.root/'target/later/link.ffstate').symlink_to(hidden)
        disguised=bytearray(192);disguised[0xB2]=0x96;disguised[0xAC:0xB0]=b'AFXP'
        self.write('target/later/disguised.ffstate',disguised)
        external=Path(self.tmp.name).parent/(self.root.name+'-extra');external.mkdir();self.addCleanup(lambda: __import__('shutil').rmtree(external))
        (external/'extra.ffstate').write_bytes(snapshot())
        rows,excluded,_=inv.scan([self.root,external],self.root,exporter.reject_rom,exporter.digest,excluded_paths=[self.root/'target/later/skip'])
        names={r['source'] for r in rows};self.assertIn('external-2/extra.ffstate',names)
        self.assertNotIn('target/later/disguised.ffstate',names);self.assertNotIn('target/later/text-catalogs.json',names)
        self.assertNotIn('target/later/link.ffstate',names);self.assertNotIn('target/cache/hidden.ffstate',names)
        reasons={r['source']:r['reason'] for r in excluded}
        for p in ('target/link','target/later/link.ffstate','target/later/disguised.ffstate','target/later/text-catalogs.json','target/later/skip'):self.assertIn(p,reasons)

    def test_catalog_pages_cover_more_than_runtime_entry_limit_and_unknown_rom(self):
        rows=[]
        for i in range(1100):
            sha=hashlib.sha256(str(i).encode()).hexdigest()
            rows.append({'source':f'target/EUR/{i}.ffstate','bytes':123,'sha256':sha,'kind':'snapshot','rom_sha256':'a'*64,'archive':f'DebugAssets/library/{sha}.ffstate'})
        rows.append({'source':'unknown.ffstate','kind':'snapshot','rom_sha256':'c'*64,'sha256':'f'*64,'bytes':123,'archive':'unknown.ffstate'})
        pages=inv.catalog_pages(rows,{'EUR':'a'*64},{})
        self.assertEqual(len(pages),5);self.assertEqual(sum(len(c['variants']) for c in pages.values()),1100)
        self.assertTrue(all(len(c['variants'])<=inv.CATALOG_PAGE for c in pages.values()))
        self.assertIn('issue',rows[-1]);self.assertNotIn('catalog',rows[-1])

    def test_preview_and_raw_snapshot_envelopes_are_authenticated(self):
        preview=bytes(240*160*4);core=snapshot()
        wrapped=b'FFTA-SL1'+len(core).to_bytes(8,'little')+hashlib.sha256(preview).digest()+preview+core
        p=self.write('wrapped.ffstate',wrapped);self.assertEqual(inv.state_identity(p)['rom_sha256'],'a'*64)
        self.assertEqual(inv.state_identity(p)['state_content_sha256'],hashlib.sha256(core).hexdigest())
        for offset in (8,48,len(wrapped)-1):
            damaged=bytearray(wrapped);damaged[offset]^=1;p.write_bytes(damaged)
            with self.assertRaises(ValueError):inv.state_identity(p)

    def test_reports_keep_nested_images_json_raw_graphics_and_audio_siblings(self):
        base='target/report/Data/config/graphics/capture/'
        self.write(base+'report.html',b'<a href="graphics.json">JSON</a><img src="images/frame.png"><a href="vram.bin">VRAM</a>')
        self.write(base+'graphics.json',b'{"status":"unverified"}')
        self.write(base+'images/frame.png',b'public synthetic image')
        self.write(base+'vram.bin',b'public synthetic VRAM')
        self.write(base+'before-mixed.pcm16le',b'public PCM')
        self.write(base+'before-hle.f32le',b'public float audio')
        self.write(base+'before-fifo0.events',b'public FIFO events')
        self.write(base+'native.bin',b'generated native code is excluded')
        self.write(base+'gba_bios.bin',b'BIOS placeholder excluded')
        self.write('target/audio/audio.json',b'{}')
        self.write('target/audio/left-fifo0.bin',b'public raw audio')
        data=self.export();inventory=json.loads(data['DEBUG-ASSETS-INVENTORY.json']);rows={r['source']:r for r in inventory['files']}
        folder=str(Path(rows[base+'report.html']['archive']).parent)
        for n in ('graphics.json','images/frame.png','vram.bin','before-mixed.pcm16le','before-hle.f32le','before-fifo0.events'):
            self.assertIn(folder+'/'+n,data)
        self.assertIn('target/audio/left-fifo0.bin',rows)
        self.assertNotIn(base+'native.bin',rows);self.assertNotIn(base+'gba_bios.bin',rows)

    def test_discovery_reports_corrupt_zip_and_rejects_disguised_rom_inside_archive(self):
        self.write('target/broken.zip',b'not a ZIP')
        header=bytearray(192);header[0xB2]=0x96;header[0xAC:0xB0]=b'AFXP'
        with zipfile.ZipFile(self.root/'target/disguised.zip','w') as z:z.writestr('progress.sav',header)
        rows,exclusions,_=inv.scan([self.root],self.root,exporter.reject_rom,exporter.digest)
        names={r['source'] for r in rows};self.assertNotIn('target/disguised.zip',names);self.assertNotIn('target/broken.zip',names)
        excluded={r['source'] for r in exclusions};self.assertIn('target/disguised.zip',excluded);self.assertIn('target/broken.zip',excluded)

    def test_representative_export_reduces_steps_but_preserves_slots_and_dependencies(self):
        for i in range(5):self.write(f'target/sequence/step-{i}.ffstate',snapshot(payload=f'step {i}'.encode()))
        for i in range(1,5):self.write(f'target/independent/slot-{i}.ffstate',snapshot(payload=f'slot {i}'.encode()))
        for i in range(4):self.write(f'target/navigation/{i:02}-move/navigation-end.ffstate',snapshot(payload=f'nav {i}'.encode()))
        self.write('target/protected/report.html',b'public synthetic report')
        for i in range(3):self.write(f'target/protected/step-{i}.ffstate',snapshot(payload=f'protected {i}'.encode()))
        self.args.keep_snapshot=[self.root/'target/sequence/step-2.ffstate']
        data=self.export();inventory=json.loads(data['DEBUG-ASSETS-INVENTORY.json']);sources={r['source'] for r in inventory['files']}
        for i in (0,2,4):self.assertIn(f'target/sequence/step-{i}.ffstate',sources)
        for i in (1,3):self.assertNotIn(f'target/sequence/step-{i}.ffstate',sources)
        for i in range(1,5):self.assertIn(f'target/independent/slot-{i}.ffstate',sources)
        for i in range(3):self.assertIn(f'target/protected/step-{i}.ffstate',sources)
        for i in (1,2):self.assertNotIn(f'target/navigation/{i:02}-move/navigation-end.ffstate',sources)
        selection=inventory['snapshot_selection'];self.assertEqual(selection['mode'],'representative')
        self.assertEqual(len(selection['omitted']),4)
        for row in selection['omitted']:
            self.assertIn(row['representative'],sources)
            self.assertTrue((self.root/row['source']).is_file())
            self.assertFalse(any(meta['sha256']==row['sha256'] for meta in json.loads(data['DEBUG-ASSETS-MANIFEST.json'])['files'].values()))
        self.args.output=self.root/'dist/all.zip';self.args.all_snapshots=True
        full=self.export();complete=json.loads(full['DEBUG-ASSETS-INVENTORY.json'])
        self.assertEqual(complete['snapshot_selection']['mode'],'all')
        self.assertEqual(complete['snapshot_selection']['omitted'],[])
        self.assertTrue(all(any(r['source']==o['source'] for r in complete['files']) for o in selection['omitted']))

    def test_selection_orders_numbers_numerically_and_separates_rom_and_bios(self):
        rows=[]
        for rom in ('a'*64,'b'*64):
            for i in (1,2,10,11):
                p=self.write(f'target/{rom}/boot-{i}.ffstate',snapshot(rom,payload=str(i).encode()))
                rows.append({'source':str(p.relative_to(self.root)),'path':p,'kind':'snapshot','sha256':exporter.digest(p),'bytes':p.stat().st_size,**inv.state_identity(p)})
        selected,omitted=inv.select_snapshots(rows)
        self.assertEqual({r['path'].stem for r in selected},{'boot-1','boot-11'})
        self.assertEqual(len(omitted),4)
        with self.assertRaisesRegex(ValueError,'was not discovered'):
            inv.select_snapshots(rows,keep=[self.root/'missing.ffstate'])

    def test_catalogs_merge_raw_and_preview_copies_of_the_same_core(self):
        core=snapshot();raw=self.write('target/scene/raw.ffstate',core)
        preview=bytes(240*160*4)
        wrapped=self.write('target/scene/preview.ffstate',b'FFTA-SL1'+len(core).to_bytes(8,'little')+hashlib.sha256(preview).digest()+preview+core)
        rows=[{'source':str(p.relative_to(self.root)),'path':p,'kind':'snapshot','sha256':exporter.digest(p),'bytes':p.stat().st_size,'archive':'DebugAssets/'+p.name,**inv.state_identity(p)} for p in (raw,wrapped)]
        pages=inv.catalog_pages(rows,{'EUR':'a'*64},{})
        entries=[e for c in pages.values() for e in c['variants']]
        self.assertEqual(len(entries),1);self.assertEqual(entries[0]['file'],'DebugAssets/preview.ffstate')

    def test_edition_entry_points_use_distinct_names_and_forward_overrides(self):
        for extended in (False,True):
            with mock.patch.object(exporter,'export') as run:
                exporter.main([],extended=extended)
                args=run.call_args.args[0]
                self.assertEqual(args.all_snapshots,extended)
                self.assertEqual('extended-EUR' in args.output.name,extended)
                self.assertEqual(args.output.parent,self.root/'dist/debug-assets')
            with mock.patch.object(exporter,'export') as run:
                exporter.main(['--output',str(self.root/'chosen.zip'),'--scan-root',str(self.root/'extra')],extended=extended)
                self.assertEqual(run.call_args.args[0].output,self.root/'chosen.zip')
                self.assertEqual(run.call_args.args[0].scan_root,[self.root/'extra'])
        entry=Path(exporter.__file__).with_name('export_debug_assets_extended.py')
        help_result=subprocess.run([sys.executable,str(entry),'--help'],capture_output=True,text=True)
        self.assertEqual(help_result.returncode,0,help_result.stderr)
        self.assertIn('--all-snapshots',help_result.stdout)

    def test_scan_skips_leased_scopes_without_deleting_or_reclaiming_them(self):
        self.write('target/active/.debug-artifacts-lease/owner.json',b'{"owner":"other thread"}')
        state=self.write('target/active/step-1.ffstate',snapshot())
        before=state.read_bytes()
        rows,exclusions,_=inv.scan([self.root],self.root,exporter.reject_rom,exporter.digest)
        self.assertFalse(any(r['source'].startswith('target/active/') for r in rows))
        self.assertTrue(any(r['source']=='target/active' and 'lease' in r['reason'] for r in exclusions))
        self.assertEqual(state.read_bytes(),before)
        self.assertTrue((self.root/'target/active/.debug-artifacts-lease/owner.json').is_file())

    @unittest.skipUnless(shutil.which('node'),'Node is optional for generated index UI checks')
    def test_search_index_defaults_to_unique_saves_and_finds_source_aliases(self):
        rows=[{'source':'target/scene.ffstate','kind':'snapshot','sha256':'a'*64,'archive':'state.ffstate'},
              {'source':'target/copy.ffstate','kind':'snapshot','sha256':'a'*64,'archive':'state.ffstate'},
              {'source':'target/report.json','kind':'support','sha256':'b'*64,'archive':'report.json'}]
        page=inv.index_html(rows,{},[],{'mode':'representative','omitted':[{}]}).decode()
        script=re.search(r'<script>(.*)</script>',page,re.S).group(1)
        stub="""const nodes=new Map();function el(){return {value:'',children:[],append(...xs){this.children.push(...xs)},replaceChildren(){this.children=[]},addEventListener(){}}};global.document={querySelector(k){if(!nodes.has(k))nodes.set(k,el());return nodes.get(k)},createElement:el};document.querySelector('#kind').value='saves';\n"""
        checks="""\nif(document.querySelector('#rows').children.length!==1)throw Error('save aliases not collapsed');if(!document.querySelector('#count').textContent.includes('2 source paths'))throw Error('aliases missing');search.value='copy';render();if(document.querySelector('#rows').children.length!==1)throw Error('alias search missing');kind.value='all';search.value='';render();if(document.querySelector('#rows').children.length!==3)throw Error('all source paths inaccessible');"""
        child=subprocess.run(['node','-e',stub+script+checks],capture_output=True,text=True)
        self.assertEqual(child.returncode,0,child.stderr)


if __name__=='__main__':unittest.main()
