"""Keep the operation inventory honest as existing debug interfaces grow."""
import json
from pathlib import Path
import re
import unittest

ROOT = Path(__file__).resolve().parents[2]


class DebugOperationInventoryTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.contract = json.loads((ROOT / 'crates/recomp/src/debug_operations.json').read_text())
        cls.operations = cls.contract['operations']
        cls.code = (ROOT / 'crates/recomp/src/debug_operations.rs').read_text()

    def test_every_declared_cli_flag_has_a_parameter_and_source_exists(self):
        sources = {o['source'] for o in self.operations}
        for source in sorted(sources):
            with self.subTest(source=source):
                path = ROOT / source
                self.assertTrue(path.is_file())
                if path.name == 'play.rs':
                    # General play/benchmark flags are outside DBG2-01. These four
                    # are the debug-specific launch inputs, inspected separately.
                    flags = {'--debug-catalog', '--debug-replay', '--debug-paused', '--record-input'}
                else:
                    text = path.read_text().split('#[cfg(test)]')[0]
                    flags = set(re.findall(r'[\"\'](--[a-z][a-z0-9-]*)[\"\']', text))
                    # These are argv for a child/reproduction launcher, not
                    # parameters accepted by the enclosing command's parser.
                    delegated = {
                        'debug_issue.rs': {'--json'},
                        'debug_session.rs': {'--record-input', '--debug-paused'},
                        'debug_campaign.py': {'--json'},
                    }.get(path.name, set())
                    all_parameters = {p['name'] for o in self.operations for p in o['parameters']}
                    self.assertFalse(delegated - all_parameters)
                    flags -= delegated
                parameters = {p['name'] for o in self.operations if o['source'] == source for p in o['parameters']}
                self.assertFalse(flags - parameters, f'Uninventoried parameters: {flags - parameters}')

    def test_every_menu_and_control_id_resolves_to_descriptor(self):
        ids = {o['id'] for o in self.operations}
        self.assertEqual(len(ids), len(self.operations))
        mapping = self.code.split('impl crate::debug_control::Action')[1].split('fn query')[0]
        mapped = set(re.findall(r'=> "([a-z][a-z.-]+)"', mapping))
        self.assertFalse(mapped - ids)
        menu_enum = (ROOT / 'crates/menu/src/lib.rs').read_text().split('pub enum MenuAction {')[1].split('\n}')[0]
        variants = set(re.findall(r'^    ([A-Z]\w*)\b', menu_enum, re.M))
        menu_map = mapping.split('pub(crate) fn menu_operation')[1]
        for variant in variants:
            self.assertRegex(menu_map, r'\b' + variant + r'\b')

    def test_implicit_dialog_sites_and_open_menu_parameters_are_visible(self):
        play = (ROOT / 'crates/recomp/src/play.rs').read_text()
        self.assertEqual(play.count('rfd::FileDialog::'), 0)
        sites = {o['external_dialog']['site'] for o in self.operations if o['external_dialog']}
        self.assertEqual(sites, {'content_picker.rs::Picker::browse_file', 'content_picker.rs::Picker::browse_folder'})
        picker = (ROOT / 'crates/recomp/src/content_picker.rs').read_text()
        self.assertEqual(picker.count('rfd::FileDialog::'), 2)
        self.assertTrue(all(not o['external_dialog']['automatic'] for o in self.operations if o['id'] in {'content.browse-file', 'content.add-folder'}))
        for op in self.operations:
            for p in op['parameters']:
                if p['current_menu'] == 'unavailable':
                    self.assertTrue(op['open_work'], (op['id'], p['name']))


if __name__ == '__main__':
    unittest.main()
