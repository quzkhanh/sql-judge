#!/usr/bin/env python3
"""Structural-only SQL Judge pack v1 validator. Does NOT execute SQL."""
from __future__ import annotations

import argparse
import json
import re
import stat
import sys
from pathlib import Path, PurePosixPath
from zipfile import ZipFile, BadZipFile

try:
    from jsonschema import Draft202012Validator
except ImportError as exc:
    raise SystemExit("Missing dependency: python3 -m pip install jsonschema>=4.0") from exc

ROOT = Path(__file__).resolve().parent.parent
MAX_FILES = 2000
MAX_ENTRIES = 4096
MAX_FILE_BYTES = 1 * 1024 * 1024
MAX_TOTAL_BYTES = 32 * 1024 * 1024
MAX_COMPRESSED_BYTES = 8 * 1024 * 1024
MAX_RELATIVE_PATH_BYTES = 512
MAX_PATH_DEPTH = 16
ALLOWED_MISC = {"README.md", "LICENSE"}


def safe_name(name: str) -> bool:
    if (
        not name
        or len(name.encode("utf-8")) > MAX_RELATIVE_PATH_BYTES
        or name.count("/") + 1 > MAX_PATH_DEPTH
        or "\\" in name
        or "\x00" in name
        or name.startswith("/")
    ):
        return False
    if re.match(r"^[a-zA-Z]:", name):
        return False
    parts = name.split("/")
    if any(part in ("", ".", "..") for part in parts):
        return False
    return PurePosixPath(name).as_posix() == name


def load_folder(folder: Path, problems: list[str]) -> dict[str, bytes]:
    if not folder.is_dir() or folder.is_symlink():
        problems.append("Source must be a real directory, not symlink")
        return {}
    files: dict[str, bytes] = {}
    total = 0
    entries = 0
    for entry in sorted(folder.rglob("*")):
        entries += 1
        if entries > MAX_ENTRIES:
            problems.append("Too many filesystem entries")
            break
        if entry.is_symlink():
            problems.append(f"Symlink forbidden: {entry.relative_to(folder)}")
            continue
        name = entry.relative_to(folder).as_posix()
        if not safe_name(name):
            problems.append(f"Unsafe path: {name}")
            continue
        if entry.is_dir():
            continue
        if not entry.is_file():
            problems.append(f"Non-regular file forbidden: {entry.relative_to(folder)}")
            continue
        try:
            if entry.stat().st_nlink > 1:
                problems.append(f"Hard link forbidden: {name}")
                continue
        except OSError as exc:
            problems.append(f"File metadata unavailable: {name}: {exc}")
            continue
        size = entry.stat().st_size
        total += size
        if size > MAX_FILE_BYTES:
            problems.append(f"File too large: {name}")
            continue
        if len(files) >= MAX_FILES or total > MAX_TOTAL_BYTES:
            problems.append("Pack resource limits exceeded")
            break
        files[name] = entry.read_bytes()
    return files


def load_zip(path: Path, problems: list[str]) -> dict[str, bytes]:
    files: dict[str, bytes] = {}
    total = 0
    compressed_total = 0
    try:
        with ZipFile(path) as zf:
            if len(zf.infolist()) > MAX_ENTRIES:
                problems.append("Too many ZIP entries")
                return {}
            seen: set[str] = set()
            for info in zf.infolist():
                name = info.filename
                compressed_total += info.compress_size
                if compressed_total > MAX_COMPRESSED_BYTES:
                    problems.append("Compressed ZIP exceeds limit")
                    break
                if info.is_dir():
                    if not safe_name(name.rstrip("/")):
                        problems.append(f"Unsafe directory entry: {name}")
                    continue
                if not safe_name(name):
                    problems.append(f"Unsafe ZIP entry path: {name}")
                    continue
                folded = name.casefold()
                if folded in seen:
                    problems.append(f"Duplicate/case-colliding ZIP entry: {name}")
                    continue
                seen.add(folded)
                mode = (info.external_attr >> 16) & 0xFFFF
                if stat.S_IFMT(mode) not in (0, stat.S_IFREG):
                    problems.append(f"ZIP symlink/special file forbidden: {name}")
                    continue
                if info.file_size > MAX_FILE_BYTES:
                    problems.append(f"ZIP entry too large: {name}")
                    continue
                if info.file_size > 100000 and info.file_size / max(info.compress_size, 1) > 150:
                    problems.append(f"Suspicious compression ratio: {name}")
                    continue
                total += info.file_size
                if total > MAX_TOTAL_BYTES:
                    problems.append("Uncompressed ZIP exceeds limit")
                    break
                data = zf.read(info)
                if len(data) != info.file_size:
                    problems.append(f"ZIP size mismatch: {name}")
                    continue
                files[name] = data
    except (BadZipFile, OSError, RuntimeError, ValueError) as exc:
        problems.append(f"ZIP cannot be read: {type(exc).__name__}: {exc}")
    return files


def json_file(files: dict[str, bytes], name: str, errors: list[str]) -> dict | None:
    if name not in files:
        errors.append(f"Missing {name}")
        return None
    try:
        value = json.loads(files[name].decode("utf-8"))
    except (UnicodeDecodeError, ValueError) as exc:
        errors.append(f"Invalid UTF-8 JSON {name}: {exc}")
        return None
    if not isinstance(value, dict):
        errors.append(f"{name}: expected JSON object")
        return None
    return value


def schema_errors(obj: dict, schema_name: str, file: str) -> list[str]:
    schema = json.loads((ROOT / "schemas" / schema_name).read_text(encoding="utf-8"))
    validator = Draft202012Validator(schema)
    errors = sorted(validator.iter_errors(obj), key=lambda error: str(error.json_path))
    return [f"{file}{error.json_path[1:]}: {error.message}" for error in errors]


