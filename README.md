# SQL Judge — Agent Implementation Kit (v1.0)

**Purpose:** repository-ready requirements, architecture, executable package contracts, TDD gates, task plan, and operating rules for building a real offline-first SQL practice desktop application.

**Target:** existing `sql-judge` scaffold created via [`agmmnn/tauri-ui`](https://github.com/agmmnn/tauri-ui) (`Vite`, `shadcn/ui` preset `b0`, starter dashboard, size optimization, invoke example, release workflow; GitHub Release artifacts: NSIS, Portable, Apple Silicon DMG, Intel DMG, AppImage, DEB). **Preserve that scaffold**. Do not re-scaffold, change UI library, replace Bun with pnpm, or rewrite the generated release workflow without inspecting actual repository files. Tauri 2 + Rust + React/TypeScript + Bun.

## Files and reading order

1. `AGENTS.md` — primary mandatory agent instructions (also `CLAUDE.md`).
2. `docs/requirements/PRD.md` — scope, user journeys, requirements, acceptance IDs.
3. `docs/requirements/ACCEPTANCE.md` — testable acceptance matrix and Definition of Done.
4. `docs/architecture/ARCHITECTURE.md` — dependency diagram, crate boundaries, domain model, data flow.
5. `docs/contracts/JUDGE-CONTRACT.md` — judging semantics, comparison, timeouts, errors.
6. `docs/contracts/PROBLEM-PACK.md` — package layout and machine-readable manifest.
7. `docs/contracts/CONNECTIONS-ENVIRONMENTS.md` — user database connections; safe provisioning/reset.
8. `docs/security/THREAT-MODEL.md` — trust boundaries and forbidden actions.
9. `docs/testing/TDD-RULES.md` — Red/Green/Refactor and verification requirements.
10. `docs/ui/UI-SPEC.md` — use the actual tauri-ui starter, UX and accessibility.
11. `docs/design/` — approved application design exports. `docs/design/SQL Judge — UI redesign.html` is the application-wide reference; per-screen artboards such as `docs/design/Home/Main.dc.html` provide exact layout and visual values.
12. `docs/agents/AI-PACK-GENERATION.md` — RULE for agents that author SQL packs.
13. `docs/agents/WORKFLOW.md` — task assignment, commits, evidence and handoffs.
14. `tasks/plan.md` + `tasks/todo.md` — dependency-ordered implementation work.
15. `docs/operations/BUILD-RELEASE.md` + `docs/operations/RUNBOOK.md` — build and runtime.
16. `schemas/` and `examples/packs/` — machine-readable v1 contract and reference fixture.

## UI design fidelity

When changing frontend UI, treat the matching export under `docs/design/` as a binding visual reference, not inspiration. Recreate its documented dimensions, spacing, colors, typography, radii, borders, shadows, states, and responsive behavior through the existing component system. Do not copy exported runtime markup or vendor assets into production.

If the reference and existing component defaults differ, use the reference values in a scoped component or deliberately update the shared primitive after checking its callers. Do not substitute arbitrary values or claim pixel fidelity without inspecting the relevant reference and validating the rendered screen.

## How to merge into an already generated scaffold

From the **parent folder** of the existing `sql-judge` directory, unzip `sql-judge-agent-kit.zip` into a temporary directory; copy the kit's top-level `AGENTS.md`, `CLAUDE.md`, `docs/`, `tasks/`, `schemas/`, `examples/`, and `scripts/` into the project. Review conflicts **before** copying; do not overwrite an existing agent instruction or unfinished plan. The ZIP contains documentation/validators and examples, **not a Tauri application**.

```bash
# Run inside existing sql-judge project after copying the kit
python3 -m pip install -r scripts/requirements.txt  # once, for validation only
python3 -m unittest discover -s scripts -p "test_*.py" -v
python3 scripts/validate_pack.py examples/packs/hr-sql-101
bun install --frozen-lockfile  # only after reviewing actual Bun lockfile and dependencies
bun run tauri dev
```

The validation script checks structural contracts without executing SQL or launching a database. Integration testing of SQL correctness is a later gate and cannot be claimed from a successful structural validation.

## Default delivery phases

- **P0:** inspect scaffold; freeze contracts; package validator + example pack.
- **P1:** managed PostgreSQL local, SQL Judge core and minimal editor-to-verdict vertical slice.
- **P2:** environment lifecycle + import UX + submissions + SQLite history.
- **P3:** external PostgreSQL explorer and Microsoft SQL Server read-only connector.
- **P4:** optional scoped external practice / SQL Server judge, cross-platform bundled runtime packaging, polish.

Avoid promising offline external server connectivity: external connections depend on the server/network being reachable. The desktop app itself has no cloud service requirement.

## Contract and source provenance

This is a **design/specification deliverable**, not an already built or verified application. Baseline design cites [Tauri 2 commands](https://v2.tauri.app/develop/calling-rust/), [tauri-ui scaffold CLI](https://github.com/agmmnn/tauri-ui/blob/master/packages/create-tauri-ui/README.md), [PostgreSQL documentation](https://www.postgresql.org/docs/current/), and [SQL Server permissions](https://learn.microsoft.com/en-us/sql/relational-databases/security/authentication-access/getting-started-with-database-engine-permissions). Agents must check actual installed dependency versions and official docs before implementation.
