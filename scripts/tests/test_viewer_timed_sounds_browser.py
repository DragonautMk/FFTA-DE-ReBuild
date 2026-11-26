"""Check timed-sound source browsing and conditional audition links."""
import copy
import hashlib
import json
import shutil
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from view_rom_data import generate, project_timed_sound_sources
from viewer_browser_harness import Chrome


def fixture(work, *, include_sequences=True, invalid_songs=False, invalid_bank=False, include_bank=True):
    dataset = work / 'dataset'; dataset.mkdir()
    row = {'tick': 12, 'channel': 3, 'opaque03': 7, 'sound_words': [0x2005, 0x0fff]}
    bank = {'sources': [
        {'offset': 0x400, 'values': {'kind': 'literal', 'values': 0x08000100}},
        {'offset': 0x404, 'values': {'kind': 'directory', 'values': [0x08000100, 0x08000200]}},
        {'offset': 0x40c, 'values': {'kind': 'counts', 'values': [2, 3]}},
        {'offset': 0x410, 'values': {'kind': 'ram-words', 'values': [4, 5]}},
        {'offset': 0x420, 'values': {'kind': 'rows', 'values': [copy.deepcopy(row)]}},
    ]}
    if invalid_bank:
        bank['sources'][-1]['values']['values'][0]['channel'] = 256
    directory = {'songs': [] if invalid_songs else [{'id': 0, 'header_id': 0}, {'id': 5, 'header_id': 5}],
                 'headers': [{'id': 0, 'tone_bank_address': None}, {'id': 5, 'tone_bank_address': None}],
                 'red_spring_sounds': [{'root_offset': 0x80, 'root_pointer': 0x08000100,
                                        'source_offset': 0x100, 'rows': [copy.deepcopy(row)]}],
                 'timed_sounds': bank if include_bank else None}
    sequences = {'driver': {'clocks': [1] * 49},
                 'tracks': [{'id': 0, 'directory_header': 5, 'track_index': 0, 'entry': 0}],
                 'instructions': [
                     {'id': 0, 'source_offset': 0, 'explicit': True,
                      'code': {'kind': 'note', 'clock': 1, 'key': 60, 'velocity': 100, 'extension': None}},
                     {'id': 1, 'source_offset': 3, 'explicit': True, 'code': {'kind': 'end'}}]}
    resources = [('m4a-directory', 'audio-directory', directory)]
    if include_sequences:
        resources.append(('m4a-sequences', 'audio-sequences', sequences))
    manifest = 'profile="europe"\nextractor_revision=334\ndecoder_revision=333\n'
    for rid, name, value in resources:
        content = json.dumps(value).encode(); (dataset / (name + '.json')).write_bytes(content)
        manifest += (f'[[resources]]\nid="audio.{rid}"\npath="{name}.json"\n'
                     f'format="{name}-v{"2" if rid == "m4a-directory" else "1"}-json"\n'
                     f'sha256="{hashlib.sha256(content).hexdigest()}"\n')
    (dataset / 'manifest.toml').write_text(manifest)
    return dataset, directory


def payload(page):
    return json.JSONDecoder().raw_decode(page.read_text().split('const data = ', 1)[1])[0]


