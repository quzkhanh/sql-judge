import * as React from "react"
import {
  BarChart3,
  BookOpen,
  Cable,
  ChartNoAxesColumn,
  Command,
  Database,
  FolderOpen,
  History,
  LayoutDashboard,
} from "lucide-react"

import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarGroup,
  SidebarGroupContent,
  SidebarGroupLabel,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
} from "@/components/ui/sidebar"

export type AppRoute =
  | "practice"
  | "environments"
  | "history"
  | "progress"
  | "packs"
  | "connections"

const navigation: Array<{
  route: AppRoute
  label: string
  icon: typeof LayoutDashboard
}> = [
  { route: "practice", label: "Practice", icon: LayoutDashboard },
  { route: "environments", label: "Environments", icon: Database },
  { route: "history", label: "History", icon: History },
  { route: "progress", label: "Progress", icon: BarChart3 },
  { route: "packs", label: "Packs", icon: FolderOpen },
  { route: "connections", label: "Connections", icon: Cable },
]

export function AppSidebar({
  activeRoute,
  ...props
}: React.ComponentProps<typeof Sidebar> & { activeRoute: AppRoute }) {
  return (
    <Sidebar collapsible="offcanvas" {...props}>
      <SidebarHeader>
        <SidebarMenu>
          <SidebarMenuItem>
            <SidebarMenuButton
              className="data-[slot=sidebar-menu-button]:p-1.5!"
              render={<a href="#practice" />}
              isActive={activeRoute === "practice"}
            >
              <Command className="size-5!" />
              <span className="text-base font-semibold">SQL Judge</span>
            </SidebarMenuButton>
          </SidebarMenuItem>
        </SidebarMenu>
      </SidebarHeader>
      <SidebarContent>
        <SidebarGroup>
          <SidebarGroupLabel>Workspace</SidebarGroupLabel>
          <SidebarGroupContent>
            <SidebarMenu>
              {navigation.map((item) => {
                const Icon = item.icon
                return (
                  <SidebarMenuItem key={item.route}>
                    <SidebarMenuButton
                      render={<a href={`#${item.route}`} />}
                      isActive={activeRoute === item.route}
                      tooltip={item.label}
                    >
                      <Icon />
                      <span>{item.label}</span>
                    </SidebarMenuButton>
                  </SidebarMenuItem>
                )
              })}
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>
        <SidebarGroup className="mt-auto">
          <SidebarGroupLabel>Local workspace</SidebarGroupLabel>
          <SidebarGroupContent>
            <div className="rounded-lg border border-sidebar-border/70 bg-sidebar-accent/40 p-3 text-xs text-sidebar-foreground/70">
              <div className="flex items-center gap-2 font-medium text-sidebar-foreground">
                <BookOpen className="size-3.5" />
                Offline-first
              </div>
              <p className="mt-1 leading-5">
                Practice content and managed PostgreSQL stay on this device.
              </p>
            </div>
          </SidebarGroupContent>
        </SidebarGroup>
      </SidebarContent>
      <SidebarFooter>
        <div className="flex items-center gap-2 rounded-lg border border-sidebar-border/70 p-2 text-xs">
          <ChartNoAxesColumn className="size-4 text-sidebar-foreground/60" />
          <div className="min-w-0">
            <p className="truncate font-medium">SQL learner</p>
            <p className="truncate text-sidebar-foreground/60">Local workspace</p>
          </div>
        </div>
      </SidebarFooter>
    </Sidebar>
  )
}
