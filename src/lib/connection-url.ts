export type ParsedConnectionProfile = {
  engine: "postgresql" | "sqlserver"
  host: string
  port: number
  database: string
  username: string
  secret: string
  tlsMode: "verify_full" | "verify_ca" | "require"
}

export type SuggestedConnectionProfile = {
  profileId: string
  displayName: string
}

const POSTGRES_PORT = 5432
const SQLSERVER_PORT = 1433

export function supportedTlsModes(
  engine: ParsedConnectionProfile["engine"]
): ParsedConnectionProfile["tlsMode"][] {
  return engine === "sqlserver"
    ? ["verify_full", "require"]
    : ["verify_full", "verify_ca", "require"]
}

export function normalizeTlsMode(
  engine: ParsedConnectionProfile["engine"],
  tlsMode: ParsedConnectionProfile["tlsMode"]
): ParsedConnectionProfile["tlsMode"] {
  return engine === "sqlserver" && tlsMode === "verify_ca"
    ? "verify_full"
    : tlsMode
}

export function parseConnectionUrl(input: string): ParsedConnectionProfile {
  const value = input.trim()
  if (!value) {
    throw new Error("Connection URL is required")
  }

  if (/^jdbc:sqlserver:\/\//i.test(value)) {
    return parseJdbcSqlServer(value)
  }

  if (/^(?:postgres|postgresql|mssql|sqlserver):\/\//i.test(value)) {
    return parseStandardUrl(value)
  }

  if (/^[^=;]+=/i.test(value)) {
    return parseAdoSqlServer(value)
  }

  throw new Error("Unsupported database URL scheme")
}

export function suggestConnectionProfile(
  parsed: ParsedConnectionProfile
): SuggestedConnectionProfile {
  const hostSlug = parsed.host
    .toLowerCase()
    .replace(/[^a-z0-9]+/gu, "-")
    .replace(/^-+|-+$/gu, "")
  const engineName = parsed.engine === "sqlserver" ? "SQL Server" : "PostgreSQL"
  return {
    profileId: `${parsed.engine}-${hostSlug || "connection"}`,
    displayName: `${engineName} ${parsed.host}/${parsed.database}`,
  }
}

function parseStandardUrl(value: string): ParsedConnectionProfile {
  let url: URL
  try {
    url = new URL(value)
  } catch {
    if (/@\/(?:[^?]*)$/u.test(value)) {
      throw new Error("Connection URL host is required")
    }
    if (/:[^/]+\//u.test(value.replace(/^[a-z]+:\/\//iu, ""))) {
      throw new Error("Connection URL port is invalid")
    }
    throw new Error("Connection URL is malformed")
  }

  if (!url.hostname) {
    throw new Error("Connection URL host is required")
  }

  const scheme = url.protocol.slice(0, -1).toLowerCase()
  const engine =
    scheme === "postgres" || scheme === "postgresql"
      ? "postgresql"
      : "sqlserver"
  const database = decodePart(url.pathname.slice(1))
  if (!database) {
    throw new Error("Connection URL database is required")
  }

  const username = decodePart(url.username)
  if (!username) {
    throw new Error("Connection URL username is required")
  }

  return {
    engine,
    host: url.hostname,
    port: url.port
      ? parsePort(url.port)
      : engine === "postgresql"
        ? POSTGRES_PORT
        : SQLSERVER_PORT,
    database,
    username,
    secret: decodePart(url.password),
    tlsMode:
      engine === "postgresql"
        ? parsePostgresTls(url.searchParams.get("sslmode"))
        : parseSqlServerTls(url.searchParams),
  }
}

function parseJdbcSqlServer(value: string): ParsedConnectionProfile {
  const rest = value.slice("jdbc:sqlserver://".length)
  const separator = rest.indexOf(";")
  const server = separator === -1 ? rest : rest.slice(0, separator)
  const properties = parseProperties(
    separator === -1 ? "" : rest.slice(separator + 1)
  )
  const { host, port } = parseSqlServerHost(server, properties)

  return createSqlServerProfile({
    host,
    port,
    database: property(
      properties,
      "databasename",
      "database",
      "initial catalog"
    ),
    username: property(properties, "user", "userid", "uid", "username"),
    secret: property(properties, "password", "pwd"),
    tlsMode: parseSqlServerTls(properties),
  })
}

function parseAdoSqlServer(value: string): ParsedConnectionProfile {
  const properties = parseProperties(value)
  const server = property(
    properties,
    "server",
    "data source",
    "address",
    "addr"
  )
  const { host, port } = parseSqlServerHost(server, properties)

  return createSqlServerProfile({
    host,
    port,
    database: property(
      properties,
      "database",
      "initial catalog",
      "databasename"
    ),
    username: property(properties, "user id", "uid", "user", "username"),
    secret: property(properties, "password", "pwd"),
    tlsMode: parseSqlServerTls(properties),
  })
}

