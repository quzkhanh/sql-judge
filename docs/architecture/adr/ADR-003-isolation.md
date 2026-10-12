# ADR-003 — Destructive lifecycle only on application-managed local environments
**Status:** Accepted as safety baseline.

**Decision:** Managed PG process with own identities and namespaces. Reset/rebuild only with durable ownership ledger + database marker verified, lock/confirmation, create replacement then pointer swap. External user profiles always read-only in MVP.

**Rationale:** Arbitrary reset or DROP on user-provided SQL server is unacceptable. Schema names alone are not ownership proof.

**Trade-off:** Higher operational effort (PG runtime, crash reconciliation), less flexibility than allowing all-powerful user SQL. Future advanced writable external mode requires separate ADR and acceptance tests.
