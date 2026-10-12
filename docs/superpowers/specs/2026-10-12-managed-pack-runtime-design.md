# Managed Pack Runtime Boundary

## Goal

Prevent problem-pack runtime validation and publication from using an arbitrary
local PostgreSQL database. Loopback connectivity is necessary but is not proof
that the database belongs to the SQL Judge managed runtime.

## Design

`judge-postgres` exposes a `ManagedPostgresSession` capability. Its constructor
accepts a connection string and expected managed `instance_id`, then requires
both of these checks before returning the capability:

1. The connection target is a loopback address or Unix socket.
2. An admin-owned runtime marker in the connected database contains the
   expected instance ID.

The marker is created by trusted managed-runtime setup and is not writable by
the fixture runner or `PUBLIC`.

`PackImporter::import` accepts only `ManagedPostgresSession`. Runtime fixture
validation uses the capability's managed-fixture operation, while ordinary
`PostgresSession` remains available for query and judge tests that do not
publish packs.

## Failure Behavior

- Missing marker, mismatched instance ID, or non-loopback target fails closed.
- The error does not include a connection string, password, or raw SQL.
- No pack is published when runtime validation fails.

## Tests

- A disposable local PostgreSQL database without the marker is rejected.
- A disposable database with an admin-owned matching marker is accepted.
- Existing pack runtime validation, idempotent publication, and cleanup tests
  remain green.

## Scope

This establishes the capability boundary for pack import. It does not yet
implement the full PostgreSQL process manager or Tauri lifecycle commands; that
remains part of T10/T13/T14.
