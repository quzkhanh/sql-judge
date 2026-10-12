# Metadata Backup and Restore Design

## Scope

Add a versioned JSON backup for local environment metadata and submission
history, with Progress and History UI actions. PostgreSQL schemas and secrets
are outside the backup boundary.

## Design

- `BackupSnapshot` version 1 contains environment metadata, submission metadata,
  and per-test verdict/timing records.
- `app-storage` exports and restores through one SQLite transaction.
- Restore validates version, strict fields, IDs, namespaces, states, duplicate
  keys, and bounded elapsed times before deleting existing metadata/history.
- Tauri bounds the encoded payload at 4 MiB, exposes typed export/restore
  commands, and refreshes the in-memory environment ledger after restore.
- The frontend downloads JSON through a browser Blob and restores only JSON
  selected by the user; native validation remains authoritative.

## Security

- No passwords, secret references, connection strings, raw SQL, runtime paths,
  or recovery journal rows are serialized.
- Restore never connects to PostgreSQL or performs schema lifecycle actions.
- Unknown JSON fields and unsupported versions fail closed.
- Errors shown in the UI are generic and do not expose filesystem or database
  details.

## Verification

- Storage roundtrip and secret-canary tests pass.
- Native Tauri export/restore test passes.
- Workspace tests, Tauri tests, clippy/fmt, frontend typecheck/build, and
  focused frontend lint pass.
- Manual History/Progress export and restore smoke remains required before
  marking T18 complete.
