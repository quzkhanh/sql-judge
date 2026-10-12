# Pack Import Design

## Scope

Complete the remaining T16 pack-import slice for local directories and
`.sqljudge.zip` archives. Imported content is validated structurally and against
the app-owned PostgreSQL runtime before immutable publication.

## Design

- `problem-pack` uses a bounded Rust ZIP reader with deflate support and the
  existing directory loader.
- `PackData::from_path` dispatches only to a real directory or a `.zip` file.
- Path traversal, symlinks, duplicate/case-colliding names, malformed archives,
  excessive file/byte limits, and extreme compression ratios fail closed.
- Tauri `import_pack` accepts a local path, validates the pack, verifies the
  managed runtime marker, executes trusted reference tests in disposable
  fixtures, then publishes the immutable digest.
- The frontend Packs screen exposes typed import status and generic failure
  messaging without displaying credentials or raw SQL.

## Security

- Runtime validation uses the managed PostgreSQL session and constrained runner.
- Publication occurs only after validation succeeds.
- No external environment is used or modified.
- Rust remains authoritative for path, ownership, runtime, and publication
  checks; frontend status is informational.

## Verification

- ZIP positive and negative tests cover valid archives, malformed input,
  traversal, and compression bombs.
- Native Tauri test imports the sample ZIP against real PostgreSQL and asserts
  published pack identity and digest.
- Workspace tests, clippy/fmt, frontend typecheck/build, and focused frontend
  lint pass.
- Manual Packs UI smoke remains required before marking T16 complete.
