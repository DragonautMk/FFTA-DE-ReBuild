"""Check isolated, authenticated offline pages for extracted files."""
import hashlib
import json
import shutil
import sys
import tempfile
import time
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from view_rom_data import generate, RESOURCE_PAGE_LIMIT
from viewer_browser_harness import Chrome


def fixture(work):
    dataset = work / 'dataset'
    dataset.mkdir()
    payloads = {
        'gameplay.example': ('nested/example #.json', b'{"records":[{"id":1,"name":"M\xc3\xb9","payload":"</script><script>globalThis.xss=1</script>"},{"id":2,"name":"Second"}]}'),
        'audio.binary': ('audio.bin', bytes([0, 127, 128, 255])),
        'engine.invalid': ('invalid.json', b'{bad'),
        'engine.large': ('large.json', b' ' * (RESOURCE_PAGE_LIMIT + 1)),
        'engine.mismatch': ('mismatch.json', b'{}'),
    }
    manifest = ['profile="test"\nextractor_revision=1\ndecoder_revision=1\n']
    for resource_id, (name, content) in payloads.items():
        path = dataset / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(content)
        sha = '0' * 64 if resource_id == 'engine.mismatch' else hashlib.sha256(content).hexdigest()
        manifest.append('[[resources]]\nid=' + json.dumps(resource_id) + '\npath=' + json.dumps(name)
                        + '\nformat="synthetic"\nsha256="' + sha + '"\n')
    manifest.append('[[resources]]\nid="engine.missing"\npath="missing.json"\nformat="synthetic"\nsha256="' + '0' * 64 + '"\n')
    (dataset / 'manifest.toml').write_text(''.join(manifest))
    return dataset, payloads


def page_data(path):
    return json.JSONDecoder().raw_decode(path.read_text().split('const data = ', 1)[1])[0]


class ResourcePageGenerationTest(unittest.TestCase):
    def test_authentication_isolation_limits_and_ownership(self):
        with tempfile.TemporaryDirectory(prefix='viewer-resource-pages-') as temp:
            work = Path(temp)
            dataset, payloads = fixture(work)
            index = generate(dataset, work / 'out', 'wiki #.html', split=True)
            parts = index.parent / (index.name + '.parts')
            index_data = page_data(index)
            rows = {row['id']: row for row in index_data['resources']}
            self.assertEqual({key for key, row in rows.items() if 'embedded_view' in row},
                             {'gameplay.example', 'audio.binary', 'engine.invalid'})
            self.assertNotIn('globalThis.xss=1', index.read_text())
            self.assertNotIn('globalThis.xss=1', (parts / 'world.html').read_text())
            self.assertEqual(rows['engine.mismatch']['status'], 'SHA-256 mismatch')
            self.assertEqual(rows['engine.missing']['status'], 'File missing')
            files = parts / 'files'
            marker = json.loads((files / '.ffta-viewer-resources.json').read_text())
            self.assertEqual(len(marker['files']), 3)
            self.assertNotIn('</script><script>globalThis.xss', (files / rows['gameplay.example']['embedded_view']).read_text())
            self.assertNotIn('tree-load-large', (files / rows['gameplay.example']['embedded_view']).read_text())
            self.assertEqual((dataset / payloads['gameplay.example'][0]).read_bytes(), payloads['gameplay.example'][1])
            generate(dataset, work / 'out', 'wiki #.html', split=True)
            page = files / rows['gameplay.example']['embedded_view']
            page.write_text('user modified this page')
            with self.assertRaisesRegex(ValueError, 'unchanged generated file'):
                generate(dataset, work / 'out', 'wiki #.html', split=True)
            self.assertEqual(page.read_text(), 'user modified this page')

            unowned = work / 'unowned'
            generate(dataset, unowned, 'wiki.html', split=True)
            foreign = unowned / 'wiki.html.parts/files'
            (foreign / '.ffta-viewer-resources.json').unlink()
            with self.assertRaisesRegex(ValueError, 'not owned'):
                generate(dataset, unowned, 'wiki.html', split=True)
            self.assertTrue(foreign.is_dir())


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class ResourcePageBrowserTest(unittest.TestCase):
    def test_offline_links_tree_and_raw_binary(self):
        with tempfile.TemporaryDirectory(prefix='viewer-resource-browser-') as temp:
            work = Path(temp)
            dataset, _ = fixture(work)
            index = generate(dataset, work / 'out', 'wiki #.html', split=True)
            browser = Chrome(work, index)
            try:
                self.assertTrue(browser.evaluate('''(()=>{
                  const row=document.querySelector('[data-resource-id="gameplay.example"]');
                  const link=[...row.querySelectorAll('a')].find(a=>a.textContent==='Browse extracted file');
                  return !!link && link.href.includes('/files/') && !document.querySelector('[data-resource-id="engine.large"] a[href*="/files/"]');
                })()'''))
                browser.click('[data-resource-id="gameplay.example"] a[href*="/files/"]')
                end = time.monotonic() + 30
                while time.monotonic() < end:
                    try:
                        if browser.evaluate('Boolean(document.querySelector("#resource-title")) && !document.querySelector("#tree").hidden'):
                            break
                    except (RuntimeError, OSError):
                        pass
                    time.sleep(.05)
                else:
                    self.fail('Resource page did not load')
                self.assertTrue(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s), row=[...q('#tree-rows').rows].find(r=>r.cells[0].textContent==='records');
                  if (!row || !q('#file-inspector > details').open || globalThis.xss ||
                      !q('#resource-index').href.endsWith('/wiki%20%23.html') || !q('#resource-scope').href.endsWith('/world.html')) return false;
                  row.querySelector('button').click();
                  q('#tree-table').checked=true;q('#tree-table').dispatchEvent(new Event('change'));
                  return q('#tree-rows').rows.length===2 && q('#tree-rows').textContent.includes('Mù') &&
                    q('#inspect-content').textContent.includes('</script>') && q('#tree-filter-control').hidden===false;
                })()'''))
                parts = index.parent / (index.name + '.parts')
                binary = next(r for r in page_data(index)['resources'] if r['id'] == 'audio.binary')
                browser.command('Page.navigate', {'url': (parts / 'files' / binary['embedded_view']).as_uri()})
                end = time.monotonic() + 30
                while time.monotonic() < end:
                    try:
                        if browser.evaluate('Boolean(document.querySelector("#resource-title")) && document.querySelector("#resource-title").textContent === "audio.binary" && document.querySelector("#inspect-content").textContent.includes("FF")'):
                            break
                    except (RuntimeError, OSError):
                        pass
                    time.sleep(.05)
                else:
                    self.fail('Binary resource page did not load')
                self.assertEqual(browser.evaluate('document.querySelector("#inspect-mode").value'), 'hex')
            finally:
                browser.close()


if __name__ == '__main__':
    unittest.main()
