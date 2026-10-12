# Start-here instruction for coding agent

Use the following text as the first instruction to any coding agent already inside the **existing** `sql-judge` repository. These docs do not authorize arbitrary destructive actions.

---

You are implementing SQL Judge in an already-generated agmmnn/tauri-ui Vite + shadcn preset b0 project. **Do not regenerate the project or replace the generated dashboard, Bun tooling, Tauri shell, size optimization or GitHub Releases workflow.**

First inspect: `AGENTS.md`, `docs/requirements/PRD.md`, `docs/requirements/ACCEPTANCE.md`, `docs/architecture/ARCHITECTURE.md`, `docs/security/THREAT-MODEL.md`, `docs/testing/TDD-RULES.md`, and `tasks/todo.md`. Inspect actual `package.json`, Bun lockfile, `src/`, `src-tauri/` config, workflow and Git state. If generated code differs from docs illustrative paths, use real code layout and note any contract changes.

Work on **the earliest dependency-unblocked task** from `tasks/todo.md` only. State its requirement and AC IDs, impacted files, a negative/edge-case test, and runnable commands before modifying code. Apply TDD: RED with expected assertion failure -> minimal GREEN -> REFACTOR -> fresh test/lint/typecheck/build output. Protect user external DBs: no admin SQL against user connections, no generic drop/truncate, no plaintext credentials. Use a disposable managed local PG instance for tests. Never mark work as done without evidence; if blocked, report limitation and the smallest unblock action.

Only after the current task meets DoD, update `tasks/todo.md` and output a structured handoff (task, files, RED/GREEN commands/results, security notes, next task). Do not attempt to implement every module in one enormous change. For framework APIs check official docs matching actual installed versions.
