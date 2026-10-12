# RULE — AI Agent generating SQL exercises

This file is the content authoring contract to hand to any AI coding agent. For repo implementation agent rules, see root `AGENTS.md`.

## Mission
Produce a **valid, importable, deterministic, challenging and verifiable** SQL Judge v1 pack, not just a collection of interesting prompts. Source of truth: `schemas/pack.schema.json`, `schemas/problem.schema.json`, `docs/contracts/PROBLEM-PACK.md` and the actual validator. If any conflict, machine schema + clarified spec wins; do not invent fields.

## Inputs to content generation
Topic, target dialect (`postgresql` or `sqlserver`), problem count, level distribution, scenario/domain and pack ID/version. If unspecified: PostgreSQL, 10 exercises, easy 30% / medium 50% / hard 20%, human-readable HR/E-Commerce dataset with synthetic non-PII content.

## Mandatory actions
1. Generate **one consistent relational schema** for the pack. Check PK/FK, NOT NULL, realistic relationships, no accidental ambiguous columns.
2. For each problem: define output grain (1 row per what?), required columns/types, projection/ordering, NULL interpretation, duplicate behavior and expected behavior on empty inputs.
3. Write unique `problem.json` and `reference.sql` for declared dialect. No network, IO, file loading, privileged extensions, server links, dynamic SQL or destructive/admin statements.
4. Write >=3 independent fixtures/problem: one public happy path; at least two hidden adversarial fixtures. Target duplicate multiplication, NULL, ties, boundary, empty/no rows, filtering, join fanout and correlated vs uncorrelated logic according to topic.
5. Reference SQL must be independent of hardcoded PKs and row physical order. Never use `SELECT *` unless task explicitly allows stable schema.
6. Run structural validator: `python3 scripts/validate_pack.py <pack-dir>`. **This does not prove query correctness.** Next run integration validation with a disposable actual engine + query comparison when implemented.
7. Peer-review statement and reference: generate at least one alternate valid solution and one plausible wrong solution. Expected accepted/rejected behavior must be validated on all fixtures.
8. Emit validation report with file list, ID uniqueness, fixture types, commands executed and real evidence. If actual DB is absent, mark runtime validation NOT RUN; never invent pass.

## Naming
- Pack ID: kebab-case globally unique within local catalog; problem ID: kebab-case within pack.
- Problems sorted into `problems/<id>/`; `reference.sql` and `tests/*.sql` are referenced from `problem.json` by relative paths.
- Files UTF-8, POSIX paths, LF recommended. Manifest JSON and schema strict; no comments in JSON.

## Content difficulty guide
- Easy: SELECT/WHERE/ORDER BY, single join, simple aggregates.
- Medium: group/aggregation/HAVING, multi-table joins, derived tables, correlated subquery, window basics.
- Hard: ranking ties, multi-stage analytics, window frames, set operations, recursive CTE when explicitly approved.
- A lesson tests **query semantics**, not incidental IDs or one convenient dataset.

## Anti-cheating / mutation checklist
- If a wrong query hardcodes observed output, at least one hidden fixture must change IDs/values and reject it.
- If expected `salary > average`, create a case with salary == average and a case with NULL salary.
- If `JOIN`, create one-to-many multiplicity and unmatched rows.
- If `GROUP BY`, include zero-match group, ties, NULL key and threshold boundaries as relevant.
- If `ORDER BY`, specify deterministic tie-breakers; otherwise mark unordered.
- If `NOT IN`/`NOT EXISTS`, include NULL in subquery side when semantic difference is relevant.

## Pack generation output
Return an importable pack folder/ZIP and short `validation-report.md`: JSON contract status, PostgreSQL/SQL Server runtime verification status, per-problem alt/wrong solution checks, known unsupported types. Do not ship incorrect or unvalidated packs as `ready`.

## Multi-agent coordination
One coordinator owns `pack.json` + schema. Workers write non-overlapping `problems/<id>/` paths; only coordinator updates manifest and runs final validation. Parallel workers must not generate incompatible table names or fixtures. Require review before merging a large pack.
