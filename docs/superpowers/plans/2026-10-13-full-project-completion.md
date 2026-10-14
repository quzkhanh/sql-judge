# SQL Judge Full Project Completion Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Close the remaining MVP gaps, verify selected release artifacts, add a separately authorized writable sandbox, and implement a disposable SQL Server Judge without weakening existing read-only or ownership boundaries.

**Architecture:** Work in four ordered phases. First close and evidence the current MVP. Then audit and verify release packaging. Next add capability-scoped writable sandbox metadata and PostgreSQL lifecycle support. Finally extend the SQL Server adapter with a separate managed Judge runner, fixture setup, constrained execution, and cleanup. Existing Explorer, managed PostgreSQL, `judge-core`, Tauri command, and Bun boundaries remain intact.

**Tech Stack:** Tauri 2, React 19, TypeScript, Bun, shadcn/ui, Rust 2024, Cargo workspace, PostgreSQL, Tiberius/Microsoft SQL Server, SQLite, Docker-based integration fixtures, existing GitHub Actions release workflow.

---

## File Map

### Existing files to modify

- `tasks/todo.md`: task status and evidence for T16/T18/T21/T22/T23-T27.
- `docs/operations/G2-LOCAL-RUN-SUBMIT-2026-10-12.md`: desktop smoke evidence.
- `docs/operations/BUILD-RELEASE.md`: artifact and runtime evidence.
- `docs/security/THREAT-MODEL.md`: writable sandbox and SQL Server Judge controls.
- `docs/contracts/CONNECTIONS-ENVIRONMENTS.md`: capability and lifecycle contract.
- `src/features/learning/packs-screen.tsx`: import states and keyboard path.
- `src/features/learning/environments-screen.tsx`: managed-only lifecycle and impact review.
- `src/features/learning/history-screen.tsx`: backup and restore states.
- `src/features/learning/progress-screen.tsx`: backup and progress states.
- `src/features/learning/connections-screen.tsx`: Explorer versus writable capability UI.
- `src/lib/judge-api.ts`: typed capability/lifecycle IPC wrappers.
- `src-tauri/src/lib.rs`: thin command wiring and Rust authorization gates.
- `crates/app-storage/src/lib.rs`: versioned connection/sandbox metadata migration.
- `crates/environment-manager/src/ledger.rs`: owned resource lifecycle state and recovery.
- `crates/judge-postgres/src/lib.rs`: bounded managed sandbox provider extension.
- `crates/judge-sqlserver/src/lib.rs`: Explorer-preserving Judge runner extension.
- `crates/judge-sqlserver/tests/sqlserver_integration.rs`: disposable SQL Server Judge scenarios.
- `scripts/test_sqlserver_harness.sh`: TLS, provisioning, runner permissions, and cleanup fixture.

### New files to create

- `docs/architecture/adr/ADR-020-scoped-writable-sandbox.md`
- `docs/architecture/adr/ADR-021-sqlserver-judge.md`
- `docs/operations/MVP-DESKTOP-SMOKE-2026-10-13.md`
- `docs/operations/RELEASE-MATRIX-2026-10-13.md`
- `crates/connection-manager/Cargo.toml`
- `crates/connection-manager/src/lib.rs`
- `crates/connection-manager/tests/capabilities.rs`
- `crates/judge-postgres/tests/sandbox_integration.rs`
- `crates/judge-sqlserver/src/runner.rs`
- `crates/judge-sqlserver/tests/judge_integration.rs`
- `examples/packs/sqlserver-tutorial/pack.json`
- `examples/packs/sqlserver-tutorial/schema/001_tables.sql`
- `examples/packs/sqlserver-tutorial/problems/basic/problem.json`
- `examples/packs/sqlserver-tutorial/problems/basic/reference.sql`
- `examples/packs/sqlserver-tutorial/problems/basic/tests/basic.sql`

## Phase A: MVP Closure

### Task 1: Close UI and traceability gaps

**Files:**

- Modify: `src/features/learning/packs-screen.tsx`
- Modify: `src/features/learning/environments-screen.tsx`
- Modify: `src/features/learning/history-screen.tsx`
- Modify: `src/features/learning/progress-screen.tsx`
- Modify: `src/lib/judge-api.ts`
- Test: `src/features/learning/environment-impact.test.ts` and existing Bun tests

