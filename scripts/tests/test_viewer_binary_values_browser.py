"""Exercise bounded, explicit binary interpretations in offline inspectors."""
import shutil
import tempfile
import unittest
from pathlib import Path

from test_viewer_resource_pages_browser import fixture, page_data
from view_rom_data import generate
from viewer_browser_harness import Chrome


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class BinaryValuesBrowserTest(unittest.TestCase):
    def test_numeric_widths_offsets_colors_bounds_and_file_replacement(self):
        with tempfile.TemporaryDirectory(prefix='viewer-binary-values-') as directory:
            work = Path(directory); dataset, _ = fixture(work)
            index = generate(dataset, work/'out', 'wiki.html', split=True)
            browser = Chrome(work, index)
            try:
                self.assertTrue(browser.evaluate('''(async()=>{
                  const q=s=>document.querySelector(s), change=e=>e.dispatchEvent(new Event('change',{bubbles:true}));
                  const wait=async f=>{const end=Date.now()+8000;while(!f()){if(Date.now()>end)throw Error('timeout');await new Promise(r=>setTimeout(r,10));}};
                  const rows=()=>[...q('#inspect-values-rows').rows],cells=i=>rows()[i]?[...rows()[i].cells].map(c=>c.textContent.trim()):[];
                  const select=(bytes,name)=>{const files=new DataTransfer();files.items.add(new File([bytes],name));
                    q('#inspect-file').files=files.files;change(q('#inspect-file'));};
                  const ready=()=>!q('#inspect-values-status').textContent.includes('Reading');
                  const data=new Uint8Array(139);data.set([0,0,0,128,31,0,224,3,0,124,255]);
                  select(data,'numbers.bin');await wait(()=>q('#inspect-status').textContent.includes('139'));
                  if(rows().length||q('#inspect-values-go').disabled)throw Error('lazy state');
                  q('#inspect-values').open=true;await wait(()=>rows().length===64&&ready());
                  if(q('#inspect-values-added').dateTime!=='2026-11-27T23:57:52Z'||q('#inspect-values-endian').disabled!==true||
                     cells(3)[2]!=='128'||cells(3)[3]!=='-128'||!q('#inspect-values-prev').disabled||q('#inspect-values-next').disabled)
                    throw Error('byte values or pagination');
                  q('#inspect-values-next').click();await wait(()=>cells(0)[0]==='0x00000040');
                  q('#inspect-values-next').click();await wait(()=>cells(0)[0]==='0x00000080');
                  if(rows().length!==11||!q('#inspect-values-next').disabled)throw Error('last page');
                  q('#inspect-values-prev').click();await wait(()=>cells(0)[0]==='0x00000040');
                  q('#inspect-values-offset').value='0x0';q('#inspect-values-go').click();await wait(()=>cells(0)[0]==='0x00000000');
                  q('#inspect-values-width').value='2';q('#inspect-values-color').checked=true;change(q('#inspect-values-width'));
                  await wait(()=>rows().length===64&&cells(1)[2]==='32768');
                  if(cells(1)[3]!=='-32768'||cells(2)[4]!=='#FF0000'||cells(3)[4]!=='#00FF00'||cells(4)[4]!=='#0000FF'||
                     q('#inspect-values-color-heading').hidden||q('#inspect-values-endian').disabled)throw Error('little endian or BGR555');
                  q('#inspect-values-endian').value='big';change(q('#inspect-values-endian'));
                  await wait(()=>cells(1)[2]==='128');if(cells(2)[2]!=='7936')throw Error('big endian halfword');
                  q('#inspect-values-width').value='4';q('#inspect-values-endian').value='little';change(q('#inspect-values-width'));
                  await wait(()=>cells(0)[2]==='2147483648');
                  if(cells(0)[3]!=='-2147483648'||!q('#inspect-values-color-heading').hidden)throw Error('32-bit signed');
                  q('#inspect-values-endian').value='big';change(q('#inspect-values-endian'));await wait(()=>cells(0)[2]==='128');
                  q('#inspect-values-offset').value='0x88';q('#inspect-values-go').click();await wait(()=>cells(0)[0]==='0x00000088');
                  if(rows().length!==1||cells(0)[1]!=='00 00 00'||cells(0)[2]!=='Incomplete'||
                     !q('#inspect-values-status').textContent.includes('incomplete'))throw Error('partial value');
                  for(const bad of ['-1','0xZZ','139','9007199254740992']){
                    q('#inspect-values-offset').value=bad;q('#inspect-values-go').click();
                    if(!q('#inspect-values-status').textContent.includes('Enter a byte offset'))throw Error('invalid '+bad);
                  }
                  select(new Uint8Array(0),'empty.bin');await wait(()=>q('#inspect-values-status').textContent.includes('empty file'));
                  if(rows().length||!q('#inspect-values-prev').disabled||!q('#inspect-values-next').disabled)throw Error('empty file');
                  let release;const pending=new Promise(resolve=>{release=resolve});const old=new File([new Uint8Array(20000)],'old.bin');
                  const slice=old.slice.bind(old);old.slice=(start,end)=>{const blob=slice(start,end);
                    return end-start===256?{arrayBuffer:async()=>{await pending;return blob.arrayBuffer();}}:blob;};
                  q('#inspect-values-width').value='4';change(q('#inspect-values-width'));openIndexedFile(old,'old.bin');
                  await wait(()=>q('#inspect-values-status').textContent.includes('Reading'));
                  select(new Uint8Array([1,2,3,4]),'new.bin');await wait(()=>rows().length===1&&cells(0)[1]==='01 02 03 04');
                  release();await new Promise(r=>setTimeout(r,30));
                  if(!q('#inspect-values-status').textContent.includes('new.bin')||cells(0)[1]!=='01 02 03 04')throw Error('stale read');
                  const length=9*1024*1024;
                  const large=new File([new Uint8Array(length),new Uint8Array([255,127,128])],'large.bin');
                  openIndexedFile(large,'large.bin');q('#inspect-values-width').value='2';q('#inspect-values-endian').value='little';
                  change(q('#inspect-values-width'));q('#inspect-values-offset').value=String(length);q('#inspect-values-go').click();
                  await wait(()=>rows().length===2&&cells(0)[1]==='FF 7F');
                  return cells(0)[2]==='32767'&&cells(1)[2]==='Incomplete'&&q('#inspect-values-status').textContent.includes('incomplete');
                })()'''))
            finally:
                browser.close()

    def test_resource_page_shows_dated_interpreter(self):
        with tempfile.TemporaryDirectory(prefix='viewer-binary-page-') as directory:
            work = Path(directory); dataset, _ = fixture(work)
            index = generate(dataset, work/'out', 'wiki.html', split=True)
            binary = next(item for item in page_data(index)['resources'] if item['id']=='audio.binary')
            page = index.parent/(index.name+'.parts')/'files'/binary['embedded_view']
            browser = Chrome(work, index)
            try:
                browser.command('Page.navigate', {'url': page.as_uri()})
                self.assertTrue(browser.evaluate('''(async()=>{const q=s=>document.querySelector(s);
                  const wait=async f=>{const end=Date.now()+5000;while(!f()){if(Date.now()>end)throw Error('timeout');await new Promise(r=>setTimeout(r,10));}};
                  await wait(()=>q('#inspect-status').textContent.includes('audio.bin'));
                  q('#inspect-values').open=true;await wait(()=>q('#inspect-values-rows').rows.length===4);
                  const row=q('#inspect-values-rows').rows[3];
                  return q('#inspect-values-added').dateTime==='2026-11-27T23:57:52Z'&&row.cells[2].textContent==='255'&&row.cells[3].textContent==='-1';})()'''))
            finally:
                browser.close()


if __name__ == '__main__':
    unittest.main()
