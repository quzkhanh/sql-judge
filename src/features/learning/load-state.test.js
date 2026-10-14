import { describe, expect, test } from "bun:test"

import { learningLoadErrorMessage, learningLoadPhase } from "./load-state"

describe("learning load errors", () => {
  test("distinguishes catalog failures from an empty catalog", () => {
    expect({
      message: learningLoadErrorMessage("catalog"),
      phase: learningLoadPhase(true, null, false),
    }).toEqual({
      message:
        "Installed problem packs could not be loaded. Retry to check the local catalog.",
      phase: "loading",
    })
  })

  test("distinguishes history failures from an empty history", () => {
    expect({
      message: learningLoadErrorMessage("history"),
      error: learningLoadPhase(false, "load failed", false),
      empty: learningLoadPhase(false, null, false),
      ready: learningLoadPhase(false, null, true),
    }).toEqual({
      message:
        "Submission history could not be loaded. Retry to check local progress.",
      error: "error",
      empty: "empty",
      ready: "ready",
    })
  })
})
