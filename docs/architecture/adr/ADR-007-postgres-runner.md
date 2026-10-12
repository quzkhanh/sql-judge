# ADR-007 — Use a dedicated synchronous PostgreSQL adapter for P1

**Status:** Accepted for local disposable runner work.

## Decision

Add `crates/judge-postgres` using the official `postgres` 0.19.14 client. The
adapter accepts a connection string supplied by native orchestration, never
logs it, and executes only through the low-privilege runner identity. It sets a
per-query PostgreSQL `statement_timeout`, rejects multiple/write/dangerous SQL
as defense-in-depth, decodes supported PostgreSQL scalar types into
`judge-core::SqlValue`, and fails closed on unsupported types or result caps.

Integration tests create an ephemeral local PostgreSQL cluster with `initdb`
and tear it down in the test process. They never use a user-configured
external database.

## Trade-offs and limits

- The synchronous client keeps the initial adapter deterministic and avoids
  adding async orchestration before Tauri services exist.
- PostgreSQL `numeric` is reported as an explicit unsupported codec until an
  exact decimal representation is selected; it is not coerced to float.
- Result caps are checked before rows are returned to the domain, but a later
  streaming implementation is still needed to reduce server/client buffering
  for hostile large results.

## Traceability

- Requirements: `REQ-004`, `REQ-007`, `REQ-012`, `REQ-017`
- Acceptance: `AC-003`, `AC-007`
- Security: `SEC-001`, `SEC-003`, `SEC-007`, `SEC-010`
