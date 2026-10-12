# Atomic Problem Pack Import Implementation Plan

> **For agentic workers:** Implement this plan task-by-task with TDD checkpoints.

**Goal:** Add a Rust `problem-pack` crate that structurally validates pack files, validates PostgreSQL references in a pre-created managed session, and atomically publishes immutable content-addressed packs.

**Architecture:** `PackData` owns bounded, path-checked bytes and converts strict JSON manifests into validated contracts. `PackImporter` receives only `&mut PostgresSession` and a runner role, executes trusted schema/fixture setup plus reference queries through the existing rollback-scoped managed fixture, and publishes only after all runtime checks pass. The registry is the digest directory tree; no mutable index is written.

**Tech Stack:** Rust 2024, `serde`/`serde_json`, `sha2`, existing `judge-core` and `judge-postgres`, standard filesystem atomic rename, local PostgreSQL integration tests.

---

### Task 1: Structural validation RED/GREEN

**Files:**
- Create: `crates/problem-pack/Cargo.toml`
- Create: `crates/problem-pack/src/lib.rs`
- Create: `crates/problem-pack/tests/import.rs`
- Modify: `Cargo.toml`

- [x] Write tests for the sample pack, unknown JSON fields, traversal/symlink sources, duplicate IDs, mismatched dialect, and undeclared files.
- [x] Run `cargo test -p problem-pack`; assert the structural tests fail for the incomplete importer.
- [x] Implement bounded directory loading, safe POSIX paths, strict `deny_unknown_fields` manifests, exact declared-file checks, UTF-8/NUL SQL checks, and hash calculation.
- [x] Rerun focused structural tests until green.

### Task 2: Runtime validation and atomic publication RED/GREEN

**Files:**
- Modify: `crates/problem-pack/src/lib.rs`
- Modify: `crates/problem-pack/tests/import.rs`

- [x] Add real disposable local PostgreSQL tests for broken reference cleanup, successful publication, same-digest idempotence, and conflicting immutable `(id, version)` rejection.
- [x] Run the focused runtime tests to observe expected failures before implementation.
- [x] Execute each trusted fixture/reference through `PostgresSession::with_managed_fixture`; reject non-PostgreSQL packs and mismatched reference output shape.
- [x] Stage files in a temporary directory, publish with same-filesystem rename under `sha256/<digest>`, and clean all stage/registry residue on failure.
- [x] Rerun focused runtime tests and inspect the registry after failures.

### Task 3: Quality verification

**Files:**
- No additional files.

- [x] Run `cargo fmt --all -- --check`.
- [x] Run `cargo clippy -p problem-pack --all-targets -- -D warnings`.
- [x] Run `cargo test -p problem-pack` and the affected `judge-postgres` tests.
- [x] Review the diff for raw connection-string acceptance, admin execution of reference/user SQL, path escapes, and partial publication.
