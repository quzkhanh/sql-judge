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

export type CreateManagedEnvironmentRequest = {
  envId: string
  packId: string
  namespace: string
}

export type CloneManagedEnvironmentRequest = {
  sourceEnvId: string
  envId: string
  namespace: string
}

export type ResetManagedEnvironmentRequest = {
  envId: string
  instanceId: string
  namespace: string
  confirmed: boolean
}

export type DeleteManagedEnvironmentRequest = {
  envId: string
  instanceId: string
  confirmed: boolean
}

export type SubmissionHistorySummary = {
  submissionId: string
  packId: string
  packVersion: string
  packHash: string
  engineVersion: string
  policyVersion: string
  verdict: string
  createdAt: string
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

export async function createManagedEnvironment(
  request: CreateManagedEnvironmentRequest
): Promise<ManagedEnvironmentSummary> {
  if (!isTauri()) {
    throw new Error(
      "Managed environments are unavailable in browser preview mode."
    )
  }

  return trackedInvoke<ManagedEnvironmentSummary>(
    "create_managed_environment",
    request
  )
}

export async function cloneManagedEnvironment(
  request: CloneManagedEnvironmentRequest
): Promise<ManagedEnvironmentSummary> {
  if (!isTauri()) {
    throw new Error(
      "Managed environments are unavailable in browser preview mode."
    )
  }

  return trackedInvoke<ManagedEnvironmentSummary>(
    "clone_managed_environment",
    request
  )
}

export async function resetManagedEnvironment(
  request: ResetManagedEnvironmentRequest
): Promise<ManagedEnvironmentSummary> {
  if (!isTauri()) {
    throw new Error(
      "Managed environments are unavailable in browser preview mode."
    )
  }

  if (!request.confirmed) {
    throw new Error("Explicit reset confirmation is required.")
  }

  return trackedInvoke<ManagedEnvironmentSummary>(
    "reset_managed_environment",
    request
  )
}

export async function deleteManagedEnvironment(
  request: DeleteManagedEnvironmentRequest
): Promise<void> {
  if (!isTauri()) {
    throw new Error(
      "Managed environments are unavailable in browser preview mode."
    )
  }

  if (!request.confirmed) {
    throw new Error("Explicit delete confirmation is required.")
  }

  await trackedInvoke<void>("delete_managed_environment", request)
}

export async function listSubmissionHistory(): Promise<
  SubmissionHistorySummary[]
> {
  if (!isTauri()) {
    return []
  }

  return trackedInvoke<SubmissionHistorySummary[]>("list_submission_history")
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
