"""Public synthetic lifecycle/coordination checks; no ROM or personal data."""
import contextlib
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time
import unittest
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import artifact_coordinator as coordinator
import debug_artifacts as manager
import local_artifacts as old
import rom_coverage as coverage


class DebugArtifactsTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory(prefix="ffta artifacts ' à ")
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)
        subprocess.run(['git', 'init', '-q', str(self.root)], check=True)
        self.patch = mock.patch.object(manager, 'ROOT', self.root)
        self.patch.start(); self.addCleanup(self.patch.stop)

    def session(self, name, *, ok=True, protocol=True):
        root = self.root / name; root.mkdir()
        for rel, data in [('ffta.gba', b'synthetic cartridge placeholder'),
                          ('portable.txt', b'synthetic portable'), ('recomp.exe', b'synthetic executable'),
                          ('Data/saves/keep.sav', b'private fixture'),
                          ('seed.ffstate', b'synthetic state'),
                          ('Data/config/recordings/run/recording.json', b'{"proof":true}'),
                          ('Data/config/code/run/index.html', b'synthetic report')]:
            p = root / rel; p.parent.mkdir(parents=True, exist_ok=True); p.write_bytes(data)
        manifest = dict(schema=1, kind='ffta-debug-session', pins={
            n: hashlib.sha256((root/n).read_bytes()).hexdigest()
            for n in ['ffta.gba', 'portable.txt', 'recomp.exe', 'seed.ffstate']})
        if protocol: manifest['artifact_coordination'] = coordinator.PROTOCOL
        (root/'debug-session.json').write_text(json.dumps(manifest))
        (root/'last-result.json').write_text(json.dumps(dict(schema=1, kind='ffta-debug-session-result', ok=ok)))
        cache = root/'Data/cache/t8'/('a'*64+'-tccmem1-ffta-spr3-g12345678.dll')
        cache.parent.mkdir(parents=True); cache.write_bytes(b'synthetic generated cache')
        (cache.parent/'unknown.bin').write_bytes(b'preserve unknown')
        cache.with_suffix('.c').write_bytes(b'synthetic source preimage; preserve')
        return root, cache

    def unit(self):
        root = self.root/'target/rom-data-completed'; root.mkdir(parents=True)
        (root/'managed-result.json').write_text('{"passed":true}')
        for name in ['interrupted', 'moved resources', 'offline dataset']:
            p = root/'windows-managed-europe'/name;p.mkdir(parents=True)
            (p/'resource.json').write_text('{"synthetic":true}')
        (root/'original.ffstate').write_bytes(b'private fixture')
        return root

    def request(self, *sessions, units=(), keep=()):
        return dict(sessions=sorted(map(str,sessions)), units=sorted(map(str,units)), keep=sorted(map(str,keep)))

    def preview(self, request, name='preview'):
        path=self.root/name; result=manager.preview(request,path); return path/'plan.json',result

    def test_preview_apply_keep_selection_and_proofs_with_spaces(self):
        done, cache=self.session('done');keep,kept=self.session('keep');failed,_=self.session('error',ok=False);legacy,_=self.session('legacy',protocol=False);unit=self.unit()
        p,plan=self.preview(self.request(done,keep,failed,legacy,units=[unit],keep=[keep]))
        self.assertEqual(plan['candidate_count'],3)
        self.assertEqual([s['status'] for s in plan['scopes'][:4]],['completed','protected','kept','protected'])
        protected={p:p.read_bytes() for root in [done,keep,failed,legacy,unit] for p in root.rglob('*') if p.is_file() and not any(p.is_relative_to(c['path']) for s in plan['scopes'] for c in s['candidates'])}
        self.assertTrue(cache.exists());receipt=manager.apply(p,self.root/'receipt.json')
        self.assertEqual(receipt['status'],'complete');self.assertEqual(len(receipt['removed']),3);self.assertFalse(cache.exists());self.assertTrue(kept.exists())
        self.assertTrue(all(path.read_bytes()==data for path,data in protected.items()))
        self.assertEqual(manager.build(self.request(done,units=[unit]))['candidate_count'],0)
        with self.assertRaises(FileExistsError):manager.apply(p,self.root/'receipt.json')

    def test_stale_or_modified_protected_inventory_refuses_every_removal(self):
        done,cache=self.session('done');p,_=self.preview(self.request(done))
        (done/'Data/saves/keep.sav').write_bytes(b'new protected state')
        with self.assertRaisesRegex(ValueError,'changed since preview'):manager.apply(p,self.root/'changed-receipt.json')
        receipt=json.loads((self.root/'changed-receipt.json').read_text());self.assertEqual(receipt['removed'],[]);self.assertTrue(cache.exists());self.assertEqual(receipt['status'],'failed')

    def test_active_lease_acquired_after_preview_blocks_before_first_unlink(self):
        one,a=self.session('one');two,b=self.session('two');p,_=self.preview(self.request(one,two))
        with coordinator.Lease(two,'synthetic-runtime'):
            with self.assertRaisesRegex(ValueError,'lease|changed since preview'):manager.apply(p,self.root/'busy.json')
            self.assertTrue(a.exists() and b.exists())
        self.assertFalse((one/coordinator.NAME).exists())
        self.assertEqual(json.loads((self.root/'busy.json').read_text())['removed'],[])

    def test_controlled_partial_error_has_receipt_and_preserves_diagnostics(self):
        done,cache=self.session('done');unit=self.unit();p,_=self.preview(self.request(done,units=[unit]))
        with mock.patch.object(old,'remove_candidate',side_effect=PermissionError('controlled removal refusal')):
            with self.assertRaisesRegex(PermissionError,'controlled'):manager.apply(p,self.root/'failed.json')
        receipt=json.loads((self.root/'failed.json').read_text());self.assertEqual(receipt['status'],'failed');self.assertEqual(receipt['removed'],[str(cache)])
        self.assertTrue((unit/'windows-managed-europe/interrupted/resource.json').exists());self.assertTrue((done/'seed.ffstate').exists())
        self.assertFalse((done/coordinator.NAME).exists())

    def test_killed_worker_leaves_protection_without_guessing_pid_liveness(self):
        done,cache=self.session('interrupted');ready=self.root/'ready'
        script="from artifact_coordinator import Lease;from pathlib import Path;import sys,time\nwith Lease(sys.argv[1],'synthetic-kill'):\n Path(sys.argv[2]).write_text('ready');time.sleep(60)"
        env=dict(os.environ,PYTHONPATH=str(Path(manager.__file__).parent));p=subprocess.Popen([sys.executable,'-c',script,str(done),str(ready)],env=env)
        try:
            until=time.monotonic()+10
            while not ready.exists():
                self.assertIsNone(p.poll());self.assertLess(time.monotonic(),until);time.sleep(.02)
            p.kill();p.wait(timeout=10)
            row=manager.build(self.request(done))['scopes'][0];self.assertEqual(row['status'],'protected');self.assertIn('interrupted',row['reason']);self.assertTrue(cache.exists())
        finally:
            if p.poll() is None:p.kill();p.wait(timeout=10)

    def test_incomplete_creation_new_session_and_cleanup_are_independent(self):
        done,cache=self.session('completed');p,_=self.preview(self.request(done));new=self.root/'new-session';new.mkdir()
        with coordinator.Lease(new,'session-create'):
            (new/'ffta.gba').write_bytes(b'synthetic incomplete copy')
            row=manager.build(self.request(new))['scopes'][0];self.assertEqual(row['status'],'protected')
            manager.apply(p,self.root/'old-cache-receipt.json')
            self.assertEqual((new/'ffta.gba').read_bytes(),b'synthetic incomplete copy');self.assertFalse(cache.exists())

    def test_tracked_unknown_shared_and_symlink_files_are_protected(self):
        done,cache=self.session('tracked');subprocess.run(['git','-C',str(self.root),'add',str(cache)],check=True)
        self.assertEqual(manager.build(self.request(done))['scopes'][0]['status'],'protected')
        (done/'debug-session.json').write_text('[]');self.assertEqual(manager.build(self.request(done))['scopes'][0]['status'],'protected')
        if os.name=='posix':
            other,cache=self.session('linked');cache.unlink();cache.symlink_to(done/'seed.ffstate')
            self.assertEqual(manager.build(self.request(other))['scopes'][0]['status'],'protected')

    def test_scope_output_tampering_and_plan_limits(self):
        done,_=self.session('done');request=self.request(done)
        with self.assertRaisesRegex(ValueError,'outside'):manager.preview(request,done/'preview')
        bad=dict(request,keep=[str(self.root/'typo')])
        with self.assertRaisesRegex(ValueError,'--keep'):manager.build(bad)
        with mock.patch.object(manager,'MAX_FILES',1):self.assertEqual(manager.build(request)['scopes'][0]['status'],'protected')
        with mock.patch.object(manager,'MAX_JSON',128):
            with self.assertRaisesRegex(ValueError,'metadata'):manager.build(request)

        p,plan=self.preview(request);plan['scopes'][0]['candidates'][0]['path']=str(done/'seed.ffstate');p.write_text(json.dumps(plan))
        with self.assertRaisesRegex(ValueError,'changed since preview'):manager.apply(p,self.root/'tampered.json')
        self.assertEqual((done/'seed.ffstate').read_bytes(),b'synthetic state')

    def test_nested_activity_protects_entire_rom_unit_and_explicit_cache_reads(self):
        unit=self.unit();active=unit/'windows-managed-europe/interrupted'
        with coordinator.Lease(active,'nested-worker'):
            row=manager.build(self.request(units=[unit]))['scopes'][0]
            self.assertEqual(row['status'],'protected');self.assertEqual(row['candidates'],[])
            self.assertEqual(old.completion_plan([unit],self.root)[0],[])
        cache=self.root/'explicit cache';dataset=cache/('a'*64)/'e8-d7';dataset.mkdir(parents=True)
        (dataset/'manifest.toml').write_text('synthetic readonly input')
        self.assertEqual(coverage.explicit_cache_root(str(dataset)),str(cache))
        from types import SimpleNamespace
        for selected,lease_root in [(str(dataset),None),('namespace-hidden-readonly-input',str(cache))]:
            args=SimpleNamespace(dataset=selected,dataset_cache=None,cache_lease=lease_root)
            def preparation(_):
                with self.assertRaisesRegex(ValueError,'lease'):
                    old.prune_coverage_cache(cache,dataset,1)
                raise ValueError('explicit reader held through operation')
            with mock.patch.object(coverage,'prepare_dataset',side_effect=preparation):
                with self.assertRaisesRegex(ValueError,'explicit reader'):coverage.generate(args)
            self.assertFalse((cache/coordinator.NAME).exists())
        self.assertEqual((dataset/'manifest.toml').read_text(),'synthetic readonly input')

    def test_total_candidate_leaf_files_are_bounded_across_rom_copy_groups(self):
        unit=self.unit()
        for name in ['interrupted','moved resources']:
            (unit/'windows-managed-europe'/name/'second.json').write_text('synthetic')
        with mock.patch.object(manager,'MAX_FILES',3):
            with self.assertRaisesRegex(ValueError,'candidate files'):
                manager.build(self.request(units=[unit]))

    def test_cache_generation_and_pruning_hold_same_lease_and_release_on_error(self):
        cache=self.root/'coverage cache';cache.mkdir();rom=cache/('a'*64);rom.mkdir()
        for rev in [8,9,10]:
            p=rom/f'e{rev}-d{rev-1}';p.mkdir();(p/'resource.json').write_text('synthetic');(p/'manifest.toml').write_text(f'source_rom_sha256="{"a"*64}"\nextractor_revision={rev}\ndecoder_revision={rev-1}\n')
        with coordinator.Lease(cache,'coverage-reader') as held:
            with self.assertRaisesRegex(ValueError,'lease'):old.prune_coverage_cache(cache,rom/'e10-d9',2)
            removed=old.prune_coverage_cache(cache,rom/'e10-d9',2,lease=held);self.assertEqual(removed,[str(rom/'e8-d7')]);self.assertTrue((rom/'e10-d9').exists())
            receipt=json.loads(next((cache/'.retention').glob('*.json')).read_text());self.assertEqual(receipt['status'],'complete');self.assertEqual(receipt['removed'],removed)
        from types import SimpleNamespace
        args=SimpleNamespace(dataset=None,dataset_cache=str(cache))
        def preparation(_):
            with self.assertRaisesRegex(ValueError,'lease'):
                with coordinator.Lease(cache,'other-writer'):pass
            raise ValueError('controlled prepare failure')
        with mock.patch.object(coverage,'prepare_dataset',side_effect=preparation):
            with self.assertRaisesRegex(ValueError,'controlled'):coverage.generate(args)
        self.assertFalse((cache/coordinator.NAME).exists());self.assertTrue((rom/'e10-d9/manifest.toml').exists())
        receipts=cache/'.retention'
        for i in range(64-len(list(receipts.iterdir()))):(receipts/f'preserved-{i}.json').write_text('synthetic receipt')
        with self.assertRaisesRegex(ValueError,'64 cache-retention'):
            old.prune_coverage_cache(cache,rom/'e10-d9',1)
        self.assertTrue((rom/'e9-d8/resource.json').exists())
        self.assertEqual(len(list(receipts.iterdir())),64)



if __name__=='__main__':unittest.main()
