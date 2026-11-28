"""Verify authenticated PCM echo settings, live playback and WAV output."""
import hashlib
import json
import shutil
import struct
import tempfile
import unittest
from pathlib import Path

from test_viewer_navbar_split_browser import wait_for
from test_viewer_scoped_browser import prepared, parse_page, REPO
from view_rom_data import project_audio_sequences, viewer_section_updates, write_split_viewer
from viewer_browser_harness import Chrome


def sequence_fixture():
    words = [0x08149f69, 0x08149f75]
    driver = {'clocks': [0, 3] + [1] * 47, 'extended_source_offset': 0x370d58, 'extended8_9': words}
    sources = [{'offset': 0x370d58, 'length': 8, 'role': 'audio/driver/extended8-9',
                'sha256': hashlib.sha256(struct.pack('<2I', *words)).hexdigest()}]
    instructions = []
    operations = [
        (2, {'kind': 'parameter', 'command': 0xbd, 'value': 0}),
        (3, {'kind': 'extended', 'selector': 8, 'value': 30}),
        (3, {'kind': 'extended', 'selector': 9, 'value': 12}),
        (3, {'kind': 'note', 'clock': 1, 'key': 60, 'velocity': 100, 'extension': None}),
        (3, {'kind': 'extended', 'selector': 8, 'value': 0}),
        (1, {'kind': 'wait', 'clock': 1}),
        (1, {'kind': 'end'}),
        (1, {'kind': 'end'}),
    ]
    offset = 0x1000
    for index, (length, command) in enumerate(operations):
        instructions.append({'id': index, 'source_offset': offset, 'explicit': True, 'code': command})
        offset += length
    tracks = [{'id': index, 'directory_header': index, 'track_index': 0, 'entry': root}
              for index, root in enumerate((0, 7))]
    return {'driver': driver, 'tracks': tracks, 'instructions': instructions}, sources


def audio_page(work):
    data, _ = prepared(work)
    catalog, sources = sequence_fixture()
    data['sequences'] = project_audio_sequences(catalog, echo_sources=sources)
    data['sequence_directory'] = {'songs': [{'id': 5, 'header_id': 0}, {'id': 6, 'header_id': 1}],
        'headers': [{'id': 0, 'tone_bank_address': 0x08001000}, {'id': 1}]}
    data['audio'] = {'path': 'audio', 'waves': [{'id': 0, 'source_offset': 0x2000, 'tone_ids': [0],
        'frequency_word': 22050, 'loop_start': 0, 'flags': 0x4000, 'samples': [127] * 8}], 'cgb_waves': [],
        'instruments': {'banks': [{'source_address': 0x08001000, 'programs': [{'selector': 0, 'tone': 0}]}],
        'tones': [{'id': 0, 'ctype': 0, 'word04': 0x08002000, 'word08': 0x8080ffff}], 'map_links': []}}
    data['timed_sound_sources'] = {'path': 'schedule', 'groups': [{'kind': 'rows', 'origin': 'fixture',
        'source_offset': 256, 'values': [{'tick': 0, 'channel': 0, 'opaque03': 0, 'sound_words': [5, 0]}]}]}
    template = (REPO / 'view_rom_data.html').read_text()
    index = write_split_viewer(data, work / 'echo', 'index.html', template)
    return index.parent / (index.name + '.parts/audio.html')


