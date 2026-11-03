"""Cleanup behavior with synthetic fixtures; never touch real project artifacts."""
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import local_artifacts as artifacts


class ArtifactCleanupTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="ffta cleanup ")
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name).resolve()
        subprocess.run(["git", "init", "-q", str(self.root)], check=True)
        self.unit = self.root / "target/rom-data-completed"
        self.unit.mkdir(parents=True)
        (self.unit / "managed-result.json").write_text('{"passed":true}')
        self.managed = self.unit / "windows-managed-europe"
        for name in ("interrupted", "moved resources", "offline dataset"):
            folder = self.managed / name
            folder.mkdir(parents=True)
            (folder / "resource.json").write_text("fixture")
        self.saved = self.unit / "start.ffstate"
        self.saved.write_bytes(b"private fixture")

    def plan(self, **kwargs):
        return artifacts.completion_plan([self.unit], self.root, **kwargs)

    def test_preview_and_apply_keep_accepted_dataset_inputs_and_results(self):
        plan, skipped = self.plan()
        self.assertFalse(skipped)
        self.assertEqual(len(plan), 2)
        self.assertTrue(all(Path(p["path"]).exists() for p in plan))
        for p in plan:
            artifacts.remove_candidate(p)
        self.assertTrue((self.managed / "offline dataset/resource.json").exists())
        self.assertTrue((self.unit / "managed-result.json").exists())
        self.assertEqual(self.saved.read_bytes(), b"private fixture")
        self.assertEqual(self.plan(), ([], []))

    def test_changed_candidate_is_refused_before_removal(self):
        plan, _ = self.plan()
        (Path(plan[0]["path"]) / "resource.json").write_text("changed fixture")
        with self.assertRaisesRegex(ValueError, "changed since preview"):
            artifacts.remove_candidate(plan[0])
        self.assertTrue(Path(plan[0]["path"]).exists())

    def test_private_or_tracked_content_is_never_deleted(self):
        private = self.managed / "interrupted/keep.sav"
        private.write_bytes(b"save")
        tracked = self.managed / "moved resources/resource.json"
        subprocess.run(["git", "-C", str(self.root), "add", str(tracked)], check=True)
        plan, skipped = self.plan()
        self.assertFalse(plan)
        self.assertEqual(len(skipped), 2)
        self.assertTrue(private.exists())
        self.assertTrue(tracked.exists())

    @unittest.skipUnless(os.name == "posix", "symlink fixture")
    def test_links_and_unknown_files_are_refused(self):
        (self.managed / "interrupted/link.json").symlink_to(self.saved)
        (self.managed / "moved resources/source.rs").write_text("source")
        plan, skipped = self.plan()
        self.assertFalse(plan)
        self.assertEqual(len(skipped), 2)
        self.assertEqual(self.saved.read_bytes(), b"private fixture")
        (self.managed / "interrupted/link.json").unlink()
        (self.managed / "moved resources/source.rs").unlink()
        import shutil
        shutil.rmtree(self.managed)
        self.managed.symlink_to(self.root, target_is_directory=True)
        self.assertEqual(self.plan(), ([], []))

    def test_only_explicitly_retired_test_executables_are_selected(self):
        for name in ("recomp-tests-managed.exe", "gba-core-tests-managed.exe", "recomp-rom-data.exe",
                     "personal-tool.exe"):
            (self.unit / name).write_bytes(b"exe")
        self.assertEqual(len(self.plan()[0]), 2)
        plan, _ = self.plan(retire_executables=True)
        self.assertEqual(len(plan), 5)
        self.assertFalse(any(p["path"].endswith("personal-tool.exe") for p in plan))

    def test_root_and_missing_evidence_are_rejected(self):
        for path in (self.root, self.root / "target", self.managed, self.root / "rom-data-other"):
            with self.assertRaises(ValueError):
                artifacts.completion_plan([path], self.root)
        (self.unit / "managed-result.json").unlink()
        with self.assertRaisesRegex(ValueError, "no result/summary"):
            self.plan()

    def cache(self, revisions):
        cache = self.root / "cache"
        rom = cache / ("a" * 64)
        rom.mkdir(parents=True)
        for rev in revisions:
            p = rom / f"e{rev}-d{rev-1}"
            p.mkdir()
            (p / "manifest.toml").write_text(
                f'source_rom_sha256="{"a"*64}"\nextractor_revision={rev}\ndecoder_revision={rev-1}\n')
            (p / "resource.json").write_text("fixture")
        return cache, rom

    def test_cache_keeps_current_and_newest_revision_numerically(self):
        cache, rom = self.cache([8, 9, 10])
        removed = artifacts.prune_coverage_cache(cache, rom / "e8-d7", 2)
        self.assertEqual(removed, [str(rom / "e9-d8")])
        self.assertTrue((rom / "e8-d7").exists())
        self.assertTrue((rom / "e10-d9").exists())
        self.assertEqual(artifacts.prune_coverage_cache(cache, rom / "e8-d7", 0), [])

    def test_cache_unknown_identity_and_private_payload_are_protected(self):
        cache, rom = self.cache([8, 9, 10])
        (rom / "e8-d7/keep.ffstate").write_bytes(b"save")
        with self.assertRaisesRegex(ValueError, "private input"):
            artifacts.prune_coverage_cache(cache, rom / "e10-d9", 2)
        self.assertTrue((rom / "e8-d7/keep.ffstate").exists())
        (rom / "e8-d7/keep.ffstate").unlink()
        (rom / "e8-d7/manifest.toml").write_text('source_rom_sha256="unknown"\n')
        self.assertEqual(artifacts.prune_coverage_cache(cache, rom / "e10-d9", 2), [])
        self.assertTrue((rom / "e8-d7").exists())

    def test_cleanup_manifest_and_report_path_validation(self):
        report = self.root / "target/cleanup.json"
        args = ["local_artifacts.py", "complete", str(self.unit), "--apply", "--report", str(report)]
        with mock.patch.object(artifacts, "ROOT", self.root), mock.patch.object(sys, "argv", args):
            self.assertEqual(artifacts.main(), 0)
        result = json.loads(report.read_text())
        self.assertEqual(len(result["removed"]), 2)
        args[-1] = str(self.root / "outside.json")
        with mock.patch.object(artifacts, "ROOT", self.root), mock.patch.object(sys, "argv", args):
            self.assertEqual(artifacts.main(), 1)


if __name__ == "__main__":
    unittest.main()
