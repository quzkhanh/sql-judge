import { describe, expect, test } from "bun:test"

import {
  normalizeTlsMode,
  parseConnectionUrl,
  supportedTlsModes,
  suggestConnectionProfile,
} from "./connection-url"

describe("parseConnectionUrl", () => {
  test("parses a PostgreSQL URL and maps sslmode", () => {
    expect(
      parseConnectionUrl(
        "postgresql://alice:p%40ss@db.example:6543/app?sslmode=require"
      )
    ).toEqual({
      engine: "postgresql",
      host: "db.example",
      port: 6543,
      database: "app",
      username: "alice",
      secret: "p@ss",
      tlsMode: "require",
    })
  })

  test("parses a SQL Server URL with secure defaults", () => {
    expect(
      parseConnectionUrl("mssql://alice:p%40ss@sql.example/app?encrypt=true")
    ).toEqual({
      engine: "sqlserver",
      host: "sql.example",
      port: 1433,
      database: "app",
      username: "alice",
      secret: "p@ss",
      tlsMode: "verify_full",
    })
  })

  test("parses a JDBC SQL Server connection string", () => {
    expect(
      parseConnectionUrl(
        "jdbc:sqlserver://sql.example:1444;databaseName=app;user=alice;password=p%40ss;encrypt=true;trustServerCertificate=false"
      )
    ).toEqual({
      engine: "sqlserver",
      host: "sql.example",
      port: 1444,
      database: "app",
      username: "alice",
      secret: "p@ss",
      tlsMode: "verify_full",
    })
  })

  test("parses an ADO SQL Server connection string", () => {
    expect(
      parseConnectionUrl(
        "Server=tcp:sql.example,1433;Initial Catalog=app;User ID=alice;Password=p%40ss;Encrypt=True;TrustServerCertificate=False"
      )
    ).toEqual({
      engine: "sqlserver",
      host: "sql.example",
      port: 1433,
      database: "app",
      username: "alice",
      secret: "p@ss",
      tlsMode: "verify_full",
    })
  })

  test("rejects unsupported schemes and insecure TLS flags", () => {
    expect(() => parseConnectionUrl("mysql://alice:secret@db/app")).toThrow(
      "Unsupported database URL scheme"
    )
    expect(() =>
      parseConnectionUrl("postgres://alice:secret@db/app?sslmode=disable")
    ).toThrow("TLS verification cannot be disabled")
    expect(() =>
      parseConnectionUrl("postgres://alice:secret@db/app?sslmode=prefer")
    ).toThrow("PostgreSQL TLS mode is unsupported")
    expect(() =>
      parseConnectionUrl(
        "Server=sql.example;Database=app;User ID=alice;Password=secret;TrustServerCertificate=True"
      )
    ).toThrow("TLS certificate verification cannot be bypassed")
    expect(() =>
      parseConnectionUrl(
        "Server=sql.example,bad;Database=app;User ID=alice;Password=secret"
      )
    ).toThrow("Connection string port is invalid")
  })

  test("rejects malformed or incomplete input without returning partial fields", () => {
    expect(() => parseConnectionUrl("")).toThrow("Connection URL is required")
    expect(() => parseConnectionUrl("postgres://alice@/app")).toThrow(
      "Connection URL host is required"
    )
    expect(() => parseConnectionUrl("postgres://alice@db:bad/app")).toThrow(
      "Connection URL port is invalid"
    )
    expect(() =>
      parseConnectionUrl("Server=sql.example;User ID=alice;Password=secret")
    ).toThrow("Connection string database is required")
  })

  test("suggests valid profile metadata when importing a URL", () => {
    const parsed = parseConnectionUrl("postgres://alice@db.example/app")

    expect(suggestConnectionProfile(parsed)).toEqual({
      profileId: "postgresql-db-example",
      displayName: "PostgreSQL db.example/app",
    })
  })

  test("exposes only adapter-supported TLS modes per engine", () => {
    expect(supportedTlsModes("sqlserver")).toEqual(["verify_full", "require"])
    expect(supportedTlsModes("postgresql")).toEqual([
      "verify_full",
      "verify_ca",
      "require",
    ])
    expect(normalizeTlsMode("sqlserver", "verify_ca")).toBe("verify_full")
    expect(normalizeTlsMode("postgresql", "verify_ca")).toBe("verify_ca")
  })
})
