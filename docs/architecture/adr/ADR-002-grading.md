# ADR-002 — Result equivalence with immutable fixtures
**Status:** Accepted as design baseline.

**Decision:** Execute reference SQL and user SQL against equivalent clean test fixtures, compare typed column schema + ordered rows or unordered multiset, and aggregate typed verdicts. Store pack hash + engine + comparator version with submissions.

**Rationale:** Multiple SQL syntax variants can be semantically equivalent; raw string and AST comparison are incorrect judge oracles.

**Risks:** Finite tests cannot prove equivalence; authoring must include adversarial fixtures; floating point/date/type normalization needs explicit policy and property tests. No accidental unordered dedup.
