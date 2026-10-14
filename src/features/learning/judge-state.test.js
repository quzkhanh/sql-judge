import { describe, expect, test } from "bun:test"

import { beginJudgeRequest, isCurrentJudgeRequest } from "./judge-state"

describe("judge request state", () => {
  test("rejects late results from an older generation or problem", () => {
    const request = beginJudgeRequest(4, "pack-a/problem-1")

    expect({
      staleGeneration: isCurrentJudgeRequest(request, 6, "pack-a/problem-1"),
      staleProblem: isCurrentJudgeRequest(
        request,
        request.generation,
        "pack-b/problem-2"
      ),
      current: isCurrentJudgeRequest(
        request,
        request.generation,
        "pack-a/problem-1"
      ),
    }).toEqual({
      staleGeneration: false,
      staleProblem: false,
      current: true,
    })
  })
})
