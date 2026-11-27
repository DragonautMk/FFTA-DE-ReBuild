"""Check source-faithful UI context templates and their offline preview."""
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
from test_viewer_scoped_browser import parse_page
from viewer_browser_harness import Chrome


def records():
    return [{'x_units8': 4 + index, 'y_units8': 3 + index,
             'opaque_02_03': [index, 255 - index], 'entry_count': 250 + index,
             'opaque_06_08': [index, 2, 3], 'flags': 8 if index == 6 else 1,
             'opaque_0a_13': list(range(index, index + 10))}
            for index in range(11)]


def raw_bytes(items):
    return b''.join(bytes([r['x_units8'], r['y_units8'], *r['opaque_02_03']])
                    + r['entry_count'].to_bytes(2, 'little')
                    + bytes([*r['opaque_06_08'], r['flags'], *r['opaque_0a_13']]) for r in items)


def fixture(work, *, profile='europe', invalid=False, known=True):
    dataset = work / 'dataset'; dataset.mkdir()
    catalog = {'records': records()}; digest = hashlib.sha256(raw_bytes(catalog['records'])).hexdigest()
    if invalid: catalog['records'][0]['opaque_0a_13'].pop()
    content = json.dumps(catalog).encode(); (dataset / 'ui.json').write_bytes(content)
    rom = view_rom_data.EUROPE_ROM_SHA256 if known else '0' * 64
    manifest = (f'profile="{profile}"\nextractor_revision=1\ndecoder_revision=1\nsource_rom_sha256="{rom}"\n'
                f'[[resources]]\nid="ui.context-templates"\npath="ui.json"\nformat="ui-context-templates-v1-json"\n'
                f'sha256="{hashlib.sha256(content).hexdigest()}"\n'
                f'[[resources.sources]]\nrole="ui-context-templates"\noffset=3755588\nlength=220\nsha256="{digest}"\n')
    (dataset / 'manifest.toml').write_text(manifest)
    with patch.object(view_rom_data, 'EUROPE_UI_CONTEXT_SHA256', digest):
        page = view_rom_data.generate(dataset, work / 'view', split=True)
    return page


def wait_for(browser, condition):
    deadline = time.monotonic() + 30
    while time.monotonic() < deadline:
        try:
            if browser.evaluate('document.readyState === "complete" && ' + condition): return
        except (RuntimeError, OSError):
            pass
        time.sleep(.05)
    raise AssertionError('Page did not reach expected state: ' + condition)


