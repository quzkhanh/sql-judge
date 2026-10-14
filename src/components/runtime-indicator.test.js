import { expect, test } from "bun:test"

import { runtimeIndicator } from "./runtime-indicator"

test("describes the managed runtime for the sidebar footer", () => {
  expect(runtimeIndicator(undefined)).toEqual({ label: "Checking runtime", detail: "PostgreSQL · local socket", tone: "muted" })
  expect(runtimeIndicator({ available: true, state: "running", message: "ok" })).toEqual({ label: "Runtime ready", detail: "PostgreSQL · local socket", tone: "ok" })
  expect(runtimeIndicator({ available: false, state: "unavailable", message: "stopped" })).toEqual({ label: "Runtime stopped", detail: "Start it from Environments", tone: "err" })
  expect(runtimeIndicator(null)).toEqual({ label: "Runtime unavailable", detail: "Desktop runtime not connected", tone: "err" })
})
