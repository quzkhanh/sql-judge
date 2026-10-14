import { describe, expect, test } from "bun:test"

import { sqlCompletionItems } from "./sql-completions"

const schema = [
  {
    name: "tasks",
    columns: [{ name: "id", type: "integer", nullable: false }],
  },
]

describe("sqlCompletionItems", () => {
  test("offers PostgreSQL snippets and active-schema identifiers", () => {
    const labels = sqlCompletionItems("postgresql", schema).map(
      (item) => item.label
    )

    expect(labels).toEqual(expect.arrayContaining(["SELECT", "WITH", "tasks", "id"]))
  })

  test("does not include schema identifiers from another problem", () => {
    expect(sqlCompletionItems("postgresql", []).map((item) => item.label)).not.toContain(
      "tasks"
    )
  })
})
