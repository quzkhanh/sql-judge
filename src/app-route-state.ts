import type { AppRoute } from "@/components/app-sidebar"

const directRoutes: AppRoute[] = [
  "home",
  "problems",
  "environments",
  "history",
  "progress",
  "packs",
  "connections",
]

export function routeForLaunch(hash = ""): AppRoute {
  return routeForHash(hash)
}

export function routeForHash(hash: string): AppRoute {
  const route = hash.slice(1)
  if ((directRoutes as string[]).includes(route)) {
    return route as AppRoute
  }
  return route.startsWith("solve/") && route.length > "solve/".length
    ? "solve"
    : "home"
}
