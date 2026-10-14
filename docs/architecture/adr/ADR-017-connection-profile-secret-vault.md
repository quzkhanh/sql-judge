# ADR-017: Connection Profiles and OS Secret Vault

- Status: Accepted
- Date: 2026-10-12
- Requirements: REQ-014, REQ-015, REQ-020
- Acceptance: AC-014, AC-015, AC-020
- Security: SEC-002, SEC-004, SEC-005
- Not applicable: AC-018 and SEC-009 cover structured logging and crash cleanup,
  respectively; neither is defined by this profile/vault decision.

## Context

SQL Judge needs reusable connection profiles for read-only external exploration.
Connection metadata is useful across sessions, but passwords and tokens must not
be stored in SQLite, JSON backups, frontend storage, logs, or error messages.

## Decision

Persist only validated profile metadata in SQLite: an opaque profile ID, display
name, engine, host, port, database, username, TLS policy, and an optional opaque
secret reference. Store passwords/tokens in the platform credential vault using
the profile ID as the account key. Native commands return metadata only.

The first implementation uses the Rust `keyring` crate with platform backends.
If the OS vault is unavailable, the app may use an in-memory session secret, but
must never fall back to plaintext persistence. External profiles remain
Explorer/read-only; connection execution is a separate capability-gated change.

Backups exclude profiles' secret values and secret references. Deleting a profile
must delete its vault entry and then its metadata, with failure reported rather
than silently leaving an untracked credential.

## Alternatives Rejected

- Encrypted SQLite: introduces app-key lifecycle and recovery risks before the
  product has a secure key-management contract.
- Plaintext SQLite/frontend storage: violates the credential storage invariant.
- Always prompt per session: safe but prevents reusable profiles and adds avoidable
  friction when an OS vault is available.

## Consequences

- Metadata can be backed up without credentials.
- Vault availability becomes an explicit capability and error state.
- Native tests require an injected in-memory vault backend; production uses the
  OS-backed implementation.
- A future SQL Server profile can reuse the metadata contract without granting
  it PostgreSQL runner permissions.
