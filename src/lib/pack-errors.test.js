import { describe, expect, test } from "bun:test"

import { isPackImportDisabled, readPackImportError } from "./pack-errors"

describe("readPackImportError", () => {
  test("preserves safe actionable native pack errors", () => {
    expect(readPackImportError("Pack source could not be loaded.")).toBe(
      "Pack source could not be loaded."
    )
    expect(
      readPackImportError("Pack runtime validation or publication failed.")
    ).toBe("Pack runtime validation or publication failed.")
    expect(
      readPackImportError({ message: "Pack source could not be loaded." })
    ).toBe("Pack source could not be loaded.")
  })

  test("explains the local PostgreSQL prerequisite", () => {
    expect(
      readPackImportError("Managed PostgreSQL runtime could not be started.")
    ).toContain("initdb")
    expect(
      readPackImportError("Managed PostgreSQL runtime could not be started.")
    ).toContain("pg_ctl")
  })

  test("does not expose unknown native error details", () => {
    expect(readPackImportError("/tmp/secret.sql: password=secret")).toBe(
      "Pack import failed closed. No unvalidated pack was published."
    )
    expect(readPackImportError(new Error("raw native error"))).toBe(
      "Pack import failed closed. No unvalidated pack was published."
    )
  })
})

describe("isPackImportDisabled", () => {
  test("disables native pack import outside desktop or while busy", () => {
    expect({
      browser: isPackImportDisabled(false, false),
      desktop: isPackImportDisabled(true, false),
      busy: isPackImportDisabled(true, true),
    }).toEqual({ browser: true, desktop: false, busy: true })
  })
})
