# ADR-016: Explicit JSON Metadata Backup

- Status: Accepted
- Date: 2026-10-12
- Requirements: REQ-013, REQ-020
- Acceptance: AC-013, AC-020
- Security: SEC-004

## Context

SQL Judge needs a portable backup for local environment metadata and submission
history. SQLite currently stores no credentials, but a raw database copy would
make future secret-bearing columns unsafe to export accidentally.

## Decision

Use a versioned, strict JSON snapshot containing only environment metadata,
submission metadata, including each submission's pack ID, pack version, and
pack digest/hash, and per-test verdict/timing records. The snapshot is
validated and restored inside one SQLite transaction. Recovery journal rows,
runtime metadata, credentials, raw SQL, filesystem paths, and secret references
are excluded.

Pack contents are never embedded in a backup. Restoring a managed environment
requires the corresponding pack to be separately imported and structurally and
runtime validated through the pack registry; backup JSON cannot publish or
execute schema SQL and cannot carry arbitrary registry payloads.

The native command bounds the serialized payload before parsing and returns
generic errors without exposing database or filesystem details.

## Alternatives Rejected

- Raw SQLite/WAL copy: can capture future secret columns and is difficult to
  validate across SQLite connection state.
- History-only export: does not satisfy environment metadata roundtrip.
- Unversioned ad-hoc JSON: makes future migrations ambiguous and weakens
  unknown-field rejection.

## Consequences

- Backup files are portable and auditable.
- Restore replaces metadata/history but never changes PostgreSQL schemas.
- New backup fields require an explicit contract version update and tests.
