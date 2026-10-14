import { Moon, Sun } from "lucide-react"

import { SidebarTrigger } from "@/components/ui/sidebar"
import { useResolvedTheme } from "@/hooks/use-resolved-theme"

const ghostIcon =
  "inline-flex size-8 items-center justify-center rounded-lg text-foreground outline-none hover:bg-hover focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-ring"

export function SiteHeader({ title }: { title: string }) {
  const { dark, toggle } = useResolvedTheme()
  return (
    <header className="flex h-14 shrink-0 items-center gap-3 border-b px-5">
      <SidebarTrigger className={ghostIcon} />
      <div className="h-4 w-px bg-border" aria-hidden="true" />
      <h1 className="text-sm font-medium">{title}</h1>
      <div className="flex-1" />
      <button type="button" className={ghostIcon} aria-label="Switch theme" onClick={toggle}>
        {dark ? <Sun className="size-4" aria-hidden="true" /> : <Moon className="size-4" aria-hidden="true" />}
      </button>
    </header>
  )
}
