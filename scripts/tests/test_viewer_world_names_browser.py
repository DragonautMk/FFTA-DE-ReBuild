"""Check source-gated world names, map selection, and text navigation."""
import copy
import hashlib
import json
import shutil
import sys
import tempfile
import time
import unittest
from pathlib import Path
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import view_rom_data
from viewer_browser_harness import Chrome


def fixture(work):
    dataset = work / 'dataset'
    dataset.mkdir()
    coordinates = {19: (13, 36), 17: (14, 26), 25: (13, 20), 26: (3, 20), 14: (19, 24), 2: (8, 11)}
    nodes = {'nodes': [{'index': index, 'x_tile': coordinates.get(index, (index, index))[0],
                       'y_tile': coordinates.get(index, (index, index))[1],
                       'links': [{'path_id': 0, 'neighbor': 0} for _ in range(4)]}
                      for index in range(30)]}
    glyphs = [{'index': i, 'width': 8, 'tile_words': [1] + [0] * 31} for i in (176, 203, 205, 1000)]
    first = [{'kind': 'glyph', 'index': 176}, {'kind': 'glyph', 'index': 203},
             {'kind': 'control_byte', 'value': 0x40}, {'kind': 'glyph', 'index': 205},
             {'kind': 'glyph', 'index': 1000}]
    texts = {'glyphs': glyphs, 'strings': [{'source_offset': 10, 'encoding': 'compact', 'units': first},
                                           {'source_offset': 20, 'encoding': 'compact',
                                            'units': [{'kind': 'glyph', 'index': 205}]}],
             'pointer_catalogs': [{'reader': 0x80165e8, 'languages': [
                 {'source_offset': 100, 'entries': [0] * 734 + [1] + [0] * 14},
                 {'source_offset': 200, 'entries': [0] * 725 + [1] + [0] * 4}]}], 'archive_catalogs': []}
    manifest = 'profile="europe"\nextractor_revision=1\ndecoder_revision=1\n'
    digests = []
    for rid, name, fmt, value in [
        ('gameplay.world-nodes', 'world-nodes.json', 'world-nodes-v1-json', nodes),
        ('text.catalogs', 'text-catalogs.json', 'text-catalogs-v1-json', texts),
    ]:
        content = json.dumps(value).encode(); (dataset / name).write_bytes(content)
        digest = hashlib.sha256(content).hexdigest(); digests.append(digest)
        manifest += (f'[[resources]]\nid="{rid}"\npath="{name}"\nformat="{fmt}"\nsha256="{digest}"\n')
    (dataset / 'manifest.toml').write_text(manifest)
    return dataset, digests


def generate_fixture(work, *, split=True):
    dataset, digests = fixture(work)
    with patch.object(view_rom_data, 'EUROPE_WORLD_NODES_SHA256', digests[0]), \
         patch.object(view_rom_data, 'EUROPE_TEXT_CATALOG_SHA256', digests[1]):
        index = view_rom_data.generate(dataset, work / 'view', split=split)
    return index, digests


def payload(page):
    return json.JSONDecoder().raw_decode(page.read_text().split('const data = ', 1)[1])[0]


