import { describe, expect, test } from "bun:test"

import { serializeDebugInvokeArgs, serializeDebugInvokeResult } from "./tauri"

const snapshot = {
  environments: [{ ownershipNonce: "nonce-canary", namespace: "sj_private" }],
  submissions: [{ packHash: "hash-canary" }],
}

describe("Tauri debug redaction", () => {
  test("redacts restore snapshot JSON from invoke arguments", () => {
    const debugArgs = serializeDebugInvokeArgs("restore_backup", {
      snapshotJson: JSON.stringify(snapshot),
    })

    expect(JSON.stringify(debugArgs)).not.toContain("nonce-canary")
    expect(JSON.stringify(debugArgs)).not.toContain("sj_private")
    expect(debugArgs).toEqual({ snapshotJson: "[redacted backup JSON]" })
  })

  test("redacts exported backup contents from invoke results", () => {
    const debugResult = serializeDebugInvokeResult("export_backup", snapshot)

    expect(JSON.stringify(debugResult)).not.toContain("nonce-canary")
    expect(JSON.stringify(debugResult)).not.toContain("hash-canary")
    expect(debugResult).toBe("[redacted backup snapshot]")
  })
})
