import type {
  InstalledPack,
  InstalledProblem,
  ProblemDifficulty,
  ProblemDialect,
} from "@/lib/judge-api"

export type { ProblemDifficulty, ProblemDialect }

export type LearningProblem = InstalledProblem & {
  problemId: string
  packId: string
  packVersion: string
  packDigest: string
}

export function catalogProblemKey(
  pack: Pick<InstalledPack, "packId" | "version" | "digest">,
  problemId: string
) {
  return `${pack.packId}@${pack.version}#${pack.digest}:${problemId}`
}

export function catalogProblems(packs: InstalledPack[]): LearningProblem[] {
  return packs.flatMap((pack) =>
    pack.problems.map((problem) => {
      return {
        ...problem,
        id: catalogProblemKey(pack, problem.id),
        problemId: problem.id,
        packId: pack.packId,
        packVersion: pack.version,
        packDigest: pack.digest,
        schema: problem.schema ?? [],
        sampleRows: problem.sampleRows ?? [],
        starterSql: problem.starterSql ?? "",
      }
    })
  )
}

export function hasInstalledProblems(
  pack: Pick<InstalledPack, "problems">
): boolean {
  return pack.problems.length > 0
}

export function canJudgeProblem(
  problem: Pick<
    LearningProblem,
    "problemId" | "packId" | "packVersion" | "packDigest"
  >
): boolean {
  return [
    problem.problemId,
    problem.packId,
    problem.packVersion,
    problem.packDigest,
  ].every((value) => value.trim().length > 0)
}

export function hasCompleteProblemMetadata(
  problem: Pick<
    LearningProblem,
    "title" | "statement" | "schema" | "sampleRows" | "starterSql"
  >
): boolean {
  return (
    problem.title.trim().length > 0 &&
    problem.statement.trim().length > 0 &&
    problem.schema.length > 0 &&
    problem.schema.every(
      (table) => table.name.trim().length > 0 && table.columns.length > 0
    ) &&
    hasSampleData(problem) &&
    Boolean(problem.starterSql?.trim())
  )
}

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

export function hasSampleData(problem: Pick<LearningProblem, "sampleRows">) {
  return sampleColumns(problem).length > 0
}

export function sampleColumns(problem: Pick<LearningProblem, "sampleRows">) {
  const firstRow = problem.sampleRows.at(0)
  return firstRow ? Object.keys(firstRow) : []
}
