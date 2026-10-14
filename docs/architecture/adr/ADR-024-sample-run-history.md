# ADR-024: Sample Runs in Submission History

Status: Accepted

## Context

ADR-023 records the problem ID and per-fixture results for judged
submissions. The History screen design also lists sample runs ("Sample run",
"Ran on sample") next to submissions, and the Progress screen reports how many
sample runs were made without counting them toward solved problems. Before this
decision `run_sample` wrote nothing to history.

## Decision

- `submissions` gains `mode TEXT NOT NULL DEFAULT 'submit' CHECK (mode IN
  ('run', 'submit'))`, added by the same presence-checked `ALTER TABLE` step
  as ADR-023's `problem_id`. Existing rows become submissions; nothing is
  backfilled. `schema_version` stays at 1 for the reasons given in ADR-023.
- `run_sample` and `submit_solution` both persist a record with the problem ID
  that Rust loaded from the digest-verified pack, the verdict, and the fixtures
  that ran with their per-fixture elapsed time. Forged problem IDs are still
  rejected before judging and never written.
- `list_submission_history` returns `mode: "run" | "submit"` and
  `tests: { testId, verdict, elapsedMs }[]` for every record.
- Backups stay at version 2. `BackupSubmission` gains an optional `mode` that
  is serialized only for runs (`"run"`), so snapshots without runs are
  byte-for-byte the ADR-023 shape. Any value other than `"run"` or `"submit"`
  is rejected on restore, and older builds reject snapshots that contain runs
  through `deny_unknown_fields`, failing closed.

## Security and privacy

Only identifiers, verdicts and timings are stored. SQL text, result rows,
fixture contents and credentials are never written for runs or submissions.

## Consequences

- History grows with every sample run; there is no retention limit yet.
- Status and Progress derivations must count `mode = "submit"` only, so a
  passing sample run never marks a problem solved.
