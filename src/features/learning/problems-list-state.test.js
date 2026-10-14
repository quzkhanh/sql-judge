import { describe, expect, test } from "bun:test"

import {
  PROBLEMS_PAGE_SIZE,
  difficultyTotals,
  filterCatalogProblems,
  pageWindow,
  paginateList,
  topicCounts,
} from "./problems-list-state"

const problems = [
  { id: "a:1", packId: "a", title: "Second salary", statement: "", difficulty: "medium", topics: ["Window functions"] },
  { id: "a:2", packId: "a", title: "Above average", statement: "", difficulty: "easy", topics: ["Subqueries", "Aggregation"] },
  { id: "b:1", packId: "b", title: "No orders", statement: "Customers", difficulty: "easy", topics: ["Joins"] },
  { id: "b:2", packId: "b", title: "Running total", statement: "", difficulty: "hard", topics: ["Window functions", "Aggregation"] },
]

describe("problems list state", () => {
  test("counts topics in first-seen order", () => {
    expect(topicCounts(problems)).toEqual([
      { topic: "Window functions", count: 2 },
      { topic: "Subqueries", count: 1 },
      { topic: "Aggregation", count: 2 },
      { topic: "Joins", count: 1 },
    ])
  })

  test("combines exact topic, pack, difficulty and text filters", () => {
    const all = { query: "", topic: "all", packId: "all", difficulty: "all" }
    expect(filterCatalogProblems(problems, all)).toHaveLength(4)
    expect(filterCatalogProblems(problems, { ...all, topic: "Aggregation" }).map((p) => p.id)).toEqual(["a:2", "b:2"])
    expect(filterCatalogProblems(problems, { ...all, topic: "Aggregation", packId: "b" }).map((p) => p.id)).toEqual(["b:2"])
    expect(filterCatalogProblems(problems, { ...all, difficulty: "easy" }).map((p) => p.id)).toEqual(["a:2", "b:1"])
    expect(filterCatalogProblems(problems, { ...all, query: "customers" }).map((p) => p.id)).toEqual(["b:1"])
    expect(filterCatalogProblems(problems, { ...all, topic: "Window" })).toHaveLength(0)
    const slugged = [{ ...problems[0], topics: ["window-functions"] }]
    expect(filterCatalogProblems(slugged, { ...all, query: "window functions" })).toHaveLength(1)
  })

  test("totals problems by difficulty", () => {
    expect(difficultyTotals(problems)).toEqual({ easy: 2, medium: 1, hard: 1 })
    expect(difficultyTotals([])).toEqual({ easy: 0, medium: 0, hard: 0 })
  })

  test("pages ten rows and clamps out-of-range pages", () => {
    const many = Array.from({ length: 23 }, (_, index) => index)
    expect(PROBLEMS_PAGE_SIZE).toBe(10)
    expect(paginateList(many, 0)).toMatchObject({ page: 0, totalPages: 3, start: 0, end: 10 })
    expect(paginateList(many, 9)).toMatchObject({ page: 2, start: 20, end: 23 })
    expect(paginateList([], 4)).toMatchObject({ page: 0, totalPages: 1, start: 0, end: 0 })
  })

  test("shows at most three page buttons around the current page", () => {
    expect(pageWindow(0, 1)).toEqual([0])
    expect(pageWindow(0, 7)).toEqual([0, 1, 2])
    expect(pageWindow(3, 7)).toEqual([2, 3, 4])
    expect(pageWindow(6, 7)).toEqual([4, 5, 6])
  })
})

describe("topic labels", () => {
  test("humanises pack topic slugs with SQL acronyms", async () => {
    const { topicLabel } = await import("./problems-list-state")
    expect(topicLabel("window-functions")).toBe("Window functions")
    expect(topicLabel("recursive-cte")).toBe("Recursive CTE")
    expect(topicLabel("cte")).toBe("CTE")
    expect(topicLabel("ctes")).toBe("CTEs")
    expect(topicLabel("joins")).toBe("Joins")
    expect(topicLabel("sql-null-handling")).toBe("SQL NULL handling")
    expect(topicLabel("Window functions")).toBe("Window functions")
  })
})

describe("collapsible topics", () => {
  const topics = ["a", "b", "c", "d", "e", "f", "g", "h"].map((topic, index) => ({ topic, count: index + 1 }))

  test("shows the most common topics first and counts the rest", async () => {
    const { visibleTopics } = await import("./problems-list-state")
    const collapsed = visibleTopics(topics, { expanded: false, selected: "all", limit: 3 })
    expect(collapsed.shown.map((t) => t.topic)).toEqual(["h", "g", "f"])
    expect(collapsed.hiddenCount).toBe(5)
    const expanded = visibleTopics(topics, { expanded: true, selected: "all", limit: 3 })
    expect(expanded.shown).toHaveLength(8)
    expect(expanded.hiddenCount).toBe(0)
  })

  test("keeps the selected topic visible while collapsed", async () => {
    const { visibleTopics } = await import("./problems-list-state")
    const result = visibleTopics(topics, { expanded: false, selected: "a", limit: 3 })
    expect(result.shown.map((t) => t.topic)).toEqual(["h", "g", "a"])
    expect(result.hiddenCount).toBe(5)
    expect(visibleTopics(topics.slice(0, 2), { expanded: false, selected: "all", limit: 3 }).hiddenCount).toBe(0)
  })
})
