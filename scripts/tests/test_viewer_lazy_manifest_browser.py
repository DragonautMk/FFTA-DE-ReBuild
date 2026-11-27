"""Verify on-demand source inventories and their offline recovery."""
import hashlib
import json
import shutil
import tempfile
import unittest
from pathlib import Path

from test_viewer_scoped_browser import REPO, parse_page, prepared
from test_viewer_lazy_pcm_browser import wait_for
from view_rom_data import write_split_viewer
from viewer_browser_harness import Chrome


def fixture(work):
    data, index = prepared(work)
    sources = [{'offset': 0x1000 + i, 'length': i + 1, 'role': 'range-' + str(i),
                'sha256': f'{i:064x}'} for i in range(720)]
    sources[30]['role'] = '<svg onload=alert(1)>'
    common = {'path': 'fixture.json', 'format': 'json', 'description': 'Fixture', 'status': 'Available',
              'sha256': 'file-digest', 'dependencies': ['fixture.other'], 'sources': sources}
    data['resources'].append({'id': 'text.catalogs', **common})
    audio = next(r for r in data['resources'] if r['id'] == 'audio.m4a-tones')
    audio.update({'sources': sources, 'sha256': 'audio-digest', 'dependencies': []})
    page = write_split_viewer(data, index.parent, index.name, (REPO / 'view_rom_data.html').read_text())
    return page, data, sources


class LazyManifestAssetTest(unittest.TestCase):
    def test_shared_asset_preserves_source_ranges_and_existing_output(self):
        with tempfile.TemporaryDirectory(prefix='viewer-lazy-manifest-') as directory:
            index, data, sources = fixture(Path(directory))
            parts = index.parent / (index.name + '.parts')
            pointer = parse_page(parts / 'text-pointers.html')
            relative = parse_page(parts / 'text-relative.html')
            samples = parse_page(parts / 'audio-samples.html')
            def item(payload, identity):
                return next(row for row in payload['resources'] if row['id'] == identity)
            rows = [item(pointer, 'text.catalogs'), item(relative, 'text.catalogs'), item(samples, 'audio.m4a-tones')]
            self.assertTrue(all('sources' not in row and row['source_count'] == len(sources) for row in rows))
            self.assertEqual(rows[0]['sources_asset'], rows[1]['sources_asset'])
            self.assertEqual(rows[0]['sources_asset'], rows[2]['sources_asset'])
            asset = parts / rows[0]['sources_asset']['href']
            content = json.JSONDecoder().raw_decode(asset.read_text().split(' = ', 2)[-1])[0]
            self.assertEqual(content['sources'], sources)
            self.assertEqual(content['key'], rows[0]['sources_asset']['key'])
            encoded = json.dumps(sources, ensure_ascii=True, separators=(',', ':')).replace('<', '\\u003c')
            self.assertEqual(hashlib.sha256(encoded.encode()).hexdigest(), content['key'])
            previous = (parts / 'text-pointers.html').read_bytes()
            asset.write_text('changed by another owner')
            with self.assertRaisesRegex(ValueError, 'manifest source asset'):
                write_split_viewer(data, index.parent, index.name, (REPO / 'view_rom_data.html').read_text())
            self.assertEqual((parts / 'text-pointers.html').read_bytes(), previous)
            self.assertEqual(asset.read_text(), 'changed by another owner')


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class LazyManifestBrowserTest(unittest.TestCase):
    def test_open_filter_paginate_and_retry_missing_or_invalid_assets(self):
        with tempfile.TemporaryDirectory(prefix='viewer-lazy-manifest-browser-') as directory:
            work = Path(directory)
            index, _, _ = fixture(work)
            parts = index.parent / (index.name + '.parts')
            page = parts / 'text-pointers.html'
            row = next(r for r in parse_page(page)['resources'] if r['id'] == 'text.catalogs')
            asset = parts / row['sources_asset']['href']
            saved = asset.with_suffix('.saved')
            browser = Chrome(work, page)
            try:
                self.assertTrue(browser.evaluate('''(()=>{
                  const r=data.resources.find(r=>r.id==='text.catalogs');
                  return !('sources' in r) && typeof globalThis.__FFTA_MANIFEST_SOURCES__==='undefined' &&
                    !document.querySelector('[data-resource-id="text.catalogs"] details').open;
                })()'''))
                asset.rename(saved)
                browser.evaluate("document.querySelector('[data-resource-id=\"text.catalogs\"] details').open=true")
                wait_for(browser, "document.querySelector('[data-resource-id=\"text.catalogs\"] details [role=status]').textContent.includes('Cannot load')")
                self.assertTrue(browser.evaluate('''(()=>{const d=document.querySelector('[data-resource-id="text.catalogs"] details');
                  return d.textContent.includes('file-digest') && d.textContent.includes('fixture.other') &&
                    !d.querySelector('table') && !d.querySelector('button').hidden})()'''))
                saved.rename(asset)
                browser.evaluate("document.querySelector('[data-resource-id=\"text.catalogs\"] details button').click()")
                wait_for(browser, "document.querySelector('[data-resource-id=\"text.catalogs\"] tbody')?.rows.length===25")
                self.assertTrue(browser.evaluate('''(()=>{
                  const d=document.querySelector('[data-resource-id="text.catalogs"] details');
                  const button=t=>[...d.querySelectorAll('button')].find(b=>b.textContent===t);
                  const body=()=>d.querySelector('tbody'),status=()=>d.querySelector('[role=status]').textContent;
                  if(!status().includes('Page 1 of 29') || body().rows[0].cells[0].textContent!=='0x1000 (4096)')throw Error('first page');
                  button('Next').click();if(!status().includes('Page 2 of 29'))throw Error('next page');
                  const filter=d.querySelector('[aria-label="Filter source ranges"]');filter.value='0x101e';button('Filter sources').click();
                  if(body().rows.length!==1 || body().rows[0].cells[2].textContent!=='<svg onload=alert(1)>' || body().querySelector('svg'))throw Error('filter');
                  filter.value='';button('Filter sources').click();const p=d.querySelector('[aria-label="Source page"]');p.value='29';button('Go to page').click();
                  if(body().rows.length!==20 || !status().includes('Page 29 of 29'))throw Error('last page');
                  const r=data.resources.find(r=>r.id==='text.catalogs');
                  return !('sources' in r) && globalThis.__FFTA_MANIFEST_SOURCES__[r.sources_asset.key].sources.length===720;
                })()'''))
                # A valid script with an invalid count must be retried after repair.
                original = asset.read_text()
                key = row['sources_asset']['key']
                asset.write_text('globalThis.__FFTA_MANIFEST_SOURCES__ = {}; globalThis.__FFTA_MANIFEST_SOURCES__['
                                 + json.dumps(key) + '] = {key:' + json.dumps(key) + ',sources:[]};')
                browser.command('Page.navigate', {'url': page.as_uri()})
                wait_for(browser, "document.readyState==='complete' && typeof data!=='undefined'")
                browser.evaluate("document.querySelector('[data-resource-id=\"text.catalogs\"] details').open=true")
                wait_for(browser, "document.querySelector('[data-resource-id=\"text.catalogs\"] details [role=status]').textContent.includes('count does not match')")
                asset.write_text(original)
                browser.evaluate("document.querySelector('[data-resource-id=\"text.catalogs\"] details button').click()")
                wait_for(browser, "document.querySelector('[data-resource-id=\"text.catalogs\"] tbody')?.rows.length===25")
            finally:
                if saved.exists():
                    saved.rename(asset)
                browser.close()


if __name__ == '__main__':
    unittest.main()
