# SQL Judge Full Project Completion Design

## Status

Approved by the user on 2026-10-13. This document covers the remaining MVP,
release, scoped writable sandbox, and SQL Server Judge work. It does not claim
that any phase is implemented until its acceptance evidence exists.

## Goals

- Close the remaining MVP gaps in T16, T18, T21, and T22.
- Verify release behavior honestly for the selected Linux, Windows, and macOS
  artifacts without claiming unrun platform builds.
- Add a separately authorized writable sandbox capability without weakening the
  existing read-only Explorer mode.
- Add a SQL Server Judge for disposable, owned environments with SQL Server
  specific policy, setup, typed results, cancellation, and cleanup.
- Preserve the existing Tauri UI scaffold, Bun toolchain, Rust crate boundaries,
  security invariants, and current document contract IDs.

## Non-Goals

- No writable access to arbitrary external or production databases.
- No generic MySQL provider.
- No hidden-test tamper-proof claims for offline packs.
- No PostgreSQL/SQL Server adapter rewrite when a focused capability extension
  is sufficient.
- No release-platform success claim without actual artifact and runtime evidence.

## Phase A: MVP Closure

### Scope

Complete T16, T18, T21, and T22, then update task and acceptance traceability.

### UI and workflow

- Packs UI supports native archive/folder selection, import status, validation
  errors, empty/loading states, and keyboard navigation.
- Environments UI shows only managed environments, exact impact previews, and
  disables destructive actions unless the record is ready and owned.
- History/Progress backup UI completes bounded metadata/history round trips and
  never includes secrets or vault references.
- Connections UI keeps PostgreSQL and SQL Server Explorer routing separate,
  supports secure URL autofill, and never probes or mutates automatically.
- A desktop smoke path covers problem selection, Run, Submit, import, reset
  confirmation, profile save, and read-only Explorer operations.

### Backend and contracts

- Keep Tauri handlers thin; preserve typed DTOs and safe error codes.
- Keep external profiles Explorer-only.
- Keep pack import validation, managed runtime ownership, atomic publication,
  and rollback in Rust/services.
- Add or update requirement-to-acceptance links for every completed task.

### Exit evidence

- AC-002 through AC-020 relevant UI and native scenarios pass.
- Real PostgreSQL and MSSQL disposable fixtures pass.
- Manual keyboard and desktop smoke results are recorded with platform and
  runtime prerequisites.

## Phase B: Release Readiness

### Workflow audit

- Preserve the generated release workflow and selected artifact matrix.
- Audit package-manager use, signing inputs, permissions, credentials, and
  generated Tauri configuration.
- Document whether managed PostgreSQL is bundled, installed separately, or an
  explicit prerequisite for each platform.

### Artifact verification

- Build each available selected artifact independently: Linux AppImage/DEB,
  Windows NSIS/Portable, and macOS Apple Silicon/Intel DMG.
- For every artifact that is actually built, verify install/launch, offline
  content browsing, managed runtime behavior, and one local Run/Submit path.
- Record unavailable runners or runtime prerequisites as limitations rather than
  marking the platform complete.

### Exit evidence

- AC-001, AC-016, AC-022 and NFR-010 have artifact-specific evidence.
- No release claim is made for an unbuilt or unlaunched platform.

## Phase C: Scoped Writable Sandbox

### Capability model

Introduce an explicit connection capability/mode separate from the current
Explorer mode:

- `Explorer`: external read-only metadata and bounded SELECT only.
- `WritableSandbox`: only disposable resources whose ownership and scope are
  verified by Rust and the database server.

The existing Explorer profile cannot be upgraded implicitly. A writable
sandbox requires a separate creation flow, explicit user opt-in, displayed
impact summary, and an ownership proof bound to a resource ID.

### Resource lifecycle

- Create or designate a disposable sandbox through a provider-specific service,
  never from arbitrary user SQL.
- Store only metadata and vault references in SQLite.
- Record provider, resource ID, ownership marker, namespace/database, lifecycle
  state, and expiration/cleanup metadata.
- Require server-side ownership verification again for reset, rebuild, and
  delete.
- Use replacement-first rebuild, durable recovery records, locks, cancellation,
  and cleanup. Fail closed on marker mismatch or ambiguous targets.
- Never fall back to `DROP DATABASE`, `DROP SCHEMA ... CASCADE`, `TRUNCATE`, or
  broad `DELETE` against an unowned target.

### Provider sequencing

PostgreSQL is implemented first using the existing managed runtime and
environment ownership model. SQL Server writable behavior is not enabled until
the SQL Server Judge provider has an equivalent disposable runtime, fixture
provisioner, runner identity, ownership marker, and cleanup proof.

### Exit evidence

- A new ADR and threat-model mapping are reviewed before implementation.
- Negative tests forge profile mode, resource ID, ownership marker, namespace,
  and confirmation values and are denied in Rust.
- Actual disposable database tests prove setup credentials are never used for
  user SQL and failed cleanup does not touch unrelated resources.

## Phase D: SQL Server Judge

### Adapter boundary

Extend `judge-sqlserver` with a capability-oriented managed runner interface;
do not reuse PostgreSQL-specific setup or dialect assumptions. Keep current
Explorer operations and error mapping stable.

The SQL Server Judge must separately implement:

- trusted schema and fixture setup using admin/provisioner credentials;
- constrained runner login/role for reference and user SQL;
- T-SQL-specific read/write policy and statement handling;
- typed SQL Server value decoding and column contracts;
- timeout, cancellation, row/result-byte limits, and rollback/cleanup;
- reference failure mapping to `JudgeError`, never `WrongAnswer`;
- per-test fixture isolation and deterministic comparison through `judge-core`.

### Runtime strategy

Development and CI use a disposable SQL Server container with TLS and a
least-privilege fixture login. Runtime packaging for end users is a separate
release decision and is not assumed from the Docker harness.

### Exit evidence

- Correlated/window or T-SQL-equivalent accepted cases and hardcoded-result
  rejection.
- Server-side denial of INSERT/UPDATE/DELETE/DDL for the runner identity.
- Metadata, typed NULL/numeric/UUID/binary values, timeout, cancellation,
  oversized results, broken reference, and cleanup tests.
- AC-004, AC-007, AC-014, AC-017, AC-021 and SEC-001/003/007/011 pass for the
  supported SQL Server Judge scope.

## Shared Testing and Quality Gates

Every implementation task follows RED -> GREEN -> REFACTOR and records the
observed failure and passing command. The affected suite must include:

- Rust unit/policy tests for pure contracts and ownership gates.
- Real PostgreSQL/MSSQL integration fixtures for database semantics and
  privilege enforcement.
- Bun TypeScript tests for parsers, typed error extraction, and UI-adjacent
  state helpers.
- `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, Rust
  tests, TypeScript typecheck, targeted ESLint, frontend build, and relevant
  release checks.
- Manual keyboard/desktop smoke evidence for UI acceptance criteria.

No skipped tests, disabled lint rules, relaxed limits, plaintext secrets, or
unverified platform claims are allowed.

## Delivery Order

1. Phase A MVP closure and traceability.
2. Phase B release audit and artifact verification available in the current
   environment.
3. Phase C writable sandbox ADR, threat-model update, and PostgreSQL provider.
4. Phase D SQL Server Judge design extension, implementation, and fixture proof.
5. Final Definition of Done review, residual-risk report, and release decision.
