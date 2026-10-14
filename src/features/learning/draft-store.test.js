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

function memoryStorage(initial = null, enabled = true) {
  const box = { value: initial, enabled }
  return {
    box,
    persistence: {
      enabled: () => box.enabled,
      read: () => box.value,
      write: (value) => {
        box.value = value
      },
    },
  }
}

test("restores persisted drafts when keeping drafts is enabled", () => {
  const { box, persistence } = memoryStorage()
  createDraftStore(100, persistence).set("a", "SELECT a")
  expect(JSON.parse(box.value)).toEqual({ a: "SELECT a" })
  expect(createDraftStore(100, persistence).get("a", "starter")).toBe("SELECT a")
})

test("ignores malformed storage and keeps only string drafts", () => {
  const { persistence } = memoryStorage('{"a":"SELECT a","b":2}')
  const store = createDraftStore(100, persistence)
  expect(store.get("a", "x")).toBe("SELECT a")
  expect(store.get("b", "starter")).toBe("starter")
  expect(createDraftStore(100, memoryStorage("{oops").persistence).get("a", "starter")).toBe("starter")
})

test("clears persisted drafts when keeping drafts is disabled", () => {
  const { box, persistence } = memoryStorage('{"a":"SELECT a"}', false)
  const store = createDraftStore(100, persistence)
  expect(store.get("a", "starter")).toBe("starter")
  store.set("a", "SELECT b")
  expect(box.value).toBeNull()
  expect(store.get("a", "starter")).toBe("SELECT b")
})
