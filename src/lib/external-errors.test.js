import { describe, expect, test } from "bun:test"

import { readExternalError } from "./external-errors"
import { serializeDebugValue } from "./debug-events"

describe("readExternalError", () => {
  test("uses a stable engine-neutral fallback for unknown failures", () => {
    expect(readExternalError({ code: "unknown" })).toBe(
      "The external database operation failed."
    )
  })

  test("preserves a structured safe message", () => {
    expect(
      readExternalError({ message: "The SQL Server could not be reached." })
    ).toBe("The SQL Server could not be reached.")
  })

  test("reads standard Error instances", () => {
    expect(readExternalError(new Error("The external query timed out."))).toBe(
      "The external query timed out."
    )
  })
})

describe("serializeDebugValue", () => {
  test("redacts connection URLs and credential fields", () => {
    expect(
      serializeDebugValue({
        url: "postgres://alice:secret@db.example/app",
        password: "secret",
        secret: "secret",
      })
    ).toEqual({
      url: "[redacted]",
      password: "[redacted]",
      secret: "[redacted]",
    })
  })
})
