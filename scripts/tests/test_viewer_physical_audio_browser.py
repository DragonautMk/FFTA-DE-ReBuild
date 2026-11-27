"""Exercise additional physical PCM, CGB and bank programs in the browser."""
import shutil
import tempfile
import unittest
from pathlib import Path

from test_viewer_physical_audio import fixture
from viewer_browser_harness import Chrome


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class PhysicalAudioBrowserTest(unittest.TestCase):
    def test_extra_waves_and_programs_use_existing_controls(self):
        with tempfile.TemporaryDirectory(prefix='viewer-physical-audio-browser-') as directory:
            work = Path(directory)
            index, _, _ = fixture(work)
            browser = Chrome(work, index.parent / (index.name + '.parts') / 'audio-samples.html')
            try:
                self.assertTrue(browser.evaluate('''(async()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change'));
                  const wait=async f=>{const end=Date.now()+5000;while(!f()){
                    if(Date.now()>end)throw Error('timeout');await new Promise(r=>setTimeout(r,10))}};
                  window.physicalSources=[];
                  class Param{constructor(v=0){this.value=v}setValueAtTime(v){this.value=v}linearRampToValueAtTime(v){this.value=v}}
                  class Node{connect(target){this.output=target}disconnect(){}}
                  window.AudioContext=class{constructor(){this.currentTime=0;this.destination={}}resume(){return Promise.resolve()}
                    createGain(){const n=new Node();n.gain=new Param(1);return n}
                    createBuffer(c,n,r){const values=new Float32Array(n);return {values,rate:r,getChannelData(){return values}}}
                    createBufferSource(){const n=new Node();n.playbackRate=new Param(1);n.start=()=>{};n.stop=()=>{};physicalSources.push(n);return n}};
                  if(data.audio.waves.length!==9 || data.audio.cgb_waves.length!==8 || data.audio.waves.some(w=>'samples' in w))
                    throw Error('initial lazy waves');
                  q('#audio>details').open=true;q('#audio-id').value='1';change(q('#audio-id'));
                  await wait(()=>Array.isArray(data.audio.waves[1].samples));
                  if(!q('#audio-detail').textContent.includes('physical.bin') || !q('#audio-id').selectedOptions[0].textContent.includes('additional'))
                    throw Error('physical PCM source');
                  q('#audio-play').click();await wait(()=>physicalSources.length===1);
                  if(physicalSources[0].buffer.values[0]!==1/128 || physicalSources[0].buffer.values[1]!==-2/128)
                    throw Error('physical PCM samples');
                  q('#audio-stop').click();q('#cgb-id').value='2';change(q('#cgb-id'));
                  if(!q('#cgb-detail').textContent.includes('physical.bin') || !q('#cgb-id').selectedOptions[0].textContent.includes('additional'))
                    throw Error('physical CGB source');
                  q('#cgb-play').click();await wait(()=>physicalSources.length===2);
                  if(Math.abs(physicalSources[1].buffer.values[0]-(1/7.5-1))>1e-6 ||
                     Math.abs(physicalSources[1].buffer.values[1]-(14/7.5-1))>1e-6)throw Error('physical CGB amplitudes');
                  q('#audio-stop').click();q('#instruments>details').open=true;
                  const banks=data.audio.instruments.banks,bank=banks.findIndex(b=>b.programs.some(p=>p.tone===429));
                  q('#instrument-bank').value=String(bank);change(q('#instrument-bank'));
                  q('#instrument-program').value=String(banks[bank].programs.findIndex(p=>p.tone===429));change(q('#instrument-program'));
                  if(!q('#instrument-source').textContent.includes('physical.bin') || !q('#instrument-program').selectedOptions[0].textContent.includes('additional') || q('#instrument-play').disabled)
                    throw Error('physical PCM program');
                  q('#instrument-play').click();await wait(()=>physicalSources.length===3);
                  if(physicalSources[2].buffer.values[1]!==-2/128)throw Error('physical PCM instrument');
                  q('#instrument-stop').click();q('#instrument-mute-cgb').checked=true;change(q('#instrument-mute-cgb'));
                  q('#instrument-program').value=String(banks[bank].programs.findIndex(p=>p.tone===437));change(q('#instrument-program'));
                  if(!q('#instrument-mute-cgb').checked || !q('#sequence-mute-cgb').checked || q('#instrument-play').disabled)
                    throw Error('physical CGB program or retained mute');
                  q('#instrument-play').click();await wait(()=>physicalSources.length===4);
                  if(physicalSources[3].output.output.gain.value!==0)throw Error('CGB mute');
                  q('#instrument-stop').click();q('#instrument-wave-open').click();
                  if(q('#cgb-id').value!=='2' || !q('#audio>details').open)throw Error('physical wave link');
                  return true;
                })()'''))
            finally:
                browser.close()


if __name__ == '__main__':
    unittest.main()
