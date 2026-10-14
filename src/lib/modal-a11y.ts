export function nextModalFocusIndex(
  currentIndex: number,
  focusableCount: number,
  backwards: boolean
): number {
  if (focusableCount <= 0) {
    return -1
  }

  const step = backwards ? -1 : 1
  return (currentIndex + step + focusableCount) % focusableCount
}

export function shouldDismissModalOnEscape(busy: boolean): boolean {
  return !busy
}

export function shouldFocusModalContainer(
  busy: boolean,
  focusableCount: number
): boolean {
  return busy && focusableCount === 0
}

export function selectModalInitialFocus<T>(
  preferred: T | null,
  focusable: T[],
  container: T
): T {
  return preferred ?? focusable[0] ?? container
}
