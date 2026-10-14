#!/usr/bin/env python3
"""Regression tests for structural pack validator (stdlib unittest + jsonschema)."""
from __future__ import annotations

import json
import os
import shutil
import stat
import tempfile
import unittest
from pathlib import Path
from zipfile import ZipFile, ZipInfo

from validate_pack import ROOT, load_zip, validate, load_folder

SAMPLE = ROOT / "examples" / "packs" / "hr-analytics"


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

    def _rewrite_problem(self, change):
        path = self.folder / "problems/above-average/problem.json"
        data = json.loads(path.read_text())
        change(data)
        path.write_text(json.dumps(data))

    def test_valid_catalog_sample_tables_accepted(self):
        self._rewrite_problem(
            lambda data: data["catalog"].__setitem__(
                "sample_tables",
                [{"name": "departments", "rows": [{"id": 1, "name": "Engineering"}]}],
            )
        )
        self.assertEqual([], validate(load_folder(self.folder, [])))

    def test_oversized_catalog_sample_table_rejected(self):
        rows = [{"id": index, "name": "x"} for index in range(21)]
        self._rewrite_problem(
            lambda data: data["catalog"].__setitem__(
                "sample_tables", [{"name": "departments", "rows": rows}]
            )
        )
        self.assertTrue(any("sample_tables" in s for s in validate(load_folder(self.folder, []))))

    def test_invalid_catalog_sample_table_name_rejected(self):
        self._rewrite_problem(
            lambda data: data["catalog"].__setitem__(
                "sample_tables", [{"name": "bad-name", "rows": []}]
            )
        )
        self.assertTrue(any("sample_tables" in s for s in validate(load_folder(self.folder, []))))

    def test_unknown_catalog_sample_table_rejected(self):
        self._rewrite_problem(
            lambda data: data["catalog"].__setitem__(
                "sample_tables", [{"name": "salaries", "rows": []}]
            )
        )
        self.assertTrue(
            any("sample table" in s for s in validate(load_folder(self.folder, [])))
        )

    def test_duplicate_catalog_sample_table_rejected(self):
        self._rewrite_problem(
            lambda data: data["catalog"].__setitem__(
                "sample_tables",
                [{"name": "departments", "rows": []}, {"name": "departments", "rows": []}],
            )
        )
        self.assertTrue(
            any("sample table" in s for s in validate(load_folder(self.folder, [])))
        )

    def test_multiline_starter_sql_accepted(self):
        self._rewrite_problem(
            lambda data: data["catalog"].__setitem__(
                "starter_sql", "SELECT e.id\r\nFROM employees AS e\n\tWHERE e.id > 0;"
            )
        )
        self.assertEqual([], validate(load_folder(self.folder, [])))

    def test_starter_sql_control_characters_rejected(self):
        for bad in ("SELECT 1\x00", "SELECT 1\rFROM employees", "SELECT 1\x1b[31m", "SELECT 1\x85"):
            self._rewrite_problem(
                lambda data, bad=bad: data["catalog"].__setitem__("starter_sql", bad)
            )
            self.assertTrue(
                any("starter_sql" in s for s in validate(load_folder(self.folder, []))),
                repr(bad),
            )

    def test_symlink_in_folder_rejected(self):
        (self.folder / "escape.sql").symlink_to(self.folder / "schema/001_tables.sql")
        problems: list[str] = []
        load_folder(self.folder, problems)
        self.assertTrue(any("Symlink forbidden" in s for s in problems))

    def test_hardlink_in_folder_rejected(self):
        source = self.folder / "README.md"
        linked = self.folder / "README-copy.md"
        os.link(source, linked)
        problems: list[str] = []
        load_folder(self.folder, problems)
        self.assertTrue(any("Hard link forbidden" in s for s in problems))

    def test_zip_link_like_entry_rejected(self):
        zip_path = Path(self.tmp.name) / "link-like.sqljudge.zip"
        info = ZipInfo("link.sql")
        info.create_system = 3
        info.external_attr = (stat.S_IFLNK | 0o777) << 16
        with ZipFile(zip_path, "w") as archive:
            archive.writestr(info, "target.sql")
        problems: list[str] = []
        load_zip(zip_path, problems)
        self.assertTrue(any("symlink/special" in s for s in problems))

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

    def test_zip_cumulative_compressed_size_rejected(self):
        zip_path = Path(self.tmp.name) / "compressed-size.sqljudge.zip"
        content = bytes(range(251)) * (900_000 // 251 + 1)
        with ZipFile(zip_path, "w") as archive:
            for index in range(10):
                info = ZipInfo(f"files/{index}.sql")
                info.compress_type = 0
                archive.writestr(info, content[:900_000])

        problems: list[str] = []
        load_zip(zip_path, problems)
        self.assertTrue(any("Compressed ZIP exceeds limit" in s for s in problems))

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
