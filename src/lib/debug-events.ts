export const DEBUG_EVENT_NAME = "ctui:debug"

type SerializedError = {
  name: string
  message: string
  stack?: string
}

export type ExternalLinkDebugEvent = {
  id: string
  kind: "external-link"
  href: string
  timestamp: string
}

export type InvokeDebugEvent = {
  id: string
  kind: "invoke"
  command: string
  args: unknown
  durationMs: number
  status: "success" | "error"
  result?: unknown
  error?: SerializedError
  timestamp: string
}

export type RuntimeEventDebugEvent = {
  id: string
  kind: "runtime-event"
  name: string
  source: "window" | "tauri-event"
  payload?: unknown
  timestamp: string
}

export type LogDebugEvent = {
  id: string
  kind: "log"
  level: "log" | "info" | "warn" | "error" | "debug"
  message: string
  timestamp: string
}

export type DebugEvent =
  | ExternalLinkDebugEvent
  | InvokeDebugEvent
  | RuntimeEventDebugEvent
  | LogDebugEvent

export function serializeError(error: unknown): SerializedError {
  if (error instanceof Error) {
    return {
      name: error.name,
      message: redactDebugText(error.message),
      stack: error.stack ? redactDebugText(error.stack) : undefined,
    }
  }

  return {
    name: "Error",
    message: redactDebugText(
      typeof error === "string"
        ? error
        : (JSON.stringify(error) ?? String(error))
    ),
  }
}

export function serializeDebugValue(value: unknown): unknown {
  if (
    value === null ||
    value === undefined ||
    typeof value === "boolean" ||
    typeof value === "number"
  ) {
    return value ?? null
  }

  if (typeof value === "string") {
    return redactDebugText(value)
  }

  if (value instanceof Error) {
    return serializeError(value)
  }

  try {
    return sanitizeDebugValue(value)
  } catch {
    return String(value)
  }
}

const REDACTED_DEBUG_KEYS = new Set([
  "connectionString",
  "connection_string",
  "hash",
  "href",
  "ownershipNonce",
  "ownership_nonce",
  "password",
  "query",
  "rows",
  "search",
  "secret",
  "sql",
  "token",
  "url",
])

const CONNECTION_DEBUG_KEYS = new Set(["connectionstring", "connectionurl"])

const CONNECTION_URI_PATTERN =
  /\b(?:jdbc:(?:postgresql|sqlserver):\/\/|postgres(?:ql)?:\/\/|sqlserver:\/\/)[^\s"'`]+/gi

function normalizedDebugKey(key: string) {
  return key.replace(/[-_]/g, "").toLowerCase()
}

function sanitizeDebugValue(value: unknown): unknown {
  if (Array.isArray(value)) {
    return value.map((item) => sanitizeDebugValue(item))
  }
  if (value && typeof value === "object") {
    return Object.fromEntries(
      Object.entries(value).map(([key, item]) => {
        const normalizedKey = normalizedDebugKey(key)
        if (CONNECTION_DEBUG_KEYS.has(normalizedKey)) {
          return [key, `[redacted ${key}]`]
        }
        if (
          REDACTED_DEBUG_KEYS.has(key) ||
          REDACTED_DEBUG_KEYS.has(normalizedKey)
        ) {
          return [
            key,
            Array.isArray(item)
              ? `[redacted ${key}: ${item.length}]`
              : "[redacted]",
          ]
        }
        return [key, sanitizeDebugValue(item)]
      })
    )
  }
  if (typeof value === "string") {
    return redactDebugText(value)
  }
  return value ?? null
}

export function redactUrl(value: string): string {
  try {
    const url = new URL(value)
    if (url.protocol === "mailto:" || url.protocol === "tel:") {
      return `${url.protocol}[redacted]`
    }
    return url.origin
  } catch {
    return "[redacted]"
  }
}

export function redactDebugText(value: string): string {
  value = value.replace(CONNECTION_URI_PATTERN, "[redacted connection URI]")
  if (
    /["']?(?:password|pwd|secret|token|connection[_-]?string|sql|query|rows|href|search|hash)["']?\s*[:=]/i.test(
      value
    )
  ) {
    return "[redacted sensitive text]"
  }
  if (
    /^\s*(?:select|with|insert|update|delete|create|alter|drop|truncate|grant|revoke|copy|do)\b/i.test(
      value
    )
  ) {
    return `[redacted SQL: ${value.length} chars]`
  }
  return value.length > 500 ? `${value.slice(0, 500)}…` : value
}

export function emitDebugEvent(detail: DebugEvent) {
  if (typeof window === "undefined") {
    return
  }

  window.dispatchEvent(
    new CustomEvent<DebugEvent>(DEBUG_EVENT_NAME, {
      detail: sanitizeDebugValue(detail) as DebugEvent,
    })
  )
}

export function forwardConsoleArguments(
  originalMethod: (...data: unknown[]) => void,
  args: unknown[]
) {
  originalMethod(...args.map((argument) => serializeDebugValue(argument)))
}
