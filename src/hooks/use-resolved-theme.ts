import { useEffect, useState } from "react"

import { useTheme } from "@/components/theme-provider"

const DARK_QUERY = "(prefers-color-scheme: dark)"

export function useResolvedTheme() {
  const { theme, setTheme } = useTheme()
  const [systemDark, setSystemDark] = useState(
    () => window.matchMedia(DARK_QUERY).matches
  )
  useEffect(() => {
    const query = window.matchMedia(DARK_QUERY)
    const update = () => setSystemDark(query.matches)
    query.addEventListener("change", update)
    return () => query.removeEventListener("change", update)
  }, [])
  const dark = theme === "dark" || (theme === "system" && systemDark)
  return { dark, toggle: () => setTheme(dark ? "light" : "dark") }
}