class TimedSoundProjectionTest(unittest.TestCase):
    def test_optional_groups_and_invalid_sources(self):
        with tempfile.TemporaryDirectory(prefix='viewer-timed-sound-projection-') as directory:
            work = Path(directory); dataset, original = fixture(work)
            index = generate(dataset, work / 'view', split=True)
            audio = payload(index.parent / 'index.html.parts/audio.html')
            groups = audio['timed_sound_sources']['groups']
            self.assertEqual([group['kind'] for group in groups], ['rows', 'literal', 'directory', 'counts', 'ram-words', 'rows'])
            self.assertEqual(groups[0]['values'][0]['sound_words'], [0x2005, 0x0fff])
            self.assertEqual(groups[1]['values'], [0x08000100])
            self.assertNotIn('timed_sound_sources', payload(index))
            self.assertNotIn('timed_sound_sources', payload(index.parent / 'index.html.parts/world.html'))
            bad = copy.deepcopy(original); bad['timed_sounds']['sources'][4]['values']['values'][0]['tick'] = True
            projected, errors = project_timed_sound_sources(bad)
            self.assertEqual(len(projected), 1); self.assertEqual(len(errors), 1)
            bad = copy.deepcopy(original); bad['red_spring_sounds'][0]['root_pointer'] = 0
            projected, errors = project_timed_sound_sources(bad)
            self.assertEqual(len(projected), 5); self.assertEqual(len(errors), 1)
            for mutate in [lambda b: b['sources'][0]['values'].update(kind='unknown'),
                           lambda b: b['sources'][1].update(offset=0x400),
                           lambda b: b['sources'][2]['values']['values'].__setitem__(0, 256),
                           lambda b: b['sources'][3]['values']['values'].__setitem__(0, True)]:
                bad = copy.deepcopy(original); mutate(bad['timed_sounds'])
                projected, errors = project_timed_sound_sources(bad)
                self.assertEqual(len(projected), 1); self.assertEqual(len(errors), 1)

    def test_partial_directory_and_older_schedules(self):
        for options, expected in [({'invalid_songs': True}, 6), ({'invalid_bank': True}, 1), ({'include_bank': False}, 1)]:
            with self.subTest(options=options), tempfile.TemporaryDirectory(prefix='viewer-timed-partial-') as directory:
                work = Path(directory); dataset, _ = fixture(work, **options)
                audio = payload(generate(dataset, work / 'view', split=True).parent / 'index.html.parts/audio.html')
                self.assertEqual(len(audio['timed_sound_sources']['groups']), expected)
                if options.get('invalid_songs'):
                    self.assertIsNone(audio['sequence_directory'])
                    self.assertTrue(audio['resources'][0]['status'].startswith('Available: timed sound sources'))
                else:
                    self.assertIsNotNone(audio['sequence_directory'])


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class TimedSoundBrowserTest(unittest.TestCase):
    def test_filter_values_and_audition_through_existing_player(self):
        with tempfile.TemporaryDirectory(prefix='viewer-timed-browser-') as directory:
            work = Path(directory); dataset, _ = fixture(work)
            index = generate(dataset, work / 'view', split=True)
            browser = Chrome(work, index.parent / 'index.html.parts/audio.html')
            try:
                self.assertEqual(browser.evaluate('''(async()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change')),
                    input=e=>e.dispatchEvent(new Event('input'));
                  const row=q('[data-resource-id="audio.m4a-directory"]');
                  row.querySelector('a[href="#timed-sounds"]').click();
                  if(q('#timed-sounds').hidden || !q('#timed-sounds>details').open ||
                     q('#timed-sound-source').options.length!==6 || q('#timed-sound-rows').rows.length!==1 ||
                     !q('#timed-sound-detail').textContent.includes('observed Red Spring'))throw Error('observed schedule');
                  const cells=q('#timed-sound-rows').rows[0].cells;
                  if(cells[0].textContent!=='0x100' || cells[1].textContent!=='12' || cells[2].textContent!=='3' ||
                     !cells[4].textContent.includes('0x2005 · candidate ID 5 · flags 0x2') || cells[5].querySelector('a'))throw Error('row data');
                  window.timedSources=[];
                  class FakeContext {
                    constructor(){this.currentTime=0;this.destination={}} resume(){return Promise.resolve()}
                    createGain(){return {gain:{setValueAtTime(){},linearRampToValueAtTime(){}},connect(){},disconnect(){}}}
                    createOscillator(){const n={frequency:{value:0},connect(){},start(){},stop(){}};timedSources.push(n);return n}
                  }
                  window.AudioContext=FakeContext;q('#sequence-voice').value='sine';change(q('#sequence-voice'));
                  q('#sequence-mute-cgb').checked=true;change(q('#sequence-mute-cgb'));
                  cells[4].querySelector('button').click();
                  for(let i=0;i<100 && !timedSources.length;i++)await new Promise(r=>setTimeout(r,10));
                  if(!timedSources.length || q('#sequence-song').value!=='1' || !q('#sequence-audio>details').open ||
                     !q('#sequence-mute-cgb').checked)throw Error('audition');
                  q('#sequence-stop').click();
                  q('#timed-sound-kind').value='directory';change(q('#timed-sound-kind'));
                  if(q('#timed-sound-rows').rows.length!==2 || q('#timed-sound-rows').rows[1].cells[0].textContent!=='0x408' ||
                     q('#timed-sound-rows').rows[1].cells[1].textContent!=='0x08000200')throw Error('directory values');
                  q('#timed-sound-kind').value='rows';change(q('#timed-sound-kind'));
                  q('#timed-sound-filter').value='0x420';input(q('#timed-sound-filter'));
                  if(q('#timed-sound-source').options.length!==1 || !q('#timed-sound-detail').textContent.includes('source fragment'))throw Error('filter');
                  q('#timed-sound-filter').value='0x999';input(q('#timed-sound-filter'));
                  if(!q('#timed-sound-source').disabled || !q('#timed-sound-detail').textContent.includes('No matching'))throw Error('empty filter');
                  return 'PASS';
                })()'''), 'PASS')
            finally:
                browser.close()

    def test_sources_remain_visible_without_song_player(self):
        with tempfile.TemporaryDirectory(prefix='viewer-timed-no-player-') as directory:
            work = Path(directory); dataset, _ = fixture(work, include_sequences=False)
            browser = Chrome(work, generate(dataset, work / 'view'))
            try:
                self.assertTrue(browser.evaluate("!document.querySelector('#timed-sounds').hidden && document.querySelector('#sequence-audio').hidden && !document.querySelector('#timed-sound-rows a') && document.querySelector('#timed-sound-play').disabled"))
            finally:
                browser.close()
