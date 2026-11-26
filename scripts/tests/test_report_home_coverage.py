"""Check the homepage links to the detailed map when an overview also exists."""

import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest
from html.parser import HTMLParser


ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'scripts'))
import report_home


class Shortcuts(HTMLParser):
    def __init__(self):
        super().__init__()
        self.links = []

    def handle_starttag(self, tag, attrs):
        attrs = dict(attrs)
        if tag == 'a' and 'shortcut' in attrs.get('class', '').split():
            self.links.append(attrs['href'])


class CoverageHomepageTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix='ffta-report-home-')
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.template = (ROOT / 'scripts/report_home.html').read_text()

    def report(self, relative, *, detailed, mtime):
        page = self.root / relative
        page.parent.mkdir(parents=True, exist_ok=True)
        marker = '<script type="application/json" id="report-data">{}</script>' if detailed else ''
        page.write_text('<title>Coverage</title>' + marker)
        (page.parent / 'coverage.json').write_text('{}')
        os.utime(page, (mtime, mtime))

    def rendered(self):
        reports = report_home.discover(self.root)
        page = report_home.render(reports, self.template, self.root)
        links = Shortcuts()
        links.feed(page)
        return page, links.links

    def test_latest_standard_map_wins_over_public_summary_and_old_copies(self):
        public = 'docs/development/rom-coverage/index.html'
        target = 'target/rom-coverage-local/index.html'
        fallback = 'out/rom-coverage/report/index.html'
        self.report(public, detailed=False, mtime=900)
        self.report(target, detailed=True, mtime=300)
        self.report(fallback, detailed=True, mtime=200)
        self.report('old-copy/index.html', detailed=True, mtime=1000)
        page, shortcuts = self.rendered()
        self.assertEqual(shortcuts, [target])
        self.assertIn('Detailed coverage map', page)
        self.assertIn('ROM coverage overview', page)
        self.assertIn('href="' + public + '"', page)

        self.report(fallback, detailed=True, mtime=400)
        _, shortcuts = self.rendered()
        self.assertEqual(shortcuts, [fallback])

    def test_custom_or_legacy_detailed_report_is_used_without_a_standard_map(self):
        public = 'docs/development/rom-coverage/index.html'
        custom = 'custom/coverage/index.html'
        self.report(public, detailed=False, mtime=900)
        self.report(custom, detailed=True, mtime=100)
        _, shortcuts = self.rendered()
        self.assertEqual(shortcuts, [custom])

        self.report(public, detailed=True, mtime=1000)
        _, shortcuts = self.rendered()
        self.assertEqual(shortcuts, [public])

    def test_overview_remains_accessible_when_no_detailed_map_exists(self):
        public = 'docs/development/rom-coverage/index.html'
        self.report(public, detailed=False, mtime=100)
        page, shortcuts = self.rendered()
        self.assertEqual(shortcuts, [public])
        self.assertIn('Coverage overview', page)
        self.assertNotIn('Detailed coverage map', page)

    @unittest.skipUnless(os.name == 'posix' and shutil.which('bash'), 'requires Bash')
    def test_linux_homepage_launcher_works_when_target_logs_are_unavailable(self):
        scripts = self.root / 'scripts'
        (scripts / 'linux').mkdir(parents=True)
        for relative in ('report_home.py', 'report_home.html', 'linux/report_home.sh'):
            shutil.copy2(ROOT / 'scripts' / relative, scripts / relative)
        target = self.root / 'target'
        target.mkdir()
        (target / 'build-logs').write_text('existing file')
        self.report('out/rom-coverage/report/index.html', detailed=True, mtime=100)

        result = subprocess.run(['bash', str(scripts / 'linux/report_home.sh'), '--python'],
                                cwd=self.root, capture_output=True, text=True, timeout=15)
        self.assertEqual(result.returncode, 0, result.stderr)
        page = (self.root / 'index.html').read_text()
        self.assertIn('href="out/rom-coverage/report/index.html"', page)
        self.assertEqual((target / 'build-logs').read_text(), 'existing file')
        self.assertEqual(len(list((self.root / 'out/report-home/build-logs').glob('*.log'))), 1)


if __name__ == '__main__':
    unittest.main()
