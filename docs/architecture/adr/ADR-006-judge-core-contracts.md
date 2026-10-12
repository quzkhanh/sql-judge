# ADR-006 — Add a pure `judge-core` contract crate

**Status:** Accepted for P1 foundation work.

## Context

Problem and result DTOs must be shared by future runners, comparator logic and
Tauri orchestration without coupling the domain to a database driver, Tauri,
SQLite or operating-system APIs.

## Decision

Create a root Cargo workspace containing `crates/judge-core`. The crate owns
strictly tagged `SqlValue` values, typed result columns/rows, problem output
policy and query limits. Validation is explicit and deterministic; unknown
serde fields and value tags are rejected. The Tauri crate remains a separate
entrypoint until it needs a deliberate dependency on the core contract.

## Consequences

- Pure contract tests run offline with no database.
- Future PostgreSQL/SQL Server adapters depend on domain types rather than the
  reverse.
- The workspace currently excludes `src-tauri` to preserve its generated
  standalone lockfile and release behavior; membership can be expanded in a
  later integration task with a separate review.

## Traceability

- Requirement: `REQ-003`, `REQ-004`, `REQ-006`
- Acceptance: `AC-003`, `AC-005`, `AC-006`
