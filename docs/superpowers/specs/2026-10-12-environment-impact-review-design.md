# Managed Environment Impact Review Design

## Scope

Improve the managed-environment destructive action UX before pack import IPC:
reset and delete must show an explicit, keyboard-reachable impact review before
the existing confirmed Tauri command is called.

## Design

- Keep the existing typed `resetManagedEnvironment` and
  `deleteManagedEnvironment` IPC contracts.
- Replace immediate `window.confirm` calls with local pending-action state.
- Render a review panel containing the environment ID, managed namespace,
  action, and exact lifecycle impact.
- Require an explicit Confirm button to call IPC with `confirmed: true`.
- Cancel clears pending state without calling IPC.
- Disable destructive controls while the environment is not `ready`, while an
  action is busy, or when the record is not managed.
- Rust authorization and ownership verification remain authoritative; the UI
  preview is informative and never a security boundary.

## Verification

- Existing Rust negative tests continue to cover forged identity, external
  mode, running state, and missing confirmation.
- Frontend typecheck/build verify the state and rendering contract.
- Manual keyboard smoke verifies focusable Confirm/Cancel controls and no IPC
  call before confirmation.

## Non-Goals

- No pack file picker or import IPC.
- No new frontend dependency.
- No change to destructive backend semantics or external database behavior.
