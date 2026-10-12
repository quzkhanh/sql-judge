import { emit } from "@tauri-apps/api/event"
import { invoke, isTauri, type InvokeArgs } from "@tauri-apps/api/core"

import {
  emitDebugEvent,
  serializeDebugValue,
  serializeError,
} from "./debug-events"

const JUDGE_COMMANDS = new Set(["run_sample", "submit_solution"])

function debugInvokeArgs(command: string, args: InvokeArgs | undefined) {
  if (!JUDGE_COMMANDS.has(command) || !args || typeof args !== "object") {
    return serializeDebugValue(args)
  }

  const record = args as Record<string, unknown>
  return serializeDebugValue({
    ...record,
    sql:
      typeof record.sql === "string"
        ? `[redacted SQL: ${record.sql.length} chars]`
        : "[redacted]",
  })
}

function debugInvokeResult(command: string, result: unknown) {
  if (!JUDGE_COMMANDS.has(command) || !result || typeof result !== "object") {
    return serializeDebugValue(result)
  }

  const record = result as Record<string, unknown>
  return serializeDebugValue({
    ...record,
    rows: Array.isArray(record.rows)
      ? `[redacted rows: ${record.rows.length}]`
      : "[redacted]",
  })
}

export async function trackedInvoke<T>(command: string, args?: InvokeArgs) {
  const startedAt = performance.now()

  try {
    const result = await invoke<T>(command, args)

    emitDebugEvent({
      id: crypto.randomUUID(),
      kind: "invoke",
      command,
      args: debugInvokeArgs(command, args),
      durationMs: Math.round((performance.now() - startedAt) * 100) / 100,
      status: "success",
      result: debugInvokeResult(command, result),
      timestamp: new Date().toISOString(),
    })

    return result
  } catch (error) {
    emitDebugEvent({
      id: crypto.randomUUID(),
      kind: "invoke",
      command,
      args: debugInvokeArgs(command, args),
      durationMs: Math.round((performance.now() - startedAt) * 100) / 100,
      status: "error",
      error: serializeError(error),
      timestamp: new Date().toISOString(),
    })

    throw error
  }
}

export async function trackedEmit<T>(event: string, payload?: T) {
  await emit(event, payload)

  emitDebugEvent({
    id: crypto.randomUUID(),
    kind: "runtime-event",
    name: event,
    source: "tauri-event",
    payload: serializeDebugValue(payload),
    timestamp: new Date().toISOString(),
  })
}

export { isTauri }
