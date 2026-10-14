import {
  describeEnvironmentImpact,
  isDestructiveActionAvailable,
  type EnvironmentAction,
} from "./environment-impact"

function assert(condition: boolean, message: string): void {
  if (!condition) {
    throw new Error(message)
  }
}

function assertEqual<T>(actual: T, expected: T): void {
  assert(
    actual === expected,
    `Expected ${String(expected)}, got ${String(actual)}`
  )
}

function assertIncludes(value: string, fragment: string): void {
  assert(value.includes(fragment), `Expected ${value} to include ${fragment}`)
}

const managedReady = {
  envId: "practice",
  instanceId: "instance-1",
  packId: "hr-sql-101",
  packVersion: "1.0.0",
  packDigest: "a".repeat(64),
  namespace: "sj_practice",
  mode: "managed" as const,
  state: "ready" as const,
}

for (const action of ["reset", "delete"] as const) {
  const impact = describeEnvironmentImpact(action, managedReady)

  assertEqual(impact.environmentId, "practice")
  assertEqual(impact.namespace, "sj_practice")
  assertIncludes(impact.summary, "managed")
  assert(impact.details.length > 0, "expected impact details")
  assertEqual(isDestructiveActionAvailable(action, managedReady, false), true)
}

const unavailableStates = [
  "running",
  "rebuilding",
  "deleting",
  "error",
] as const
for (const state of unavailableStates) {
  assertEqual(
    isDestructiveActionAvailable(
      "delete" satisfies EnvironmentAction,
      {
        ...managedReady,
        state,
      },
      false
    ),
    false
  )
}

assertEqual(isDestructiveActionAvailable("reset", managedReady, true), false)
assertEqual(
  isDestructiveActionAvailable(
    "delete",
    {
      ...managedReady,
      mode: "external_explorer",
    },
    false
  ),
  false
)
