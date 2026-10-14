export const PROBLEM_PAGE_SIZE = 5
export const STOP_GRACE_PERIOD_MS = 1_500

export function problemsForPack<T extends { packId: string }>(
  problems: T[],
  packId: string
) {
  return problems.filter((problem) => problem.packId === packId)
}

export function paginateProblems<T>(items: T[], requestedPage: number) {
  const totalPages = Math.max(1, Math.ceil(items.length / PROBLEM_PAGE_SIZE))
  const page = Math.min(Math.max(0, requestedPage), totalPages - 1)
  const start = page * PROBLEM_PAGE_SIZE

  return {
    items: items.slice(start, start + PROBLEM_PAGE_SIZE),
    page,
    totalPages,
    start,
    end: Math.min(start + PROBLEM_PAGE_SIZE, items.length),
  }
}

export function pageForProblem<T extends { id: string }>(
  items: T[],
  problemId: string
) {
  const index = items.findIndex((item) => item.id === problemId)
  return index < 0 ? 0 : Math.floor(index / PROBLEM_PAGE_SIZE)
}

export function stopVisibleAfter(elapsedMs: number) {
  return elapsedMs >= STOP_GRACE_PERIOD_MS
}