class ObservedWorldNameProjectionTest(unittest.TestCase):
    def test_scoped_projection_and_identity_gates(self):
        with tempfile.TemporaryDirectory(prefix='viewer-world-names-projection-') as directory:
            index, digests = generate_fixture(Path(directory))
            parts = index.parent / 'index.html.parts'
            world, text = payload(parts / 'world-map.html'), payload(parts / 'text-pointers.html')
            name = world['world_location_names']
            self.assertEqual(name['locations'][0], {'node_index': 19, 'region_index': 7, 'name_index': 725,
                                                    'names': [[176, 203, [0x40], 205, 1000], [205]]})
            self.assertEqual([(row['node_index'], row['region_index'], row['name_index']) for row in name['locations']],
                             [(19, 7, 725), (17, 1, 719), (25, 2, 720), (26, 6, 724), (14, 11, 729), (2, 16, 734)])
            self.assertEqual(name['reader_index'], 0)
            self.assertNotIn('pointer_texts', world)
            for page in (index, parts / 'text.html', parts / 'audio.html'):
                self.assertNotIn('world_location_names', payload(page))
            with patch.object(view_rom_data, 'EUROPE_WORLD_NODES_SHA256', digests[0]), \
                 patch.object(view_rom_data, 'EUROPE_TEXT_CATALOG_SHA256', digests[1]):
                project = view_rom_data.project_observed_world_names
                nodes, texts = world['world_nodes'], text['pointer_texts']
                self.assertIsNone(project('usa', *digests, nodes, texts))
                self.assertIsNone(project('europe', 'wrong', digests[1], nodes, texts))
                self.assertIsNone(project('europe', digests[0], 'wrong', nodes, texts))
                self.assertIsNone(project('europe', *digests, None, texts))
                self.assertIsNone(project('europe', *digests, nodes, None))
                changed = copy.deepcopy(nodes); changed['nodes'][19]['x_tile'] += 1
                self.assertEqual([item['node_index'] for item in project('europe', *digests, changed, texts)['locations']],
                                 [17, 25, 26, 14, 2])
                changed = copy.deepcopy(nodes); changed['nodes'] = changed['nodes'][:15]
                self.assertEqual([item['node_index'] for item in project('europe', *digests, changed, texts)['locations']], [14, 2])
                changed = copy.deepcopy(texts); changed['catalogs'].append(changed['catalogs'][0])
                self.assertIsNone(project('europe', *digests, nodes, changed))
                changed = copy.deepcopy(texts); changed['catalogs'][0]['reader'] = 0
                self.assertIsNone(project('europe', *digests, nodes, changed))
                changed = copy.deepcopy(texts); changed['catalogs'][0]['languages'][0]['entries'] = [0]
                partial = project('europe', *digests, nodes, changed)
                self.assertEqual(partial['locations'][0]['names'], [None, [205]])


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class ObservedWorldNameBrowserTest(unittest.TestCase):
    def test_scoped_name_selection_and_source_link(self):
        with tempfile.TemporaryDirectory(prefix='viewer-world-names-browser-') as directory:
            work = Path(directory); index, _ = generate_fixture(work)
            browser = Chrome(work, index.parent / 'index.html.parts/world-map.html')
            try:
                self.assertTrue(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change'));
                  if(q('#world-observed-names').hidden || q('#world-observed-names').open ||
                     q('#world-name-rows').rows.length!==6 || q('#world-name-rows').rows[0].cells[1].textContent!=='Ab d' ||
                     document.querySelectorAll('#world-name-rows canvas').length!==5 || !q('#world-name-rows .text-gap') ||
                     !q('#world-selected-name').hidden || typeof data.pointer_texts!=='undefined') return false;
                  for(const [node,id] of [[17,719],[25,720],[26,724],[14,729],[2,734]]){
                    q('[aria-label="Select observed node '+node+'"]').click();
                    const row=[...q('#world-name-rows').rows].find(item=>item.cells[0].textContent===String(node));
                    if(q('#world-selected-name').hidden||!q('#world-detail').textContent.includes('node '+node)||
                       !row.querySelector('a').href.includes('job_entry='+id))return false;
                  }
                  q('[aria-label="Select observed node 19"]').click();
                  if(q('#world-selected-name').hidden || q('#world-selected-name').textContent!=='Observed name: Ab d' ||
                     !q('#world-detail').textContent.includes('node 19'))return false;
                  q('#world-name-language').value='1';change(q('#world-name-language'));
                  if(q('#world-selected-name').textContent!=='Observed name: d' ||
                     q('#world-name-rows').rows[0].cells[1].textContent!=='d')return false;
                  q('[aria-label="Node 0"]').click();
                  return q('#world-selected-name').hidden && q('#world-name-language').value==='1';
                })()'''))
                browser.evaluate("document.querySelector('#world>details').open=true; document.querySelector('#world-observed-names').open=true")
                hit = browser.evaluate('''(()=>{
                  const a=document.querySelector('#world-name-rows a');
                  for(let p=a.parentElement;p;p=p.parentElement)if(p.tagName==='DETAILS')p.open=true;
                  a.scrollIntoView({block:'center'});
                  const r=a.getClientRects()[0],x=r.left+r.width/2,y=r.top+r.height/2,e=document.elementFromPoint(x,y);
                  return {x,y,match:e===a||a.contains(e)};
                })()''')
                self.assertTrue(hit['match'], 'Name link is not clickable: ' + json.dumps(hit))
                point = {'x': hit['x'], 'y': hit['y']}
                browser.command('Input.dispatchMouseEvent', {'type': 'mouseMoved', **point})
                browser.command('Input.dispatchMouseEvent', {'type': 'mousePressed', **point, 'button': 'left', 'clickCount': 1})
                browser.command('Input.dispatchMouseEvent', {'type': 'mouseReleased', **point, 'button': 'left', 'clickCount': 1})
                end = time.monotonic() + 30
                while time.monotonic() < end:
                    try:
                        if browser.evaluate("document.readyState==='complete' && typeof data!=='undefined' && data.wiki.current==='text-pointers'"):
                            break
                    except (RuntimeError, OSError):
                        pass
                    time.sleep(.05)
                else:
                    self.fail('Observed name text page did not load')
                self.assertTrue(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s);
                  return q('#pointer-texts>details').open && q('#pointer-reader').value==='0' &&
                    q('#pointer-language').value==='1' && q('#pointer-entry').value==='725';
                })()'''))
            finally:
                browser.close()

    def test_single_page_source_link_and_unrecognized_dataset(self):
        with tempfile.TemporaryDirectory(prefix='viewer-world-names-single-') as directory:
            work = Path(directory); index, _ = generate_fixture(work, split=False)
            browser = Chrome(work, index)
            try:
                self.assertTrue(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s);
                  q('#world>details').open=true;q('#world-observed-names').open=true;
                  q('#world-name-rows a').click();
                  return q('#pointer-texts>details').open && q('#pointer-entry').value==='725';
                })()'''))
            finally:
                browser.close()
        with tempfile.TemporaryDirectory(prefix='viewer-world-names-unknown-') as directory:
            work = Path(directory); dataset, _ = fixture(work)
            index = view_rom_data.generate(dataset, work / 'view', split=True)
            browser = Chrome(work, index.parent / 'index.html.parts/world-map.html')
            try:
                self.assertTrue(browser.evaluate("!document.querySelector('#world').hidden && document.querySelector('#world-observed-names').hidden"))
            finally:
                browser.close()
