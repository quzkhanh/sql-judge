# ADR-020: Pin Managed Environments to Pack Identity

Status: Accepted

## Context

Managed environment records previously stored only a pack ID. Registry lookup
could therefore select a newer version when an environment was reprovisioned or
restored. A digest-verified pack registry is immutable by content, so the
environment can safely retain the exact version and digest selected at create
time.

## Decision

Managed records persist `pack_id`, `pack_version`, and the 64-character content
digest. Create requests must provide an installed exact identity; provisioning,
reset, recovery, and backup restore resolve that identity with an exact digest
lookup and fail closed when it is unavailable. Existing SQLite ledgers migrate
with empty identity fields; managed operations reject those legacy records until
they are recreated or otherwise explicitly repaired. External explorer records
may retain empty pack identity fields.

Managed pack publication occurs before schema provisioning so an immutable
publication conflict cannot create a database schema. If provisioning fails
after a new publication, only that publication and empty staging directories
are removed.

## Consequences

- Backups include the exact managed pack version and digest and use backup
  version 2.
- The UI displays the selected pack version and digest prefix and sends both
  values in create requests.
- Pack registry lookup still supports latest-version discovery for explicit pack
  inspection, but lifecycle operations never use it.
