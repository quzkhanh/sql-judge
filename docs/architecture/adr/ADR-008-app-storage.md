# ADR-008 — Keep SQLite history separate from SQL practice databases

**Status:** Accepted for P2 persistence.

## Decision

Add `crates/app-storage` around SQLite for local submission metadata and
history only. Migrations create versioned submission and per-test result tables
with pack hash, engine version and policy version. The API intentionally has no
password, connection-string, SQL-backend or secret fields.

Writes use one SQLite transaction so a failed submission insert cannot leave
partial per-test rows. WAL mode is enabled for restart-safe local history.

## Consequences

- SQLite is never passed to a SQL exercise runner.
- History can be reopened after restart and independently backed up later.
- SQLite library versions and native build requirements are isolated from the
  Tauri shell and PostgreSQL adapter.

## Traceability

- Requirement: `REQ-013`
- Acceptance: `AC-013`
- Security: `SEC-004`