- [ ] **Step 1: Write failing UI-adjacent tests** for safe pack error extraction, managed-only environment action eligibility, and backup state messages. Run `bun test src/features/learning/environment-impact.test.ts src/lib/pack-errors.test.js src/lib/connection-errors.test.js`; expected failures must identify the missing state contract.
- [ ] **Step 2: Implement the smallest UI changes**: preserve the current dialog picker, show loading/error/success state, keep external records out of destructive controls, and ensure backup restore reports bounded failure without exposing snapshot content.
- [ ] **Step 3: Run focused tests** with `bun test src/features/learning/environment-impact.test.ts src/lib/pack-errors.test.js src/lib/connection-errors.test.js`; expected result is all tests passing.
- [ ] **Step 4: Run `bun run typecheck` and `bun run build`**; fix only affected type or rendering errors.
- [ ] **Step 5: Record manual keyboard checks** in `docs/operations/MVP-DESKTOP-SMOKE-2026-10-13.md`: navigate problem list, editor, Run, Submit, pack import, environment impact dialog, profile save, and Explorer without a mouse.
- [ ] **Step 6: Update `tasks/todo.md`** to mark T16/T18/T21/T22 only with the actual commands and desktop evidence collected.

### Task 2: Remove current lint blockers

**Files:**

- Modify: `src/components/chart-area-interactive.tsx`
- Modify: `src/components/debug-panel.tsx`
- Modify: `src/hooks/use-mobile.ts`

- [ ] **Step 1: Run the failing baseline** with `bun run lint`; capture the existing five rule failures and line numbers.
- [ ] **Step 2: Add no new behavior tests** because these are React effect/lint correctness fixes; preserve behavior by replacing synchronous effect state initialization with lazy state or event-driven updates.
- [ ] **Step 3: Implement the minimal effect-safe changes** without `eslint-disable`, `useMemo`, or unrelated refactors.
- [ ] **Step 4: Run `bun run lint`, `bun run typecheck`, and `bun run build`;** all must pass before Phase A closes.

## Phase B: Release Readiness

### Task 3: Audit release workflow and runtime prerequisite

**Files:**

- Inspect/modify: `.github/workflows/release.yml`
- Inspect/modify: `src-tauri/tauri.conf.json`
- Modify: `docs/operations/BUILD-RELEASE.md`
- Create: `docs/operations/RELEASE-MATRIX-2026-10-13.md`

