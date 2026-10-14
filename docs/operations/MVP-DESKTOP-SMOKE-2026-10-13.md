# MVP Desktop Smoke Evidence - 2026-10-13

## Automated Evidence

- RED observed before the backup message implementation:
  `bun test src/lib/backup-errors.test.js` returned `0 pass`, `1 fail`, and
  `1 error` with `Cannot find module './backup-errors'`.
- RED observed before the shared modal focus helper implementation:
  `bun test src/lib/modal-a11y.test.js` returned `0 pass`, `1 fail`, and
  `1 error` with `Cannot find module './modal-a11y'`.
- RED observed before the busy Escape policy implementation:
  `bun test src/lib/modal-a11y.test.js` returned `0 pass`, `1 fail`, and
  `1 error` because `shouldDismissModalOnEscape` was not exported.
- RED observed before command-specific backup debug redaction:
  `bun test src/lib/tauri-debug.test.js` returned `0 pass`, `1 fail`, and
  `1 error` because `serializeDebugInvokeArgs` was not exported.
- Focused UI-adjacent tests passed:
  `bun test src/lib/backup-errors.test.js src/lib/backup-export.test.js src/lib/modal-a11y.test.js src/lib/tauri-debug.test.js src/features/learning/environment-impact.test.ts src/lib/pack-errors.test.js src/lib/connection-errors.test.js src/lib/connection-url.test.js`
  (`25 pass`, `0 fail`, `58 expect() calls`).
- Full Bun suite passed: `bun test` (`44 pass`, `0 fail`, `102 expect() calls`).
- Native app-storage suite passed: `cargo test -p app-storage --test storage`
  (`17 passed`, `0 failed`). Native Tauri suite passed:
  `cargo test --manifest-path src-tauri/Cargo.toml --lib` (`41 passed`,
  `0 failed`), including journaled startup recovery, running-runtime adoption,
  stopped-runtime non-adoption, crash-after-schema-created, mismatched recovery
  identity, pending recovery export, oversized export, recovery cleanup ownership,
  and direct SQL Server operation authorization regressions.
- Pack import suite passed: `cargo test -p problem-pack --test import`
  (`30 passed`, `0 failed`).
- PostgreSQL integration suites passed: 19 integration, 9 policy, and 4 runtime
  tests in `cargo test --workspace -- --test-threads=1`.
- The PostgreSQL integration suite includes constrained managed schema SQL
  execution, and the native suite covers malformed judge policy errors plus
  boolean and large-integer result JSON typing.
- SQL Server unit/policy suites passed:
  `cargo test -p judge-sqlserver --all-targets --features sqlserver-integration`
  (`10 passed`, `0 failed`), with 3 fixture tests explicitly ignored. The
  disposable SQL Server Docker harness is **BLOCKED** because the Docker daemon
  is unavailable.
- Direct SQL Server Tauri metadata/SELECT paths now establish a fresh explorer,
  re-run `test_connection`, and deny the operation unless the backend verifies
  `read_only=true`; native tests cover denied writer/unverified and allowed
  verified operation paths.
- Managed schema setup cleanup now removes only a namespace created by the
  current attempt; setup failures preserve pre-existing matching-owned
  namespaces. Recovery cleanup treats an absent namespace as a safe no-op and
  retains recovery on existing marker/ownership mismatch.
- Startup behavior is launchable and fail-closed: clean state loads without
  starting a stopped managed runtime; an already-running runtime is adopted only
  after its runtime marker and instance identity verify. If recovery is pending
  but the runtime is stopped or unavailable, Tauri registers managed state with
  recovery pending and leaves the explicit Start Managed Runtime path available;
  managed resource operations remain blocked until start and reconciliation
  succeed. Mismatched markers remain unavailable for explicit recovery handling.
- Workspace Rust tests and `cargo clippy --workspace --all-targets --all-features
-- -D warnings` passed; the Tauri crate's separate all-target clippy check also
  passed. Repository-wide `bun run lint` passed.
- PostgreSQL harness passed: loopback-only server and low-privilege runner were
  enforced. SQL Server harness remains **BLOCKED** because the Docker daemon is
  unavailable.
