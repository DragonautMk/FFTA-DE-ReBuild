"""Check wrapped wiki menus remain visible and clickable above the navbar."""

import json
from pathlib import Path
import re
import shutil
import tempfile
import unittest

from viewer_browser_harness import Chrome


ROOT = Path(__file__).resolve().parents[2]


def fixture(work, template_name='view_rom_data.html'):
    template = (ROOT / template_name).read_text(encoding='utf-8')
    style = re.search(r'<style>(.*?)</style>', template, re.S).group(1)
    style = style.replace('/* WIKI NAVIGATION CSS */',
                          (ROOT / 'scripts/viewer_navigation.css').read_text(encoding='utf-8'))
    change = {'kind': 'updated', 'at': '2026-11-28T00:18:00+01:00'}
    page_ids = [
        ('index', 'Resource index'),
        ('audio', 'Soundtracks'),
        ('audio-samples', 'Waveforms and instruments'),
        ('audio-noise', 'CGB noise'),
        ('actors', 'Actors'),
        ('indexed-graphics', 'Indexed graphics'),
        ('interface', 'Interface'),
        ('text', 'Text'),
        ('world', 'World'),
    ]
    pages = [{'id': identifier, 'label': label, 'href': identifier + '.html',
              'sections': [{'id': 'section', 'label': label + ' section', 'change': change}]}
             for identifier, label in page_ids]
    data = {'wiki': {'current': 'index', 'name_search': True, 'pages': pages}}
    navigation = (ROOT / 'scripts/viewer_navigation.js').read_text(encoding='utf-8')
    page = work / 'index.html'
    page.write_text(
        '<!doctype html><html><head><meta name="viewport" content="width=device-width,initial-scale=1">'
        '<style>' + style + '</style></head><body data-index-ready="true">'
        '<nav id="wiki-pages" class="wiki-navbar" hidden><div id="wiki-page-links"></div></nav>'
        '<h1>Fixture</h1><script>const data = ' + json.dumps(data) + ';\n' +
        navigation + '</script></body></html>', encoding='utf-8')
    return page


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class ViewerNavbarStackingBrowserTest(unittest.TestCase):
    def test_audio_menu_is_on_top_of_later_navigation_rows(self):
        with tempfile.TemporaryDirectory(prefix='viewer-navbar-stacking-') as directory:
            work = Path(directory)
            browser = Chrome(work, fixture(work))
            try:
                browser.command('Emulation.setDeviceMetricsOverride',
                                {'width': 900, 'height': 1000, 'deviceScaleFactor': 1, 'mobile': False})
                result = browser.evaluate('''(() => {
                  const nav = document.querySelector('#wiki-pages');
                  const audio = nav.querySelector('[data-category="Audio"]');
                  const before = nav.getBoundingClientRect().height;
                  audio.open = true;
                  const menu = audio.querySelector('.wiki-menu');
                  const menuRect = menu.getBoundingClientRect();
                  const summaries = [...nav.querySelectorAll('.wiki-page-category > summary')]
                    .filter(summary => summary.parentElement !== audio);
                  const overlaps = summaries.map(summary => {
                    const rect = summary.getBoundingClientRect();
                    const left = Math.max(rect.left, menuRect.left, 0);
                    const right = Math.min(rect.right, menuRect.right, innerWidth);
                    const top = Math.max(rect.top, menuRect.top, 0);
                    const bottom = Math.min(rect.bottom, menuRect.bottom, innerHeight);
                    if (right - left < 2 || bottom - top < 2) return null;
                    const hit = document.elementFromPoint((left + right) / 2, (top + bottom) / 2);
                    return {label: summary.textContent, covered: menu.contains(hit)};
                  }).filter(Boolean);
                  return {overlaps, before, after: nav.getBoundingClientRect().height,
                          visible: menuRect.width > 0 && menuRect.height > 0};
                })()''')
                self.assertTrue(result['visible'])
                self.assertTrue(result['overlaps'], 'fixture must create a wrapped navigation row')
                self.assertTrue(all(item['covered'] for item in result['overlaps']), result)
                self.assertAlmostEqual(result['before'], result['after'], delta=1)
            finally:
                browser.close()

    def test_all_categories_fit_across_wrapped_desktop_widths(self):
        with tempfile.TemporaryDirectory(prefix='viewer-navbar-widths-') as directory:
            work = Path(directory)
            browser = Chrome(work, fixture(work))
            try:
                for width in (761, 800, 861, 900, 1341, 1401):
                    with self.subTest(width=width):
                        browser.command('Emulation.setDeviceMetricsOverride',
                                        {'width': width, 'height': 1000, 'deviceScaleFactor': 1,
                                         'mobile': False})
                        clipped = browser.evaluate('''(async () => {
                          const clipped = [];
                          for (const group of document.querySelectorAll('#wiki-page-links > details')) {
                            await new Promise(resolve => {
                              group.addEventListener('toggle', resolve, {once: true});
                              group.open = true;
                            });
                            const box = group.querySelector('.wiki-menu').getBoundingClientRect();
                            if (box.left < 0 || box.right > innerWidth)
                              clipped.push({category: group.dataset.category, left: box.left,
                                            right: box.right, width: innerWidth});
                            await new Promise(resolve => {
                              group.addEventListener('toggle', resolve, {once: true});
                              group.open = false;
                            });
                          }
                          return clipped;
                        })()''')
                        self.assertEqual(clipped, [])

                browser.evaluate('''(async () => {
                  const graphics = document.querySelector('[data-category="Graphics"]');
                  await new Promise(resolve => {
                    graphics.addEventListener('toggle', resolve, {once: true});
                    graphics.open = true;
                  });
                })()''')
                browser.command('Emulation.setDeviceMetricsOverride',
                                {'width': 800, 'height': 1000, 'deviceScaleFactor': 1,
                                 'mobile': False})
                result = browser.evaluate('''(async () => {
                  await new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve)));
                  const box = document.querySelector('[data-category="Graphics"] .wiki-menu').getBoundingClientRect();
                  return {left: box.left, right: box.right, width: innerWidth};
                })()''')
                self.assertGreaterEqual(result['left'], 0, result)
                self.assertLessEqual(result['right'], result['width'], result)
            finally:
                browser.close()

    def test_index_template_keeps_menu_above_wrapped_rows(self):
        with tempfile.TemporaryDirectory(prefix='viewer-navbar-index-') as directory:
            work = Path(directory)
            browser = Chrome(work, fixture(work, 'scripts/viewer_index.html'))
            try:
                browser.command('Emulation.setDeviceMetricsOverride',
                                {'width': 900, 'height': 1000, 'deviceScaleFactor': 1,
                                 'mobile': False})
                result = browser.evaluate('''(() => {
                  const nav = document.querySelector('#wiki-pages');
                  const audio = nav.querySelector('[data-category="Audio"]');
                  const before = nav.getBoundingClientRect().height;
                  audio.open = true;
                  const menu = audio.querySelector('.wiki-menu');
                  const menuRect = menu.getBoundingClientRect();
                  const summaries = [...nav.querySelectorAll('.wiki-page-category > summary')]
                    .filter(summary => summary.parentElement !== audio);
                  const covered = summaries.flatMap(summary => {
                    const rect = summary.getBoundingClientRect();
                    const left = Math.max(rect.left, menuRect.left);
                    const right = Math.min(rect.right, menuRect.right);
                    const top = Math.max(rect.top, menuRect.top);
                    const bottom = Math.min(rect.bottom, menuRect.bottom);
                    if (right - left < 2 || bottom - top < 2) return [];
                    return [menu.contains(document.elementFromPoint((left + right) / 2,
                                                                  (top + bottom) / 2))];
                  });
                  return {covered, before, after: nav.getBoundingClientRect().height};
                })()''')
                self.assertTrue(result['covered'])
                self.assertTrue(all(result['covered']), result)
                self.assertAlmostEqual(result['before'], result['after'], delta=1)
            finally:
                browser.close()

    def test_mobile_menu_and_keyboard_close_remain_usable(self):
        with tempfile.TemporaryDirectory(prefix='viewer-navbar-mobile-') as directory:
            work = Path(directory)
            browser = Chrome(work, fixture(work))
            try:
                browser.command('Emulation.setDeviceMetricsOverride',
                                {'width': 375, 'height': 850, 'deviceScaleFactor': 1, 'mobile': True})
                result = browser.evaluate('''(() => {
                  const audio = document.querySelector('[data-category="Audio"]');
                  const summary = audio.querySelector(':scope > summary');
                  summary.focus(); summary.click();
                  const menu = audio.querySelector('.wiki-menu');
                  const box = menu.getBoundingClientRect();
                  const link = menu.querySelector('a');
                  const r = link.getBoundingClientRect();
                  const hit = document.elementFromPoint((r.left + r.right) / 2, (r.top + r.bottom) / 2);
                  return {open: audio.open, left: box.left, right: box.right, width: innerWidth,
                          hit: link.contains(hit), overflow: document.documentElement.scrollWidth};
                })()''')
                self.assertTrue(result['open'])
                self.assertGreaterEqual(result['left'], 0)
                self.assertLessEqual(result['right'], result['width'])
                self.assertLessEqual(result['overflow'], result['width'] + 2)
                self.assertTrue(result['hit'], result)
                browser.command('Input.dispatchKeyEvent', {'type': 'keyDown', 'key': 'Escape',
                                                           'code': 'Escape', 'windowsVirtualKeyCode': 27})
                browser.command('Input.dispatchKeyEvent', {'type': 'keyUp', 'key': 'Escape',
                                                           'code': 'Escape', 'windowsVirtualKeyCode': 27})
                self.assertTrue(browser.evaluate('''(() => {
                  const audio = document.querySelector('[data-category="Audio"]');
                  return !audio.open && document.activeElement === audio.querySelector(':scope > summary');
                })()'''))
            finally:
                browser.close()

    def test_short_mobile_view_keeps_menu_and_links_on_screen(self):
        with tempfile.TemporaryDirectory(prefix='viewer-navbar-short-') as directory:
            work = Path(directory)
            browser = Chrome(work, fixture(work))
            try:
                browser.command('Emulation.setDeviceMetricsOverride',
                                {'width': 320, 'height': 480, 'deviceScaleFactor': 1,
                                 'mobile': True})
                result = browser.evaluate('''(async () => {
                  const audio = document.querySelector('[data-category="Audio"]');
                  await new Promise(resolve => {
                    audio.addEventListener('toggle', resolve, {once: true});
                    audio.open = true;
                  });
                  const menu = audio.querySelector('.wiki-menu');
                  const box = menu.getBoundingClientRect();
                  const link = menu.querySelector('a');
                  const r = link.getBoundingClientRect();
                  const hit = document.elementFromPoint((r.left + r.right) / 2, (r.top + r.bottom) / 2);
                  return {top: box.top, bottom: box.bottom, height: innerHeight, hit: link.contains(hit)};
                })()''')
                self.assertGreaterEqual(result['top'], 0, result)
                self.assertLessEqual(result['bottom'], result['height'], result)
                self.assertTrue(result['hit'], result)
            finally:
                browser.close()


if __name__ == '__main__':
    unittest.main()
