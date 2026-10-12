# Managed Pack Runtime Boundary Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Require problem-pack runtime validation to use an app-owned managed PostgreSQL capability rather than an arbitrary local `PostgresSession`.

**Architecture:** Keep `PostgresSession` as the general query primitive, and add a narrower `ManagedPostgresSession` wrapper that can only be constructed after loopback and admin-owned runtime-marker verification. Change `PackImporter` to consume the wrapper, so publication is capability-gated without changing ordinary judge queries.

**Tech Stack:** Rust workspace, `postgres` crate, disposable local PostgreSQL integration tests, Cargo test.

---

### Task 1: Add the failing managed-runtime acceptance test

**Files:**
- Modify: `crates/problem-pack/tests/import.rs:1-317`
- Test: `crates/problem-pack/tests/import.rs`

- [ ] **Step 1: Add an unmarked-database rejection test.**

Add a test that connects to the disposable database through the future managed constructor and asserts the stable connection/policy error:

```rust
#[test]
fn managed_pack_session_rejects_database_without_runtime_marker() {
    let server = LocalPostgres::start();
    let error = judge_postgres::ManagedPostgresSession::connect(
        &server.connection_string(),
        "test-instance",
    )
    .expect_err("unmarked database must not become a managed pack target");
    assert_eq!(error, judge_postgres::QueryExecutionError::OwnershipVerificationFailed);
}
```

Add `LocalPostgres::connection_string()` returning the existing Unix-socket connection string without changing the database setup.

- [ ] **Step 2: Run the test and verify the expected RED failure.**

Run:

```bash
cargo test -p problem-pack --test import managed_pack_session_rejects_database_without_runtime_marker
```

Expected: compilation fails because `ManagedPostgresSession` and `connection_string` do not exist yet. This confirms the test targets the missing capability.

### Task 2: Implement the capability and marker verification

**Files:**
- Modify: `crates/judge-postgres/src/lib.rs:68-80`
- Modify: `crates/problem-pack/src/lib.rs:1-20,272-282,365-390`

- [ ] **Step 1: Add the minimal managed-session wrapper.**

Define a public wrapper beside `PostgresSession`:

```rust
pub struct ManagedPostgresSession {
    session: PostgresSession,
    instance_id: String,
}
```

Implement `connect(connection_string, expected_instance_id)` by first calling the existing local-only `PostgresSession::connect`, then querying an admin-owned marker table with a parameterized query. Return `OwnershipVerificationFailed` unless exactly one marker row matches the expected ID. Reject an empty expected ID with `PolicyViolation`. Do not include connection strings or SQL in errors.

Expose only the managed operations needed by pack validation:

```rust
pub fn with_managed_fixture<T, F>(
    &mut self,
    runner_role: &str,
    schema_sql: &str,
    fixture_sql: &str,
    operation: F,
) -> Result<T, QueryExecutionError>
where
    F: FnOnce(&mut PostgresSession) -> Result<T, QueryExecutionError>;
```

The method delegates to the inner session; the wrapper prevents callers from constructing pack imports without the marker check.

- [ ] **Step 2: Change `PackImporter` to require the wrapper.**

Replace the `PostgresSession` parameter in `PackImporter::import` and `validate_runtime` with `ManagedPostgresSession`. Call the wrapper’s fixture method and pass the inner `PostgresSession` to the existing reference-query closure. Keep publication after validation only.

- [ ] **Step 3: Run the focused test and verify GREEN.**

Run:

```bash
cargo test -p problem-pack --test import managed_pack_session_rejects_database_without_runtime_marker
```

Expected: the test passes, with the unmarked disposable database rejected before pack import.

### Task 3: Mark the disposable managed database and preserve pack-import tests

**Files:**
- Modify: `crates/problem-pack/tests/import.rs:285-317`
- Modify: `crates/judge-postgres/tests/integration.rs` test setup helper if it shares the same disposable server

- [ ] **Step 1: Create an admin-owned runtime marker in the pack-import harness.**

Extend the trusted `psql` setup SQL with:

```sql
CREATE TABLE public.__sql_judge_runtime (instance_id text PRIMARY KEY);
INSERT INTO public.__sql_judge_runtime (instance_id) VALUES ('test-instance');
REVOKE ALL ON TABLE public.__sql_judge_runtime FROM PUBLIC;
```

Do not grant the marker table to `sqljudge_runner`.

- [ ] **Step 2: Update successful import tests to use the managed constructor.**

Change `admin_session()` to return `ManagedPostgresSession` and call `connect(..., "test-instance")`. The existing tests must continue to exercise runtime validation, cleanup, immutable publication, and idempotency through the capability.

- [ ] **Step 3: Run the focused import suite.**

Run:

```bash
cargo test -p problem-pack --test import
```

Expected: all import tests pass, including marker rejection, runtime failure cleanup, successful publication, idempotency, and immutable-version conflict behavior.

### Task 4: Verify the workspace and commit the boundary

**Files:**
- Review: `crates/judge-postgres/src/lib.rs`
- Review: `crates/problem-pack/src/lib.rs`
- Review: `crates/problem-pack/tests/import.rs`

- [ ] **Step 1: Run formatting and affected tests.**

```bash
cargo fmt --all
cargo test -p judge-postgres --all-targets
cargo test -p problem-pack --all-targets
```

Expected: formatting produces no diff after the first run and all affected Rust tests pass.

- [ ] **Step 2: Run the full Rust quality gate.**

```bash
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
```

Expected: all tests and clippy pass without warnings.

- [ ] **Step 3: Review the diff for security regressions.**

Confirm the diff contains no credentials, no connection-string logging, no dynamic identifier interpolation from unvalidated input, no publication before runtime validation, and no grants on the runtime marker to `PUBLIC` or the fixture runner.

- [ ] **Step 4: Commit only the boundary implementation.**

```bash
git add crates/judge-postgres/src/lib.rs crates/problem-pack/src/lib.rs crates/problem-pack/tests/import.rs
git commit -m "feat: gate pack import on managed postgres runtime"
```
