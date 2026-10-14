import { describe, expect, test } from "bun:test"

import { readConnectionProfileError } from "./connection-errors"

describe("readConnectionProfileError", () => {
  test("preserves safe native validation errors from string or object IPC failures", () => {
    expect(
      readConnectionProfileError("The connection profile is invalid.")
    ).toBe("The connection profile is invalid.")
    expect(
      readConnectionProfileError({
        message: "Credential vault is unavailable.",
      })
    ).toBe("Credential vault is unavailable.")
  })

  test("hides unknown native details", () => {
    expect(readConnectionProfileError("password=secret")).toBe(
      "Connection profile could not be saved."
    )
  })
})
