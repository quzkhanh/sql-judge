import type { ManagedEnvironmentSummary } from "@/lib/judge-api"

export type EnvironmentAction = "reset" | "delete"

type ImpactDescription = {
  environmentId: string
  namespace: string
  summary: string
  details: string[]
}

export function describeEnvironmentImpact(
  action: EnvironmentAction,
  environment: ManagedEnvironmentSummary
): ImpactDescription {
  if (action === "reset") {
    return {
      environmentId: environment.envId,
      namespace: environment.namespace,
      summary: "Rebuild this managed environment in an isolated replacement.",
      details: [
        "A replacement namespace is provisioned and validated first.",
        "The environment switches only after the replacement is ready.",
        "The old namespace is removed only after its ownership marker matches.",
      ],
    }
  }

  return {
    environmentId: environment.envId,
    namespace: environment.namespace,
    summary: "Delete this managed environment and its owned schema.",
    details: [
      "The managed schema and its environment record will be removed.",
      "External databases and explorer connections are not touched.",
      "This action cannot be undone from the app.",
    ],
  }
}

export function isDestructiveActionAvailable(
  _action: EnvironmentAction,
  environment: ManagedEnvironmentSummary,
  busy: boolean
): boolean {
  return (
    !busy && environment.mode === "managed" && environment.state === "ready"
  )
}
