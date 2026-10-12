# ADR-012: Managed Namespace Schema Provisioning

- Status: Accepted
- Date: 2026-10-12

## Decision

Managed PostgreSQL environments may provision trusted pack schema SQL only
after the namespace ownership marker has been created and verified. Schema
setup runs in a transaction with `search_path` limited to the quoted managed
namespace and `pg_catalog`.

The setup policy rejects role/database/schema administration, public-schema
references, privileged functions, and statements outside the supported table,
index, and insert forms. Runner credentials are not used for setup.

## Rationale

This keeps identical pack table names isolated by environment while preventing
pack setup text from writing to `public` or changing database-level security
state. A setup failure is returned without publishing or treating the schema
as ready.

## Consequences

- Managed setup is not a general PostgreSQL migration facility.
- External databases cannot use this provisioning path.
- Full process lifecycle and crash cleanup remain T13/T14 runtime work.
