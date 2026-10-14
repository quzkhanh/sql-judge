# ADR-019: External SQL Server Explorer Capability

- Status: Accepted
- Date: 2026-10-13
- Requirements: REQ-014, REQ-015, REQ-021
- Acceptance: AC-014, AC-015
- Security: SEC-002, SEC-004, SEC-005, SEC-007, SEC-011
- Not applicable: REQ-024 and AC-021 cover future SQL Server judging and
  advanced external writable mode; this ADR explicitly excludes both.

## Context

SQL Server is a separate external Explorer target, not a PostgreSQL dialect.
Its T-SQL grammar, TDS protocol, catalog views, TLS behavior, result types and
permission model differ from PostgreSQL. Reusing the PostgreSQL adapter would
make those differences implicit and could route a SQL Server profile through
the wrong safety policy.

## Decision

Create a separate `judge-sqlserver` crate backed by Tiberius. It exposes only
Test Connection, catalog metadata inspection, and bounded read-only T-SQL
SELECT. It does not expose judging, provisioning, reset, drop, or generic
database lifecycle operations.

The adapter uses encrypted TLS connections and rejects `trust server
certificate` behavior. It runs queries with bounded timeout/rows/bytes and
consumes metadata and dependency result sets row-by-row rather than using an
unbounded result collector. The server login must be least-privilege; the UI
read-only label is never treated as the security boundary.

Tauri routes SQL Server profiles only to this adapter through separate typed
commands. Secrets are read from the existing vault boundary and never cross to
the frontend or logs.

## Alternatives Rejected

- Reusing `ExternalPostgresExplorer`: wrong wire protocol, catalog SQL and
  policy semantics.
- ODBC: depends on external driver installation and complicates packaging,
  deterministic tests and error mapping.
- SQLx MSSQL: not selected as the stable adapter base for this capability.
- UI-only write blocking: forged IPC and malicious T-SQL could bypass it.

## Consequences

- Tiberius/Tokio/TLS dependencies are added only to the SQL Server adapter.
- A disposable SQL Server fixture with a SELECT-only login is required before
  T21 can be declared verified.
- Unsupported T-SQL/value types produce explicit errors rather than silently
  coercing data.
- Future SQL Server judging remains a separate capability and is not implied by
  Explorer support.

## Verification Harness

`scripts/test_sqlserver_harness.sh` provisions a disposable SQL Server 2022
Developer container on a loopback-only random port. It creates a short-lived
local CA and server certificate, configures SQL Server for encrypted
connections, creates a SELECT-only login, and runs the feature-gated
`sqlserver_integration` test. The script also verifies INSERT and DROP denial
directly with the low-privilege login before running the adapter tests.

Run it with `bun run test:sqlserver-harness`. If Docker is unavailable, the
command reports T21 as blocked rather than treating an unexecuted integration
suite as passing. The container image and TLS setup follow Microsoft's SQL
Server Linux Docker and encrypted-connection guidance:

- https://learn.microsoft.com/en-us/sql/linux/install-upgrade/quickstart-install-docker
- https://learn.microsoft.com/en-us/sql/linux/security/encrypted-connections
