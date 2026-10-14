# Final Review Findings Fix Implementation Plan

> **For agentic workers:** Execute this plan inline without creating a commit; preserve all existing worktree changes.

**Goal:** Close the remaining five review findings without weakening read-only, pack-import, backup, or input-boundary safety.

**Architecture:** Keep external query validation at the native boundary before credentials are resolved or network connections are opened. Make managed environment provisioning resolve only immutable, digest-verified validated packs from the local registry. Keep backups metadata-only and validate pack identity metadata without embedding pack contents.

**Tech Stack:** Rust workspace, Tiberius, PostgreSQL client, SQLite metadata storage, Tauri IPC, Bun/Prettier.

---

### Task 1: SQL Server dependency validation

**Files:**

- Modify `crates/judge-sqlserver/src/lib.rs`
- Modify `crates/judge-sqlserver/tests/policy.rs`
- Modify `crates/judge-sqlserver/tests/sqlserver_integration.rs` if fixture coverage is available

- [x] Add policy coverage for indirect and unresolved dependencies.
- [x] Inspect `sys.sql_expression_dependencies` inside the read-only transaction and reject unresolved or cross-database dependencies before the user SELECT.
- [x] Preserve direct lexical rejection and fail closed on metadata query errors.
- [x] Focused SQL Server tests pass; the Docker fixture remains blocked because Docker is unavailable.

### Task 2: Registry-backed managed pack selection

**Files:**

- Modify `crates/problem-pack/src/lib.rs`
- Modify `crates/problem-pack/tests/import.rs`
- Modify `src-tauri/src/lib.rs`

- [x] Add tests for loading non-default published packs and exposing validated problem judge inputs.
- [x] Add digest-verified exact registry lookup and reject tampered or latest-only resolution.
- [x] Replace hardcoded managed provisioning and judging inputs with validated registry data.
- [x] Update native environment and judge regressions to import their fixture pack before use.

### Task 3: Native external SQL input bound

**Files:**

- Modify `src-tauri/src/lib.rs`
- Add/update native tests in `src-tauri/src/lib.rs`

- [x] Add native boundary coverage for PostgreSQL and SQL Server SQL and limit requests.
- [x] Reject SQL and invalid query limits before profile lookup, secret access, connection, or statement preparation.
- [x] Keep adapter result/statement limits unchanged.

### Task 4: Metadata-only backup contract

**Files:**

- Modify `crates/app-storage/src/lib.rs`
- Modify `crates/app-storage/tests/storage.rs`
- Modify `src/lib/judge-api.ts` only if the typed contract needs alignment
- Update `docs/architecture/adr/ADR-016-json-metadata-backup.md`

- [x] Add tests requiring pack ID, version, and digest identity metadata while rejecting embedded SQL/registry fields.
- [x] Preserve secret-free metadata-only export and require separately imported validated packs at restore/use time.
- [x] Document that backup restore does not import or trust pack contents.

### Task 5: Task status and formatting

**Files:**

- Modify `tasks/todo.md`
- Format all changed TS/TSX/MD/JSON files with the repository formatter.

- [x] Correct stale task claims/counts and distinguish verified work from Docker/desktop/validator-blocked acceptance.
- [x] Run frontend checks, Rust tests, Clippy, fmt, pack tests, harnesses, Prettier, and `git diff --check`.
- [x] Report exact command results and blockers without committing.

### Task 6: Pack SQL isolation

**Files:**

- Modify `crates/judge-postgres/src/lib.rs`
- Modify `crates/problem-pack/src/lib.rs`
- Modify `crates/judge-postgres/tests/policy.rs`
- Modify `crates/problem-pack/tests/import.rs`

- [x] Add tests for `CREATE TABLE sj_other.evil`, three-part/database-qualified identifiers, and valid unqualified schema SQL.
- [x] Preserve only unqualified identifiers plus explicit current-namespace and `pg_catalog` references at the setup boundary.
- [x] Apply the same policy to structural pack schema validation so poisoned packs fail before runtime execution.

### Task 7: Atomic pack publication and managed environment pins

**Files:**

- Modify `crates/problem-pack/src/lib.rs`
- Modify `crates/problem-pack/tests/import.rs`
- Modify `crates/environment-manager/src/lib.rs`
- Modify `crates/environment-manager/src/ledger.rs`
- Modify `crates/app-storage/src/lib.rs`
- Modify `crates/app-storage/tests/storage.rs`
- Modify `src-tauri/src/lib.rs`
- Modify `src/lib/judge-api.ts`

- [x] Add publish-conflict/no-schema-left-behind coverage.
- [x] Publish the immutable pack before managed namespace provisioning and keep failed publication side-effect free.
- [x] Add pinned version/digest fields, migrate old SQLite rows with empty legacy pins, reject ambiguous/latest-only resolution, and preserve metadata-only backup fields.
- [x] Make create, clone, reset, recovery, storage, backup, judging, and frontend summary paths carry the exact pin.

### Task 8: SQL Server timeout classification

**Files:**

- Modify `crates/judge-sqlserver/src/lib.rs`
- Modify `crates/judge-sqlserver/tests/policy.rs`

- [x] Add mapping tests for timed-out connection versus timed-out query operation.
- [x] Map operation-context `TimedOut` errors to `QueryTimeout` while retaining connection timeout as `ServerUnreachable`.

### Task 9: Latest hardening findings

- [x] Inspect raw SQL Server `ColumnData` cell lengths before decoding and reject a single oversized cell while preserving row and result-byte budgets.
- [x] Reject filesystem hardlinks using platform metadata link counts and reject link-like non-regular ZIP metadata without treating duplicate contents as links.
- [x] Add explicit zero-problem and unavailable-metadata UI states; disable Run/Submit until schema, sample output, and starter SQL metadata are complete.
- [x] Add negative/helper regressions and format the security, example-pack, plan, and task documentation.

## Verification Matrix

| Check                                                                          | Result                                       |
| ------------------------------------------------------------------------------ | -------------------------------------------- |
| `cargo test --workspace -- --test-threads=1`                                   | Passed; PostgreSQL policy target has 9 tests |
| `cargo test --manifest-path src-tauri/Cargo.toml --lib`                        | Passed, 41 tests                             |
| `cargo test -p problem-pack --test import`                                     | Passed, 30 tests                             |
| `cargo test -p judge-sqlserver --all-targets --features sqlserver-integration` | Passed, 10 tests; 3 fixture tests ignored    |
| `cargo test -p app-storage --test storage`                                     | Passed, 17 tests                             |
| `bun test`                                                                     | Passed, 44 tests / 102 expectations          |
| Focused UI-adjacent Bun command                                                | Passed, 25 tests / 58 expectations           |
| `bun run typecheck` / `bun run lint` / `bun run build`                         | Passed                                       |
| Rust Clippy / fmt / `git diff --check`                                         | Passed                                       |
| `bash scripts/test_pg_harness.sh`                                              | Passed                                       |
| `bash scripts/test_sqlserver_harness.sh`                                       | Blocked: Docker daemon unavailable           |
| `python3 scripts/validate_pack.py examples/packs/hr-sql-101`                   | Blocked: `jsonschema` unavailable            |
| Desktop/manual smoke                                                           | Pending; no desktop-control tool             |
| Six-platform release artifact matrix                                           | Unverified; builds/install smoke not run     |
