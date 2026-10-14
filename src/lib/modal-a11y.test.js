import { describe, expect, test } from "bun:test"

import {
  nextModalFocusIndex,
  selectModalInitialFocus,
  shouldFocusModalContainer,
  shouldDismissModalOnEscape,
} from "./modal-a11y"

describe("nextModalFocusIndex", () => {
  test("wraps forward and backward through modal focusables", () => {
    expect(nextModalFocusIndex(2, 3, false)).toBe(0)
    expect(nextModalFocusIndex(0, 3, true)).toBe(2)
  })

  test("keeps focus at the only focusable element", () => {
    expect(nextModalFocusIndex(0, 1, false)).toBe(0)
    expect(nextModalFocusIndex(0, 1, true)).toBe(0)
  })

  test("does not dismiss while modal work is busy", () => {
    expect(shouldDismissModalOnEscape(false)).toBe(true)
    expect(shouldDismissModalOnEscape(true)).toBe(false)
  })

  test("falls back to the modal container when every control is disabled", () => {
    const container = { name: "dialog" }
    expect(selectModalInitialFocus(null, [], container)).toBe(container)
    expect(shouldFocusModalContainer(true, 0)).toBe(true)
    expect(shouldFocusModalContainer(false, 0)).toBe(false)
    expect(shouldFocusModalContainer(true, 1)).toBe(false)
  })
})
