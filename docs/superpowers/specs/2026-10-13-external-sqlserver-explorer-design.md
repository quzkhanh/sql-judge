# External SQL Server Explorer Design

## Scope

T21 adds Microsoft SQL Server external Explorer support for AC-014, AC-015 and
SEC-011: Test Connection, catalog metadata, and bounded read-only T-SQL SELECT.
It excludes judge execution, writes, reset, provisioning, drop, writable
external namespaces and generic PostgreSQL code paths.

## Adapter Boundary

`judge-sqlserver` is a standalone Rust crate using Tiberius. It owns TDS
connection setup, TLS configuration, T-SQL policy, SQL Server catalog queries,
typed row decoding, timeout handling, result bounds and safe error mapping.
It may depend on `judge-core` result/value contracts but not on Tauri,
app-storage, PostgreSQL drivers or frontend code.

Tauri commands are thin orchestration:

- `test_external_sqlserver`
- `list_external_sqlserver_metadata`
- `execute_external_sqlserver_select`

They load the SQL Server profile and secret natively, construct the adapter in
memory, scrub errors, and return typed DTOs. PostgreSQL profiles cannot enter
these commands, and SQL Server profiles cannot enter PostgreSQL commands.

## TLS and Connection Policy

- Encrypted TLS is required.
- Certificate validation is enabled by default.
- `disable` and `trust server certificate` are rejected.
- Wrong credentials, TLS failures and unreachable endpoints map to stable error
  codes without raw driver text, passwords or connection strings.

## T-SQL Policy

The adapter rejects empty, multi-statement and non-SELECT/CTE input. It rejects
`INSERT`, `UPDATE`, `DELETE`, `MERGE`, `DROP`, `ALTER`, `TRUNCATE`, `EXEC`,
`EXECUTE`, `xp_cmdshell`, `OPENROWSET`, `OPENDATASOURCE`, `WAITFOR`, and
cross-database references as defense-in-depth. The SQL Server login must also
be created with SELECT-only permissions because parser checks are not a
sandbox.

Every accepted query runs in a bounded transaction and is rolled back after
row decoding. Statement timeout, max rows and max result bytes are enforced by
the adapter. No setup or destructive SQL is ever issued against the external
database.

## Metadata and Values

Use fixed, parameterized `INFORMATION_SCHEMA`/`sys` catalog queries for visible
schemas, tables and columns. Apply metadata row/byte limits. Map supported SQL
Server values to the existing typed result contract: NULL, bit, signed/unsigned
integers, decimal, float, text, binary, date/time and GUID. Return a stable
unsupported-type error for other values.

## UI

Connections uses the existing profile form and vault flow. Selecting a SQL
Server profile enables the SQL Server Explorer actions and displays a clear
engine-specific status. It shows Test Connection, metadata and bounded SELECT
states with keyboard-accessible labels, loading, empty, timeout, limit and
error views. It never shows reset, drop, write or judge controls for the
profile.

## Verification

- RED tests for T-SQL policy, TLS rejection, engine routing, value mapping and
  bounded results.
- Disposable SQL Server integration fixture with a SELECT-only login verifies
  Test Connection, metadata, valid SELECT, and server-side denial of INSERT and
  DROP.
- Negative cases cover `EXEC`, `xp_cmdshell`, cross-database access,
  multi-statements, `WAITFOR`, wrong credentials, invalid TLS, offline server,
  oversized results and unsupported types.
- Secret canaries are absent from SQLite, backups, debug events, Tauri DTOs and
  error messages.
- Run workspace tests/clippy/fmt, Tauri tests and frontend typecheck/build.
- If the disposable SQL Server fixture is unavailable, report T21 as blocked
  rather than marking integration acceptance as passed.
