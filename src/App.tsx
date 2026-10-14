import { useEffect, useState, type CSSProperties } from "react"

import { AppSidebar, type AppRoute } from "@/components/app-sidebar"
import { SiteHeader } from "@/components/site-header"
import { SidebarInset, SidebarProvider } from "@/components/ui/sidebar"
import { TooltipProvider } from "@/components/ui/tooltip"
import { ConnectionsScreen } from "@/features/learning/connections-screen"
import { EnvironmentsScreen } from "@/features/learning/environments-screen"
import { HistoryScreen } from "@/features/learning/history-screen"
import { LearningWorkspace } from "@/features/learning/learning-workspace"
import { PacksScreen } from "@/features/learning/packs-screen"
import { ProgressScreen } from "@/features/learning/progress-screen"

const routeTitles: Record<AppRoute, string> = {
  practice: "Practice",
  environments: "Environments",
  history: "History",
  progress: "Progress",
  packs: "Packs",
  connections: "Connections",
}

function readRoute(): AppRoute {
  const route = window.location.hash.slice(1) as AppRoute
  return route in routeTitles ? route : "practice"
}

export default function App() {
  const [route, setRoute] = useState<AppRoute>(() => readRoute())

  useEffect(() => {
    const handleHashChange = () => setRoute(readRoute())
    window.addEventListener("hashchange", handleHashChange)
    if (!window.location.hash) {
      window.history.replaceState(null, "", "#practice")
    }
    return () => window.removeEventListener("hashchange", handleHashChange)
  }, [])

  return (
    <TooltipProvider>
      <SidebarProvider
        style={
          {
            "--sidebar-width": "calc(var(--spacing) * 72)",
            "--header-height": "calc(var(--spacing) * 12)",
          } as CSSProperties
        }
      >
        <AppSidebar activeRoute={route} variant="inset" />
        <SidebarInset>
          <SiteHeader title={routeTitles[route]} />
          {route === "practice" ? <LearningWorkspace /> : null}
          {route === "environments" ? <EnvironmentsScreen /> : null}
          {route === "history" ? <HistoryScreen /> : null}
          {route === "progress" ? <ProgressScreen /> : null}
          {route === "packs" ? <PacksScreen /> : null}
          {route === "connections" ? <ConnectionsScreen /> : null}
        </SidebarInset>
      </SidebarProvider>
    </TooltipProvider>
  )
}
