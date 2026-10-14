import { describe, expect, test } from "bun:test"

import { nativeRequest } from "./judge-api"

describe("nativeRequest", () => {
  test("wraps Tauri struct arguments under their parameter name", () => {
    const request = { path: "/tmp/pack.sqljudge.zip" }

    expect(nativeRequest(request)).toEqual({ request })
  })
})
