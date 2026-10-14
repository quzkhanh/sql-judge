export type LearningLoadResource =
  "catalog" | "history" | "profiles" | "environments" | "connection"
export type LearningLoadPhase = "loading" | "error" | "empty" | "ready"

export function learningLoadErrorMessage(
  resource: LearningLoadResource
): string {
  switch (resource) {
    case "catalog":
      return "Installed problem packs could not be loaded. Retry to check the local catalog."
    case "history":
      return "Submission history could not be loaded. Retry to check local progress."
    case "profiles":
      return "Connection profiles could not be loaded. Retry to check saved profiles."
    case "environments":
      return "Managed environments could not be loaded. Retry to check managed state."
    case "connection":
      return "Connection test could not be completed. Retry to verify read-only access."
  }
}

export function learningLoadPhase(
  loading: boolean,
  error: string | null,
  hasItems: boolean
): LearningLoadPhase {
  if (loading) {
    return "loading"
  }
  if (error) {
    return "error"
  }
  return hasItems ? "ready" : "empty"
}
