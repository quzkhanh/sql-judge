import type { AppRoute } from "@/components/app-sidebar"

const directRoutes: AppRoute[] = [
  "home",
  "problems",
  "environments",
  "history",
  "progress",
  "packs",
  "connections",
  "explorer",
  "settings",
]

export function routeForLaunch(hash = ""): AppRoute {
  return routeForHash(hash)
}

export function routeForHash(hash: string): AppRoute {
  const route = hash.slice(1).split("?", 1)[0]
  if ((directRoutes as string[]).includes(route)) {
    return route as AppRoute
  }
  return route.startsWith("solve/") && route.length > "solve/".length
    ? "solve"
    : "home"
}

export function hashParams(hash: string) {
  const query = hash.indexOf("?")
  return new URLSearchParams(query < 0 ? "" : hash.slice(query + 1))
}

export function problemHref(problemKey: string) {
  return `#solve/${encodeURIComponent(problemKey)}`
}
