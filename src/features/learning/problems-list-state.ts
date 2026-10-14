import type { ProblemDifficulty } from "./catalog"

export const PROBLEMS_PAGE_SIZE = 10
const PAGE_WINDOW_SIZE = 3

type ListProblem = {
  packId: string
  title: string
  statement: string
  difficulty: ProblemDifficulty
  topics: string[]
}

export type ProblemListFilters = {
  query: string
  topic: string | "all"
  packId: string | "all"
  difficulty: ProblemDifficulty | "all"
}

export function topicCounts(problems: Pick<ListProblem, "topics">[]) {
  const counts = new Map<string, number>()
  for (const problem of problems) {
    for (const topic of problem.topics) {
      counts.set(topic, (counts.get(topic) ?? 0) + 1)
    }
  }
  return Array.from(counts, ([topic, count]) => ({ topic, count }))
}

export function filterCatalogProblems<T extends ListProblem>(
  problems: T[],
  filters: ProblemListFilters
) {
  const query = filters.query.trim().toLowerCase()
  return problems.filter(
    (problem) =>
      (filters.topic === "all" || problem.topics.includes(filters.topic)) &&
      (filters.packId === "all" || problem.packId === filters.packId) &&
      (filters.difficulty === "all" ||
        problem.difficulty === filters.difficulty) &&
      (query.length === 0 ||
        [problem.title, problem.statement, ...problem.topics, ...problem.topics.map(topicLabel)]
          .join(" ")
          .toLowerCase()
          .includes(query))
  )
}

export function difficultyTotals(
  problems: Pick<ListProblem, "difficulty">[]
): Record<ProblemDifficulty, number> {
  const totals = { easy: 0, medium: 0, hard: 0 }
  for (const problem of problems) {
    totals[problem.difficulty] += 1
  }
  return totals
}

export function paginateList<T>(items: T[], requestedPage: number) {
  const totalPages = Math.max(1, Math.ceil(items.length / PROBLEMS_PAGE_SIZE))
  const page = Math.min(Math.max(0, requestedPage), totalPages - 1)
  const start = page * PROBLEMS_PAGE_SIZE
  const end = Math.min(start + PROBLEMS_PAGE_SIZE, items.length)
  return { items: items.slice(start, end), page, totalPages, start, end }
}

export function pageWindow(page: number, totalPages: number) {
  const size = Math.min(PAGE_WINDOW_SIZE, totalPages)
  const first = Math.min(Math.max(0, page - 1), totalPages - size)
  return Array.from({ length: size }, (_, index) => first + index)
}

const TOPIC_ACRONYMS: Record<string, string> = {
  cte: "CTE",
  ctes: "CTEs",
  sql: "SQL",
  null: "NULL",
  nulls: "NULLs",
}

// Pack topics are slugs (`window-functions`); the artboards show sentence-case labels.
export function topicLabel(topic: string) {
  const words = topic
    .split("-")
    .map((word) => TOPIC_ACRONYMS[word] ?? word)
  const first = words[0] ?? ""
  return [first.charAt(0).toUpperCase() + first.slice(1), ...words.slice(1)].join(" ")
}

export function visibleTopics(
  topics: Array<{ topic: string; count: number }>,
  options: { expanded: boolean; selected: string; limit: number }
) {
  const ranked = [...topics].sort((left, right) => right.count - left.count)
  if (options.expanded || ranked.length <= options.limit) {
    return { shown: ranked, hiddenCount: 0 }
  }
  let shown = ranked.slice(0, options.limit)
  const selected = ranked.find((entry) => entry.topic === options.selected)
  if (selected && !shown.includes(selected)) {
    shown = [...shown.slice(0, options.limit - 1), selected]
  }
  return { shown, hiddenCount: ranked.length - shown.length }
}