- Python pack validation remains **BLOCKED** because the environment lacks the
  `jsonschema` dependency. The six-platform release artifact matrix has not been
  run, so release packaging remains **UNVERIFIED**.
- The Bun setup has no DOM/React component test harness. The modal regression
  covers the shared focus-wrap algorithm; the integrated focus, Escape,
  restoration, and inert-background checks remain manual.
- Frontend typecheck passed: `bun run typecheck`.
- Production frontend build passed: `bun run build`.
- Full ESLint passed after replacing the scaffold's mount-time state effects with
  external-store or lazy initialization patterns.
- Changed-file Prettier check passed after `bunx prettier --write`. A separate
  repository-wide check reports formatting differences in untouched
  scaffold/docs files; those files were not changed.
- `git diff --check` passed for the changed frontend files.
- Existing native Run/Submit evidence remains in
  `docs/operations/G2-LOCAL-RUN-SUBMIT-2026-10-12.md`; this task did not use an
  external database or credential.

## Desktop Smoke Procedure

Run `bun run tauri dev` with port 3000 available, then use keyboard navigation
only for the following checks:

| Area             | Check                                                                                                                                                                                                                                                                                 | Status                                                            |
| ---------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------- |
| Practice         | Focus and select a problem, edit the SQL editor, run, and inspect bounded results                                                                                                                                                                                                     | **BLOCKED/PENDING** - port 3000 occupied; no desktop-control tool |
| Practice         | Submit an accepted query and a wrong query; confirm distinct verdicts                                                                                                                                                                                                                 | **BLOCKED/PENDING** - port 3000 occupied; no desktop-control tool |
| Packs            | Open the archive/folder picker, import a local pack, and observe progress, success, and bounded error states                                                                                                                                                                          | **BLOCKED/PENDING** - port 3000 occupied; no desktop-control tool |
| Environments     | Select a ready managed environment, open the `alertdialog`, verify initial focus, Tab/Shift+Tab trap, Escape/cancel while idle, focus restoration, inert/`aria-hidden` background, and that Escape/cancel cannot dismiss during in-flight rebuild/delete, then confirm rebuild/delete | **BLOCKED/PENDING** - port 3000 occupied; no desktop-control tool |
| Connections      | Save a profile without exposing its secret, verify TLS choices are limited to Verify full/CA/Require TLS, then test the Explorer connection                                                                                                                                           | **BLOCKED/PENDING** - port 3000 occupied; no desktop-control tool |
| Explorer         | Inspect metadata and run bounded SELECT; confirm no reset/delete controls are present                                                                                                                                                                                                 | **BLOCKED/PENDING** - port 3000 occupied; no desktop-control tool |
| History/Progress | Select a backup, verify the full page background (including sibling controls) is inert/`aria-hidden`, confirm initial focus, Tab/Shift+Tab trap, Escape/cancel, focus restoration, disabled file input, then confirm restore and unchanged PostgreSQL schemas                         | **BLOCKED/PENDING** - port 3000 occupied; no desktop-control tool |

## Current Smoke Status

The fresh `bun run tauri dev` attempt was blocked because port 3000 was already
occupied by another Vite process. The current session has no desktop-control
tool to focus and inspect the visible Tauri window. The listed UI checks remain
pending manual confirmation; no desktop acceptance claim is made from the
automated frontend checks alone.

## Security Checks

- External profiles remain Explorer-only; destructive environment controls are
  enabled only for ready managed records and still require Rust-side ownership
  and confirmation checks.
- Backup messages expose bounded status text only. Snapshot content, passwords,
  connection strings, and raw SQL are not rendered in error messages.
- Native restore rejects omitted or changed managed identities and pending
  recovery before any replacement. A valid restore replaces submission history
  transactionally while preserving external explorer metadata; recovery journal
  rows are not restored or preserved.
- `export_backup` results and `restore_backup` `snapshotJson` arguments are
  command-specifically redacted from debug events; SQL/secret redaction remains
  covered by the existing generic sanitizer.
