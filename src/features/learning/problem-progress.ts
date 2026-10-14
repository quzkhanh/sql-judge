import type { JudgeVerdict, ProblemDifficulty } from "@/lib/judge-api"
import { catalogProblemKey } from "./catalog"
import { isJudgeVerdict, verdictPresentation } from "./verdict-display"

const MINUTE = 60_000
const HOUR = 60 * MINUTE
const DAY = 24 * HOUR

export type ProblemStatus = "solved" | "attempted"
export type StatusFilter = "all" | ProblemStatus | "unattempted"

export type ProblemProgress = {
  status: ProblemStatus
  count: number
  last: { verdict: string; at: number }
}

type HistoryEntry = {
  packId: string
  packVersion: string
  packHash: string
  problemId: string | null
  verdict: string
  createdAt: string
  mode?: "run" | "submit"
}

export function submittedAtMs(createdAt: string) {
  if (/^\d+$/.test(createdAt)) {
    return Number(createdAt) * 1000
  }
  const parsed = Date.parse(createdAt)
  return Number.isNaN(parsed) ? undefined : parsed
}

export function relativeTime(at: number, now: number) {
  const elapsed = Math.max(0, now - at)
  if (elapsed < MINUTE) return "just now"
  if (elapsed < HOUR) return `${Math.floor(elapsed / MINUTE)} min ago`
  if (elapsed < DAY) return `${Math.floor(elapsed / HOUR)} h ago`
  const days = Math.floor(elapsed / DAY)
  if (days < 2) return "yesterday"
  if (days < 7) return `${days} days ago`
  if (days < 14) return "last week"
  if (days < 30) return `${Math.floor(days / 7)} weeks ago`
  const months = Math.max(1, Math.floor(days / 30))
  return months === 1 ? "last month" : `${months} months ago`
}

// Legacy submissions without a problem id cannot be attributed and are skipped;
// sample runs (ADR-024) never count toward solved or attempted.
export function problemProgress(
  history: HistoryEntry[],
  keyFor: (packId: string, version: string, digest: string, problemId: string) => string
) {
  const progress = new Map<string, ProblemProgress>()
  for (const entry of history) {
    const at = submittedAtMs(entry.createdAt)
    if (!entry.problemId || at === undefined || entry.mode === "run") {
      continue
    }
    const key = keyFor(entry.packId, entry.packVersion, entry.packHash, entry.problemId)
    const current = progress.get(key)
    const solved = current?.status === "solved" || entry.verdict === "Accepted"
    progress.set(key, {
      status: solved ? "solved" : "attempted",
      count: (current?.count ?? 0) + 1,
      last: current && current.last.at > at ? current.last : { verdict: entry.verdict, at },
    })
  }
  return progress
}

const shortVerdicts: Partial<Record<JudgeVerdict, string>> = {
  TimeLimitExceeded: "Timeout",
  ResultLimitExceeded: "Result limit",
}

export function verdictShortLabel(verdict: string) {
  return (
    (isJudgeVerdict(verdict)
      ? (shortVerdicts[verdict] ?? verdictPresentation({ status: "completed", verdict }).label)
      : verdict)
  )
}

export function lastAttemptLabel(last: ProblemProgress["last"] | undefined, now: number) {
  return last ? `${verdictShortLabel(last.verdict)} · ${relativeTime(last.at, now)}` : "—"
}

export function matchesStatus(progress: Pick<ProblemProgress, "status"> | undefined, filter: StatusFilter) {
  if (filter === "all") return true
  if (filter === "unattempted") return !progress
  return progress?.status === filter
}

export function progressTotals(
  problems: Array<{ id: string; difficulty: ProblemDifficulty }>,
  progress: Map<string, Pick<ProblemProgress, "status">>
) {
  const totals = {
    solved: 0,
    total: problems.length,
    easy: { solved: 0, total: 0 },
    medium: { solved: 0, total: 0 },
    hard: { solved: 0, total: 0 },
  }
  for (const problem of problems) {
    const solved = progress.get(problem.id)?.status === "solved"
    totals[problem.difficulty].total += 1
    if (solved) {
      totals.solved += 1
      totals[problem.difficulty].solved += 1
    }
  }
  return totals
}

export function catalogKeyFor(packId: string, version: string, digest: string, problemId: string) {
  return catalogProblemKey({ packId, version, digest }, problemId)
}

export function historyProblemKey(
  entry: Pick<HistoryEntry, "packId" | "packVersion" | "packHash" | "problemId">
) {
  return entry.problemId
    ? catalogKeyFor(entry.packId, entry.packVersion, entry.packHash, entry.problemId)
    : null
}

export function resumeProblemId(progress: Map<string, Pick<ProblemProgress, "status" | "last">>) {
  let latest: [string, number] | null = null
  for (const [key, { status, last }] of progress) {
    if (status === "attempted" && (!latest || last.at > latest[1])) {
      latest = [key, last.at]
    }
  }
  return latest?.[0] ?? null
}
