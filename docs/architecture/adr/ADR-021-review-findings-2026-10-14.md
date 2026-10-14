# ADR-021: Fail-Closed Dependencies and Native Pack Catalogs

Status: Accepted

## Decision

SQL Server explorer dependency inspection uses browse-mode result metadata and
rejects non-zero metadata errors, unresolved schema/object pairs, cross-database
sources, synonyms, views, table-valued functions, and unknown user-defined
function calls. This is conservative because the offline harness does not
provide a reliable indirect view/synonym fixture; no hidden-dependency coverage
is claimed.

The native pack catalog is the only source for learning pack/problem selection.
It returns bounded, validated manifest metadata and exact digest identity only.
Schema SQL, reference SQL, fixture SQL, and arbitrary pack files never cross the
native/UI boundary. Environment creation sends the selected exact pack identity
back to Rust for registry verification.

Managed judging also loads the exact digest-verified pack and problem manifest
in Rust. Run evaluates declared visible tests; Submit evaluates all declared
tests. The UI never supplies schema, reference, fixture, output-policy, or
limit SQL to the judge, and submission history records the validated pack
identity.

External SELECT IPC validates SQL size and query limits before profile lookup,
secret-vault access, or driver connection. Window error messages use the same
redaction serializer as other debug events before entering the debug panel.

Catalog problem keys include pack ID, semantic version, digest, and problem ID.
Validated catalog metadata may include bounded table/column descriptions,
public sample rows, and starter SQL; reference SQL and hidden fixtures remain
native-only.

## Consequences

- Invalid or tampered installed registry entries make catalog loading fail closed.
- SQL Server queries using unsupported indirect dependency shapes fail as
  `QueryRejected` instead of being treated as safe.
- The UI no longer assumes the HR sample pack exists.
- Imported packs can be judged without embedded problem IDs or fixture SQL.
- Invalid external limits cannot trigger profile or credential access.
- Runtime error messages do not bypass debug redaction.
- Immutable catalog versions cannot overwrite each other's selection or drafts.
