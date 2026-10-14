export type JudgeRequest = {
  generation: number
  problemId: string
}

export function beginJudgeRequest(
  generation: number,
  problemId: string
): JudgeRequest {
  return { generation: generation + 1, problemId }
}

export function isCurrentJudgeRequest(
  request: JudgeRequest,
  generation: number,
  problemId: string
): boolean {
  return request.generation === generation && request.problemId === problemId
}
