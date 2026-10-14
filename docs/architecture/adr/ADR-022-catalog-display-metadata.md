# ADR-022: Catalog Display Metadata for the Solve Screen

Status: Accepted

## Context

The redesigned Problems and Solve screens (`docs/design/Problems-html`,
`docs/design/Solve — submit verdict-html`) show, per problem, the expected
return columns with types, an example with input tables and output, the row
order rule, the number of fixtures a submission is judged on, and the time
limit. ADR-021 limited the native catalog to identity, statement, catalog
schema, public sample output rows and starter SQL, so the UI had no validated
source for the example input tables or the judging summary.

## Decision

`problem.json` gains an optional, additive `catalog.sample_tables` field inside
the existing optional `catalog` block of `sqljudge.pack/v1`. This follows the
same precedent as `catalog` itself: an optional display-only block whose absence
leaves every existing v1 pack valid, so no spec version bump is required. Packs
that omit it expose an empty list.

`catalog.sample_tables` is a list of `{ name, rows }`:

- at most 8 tables; names match the catalog identifier pattern
  `^[A-Za-z_][A-Za-z0-9_]*$` (max 128 bytes), are unique, and must name a table
  declared in `catalog.schema` when that list is non-empty;
- at most 20 rows per table; each row is a flat object with at most 128
  identifier keys whose values are `null`, boolean, number, or a string of at
  most 4 KiB without control characters (the same bounds as `sample_rows`);
- unknown properties are rejected by both the JSON Schema and the Rust
  `deny_unknown_fields` manifest.

The Rust importer enforces these bounds during structural validation, before
publication, and again when building catalog entries. `scripts/validate_pack.py`
enforces them through `schemas/problem.schema.json` plus explicit uniqueness and
table-membership checks.

`ProblemCatalogEntry` additionally exposes, derived from the already-validated
manifest: `outputColumns` (`output.columns` as `{ name, type }`), `rowOrder`
(`ordered` or `unordered`), `timeLimitMs` (`limits.statement_timeout_ms`),
`fixtureCount` (number of declared tests) and `sampleTables`.

## Security and grading boundaries

- `sample_tables` is display-only. It is never loaded into a database, never
  compared against submissions and never used to derive a verdict. Fixture SQL
  remains the only judged input data, so a pack author is responsible for
  keeping the example consistent with the public fixture; the bundled sample
  pack does so and is checked by tests.
- The output contract, row order and time limit are already part of the public
  problem contract shown to users. The fixture count reveals only how many tests
  exist; hidden fixture IDs, contents and expected results remain native-only.
  Hidden tests on an offline client are still not secret (see `AGENTS.md`).
- All values are bounded, flat JSON scalars, so the IPC payload stays small and
  cannot carry nested or executable content.

## Consequences

- The Solve screen can render "Return columns", the example input/output and
  the "Judged on N fixtures · limit" footer from validated native data only.
- Row objects are serialized with sorted keys, so key order is not meaningful;
  the UI should order example columns by the matching `catalog.schema` table and
  show only the keys present in the rows.
- Starter SQL may span lines (amended in phase 2): `\n`, `\r\n` and `\t` are
  the only control characters accepted in `catalog.starter_sql`; a lone `\r`,
  NUL, escape sequences and every other C0/C1 control character are rejected.
  All other catalog strings (sample cells, schema types) still reject every
  control character. `scripts/validate_pack.py` mirrors the rule.
- Starter SQL is a starting point, not an answer key. The bundled
  `second-highest-salary` starter is the 14-line `DENSE_RANK` query from the
  Solve artboard; a native test proves it is judged Wrong Answer (basic,
  salary-ties and null-salaries fail; empty-department passes).
- The two earlier sample packs were replaced by `examples/packs/hr-analytics`
  (`examples/hr-analytics.sqljudge.zip`), which uses every catalog field.
