# Latest Review Findings Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Close the five latest review findings while preserving exact managed pack pinning, metadata-only backups, fail-closed database policy, and the existing dirty worktree.

**Architecture:** Native restore validates managed pack identity against the persisted exact ID/version/digest before SQLite replacement. SQL Server uses browse-mode result dependency metadata and rejects unresolved or unsupported indirect dependency signals. Debug forwarding sanitizes recursively at the JS boundary. A bounded native pack catalog exposes only validated metadata and problem manifests; the UI selects catalog entries and never receives SQL payloads.

**Tech Stack:** Rust workspace, Tauri native commands, PostgreSQL/SQL Server adapters, SQLite metadata storage, TypeScript/React, Bun tests, ESLint, Prettier.

---

### Task 1: Exact Backup Identity

**Files:** `src-tauri/src/lib.rs`, `crates/app-storage/src/lib.rs`, native/app-storage tests.

- [x] Add failing tests for changed managed `pack_version` and changed `pack_digest` during restore.
- [x] Run those tests and verify the restore currently accepts the changed identity or reports the missing comparison.
- [x] Compare all managed identity fields before any metadata replacement and keep backup version 2 metadata-only.
- [x] Verify failed restore preserves the existing environment, submission history, and external metadata.

### Task 2: SQL Server Dependency Policy

**Files:** `crates/judge-sqlserver/src/lib.rs`, SQL Server policy tests, ADR.

- [x] Add failing tests for unresolved dependency metadata and browse-mode indirect dependency rows.
- [x] Use browse/dependency metadata mode `1`, reject unresolved/error rows, and reject unsupported indirect dependency constructs conservatively.
- [x] Run crate tests and record ignored integration/harness results separately.

### Task 3: Debug Argument Sanitization

**Files:** `src/lib/debug-events.ts`, `src/lib/debug-events.test.js`, debug panel tests if needed.

- [x] Add failing forwarding tests with password, token, ownership nonce, SQL, nested objects, arrays, and multiple arguments.
- [x] Implement recursive bounded sanitization while preserving argument count and safe multi-argument forwarding.
- [x] Verify no sensitive value reaches the captured native console/log sink.

### Task 4: Validated Pack Catalog

**Files:** `crates/problem-pack/src/lib.rs`, `src-tauri/src/lib.rs`, `src/lib/judge-api.ts`, pack/environment/learning UI files, Rust/TS tests.

- [x] Add a native catalog DTO and failing non-default pack catalog tests.
- [x] Expose only digest-verified pack metadata and bounded problem manifests; reject invalid/missing registry entries.
- [x] Replace static learning catalog and hardcoded environment pack selection with catalog state and exact pack identity requests.
- [x] Keep SQL and pack file contents native-only.

### Task 5: Traceability and Full Verification

**Files:** ADR/task/plan documentation and changed source files only.

- [x] Update ADR/task evidence for all five findings.
- [x] Run changed-file Prettier, Bun test/lint/typecheck/build, workspace/native Rust tests, Clippy `-D warnings`, fmt, PostgreSQL harness, SQL Server crate tests and ignored-test report, and `git diff --check`.
- [x] Report unavailable external prerequisites without claiming success and do not commit.

Final counts for the review-finding work: Bun 44 tests / 102 expectations; the focused UI-adjacent command passes 25 tests / 58 expectations; app-storage 17 tests; native Tauri 41 tests; problem-pack import 30 tests; SQL Server 10 unit/policy tests (7 unit plus 3 policy), with 3 fixture tests ignored. PostgreSQL now has 19 integration, 9 policy, and 4 runtime tests. Docker SQL Server, Python `jsonschema`, and desktop smoke remain unavailable; the six-platform release artifact matrix remains unverified.

### Follow-up hardening (2026-10-14)

- [x] Replace SQL Server metadata/dependency result collection with row-by-row streaming and explicit row/byte budgets.
- [x] Redact ADO `Pwd=` connection-string credentials alongside URI/JDBC/ADO `Password=` forms.
- [x] Enforce cumulative compressed-byte limits in Rust and Python ZIP loaders with negative coverage.
- [x] Reject filesystem hardlinks and link-like non-regular ZIP metadata under the conservative archive policy.
- [x] Add installed-pack/problem metadata empty states and disable judging without complete metadata.
- [x] Correct verification counts and security evidence for the follow-up changes.
