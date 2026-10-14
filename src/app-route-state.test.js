import { expect, test } from "bun:test"

import { hashParams, problemHref, routeForLaunch } from "./app-route-state"

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

test("reaches Explorer and Settings and ignores query parameters", () => {
  expect(routeForLaunch("#explorer")).toBe("explorer")
  expect(routeForLaunch("#settings")).toBe("settings")
  expect(routeForLaunch("#problems?pack=hr-analytics")).toBe("problems")
  expect(routeForLaunch("#explorer?profile=replica")).toBe("explorer")
})

test("reads query parameters from the hash", () => {
  expect(hashParams("#problems?pack=hr-analytics").get("pack")).toBe("hr-analytics")
  expect(hashParams("#problems").get("pack")).toBeNull()
})

test("builds Solve links that round-trip the catalog key", () => {
  const key = "hr@1.2.0#sha256:abc:second-salary"
  const href = problemHref(key)
  expect(routeForLaunch(href)).toBe("solve")
  expect(decodeURIComponent(href.slice("#solve/".length))).toBe(key)
})
