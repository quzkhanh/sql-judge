# ADR-001 — Preserve generated tauri-ui Vite scaffold
**Status:** Accepted as design baseline, pending inspection of actual checkout.

**Context:** User completed agmmnn/tauri-ui CLI with Vite + b0 starter dashboard + size optimization + invoke example + workflow and six release artifacts.

**Decision:** Extend existing `src/` and `src-tauri/`, use generated shadcn/ui patterns, Bun dependency tooling and Tauri native shell. Add Rust Cargo crates incrementally without re-scaffolding.

**Trade-off:** Must adapt path conventions discovered in actual repo; cannot assume older illustrative monorepo tree.

**Alternatives rejected:** Fresh UI scaffold (loss of selected features/workflow), swapping to pnpm and entirely different UI library (unnecessary migration risk).
