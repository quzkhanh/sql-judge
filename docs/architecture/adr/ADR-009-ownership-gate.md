# ADR-009 — Verify managed environment ownership in native policy

**Status:** Accepted for P2 lifecycle work.

## Decision

`environment-manager` treats environment mode, environment ID, database
instance ID, ownership nonce and lifecycle state as authorization inputs. A
destructive request must match the durable marker exactly, target a managed
environment in a safe state, and carry explicit confirmation. External
explorer records are denied unconditionally.

This policy is pure Rust and emits no SQL. Future Tauri handlers must invoke it
after receiving the UI request and before any replacement or cleanup action.

## Consequences

- Forged frontend IDs and copied stale markers fail closed.
- Running/rebuilding/deleting environments cannot be concurrently destroyed.
- Durable ledger persistence and replacement-first lifecycle remain separate
  tasks; this crate does not authorize arbitrary external connections.

## Traceability

- Requirement: `REQ-011`
- Acceptance: `AC-011`, `AC-012`
- Security: `SEC-002`, `SEC-008`, `SEC-009`
