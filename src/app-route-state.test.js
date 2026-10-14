import { expect, test } from "bun:test"

import { routeForLaunch } from "./app-route-state"

test("opens the application on Home", () => {
  expect(routeForLaunch()).toBe("home")
})
