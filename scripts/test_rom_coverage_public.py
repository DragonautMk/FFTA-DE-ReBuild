"""Regression checks for the public aggregate coverage export."""
import json
from pathlib import Path
import tempfile
import unittest

import rom_coverage_public


class PublicCoverageTests(unittest.TestCase):
    def test_only_allowlisted_numeric_totals_are_published(self):
        totals = {key: 10 for key, _ in rom_coverage_public.METRICS}
        totals['rom'] = 100
        report = {'schema': 3, 'profile': 'europe', 'totals': totals, 'rom_sha256': 'private-hash',
                  'resources': [{'data': 'private-game-data'}],
                  'audits': [{'case': 'private-case'}], 'private': '<script>private</script>'}
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / 'private.json'
            source.write_text(json.dumps(report))
            rom_coverage_public.publish(source, root / 'public')
            page = (root / 'public/index.html').read_text()
            data = json.loads((root / 'public/coverage.json').read_text())
            self.assertEqual(data, {'schema': 1, 'totals': totals})
            self.assertIn('10.00%', page)
            self.assertNotIn('private', page)
            self.assertNotIn('private', json.dumps(data))
            self.assertEqual(json.loads(source.read_text()), report)

    def test_invalid_values_preserve_existing_output(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / 'private.json'
            source.write_text(json.dumps({'schema': 3, 'profile': 'europe', 'totals': {'rom': 1}}))
            output = root / 'public'
            output.mkdir()
            (output / 'index.html').write_text('unchanged')
            with self.assertRaises(KeyError):
                rom_coverage_public.publish(source, output)
            self.assertEqual((output / 'index.html').read_text(), 'unchanged')


if __name__ == '__main__':
    unittest.main()
