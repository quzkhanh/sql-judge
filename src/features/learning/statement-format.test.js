import { expect, test } from "bun:test"

import { statementBlocks } from "./statement-format"

test("splits paragraphs on blank lines and keeps single newlines inside", () => {
  expect(statementBlocks("First line\nstill first.\n\n\nSecond.")).toEqual([
    [{ kind: "text", text: "First line\nstill first." }],
    [{ kind: "text", text: "Second." }],
  ])
})

test("parses **strong** and `code` spans without HTML", () => {
  expect(statementBlocks("Return the **second highest** salary or `NULL`.")).toEqual([
    [
      { kind: "text", text: "Return the " },
      { kind: "strong", text: "second highest" },
      { kind: "text", text: " salary or " },
      { kind: "code", text: "NULL" },
      { kind: "text", text: "." },
    ],
  ])
})

test("leaves unmatched markers and markup-looking text as plain text", () => {
  expect(statementBlocks("a ** b and ` c <b>x</b>")).toEqual([[{ kind: "text", text: "a ** b and ` c <b>x</b>" }]])
  expect(statementBlocks("   ")).toEqual([])
})
