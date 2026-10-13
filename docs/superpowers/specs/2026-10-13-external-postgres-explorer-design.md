# External PostgreSQL Explorer Design

## Scope

T20 implements PostgreSQL external Explorer support for AC-014 and AC-015:
Test Connection, schema/table/column metadata, and bounded read-only SELECT.
It does not implement judging, reset, provisioning, writes, SQL Server, or
external writable namespaces.

## Boundaries

`ExternalPostgresExplorer` is a separate capability in `judge-postgres` and
does not expose managed ownership, fixture, namespace, reset, or provisioning
methods. Tauri commands remain thin orchestration and map adapter failures to
safe typed responses.

The frontend sends a profile ID and, for SELECT, SQL plus bounded limits. It
never sends a password. Native code loads profile metadata from SQLite and the
secret from the credential vault/session fallback, constructs the adapter in
memory, performs one operation, and drops the connection.

## TLS Policy

- `verify_full`: system certificate roots and hostname validation required.
- `verify_ca`: certificate chain validation required; hostname matching is the
  only explicit relaxation.
- `require`: encrypted, certificate-verified connection.
- `disable`: rejected until an explicit local-development opt-in contract exists.

TLS, authentication, and server errors are mapped without returning raw driver
messages, credentials, connection strings, or sensitive endpoint details.

## Operations

### Test Connection

Connect with the selected profile and run a fixed, parameterized read-only
probe. Return only database name, current user, server version, and capability
flags. No setup, write, extension, or destructive probe is performed.

### Metadata

Use fixed PostgreSQL catalog queries for visible schemas, relations, and column
types. Apply explicit row/byte/time bounds. Do not interpolate schema/table
names into SQL; use parameters or validated identifiers.

### Read-only SELECT

Reject empty, multi-statement, and non-SELECT input as defense-in-depth. Start
a `READ ONLY` transaction, set bounded statement/idle timeouts, execute the
query, decode typed columns/rows, enforce max rows and result bytes, then
rollback/close. The external account must also be least-privileged; a UI toggle
is not treated as a security boundary.

## Error Contract

Native responses use stable codes:

- `profile_invalid`
- `secret_unavailable`
- `tls_failed`
- `authentication_failed`
- `server_unreachable`
- `query_rejected`
- `query_timeout`
- `result_limit_exceeded`
- `external_database_error`

Messages are actionable but scrub raw driver text, passwords, connection
strings, SQL text, and confidential catalog details.

## UI

Connections shows an explicit `Test connection` action, then metadata and
SELECT panels only after the user chooses a profile. It renders success,
loading, empty, timeout, limit, and error states and supports keyboard labels
and focus. SQL Server profiles are visibly deferred to T21. No external
profile exposes reset, drop, write, judge, or auto-probe actions.

## Verification

- RED tests for external adapter policy, TLS rejection, and bounded read-only
  execution.
- Disposable PostgreSQL fixture with a low-privilege account verifies test,
  metadata, valid SELECT, and server-side rejection of INSERT/UPDATE/DROP.
- Negative cases cover multi-statements, `pg_sleep`, oversized results, wrong
  credentials, invalid certificates/hostnames, offline server, and SQL Server
  profile routing.
- Secret canaries are absent from SQLite, backups, debug output, Tauri results,
  and error messages.
- Run Rust workspace tests/clippy/fmt plus Tauri tests and frontend
  typecheck/build.
