import { readFileSync } from "node:fs"
import { describe, expect, test } from "bun:test"

const viteConfig = readFileSync(new URL("../vite.config.ts", import.meta.url), "utf8")
const tauriConfig = JSON.parse(
  readFileSync(new URL("../src-tauri/tauri.conf.json", import.meta.url), "utf8")
)

describe("desktop development server", () => {
  test("uses the same collision-resistant port for Vite and Tauri", () => {
    const vitePort = viteConfig.match(/port:\s*(\d+)/u)?.[1]
    const devUrl = tauriConfig.build.devUrl

    expect(vitePort).toBe("1420")
    expect(devUrl).toBe("http://localhost:1420")
  })
})
