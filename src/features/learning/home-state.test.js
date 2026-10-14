import { expect, test } from "bun:test"

import { homeProblems } from "./home-state"

const problems = [
  { id: "one", packId: "hr", title: "First" },
  { id: "two", packId: "hr", title: "Second" },
  { id: "three", packId: "hr", title: "Third" },
  { id: "four", packId: "hr", title: "Fourth" },
  { id: "other", packId: "sales", title: "Other pack" },
]

test("selects the resume problem and three following problems from one pack", () => {
  expect(homeProblems(problems, "hr", "two")).toEqual({
    resume: problems[1],
    next: [problems[2], problems[3], problems[0]],
  })
})

test("falls back to the first pack problem when no saved draft is available", () => {
  expect(homeProblems(problems, "hr", "missing")).toEqual({
    resume: problems[0],
    next: [problems[1], problems[2], problems[3]],
  })
})
