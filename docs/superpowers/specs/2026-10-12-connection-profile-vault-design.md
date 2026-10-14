# Connection Profile and Secret Vault Design

## Scope

Add persistent connection metadata and a native secret-vault boundary for
read-only external explorer profiles. Do not add external query execution or
destructive permissions in this slice.

## Stored Metadata

SQLite stores `profile_id`, `display_name`, `engine`, `host`, `port`, `database`,
`username`, `tls_mode`, and optional `secret_ref`. It never stores a password or
token. Values are validated before insertion and on read.

## Secret Boundary

The native vault interface supports `set`, `get`, and `delete`. Production uses
the OS credential vault through `keyring`; tests use an injected memory backend.
Vault errors are structured and generic at the Tauri boundary. Secret values are
never serialized into response types or debug/error text.

## Commands

- `list_connection_profiles`: metadata only.
- `save_connection_profile`: validate metadata and optionally save a secret.
- `delete_connection_profile`: delete the vault entry before metadata.
- `test_connection_profile`: future capability-gated operation; not implemented
  in this slice.

## Negative Cases

- Empty IDs, names, hosts, databases, or usernames are rejected.
- Ports outside `1..=65535` are rejected.
- Unknown engines and TLS policies are rejected.
- Secret values and references are absent from backup JSON.
- Vault failure never writes a secret to SQLite or returns it to the frontend.
