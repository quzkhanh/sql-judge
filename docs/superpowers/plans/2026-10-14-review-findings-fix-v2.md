# Latest Review Findings Fix Implementation Plan

> **For agentic workers:** Execute this plan inline without creating a commit. Preserve all existing worktree changes.

**Goal:** Make imported catalog entries immutable-version-safe, expose only validated bounded learning metadata, harden pack loading, and close connection-string debug leaks.

**Architecture:** Rust remains the trust boundary. Pack manifests may declare bounded catalog metadata; validated Rust DTOs expose only that metadata, while reference SQL and hidden fixtures remain native-only. Frontend keys and requests carry the complete immutable pack identity. Directory and ZIP loaders share explicit path, depth, entry, and byte limits.

**Tech Stack:** Rust workspace, Tauri, `serde`, Bun/TypeScript, React, Prettier.

---

### Task 1: Catalog identity and validated metadata

**Files:**

- Modify: `schemas/problem.schema.json`
- Modify: `crates/problem-pack/src/lib.rs`
- Modify: `crates/problem-pack/tests/import.rs`
- Modify: `src/lib/judge-api.ts`
- Modify: `src/features/learning/catalog.ts`
- Modify: `src/features/learning/learning-workspace.tsx`
- Test: `src/features/learning/catalog.test.js`

- [x] Add failing tests for two immutable versions producing distinct catalog keys, and for populated/absent bounded metadata.
- [x] Add optional strict `catalog` metadata to the problem manifest with bounded schema tables, public rows, and starter SQL.
- [x] Validate metadata limits and expose it through `ProblemCatalogEntry`; do not include reference SQL or hidden fixtures.
- [x] Build frontend keys from `packId`, `version`, `digest`, and problem ID; use the key for selection/drafts and exact request fields.
- [x] Render validated schema/sample/starter metadata and explicit unavailable states.
- [x] Run focused Rust and Bun tests.

### Task 2: Pack import resource hardening

**Files:**

- Modify: `crates/problem-pack/src/lib.rs`
- Modify: `crates/problem-pack/tests/import.rs`
- Modify: `scripts/validate_pack.py`
- Modify: `docs/security/THREAT-MODEL.md`
- Modify: `docs/architecture/adr/ADR-015-bounded-rust-zip-pack-loader.md`

- [x] Add failing directory and ZIP tests for path length, nesting depth, total entries/files, and recursive traversal bounds.
- [x] Enforce shared limits before recursion/allocation and retain symlink, traversal, duplicate, compression, and byte limits.
- [x] Mirror structural limits in the Python validator.
- [x] Update SEC-006 evidence and run focused import tests.

### Task 3: Debug connection URI redaction

**Files:**

- Modify: `src/lib/debug-events.ts`
- Modify: `src/lib/debug-events.test.js`

- [x] Add failing probes for raw PostgreSQL, JDBC, SQL Server, ADO-style URI text, and nested `connectionUrl`/`connectionString` values.
- [x] Redact URI credentials/query secrets and case variants of connection keys recursively while preserving safe diagnostics.
- [x] Run focused debug tests.

### Task 4: Verification and traceability

**Files:**

- Modify: `tasks/todo.md`
- Modify: `docs/architecture/adr/ADR-021-review-findings-2026-10-14.md`
- Modify: `docs/superpowers/plans/2026-10-14-review-findings-fix-continuation.md`

- [x] Record exact test counts and requirement mapping for AC-002/003/004, AC-008/009, SEC-006, and debug privacy.
- [x] Run exact Rust format checks, frontend checks, workspace/native/pack/SQL Server tests, Clippy, PostgreSQL harness, changed-file Prettier, and `git diff --check`.
- [x] Record Docker/jsonschema/manual-smoke blockers without claiming them as passing.

## Final Evidence

- `cargo test --workspace -- --test-threads=1`: passed.
- `cargo test --manifest-path src-tauri/Cargo.toml --lib`: 41 passed.
- `cargo test -p problem-pack --test import`: 30 passed.
- `cargo test -p judge-sqlserver --all-targets --features sqlserver-integration`: 10 unit/policy tests passed; 3 fixture tests ignored.
- `bun test`: 44 passed, 102 expectations.
- Focused UI-adjacent Bun command: 25 passed, 58 expectations.
- Frontend lint, typecheck, build, both exact fmt checks, both Clippy commands, Prettier, and `git diff --check`: passed.
- PostgreSQL harness passed. SQL Server Docker harness is blocked by unavailable Docker. Python pack validator is blocked by missing `jsonschema`. Desktop smoke remains pending. The six-platform release artifact matrix remains unverified.
