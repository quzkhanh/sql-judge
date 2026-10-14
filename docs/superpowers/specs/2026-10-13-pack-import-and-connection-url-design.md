# Pack Import and Connection URL Design

## Scope

Fix the pack import workflow and add local connection URL autodetection. Pack
import remains a native, owned-runtime operation. URL parsing only fills the
existing connection form; it never connects to a server or persists a secret
until the user explicitly saves the profile.

## Pack Import

Add the Tauri dialog plugin with capability-scoped permissions. Packs can be
selected as either a `.sqljudge.zip` archive or a real directory. The existing
Rust `PackData` loader, structural validator, managed runtime ownership check,
runtime validation, and immutable publication remain the security boundary.

The UI shows native command errors with safe actionable text. It does not show
raw SQL, credentials, driver errors, or stack traces. Browser preview keeps the
manual path fallback and reports that native selection requires the desktop
application.

The dialog plugin is the only new dependency. It is preferred over passing
archive bytes through the webview because it preserves native filesystem path
handling, keeps pack size limits in the Rust loader, and allows directory
selection without weakening the existing path and symlink validation.

## Connection URL Parsing

Add a pure parser for:

- `postgres://` and `postgresql://` URLs
- `mssql://` and `sqlserver://` URLs
- `jdbc:sqlserver://` URLs
- SQL Server ADO-style semicolon connection strings

The parser returns only normalized profile fields: engine, host, port,
database, username, password, and TLS mode. PostgreSQL defaults to port 5432;
SQL Server defaults to 1433. TLS defaults to `verify_full`. Explicit
`trustServerCertificate=true` or equivalent is represented as a rejected TLS
policy, not silently downgraded. Unsupported schemes, missing hosts, malformed
ports, and ambiguous connection strings fail without partial mutation.

The UI provides an Import URL action beside the profile form. The password is
placed only in the transient password input and is redacted from debug invoke
events when the profile is saved.

## Testing

- Bun tests cover every supported URL family, percent-encoding, defaults,
  TLS rejection, malformed input, and secret non-logging behavior.
- Native tests cover pack error mapping and dialog command registration.
- Existing structural/runtime pack tests and workspace/native/frontend checks
  remain required.
