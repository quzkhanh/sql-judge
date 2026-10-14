import type { ManagedRuntimeStatus } from "@/lib/judge-api"

export type RuntimeIndicator = {
  label: string
  detail: string
  tone: "ok" | "err" | "muted"
}

// undefined = still loading; null = the status call failed or no native runtime exists.
export function runtimeIndicator(
  status: ManagedRuntimeStatus | null | undefined
): RuntimeIndicator {
  if (status === undefined) {
    return { label: "Checking runtime", detail: "PostgreSQL · local socket", tone: "muted" }
  }
  if (status === null) {
    return { label: "Runtime unavailable", detail: "Desktop runtime not connected", tone: "err" }
  }
  return status.available && status.state === "running"
    ? { label: "Runtime ready", detail: "PostgreSQL · local socket", tone: "ok" }
    : { label: "Runtime stopped", detail: "Start it from Environments", tone: "err" }
}