- [ ] **Step 1: Write the failing audit checklist** in `RELEASE-MATRIX-2026-10-13.md` for Linux AppImage/DEB, Windows NSIS/Portable, and macOS Apple Silicon/Intel DMG, including runtime availability, signing, and secret-scrubbing checks.
- [ ] **Step 2: Run the workflow/config audit** with `git diff -- .github/workflows/release.yml src-tauri/tauri.conf.json`, `bun pm ls`, and `cargo metadata --format-version 1`; record package manager, bundle targets, signing variables, and managed PostgreSQL assumptions.
- [ ] **Step 3: Preserve generated release structure** and make only additive corrections needed for the documented matrix; do not claim an artifact that cannot be built on the current runner.
- [ ] **Step 4: Run available packaging commands** from the existing workflow, archive command output and artifact paths, and document unavailable OS runners as unverified.
- [ ] **Step 5: Run `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `bun run lint`, `bun run typecheck`, and `bun run build` before marking T23 complete.

### Task 4: Verify managed PostgreSQL distribution decision

**Files:**

- Inspect/modify: `docs/operations/RUNBOOK.md`
- Modify: `docs/operations/BUILD-RELEASE.md`
- Inspect/modify: `src-tauri/src/lib.rs`
- Inspect/modify: `crates/judge-postgres/src/orchestrator.rs`

- [ ] **Step 1: Add a failing clean-machine checklist** covering missing runtime, runtime startup, Unix socket/loopback binding, data directory permissions, and shutdown cleanup.
- [ ] **Step 2: Verify current behavior** with a missing-runtime test and a fresh runtime start/stop test; never silently download or connect to an external database.
- [ ] **Step 3: Implement only the selected distribution behavior**: either explicit install prerequisite messaging or a verified bundled runtime path per platform; do not add unreviewed download logic.
- [ ] **Step 4: Run native runtime tests and update `BUILD-RELEASE.md`** with platform-specific evidence and prerequisites.

## Phase C: Scoped Writable Sandbox

### Task 5: Add capability and ownership contracts

**Files:**

- Create: `crates/connection-manager/Cargo.toml`
- Create: `crates/connection-manager/src/lib.rs`
- Create: `crates/connection-manager/tests/capabilities.rs`
- Modify: root `Cargo.toml`
- Modify: `crates/app-storage/src/lib.rs`
- Modify: `crates/app-storage/tests/storage.rs`
- Modify: `docs/contracts/CONNECTIONS-ENVIRONMENTS.md`
- Create: `docs/architecture/adr/ADR-020-scoped-writable-sandbox.md`
- Modify: `docs/security/THREAT-MODEL.md`

- [ ] **Step 1: Write RED tests** for `Explorer` versus `WritableSandbox`, forged capability escalation, missing resource ID, missing ownership marker, invalid lifecycle transition, and secret-free serialization.
- [ ] **Step 2: Run the focused crate tests** with `cargo test -p connection-manager`; expected failure must be the absent capability contract.
- [ ] **Step 3: Implement pure capability types** in `connection-manager` with validated provider/mode/resource ID/ownership marker/lifecycle state. The crate must not depend on Tauri, SQL drivers, SQLite, or OS APIs.
- [ ] **Step 4: Add an additive SQLite migration** for mode, provider resource ID, ownership marker, namespace/database scope, lifecycle state, and cleanup metadata. Existing Explorer rows migrate to `Explorer` and remain non-destructive.
- [ ] **Step 5: Run `cargo test -p connection-manager -p app-storage`** and verify no secret/password/vault value appears in serialized metadata.
- [ ] **Step 6: Finalize ADR-020 and threat-model mapping** before any writable command is exposed.

### Task 6: Implement PostgreSQL writable sandbox lifecycle

**Files:**

- Modify: `crates/environment-manager/src/ledger.rs`
- Modify: `crates/environment-manager/tests/lifecycle.rs`
- Modify: `crates/environment-manager/tests/ownership.rs`
- Modify: `crates/judge-postgres/src/lib.rs`
- Create: `crates/judge-postgres/tests/sandbox_integration.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src/lib/judge-api.ts`
- Modify: `src/features/learning/connections-screen.tsx`
- Modify: `src/features/learning/environments-screen.tsx`

- [ ] **Step 1: Write RED ownership/lifecycle tests**: direct IPC with Explorer mode, forged resource ID, forged marker, cross-namespace reset, missing confirmation, running-judge reset, and cleanup failure must all be denied or preserve the old resource.
- [ ] **Step 2: Run `cargo test -p environment-manager -p judge-postgres --test sandbox_integration`;** confirm the failures are authorization/lifecycle assertions rather than compile errors.
- [ ] **Step 3: Implement Rust-side capability checks** before provider access, using an owned resource ID and marker verified against the database. UI confirmation is advisory only.
- [ ] **Step 4: Implement replacement-first rebuild and cleanup** using existing recovery/ledger patterns. No fallback destructive SQL is allowed for unknown targets.
- [ ] **Step 5: Add Tauri commands and typed frontend wrappers** for create/inspect/reset/rebuild/delete only for `WritableSandbox`; retain Explorer commands unchanged.
- [ ] **Step 6: Add UI impact review** showing provider, resource, namespace, data-loss scope, and confirmation text; disable controls until state is ready and capability is writable.
- [ ] **Step 7: Run real disposable PostgreSQL integration tests**, native tests, Bun tests, `cargo clippy --workspace --all-targets -- -D warnings`, and frontend build.

## Phase D: SQL Server Judge

### Task 7: Define SQL Server Judge contract and fixture pack

**Files:**

- Create: `docs/architecture/adr/ADR-021-sqlserver-judge.md`
- Modify: `crates/judge-sqlserver/Cargo.toml`
- Create: `examples/packs/sqlserver-tutorial/pack.json`
- Create: `examples/packs/sqlserver-tutorial/schema/001_tables.sql`
- Create: `examples/packs/sqlserver-tutorial/problems/basic/problem.json`
- Create: `examples/packs/sqlserver-tutorial/problems/basic/reference.sql`
- Create: `examples/packs/sqlserver-tutorial/problems/basic/tests/basic.sql`
- Modify: `crates/judge-sqlserver/tests/policy.rs`

- [ ] **Step 1: Write RED contract tests** for runner identity separation, T-SQL fixture setup restrictions, accepted bounded SELECT, hardcoded result rejection, broken reference => `JudgeError`, and unsupported DDL/procedure/function cases.
- [ ] **Step 2: Run `cargo test -p judge-sqlserver --all-targets`;** confirm new tests fail because the managed Judge contract is absent.
- [ ] **Step 3: Define the SQL Server-specific contract** in ADR-021: admin/provisioner is setup-only, runner is least privilege, each test starts from equivalent fixture state, and Explorer remains read-only.
- [ ] **Step 4: Validate the example pack** with `python3 scripts/validate_pack.py examples/packs/sqlserver-tutorial` and structural Rust pack tests.

### Task 8: Implement SQL Server managed Judge runner

**Files:**

- Create: `crates/judge-sqlserver/src/runner.rs`
- Modify: `crates/judge-sqlserver/src/lib.rs`
- Create: `crates/judge-sqlserver/tests/judge_integration.rs`
- Modify: `crates/judge-sqlserver/tests/sqlserver_integration.rs`
- Modify: `scripts/test_sqlserver_harness.sh`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Add RED disposable fixture tests** for schema/fixture setup, runner login permissions, typed values, result limits, timeout, cancellation, cleanup, and reference failure mapping.
- [ ] **Step 2: Extend the harness** with separate provisioner and runner credentials, owned fixture database/resource marker, TLS, and server-side denial of runner DML/DDL. Keep passwords out of output.
- [ ] **Step 3: Implement `SqlServerJudgeRunner`** with explicit setup and runner sessions, T-SQL-specific setup statements, bounded execution, and typed result conversion. Never pass admin credentials into user/reference execution.
- [ ] **Step 4: Implement per-test cleanup** with rollback/drop only for the verified owned fixture resource; preserve unrelated databases and fail closed on marker mismatch.
- [ ] **Step 5: Add thin Tauri orchestration** that loads immutable pack metadata, validates capability/resource ownership, invokes the runner, maps errors, and persists only typed history.
- [ ] **Step 6: Run `DOCKER_HOST=unix:///var/run/docker.sock bun run test:sqlserver-harness`** and `cargo test -p judge-sqlserver --all-targets`; expected result includes all Explorer and Judge tests passing.

