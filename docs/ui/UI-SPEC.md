# UI Specification — preserve agmmnn/tauri-ui Vite starter

## Fixed foundation
The user already selected `Vite`, `Starter dashboard`, `Size optimization`, `Invoke example`, `Release workflow`, preset `b0`; release artifacts: Windows NSIS+Portable, macOS arm64/x64 DMG, Linux AppImage+DEB. **Do not regenerate or replace UI scaffold**. Maintain its native desktop defaults, theme provider, external-link handling, debug panel, and upstream shadcn/ui components. Inspect repo `package.json`, `src/`, `src-tauri/`, existing icons and CI before applying any code changes.

## Navigation IA
Left sidebar: Dashboard, Problems, Workspace, Environments, Connections, Packs, Submissions, Settings. Existing dashboard stays accessible and can be adapted to progress. Avoid replacing entire dashboard with new bespoke component library.

## Core views
- **Dashboard**: solved count, daily study progress, recent submissions, recommended topics, DB runtime status.
- **Problems**: filters (dialect/difficulty/tags/solved), list, empty state, search and pagination.
- **Problem Detail/Workspace**: resizable panes: left statement + schema + sample; right Monaco query editor; bottom bounded results/diffs/status; actions Run and Submit clearly distinct; persisted draft per problem. Keyboard shortcuts must not interfere with browser/WebView/accessibility.
- **Connections**: saved profiles without secrets, engine toggle PG/MSSQL, host/port/db/user, TLS mode, credentials through native secure path, Test Connection (read-only), technical errors scrubbed.
- **Environments**: managed/external badges, owned env states, Create/Reset/Rebuild/Clone/Delete; dangerous actions only on managed with typed confirmation and impact summary. Disabled buttons explain WHY.
- **Packs**: drop `.sqljudge.zip`, import progress/stages, parser errors with filenames/JSON paths, detected dialect/version/hash, review before install; reference/sample tests run in disposable DB.
- **Submissions**: status, elapsed, pass count, per-test category, stable link, hidden fixture content never disclosed; trace ID for error reports.
- **Settings**: local PG runtime health, safe limits within caps, appearance, backups excluding credentials, accessibility, diagnostics.

## Interaction states
For EVERY asynchronous page: idle, loading, empty, success, recoverable error, permission-denied, offline/unavailable, cancellation. Do not conflate SQL compile error, fixture error, authentication error and fatal engine error. Use skeleton/placeholder UI consistent with shadcn.

## Visual and accessibility requirements
- Reuse existing shadcn components/tokens; no new global palette/theme overriding b0 without review.
- Accessible names for all fields/buttons; focus visible; use icons + text for verdict; not colors alone.
- Keyboard-only problem selection and run/submit flow; scalable text and resilient layout from 1024px wide desktop to high resolution.
- SQL editor should be labeled with instructions; ensure screen-reader usable fallback and accessible Results table.
- Show engine/dialect near editor to avoid sending T-SQL to PG or vice versa.
- Password field never autofills into logs or persisted component state beyond session.

## Typed IPC conventions
Create feature-local hooks calling `src/lib/tauri-api.ts` typed wrapper; Tauri commands must validate boundaries even when frontend uses Zod. No direct SQL driver from JS/WebView and no direct filesystem read of arbitrary user-selected path. Progress events keyed by `submissionId`, unsubscribe on unmount, recover via stored state after restart.

## UI acceptance examples
- Keyboard-only user can choose problem, edit SQL, Run, Submit, inspect verdict.
- User on external MSSQL profile never sees working Reset; direct forged IPC reset is also denied by backend.
- Password is never displayed after save; wrong TLS certificate yields actionable error.
- Import errors show exact manifest path; no partial install remains.
- Existing generated release workflow and dashboard are unchanged unless a measured incremental task needs modification.
