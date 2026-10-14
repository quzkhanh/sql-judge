import * as React from "react"
import {
  BarChart3,
  BookOpen,
  Cable,
  Database,
  FolderOpen,
  History,
  Home,
  LayoutDashboard,
  Search,
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
  | "home"
  | "practice"
  | "environments"
  | "history"
  | "progress"
  | "packs"
  | "connections"

type NavigationItem = {
  route: AppRoute
  label: string
  icon: typeof LayoutDashboard
}

const navigation: NavigationItem[] = [
  { route: "home", label: "Home", icon: Home },
  { route: "practice", label: "Practice", icon: LayoutDashboard },
  { route: "history", label: "History", icon: History },
  { route: "progress", label: "Progress", icon: BarChart3 },
]

const libraryNavigation: NavigationItem[] = [
  { route: "packs", label: "Packs", icon: FolderOpen },
]

const dataNavigation: NavigationItem[] = [
  { route: "environments", label: "Environments", icon: Database },
  { route: "connections", label: "Connections", icon: Cable },
]

export function AppSidebar({
  activeRoute,
  ...props
}: React.ComponentProps<typeof Sidebar> & { activeRoute: AppRoute }) {
  return (
    <Sidebar collapsible="offcanvas" {...props}>
      <SidebarHeader>
        <SidebarMenu className="mt-1">
          <SidebarMenuItem>
            <SidebarMenuButton
              className="h-auto! data-[slot=sidebar-menu-button]:p-1.5!"
              render={<a href="#home" />}
              isActive={activeRoute === "home"}
            >
              <span className="flex size-7 items-center justify-center rounded-lg bg-sidebar-primary font-mono text-xs font-medium text-sidebar-primary-foreground">SJ</span>
              <span className="flex flex-col leading-tight"><span className="text-base font-semibold">SQL Judge</span><span className="text-xs font-normal text-sidebar-foreground/60">Local workspace</span></span>
            </SidebarMenuButton>
          </SidebarMenuItem>
        </SidebarMenu>
      </SidebarHeader>
      <SidebarContent>
        <SidebarGroup className="pb-1">
          <SidebarGroupContent>
            <button
              type="button"
              className="flex h-[34px] w-full items-center justify-between rounded-lg border border-sidebar-border bg-sidebar px-3 text-sm text-sidebar-foreground/60 shadow-none hover:bg-sidebar-accent focus-visible:ring-2 focus-visible:ring-ring"
            >
              <span className="flex items-center gap-2"><Search className="size-4" /><span>Search</span></span>
              <kbd className="h-5 rounded-[5px] border border-sidebar-border bg-sidebar px-1.5 font-mono text-[11px] leading-5">Ctrl K</kbd>
            </button>
          </SidebarGroupContent>
        </SidebarGroup>
        <SidebarGroup>
          <SidebarGroupLabel>Learn</SidebarGroupLabel>
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
        <SidebarGroup>
          <SidebarGroupLabel>Library</SidebarGroupLabel>
          <SidebarGroupContent>
            <NavigationItems items={libraryNavigation} activeRoute={activeRoute} />
          </SidebarGroupContent>
        </SidebarGroup>
        <SidebarGroup>
          <SidebarGroupLabel>Data</SidebarGroupLabel>
          <SidebarGroupContent>
            <NavigationItems items={dataNavigation} activeRoute={activeRoute} />
          </SidebarGroupContent>
        </SidebarGroup>
        <SidebarGroup className="mt-auto">
          <SidebarGroupContent>
            <a href="#environments" className="block rounded-lg p-3 text-xs text-sidebar-foreground/70 hover:bg-sidebar-accent">
              <div className="flex items-center gap-2 font-medium text-sidebar-foreground">
                <BookOpen className="size-3.5" />
                Runtime ready
              </div>
              <p className="mt-1 leading-5">
                Local practice workspace
              </p>
            </a>
          </SidebarGroupContent>
        </SidebarGroup>
      </SidebarContent>
      <SidebarFooter>
        <div className="flex items-center gap-2 rounded-lg p-2 text-xs">
          <span className="size-2 rounded-full bg-emerald-500" />
          <div className="min-w-0">
            <p className="truncate font-medium">SQL learner</p>
            <p className="truncate text-sidebar-foreground/60">
              Local workspace
            </p>
          </div>
        </div>
      </SidebarFooter>
    </Sidebar>
  )
}

function NavigationItems({
  items,
  activeRoute,
}: {
  items: NavigationItem[]
  activeRoute: AppRoute
}) {
  return (
    <SidebarMenu>
      {items.map((item) => {
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
  )
}
