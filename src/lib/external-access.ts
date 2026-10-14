export function isReadOnlyVerified(
  readOnly: boolean | null | undefined
): boolean {
  return readOnly === true
}

export function readOnlyStatusLabel(
  readOnly: boolean | null | undefined
): string {
  return isReadOnlyVerified(readOnly)
    ? "read-only access verified"
    : "read-only access not verified"
}
