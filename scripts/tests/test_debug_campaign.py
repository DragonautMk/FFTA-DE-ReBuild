import contextlib
import copy
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import debug_campaign as c
from artifact_coordinator import Lease

H = 'a' * 64


class Campaign(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory(prefix='ffta-campaign-')
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)

    def write(self, path, data):
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(json.dumps(data))
        return {'path': str(path.relative_to(self.root)), 'sha256': hashlib.sha256(path.read_bytes()).hexdigest()}

    def fixture(self, name='case', outcome='passed', audit=None):
        folder = self.root / name
        p = dict(schema=1, kind='ffta-debug-replay-profile', case='DS-20-EUR', frames=2, final_clock=100,
                 runtime_sha256=H, rom_sha256=H, input_sha256=H, end_sha256=H,
                 start_sha256=H, bios_sha256=H, dataset_sha256=None, library_sha256=None)
        r = dict(schema=1, kind='ffta-debug-replay-result', run='SYNTHETIC-'+name, case=p['case'], outcome=outcome,
                 frames=2, completed=2, message='PUBLIC SYNTHETIC', state_sha256=H, clock=100,
                 **{key: p[key] for key in c.IDENTITIES})
        if audit is not None:
            p['audit'] = dict(preset='bus', ranges=[[0, 4]], limit=16, **{'from': 0, 'to': 2})
            r.update(audit_truncated=False, audit_window_complete=True)
        case = {'id': name, 'ds': ['DS-20'], 'families': ['task'], 'transitions': ['start'],
                'profile': self.write(folder/'replay.json', p), 'evidence': {'result': self.write(folder/'result.json', r)}}
        if audit is not None:
            a = dict(schema=1, kind='ffta-rom-read-audit', run=r['run'], case=p['case'],
                     runtime_sha256=H, rom_sha256=H, input_sha256=H, dataset_manifest_sha256=None,
                     matched_requests=audit, dropped_requests=0, truncated=False,
                     window=dict(relative_from=0, relative_to_exclusive=2, complete=True, replay_completed=2, replay_outcome=outcome),
                     filter=dict(preset='bus', physical_ranges=[[0, 4]], unique_event_limit=16))
            case['evidence']['audit'] = self.write(folder/'audit.json', a)
        return case

    def manifest(self, cases):
        path = self.root/'manifest.json'
        self.write(path, dict(schema=1, kind='ffta-debug-campaign', label="SYNTHETIC ' à <b>", cases=cases))
        return path

    def edit_pin(self, ref, modify):
        path = self.root/ref['path'];data = json.loads(path.read_text());modify(data)
        ref.update(self.write(path, data))

    def test_strict_json_profile_shapes_and_undeclared_prior_results(self):
        case=self.fixture('strict')
        for data in ['{"schema":1,"schema":1}', '{"schema":NaN}']:
            (self.root/'invalid.json').write_text(data)
            with self.assertRaises(ValueError):c.value(self.root/'invalid.json',1024)
        for key,value in [('frames',True),('capture',[]),('runtime_sha256',11)]:
            bad=self.fixture('shape-'+key);self.edit_pin(bad['profile'],lambda p:p.update({key:value}))
            self.assertEqual(c.inspect(self.root,bad,True)['outcome'],'incompatible')
        case.pop('evidence');manifest=self.manifest([case])
        self.write(self.root/'strict/Data/config/replay/result.json', {'outcome':'passed'})
        args=c.parser().parse_args(['run-case',str(manifest),'strict','--reason','Missing declaration is not authorization','--out',str(self.root/'intents/no')])
        with self.assertRaisesRegex(ValueError,'reopen-reason'):c.run_case(args,lambda *a:None)
        self.assertFalse(args.out.exists())

    def test_selection_is_explicit_and_filters_do_not_reopen_others(self):
        a, b = self.fixture('a'), self.fixture('b')
        b['families']=['audio'];b['ds']=['DS-03'];b['transitions']=['end']
        with self.assertRaises(ValueError):c.selection([a,b])
        self.assertEqual(c.selection([a,b], families=['audio'], ds=['DS-03']), {'b'})
        self.assertEqual(c.selection([a,b], ids=['a','b'], transitions=['start']), {'a'})
        with self.assertRaises(ValueError):c.selection([a,b], ids=['unknown'])
        self.assertEqual(c.selection([a,b], families=['none']), set())
        with mock.patch.object(c, 'pinned', side_effect=AssertionError('Unselected evidence opened')):
            self.assertEqual(c.inspect(self.root, a, False)['outcome'], 'not-executed')

    def test_all_four_outcomes_are_recorded_without_dispatch_or_status_promotion(self):
        cases=[self.fixture('positive'),self.fixture('negative','diverged'),self.fixture('bad'),{'id':'missing','ds':['DS-03']}]
        cases[2]['profile']['sha256']='0'*64
        manifest=self.manifest(cases)
        args=c.parser().parse_args(['report',str(manifest),'--ds','DS-20','--ds','DS-03','--out',str(self.root/'reports/one')])
        with mock.patch.object(os, 'execv', side_effect=AssertionError('Automatic replay')):
            report=c.report(args)
        self.assertEqual([r['outcome'] for r in report['cases']],['positive','negative','incompatible','not-executed'])
        self.assertEqual(report['cases'][0]['evidence'],'reused-pinned')
        self.assertIn('UNKNOWN',report['scope'])
        html=(args.out/'index.html').read_text()
        self.assertIn('\\u003cb>',html)
        self.assertNotIn("SYNTHETIC ' à <b>",html)
        self.assertTrue((args.out/'.gitignore').exists())

    def test_changed_result_identity_and_endpoint_do_not_pass(self):
        for key,new in [('runtime_sha256','b'*64),('case','other'),('clock',101),('state_sha256','b'*64),('completed',1),('outcome','running')]:
            case=self.fixture('identity-'+key)
            self.edit_pin(case['evidence']['result'],lambda r:r.update({key:new}))
            self.assertEqual(c.inspect(self.root,case,True)['outcome'],'incompatible',key)
        case=self.fixture('changed');(self.root/case['evidence']['result']['path']).write_text('{}')
        self.assertEqual(c.inspect(self.root,case,True)['outcome'],'incompatible')

    def test_zero_is_only_in_complete_selected_window_partial_and_unavailable_explicit(self):
        case=self.fixture('zero',audit=0);row=c.inspect(self.root,case,True)
        self.assertEqual(row['reads']['state'],'zero-in-selected-window')
        self.assertIn('UNKNOWN',row['reads']['meaning'])
        case=self.fixture('partial',audit=0)
        self.edit_pin(case['evidence']['audit'],lambda a:a.update(truncated=True))
        self.edit_pin(case['evidence']['result'],lambda r:r.update(audit_truncated=True))
        self.assertEqual(c.inspect(self.root,case,True)['reads']['state'],'partial-zero')
        case=self.fixture('observed',audit=4)
        self.assertEqual(c.inspect(self.root,case,True)['reads']['matched_requests'],4)
        case=self.fixture('no-audit')
        self.assertEqual(c.inspect(self.root,case,True)['reads']['state'],'unavailable')

    def test_mismatched_audit_run_window_and_counts_are_incompatible(self):
        for key,new in [('run','other'),('dropped_requests',8),('window',[])]:
            case=self.fixture('audit-'+key,audit=4)
            self.edit_pin(case['evidence']['audit'],lambda a:a.update({key:new}))
            self.assertEqual(c.inspect(self.root,case,True)['outcome'],'incompatible',key)
        case=self.fixture('window',audit=4)
        self.edit_pin(case['evidence']['audit'],lambda a:a['window'].update(relative_to_exclusive=1))
        self.assertEqual(c.inspect(self.root,case,True)['outcome'],'incompatible')

    def test_manifest_and_output_limits_leave_prior_reports_untouched(self):
        case=self.fixture('bound')
        for cases in [[case]*2,[{'id':str(i)} for i in range(33)]]:
            with self.assertRaises(ValueError):c.load(self.manifest(cases))
        manifest=self.manifest([case])
        manifest.write_bytes(b' '*(c.MAX_MANIFEST+1))
        with self.assertRaises(ValueError):c.load(manifest)
        store=self.root/'outputs';store.mkdir()
        for i in range(8):(store/str(i)).mkdir()
        with self.assertRaises(ValueError):c.output_dir(store/'ninth')
        self.assertEqual(len(list(store.iterdir())),8)

    def test_attested_active_scope_is_protected_and_legacy_snapshot_remains_metadata_only(self):
        case=self.fixture('leased')
        session=self.root/'leased/debug-session.json';self.write(session,{'artifact_coordination':'directory-v1'})
        with Lease(session.parent,'SYNTHETIC owner'):
            row=c.inspect(self.root,case,True);self.assertEqual(row['outcome'],'incompatible')
            self.assertIn('lease',row['reason'])
        row=c.inspect(self.root,case,True)
        self.assertEqual(row['outcome'],'positive');self.assertEqual(row['coordination'],'directory-v1')
        session.unlink()
        self.assertIn('unknown',c.inspect(self.root,case,True)['coordination'])

    @unittest.skipIf(os.name=='nt','POSIX symlink fixture')
    def test_symlinks_and_shared_metadata_refuse_without_following(self):
        case=self.fixture('links');profile=self.root/case['profile']['path']
        alias=profile.parent/'alias';alias.symlink_to(profile);case['profile']['path']=str(alias.relative_to(self.root))
        self.assertEqual(c.inspect(self.root,case,True)['outcome'],'incompatible')
        alias.unlink();os.link(profile,alias);case['profile']['path']=str(profile.relative_to(self.root))
        self.assertEqual(c.inspect(self.root,case,True)['outcome'],'incompatible')

    def test_moved_workspace_keeps_pins_and_relative_report_links(self):
        case=self.fixture('move');manifest=self.manifest([case])
        moved=self.root/"moved ' à";moved.mkdir()
        shutil.move(str(self.root/'move'),moved/'move');shutil.move(str(manifest),moved/'manifest.json')
        args=c.parser().parse_args(['report',str(moved/'manifest.json'),'--case','move','--out',str(moved/'reports/new')])
        report=c.report(args)
        for key in ('profile','result'):self.assertTrue((args.out/report['cases'][0][key]).is_file())

    def test_dispatch_needs_exact_case_runtime_and_repeat_reason_and_records_intent(self):
        case=self.fixture('run');runtime=self.root/'run/recomp';runtime.write_bytes(b'PUBLIC SYNTHETIC runtime')
        self.edit_pin(case['profile'],lambda p:p.update(runtime_sha256=hashlib.sha256(runtime.read_bytes()).hexdigest()))
        manifest=self.manifest([case])
        args=c.parser().parse_args(['run-case',str(manifest),'run','--reason','Affected source changed','--out',str(self.root/'intents/one')])
        with self.assertRaisesRegex(ValueError,'reopen-reason'):c.run_case(args,lambda *a:None)
        args.reopen_reason='New field affected; old acceptance stays recorded'
        calls=[];intent=c.run_case(args,lambda exe,argv:calls.append((exe,argv)))
        self.assertEqual(len(calls),1);self.assertEqual(calls[0][1][1:3],['debug-replay','run'])
        self.assertEqual(intent['state'],'dispatch-intent, not a completed result')
        self.assertTrue((args.out/'intent.json').is_file())
        runtime.write_bytes(b'CHANGED')
        args.out=self.root/'intents/two'
        with self.assertRaisesRegex(ValueError,'Changed profile runtime'):c.run_case(args,lambda *a:None)
        self.assertFalse(args.out.exists())

    @unittest.skipIf(os.name=='nt','POSIX public executable fixture')
    def test_cli_dispatch_executes_only_public_fake_runtime_and_keeps_receipt(self):
        case=self.fixture('fake');case.pop('evidence');runtime=self.root/'fake/recomp'
        runtime.write_text('#!/usr/bin/env python3\nimport json,sys\nprint(json.dumps({"SYNTHETIC":True,"argv":sys.argv[1:]}))\n')
        runtime.chmod(0o700);self.edit_pin(case['profile'],lambda p:p.update(runtime_sha256=hashlib.sha256(runtime.read_bytes()).hexdigest()))
        manifest=self.manifest([case])
        out=self.root/'dispatch/one'
        p=subprocess.run([sys.executable,c.__file__,'run-case',str(manifest),'fake','--reason','PUBLIC runner fixture','--out',str(out)],capture_output=True,text=True,timeout=10)
        self.assertEqual(p.returncode,0,p.stderr)
        self.assertEqual(json.loads(p.stdout)['argv'][:2],['debug-replay','run'])
        self.assertTrue(json.loads(p.stdout)['SYNTHETIC'])
        self.assertTrue((out/'intent.json').is_file())

if __name__=='__main__':unittest.main()
