"""Check on-demand, offline song graphs and timed audition loading."""
import hashlib
import json
import shutil
import tempfile
import unittest
from pathlib import Path

from test_viewer_timed_sounds_browser import fixture as base_fixture
from view_rom_data import generate, split_sequence_instructions, viewer_section_updates
from viewer_browser_harness import Chrome


def fixture(work):
    dataset, directory = base_fixture(work)
    directory['songs'].append({'id': 6, 'header_id': 6})
    directory['headers'].append({'id': 6, 'tone_bank_address': None})
    directory['red_spring_sounds'][0]['rows'][0]['sound_words'] = [5, 6]
    directory['timed_sounds']['sources'][-1]['values']['values'][0]['sound_words'] = [5, 6]
    old_directory = (dataset / 'audio-directory.json').read_bytes(); raw_directory = json.dumps(directory).encode()
    (dataset / 'audio-directory.json').write_bytes(raw_directory)
    path = dataset / 'audio-sequences.json'; old = path.read_bytes(); seq = json.loads(old)
    seq['tracks'].append({'id': 1, 'directory_header': 6, 'track_index': 0, 'entry': 2})
    seq['instructions'].extend([
        {'id': 2, 'source_offset': 100, 'explicit': True,
         'code': {'kind': 'note', 'clock': 1, 'key': 64, 'velocity': 100, 'extension': None}},
        {'id': 3, 'source_offset': 103, 'explicit': True, 'code': {'kind': 'end'}},
    ])
    raw = json.dumps(seq).encode(); path.write_bytes(raw)
    manifest = dataset / 'manifest.toml'; text = manifest.read_text().replace(hashlib.sha256(old).hexdigest(), hashlib.sha256(raw).hexdigest())
    text = text.replace(hashlib.sha256(old_directory).hexdigest(), hashlib.sha256(raw_directory).hexdigest()); manifest.write_text(text)
    index = generate(dataset, work / 'view', split=True)
    page = index.parent / 'index.html.parts/audio.html'
    return page, json.JSONDecoder().raw_decode(page.read_text().split('const data = ', 1)[1])[0]


