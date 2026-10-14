# Review Findings Fix Implementation Plan

> **For agentic workers:** Execute this plan inline with focused RED/GREEN verification. Do not commit; preserve unrelated worktree changes.

**Goal:** Close the remaining startup recovery, SQL Server capability, provisioning cleanup, traceability, and verification findings without weakening ownership or read-only safety.

**Architecture:** Tauri startup will reconcile persisted recovery journals before exposing managed state and will adopt only an already-running managed PostgreSQL process whose marker and identity match. SQL Server capability detection will be conservative and independent of application-side read-only transaction enforcement. Managed namespace setup will clean up only a newly-created namespace after setup failure, using the existing ownership verifier.

**Tech Stack:** Rust workspace, Tauri commands, `judge-postgres`, `judge-sqlserver`, `problem-pack`, Bun/Vite React frontend, Markdown traceability.

---

### Task 1: Startup Reaper and Runtime Adoption

**Files:**

- Modify: `src-tauri/src/lib.rs` startup setup and recovery helpers.
- Modify: `crates/judge-postgres/src/runtime.rs` managed runtime probe/adoption.
- Test: native Tauri tests in `src-tauri/src/lib.rs` and runtime tests in `crates/judge-postgres`.

- [x] Add failing tests proving clean/stopped runtime directories are not auto-started, matching running markers are adopted, and mismatched markers fail closed.
- [x] Verify `load_environment_state_for_startup` is used by the actual Tauri builder setup before `manage` registers `ManagedEnvironmentState`.
- [x] Add an adoption path that verifies the owned runtime marker, instance identity, socket/data directory, and running process before returning a runtime handle; never stop or delete a failed verification target.
- [x] Run focused native/runtime tests, then workspace tests and clippy.

### Task 2: Conservative SQL Server Read-Only Truth

**Files:**

- Modify: `crates/judge-sqlserver/src/lib.rs` permission probe.
- Test: `crates/judge-sqlserver/tests/policy.rs` or a focused adapter test.
- Modify: `src/features/learning/connections-screen.tsx` status text/action guards if required.
- Test: `src/lib/external-access.test.js` UI capability helper coverage.

- [x] Add a failing capability test for an escalation/write-capable role result returning `read_only=false`.
- [x] Query server roles, database roles, effective database write permissions, and object-level write permissions conservatively; treat null, unsupported, or failed checks as not verified.
- [x] Preserve `BEGIN READ ONLY`/application SQL policy enforcement for metadata and SELECT operations.
- [x] Ensure the UI says read-only access is not verified and disables metadata/SELECT whenever the backend reports false.
- [x] Run SQL Server policy tests and focused frontend tests/lint.

### Task 3: Ownership-Verified Provisioning Cleanup

**Files:**

- Modify: `crates/judge-postgres/src/lib.rs` namespace provisioning/cleanup.
- Modify: `crates/problem-pack/src/lib.rs` managed import error handling if needed.
- Modify: `src-tauri/src/lib.rs` provisioning/recovery mapping if needed.
- Test: PostgreSQL/problem-pack/Tauri regression tests covering setup failure and journal retention.

- [x] Add a failing schema/setup regression where marker creation succeeds and setup fails; assert the owned namespace is removed and no published pack/ready ledger record remains.
- [x] Reuse the ownership-checked namespace drop operation with the same environment identity; do not use an unscoped fallback drop.
- [x] Preserve the original setup error when cleanup succeeds; return a safe cleanup failure when cleanup cannot be proven and retain recovery state.
- [x] Add mismatch/unowned cleanup denial coverage where feasible.
- [x] Run focused PostgreSQL, problem-pack, and native tests.

### Task 4: Traceability and Blocker Documentation

**Files:**

- Modify: `tasks/todo.md`.
- Modify: `docs/operations/MVP-DESKTOP-SMOKE-2026-10-13.md`.

- [x] Remove stale full-lint failure wording.
- [x] Keep T16, T17, and T18 unchecked because desktop evidence is still missing.
- [x] Record SQL Server Docker unavailability and desktop smoke/manual-control blockers accurately.
- [x] Run `git diff --check`.

### Task 5: Full Verification

- [x] Run `bun test`, full Bun lint, typecheck, production build, Prettier check, and frontend diff checks.
- [x] Run workspace Rust tests, native Tauri tests, app-storage tests, SQL Server crate tests, PostgreSQL harness, workspace/native clippy with `-D warnings`, and Rustfmt checks.
- [x] Report exact pass/fail output, blocked external prerequisites, changed files, and leave the worktree uncommitted.

## Latest Review Follow-up

- [x] Direct SQL Server IPC re-probes read-only capability before metadata and bounded SELECT; native tests cover denied and allowed operation paths.
- [x] Provisioning cleanup records namespace existence before setup and preserves pre-existing matching-owned namespaces.
- [x] Recovery cleanup distinguishes absent namespaces from existing ownership mismatches.
- [x] Backup documentation states that restore replaces submission history and rejects pending recovery before mutation; it does not claim recovery rows are preserved.
- [x] Final verification results are recorded in `docs/operations/MVP-DESKTOP-SMOKE-2026-10-13.md`; Bun passes 44 tests / 102 expectations (focused UI-adjacent command: 25 / 58), app-storage passes 17 tests, native Tauri passes 41 tests, pack import passes 30 tests, and SQL Server passes 10 unit/policy tests with 3 fixture tests ignored. Workspace/native Rust, Rust/changed-file Prettier formatting, clippy, PostgreSQL harness, and diff checks pass. Docker SQL Server, Python `jsonschema`, desktop smoke, and the six-platform release artifact matrix remain blocked or unverified by documented prerequisites.
