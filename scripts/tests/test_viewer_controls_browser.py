"""Exercise offline viewer controls through actual Chrome input events."""
import json
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from viewer_browser_harness import Chrome


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class ViewerControlsTest(unittest.TestCase):
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
             class FakeAudioContext {constructor(){this.currentTime=0;this.destination={};instances.push(this)} resume(){return Promise.resolve()} createBuffer(channels,length,rate){return {sampleRate:rate,getChannelData(){return new Float32Array(length)}}} createBufferSource(){const n={buffer:null,playbackRate:{value:1},connect(){},disconnect(){},start(){},stop(){}};sources.push(n);return n} createOscillator(){const n={frequency:{value:0},connect(){},start(){},stop(){}};sources.push(n);return n} createGain(){return {gain:{setValueAtTime(){},linearRampToValueAtTime(){}},connect(){}}}}
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
