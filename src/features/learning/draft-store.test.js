import { expect, test } from "bun:test"

import { createDraftStore } from "./draft-store"

test("keeps drafts per problem across workspace remounts", () => {
  const store = createDraftStore(10)
  expect(store.get("a", "SELECT 1")).toBe("SELECT 1")
  store.set("a", "SELECT a")
  store.set("b", "SELECT b")
  expect(store.get("a", "starter")).toBe("SELECT a")
  expect(store.get("b", "starter")).toBe("SELECT b")
})

test("bounds stored SQL and keeps an intentionally empty draft", () => {
  const store = createDraftStore(5)
  store.set("a", "SELECT 123")
  expect(store.get("a", "")).toBe("SELEC")
  store.set("a", "")
  expect(store.get("a", "starter")).toBe("")
  store.set("", "ignored")
  expect(store.get("", "fallback")).toBe("fallback")
})
