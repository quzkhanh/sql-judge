# G2 Local Run-to-Submit E2E Design

## Scope

Close checkpoint G2 for the local learning flow: Run a correct SQL solution,
Submit an incorrect solution, and cancel a long-running query through the
desktop UI backed by the app-owned PostgreSQL runtime.

This work uses native end-to-end coverage plus a documented desktop UI smoke
check. It does not add browser automation or a second test framework.

## Goals

- Prove the native Run and Submit request path against real disposable
  PostgreSQL.
- Verify `Accepted` and `WrongAnswer` remain distinct at the native boundary.
- Verify the preview columns and rows consumed by the learning workspace.
- Verify submission history is written only for Submit.
- Provide a repeatable manual UI smoke procedure for Run, Submit, and Cancel.
- Record unavailable-runtime behavior honestly instead of treating browser
  preview as a successful judge run.

## Non-Goals

- No Playwright, Vitest, WebDriver, or other new test dependency.
- No changes to the SQL policy, result limits, cancellation protocol, or
  external database behavior.
- No modification of the existing debug panel or debug event changes.
- No claim of automated pixel or browser-level coverage.

## Design

### Native E2E boundary

The existing Tauri native test harness will use a temporary SQLite state and an
app-owned Unix-socket PostgreSQL runtime. It will exercise the same request
dispatcher used by the `run_sample` and `submit_solution` commands rather than
mocking the judge or database.

The test will:

1. Start the managed runtime and load the supported public problem.
2. Run the reference-equivalent query with mode `run`.
3. Assert `completed`, `Accepted`, expected column names, and bounded preview
   rows.
4. Submit `SELECT 1` with mode `submit`.
5. Assert `completed`, `WrongAnswer`, and exactly one history record.

Existing PostgreSQL integration tests remain responsible for typed result
decoding, cancellation, watchdog timeout, policy rejection, and result limits.
The Tauri test only verifies that those backend semantics are mapped into the
native command response used by the UI.

### UI smoke flow

Run the desktop app with `bun run tauri dev`, start the managed runtime from
the Environments screen if necessary, and navigate to Practice.

The smoke checklist will cover:

- Run a correct query and observe an Accepted banner and a non-empty result
  table.
- Submit `SELECT 1` and observe WrongAnswer without a false Accepted state.
- Start a long-running query, press Cancel, and observe Cancelled.
- Confirm browser preview mode reports native runner unavailable rather than
  returning a fabricated result.

The checklist will identify the exact problem, SQL, expected verdict, and
failure evidence to capture. It is an operational acceptance check, not a
substitute for native or PostgreSQL automated tests.

## Error Handling and Security

- Tests use only the temporary managed runtime and disposable SQLite state.
- No external connection, production database, or user credential is used.
- The UI continues to consume typed `JudgeRunResult` responses and displays
  unavailable/error states distinctly.
- Cancellation remains request-ID scoped; no global cancellation behavior is
  introduced.
- A failed smoke run is recorded as failed evidence and is not silently
  converted into a passing checkpoint.

## Verification

- `cargo test --manifest-path src-tauri/Cargo.toml --lib --bins`
- `cargo test --workspace`
- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `bun run typecheck`
- `bun run build`
- Manual `bun run tauri dev` smoke checklist with real local PostgreSQL.

## Acceptance Mapping

- AC-003: typed Run/Submit request and response path.
- AC-004: real PostgreSQL execution for Run and Submit.
- AC-007: distinct Accepted, WrongAnswer, Cancelled, and error outcomes.
- AC-013: Submit history persistence without recording Run as a submission.
- AC-017: cancellation remains request-scoped and fail-closed.
