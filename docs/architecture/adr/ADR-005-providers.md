# ADR-005 — Capability-based PostgreSQL and SQL Server providers
**Status:** Accepted as modular design baseline.

**Decision:** Separate query metadata/browser interface from privileged managed environment provisioner. PostgreSQL local judge ships first; external PostgreSQL and SQL Server explorer supports read-only queries. T-SQL judge deferred pending disposable test sandbox and per-dialect error/type/permission tests.

**Rationale:** SQL dialects have incompatible schemas, functions, data types, permissions and administrative workflows. Generic `drop_all_tables()` provider abstraction is dangerous.

**Trade-off:** Some code duplication in adapters in exchange for explicit correct semantics and least privilege.