function createSqlServerProfile(fields: {
  host: string
  port: number
  database: string
  username: string
  secret: string
  tlsMode: ParsedConnectionProfile["tlsMode"]
}): ParsedConnectionProfile {
  if (!fields.database) {
    throw new Error("Connection string database is required")
  }
  if (!fields.username) {
    throw new Error("Connection string username is required")
  }
  return {
    engine: "sqlserver",
    ...fields,
  }
}

function parseSqlServerHost(
  rawHost: string,
  properties: Map<string, string>
): { host: string; port: number } {
  const server = decodePart(rawHost.trim().replace(/^tcp:/i, ""))
  if (!server) {
    throw new Error("Connection string host is required")
  }

  const match = server.match(/^(.+?)(?:,(\d+))?$/u)
  if (!match) {
    throw new Error("Connection string host is required")
  }
  let host = match[1].trim()
  let portValue = match[2] ?? property(properties, "port")
  if (server.includes(",") && !match[2]) {
    throw new Error("Connection string port is invalid")
  }
  if (!portValue) {
    const colonPort = host.match(/^(.*):(\d+)$/u)
    if (colonPort) {
      host = colonPort[1]
      portValue = colonPort[2]
    } else if (host.includes(":")) {
      throw new Error("Connection string port is invalid")
    }
  }
  if (!host) {
    throw new Error("Connection string host is required")
  }
  return { host, port: portValue ? parsePort(portValue) : SQLSERVER_PORT }
}

function parseSqlServerTls(
  properties: URLSearchParams | Map<string, string>
): ParsedConnectionProfile["tlsMode"] {
  const encrypt = getProperty(properties, "encrypt")
  const trustServerCertificate = getProperty(
    properties,
    "trustservercertificate"
  )
  if (trustServerCertificate?.toLowerCase() === "true") {
    throw new Error("TLS certificate verification cannot be bypassed")
  }
  if (
    trustServerCertificate &&
    trustServerCertificate.toLowerCase() !== "false"
  ) {
    throw new Error("SQL Server TLS mode is unsupported")
  }
  if (encrypt?.toLowerCase() === "false") {
    throw new Error("TLS verification cannot be disabled")
  }
  if (
    encrypt &&
    !["true", "yes", "strict", "mandatory"].includes(encrypt.toLowerCase())
  ) {
    throw new Error("SQL Server TLS mode is unsupported")
  }
  return "verify_full"
}

function parsePostgresTls(
  value: string | null
): ParsedConnectionProfile["tlsMode"] {
  switch (value?.toLowerCase()) {
    case "disable":
      throw new Error("TLS verification cannot be disabled")
    case "verify-ca":
      return "verify_ca"
    case "require":
      return "require"
    case "verify-full":
    case null:
    case undefined:
      return "verify_full"
    default:
      throw new Error("PostgreSQL TLS mode is unsupported")
  }
}

function parseProperties(value: string): Map<string, string> {
  const result = new Map<string, string>()
  for (const part of splitProperties(value)) {
    const separator = part.indexOf("=")
    if (separator <= 0) continue
    const key = part.slice(0, separator).trim().toLowerCase()
    const rawValue = part.slice(separator + 1).trim()
    result.set(key, decodePart(stripBraces(rawValue)))
  }
  return result
}

function splitProperties(value: string): string[] {
  const parts: string[] = []
  let start = 0
  let braces = 0
  let quote: string | null = null
  for (let index = 0; index < value.length; index += 1) {
    const character = value[index]
    if (quote) {
      if (character === quote && value[index - 1] !== "\\") quote = null
    } else if (character === "'" || character === '"') {
      quote = character
    } else if (character === "{") {
      braces += 1
    } else if (character === "}") {
      braces = Math.max(0, braces - 1)
    } else if (character === ";" && braces === 0) {
      parts.push(value.slice(start, index))
      start = index + 1
    }
  }
  parts.push(value.slice(start))
  return parts.filter((part) => part.trim())
}

function property(properties: Map<string, string>, ...keys: string[]): string {
  for (const key of keys) {
    const value = properties.get(key)
    if (value) return value
  }
  return ""
}

function getProperty(
  properties: URLSearchParams | Map<string, string>,
  key: string
): string | null {
  if (properties instanceof URLSearchParams) {
    return properties.get(key)
  }
  return properties.get(key) ?? null
}

function parsePort(value: string): number {
  if (!/^\d+$/u.test(value)) {
    throw new Error("Connection URL port is invalid")
  }
  const port = Number(value)
  if (!Number.isInteger(port) || port < 1 || port > 65535) {
    throw new Error("Connection URL port is invalid")
  }
  return port
}

function decodePart(value: string): string {
  try {
    return decodeURIComponent(value)
  } catch {
    throw new Error("Connection URL contains invalid encoding")
  }
}

function stripBraces(value: string): string {
  return value.startsWith("{") && value.endsWith("}")
    ? value.slice(1, -1)
    : value
}
