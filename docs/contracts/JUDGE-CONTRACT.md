# Judge Contract (JUDGE-v1)

## Scope
MVP: deterministic result-set `SELECT` problems for PostgreSQL, eventually SQL Server. References + fixtures specify expected output; the engine **does not** check text similarity or AST equality. Verdict correctness is conditional on fixture quality, query rules, engine version and comparator policy; it cannot prove SQL equivalence on all possible databases.

## Input
`SubmissionRequest { problem_key, problem_version?, sql: UTF-8 <= 65536 bytes, requested_mode: RUN|SUBMIT }`. `RUN` uses publicly visible sample fixture; `SUBMIT` uses configured full suite. User SQL must be exactly one read-only permitted statement under dialect-specific policy, executed with non-privileged DB identity inside controlled test target. Never concatenate editor SQL into a privileged setup query.

## Oracle and fixture semantics
- Each test fixture is **complete dataset seed** applied after `schema/*.sql` to a clean environment. A fixture may reference only declared schema and pack files.
- Reference SQL result is expected value only if reference successful and validated at package import (not a guarantee of correctness).
- Reference and user SQL must see logically identical snapshots, role/session settings, collation, timezone and search path. Use fresh sessions/rollback snapshots or cloned ephemeral targets; no shared mutable session.
- No hidden dependence on auto-increment sequence state, row physical order or wall-clock unless frozen as an explicit exercise constraint.
- `LIMIT`, `TOP`, window tie ordering, aggregate numeric behavior and `NULL` semantics must be specified by question.

## Result policy
- `row_order: ordered|unordered`: ordered compares row positions; unordered compares a **multiset/bag** and preserves duplicate multiplicity.
- `columns`: always compare count and position; by default compare names and logical types; options must be explicit in manifest and never auto-relax based on user output.
- SQL NULL is its own tagged value, not string `"NULL"`, false or 0.
- `NUMERIC/DECIMAL`: canonical exact decimal preserving numeric equality (e.g., 1.0 == 1.00 when allowed); `FLOAT`: strict canonical bit/NaN policy by default, configured finite epsilon only for explicitly approximate exercises (not transitive; avoid hash-based float bag matching in epsilon mode).
- Timestamps are normalized by type and timezone policy; text collations/case not silently normalized. JSON/arrays are either typed canonicalized by explicit contract or unsupported with a clear verdict; do not JSON stringify everything.
- Output schema mismatch -> WrongAnswer with safe high-level diff. For hidden fixtures, show failed test identifier/category but not seed or reference rows.
- Comparison code must be pure and testable with property-based tests; multiset count check: `count(actual,row) == count(expected,row)` for all normalized rows.

## Verdict priority
1. Package/schema/fixture/reference fails -> `JudgeError` (do not blame user).
2. Policy violation -> `PolicyViolation`; user parse failure -> `SyntaxError`.
3. Runtime DB error -> `RuntimeError`; elapsed wall/statement deadline -> `TimeLimitExceeded`; resource cap -> `ResultLimitExceeded`; user cancel -> `Cancelled`.
4. Successful SQL but schema/rows differ -> `WrongAnswer`.
5. All tests match -> `Accepted`.

For multiple tests: run all permitted tests for diagnostic value or stop on hard resource/security failure. Aggregate verdict deterministic by documented severity policy; do not report Accepted unless all relevant tests passed; infrastructure failure must fail closed.

## Bounds (initial configuration, verify by benchmark)
- One submission at a time per managed runner; 2s statement timeout/test, 5s wall watchdog/test.
- Max 10,000 result rows; <=16 MiB total decoded results; <=64 KiB SQL.
- Do not silently rewrite user SQL with `LIMIT`/`TOP`, since that changes semantics. Stop fetching, mark ResultLimitExceeded and cancel the backend execution if exceeded.
- Query plan introspection is an advanced opt-in, never execute `EXPLAIN ANALYZE` on arbitrary external SQL by default.

## Required comparator test matrix
| Case | Expected |
|---|---|
| unordered `[1,1,2]` vs `[2,1,1]` | accept |
| unordered `[1,1,2]` vs `[1,2,2]` | reject |
| ordered `[1,2]` vs `[2,1]` | reject |
| `NULL` vs text `'NULL'` | reject |
| integer 1 vs decimal 1.0 | explicit policy, default reject mismatched column types |
| empty result with correct columns vs zero columns | reject |
| same values but wrong column labels (strict mode) | reject |
| float NaN/Infinity | explicit normalization & tests, no panics |
| truncated results | ResultLimitExceeded, never Accepted |
| fixture/reference execution failure | JudgeError |
| hard-coded sample row IDs | rejects on targeted hidden fixtures |
| user query DML/DDL/network-capable function | PolicyViolation or server-denied runtime; no side effects |

## Error DTO
`{ code, category, safe_message, sql_state?, test_case_id?, elapsed_ms, trace_id }` with optional developer diagnostics behind a local opt-in; no credentials or hidden fixture text. User-visible query error messages should not leak confidential metadata from external connections.
