# Operations Runbook — local DB, judge and import

## Check app / managed engine health
1. Open Settings -> Database Runtime; verify managed engine mode, version and instance ownership.
2. For development, check local dedicated PG test container/process health; no automatic connection to default user PG without verification.
3. If runtime missing, offer setup guidance; problem catalog and history must remain usable.

## `JudgeError` incident
Find `submission_id` + trace ID in structured logs. Distinguish setup/schema import failure, reference SQL failure, runner disconnection, timeout, result resource limit, and cleanup failure. Reproduce with immutable pack hash and engine version in a disposable environment. If reference fails, report broken pack; do not blame user's SQL.

## Pack import fails
Inspect stage: ZIP safety -> JSON Schema -> referenced file checks -> SQL dialect compatibility -> actual DB schema/fixture apply -> reference output typed validation. A structural validator pass is not enough. On failure, discard staged content and ephemeral DB only after ownership check; preserve previous pack version.

## Environment stuck after crash
Run startup reconciliation on durable environment ledger. Only touch owned environments whose DB marker and instance ID match. If mismatch, mark `needs_recovery`; never generic drop. Show safe manual recovery guidance and options to export history first.

## External connection unavailable
Never block app launch or local packs. Display secure TLS/auth/network distinction; avoid retry storms, use bounded backoff, do not log secrets. User-provided servers may be off, behind VPN or have disabled read permissions.

## Reset/rebuild requested
Preview exact managed environment and table count, obtain explicit confirmation, lock submissions, build fresh environment, validate, switch pointer, retire only verified old env. The UI never exposes a universal `truncate all tables` action for external DBs.

## Data recovery
SQLite metadata may be backed up while respecting database consistency; exports exclude `secret_ref` and credential bytes. Packs are immutable and content-hashed; restore app ledger separately from managed DB data and detect missing runtime/resources.
