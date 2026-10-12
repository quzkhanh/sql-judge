# ADR-010 — Persist managed environment ownership metadata locally

**Status:** Accepted for P2 lifecycle work.

## Decision

Persist managed environment metadata in a dedicated SQLite ledger owned by the
application. The ledger stores environment ID, managed database instance ID,
pack identity, namespace, ownership nonce, mode and lifecycle state. It never
stores connection strings, passwords or runner credentials.

The storage API accepts typed `EnvironmentRecord` values from
`environment-manager`; SQLite uniqueness protects environment IDs and
namespaces. PostgreSQL ownership markers remain a separate runtime concern and
must match the local ledger before destructive operations.

## Rationale

Restart reconciliation requires durable application state, while submission
history must remain independently queryable. Keeping the ledger in a separate
table preserves that separation without turning SQLite into a practice
backend.

## Consequences

- Tauri lifecycle handlers can reload records after restart before inspecting
  managed PostgreSQL resources.
- A failed metadata write cannot partially update the ledger because each
  record replacement is transactional.
- Durable persistence does not authorize an external database; native
  ownership checks and runtime marker verification remain mandatory.

## Traceability

- Requirement: `REQ-011`
- Acceptance: `AC-010`, `AC-011`, `AC-012`
- Security: `SEC-002`, `SEC-004`, `SEC-008`, `SEC-009`
