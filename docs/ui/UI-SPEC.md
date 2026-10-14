# UI Specification — SQL Judge design contract

## Source of truth

`docs/design/SQL Judge — UI redesign.html` is the application-wide visual contract. Matching exports under `docs/design/` are screen-level contracts. Their declared values define layout, tokens, spacing, typography, radii, borders, shadows and states. Reimplement them with the existing Vite, React, Tailwind and shadcn/ui system. Do not ship copied export markup, vendor React bundles or export runtime scripts.

## Required references

| Surface | Reference |
| --- | --- |
| Shell and shared tokens | `SQL Judge — UI redesign.html` |
| Home | `Home/Main.dc.html` |
| Problems | `Problems-html/Problems.dc.html`, `Problems (dark)-html/Problems.dc.html` |
| Solve | `Solve — sample run (dark)-html/Solve.dc.html`, `Solve — submit verdict-html/Solve.dc.html` |
| Packs/import | `Packs-html/Packs.dc.html`, `Import review-html/ImportPack.dc.html` |
| Environments/reset | `Environments-html/Environments.dc.html`, `Reset confirmation-html/ResetDialog.dc.html` |
| History/Progress | `History-html/History.dc.html`, `Progress-html/Progress.dc.html` |
| Connections/Explorer/Settings | `Connections-html/Connections.dc.html`, `Explorer-html/Explorer.dc.html`, `Settings-html/Settings.dc.html` |

## Shell requirements

- Home is the default route.
- Desktop sidebar is 248px; header is 56px. Navigation groups are Learn, Library and Data.
- Primary controls are 36px tall with 14px horizontal padding, 8px gap and 8px radius. Small controls are 32px tall with 10px horizontal padding.
- Use exact light/dark tokens from the export. Prefer Geist and Geist Mono when locally available; retain installed offline font fallback otherwise.
- Change a shared primitive only after reviewing all callers. Otherwise use a scoped override.

## Behavior, accessibility and safety

Every screen must match its artboard while preserving typed IPC, keyboard navigation, visible focus, responsive behavior, loading/empty/error/cancelled states, no color-only verdicts and all database safety invariants. UI fidelity never permits secret display, external destructive action, frontend-only ownership checks or false success state.

## Visual verification

Before checking a frontend task: inspect the application and screen export; render at the artboard viewport and supported responsive widths; reconcile visible token, geometry, typography, spacing, state and focus differences; record comparison evidence. A passing build or source review alone does not establish design fidelity.
