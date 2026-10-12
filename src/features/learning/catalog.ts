export type ProblemDifficulty = "easy" | "medium" | "hard"
export type ProblemDialect = "postgresql" | "sqlserver"

export type SchemaColumn = {
  name: string
  type: string
  nullable?: boolean
}

export type LearningProblem = {
  id: string
  title: string
  difficulty: ProblemDifficulty
  dialect: ProblemDialect
  topics: string[]
  statement: string
  schema: Array<{
    name: string
    columns: SchemaColumn[]
  }>
  sampleRows: Array<Record<string, string | number | null>>
  starterSql: string
}

export const learningProblems: LearningProblem[] = [
  {
    id: "above-average",
    title: "Employees above their department average",
    difficulty: "medium",
    dialect: "postgresql",
    topics: ["correlated-subquery", "aggregation", "window-functions"],
    statement:
      "Return one row per employee whose salary is strictly greater than the average of non-NULL salaries in their own department. Do not rely on row ordering.",
    schema: [
      {
        name: "departments",
        columns: [
          { name: "id", type: "integer" },
          { name: "name", type: "text" },
        ],
      },
      {
        name: "employees",
        columns: [
          { name: "id", type: "integer" },
          { name: "name", type: "text" },
          { name: "department_id", type: "integer" },
          { name: "salary", type: "numeric", nullable: true },
        ],
      },
    ],
    sampleRows: [
      { employee_id: 1, employee_name: "Alice" },
      { employee_id: 4, employee_name: "David" },
    ],
    starterSql:
      "SELECT e.id AS employee_id, e.name AS employee_name\nFROM employees AS e\nWHERE e.salary > (\n  SELECT AVG(peer.salary)\n  FROM employees AS peer\n  WHERE peer.department_id = e.department_id\n);",
  },
  {
    id: "first-order",
    title: "Find each customer's first order",
    difficulty: "easy",
    dialect: "postgresql",
    topics: ["window-functions", "date-filtering"],
    statement:
      "Return each customer and the date of their first completed order. Customers without completed orders must be excluded.",
    schema: [
      {
        name: "orders",
        columns: [
          { name: "customer_id", type: "integer" },
          { name: "ordered_at", type: "timestamp" },
          { name: "status", type: "text" },
        ],
      },
    ],
    sampleRows: [
      { customer_id: 11, first_order_at: "2025-01-03 09:00:00" },
      { customer_id: 14, first_order_at: "2025-02-18 12:30:00" },
    ],
    starterSql:
      "SELECT customer_id, MIN(ordered_at) AS first_order_at\nFROM orders\nWHERE status = 'completed'\nGROUP BY customer_id;",
  },
]

export function filterProblems(
  problems: LearningProblem[],
  query: string,
  dialect: ProblemDialect | "all",
  difficulty: ProblemDifficulty | "all"
) {
  const normalizedQuery = query.trim().toLowerCase()

  return problems.filter((problem) => {
    const matchesQuery =
      normalizedQuery.length === 0 ||
      [problem.title, problem.statement, ...problem.topics]
        .join(" ")
        .toLowerCase()
        .includes(normalizedQuery)
    const matchesDialect = dialect === "all" || problem.dialect === dialect
    const matchesDifficulty =
      difficulty === "all" || problem.difficulty === difficulty

    return matchesQuery && matchesDialect && matchesDifficulty
  })
}
