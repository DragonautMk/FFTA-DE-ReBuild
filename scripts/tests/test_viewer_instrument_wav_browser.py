"""Check playable WAV exports for resolved PCM and CGB instruments."""
import base64
import io
import shutil
import tempfile
import unittest
import wave
from pathlib import Path

from test_viewer_instruments_browser import fixture
from viewer_browser_harness import Chrome


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class ViewerInstrumentWavTest(unittest.TestCase):
    def test_wav_resampling_loops_and_controls(self):
        with tempfile.TemporaryDirectory(prefix='viewer-instrument-wav-') as directory:
            work = Path(directory)
            browser = Chrome(work, fixture(work))
            try:
                result = browser.evaluate('''(async()=>{
                  const q=s=>document.querySelector(s), change=e=>e.dispatchEvent(new Event('change'));
                  function data(bytes){
                    const view=new DataView(bytes.buffer,bytes.byteOffset,bytes.byteLength);
                    if(String.fromCharCode(...bytes.slice(0,4))!=='RIFF' || String.fromCharCode(...bytes.slice(8,12))!=='WAVE' ||
                       view.getUint32(24,true)!==32000 || view.getUint16(34,true)!==8 ||
                       view.getUint32(4,true)!==bytes.length-8)throw Error('WAV header');
                    return [...bytes.slice(44,44+view.getUint32(40,true))];
                  }
                  let sample={kind:'pcm',frequency:32000,wave:{flags:0,loop_start:0}};
                  let values=data(instrumentWaveBytes(sample,[-1,0,.5,1],.1));
                  if(values.join(',')!=='0,128,192,255')throw Error('PCM one-shot');
                  sample.wave={flags:0x4000,loop_start:1};values=data(instrumentWaveBytes(sample,[-1,0,1],.1));
                  if(values.length!==3200 || values.slice(0,7).join(',')!=='0,128,255,128,255,128,255')throw Error('PCM loop');
                  sample={kind:'pcm-fixed',frequency:16000,wave:{flags:0,loop_start:0}};
                  values=data(instrumentWaveBytes(sample,[-1,1],.1));
                  if(values.join(',')!=='0,128,255,255')throw Error('fixed PCM interpolation');
                  sample={kind:'cgb-square',frequency:1000};values=data(instrumentWaveBytes(sample,[-1,1],.1));
                  if(values.length!==3200 || [0,8,16,24,32].map(i=>values[i]).join(',')!=='0,128,255,128,0')throw Error('CGB cycle');
                  let refused=false;try{instrumentWaveBytes(sample,[-1,1],.01)}catch(_){refused=true}
                  if(!refused)throw Error('duration limit');
                  window.exported=[];URL.createObjectURL=blob=>{exported.push(blob);return 'blob:instrument-test'};
                  URL.revokeObjectURL=()=>{};HTMLAnchorElement.prototype.click=function(){window.exportedName=this.download};
                  async function download(program){
                    q('#instrument-program').value=String(program);change(q('#instrument-program'));
                    q('#instrument-download').click();
                    return data(new Uint8Array(await exported.at(-1).arrayBuffer()));
                  }
                  q('#instrument-duration').value='.1';values=await download(0);
                  if(values.length!==3200 || !exportedName.includes('tone-0-key-60'))throw Error('PCM download');
                  q('#instrument-duration').value='1';values=await download(7);
                  if(values.length!==Math.ceil(2205/15768*32000))throw Error('short fixed PCM download');
                  q('#sequence-mute-cgb').checked=true;change(q('#sequence-mute-cgb'));
                  q('#instrument-duration').value='.1';values=await download(2);
                  if(values.length!==3200 || Math.min(...values)!==0 || Math.max(...values)!==255 ||
                     !q('#instrument-mute-cgb').checked || !q('#instrument-status').textContent.includes('mute does not apply'))throw Error('square download');
                  values=await download(3);
                  if(values.length!==3200 || Math.min(...values)>15 || Math.max(...values)<240)throw Error('CGB wave download');
                  q('#instrument-program').value='5';change(q('#instrument-program'));
                  if(!q('#instrument-download').disabled)throw Error('unsupported tone');
                  q('#instrument-program').value='1';change(q('#instrument-program'));
                  q('#instrument-fixed-rate').value='999';q('#instrument-fixed-rate').dispatchEvent(new Event('input'));
                  if(!q('#instrument-download').disabled)throw Error('invalid fixed rate');
                  q('#instrument-program').value='0';change(q('#instrument-program'));
                  const before=exported.length;q('#instrument-duration').value='.01';q('#instrument-download').click();
                  if(exported.length!==before || !q('#instrument-status').textContent.includes('duration'))throw Error('invalid duration');
                  return 'PASS';
                })()''')
                self.assertEqual(result, 'PASS')
                for index in range(4):
                    encoded = browser.evaluate('''(async()=>{
                      const bytes=new Uint8Array(await exported[INDEX].arrayBuffer());
                      let binary='';for(const byte of bytes)binary+=String.fromCharCode(byte);
                      return btoa(binary);
                    })()'''.replace('INDEX', str(index)))
                    with wave.open(io.BytesIO(base64.b64decode(encoded)), 'rb') as exported_wave:
                        self.assertEqual((exported_wave.getnchannels(), exported_wave.getsampwidth(),
                                          exported_wave.getframerate()), (1, 1, 32000))
                        self.assertGreater(len(exported_wave.readframes(exported_wave.getnframes())), 0)
            finally:
                browser.close()