class UiContextProjectionTest(unittest.TestCase):
    def test_roundtrip_validation_and_identity_guard(self):
        items = records(); raw = raw_bytes(items); digest = hashlib.sha256(raw).hexdigest()
        source = {'offset': 0x394e44, 'length': len(raw), 'sha256': digest}
        with patch.object(view_rom_data, 'EUROPE_UI_CONTEXT_SHA256', digest):
            data = view_rom_data.project_ui_context_templates({'records':items}, source=source, profile='europe',
                                                              rom_sha256=view_rom_data.EUROPE_ROM_SHA256)
            self.assertEqual(data['records'], items)
            self.assertEqual(data['category_templates'], list(view_rom_data.EUROPE_UI_CONTEXT_CATEGORIES))
            self.assertEqual(raw_bytes(data['records']), raw)
            for profile, rom in [('usa', view_rom_data.EUROPE_ROM_SHA256), ('europe', '0' * 64)]:
                self.assertEqual(view_rom_data.project_ui_context_templates({'records':items}, source=source,
                                 profile=profile, rom_sha256=rom)['category_templates'], [])
            self.assertEqual(view_rom_data.project_ui_context_templates({'records':items}, profile='europe',
                             rom_sha256=view_rom_data.EUROPE_ROM_SHA256)['category_templates'], [])
        for mutate in (lambda d:d['records'].pop(), lambda d:d['records'][0]['opaque_02_03'].pop(),
                       lambda d:d['records'][0].__setitem__('entry_count',65536),
                       lambda d:d['records'][0].__setitem__('flags',True),
                       lambda d:d['records'][0]['opaque_0a_13'].__setitem__(0,-1)):
            bad = {'records':copy.deepcopy(items)}; mutate(bad)
            with self.subTest(mutation=mutate), self.assertRaises(ValueError):
                view_rom_data.project_ui_context_templates(bad)
        for bad_source in (dict(source, length=219), dict(source, sha256='0'*64), dict(source, offset=-1)):
            with self.subTest(source=bad_source), self.assertRaises(ValueError):
                view_rom_data.project_ui_context_templates({'records':items}, source=bad_source)

    def test_scope_and_invalid_input(self):
        for invalid in (False, True):
            with self.subTest(invalid=invalid), tempfile.TemporaryDirectory() as directory:
                page=fixture(Path(directory), invalid=invalid); parts=page.parent/(page.name+'.parts')
                home, interface, other = map(parse_page, (page, parts/'interface-ui.html', parts/'other.html'))
                row=home['resources'][0]
                self.assertEqual(row['viewer_scope'],'interface-ui')
                self.assertEqual(row['status'].startswith('Available:'), not invalid)
                self.assertEqual(bool(interface['ui_context_templates']), not invalid)
                self.assertNotIn('ui_context_templates', other)
                self.assertNotIn('ui_context_templates', home)


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class UiContextBrowserTest(unittest.TestCase):
    def test_navbar_plot_categories_and_bookmarked_selection(self):
        with tempfile.TemporaryDirectory(prefix='viewer-ui-context-') as directory:
            work=Path(directory); page=fixture(work); browser=Chrome(work,page)
            try:
                browser.click('#wiki-page-links a[href$="/interface-ui.html#ui-context-templates"]')
                wait_for(browser,"typeof data !== 'undefined' && data.wiki.current === 'interface-ui' && location.hash === '#ui-context-templates'")
                self.assertEqual(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change',{bubbles:true}));
                  const section=q('#ui-context-templates'), templates=q('#ui-context-template'), category=q('#ui-context-category');
                  if(section.hidden || !section.querySelector(':scope > details').open || templates.options.length!==11 ||
                     q('#ui-context-category-control').hidden || q('#ui-context-category-rows').rows.length!==15)throw Error('view');
                  for(let i=0;i<data.ui_context_templates.category_templates.length;i++){
                    category.value=String(i+1);change(category);
                    if(Number(templates.value)!==data.ui_context_templates.category_templates[i])throw Error('category '+i);
                  }
                  category.value='2';change(category);
                  if(templates.value!=='6' || !q('#ui-context-detail').textContent.includes('bit 3 set') ||
                     q('#ui-context-fields').rows.length!==7 || !q('#ui-context-source').textContent.includes('0x394EBC'))throw Error('fields');
                  templates.value='0';change(templates);if(category.value!=='')throw Error('manual selection');
                  const canvas=q('#ui-context-canvas'),rect=canvas.getBoundingClientRect();
                  canvas.dispatchEvent(new MouseEvent('click',{clientX:rect.left+(32/240)*rect.width,clientY:rect.top+(24/160)*rect.height}));
                  if(templates.value!=='0' || !canvas.getAttribute('aria-label').includes('32, 24'))throw Error('plot');
                  category.value='2';change(category);
                  return section.querySelector(':scope > details > p > a').href;
                })()''').split('#')[-1], 'ui-context-templates')
                bookmark=browser.evaluate("document.querySelector('#ui-context-templates > details > p > a').href")
                browser.command('Page.navigate', {'url':bookmark})
                wait_for(browser,"typeof data !== 'undefined' && data.wiki.current==='interface-ui' && document.querySelector('#ui-context-template').value==='6'")
                self.assertTrue(browser.evaluate("document.querySelector('#ui-context-category').value==='2' && document.querySelector('#ui-context-templates>details').open"))
            finally:
                browser.close()

    def test_unknown_source_has_no_category_mapping(self):
        with tempfile.TemporaryDirectory(prefix='viewer-ui-context-unknown-') as directory:
            work=Path(directory); page=fixture(work, known=False)
            browser=Chrome(work,page.parent/(page.name+'.parts/interface-ui.html'))
            try:
                self.assertTrue(browser.evaluate('''(()=>{const q=s=>document.querySelector(s);
                  return !q('#ui-context-templates').hidden && q('#ui-context-category-control').hidden &&
                    q('#ui-context-category-mapping').hidden && q('#ui-context-template').options.length===11;})()'''))
            finally:
                browser.close()
