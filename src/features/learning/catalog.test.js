import { describe, expect, test } from "bun:test"

import {
  catalogProblems,
  hasCompleteProblemMetadata,
  hasInstalledProblems,
  hasSampleData,
  sampleColumns,
} from "./catalog"

const completeProblem = {
  title: "A complete problem",
  statement: "Return the requested rows.",
  schema: [{ name: "items", columns: [{ name: "id", type: "integer" }] }],
  sampleRows: [{ id: 1 }],
  starterSql: "SELECT id FROM items",
}

describe("installed pack/problem states", () => {
  test("identifies an installed pack with no problems", () => {
    expect(hasInstalledProblems({ problems: [] })).toBe(false)
    expect(hasInstalledProblems({ problems: [{ id: "one" }] })).toBe(true)
  })

  test("requires schema, sample output, and starter SQL for judging", () => {
    expect(hasCompleteProblemMetadata(completeProblem)).toBe(true)
    expect(hasCompleteProblemMetadata({ ...completeProblem, schema: [] })).toBe(
      false
    )
    expect(
      hasCompleteProblemMetadata({ ...completeProblem, sampleRows: [] })
    ).toBe(false)
    expect(
      hasCompleteProblemMetadata({ ...completeProblem, starterSql: null })
    ).toBe(false)
  })
})

describe("hasSampleData", () => {
  test("rejects empty sample rows without indexing a first row", () => {
    expect(hasSampleData({ sampleRows: [] })).toBe(false)
    expect(hasSampleData({ sampleRows: [{}] })).toBe(false)
  })

  test("accepts a non-empty sample row", () => {
    expect(hasSampleData({ sampleRows: [{ id: 1 }] })).toBe(true)
    expect(sampleColumns({ sampleRows: [{ id: 1 }] })).toEqual(["id"])
  })

  test("keys immutable versions by the complete pack identity", () => {
    const problem = {
      id: "same-problem",
      title: "Same problem",
      difficulty: "easy",
      dialect: "postgresql",
      topics: ["testing"],
      statement: "A sufficiently long problem statement.",
      schema: [{ name: "items", columns: [{ name: "id", type: "integer" }] }],
      sampleRows: [{ id: 1 }],
      starterSql: "SELECT id FROM items",
    }
    const first = catalogProblems([
      {
        packId: "same-pack",
        version: "1.0.0",
        digest: "a".repeat(64),
        title: "First",
        dialect: "postgresql",
        problems: [problem],
      },
    ])[0]
    const second = catalogProblems([
      {
        packId: "same-pack",
        version: "2.0.0",
        digest: "b".repeat(64),
        title: "Second",
        dialect: "postgresql",
        problems: [problem],
      },
    ])[0]

    expect(first.id).not.toBe(second.id)
    expect(first.id).toContain("same-pack@1.0.0#")
    expect(second.id).toContain("same-pack@2.0.0#")
    expect(first.schema).toEqual(problem.schema)
    expect(first.sampleRows).toEqual(problem.sampleRows)
    expect(first.starterSql).toBe(problem.starterSql)
  })
})