def validate(files: dict[str, bytes]) -> list[str]:
    errors: list[str] = []
    folded: set[str] = set()
    for name, content in files.items():
        if not safe_name(name):
            errors.append(f"Unsafe path: {name}")
        if name.casefold() in folded:
            errors.append(f"Case-insensitive path collision: {name}")
        folded.add(name.casefold())
        if len(content) > MAX_FILE_BYTES:
            errors.append(f"Excessive file size: {name}")
    if (
        len(files) > MAX_FILES
        or len(files) > MAX_ENTRIES
        or sum(len(v) for v in files.values()) > MAX_TOTAL_BYTES
    ):
        errors.append("Pack resource limits exceeded")
    pack = json_file(files, "pack.json", errors)
    if pack is None:
        return errors
    errors += schema_errors(pack, "pack.schema.json", "pack.json")
    if errors:
        return errors
    expected = {"pack.json", *pack["schema"], *pack["problems"]}
    # All declared schema paths must exist and be UTF-8 SQL with nonempty content.
    for schema_path in pack["schema"]:
        check_sql(files, schema_path, errors)
    for problem_path in pack["problems"]:
        problem = json_file(files, problem_path, errors)
        if problem is None:
            continue
        errs = schema_errors(problem, "problem.schema.json", problem_path)
        errors.extend(errs)
        if errs:
            continue
        root = PurePosixPath(problem_path).parent
        if root.name != problem["id"]:
            errors.append(f"Problem ID does not match folder: {problem_path}")
        if problem["dialect"] != pack["dialect"]:
            errors.append(f"Dialect mismatch: {problem_path}")
        reference_path = (root / problem["reference"]).as_posix()
        expected.add(reference_path)
        check_sql(files, reference_path, errors)
        test_ids: set[str] = set()
        test_paths: set[str] = set()
        visible = hidden = 0
        for test in problem["tests"]:
            if test["id"] in test_ids:
                errors.append(f"Duplicate test id {test['id']} in {problem_path}")
            test_ids.add(test["id"])
            if test["hidden"]:
                hidden += 1
            else:
                visible += 1
            test_path = (root / test["file"]).as_posix()
            if test_path in test_paths:
                errors.append(f"Duplicate test file: {test_path}")
            test_paths.add(test_path)
            expected.add(test_path)
            check_sql(files, test_path, errors)
        if visible < 1 or hidden < 2:
            errors.append(f"{problem_path}: requires >=1 visible and >=2 hidden fixtures")
        catalog = problem.get("catalog", {})
        if "starter_sql" in catalog and not starter_sql_is_allowed(catalog["starter_sql"]):
            errors.append(
                f"{problem_path}: catalog starter_sql contains a control character "
                "(only \\n, \\r\\n and \\t are allowed)"
            )
        catalog_tables = {table["name"] for table in catalog.get("schema", [])}
        sample_tables: set[str] = set()
        for table in catalog.get("sample_tables", []):
            if table["name"] in sample_tables:
                errors.append(f"{problem_path}: duplicate catalog sample table {table['name']}")
            sample_tables.add(table["name"])
            if catalog_tables and table["name"] not in catalog_tables:
                errors.append(
                    f"{problem_path}: catalog sample table {table['name']} is not in catalog.schema"
                )
        column_names = [col["name"] for col in problem["output"]["columns"]]
        if len(column_names) != len(set(column_names)):
            errors.append(f"{problem_path}: duplicate output column names")
    extras = set(files) - expected - ALLOWED_MISC
    if extras:
        errors.append(f"Undeclared files: {', '.join(sorted(extras))}")
    return errors


def starter_sql_is_allowed(sql: str) -> bool:
    """Mirror of the Rust rule: only \\n, \\r\\n and \\t control characters are allowed."""
    index = 0
    while index < len(sql):
        character = sql[index]
        code = ord(character)
        if character == "\r" and sql[index + 1 : index + 2] == "\n":
            index += 2
            continue
        if character not in "\n\t" and (code < 0x20 or 0x7F <= code <= 0x9F):
            return False
        index += 1
    return True


def check_sql(files: dict[str, bytes], path: str, errors: list[str]) -> None:
    content = files.get(path)
    if content is None:
        errors.append(f"Missing SQL file: {path}")
        return
    if not content.strip():
        errors.append(f"Empty SQL file: {path}")
        return
    try:
        decoded = content.decode("utf-8")
    except UnicodeDecodeError:
        errors.append(f"Non UTF-8 SQL file: {path}")
        return
    if "\x00" in decoded:
        errors.append(f"NUL byte in SQL file: {path}")


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("path", type=Path, help="Pack folder or .sqljudge.zip path")
    args = ap.parse_args()
    load_errors: list[str] = []
    if args.path.is_dir():
        files = load_folder(args.path, load_errors)
    elif args.path.is_file() and args.path.suffix.lower() == ".zip":
        files = load_zip(args.path, load_errors)
    else:
        files = {}
        load_errors.append("Expected a pack directory or .zip file")
    errors = load_errors + validate(files)
    report = {
        "structural_status": "valid" if not errors else "invalid",
        "database_status": "not_run",
        "files_inspected": len(files),
        "errors": errors,
        "note": "Structural checks only. No SQL executed; not a trusted runtime validation."
    }
    print(json.dumps(report, ensure_ascii=False, indent=2))
    return 0 if not errors else 1


if __name__ == "__main__":
    sys.exit(main())
