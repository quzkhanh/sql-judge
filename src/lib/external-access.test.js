import { describe, expect, test } from "bun:test"

import { isReadOnlyVerified, readOnlyStatusLabel } from "./external-access"

describe("external access status", () => {
  test("only treats an explicit readOnly response as verified", () => {
    expect(isReadOnlyVerified(true)).toBe(true)
    expect(isReadOnlyVerified(false)).toBe(false)
    expect(isReadOnlyVerified(undefined)).toBe(false)
    expect(readOnlyStatusLabel(true)).toBe("read-only access verified")
    expect(readOnlyStatusLabel(false)).toBe("read-only access not verified")
  })
})
