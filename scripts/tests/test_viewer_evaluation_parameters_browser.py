"""Check authenticated score/threshold views and exact diagnostic arithmetic."""
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
    dataset = work / 'dataset'; dataset.mkdir()
    source, bias = 0x6e79a2, 0x6e79c2
    divisors, weights = [1, 2, 3, 7, 128, 255, 9], [255, 6, 13, 255, 8, 2, 17]
    catalog = {'score_offset': source, 'threshold_address_bias': bias,
               'terms': [{'id': i, 'source_offset': source + i * 2, 'divisor': divisors[i], 'weight': weights[i]}
                         for i in range(7)],
               'bonuses': [{'id': i + 7, 'source_offset': source + 14 + i * 2, 'byte00': i + 9, 'weight': 17 + i}
                           for i in range(2)],
               'thresholds': [{'id': i + 1, 'source_offset': bias + 4 + i * 4, 'threshold': i * 50,
                               'bytes01_03': [i, i + 1, i + 2]} for i in range(5)]}
    content = json.dumps(catalog).encode(); digest = hashlib.sha256(content).hexdigest()
    (dataset / 'evaluation.json').write_bytes(content)
    (dataset / 'manifest.toml').write_text(
        f'profile="{profile}"\nextractor_revision=1\ndecoder_revision=1\n'
        '[[resources]]\nid="gameplay.evaluation-parameters"\npath="evaluation.json"\n'
        f'format="evaluation-parameters-v1-json"\nsha256="{digest}"\n')
    with patch.object(view_rom_data, 'EUROPE_EVALUATION_PARAMETERS_SHA256', digest if verified else 'wrong'):
        page = view_rom_data.generate(dataset, work / 'view', split=True)
    return page, catalog


