import { isTauri, trackedInvoke } from "@/lib/tauri"

export type JudgeVerdict =
  | "Accepted"
  | "WrongAnswer"
  | "SyntaxError"
  | "RuntimeError"
  | "TimeLimitExceeded"
  | "ResultLimitExceeded"
  | "PolicyViolation"
  | "Cancelled"
  | "JudgeError"

export type JudgeRunRequest = {
  problemId: string
  sql: string
  mode: "run" | "submit"
}

export type JudgeRunResult = {
  status: "completed" | "unavailable" | "error"
  verdict?: JudgeVerdict
  columns?: Array<{ name: string; type: string }>
  rows?: Array<Array<string | number | null>>
  durationMs: number
  message: string
}

export type ManagedEnvironmentSummary = {
  envId: string
  instanceId: string
  packId: string
  namespace: string
  mode: "managed" | "external_explorer"
  state:
    | "provisioning"
    | "ready"
    | "running"
    | "rebuilding"
    | "deleting"
    | "needs_recovery"
    | "error"
}

export type ManagedRuntimeStatus = {
  available: boolean
  state: "running" | "unavailable"
  message: string
}

export async function runJudge(
  request: JudgeRunRequest
): Promise<JudgeRunResult> {
  if (!isTauri()) {
    return {
      status: "unavailable",
      durationMs: 0,
      message:
        "The native PostgreSQL runner is not connected in browser preview mode.",
    }
  }

  return trackedInvoke<JudgeRunResult>(
    request.mode === "run" ? "run_sample" : "submit_solution",
    request
  )
}

export async function listManagedEnvironments(): Promise<
  ManagedEnvironmentSummary[]
> {
  if (!isTauri()) {
    return []
  }

  return trackedInvoke<ManagedEnvironmentSummary[]>("list_managed_environments")
}

export async function inspectManagedEnvironment(
  envId: string
): Promise<ManagedEnvironmentSummary | null> {
  if (!isTauri()) {
    return null
  }

  return trackedInvoke<ManagedEnvironmentSummary | null>(
    "inspect_managed_environment",
    { envId }
  )
}

export async function getManagedRuntimeStatus(): Promise<ManagedRuntimeStatus> {
  if (!isTauri()) {
    return {
      available: false,
      state: "unavailable",
      message:
        "The managed PostgreSQL runtime is unavailable in browser preview mode.",
    }
  }

  return trackedInvoke<ManagedRuntimeStatus>("managed_runtime_status")
}

export async function startManagedRuntime(): Promise<ManagedRuntimeStatus> {
  if (!isTauri()) {
    return {
      available: false,
      state: "unavailable",
      message:
        "The managed PostgreSQL runtime can only start in the desktop app.",
    }
  }

  return trackedInvoke<ManagedRuntimeStatus>("start_managed_runtime")
}
