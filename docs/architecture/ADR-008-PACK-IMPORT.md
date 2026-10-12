# ADR-008: Atomic Content-Addressed Pack Import

## Status
Accepted for T15.

## Decision
Use a focused Rust `problem-pack` crate. It receives already-loaded, structurally validated pack bytes and a pre-created `judge_postgres::PostgresSession`; it never accepts a connection string or creates an external target. Trusted pack schema and fixtures execute only through `PostgresSession::with_managed_fixture`, and reference SQL runs after the managed runner role is set.

Validated files are hashed deterministically with SHA-256 and published as `sha256/<digest>` by same-filesystem directory rename. The digest tree is the immutable registry; an existing `(id, version)` with another digest is rejected. Temporary staging is cleaned on every publication failure.

## Consequences

- `sha2` is a small, audited dependency used only for content identity.
- Runtime validation requires the caller to provide the app-managed local PostgreSQL session and low-privilege runner role; process provisioning remains T13/T24 scope.
- The importer does not execute candidate/user SQL and does not make offline packs tamper-proof.
