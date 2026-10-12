# Agent execution protocol — build the app one verifiable slice at a time

## Agent roles
- **Coordinator (sole shared-state owner):** reads contracts, assigns Task IDs, resolves spec conflicts, merges PRs, owns `tasks/` and ADR updates. Does not write concurrently to other agents' files.
- **Rust domain agent:** `judge-core` types/comparator/verdict and tests.
- **DB integration agent:** managed PG harness, adapter, fixture/test runner; no external user DB.
- **Safety agent:** threat reviews, negative tests, scoped ownership/reset and secret/TLS controls.
- **Frontend agent:** preserve tauri-ui starter; feature view + typed IPC + UI tests.
- **Content agent:** generates isolated problem folders from pack schema, never modifies shared pack/schema without coordinator review.
- **QA/release agent:** verifies install, smoke flow and artifact matrix; never claims untested platform success.

## Work allocation
Small tasks should touch <=5 files. No two agents concurrently edit the same file or an in-progress SQLite migration, `Cargo.toml` workspace manifest, `package.json`, lockfile, Tauri capability policy, `.github/workflows`, `pack.json`, or central design docs. Use branches/worktrees for independent agents; serialize shared contract changes first.

## Ticket protocol
Each ticket must contain:
```md
Task: TNN — short verb phrase
Context: requirement/AC IDs and exact docs paths
Scope: exact files / crate / feature
Acceptance: 1–3 observable assertions incl. one negative case
RED: command to reproduce failing test
GREEN: command to verify behavior
Security: blast radius and credentials/runtime assumptions
Dependencies: task IDs
Done evidence: stdout/CI artifact, links/commit
```
Coordinator selects tasks from `tasks/todo.md` in dependency order; never blanket instruct "build whole app" without bounded stage gates.

## Verification handoff format
```md
Task ID:
Requirements/acceptance:
Changed files:
RED evidence (command + expected assertion failure):
GREEN evidence (command + test counts):
Lint/typecheck/build evidence:
Security/regression notes:
Risks/deferred checks:
Next unblocked task:
```
If a command cannot run, state `NOT RUN` and why; do not turn unavailable DB/GitHub CI into pretend success.

## Source-driven API workflow
Inspect pinned dependency manifests/lockfiles and original scaffold. For Tauri command signature/capabilities, use official `https://v2.tauri.app/develop/calling-rust/` and `https://v2.tauri.app/security/capabilities/`; for tauri-ui details, `https://github.com/agmmnn/tauri-ui`. For PostgreSQL and SQL Server semantics/security, consult version-matched first-party docs. Record relevant URL in design PR notes for nontrivial API-dependent decisions.

## Escalation and stop rules
- Stop on unclear ownership/permission for destructive DB action; request explicit human authorization and document exact target.
- Stop before adopting new package manager, replacing UI system, changing release platforms or silently adding a runtime distribution mode.
- Stop on tests that can only run against a real, non-test external DB; create safe disposable fixture instead.
- Stop when current task requires broad architecture change; update contract + ADR and split work.
- Do not block independent read-only UI work while DB harness is unavailable, but never claim end-to-end Judge finished.

## Review checklist
One reviewer verifies AC trace, code boundaries, at least one negative test, real command results, no unsafe DB write paths, log secrecy, coverage/risk tradeoffs and preservation of generated scaffold. Coordinator accepts only verified task then updates task checklist.
