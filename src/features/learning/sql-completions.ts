import type { CatalogTable, ProblemDialect } from "@/lib/judge-api"

export type SqlCompletionItem = {
  label: string
  detail: string
  insertText: string
  kind: "keyword" | "snippet" | "table" | "column"
}

const KEYWORDS = ["SELECT", "WITH", "JOIN", "GROUP BY", "ORDER BY"]

const SNIPPETS = [
  {
    label: "SELECT columns FROM table",
    insertText: "SELECT ${1:columns}\nFROM ${2:table}",
  },
  {
    label: "WITH name AS (...) SELECT",
    insertText: "WITH ${1:name} AS (\n  ${2:SELECT ...}\n)\nSELECT ${3:*}\nFROM ${1:name}",
  },
]

export function sqlCompletionItems(
  dialect: ProblemDialect,
  schema: CatalogTable[]
): SqlCompletionItem[] {
  const keywords = KEYWORDS.map((label) => ({
    label,
    detail: `${dialect} keyword`,
    insertText: label,
    kind: "keyword" as const,
  }))
  const snippets = SNIPPETS.map((snippet) => ({
    ...snippet,
    detail: `${dialect} snippet`,
    kind: "snippet" as const,
  }))
  const schemaItems = schema.flatMap((table) => [
    {
      label: table.name,
      detail: "table",
      insertText: table.name,
      kind: "table" as const,
    },
    ...table.columns.map((column) => ({
      label: column.name,
      detail: `${table.name}.${column.name} · ${column.type}`,
      insertText: column.name,
      kind: "column" as const,
    })),
  ])

  return [...keywords, ...snippets, ...schemaItems]
}
