import { expect, test } from "bun:test"

import { routeForLaunch } from "./app-route-state"

test("opens the application on Home", () => {
  expect(routeForLaunch()).toBe("home")
})

test("opens the Problems list from its route", () => {
  expect(routeForLaunch("#problems")).toBe("problems")
})

test("opens Solve from a problem route", () => {
  expect(routeForLaunch("#solve/hr%401.2%23digest%3Asecond-salary")).toBe("solve")
})

test("keeps every sidebar route reachable by hash", () => {
  for (const route of ["history", "progress", "packs", "environments", "connections"]) {
    expect(routeForLaunch(`#${route}`)).toBe(route)
  }
  expect(routeForLaunch("#solve/")).toBe("home")
  expect(routeForLaunch("#unknown")).toBe("home")
})
