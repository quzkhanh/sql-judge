# ADR-011 — Stream PostgreSQL result rows before applying caps

**Status:** Accepted for the managed judge adapter.

## Decision

Use PostgreSQL's fallible row iterator for managed query results instead of
materializing the complete result set with `Client::query`. The adapter counts
rows and serialized result bytes while consuming the stream and stops with a
typed limit error when a cap is exceeded.

## Rationale

Fetching all rows before checking limits allows a large or untrusted query to
consume excessive memory even when the final result is rejected. Streaming
keeps memory proportional to the bounded result being retained.

## Consequences

- The adapter adds a direct `fallible-iterator` dependency matching the
  PostgreSQL client version.
- A single oversized row can still be decoded before its byte limit is
  reported; database statement and row limits remain defense in depth.
- Cancellation and process-level resource limits remain runtime responsibilities.

## Traceability

- Requirement: `REQ-004`
- Acceptance: `AC-004`, `AC-007`
- Security: `SEC-001`, `SEC-003`, `SEC-010`