### Task 9: SQL Server Judge UI and end-to-end acceptance

**Files:**

- Modify: `src/lib/judge-api.ts`
- Modify: `src/features/learning/connections-screen.tsx`
- Modify: `src/features/learning/learning-workspace.tsx`
- Modify: `src/components/debug-panel.tsx`
- Modify: `docs/operations/MVP-DESKTOP-SMOKE-2026-10-13.md`

- [ ] **Step 1: Write RED frontend tests** for capability visibility, unavailable runtime state, Judge versus Explorer operation labels, bounded result state, timeout/cancel state, and no password/raw SQL in debug events.
- [ ] **Step 2: Add typed Judge commands and event DTOs** without exposing setup credentials or fixture internals.
- [ ] **Step 3: Add UI state** for explicit local SQL Server Judge only; external profiles remain Explorer-only and cannot display Judge controls.
- [ ] **Step 4: Run frontend tests, typecheck, targeted ESLint, and build.**
- [ ] **Step 5: Run manual desktop acceptance** for Run/Submit, cancellation, timeout, wrong answer, accepted result, and external destructive-control absence.

## Final Verification and Handoff

### Task 10: Definition of Done and residual-risk report

**Files:**

- Modify: `tasks/todo.md`
- Modify: `docs/requirements/ACCEPTANCE.md`
- Modify: `docs/security/THREAT-MODEL.md`
- Modify: `docs/operations/BUILD-RELEASE.md`
- Create: `docs/operations/FINAL-VERIFICATION-2026-10-13.md`

- [ ] **Step 1: Run full verification**:
  - `cargo fmt --all -- --check`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo test --workspace`
  - `cargo test --manifest-path src-tauri/Cargo.toml --lib`
  - `bun test src/lib src/features`
  - `bun run lint`
  - `bun run typecheck`
  - `bun run build`
  - `DOCKER_HOST=unix:///var/run/docker.sock bun run test:pg-harness`
  - `DOCKER_HOST=unix:///var/run/docker.sock bun run test:sqlserver-harness`
- [ ] **Step 2: Check each AC/SEC item** against fresh command output or recorded manual/platform evidence; mark unverified platform items explicitly.
- [ ] **Step 3: Run secret and destructive-operation review**: search logs, backup JSON, SQLite serialization, command args, and SQL paths for passwords/connection strings; verify no unowned destructive fallback exists.
- [ ] **Step 4: Record residual risks** including offline-pack inspectability, platform runners not available locally, runtime distribution prerequisites, and any excluded SQL Server packaging behavior.
- [ ] **Step 5: Update `tasks/todo.md` only after evidence exists** and provide the required handoff: task ID, files changed, requirements, commands/results, security assumptions, limitations, and next safe task.
