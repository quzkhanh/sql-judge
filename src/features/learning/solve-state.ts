import type {
  JudgeRunResult,
  JudgeTestResult,
  ManagedEnvironmentSummary,
} from "@/lib/judge-api"

export function adjacentProblems<T extends { id: string }>(
  problems: T[],
  problemId: string
) {
  const index = problems.findIndex((problem) => problem.id === problemId)
  if (index < 0) {
    return { previous: undefined, next: undefined, position: 0 }
  }
  return {
    previous: problems[index - 1],
    next: problems[index + 1],
    position: index + 1,
  }
}

function plural(count: number, noun: string) {
  return `${count} ${noun}${count === 1 ? "" : "s"}`
}

export function resultSummary(result: JudgeRunResult) {
  const duration = `${result.durationMs} ms`
  if (result.status !== "completed" || !result.columns) {
    return duration
  }
  return [
    plural(result.rows?.length ?? 0, "row"),
    plural(result.columns.length, "column"),
    duration,
  ].join(" · ")
}

export function environmentForProblem(
  environments: ManagedEnvironmentSummary[],
  problem: { packId: string; packVersion: string; packDigest: string }
) {
  const pinned = environments.filter(
    (environment) =>
      environment.mode === "managed" &&
      environment.packId === problem.packId &&
      environment.packVersion === problem.packVersion &&
      environment.packDigest === problem.packDigest
  )
  const chosen =
    pinned.find((environment) => environment.state === "ready") ?? pinned[0]
  return chosen
    ? { namespace: chosen.namespace, ready: chosen.state === "ready" }
    : undefined
}

// Native catalog rows are BTreeMaps (alphabetical keys); restore the declared column order.
export function orderedColumns(
  rows: Array<Record<string, unknown>>,
  preferred: string[]
) {
  const present: string[] = []
  for (const row of rows) {
    for (const key of Object.keys(row)) {
      if (!present.includes(key)) {
        present.push(key)
      }
    }
  }
  return [
    ...preferred.filter((name) => present.includes(name)),
    ...present.filter((name) => !preferred.includes(name)),
  ]
}

export function rowOrderNote(rowOrder: "ordered" | "unordered") {
  return rowOrder === "unordered"
    ? "Any row order is accepted. Rows are compared as a multiset, so duplicate rows count."
    : "Row order matters. Rows are compared in order, and duplicate rows count."
}

export function limitLabel(timeLimitMs: number) {
  return timeLimitMs >= 1000
    ? `${Number((timeLimitMs / 1000).toFixed(1))} s limit`
    : `${timeLimitMs} ms limit`
}

export function fixtureSummary(tests: JudgeTestResult[]) {
  if (tests.length === 0) {
    return undefined
  }
  const passed = tests.filter((test) => test.verdict === "Accepted").length
  return `${passed} of ${tests.length} fixture${tests.length === 1 ? "" : "s"} passed`
}

export function initialFixture(tests: JudgeTestResult[]) {
  return Math.max(0, tests.findIndex((test) => test.verdict !== "Accepted"))
}

export function fixtureName(testId: string) {
  return testId.replaceAll("-", " ")
}
