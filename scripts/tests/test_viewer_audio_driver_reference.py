"""Check source-bound driver tables and their song-player links."""
import hashlib
import json
from pathlib import Path
import shutil
import struct
import tempfile
import unittest

from test_viewer_navbar_split_browser import wait_for
from test_viewer_pcm_echo_browser import sequence_fixture
from test_viewer_scoped_browser import prepared, parse_page, REPO
from view_rom_data import project_audio_driver_reference, project_audio_extended_reference, project_audio_sequences
from view_rom_data import viewer_section_updates, write_split_viewer
from viewer_browser_harness import Chrome


def span(offset, role, raw):
    return {'offset': offset, 'length': len(raw), 'role': role, 'sha256': hashlib.sha256(raw).hexdigest()}


def catalogs():
    sequence, sources = sequence_fixture()
    driver = sequence['driver']
    driver.update(clock_source_offset=0x370d04, command_source_offset=0x370a90,
                  commands=[0x08140001 + i * 2 for i in range(36)])
    sources.extend([span(0x370d04, 'audio/driver/clocks', bytes(driver['clocks'])),
                    span(0x370a90, 'audio/driver/commands', struct.pack('<36I', *driver['commands']))])
    entries = [{'selector': selector, 'callback': 0x08149001 + selector * 2} for selector in (*range(8), 10, 11)]
    extended = {'source_offset': 0x370d38, 'entries': entries}
    extra_sources = [span(0x370d38, 'audio/extended-dispatch/slots-0-7',
                          struct.pack('<8I', *(entry['callback'] for entry in entries[:8]))),
                     span(0x370d60, 'audio/extended-dispatch/slots-10-11',
                          struct.pack('<2I', *(entry['callback'] for entry in entries[8:])))]
    return sequence, sources, extended, extra_sources


def fixture(work):
    data, _ = prepared(work)
    sequence, sources, extended, extra_sources = catalogs()
    data['sequences'] = project_audio_sequences(sequence, echo_sources=sources)
    data['sequence_directory'] = {'songs': [{'id': 6, 'header_id': 1}, {'id': 5, 'header_id': 0}],
                                  'headers': [{'id': 0}, {'id': 1}]}
    reference = project_audio_driver_reference(sequence['driver'], sources)
    reference['extensions'] += project_audio_extended_reference(extended, extra_sources)
    reference['extensions'].sort(key=lambda row: row['selector'])
    data['audio_driver_reference'] = reference
    for identity, name in [('audio.m4a-extended-dispatch', 'audio-ext.json'), ('audio.m4a-sequences', 'sequence.json')]:
        data['resources'].append({'id': identity, 'path': name, 'format': 'fixture', 'description': 'Fixture',
                                  'status': 'Available', 'sources': []})
    index = write_split_viewer(data, work / 'reference', 'index.html', (REPO / 'view_rom_data.html').read_text())
    return index, index.parent / (index.name + '.parts/audio-driver.html')


class AudioDriverReferenceProjectionTest(unittest.TestCase):
    def test_each_driver_table_requires_its_own_matching_source(self):
        sequence, sources, _, _ = catalogs()
        reference = project_audio_driver_reference(sequence['driver'], sources)
        self.assertEqual(reference['clocks']['values'], sequence['driver']['clocks'])
        self.assertEqual(reference['commands']['values'], sequence['driver']['commands'])
        self.assertEqual([entry['selector'] for entry in reference['extensions']], [8, 9])
        sources[0]['sha256'] = '0' * 64
        limited = project_audio_driver_reference(sequence['driver'], sources)
        self.assertEqual(set(limited), {'clocks', 'commands'})
        sources.append(sources[1].copy())
        self.assertEqual(set(project_audio_driver_reference(sequence['driver'], sources)), {'commands'})

    def test_extension_word_order_shape_and_authentication(self):
        _, _, extended, sources = catalogs()
        projected = project_audio_extended_reference(extended, sources)
        self.assertEqual([entry['selector'] for entry in projected], [*range(8), 10, 11])
        self.assertEqual([entry['source_offset'] for entry in projected],
                         [0x370d38 + 4 * entry['selector'] for entry in projected])
        extended['entries'][0]['callback'] += 2
        with self.assertRaisesRegex(ValueError, 'source span'):
            project_audio_extended_reference(extended, sources)
        extended['entries'][0]['callback'] &= ~1
        with self.assertRaisesRegex(ValueError, 'selector or pointer'):
            project_audio_extended_reference(extended, sources)

    def test_scoped_reference_retains_only_small_tables(self):
        with tempfile.TemporaryDirectory(prefix='viewer-audio-driver-scope-') as directory:
            index, page = fixture(Path(directory))
            driver = parse_page(page)
            self.assertNotIn('audio_driver_reference', parse_page(index))
            self.assertNotIn('audio_driver_reference', parse_page(page.with_name('audio.html')))
            self.assertNotIn('audio', driver)
            self.assertNotIn('sequences', driver)
            self.assertNotIn('sequence_directory', driver)
            self.assertEqual(driver['audio_driver_reference']['echo_songs'], [{'id': 5, 'option': 1}])
            self.assertEqual({item['id'] for item in driver['resources']}, {'audio.m4a-sequences', 'audio.m4a-extended-dispatch'})


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class AudioDriverReferenceBrowserTest(unittest.TestCase):
    def test_reference_disclosures_and_exact_audition_route(self):
        with tempfile.TemporaryDirectory(prefix='viewer-audio-driver-browser-') as directory:
            work = Path(directory); index, page = fixture(work)
            browser = Chrome(work, index)
            try:
                href = browser.evaluate("document.querySelector('[data-category=\"Audio\"] a[href$=\"audio-driver.html#audio-driver-extensions\"]').href")
                browser.command('Page.navigate', {'url': href})
                wait_for(browser, "document.querySelector('#audio-driver-extensions > details').open")
                stamp = viewer_section_updates()['audio-driver']['audio-driver-clocks']['at']
                song = browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s);
                  const clocks=q('#audio-driver-clock-rows').rows, commands=q('#audio-driver-command-rows').rows;
                  const selectors=q('#audio-driver-extension-rows').rows;
                  if(clocks.length!==49||commands.length!==36||selectors.length!==12)throw Error('table size');
                  if(clocks[48].cells[2].textContent!=='0xB0'||clocks[48].cells[3].textContent!=='0xFF')throw Error('clock mapping');
                  if(commands[8].cells[4].textContent!=='Patched in RAM'||commands[30].cells[1].textContent!=='—')throw Error('command table');
                  if(selectors[8].cells[3].textContent!=='PCM echo level'||selectors[9].cells[3].textContent!=='PCM echo length')
                    throw Error('known extension names');
                  for(const id of ['audio-driver-clocks','audio-driver-commands','audio-driver-extensions']){
                    if(q('#'+id+'>details>summary time').dateTime!==STAMP)throw Error('badge '+id);
                  }
                  if(globalThis.__FFTA_SEQUENCE_INSTRUCTIONS__||globalThis.__FFTA_PCM_WAVES__)throw Error('eager audio resources');
                  return q('#audio-driver-echo-songs a').href;
                })()'''.replace('STAMP', json.dumps(stamp)))
                browser.command('Page.navigate', {'url': song})
                wait_for(browser, "document.querySelector('#sequence-song').selectedOptions[0].textContent==='5' && !document.querySelector('#sequence-play').disabled")
                self.assertTrue(browser.evaluate("document.querySelector('#sequence-driver-navigation a').href.endsWith('audio-driver.html#audio-driver-clocks')"))
            finally:
                browser.close()
