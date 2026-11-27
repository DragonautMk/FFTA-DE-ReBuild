"""Exercise bounded sequence modulation, live playback and WAV scheduling."""
import json
import random
import shutil
import tempfile
import unittest
from pathlib import Path

from test_viewer_sequence_wav_browser import fixture as audio_fixture
from viewer_browser_harness import Chrome


def fixture(work):
    page = audio_fixture(work)
    text = page.read_text()
    start = text.index('const data = ') + len('const data = ')
    data, length = json.JSONDecoder().raw_decode(text[start:])
    instructions, tracks = [], []
    for mode in range(3):
        entry = len(instructions)
        codes = [['parameter', 0xbd, 0], ['parameter', 0xbe, 64], ['parameter', 0xbf, 64],
                 ['parameter', 0xc2, 16], ['parameter', 0xc3, 2], ['parameter', 0xc4, 32],
                 ['parameter', 0xc5, mode], ['note', 2, 60, 100, None], ['wait', 2], ['end']]
        instructions.extend({'source_offset': entry + i, 'next': None if code[0] == 'end' else entry + i + 1, 'code': code}
                            for i, code in enumerate(codes))
        tracks.append({'directory_header': 0, 'track_index': mode, 'entry': entry})
    data['sequences']['instructions'] = instructions
    data['sequences']['tracks'] = tracks
    data['sequences']['clocks'] = [0, 1, 16] + [1] * 46
    data['timed_sound_sources'] = {'path': 'test-schedule', 'groups': [
        {'kind': 'rows', 'origin': 'test', 'source_offset': 1,
         'values': [{'tick': 0, 'channel': 0, 'opaque03': 0, 'sound_words': [7]},
                    {'tick': 2, 'channel': 0, 'opaque03': 0, 'sound_words': [7]}]}]}
    page.write_text(text[:start] + json.dumps(data) + text[start + length:])
    return page


