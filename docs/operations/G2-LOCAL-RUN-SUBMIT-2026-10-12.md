# G2 Local Run-to-Submit Evidence — 2026-10-12

## Automated Evidence

- Native Tauri flow passed:
  `cargo test --manifest-path src-tauri/Cargo.toml --lib native_run_executes_sample_against_managed_runtime -- --nocapture`
- The test uses a temporary SQLite state and app-owned Unix-socket PostgreSQL
  runtime.
- The correct reference-equivalent query returned `completed` + `Accepted`,
  expected columns, and two preview rows.
- `SELECT 1` submitted through the native path returned `completed` +
  `WrongAnswer`.
- History contained exactly one record after one Run and one Submit, proving
  Run does not create submission history.
- Tauri startup reached `main webview finished loading` when launched against
  the existing Vite server with:
  `cargo run --manifest-path src-tauri/Cargo.toml --no-default-features --color always`

## UI Smoke Procedure

The following must be checked in the visible desktop window:

1. Run `bun run tauri dev` with port 3000 available.
2. Open `Practice` and run the reference-equivalent query for the first
   problem.
3. Confirm the result banner says `Accepted` and the result table has rows.
4. Submit `SELECT 1`.
5. Confirm the result banner says `WrongAnswer`, not `Accepted`.
6. Run a long query and press `Cancel`.
7. Confirm the result banner says `Cancelled`.
8. In browser preview mode, confirm the native runner unavailable message is
   shown and no fabricated result is displayed.

## Current Smoke Status

The terminal environment can start the webview, but this session has no
desktop-control tool to click and inspect the visible Tauri window. The UI
smoke steps therefore remain pending manual confirmation and G2 is not marked
complete solely from the native test.

## Notes

- The first `bun run tauri dev` attempt was blocked because port 3000 was
  already occupied by an existing Vite process; the native binary was then
  launched directly against that server successfully.
- No external database or credential was used.
