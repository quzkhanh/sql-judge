# ADR-004 — Machine-validated immutable problem packs
**Status:** Accepted as content baseline.

**Decision:** JSON manifest with strict JSON Schema 2020-12, POSIX relative path conventions, reference SQL plus independent full fixture files, semantic version + content hash, two-stage structural and runtime validation, atomic publish.

**Rationale:** AI agents need a deterministic contract, not prose-only RULE. JSON avoids YAML loader ambiguity and enables straightforward validation.

**Trade-off:** More files than storing challenges in a single SQLite row; benefits include reviewable diffs, reproducibility and easy ZIP import/export.
