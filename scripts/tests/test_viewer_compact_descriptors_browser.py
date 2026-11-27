"""Browse physical descriptors and follow their bidirectional animation links."""
import json
import shutil
import tempfile
import unittest
from pathlib import Path

from test_viewer_physical_compact_browser import fixture
from view_rom_data import viewer_section_updates
from viewer_browser_harness import Chrome


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class CompactDescriptorsBrowserTest(unittest.TestCase):
    def test_filter_context_links_and_updated_badges(self):
        with tempfile.TemporaryDirectory(prefix='viewer-compact-descriptors-') as directory:
            work = Path(directory); index = fixture(work, descriptors=True, contexts=True)
            browser = Chrome(work, index.parent / (index.name + '.parts') / 'objects-physical.html')
            try:
                expected = viewer_section_updates()['objects-physical']
                self.assertTrue(browser.evaluate('''(()=>{
                  const changes=EXPECTED,q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change'));
                  const animation=q('#obj-animations > details > summary time'), descriptors=q('#physical-compact-descriptors > details > summary time');
                  if(!animation || animation.dateTime!==changes['obj-animations'].at || !animation.textContent.startsWith('Updated ') ||
                     !descriptors || descriptors.dateTime!==changes['physical-compact-descriptors'].at || !descriptors.textContent.startsWith('Added '))throw Error('badges');
                  if(q('#physical-compact-descriptors').hidden || q('#physical-descriptor-rows').rows.length!==2 ||
                     !q('#obj-animation-bank').options[0].textContent.includes('Example context') ||
                     !q('#obj-animation-sequence').options[0].textContent.includes('Example context'))throw Error('context labels');
                  const filter=q('#physical-descriptor-filter');filter.value='example context';filter.dispatchEvent(new Event('input'));
                  if(q('#physical-descriptor-rows').rows.length!==1 || q('#physical-descriptor-rows').rows[0].cells[4].textContent!=='Example context')throw Error('context filter');
                  q('#physical-descriptor-rows').rows[0].querySelector('a').click();
                  if(q('#obj-animation-bank').value!=='0' || q('#obj-animation-sequence').value!=='0' ||
                     !q('#obj-animation-observed-contexts').textContent.includes('Observed during: Example context'))throw Error('open animation');
                  const reverse=q('#obj-animation-physical-references a');if(!reverse)throw Error('missing reverse link');reverse.click();
                  if(q('#physical-descriptor-bank').value!=='0' || q('#physical-descriptor-selector').value!=='0' ||
                     q('#physical-descriptor-rows').rows.length!==1 || !q('#physical-compact-descriptors>details').open)throw Error('reverse link');
                  q('#physical-descriptor-selector').value='999999';q('#physical-descriptor-selector').dispatchEvent(new Event('input'));
                  return q('#physical-descriptor-rows').rows.length===0 && q('#physical-descriptor-page').textContent.includes('0 to 65535');
                })()'''.replace('EXPECTED', json.dumps(expected))))
            finally:
                browser.close()

    def test_unrecognized_sources_do_not_gain_names(self):
        with tempfile.TemporaryDirectory(prefix='viewer-compact-descriptors-unverified-') as directory:
            work = Path(directory); index = fixture(work, descriptors=True)
            browser = Chrome(work, index.parent / (index.name + '.parts') / 'objects-physical.html')
            try:
                self.assertTrue(browser.evaluate('''(()=>{const q=s=>document.querySelector(s);
                  return q('#obj-animation-observed-contexts').hidden && q('#physical-descriptor-rows').rows[0].cells[4].textContent==='—' &&
                    !q('#obj-animation-bank').options[0].textContent.includes('observed:');})()'''))
            finally:
                browser.close()


if __name__ == '__main__':
    unittest.main()