class LazySequenceProjectionTest(unittest.TestCase):
    def test_graphs_include_next_pattern_jump_and_cycles(self):
        instructions = [
            {'source_offset': 0, 'next': 1, 'code': ['pattern', 3]},
            {'source_offset': 1, 'next': 2, 'code': ['wait', 1]},
            {'source_offset': 2, 'next': None, 'code': ['end']},
            {'source_offset': 3, 'next': 4, 'code': ['jump', 3]},
            {'source_offset': 4, 'next': None, 'code': ['return']},
            {'source_offset': 5, 'next': None, 'code': ['end']},
        ]
        seq = {'path': 'seq', 'clocks': [1]*49, 'instructions': instructions,
               'tracks': [{'directory_header': 1, 'track_index': 0, 'entry': 0},
                          {'directory_header': 2, 'track_index': 0, 'entry': 5}]}
        assets, view = split_sequence_instructions(seq)
        self.assertEqual(view['instructions'], []); self.assertEqual(view['instruction_count'], 6)
        self.assertEqual(view['instruction_assets']['1']['count'], 5); self.assertEqual(view['instruction_assets']['2']['count'], 1)
        self.assertEqual(len(assets), 2); self.assertEqual(seq['instructions'], instructions)

    def test_split_page_keeps_only_reachable_asset_references(self):
        with tempfile.TemporaryDirectory(prefix='viewer-lazy-song-projection-') as directory:
            page, data = fixture(Path(directory)); seq = data['sequences']
            self.assertEqual(seq['instructions'], []); self.assertEqual(seq['instruction_count'], 4)
            self.assertEqual(set(seq['instruction_assets']), {'5','6'})
            self.assertTrue(all((page.parent / item['href']).is_file() for item in seq['instruction_assets'].values()))
            self.assertNotEqual(seq['instruction_assets']['5']['key'], seq['instruction_assets']['6']['key'])


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class LazySequenceBrowserTest(unittest.TestCase):
    def test_rapid_switch_mutes_and_demand_loading(self):
        with tempfile.TemporaryDirectory(prefix='viewer-lazy-song-race-') as directory:
            work = Path(directory); page, _ = fixture(work); browser = Chrome(work,page)
            try:
                expected = viewer_section_updates()['audio']['sequence-audio']
                self.assertTrue(browser.evaluate('''(async()=>{const expected=EXPECTED,q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change')),
                  wait=async f=>{const end=Date.now()+5000;while(!f()){if(Date.now()>end)throw Error('timeout');await new Promise(r=>setTimeout(r,10));}};
                  const badge=q('#sequence-audio>details>summary time');if(!badge||badge.dateTime!==expected.at||!badge.textContent.startsWith('Updated '))throw Error('badge');
                  if(data.sequences.instructions.length || Object.keys(globalThis.__FFTA_SEQUENCE_INSTRUCTIONS__||{}).length)throw Error('eager');
                  const append=document.head.append.bind(document.head),held=[];
                  document.head.append=e=>{if(e.src&&e.src.includes('sequence-instructions-'))held.push(e);else append(e)};
                  q('#sequence-mute-cgb').checked=true;change(q('#sequence-mute-cgb'));
                  q('#sequence-song').value='1';change(q('#sequence-song'));q('#sequence-song').value='2';change(q('#sequence-song'));
                  if(held.length!==2||!q('#sequence-play').disabled)throw Error('pending');
                  append(held[0]);await wait(()=>data.sequences.instructions[0]);
                  if(q('#sequence-song').value!=='2'||!q('#sequence-play').disabled||!q('#sequence-status').textContent.includes('Loading'))throw Error('stale selection');
                  append(held[1]);await wait(()=>!q('#sequence-play').disabled);
                  if(!q('#sequence-mute-cgb').checked||q('#sequence-note-rows').rows[0].cells[3].textContent!=='64'||
                     data.sequences.instructions.filter(Boolean).length!==4)throw Error('selected data');
                  document.head.append=append;return true;})()'''.replace('EXPECTED',json.dumps(expected))))
            finally:browser.close()

    def test_missing_asset_can_be_retried(self):
        with tempfile.TemporaryDirectory(prefix='viewer-lazy-song-retry-') as directory:
            work = Path(directory); page, data = fixture(work); asset=page.parent/data['sequences']['instruction_assets']['5']['href'];backup=asset.with_suffix('.held')
            asset.rename(backup);browser=Chrome(work,page)
            try:
                self.assertTrue(browser.evaluate('''(async()=>{const q=s=>document.querySelector(s);q('#sequence-song').value='1';q('#sequence-song').dispatchEvent(new Event('change'));
                  const end=Date.now()+5000;while(q('#sequence-retry-load').hidden){if(Date.now()>end)throw Error('no load failure');await new Promise(r=>setTimeout(r,10));}
                  return q('#sequence-play').disabled&&q('#sequence-status').textContent.includes('Cannot load');})()'''))
                backup.rename(asset)
                self.assertTrue(browser.evaluate('''(async()=>{const q=s=>document.querySelector(s);q('#sequence-retry-load').click();
                  const end=Date.now()+5000;while(q('#sequence-play').disabled){if(Date.now()>end)throw Error(q('#sequence-status').textContent);await new Promise(r=>setTimeout(r,10));}
                  return q('#sequence-retry-load').hidden&&data.sequences.instructions.filter(Boolean).length===2;})()'''))
            finally:browser.close()

    def test_timed_mix_loads_other_songs_and_cancel_stops_pending_playback(self):
        with tempfile.TemporaryDirectory(prefix='viewer-lazy-timed-') as directory:
            work = Path(directory); page, _ = fixture(work); browser=Chrome(work,page)
            try:
                self.assertTrue(browser.evaluate('''(async()=>{const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change')),
                  wait=async f=>{const end=Date.now()+5000;while(!f()){if(Date.now()>end)throw Error('timeout '+q('#timed-sound-play-status').textContent);await new Promise(r=>setTimeout(r,10));}};
                  window.sources=[];class Param{constructor(v=0){this.value=v}setValueAtTime(v){this.value=v}linearRampToValueAtTime(v){this.value=v}}
                  class Node{connect(t){this.output=t}disconnect(){}}class FakeContext{constructor(){this.currentTime=0;this.destination={}}resume(){return Promise.resolve()}
                    createGain(){const n=new Node();n.gain=new Param(1);return n}createOscillator(){const n=new Node();n.frequency=new Param();n.start=t=>n.started=t;n.stop=t=>n.stopped=t;sources.push(n);return n}}
                  window.AudioContext=FakeContext;q('#sequence-song').value='1';change(q('#sequence-song'));await wait(()=>!q('#sequence-play').disabled);
                  const append=document.head.append.bind(document.head),held=[];document.head.append=e=>{if(e.src&&e.src.includes('sequence-instructions-'))held.push(e);else append(e)};
                  q('#timed-sound-rate').value='100';q('#timed-sound-rate').dispatchEvent(new Event('input'));q('#timed-sound-play').click();
                  await wait(()=>held.length===1);if(q('#timed-sound-stop').disabled||!q('#timed-sound-play-status').textContent.includes('Loading'))throw Error('pending timed');
                  q('#timed-sound-stop').click();append(held[0]);await wait(()=>data.sequences.instructions[2]);await new Promise(r=>setTimeout(r,80));
                  if(sources.length||!q('#timed-sound-stop').disabled)throw Error('cancel');
                  document.head.append=append;q('#timed-sound-play').click();await wait(()=>sources.length===2);
                  return q('#timed-sound-play-status').textContent.includes('Playing an approximate timed schedule');})()'''))
            finally:browser.close()


if __name__ == '__main__':unittest.main()
