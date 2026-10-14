import { describe, expect, test } from "bun:test"

import {
  forwardConsoleArguments,
  serializeDebugValue,
  serializeError,
} from "./debug-events"

describe("forwardConsoleArguments", () => {
  test("preserves every original console argument", () => {
    const received = []
    forwardConsoleArguments(
      (...args) => received.push(args),
      ["label", { value: 7 }]
    )
    expect(received).toEqual([["label", { value: 7 }]])
  })

  test("sanitizes every forwarded argument recursively", () => {
    const received = []
    const secret = "password=do-not-forward"
    forwardConsoleArguments(
      (...args) => received.push(args),
      [
        "label",
        {
          password: secret,
          ownershipNonce: "nonce-do-not-forward",
          nested: [{ sql: "SELECT secret FROM accounts" }],
        },
        new Error(secret),
      ]
    )

    expect(received).toHaveLength(1)
    expect(received[0]).toHaveLength(3)
    expect(JSON.stringify(received)).not.toContain(secret)
    expect(JSON.stringify(received)).not.toContain("nonce-do-not-forward")
    expect(JSON.stringify(received)).not.toContain(
      "SELECT secret FROM accounts"
    )
    expect(received[0][0]).toBe("label")
  })

  test("sanitizes window error messages through the error serializer", () => {
    const secret = "password=do-not-store"
    const serialized = serializeError(new Error(secret))

    expect(serialized.message).not.toContain(secret)
    expect(serialized.message).toBe("[redacted sensitive text]")
  })

  test("redacts database connection URI credentials and query secrets", () => {
    const received = []
    forwardConsoleArguments(
      (...args) => received.push(args),
      [
        "postgres://user:password@db.example.test/app?sslmode=require&token=secret",
        "jdbc:sqlserver://db.example.test:1433/app;user=sa;password=secret",
        "Server=db.example.test;User ID=sa;Password=secret;Database=training",
        "Server=db.example.test;Uid=sa;Pwd=secret;Database=training",
      ]
    )

    expect(JSON.stringify(received)).not.toContain("password@db.example.test")
    expect(JSON.stringify(received)).not.toContain("token=secret")
    expect(JSON.stringify(received)).not.toContain("password=secret")
    expect(JSON.stringify(received)).not.toContain("Pwd=secret")
    expect(received[0][0]).toContain("[redacted connection URI]")
  })

  test("redacts nested connection URL keys without losing safe diagnostics", () => {
    const serialized = serializeDebugValue({
      status: "connected",
      nested: {
        connectionUrl:
          "postgresql://user:password@localhost/training?sslmode=require",
        connectionString: "Server=db;Password=secret;Database=training",
      },
    })

    expect(serialized).toEqual({
      status: "connected",
      nested: {
        connectionUrl: "[redacted connectionUrl]",
        connectionString: "[redacted connectionString]",
      },
    })
  })
})