def reference(events, note_ticks, end_tick, maximum):
    def signed8(value):
        value &= 255
        return value - 256 if value >= 128 else value
    speed, delay, depth, mode, phase, counter, value = 22, 0, 0, 0, 0, 0, 0
    steps, position, work, limited = [], 0, 0, False
    notes = {tick: note_ticks.count(tick) for tick in set(note_ticks)}
    for tick in range(end_tick):
        stop = False
        while position < len(events) and events[position]['tick'] == tick:
            code = events[position]['code']
            position += 1
            if code[0] == 'end':
                if value:
                    value = 0
                    steps.append((tick, mode, value))
                stop = True
                break
            if code[0] == 'note' and notes.get(tick, 0):
                notes[tick] -= 1
                counter = delay
                if delay:
                    phase = 0
                    if value:
                        value = 0
                        steps.append((tick, mode, value))
            elif code[0] == 'parameter':
                command, operand = code[1:]
                if command == 0xc2:
                    speed = operand
                    if not speed:
                        phase = 0
                        if value:
                            value = 0
                            steps.append((tick, mode, value))
                elif command == 0xc3:
                    delay = operand
                elif command == 0xc4:
                    depth = operand
                    if not depth:
                        phase = 0
                        if value:
                            value = 0
                            steps.append((tick, mode, value))
                elif command == 0xc5 and mode != operand:
                    mode = operand
                    if value:
                        steps.append((tick, mode, value))
        if stop:
            break
        if speed and depth:
            if work >= maximum:
                limited = True
                break
            work += 1
            if counter:
                counter -= 1
            else:
                total = phase + speed
                phase = total & 255
                triangle = signed8(total) if signed8(total - 64) < 0 else 128 - total
                next_value = signed8((triangle * depth) >> 6)
                if next_value != value:
                    value = next_value
                    steps.append((tick, mode, value))
        elif position >= len(events):
            break
    return {'steps': [list(step) for step in steps], 'limited': limited}


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class SequenceModulationBrowserTest(unittest.TestCase):
    def test_independent_tick_model_and_limits(self):
        with tempfile.TemporaryDirectory(prefix='viewer-lfo-model-') as directory:
            work = Path(directory)
            browser = Chrome(work, fixture(work))
            try:
                rng = random.Random(14860)
                cases = []
                for _ in range(500):
                    end = rng.randrange(5, 55)
                    events = [{'tick': 0, 'code': ['parameter', 0xc2, rng.randrange(256)]},
                              {'tick': 0, 'code': ['parameter', 0xc3, rng.randrange(6)]},
                              {'tick': 0, 'code': ['parameter', 0xc4, rng.randrange(128)]},
                              {'tick': 0, 'code': ['parameter', 0xc5, rng.randrange(4)]},
                              {'tick': 0, 'code': ['note', 1, 60, 100, None]}]
                    for _ in range(rng.randrange(5)):
                        tick = rng.randrange(1, end)
                        command = rng.choice([0xc2, 0xc3, 0xc4, 0xc5, 'note'])
                        code = ['note', 1, 60, 100, None] if command == 'note' else [
                            'parameter', command, rng.randrange(4 if command == 0xc5 else 256)]
                        events.append({'tick': tick, 'code': code})
                    events.sort(key=lambda item: item['tick'])
                    notes = [e['tick'] for e in events if e['code'][0] == 'note']
                    maximum = rng.randrange(1, 70)
                    cases.append({'events': events, 'notes': notes, 'end': end, 'maximum': maximum})
                actual = browser.evaluate('''(()=>{const cases=CASES;return cases.map(c=>{
                  const result=sequenceLfoSteps(c.events,c.notes.map(start=>({start})),
                    {endTick:c.end,timeAt:tick=>tick/10},c.end/10,c.maximum);
                  return {steps:result.steps.map(x=>[x.tick,x.mode,x.value]),limited:result.limited};});})()'''.replace('CASES', json.dumps(cases)))
                self.assertEqual(actual, [reference(c['events'], c['notes'], c['end'], c['maximum']) for c in cases])
            finally:
                browser.close()

    def test_pitch_volume_pan_live_wav_timed_and_setting(self):
        with tempfile.TemporaryDirectory(prefix='viewer-lfo-audio-') as directory:
            work = Path(directory)
            browser = Chrome(work, fixture(work))
            try:
                self.assertTrue(browser.evaluate('''(async()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change')),
                    input=e=>e.dispatchEvent(new Event('input'));
                  const wait=async fn=>{const end=Date.now()+5000;while(!fn()){if(Date.now()>end)throw Error('timeout');await new Promise(r=>setTimeout(r,10))}};
                  window.live=[];window.offlineNodes=[];window.exports=[];window.contexts=[];
                  class Param{constructor(v=0){this.value=v;this.events=[]}setValueAtTime(v,t){this.value=v;this.events.push([v,t])}linearRampToValueAtTime(v,t){this.value=v;this.events.push([v,t])}}
                  class Node{connect(target){this.output=target}disconnect(){}}
                  class Base{
                    constructor(list){this.currentTime=0;this.destination={};this.list=list}
                    createGain(){const n=new Node();n.gain=new Param(1);return n}
                    createStereoPanner(){const n=new Node();n.pan=new Param(0);return n}
                    createBuffer(c,n,r){return {getChannelData(){return new Float32Array(n)}}}
                    createBufferSource(){const n=new Node();n.playbackRate=new Param(1);n.start=t=>n.started=t;n.stop=t=>n.stopped=t;this.list.push(n);return n}
                    createOscillator(){const n=new Node();n.frequency=new Param(0);n.start=t=>n.started=t;n.stop=t=>n.stopped=t;this.list.push(n);return n}
                  }
                  window.AudioContext=class extends Base{constructor(){super(live);contexts.push(this)}resume(){return Promise.resolve()}};
                  window.OfflineAudioContext=class extends Base{
                    constructor(channels,frames,rate){super(offlineNodes);this.frames=frames;this.rate=rate}
                    startRendering(){return Promise.resolve({numberOfChannels:2,length:this.frames,sampleRate:this.rate,getChannelData:()=>new Float32Array(this.frames)})}
                  };
                  URL.createObjectURL=()=> 'blob:fake';URL.revokeObjectURL=()=>{};HTMLAnchorElement.prototype.click=function(){exports.push(this.download)};
                  q('#sequence-use-tempo').checked=false;change(q('#sequence-use-tempo'));
                  q('#sequence-rate').value='10';input(q('#sequence-rate'));q('#sequence-duration').value='2';input(q('#sequence-duration'));
                  q('#sequence-voice').value='sine';change(q('#sequence-voice'));
                  if(!q('#sequence-control-rows').textContent.includes('Modulation speed') || !q('#sequence-modulation').checked)throw Error('controls');
                  q('#sequence-play').click();await wait(()=>live.length===3);
                  const close=(a,b)=>Math.abs(a-b)<1e-7, expected=440*Math.pow(2,(60-69)/12)*Math.pow(2,.5/12);
                  if(!live[0].frequency.events.some(([v,t])=>close(v,expected)&&close(t,.25)))throw Error('pitch LFO');
                  if(!live[1].output.output.gain.events.some(([v,t])=>close(v,64/127*136/128)&&close(t,.25)))throw Error('volume LFO');
                  if(!live[2].output.output.output.pan.events.some(([v,t])=>close(v,8/128)&&close(t,.25)))throw Error('pan LFO');
                  q('#sequence-download').click();await wait(()=>exports.length===1);
                  const compare=(a,b)=>a.length===b.length&&a.every((x,i)=>close(x[0],b[i][0])&&close(x[1]-.05,b[i][1]));
                  if(!compare(live[0].frequency.events,offlineNodes[0].frequency.events) ||
                     !compare(live[1].output.output.gain.events,offlineNodes[1].output.output.gain.events) ||
                     !compare(live[2].output.output.output.pan.events,offlineNodes[2].output.output.output.pan.events))throw Error('WAV scheduling');
                  q('#timed-sound-rate').value='10';input(q('#timed-sound-rate'));q('#timed-sound-duration').value='2';input(q('#timed-sound-duration'));
                  const before=live.length;q('#timed-sound-play').click();await wait(()=>live.length>=before+6);
                  if(!live[before].frequency.events.some(([v,t])=>close(v,expected)&&close(t,.25)) ||
                     !live[before+3].frequency.events.some(([v,t])=>close(v,expected)&&close(t,.45)))throw Error('timed LFO');
                  q('#sequence-modulation').checked=false;change(q('#sequence-modulation'));
                  const plain=live.length;q('#sequence-play').click();await wait(()=>live.length>=plain+3);
                  if(live[plain].frequency.events.some(([v])=>!close(v,440*Math.pow(2,(60-69)/12))))throw Error('disabled');
                  q('#sequence-track').value='0';change(q('#sequence-track'));
                  if(q('#sequence-modulation').checked || localStorage.getItem('ffta.viewer.sequence.modulation')!=='0')throw Error('setting persistence');
                  q('#sequence-track-controls').checked=false;change(q('#sequence-track-controls'));
                  return q('#sequence-modulation').disabled && !q('#sequence-modulation').checked;
                })()'''))
            finally:
                browser.close()
