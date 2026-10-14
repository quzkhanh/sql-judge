import { describe, expect, test } from "bun:test"

import {
  PROBLEM_PAGE_SIZE,
  pageForProblem,
  paginateProblems,
  problemsForPack,
  stopVisibleAfter,
} from "./practice-workspace-state"

const problems = Array.from({ length: 12 }, (_, index) => ({
  id: `pack-a:${index + 1}`,
  packId: "pack-a",
  title: `Problem ${index + 1}`,
}))

describe("practice workspace state", () => {
  test("scopes the rail to the selected pack", () => {
    expect(
      problemsForPack(
        [...problems, { id: "pack-b:1", packId: "pack-b", title: "Other" }],
        "pack-a"
      )
    ).toHaveLength(12)
  })

  test("paginates five problems and clamps invalid pages", () => {
    expect(PROBLEM_PAGE_SIZE).toBe(5)
    expect(paginateProblems(problems, 0).items).toHaveLength(5)
    expect(paginateProblems(problems, 99)).toMatchObject({
      page: 2,
      totalPages: 3,
    })
  })

  test("maps quick-jump selection to its containing page", () => {
    expect(pageForProblem(problems, "pack-a:11")).toBe(2)
    expect(pageForProblem(problems, "missing")).toBe(0)
  })

  test("shows Stop only after the running grace period", () => {
    expect(stopVisibleAfter(1_499)).toBe(false)
    expect(stopVisibleAfter(1_500)).toBe(true)
  })
})
