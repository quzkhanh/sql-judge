import { describe, expect, test } from "bun:test"

import {
  beginExplorerRequest,
  clearExplorerOutput,
  isCurrentExplorerRequest,
  shouldApplyProfileLoad,
} from "./connections-state"

describe("external explorer output state", () => {
  test("clears output and rejects late Explorer test responses", () => {
    const request = beginExplorerRequest(3, "profile-1")

    expect({
      cleared: clearExplorerOutput({
        metadata: { tables: [{ name: "sample" }] },
        queryResult: { columns: [], rows: [] },
        selectedProfileId: "profile-1",
      }),
      staleGeneration: isCurrentExplorerRequest(request, 5, "profile-1"),
      staleProfile: isCurrentExplorerRequest(
        request,
        request.generation,
        "profile-2"
      ),
      current: isCurrentExplorerRequest(
        request,
        request.generation,
        "profile-1"
      ),
    }).toEqual({
      cleared: {
        metadata: null,
        queryResult: null,
        selectedProfileId: "profile-1",
      },
      staleGeneration: false,
      staleProfile: false,
      current: true,
    })
  })
})

describe("connection profile load state", () => {
  test("rejects a load that started before a local profile mutation", () => {
    expect({
      stale: shouldApplyProfileLoad(0, 1),
      current: shouldApplyProfileLoad(1, 1),
    }).toEqual({ stale: false, current: true })
  })
})
