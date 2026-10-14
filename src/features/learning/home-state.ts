export function homeProblems<T extends { id: string; packId: string }>(
  problems: T[],
  packId: string,
  draftProblemId: string
) {
  const packProblems = problems.filter((problem) => problem.packId === packId)
  const resumeIndex = Math.max(
    0,
    packProblems.findIndex((problem) => problem.id === draftProblemId)
  )
  const resume = packProblems[resumeIndex]

  return {
    resume,
    next: packProblems
      .slice(resumeIndex + 1)
      .concat(packProblems.slice(0, resumeIndex))
      .slice(0, 3),
  }
}
