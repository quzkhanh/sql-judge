# TDD Rules — binding for all implementation agents

## Non-negotiable cycle
**RED → GREEN → REFACTOR** per observable behavior. Write test first, run it and confirm meaningful expected failure, implement minimum production change, rerun until passing, refactor only after green and rerun. For bugfix, reproduce symptom before patch. Record actual commands + outputs in task handoff.

A failing compile/missing dependency is *not* the desired RED for behavior tests; adjust harness until test fails for the expected assertion. Avoid changing tests merely to match implementation.

## Test types and ownership
| Level | Tools (choose after inspecting scaffold) | Required behavior |
|---|---|---|
| Rust unit | built-in `#[test]`, `cargo test` | comparator, normalization, policy, verdict, manifest semantics |
| Rust property | `proptest` (approval before adding) | unordered bag invariance, multiplicity, normalization roundtrips |
| Rust integration | actual disposable PostgreSQL / MSSQL instance | sql dialect, permission denial, fixtures, cancel, cleanup |
| TypeScript unit | generated repo test runner or introduce Vitest after approval | typed invoke adapters, UI state, validation |
| UI component | Testing Library if configured | keyboard navigation, validation/errors, loading/verdict |
| E2E desktop | Tauri WebDriver/platform-specific testing after framework setup | open/run/submit/import/reset flows |
| Release | existing matrix workflow | six selected artifacts with explicit integration caveats |

## Critical RED-first examples
1. `unordered_comparator_preserves_duplicates`: same sets/different counts must reject.
2. `unordered_comparator_ignores_row_order`: permuted rows must pass.
3. `null_is_not_text_null`: typed NULL distinct from string.
4. `user_sql_cannot_write_even_if_parser_misses`: guard plus SQL runner read-only restrictions.
5. `reference_query_failure_is_judge_error`: no WrongAnswer on broken pack.
6. `reset_rejects_unmanaged_connection_even_with_forged_id`: Rust guard not UI-only.
7. `import_rejects_symlink_and_zip_traversal`: staging directory remains unchanged outside target.
8. `rebuild_failure_preserves_active_environment`: old env stays ready.
9. `submission_persists_after_restart`: SQLite migration correct.
10. `timeout_cancels_and_cleans`: sandbox unaffected for following judge.

## Dependency isolation
Pure comparison unit tests must NOT connect to PostgreSQL. SQL semantics, permission and cancellation tests MUST use real disposable engines; mocks alone cannot prove correctness. Integration DB instance belongs to test harness; never rely on user's real DB. Keep fixtures checked in and reproducible.

## Commands — discover before running
Inspect current `package.json` scripts before choosing Bun commands. Typical examples (not promises):
```bash
bun install --frozen-lockfile
bun run build
bun run tauri dev
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
python3 scripts/validate_pack.py examples/packs/hr-analytics
```
If no Cargo workspace exists yet, use `cargo test --manifest-path src-tauri/Cargo.toml`; if no JS test script exists, add one through an approved task, don't claim a nonexistent `bun test` suite.

## Coverage and quality bars
Minimum targets *after testing infrastructure exists*: >=90% branch coverage for comparator/ownership-policy/package-path logic, >=80% line coverage for new Rust domain/service code, 100% critical security scenarios enumerated in `docs/security/THREAT-MODEL.md`. Coverage is a signal, not proof; never weaken limits to green. Capture baseline before enforcement and track named exclusions (platform adapters/entrypoints) in ADR.

## No false green
- No `test.skip`, `#[ignore]`, `#[allow(unused)]` masking failures, blanket `eslint-disable`, `@ts-ignore`, snapshot updates without review.
- No mocks where the actual SQL engine's behavior is the subject.
- No running SQL against external production DB in tests.
- No deleting/weakening assertions to fix failing CI.
- No claims of passing platforms, runtimes, security tests without fresh output.

## Exit quality gate per task
1. Show observed RED with expected assertion failure.
2. Show GREEN test; focused regression and affected suites green.
3. Show `cargo fmt` and `clippy` for Rust, typecheck/lint/build for frontend if present.
4. Explain negative cases, security implications, unsupported platforms and manual smoke needs.
5. Update requirement->acceptance traceability and `tasks/todo.md` status only after proof.
