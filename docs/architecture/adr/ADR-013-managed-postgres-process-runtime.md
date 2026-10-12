# ADR-013: Managed PostgreSQL Process Runtime

- Status: Accepted
- Date: 2026-10-12

## Decision

The managed PostgreSQL prototype starts an app-owned cluster with `initdb` and
`pg_ctl` using argument arrays, stores it under a caller-provided app data
directory, and exposes only a Unix-domain socket connection. TCP listening is
disabled. The runtime initializes an admin-owned instance marker and a
`NOLOGIN` constrained runner role before returning its connection string.

The runtime fails closed when the data path is not absolute, already exists,
required binaries are unavailable, the process cannot start, or marker setup
fails. Dropping the runtime stops only the process it started; data-directory
retention and platform packaging remain explicit lifecycle decisions.

## Rationale

This supplies a disposable, app-owned target for pack provisioning and judging
without credentials or accidental access to an external database. The marker
is then required by `ManagedPostgresSession`, so a loopback connection alone
cannot authorize pack import.

## Consequences

- Clean-machine packaging of `initdb`/`pg_ctl` is still T24 work.
- The runtime currently targets Unix-socket PostgreSQL environments; Windows
  process/service packaging needs a separate verified implementation.
- Tauri command wiring, crash reconciliation, and persistent lifecycle state
  remain T10/T13/T14 work.
