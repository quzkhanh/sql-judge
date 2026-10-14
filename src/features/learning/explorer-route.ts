import { hashParams, routeForHash } from "@/app-route-state"

export function explorerHref(profileId: string) {
  return `#explorer?${new URLSearchParams({ profile: profileId })}`
}

export function profileIdFromHash(hash: string): string | null {
  return routeForHash(hash) === "explorer" ? hashParams(hash).get("profile") : null
}