def expected_score(catalog, values, predicates):
    contributions = []
    for index, (term, word) in enumerate(zip(catalog['terms'], values)):
        if not term['divisor']:
            return None
        if index == 0:
            quotient = (word // term['divisor']) & 255
        else:
            signed = word if word < 0x80000000 else word - 0x100000000
            quotient = (abs(signed) // term['divisor']) * (-1 if signed < 0 else 1)
        contributions.append((quotient * term['weight']) & 0xffffffff)
    active = [predicates[0] & 255 == 1, predicates[1] & 255 == 0]
    score = (sum(contributions) + sum(item['weight'] for item, enabled in zip(catalog['bonuses'], active) if enabled)) & 0xffffffff
    return {'score': score, 'contributions': contributions, 'active': active}


class EvaluationProjectionTest(unittest.TestCase):
    def test_projection_scope_identity_and_invalid_records(self):
        for options in ({}, {'verified': False}, {'profile': 'usa'}):
            with self.subTest(options=options), tempfile.TemporaryDirectory(prefix='viewer-evaluation-project-') as directory:
                page, catalog = fixture(Path(directory), **options); parts = page.parent / 'index.html.parts'
                home, world, audio = map(parse_page, (page, parts / 'world.html', parts / 'audio.html'))
                self.assertNotIn('evaluation_parameters', home); self.assertNotIn('evaluation_parameters', audio)
                if options.get('profile') == 'usa':
                    self.assertIsNone(world['evaluation_parameters']); continue
                projected = world['evaluation_parameters']
                for key in catalog: self.assertEqual(projected[key], catalog[key])
                self.assertEqual(projected['reader_verified'], options.get('verified', True))
                self.assertIn('evaluation-parameters', [s['id'] for p in home['wiki']['pages'] if p['id'] == 'world' for s in p['sections']])
                self.assertTrue(next(r for r in world['resources'] if r['id']=='gameplay.evaluation-parameters')['status'].startswith('Available'))
                for mutate in (lambda c:c.update(score_offset=True), lambda c:c['terms'].pop(),
                               lambda c:c['terms'][0].update(divisor=True), lambda c:c['terms'][1].update(weight=256),
                               lambda c:c['bonuses'][0].update(id=0), lambda c:c['thresholds'][0].update(source_offset=0),
                               lambda c:c['thresholds'][1]['bytes01_03'].append(1)):
                    altered = copy.deepcopy(catalog); mutate(altered)
                    with self.assertRaises(ValueError): view_rom_data.project_evaluation_parameters(altered)


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class EvaluationBrowserTest(unittest.TestCase):
    def test_calculator_independent_arithmetic_and_navigation(self):
        with tempfile.TemporaryDirectory(prefix='viewer-evaluation-browser-') as directory:
            work = Path(directory); page, catalog = fixture(work); browser = Chrome(work, page)
            try:
                browser.click('#rows [data-resource-id="gameplay.evaluation-parameters"] a')
                wait_for(browser, "data.wiki.current==='world' && !document.querySelector('#evaluation-parameters').hidden")
                browser.click('#rows [data-resource-id="gameplay.evaluation-parameters"] a[href="#evaluation-parameters"]')
                state = browser.evaluate('''(()=>{const q=s=>document.querySelector(s),input=e=>e.dispatchEvent(new Event('input',{bubbles:true}));
                  if(q('#evaluation-terms').rows.length!==7 || q('#evaluation-bonuses').rows.length!==2 ||
                     q('#evaluation-thresholds').rows.length!==5 || q('#evaluation-score-calculator').hidden ||
                     !q('#evaluation-parameters>details').open)throw Error('section or source');
                  q('#evaluation-value-0').value='0xFFFFFFFF';input(q('#evaluation-value-0'));
                  q('#evaluation-value-1').value='-2147483648';input(q('#evaluation-value-1'));
                  q('#evaluation-predicate-0').value='257';input(q('#evaluation-predicate-0'));
                  q('#evaluation-predicate-1').value='256';input(q('#evaluation-predicate-1'));
                  const score=q('#evaluation-score-result').textContent;
                  q('#evaluation-class').value='1';q('#evaluation-class').dispatchEvent(new Event('change'));
                  q('#evaluation-draw').value='-1';input(q('#evaluation-draw'));
                  const threshold=q('#evaluation-threshold-result').textContent;
                  q('#evaluation-value-0').value='4294967296';input(q('#evaluation-value-0'));
                  return {score,threshold,invalid:q('#evaluation-score-result').textContent};})()''')
                expected = expected_score(catalog, [0xffffffff, 0x80000000, 0, 0, 0, 0, 0], [257, 256])
                self.assertIn('Score: ' + str(expected['score']), state['score'])
                self.assertIn('7 applied, 8 applied', state['score']); self.assertIn('Signed remainder -1 is below 0', state['threshold'])
                self.assertIn('valid 32-bit', state['invalid'])
                rng = random.Random(53952)
                words = [0, 1, 255, 256, 32767, 32768, 65535, 65536, 0x7fffffff, 0x80000000, 0xffffffff]
                cases = [{'values': [rng.choice(words) if i % 3 == 0 else rng.getrandbits(32) for i in range(7)],
                          'flags': [rng.choice(words + [257]), rng.choice(words)]} for _ in range(800)]
                actual = browser.evaluate('''(()=>{const cases=CASES;return cases.map(c=>evaluationScore(data.evaluation_parameters,c.values,c.flags));})()'''.replace('CASES', json.dumps(cases)))
                self.assertEqual(actual, [expected_score(catalog, c['values'], c['flags']) for c in cases])
                threshold_words = words + [0xffff0001, 0xffff8001, 0xffffff80]
                observed = browser.evaluate('''(()=>{const words=WORDS;return words.flatMap(w=>[0,1,127,255].map(t=>evaluationThreshold(w,t)));})()'''.replace('WORDS', json.dumps(threshold_words)))
                expected_thresholds = []
                for word in threshold_words:
                    low = word & 65535; signed = low if low < 32768 else low - 65536
                    remainder = (abs(signed) % 256) * (-1 if signed < 0 else 1)
                    expected_thresholds.extend({'remainder': remainder, 'accepted': remainder < threshold} for threshold in (0,1,127,255))
                self.assertEqual(observed, expected_thresholds)
                self.assertIsNone(browser.evaluate("(()=>{const c=structuredClone(data.evaluation_parameters);c.terms[1].divisor=0;return evaluationScore(c,Array(7).fill(0),[0,0]);})()"))
                self.assertTrue(browser.evaluate("(()=>{const q=s=>document.querySelector(s);q('#evaluation-class').value='0';q('#evaluation-class').dispatchEvent(new Event('change'));return q('#evaluation-threshold-result').textContent.includes('skips');})()"))
            finally:
                browser.close()

    def test_unverified_source_hides_calculators(self):
        with tempfile.TemporaryDirectory(prefix='viewer-evaluation-unverified-') as directory:
            work = Path(directory); page, _ = fixture(work, verified=False)
            browser = Chrome(work, page.parent / 'index.html.parts/world.html')
            try:
                self.assertTrue(browser.evaluate('''(()=>{const q=s=>document.querySelector(s);return !q('#evaluation-parameters').hidden &&
                  !q('#evaluation-unverified').hidden && q('#evaluation-score-calculator').hidden && q('#evaluation-threshold-calculator').hidden &&
                  q('#evaluation-terms').rows.length===7 && q('#evaluation-thresholds').rows.length===5;})()'''))
            finally:
                browser.close()