class PcmEchoProjectionTest(unittest.TestCase):
    def test_recognition_requires_both_callbacks_and_the_source_binding(self):
        catalog, sources = sequence_fixture()
        self.assertNotIn('pcm_echo', project_audio_sequences(catalog))
        self.assertTrue(project_audio_sequences(catalog, echo_sources=sources)['pcm_echo'])
        for change in ({'role': 'unknown'}, {'length': 4}, {'offset': 0}, {'sha256': '0' * 64}):
            with self.subTest(change=change):
                self.assertNotIn('pcm_echo', project_audio_sequences(catalog, echo_sources=[{**sources[0], **change}]))
        catalog['driver']['extended8_9'][1] += 4
        self.assertNotIn('pcm_echo', project_audio_sequences(catalog, echo_sources=sources))

    def test_echo_song_links_use_lazy_song_metadata(self):
        with tempfile.TemporaryDirectory(prefix='viewer-pcm-echo-scope-') as directory:
            page = audio_page(Path(directory))
            sequences = parse_page(page)['sequences']
            self.assertEqual(sequences['pcm_echo_headers'], [0])
            self.assertEqual(sequences['instructions'], [])
            self.assertNotIn('sequences', parse_page(page.with_name('audio-samples.html')))


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class PcmEchoBrowserTest(unittest.TestCase):
    def test_echo_tail_in_live_timed_and_exported_output(self):
        with tempfile.TemporaryDirectory(prefix='viewer-pcm-echo-browser-') as directory:
            work = Path(directory)
            page = audio_page(work)
            browser = Chrome(work, page)
            try:
                timestamp = viewer_section_updates()['audio']['sequence-pcm-echo']['at']
                result = browser.evaluate('''(async()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change'));
                  const wait=async f=>{const end=Date.now()+8000;while(!f()){if(Date.now()>end)throw Error('timeout');await new Promise(r=>setTimeout(r,10))}};
                  const step=280896/16777216,near=(a,b)=>Math.abs(a-b)<1e-8;
                  const tail=framePcmEnvelope(0x8080ffff,3*step,2,30,12);
                  if(!near(tail.duration,18*step)||!near(tail.points[6][1],30/255)||!near(tail.points[7][1],0))throw Error('release tail');
                  const decay=framePcmEnvelope(0x800000ff,7*step,2,30,4);
                  if(!near(decay.duration,5*step)||!near(decay.points[1][1],30/255))throw Error('zero decay tail');
                  const clipped=framePcmEnvelope(0x8080ffff,3*step,9*step,30,12);
                  if(!near(clipped.duration,9*step)||clipped.points.at(-1)[1]!==30/255)throw Error('bounded echo');
                  if(!near(framePcmEnvelope(0x8080ffff,3*step,2,30,0).duration,10*step))throw Error('zero length');
                  const echo=q('#sequence-use-pcm-echo'), badge=q('#sequence-pcm-echo>summary time');
                  if(!echo.checked||!badge||badge.dateTime!==TIMESTAMP||q('#sequence-pcm-echo').hidden)throw Error('control or badge');
                  q('#sequence-pcm-echo-songs button').click();
                  await wait(()=>!q('#sequence-play').disabled);
                  q('#sequence-use-tempo').checked=false;change(q('#sequence-use-tempo'));
                  for(const [id,value] of [['sequence-rate','60'],['sequence-duration','1']]){
                    q('#'+id).value=value;q('#'+id).dispatchEvent(new Event('input'));
                  }
                  if(!q('#sequence-note-rows').textContent.includes('echo level 30 for 12 frames')||
                     !q('#sequence-control-rows').textContent.includes('PCM echo length (frames)'))throw Error('note or command details');
                  const start=AudioBufferSourceNode.prototype.start, stop=AudioBufferSourceNode.prototype.stop;
                  const starts=new WeakMap(),durations=[];
                  AudioBufferSourceNode.prototype.start=function(when,...args){starts.set(this,when);return start.call(this,when,...args)};
                  AudioBufferSourceNode.prototype.stop=function(when){if(Number.isFinite(when)&&starts.has(this))durations.push(when-starts.get(this));return stop.call(this,when)};
                  q('#sequence-play').click();await wait(()=>durations.length===1);
                  if(!near(durations[0],18*step))throw Error('live tail');
                  q('#sequence-stop').click();
                  q('#timed-sound-duration').value='1';q('#timed-sound-duration').dispatchEvent(new Event('input'));
                  q('#timed-sound-play').click();await wait(()=>durations.length===2);
                  if(!near(durations[1],durations[0]))throw Error('timed tail');
                  q('#timed-sound-stop').click();
                  const blobs=[],url=URL.createObjectURL.bind(URL);
                  URL.createObjectURL=blob=>{blobs.push(blob);return url(blob)};
                  HTMLAnchorElement.prototype.click=function(){};
                  async function save(){const count=blobs.length;q('#sequence-download').click();await wait(()=>blobs.length>count);
                    return new DataView(await blobs[count].arrayBuffer());}
                  const wet=await save();echo.checked=false;change(echo);
                  const dry=await save();
                  if(wet.getUint32(40,true)/4!==Math.ceil(18*step*32000)||dry.getUint32(40,true)/4!==Math.ceil(10*step*32000))
                    throw Error('WAV frame counts');
                  if(wet.getInt16(44+4*Math.floor(14*step*32000),true)<=0)throw Error('WAV echo samples');
                  q('#sequence-song').value='1';change(q('#sequence-song'));
                  if(echo.checked)throw Error('echo selection lost on song switch');
                  q('#sequence-frame-envelope').checked=false;change(q('#sequence-frame-envelope'));
                  if(!echo.disabled)throw Error('smooth envelope guard');
                  return 'PASS';
                })()'''.replace('TIMESTAMP', json.dumps(timestamp)))
                self.assertEqual(result, 'PASS')
                browser.command('Page.navigate', {'url': page.as_uri()})
                wait_for(browser, "document.querySelector('#sequence-use-pcm-echo') !== null")
                self.assertTrue(browser.evaluate("document.querySelector('#sequence-use-pcm-echo').checked===false"))
            finally:
                browser.close()
