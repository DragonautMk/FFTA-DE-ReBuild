"""ROM-free audit ingestion checks, using independent sets of physical bytes."""
import contextlib
import copy
import io
import json
from pathlib import Path
import tempfile
from types import SimpleNamespace
import unittest
from unittest import mock

import rom_coverage as coverage
import rom_coverage_audit as audit

ROM = 'a' * 64
MANIFEST = 'b' * 64


def intervals(points):
    result = []
    for p in sorted(points):
        if result and result[-1][1] == p:
            result[-1][1] += 1
        else:
            result.append([p, p + 1])
    return result


def points(spans):
    return {p for a,b in spans for p in range(a,b)}


def report():
    return dict(schema=2, rom_sha256=ROM, manifest_sha256=MANIFEST,
                generated_at='2026-10-28T12:00:00+00:00', generation='e1-d1', source_revision='fixture',
                reviewed_generation='e1-d1', evidence_date='2026-10-28', pending_review=[], historical_dataset=False,
                totals=dict(rom=64, identified_data=16, runtime_available=16, runtime_families=1,
                            code_candidates=4, ordinary_resource_extents=16),
                layers=dict(identified_data=[[8,24]], runtime_available=[[8,24]],
                            code_candidates=[[0,4]], ordinary_resource_extents=[[8,24]]),
                resources=[dict(id='one', runtime_available=True, ordinary_positive=True, ranges=[[8,24]])])


def fixture(case='DS-20-EUR', run='one', shift=0):
    reads = [('Bus','Raw',8+shift,4,2), ('AudioValue','Typed',10+shift,4,500),
             ('SpritePeek','Raw',28+shift,2,1)]
    events = []
    for reader, service, start, width, requests in reads:
        events.append(dict(key=dict(read=dict(reader=reader, guest_address=0x0a000000+start,
            physical_offset=start, length=width), service=service, instruction=None),
            first_guest_frame=100, last_guest_frame=101, requests=requests))
    def byte_set(test):
        return {p for e in events if test(e) for p in range(e['key']['read']['physical_offset'],
                 e['key']['read']['physical_offset']+e['key']['read']['length'])}
    def inv(test):
        b = byte_set(test)
        return dict(distinct_bytes=len(b), physical_intervals=intervals(b))
    all_bytes = byte_set(lambda e: True)
    bus = byte_set(lambda e:e['key']['read']['reader']=='Bus')
    host = byte_set(lambda e:e['key']['read']['reader']!='Bus')
    return dict(schema=1, kind='ffta-rom-read-audit', rom_sha256=ROM, runtime_sha256='c'*64,
                dataset_manifest_sha256=None, input_sha256='d'*64, case=case, run=run,
                window=dict(relative_from=0, relative_to_exclusive=2, frames_entered=2,
                            replay_completed=2, complete=True, replay_outcome='passed'),
                filter=dict(preset='all', physical_ranges=[], unique_event_limit=50000),
                coverage=dict(readers=list(audit.READERS), fetches='Fetch excluded',
                              instruction_scope='Scope may be inherited', not_observed=['Other threads'],
                              absence='Absence is not negative evidence'),
                events=events, retained_events=len(events), physical_intervals=intervals(all_bytes),
                observed_distinct_bytes=len(all_bytes), bus=inv(lambda e:e['key']['read']['reader']=='Bus'),
                host=inv(lambda e:e['key']['read']['reader']!='Bus'), bus_host_shared_distinct_bytes=len(bus&host),
                serving={s.lower():inv(lambda e:e['key']['service']==s) for s in audit.SERVICES},
                matched_requests=503, dropped_requests=0, fetch_bus_requests_excluded=1000, truncated=False,
                known_plan=dict(compared=False, manifest_sha256=None))


class AuditImportTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(); self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)

    def save(self, data, name='audit.json'):
        p = self.root / name; p.write_text(json.dumps(data)); return p

    def test_repeated_requests_mirrors_and_duplicate_inputs_never_multiply_bytes(self):
        p = self.save(fixture()); r=report()
        audit.attach_audits(r,[p,p])
        self.assertEqual(r['totals']['scene_observed'],8)
        self.assertEqual(r['totals']['scene_audits'],1)
        self.assertEqual(r['totals']['scene_typed'],4)
        self.assertEqual(r['totals']['scene_raw'],6)  # 2B Typed/Raw overlap.
        self.assertEqual(r['totals']['scene_outside_identified'],2)
        self.assertEqual(r['resources'][0]['scene_ranges'],[[8,14]])
        self.assertEqual(r['layers']['ordinary_resource_extents'],[[8,24]])
        self.assertIsNone(r['totals']['ordinary_distinct_read_bytes'])
        self.assertEqual(r['audits'][0]['matched_requests'],503)
        self.assertNotIn('events',r['audits'][0])
        self.assertNotIn(str(self.root),json.dumps(r))

    def test_multiple_runs_of_one_case_are_not_cross_case_sharing(self):
        first=self.save(fixture(), 'one.json'); second=self.save(fixture(run='two',shift=2),'two.json')
        r=report(); audit.attach_audits(r,[first,second])
        self.assertEqual(r['totals']['scene_shared'],0)
        self.assertEqual(points(r['layers']['scene_observed']),points([[8,16],[28,32]]))
        other=fixture(case='DS-21-EUR',run='three',shift=2)
        audit.attach_audits(r,[first,self.save(other,'three.json')])
        self.assertEqual(points(r['layers']['scene_shared']),points([[10,14]]))

    def test_truncated_partial_and_divergent_reports_remain_lower_bound_evidence(self):
        d=fixture();d['filter']['unique_event_limit']=3;d['truncated']=True
        d['dropped_requests']=900;d['matched_requests']+=900
        d['window'].update(relative_to_exclusive=10, complete=False, replay_outcome='cancelled')
        r=report();audit.attach_audits(r,[self.save(d)])
        self.assertTrue(r['audits'][0]['truncated']);self.assertTrue(r['audits'][0]['partial'])
        self.assertEqual(r['totals']['scene_observed'],8)
        self.assertTrue(r['resources'][0]['ordinary_positive'])  # Unrelated reviewed proof unchanged.
        r['resources'][0]['ordinary_positive']=False
        d['window'].update(relative_to_exclusive=2, complete=True, replay_outcome='diverged')
        audit.attach_audits(r,[self.save(d)])
        self.assertTrue(r['audits'][0]['partial']);self.assertFalse(r['resources'][0]['ordinary_positive'])

    def test_width_crossing_filter_includes_whole_actual_read(self):
        d=fixture();d['filter']['physical_ranges']=[[11,29]]
        r=report();audit.attach_audits(r,[self.save(d)])
        self.assertEqual(r['layers']['scene_observed'],[[8,14],[28,30]])

    def test_known_plan_requires_exact_current_intersections_or_is_only_provenance(self):
        d=fixture();d['known_plan']=dict(compared=True,manifest_sha256=MANIFEST,
            within_distinct_bytes=6,outside_distinct_bytes=2,resources=[dict(id='one',
            observed_distinct_bytes=6,physical_intervals=[[8,14]])])
        r=report();audit.attach_audits(r,[self.save(d)])
        self.assertTrue(r['audits'][0]['known_matches_atlas'])
        d['known_plan']['resources'][0]['physical_intervals']=[[8,15]]
        with self.assertRaises(ValueError):audit.attach_audits(report(),[self.save(d)])
        d['known_plan']['manifest_sha256']='e'*64
        r=report();audit.attach_audits(r,[self.save(d)])
        self.assertFalse(r['audits'][0]['known_matches_atlas'])
        self.assertEqual(r['resources'][0]['scene_ranges'],[[8,14]])

    def test_wrong_rom_and_malformed_or_inflated_claims_are_rejected(self):
        edits=[lambda d:d.update(rom_sha256='e'*64),lambda d:d.update(schema=True),
          lambda d:d.update(observed_distinct_bytes=999),lambda d:d.update(matched_requests=999),
          lambda d:d.update(truncated=True),lambda d:d.update(retained_events=1),
          lambda d:d['events'].append(copy.deepcopy(d['events'][0])),
          lambda d:d['events'][0]['key']['read'].update(physical_offset=9),
          lambda d:d['events'][0]['key']['read'].update(length=100),
          lambda d:d['events'][0]['key'].update(service='Invented'),
          lambda d:d['events'][0]['key']['read'].update(reader='NativeCodeFetch'),
          lambda d:d['coverage'].update(readers=['Bus']),lambda d:d['filter'].update(preset='host'),
          lambda d:d['window'].update(frames_entered=3),
          lambda d:d['events'][0].update(last_guest_frame=102),
          lambda d:d.update(physical_intervals=[[8,14],[8,14],[28,30]]),
          lambda d:d['serving']['typed'].update(distinct_bytes=99)]
        for i,edit in enumerate(edits):
            with self.subTest(i=i):
                d=fixture();edit(d)
                with self.assertRaises(ValueError):audit.attach_audits(report(),[self.save(d)])

    def test_conflicting_same_run_and_bounded_input_limits(self):
        a=self.save(fixture(),'one.json');b=self.save(fixture(shift=2),'two.json')
        with self.assertRaisesRegex(ValueError,'conflicting'):audit.attach_audits(report(),[a,b])
        with self.assertRaisesRegex(ValueError,'at most 32'):audit.attach_audits(report(),[a]*33)
        with mock.patch.object(audit,'MAX_FILE',10):
            with self.assertRaisesRegex(ValueError,'32 MiB'):audit.attach_audits(report(),[a])
        with self.assertRaises(ValueError):coverage.checked_span(True,4,64)

    def test_empty_selection_does_not_imply_no_reads(self):
        r=report();audit.attach_audits(r,[])
        self.assertEqual(r['totals']['scene_audits'],0)
        self.assertEqual(r['layers']['scene_observed'],[])
        self.assertIsNone(r['totals']['ordinary_distinct_read_bytes'])

    def test_no_entered_frames_is_partial_zero_inventory_not_complete_coverage(self):
        d=fixture();d['events']=[];d['retained_events']=0;d['matched_requests']=0
        d['physical_intervals']=[];d['observed_distinct_bytes']=0;d['bus_host_shared_distinct_bytes']=0
        d['bus']=dict(physical_intervals=[],distinct_bytes=0);d['host']=copy.deepcopy(d['bus'])
        d['serving']={s.lower():copy.deepcopy(d['bus']) for s in audit.SERVICES}
        d['window'].update(frames_entered=0,replay_completed=0,complete=False,replay_outcome='invalid')
        r=report();audit.attach_audits(r,[self.save(d)])
        self.assertTrue(r['audits'][0]['partial'])
        self.assertEqual(r['totals']['scene_observed'],0)
        self.assertIsNone(r['totals']['ordinary_distinct_read_bytes'])

    def test_unknown_payload_fields_cannot_enter_metadata_output(self):
        d=fixture()
        for section in ('window','filter','coverage'):
            d[section]['unknown_payload']='DO_NOT_PUBLISH_PRIVATE_DATA'
        d['unknown_payload']='DO_NOT_PUBLISH_PRIVATE_DATA'
        r=report();audit.attach_audits(r,[self.save(d)])
        self.assertNotIn('DO_NOT_PUBLISH_PRIVATE_DATA',json.dumps(r))
        self.assertNotIn('unknown_payload',json.dumps(r))

    def test_comparison_shows_churn_even_when_total_is_equal(self):
        old=report();old['generation']='e0-d0';new=report();new['layers']['identified_data']=[[10,26]]
        new['totals']['identified_data']=16
        c=audit.compare_reports(new,old,'e'*64)['layers']['identified_data']
        self.assertEqual((c['before'],c['after'],c['delta']),(16,16,0))
        self.assertEqual(c['added'],[[24,26]]);self.assertEqual(c['removed'],[[8,10]])
        old['rom_sha256']='f'*64
        with self.assertRaisesRegex(ValueError,'ROM identity'):audit.compare_reports(new,old,'e'*64)

    def test_generation_failure_preserves_both_existing_outputs(self):
        old=report(); old['rom_sha256']='e'*64
        old_path=self.save(old,'previous.json')
        out=self.root/'out';out.mkdir()
        for name in ('index.html','coverage.json'):(out/name).write_text('preserved')
        args=SimpleNamespace(output=out,compare=old_path)
        with mock.patch.object(coverage,'prepare_dataset'),mock.patch.object(coverage,'build',return_value=report()):
            with self.assertRaisesRegex(ValueError,'invalid --compare'):coverage.generate(args)
        self.assertTrue(all((out/name).read_text()=='preserved' for name in ('index.html','coverage.json')))

    def test_generation_comparison_retains_identity_and_ignores_own_comparison_in_semantics(self):
        old=self.save(report(),'previous.json'); r=report()
        audit.attach_audits(r,[self.save(fixture())])
        args=SimpleNamespace(output=self.root/'out',compare=old)
        with mock.patch.object(coverage,'prepare_dataset'),mock.patch.object(coverage,'build',return_value=r), \
             contextlib.redirect_stdout(io.StringIO()):coverage.generate(args)
        emitted=json.loads((args.output/'coverage.json').read_text())
        self.assertFalse(emitted['comparison']['scene_selection_same'])
        self.assertEqual(emitted['comparison']['manifest_sha256'],MANIFEST)
        self.assertEqual(emitted['comparison']['generated_at'],report()['generated_at'])
        self.assertNotIn('comparison',coverage.semantic_report(emitted))
        self.assertEqual(emitted['schema'],3)


if __name__=='__main__':unittest.main()
