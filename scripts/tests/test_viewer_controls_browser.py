"""Exercise offline viewer controls through actual Chrome input events."""
import json
import shutil
import subprocess
import tempfile
import time
import unittest
from pathlib import Path
from unittest import mock

from viewer_browser_harness import Chrome


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class ViewerControlsTest(unittest.TestCase):
    def test_mute_cgb_routing_and_persistence(self):
        with tempfile.TemporaryDirectory(prefix='viewer-cgb-') as directory:
            work = Path(directory)
            repo = Path(__file__).resolve().parents[2]
            bank_address = 0x08001000
            tones = [{'id': index, 'ctype': kind, 'word04': word}
                     for index, (kind, word) in enumerate([
                         (0, 0x08002000), (3, 0x08003000), (1, 0), (2, 0),
                         (4, 0), (0x40, 0), (3, 0x08003000), (8, 0),
                     ])]
            bank = {'source_address': bank_address,
                    'programs': [{'selector': i, 'tone': i} for i in (0, 1, 2, 3, 4, 5, 7)]}
            instructions, tracks = [], []
            # The unmapped program deliberately remains audible when CGB is muted.
            for index, program in enumerate([0, 1, 2, 3, 4, 5, 99, 7]):
                entry = len(instructions)
                instructions.extend([
                    {'source_offset': entry, 'next': entry + 1, 'code': ['parameter', 0xbd, program]},
                    {'source_offset': entry + 1, 'next': entry + 2, 'code': ['note', 3, 60, 100, None]},
                    {'source_offset': entry + 2, 'next': None, 'code': ['end']},
                ])
                tracks.append({'directory_header': 0, 'track_index': index, 'entry': entry})
            tracks.append({'directory_header': 1, 'track_index': 0, 'entry': 3})
            data = {
                'profile': 'synthetic', 'extractor': 1, 'decoder': 1, 'resources': [],
                'audio': {'path': 'audio.json', 'waves': [{
                    'id': 0, 'source_offset': 0x2000, 'tone_ids': [0], 'samples': [0, 64, 0, -64],
                    'frequency_word': 14069760, 'loop_start': 0, 'flags': 0,
                }], 'cgb_waves': [{
                    'id': 0, 'source_offset': 0x3000, 'tone_ids': [1, 6], 'amplitudes': list(range(16)) * 2,
                }], 'instruments': {'banks': [bank], 'tones': tones,
                                   'map_links': [{'parent_tone': 5, 'key': 60, 'tone': 6}]}},
                'sequence_directory': {'songs': [{'id': i, 'header_id': i} for i in range(2)],
                                       'headers': [{'id': i, 'tone_bank_address': bank_address} for i in range(2)]},
                'sequences': {'path': 'sequences.json', 'clocks': [0, 1, 2, 30] + [1] * 45,
                              'tracks': tracks, 'instructions': instructions},
            }
            template = (repo / 'view_rom_data.html').read_text()
            self.assertEqual(template.count('/* DATA */'), 1)
            page = work / 'synthetic.html'
            page.write_text(template.replace('/* DATA */', json.dumps(data)))
            browser = Chrome(work, page)
            try:
                result = browser.evaluate('''(async()=>{
                  const q=s=>document.querySelector(s), change=e=>e.dispatchEvent(new Event('change'));
                  const wait=async fn=>{const end=Date.now()+4000;while(!fn()){if(Date.now()>end)throw Error('timeout');await new Promise(r=>setTimeout(r,10))}};
                  window.sources=[];window.buses=[];
                  class FakeContext {
                    constructor(){this.currentTime=0;this.destination={kind:'destination'}}
                    resume(){return Promise.resolve()}
                    createBuffer(channels,length,rate){return {getChannelData(){return new Float32Array(length)}}}
                    createGain(){const node={gain:{value:1,setValueAtTime(v){this.value=v},linearRampToValueAtTime(){}},
                      connect(target){this.output=target},disconnect(){this.disconnected=true}};buses.push(node);return node}
                    createBufferSource(){return this.makeSource('sample')}
                    createOscillator(){return this.makeSource('sine')}
                    makeSource(kind){const node={kind,playbackRate:{value:1},frequency:{value:0},
                      connect(target){this.output=target},start(){},stop(){}};sources.push(node);return node}
                  }
                  window.AudioContext=FakeContext;
                  const mute=q('#sequence-mute-cgb'),track=q('#sequence-track'),song=q('#sequence-song');
                  if(mute.checked)throw Error('default mute');
                  q('#sequence-play').click();await wait(()=>sources.length===8);
                  const bus=buses[0],destination=bus.output;
                  if(bus.gain.value!==1)throw Error('initial gain');
                  for(const i of [1,2,3,4,5])if(sources[i].output.output!==bus)throw Error('CGB routing '+i);
                  for(const i of [0,6,7])if(sources[i].output.output!==destination)throw Error('non-CGB routing '+i);
                  mute.checked=true;change(mute);
                  if(bus.gain.value!==0 || sources.length!==8 || q('#sequence-stop').disabled)throw Error('live mute');
                  if(![...document.querySelectorAll('#sequence-note-rows tr')].some(r=>r.cells[6].textContent.includes('CGB · sine fallback · muted')))throw Error('fallback label');
                  mute.checked=false;change(mute);if(bus.gain.value!==1)throw Error('live unmute');
                  mute.checked=true;change(mute);track.value='1';change(track);
                  if(!mute.checked || !bus.disconnected)throw Error('track persistence and cleanup');
                  q('#sequence-play').click();await wait(()=>sources.length===9);
                  const nextBus=buses.findLast(n=>n.output===destination);
                  if(nextBus.gain.value!==0 || sources[8].output.output!==nextBus)throw Error('muted track');
                  q('#sequence-voice').value='sine';change(q('#sequence-voice'));q('#sequence-play').click();
                  await wait(()=>sources.length===10);
                  const sineBus=buses.findLast(n=>n.output===destination);
                  if(!mute.checked || sources[9].kind!=='sine' || sources[9].output.output!==sineBus || sineBus.gain.value!==0)throw Error('sine mode mute');
                  song.value='1';change(song);if(!mute.checked)throw Error('song persistence');
                  q('#sequence-play').click();await wait(()=>sources.length===11);
                  const songBus=buses.findLast(n=>n.output===destination);
                  if(songBus.gain.value!==0 || sources[10].output.output!==songBus)throw Error('muted next song');
                  q('#sequence-stop').click();return 'PASS';
                })()''')
                self.assertEqual(result, 'PASS')
                browser.command('Page.reload')
                deadline = time.monotonic() + 15
                while time.monotonic() < deadline:
                    try:
                        if browser.evaluate('document.readyState === "complete" && document.querySelector("#sequence-mute-cgb")?.checked'):
                            break
                    except RuntimeError:
                        pass
                    time.sleep(.05)
                else:
                    self.fail('Mute setting did not survive page reload')
            finally:
                browser.close()

    def test_failed_browser_setup_releases_the_process(self):
        with tempfile.TemporaryDirectory(prefix='viewer-failure-') as directory:
            work = Path(directory)
            page = work / 'page.html'
            page.write_text('<!doctype html><title>Browser setup</title>')
            processes, links = [], []
            popen, make_dir = subprocess.Popen, tempfile.TemporaryDirectory

            def track_process(*args, **kwargs):
                process = popen(*args, **kwargs)
                processes.append(process)
                return process

            def track_directory(*args, **kwargs):
                result = make_dir(*args, **kwargs)
                links.append(result.name)
                return result

            with (mock.patch('viewer_browser_harness.subprocess.Popen', side_effect=track_process),
                  mock.patch('viewer_browser_harness.tempfile.TemporaryDirectory', side_effect=track_directory),
                  mock.patch('viewer_browser_harness.urllib.request.urlopen', side_effect=RuntimeError('test setup failure'))):
                with self.assertRaisesRegex(RuntimeError, 'test setup failure'):
                    Chrome(work, page)
            self.assertEqual(len(processes), 1)
            self.assertIsNotNone(processes[0].poll())
            self.assertEqual(len(links), 1)
            self.assertFalse(Path(links[0]).exists())

    def test_audio_text_and_sprite_controls(self):
        with tempfile.TemporaryDirectory(prefix='viewer-controls-') as directory:
            work = Path(directory)
            repo = Path(__file__).resolve().parents[2]
            glyph = {'index': 7, 'width': 8, 'tile_words': [1] + [0] * 31}
            langs = [{'language_id': i, 'entries': [i]} for i in range(2)]
            strings = [{'encoding': 'compact', 'units': [
                {'kind': 'glyph', 'index': 7}, {'kind': 'control_byte', 'value': i + 1},
            ]} for i in range(2)]
            pointer_strings = [{'encoding': 'compact', 'source_offset': 10 + i, 'units': [7, [i + 1]]}
                               for i in range(2)]

            def pointer_languages(count):
                return [{'source_offset': 20 + i, 'entries': [i]} for i in range(count)]

            def relative_languages(count):
                return [{'source_offset': 20 + i, 'programs': [
                    {'source_offset': 30 + i, 'header': 3, 'units': [7, [0x63]]},
                ]} for i in range(count)]

            def frame(index):
                return {'block': 0, 'object_source': 100 + index, 'object_count': 0, 'objects': [], 'duration': 1}

            scripts = [{'id': i, 'banks': [{'source_offset': 100 + i, 'programs': [
                {'source_offset': 110 + i, 'header': 3, 'units': [7, [0x63]]},
            ]}]} for i in range(2)]
            data = {
                'profile': 'synthetic', 'extractor': 1, 'decoder': 1, 'resources': [],
                'short_texts': {'path': 'text.json', 'languages': langs, 'strings': strings, 'glyphs': [glyph]},
                'pointer_texts': {'path': 'pointer.json', 'catalogs': [
                    {'reader': 1, 'languages': pointer_languages(2)},
                    {'reader': 2, 'languages': pointer_languages(1)},
                ], 'strings': pointer_strings, 'glyphs': [glyph]},
                'relative_texts': {'path': 'relative.json', 'catalogs': [
                    {'reader': 1, 'languages': relative_languages(2)},
                    {'reader': 2, 'languages': relative_languages(1)},
                ], 'glyphs': [glyph]},
                'script_texts': {'path': 'script.json', 'languages': scripts, 'glyphs': [glyph]},
                'sprite_pixels': {'path': 'sprite.json',
                                  'blocks': [{'source_offset': 10, 'tiles': [{'indices': [[0] * 8 for _ in range(8)]}]}],
                                  'sprites': [{'id': 10, 'frames': [frame(i) for i in range(3)]},
                                              {'id': 11, 'frames': [frame(i) for i in range(2)]}]},
                'audio': {'path': 'audio.json', 'waves': [{'id': 0, 'samples': [0] * 24000, 'loop_start': 8000}],
                          'cgb_waves': [{'id': 0, 'amplitudes': list(range(16)) * 2}]},
                'sequence_directory': {'songs': [{'id': 0, 'header_id': 0}],
                                       'headers': [{'id': 0, 'tone_bank_address': None}]},
                'sequences': {'path': 'sequences.json', 'clocks': [0, 1, 2, 30] + [1] * 45,
                              'tracks': [{'directory_header': 0, 'track_index': 0, 'entry': 0}],
                              'instructions': [{'source_offset': 1, 'next': 1, 'code': ['note', 3, 60, 100, None]},
                                               {'source_offset': 2, 'next': None, 'code': ['end']}]},
            }
            template = (repo / 'view_rom_data.html').read_text()
            self.assertEqual(template.count('/* DATA */'), 1)
            page = work / 'synthetic.html'
            page.write_text(template.replace('/* DATA */', json.dumps(data)))
            browser = Chrome(work, page)
            try:
             setup='''(()=>{window.instances=[];window.sources=[];
             class FakeAudioContext {constructor(){this.currentTime=0;this.destination={};instances.push(this)} resume(){return Promise.resolve()} createBuffer(channels,length,rate){return {sampleRate:rate,getChannelData(){return new Float32Array(length)}}} createBufferSource(){const n={buffer:null,playbackRate:{value:1},connect(){},disconnect(){},start(){},stop(){}};sources.push(n);return n} createOscillator(){const n={frequency:{value:0},connect(){},start(){},stop(){}};sources.push(n);return n} createGain(){return {gain:{setValueAtTime(){},linearRampToValueAtTime(){}},connect(){},disconnect(){}}}}
             window.AudioContext=FakeAudioContext;for(const id of ['audio','sequence-audio','texts','pointer-texts','relative-texts','script-texts','sprites'])document.querySelector('#'+id+' > details').open=true;return 'PASS';})()'''
             assert browser.evaluate(setup)=='PASS'
             checks='''(async()=>{
              const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change')),wait=async(f,t=4000)=>{let end=Date.now()+t;while(!f()){if(Date.now()>end)throw Error('timeout');await new Promise(r=>setTimeout(r,10))}},delay=()=>new Promise(r=>setTimeout(r,140));
              q('#text-language').value='1';change(q('#text-language'));q('#text-pin').checked=true;change(q('#text-pin'));
              for(const id of ['text','pointer','relative','script'])if(!q('#'+id+'-pin').checked || q('#'+id+'-language').selectedOptions[0].textContent!=='1')throw Error('pin sync '+id);
              q('#pointer-reader').value='1';change(q('#pointer-reader'));if(!q('#pointer-pin-status').textContent.includes('unavailable'))throw Error('unavailable language');
              q('#pointer-reader').value='0';change(q('#pointer-reader'));if(q('#pointer-language').value!=='1')throw Error('reader pin');
              q('#script-hide-commands').checked=true;change(q('#script-hide-commands'));
              for(const id of ['text','pointer','relative','script'])if(!q('#'+id+'-hide-commands').checked || q('#'+id+'-preview').querySelector('span'))throw Error('hide '+id);
              q('#relative-hide-commands').checked=false;change(q('#relative-hide-commands'));
              for(const id of ['text','pointer','relative','script'])if(q('#'+id+'-hide-commands').checked || !q('#'+id+'-preview').querySelector('span'))throw Error('show '+id);
              q('#script-language').value='0';change(q('#script-language'));for(const id of ['text','pointer','relative','script'])if(q('#'+id+'-language').selectedOptions[0].textContent!=='0')throw Error('language change '+id);
              q('#sprite-rate').value='40';q('#sprite-frame').value='1';change(q('#sprite-frame'));q('#sprite-loop').checked=false;q('#sprite-play').click();
              await wait(()=>q('#sprite-animation-status').textContent.includes('finished'));if(q('#sprite-frame').value!=='2' || !q('#sprite-stop').disabled)throw Error('one-shot from frame');
              q('#sprite-loop').checked=true;q('#sprite-frame').value='1';change(q('#sprite-frame'));q('#sprite-play').click();
              q('#sprite-id').value='1';change(q('#sprite-id'));if(!q('#sprite-loop').checked || q('#sprite-rate').value!=='40' || q('#sprite-stop').disabled || q('#sprite-frame').value!=='1')throw Error('sprite settings');
              await wait(()=>q('#sprite-frame').value==='0');await delay();if(q('#sprite-stop').disabled)throw Error('continuous loop');q('#sprite-stop').click();
              q('#audio-rate').value='8000';q('#audio-loop').checked=true;q('#audio-play').click();await wait(()=>instances.length>0 && !q('#audio-stop').disabled);let c=instances[0];c.currentTime=2;await delay();
              if(Math.abs(q('#audio-progress').value-2)>.03 || !q('#audio-progress-label').textContent.includes('looping'))throw Error('PCM progress');
              c.currentTime=3.5;await delay();if(Math.abs(q('#audio-progress').value-1.5)>.03)throw Error('PCM loop position');
              q('#audio-stop').click();if(!q('#audio-progress-label').textContent.includes('Stopped'))throw Error('PCM stop');
              q('#audio-loop').checked=false;q('#audio-play').click();await wait(()=>!q('#audio-stop').disabled);sources[sources.length-1].onended();if(q('#audio-progress').value!==3 || !q('#audio-progress-label').textContent.includes('Finished'))throw Error('PCM finish');
              q('#cgb-play').click();await wait(()=>!q('#audio-stop').disabled);c.currentTime+=.4;await delay();if(Math.abs(q('#cgb-progress').value-.4)>.03)throw Error('CGB progress');q('#audio-stop').click();
              q('#sequence-play').click();await wait(()=>instances.length===2 && !q('#sequence-stop').disabled);let s=instances[1];s.currentTime=.3;await delay();
              if(Math.abs(q('#sequence-progress').value-.25)>.03 || q('#sequence-playhead').hidden || parseFloat(q('#sequence-playhead').style.left)<=0)throw Error('sequence progress');
              q('#sequence-stop').click();if(!q('#sequence-progress-label').textContent.includes('Stopped'))throw Error('sequence stop');
              q('#sequence-play').click();await wait(()=>!q('#sequence-stop').disabled);let n=sources[sources.length-1];n.onended();if(!q('#sequence-progress-label').textContent.includes('Finished'))throw Error('sequence finish');
              return 'PASS';
             })()'''
             assert browser.evaluate(checks)=='PASS'
             for select,button in [('#audio-id','#audio-play'),('#cgb-id','#cgb-play'),('#sequence-song','#sequence-play'),('#sequence-track','#sequence-play'),('#sequence-voice','#sequence-play')]:
              assert browser.evaluate(f"(()=>{{const e=document.querySelector('{select}');e.focus();return document.activeElement===e}})()")
              browser.click(button)
              focused=browser.evaluate('document.activeElement.id')
              if focused!=select[1:]:raise RuntimeError('focus lost '+select+' -> '+str(focused))
            finally:browser.close()
