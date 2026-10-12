#!/usr/bin/env python3
"""Regression tests for structural pack validator (stdlib unittest + jsonschema)."""
from __future__ import annotations

import json
import shutil
import tempfile
import unittest
from pathlib import Path
from zipfile import ZipFile, ZipInfo

from validate_pack import ROOT, load_zip, validate, load_folder

SAMPLE = ROOT / "examples" / "packs" / "hr-sql-101"


class PackValidatorTests(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.folder = Path(self.tmp.name) / "pack"
        shutil.copytree(SAMPLE, self.folder)

    def test_sample_is_valid(self):
        self.assertEqual([], validate(load_folder(self.folder, [])))

    def test_unknown_manifest_field_rejected(self):
        manifest = self.folder / "pack.json"
        data = json.loads(manifest.read_text())
        data["evil"] = True
        manifest.write_text(json.dumps(data))
        self.assertTrue(any("Additional properties" in s for s in validate(load_folder(self.folder, []))))

    def test_missing_reference_rejected(self):
        (self.folder / "problems/above-average/reference.sql").unlink()
        self.assertTrue(any("Missing SQL" in s for s in validate(load_folder(self.folder, []))))

    def test_wrong_dialect_rejected(self):
        path = self.folder / "problems/above-average/problem.json"
        data = json.loads(path.read_text())
        data["dialect"] = "sqlserver"
        path.write_text(json.dumps(data))
        self.assertTrue(any("Dialect mismatch" in s for s in validate(load_folder(self.folder, []))))

    def test_duplicate_test_id_rejected(self):
        path = self.folder / "problems/above-average/problem.json"
        data = json.loads(path.read_text())
        data["tests"][1]["id"] = data["tests"][0]["id"]
        path.write_text(json.dumps(data))
        self.assertTrue(any("Duplicate test id" in s for s in validate(load_folder(self.folder, []))))

    def test_symlink_in_folder_rejected(self):
        (self.folder / "escape.sql").symlink_to(self.folder / "schema/001_tables.sql")
        problems: list[str] = []
        load_folder(self.folder, problems)
        self.assertTrue(any("Symlink forbidden" in s for s in problems))

    def test_zip_path_traversal_rejected(self):
        zip_path = Path(self.tmp.name) / "attack.sqljudge.zip"
        with ZipFile(zip_path, "w") as archive:
            archive.writestr("../outside.sql", "SELECT 1")
        problems: list[str] = []
        load_zip(zip_path, problems)
        self.assertTrue(any("Unsafe ZIP entry path" in s for s in problems))

    def test_zip_duplicate_entries_rejected(self):
        zip_path = Path(self.tmp.name) / "attack.sqljudge.zip"
        with ZipFile(zip_path, "w") as archive:
            archive.writestr("pack.json", "{}")
            archive.writestr("PACK.json", "{}")
        problems: list[str] = []
        load_zip(zip_path, problems)
        self.assertTrue(any("Duplicate/case-colliding" in s for s in problems))

    def test_extra_file_rejected(self):
        (self.folder / "run.sh").write_text("echo hello")
        self.assertTrue(any("Undeclared files" in s for s in validate(load_folder(self.folder, []))))

    def test_package_exposes_safe_validation_command(self):
        package = json.loads((ROOT / "package.json").read_text(encoding="utf-8"))
        self.assertEqual(
            "python3 scripts/validate_pack.py",
            package["scripts"]["validate:pack"],
        )


if __name__ == "__main__":
    unittest.main()
