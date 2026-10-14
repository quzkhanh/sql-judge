# ADR-014: Monaco SQL Editor

- Status: Accepted
- Date: 2026-10-12
- Requirements: REQ-003, REQ-004, REQ-019
- Acceptance: AC-002, AC-003
- Security: SEC-007

## Decision

Use `@monaco-editor/react` for the desktop and browser-preview SQL editor. Monaco provides SQL syntax highlighting, keyboard editing behavior, accessible labels, and automatic layout without changing the judge IPC contract.

The editor remains controlled by the per-problem draft state, disables features unrelated to SQL practice such as the minimap, and enforces the existing 64 KiB SQL input limit before invoking native commands. Result rendering remains separately bounded by the UI and by native query limits.

## Consequences

- The frontend bundle grows, but the editor behavior is consistent across supported desktop platforms and browser preview.
- Monaco workers are bundled by Vite through the package's standard loader; no remote assets or network access are required.
- The existing textarea fallback is not retained because it would create two divergent editor behaviors.
