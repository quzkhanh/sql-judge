import { useEffect, useState } from "react"
import { Database, Plus, RefreshCw, RotateCcw, Trash2 } from "lucide-react"

import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import {
  createManagedEnvironment,
  deleteManagedEnvironment,
  getManagedRuntimeStatus,
  listManagedEnvironments,
  resetManagedEnvironment,
  startManagedRuntime,
  type ManagedEnvironmentSummary,
  type ManagedRuntimeStatus,
} from "@/lib/judge-api"
import { cn } from "@/lib/utils"

export function EnvironmentsScreen() {
  const [runtimeStatus, setRuntimeStatus] =
    useState<ManagedRuntimeStatus | null>(null)
  const [environments, setEnvironments] = useState<ManagedEnvironmentSummary[]>(
    []
  )
  const [namespace, setNamespace] = useState("sj_practice")
  const [busy, setBusy] = useState(true)
  const [message, setMessage] = useState<string | null>(null)

  async function refresh() {
    setBusy(true)
    try {
      const [status, records] = await Promise.all([
        getManagedRuntimeStatus(),
        listManagedEnvironments(),
      ])
      setRuntimeStatus(status)
      setEnvironments(records)
      setMessage(null)
    } catch {
      setMessage("Managed environments could not be loaded.")
    } finally {
      setBusy(false)
    }
  }

  useEffect(() => {
    let active = true
    void Promise.all([getManagedRuntimeStatus(), listManagedEnvironments()])
      .then(([status, records]) => {
        if (active) {
          setRuntimeStatus(status)
          setEnvironments(records)
          setMessage(null)
        }
      })
      .catch(() => {
        if (active) {
          setMessage("Managed environments could not be loaded.")
        }
      })
      .finally(() => {
        if (active) {
          setBusy(false)
        }
      })
    return () => {
      active = false
    }
  }, [])

  async function startRuntime() {
    setBusy(true)
    try {
      setRuntimeStatus(await startManagedRuntime())
      setMessage("Managed PostgreSQL is ready.")
    } catch {
      setMessage("The managed PostgreSQL runtime could not be started.")
    } finally {
      setBusy(false)
    }
  }

  async function createEnvironment() {
    const nextNamespace = namespace.trim()
    if (!nextNamespace) {
      setMessage("Enter a managed schema namespace first.")
      return
    }
    setBusy(true)
    try {
      await createManagedEnvironment({
        envId: "practice",
        packId: "hr-sql-101",
        namespace: nextNamespace,
      })
      await refresh()
      setMessage("Practice environment is ready.")
    } catch {
      setMessage("Practice environment could not be provisioned.")
      setBusy(false)
    }
  }

  async function resetEnvironment(environment: ManagedEnvironmentSummary) {
    const nextNamespace = namespace.trim()
    if (
      !nextNamespace ||
      !window.confirm(`Replace ${environment.namespace} with ${nextNamespace}?`)
    ) {
      return
    }
    setBusy(true)
    try {
      await resetManagedEnvironment({
        envId: environment.envId,
        instanceId: environment.instanceId,
        namespace: nextNamespace,
        confirmed: true,
      })
      await refresh()
      setMessage("Environment rebuilt and old schema cleaned up.")
    } catch {
      setMessage("Environment rebuild failed closed.")
      setBusy(false)
    }
  }

  async function deleteEnvironment(environment: ManagedEnvironmentSummary) {
    if (
      !window.confirm(
        `Delete ${environment.envId} and schema ${environment.namespace}? This cannot be undone.`
      )
    ) {
      return
    }
    setBusy(true)
    try {
      await deleteManagedEnvironment({
        envId: environment.envId,
        instanceId: environment.instanceId,
        confirmed: true,
      })
      await refresh()
      setMessage("Environment deleted.")
    } catch {
      setMessage("Environment deletion failed closed.")
      setBusy(false)
    }
  }

  return (
    <main className="min-h-0 flex-1 overflow-y-auto p-4 lg:p-6">
      <div className="mx-auto flex max-w-6xl flex-col gap-6">
        <header className="flex flex-col gap-2 border-b pb-5 sm:flex-row sm:items-end sm:justify-between">
          <div>
            <p className="text-xs font-semibold tracking-[0.18em] text-primary uppercase">
              Data plane
            </p>
            <h2 className="mt-1 text-2xl font-semibold tracking-tight">
              Environments
            </h2>
            <p className="mt-1 max-w-2xl text-sm text-muted-foreground">
              Create isolated, app-owned PostgreSQL schemas for practice packs.
            </p>
          </div>
          <Button variant="outline" onClick={() => void refresh()} disabled={busy}>
            <RefreshCw className={cn(busy && "animate-spin")} />
            Refresh
          </Button>
        </header>

        <div className="grid gap-4 lg:grid-cols-[minmax(260px,0.8fr)_minmax(0,1.6fr)]">
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2 text-base">
                <Database className="size-4" /> Runtime
              </CardTitle>
              <CardDescription>
                The managed runner is local, socket-only, and isolated from external databases.
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="flex items-center gap-3 rounded-lg border p-3">
                <span
                  className={cn(
                    "size-2.5 rounded-full",
                    runtimeStatus?.available ? "bg-emerald-500" : "bg-amber-500"
                  )}
                  aria-hidden="true"
                />
                <div>
                  <p className="text-sm font-medium">
                    {runtimeStatus?.available ? "Runtime ready" : "Runtime unavailable"}
                  </p>
                  <p className="text-xs text-muted-foreground">
                    {runtimeStatus?.message ?? "Checking managed PostgreSQL..."}
                  </p>
                </div>
              </div>
              {runtimeStatus && !runtimeStatus.available ? (
                <Button onClick={() => void startRuntime()} disabled={busy}>
                  Start managed runtime
                </Button>
              ) : null}
              <div className="rounded-lg bg-muted/40 p-3 text-xs leading-5 text-muted-foreground">
                Destructive actions require a managed ownership marker, matching instance identity,
                and explicit confirmation in the desktop app.
              </div>
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle className="text-base">Managed schemas</CardTitle>
              <CardDescription>
                Only schemas owned by this SQL Judge runtime appear here.
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="flex flex-col gap-2 sm:flex-row sm:items-end">
                <div className="flex-1 space-y-1.5">
                  <Label htmlFor="environment-namespace">New namespace</Label>
                  <Input
                    id="environment-namespace"
                    value={namespace}
                    onChange={(event) => setNamespace(event.target.value)}
                    placeholder="sj_practice"
                    className="font-mono"
                  />
                </div>
                <Button onClick={() => void createEnvironment()} disabled={busy}>
                  <Plus />
                  Create
                </Button>
              </div>

              {environments.length > 0 ? (
                <div className="grid gap-3 md:grid-cols-2">
                  {environments.map((environment) => (
                    <div key={environment.envId} className="rounded-lg border p-3">
                      <div className="flex items-start justify-between gap-3">
                        <div className="min-w-0">
                          <p className="truncate font-mono text-sm font-semibold">
                            {environment.namespace}
                          </p>
                          <p className="mt-1 text-xs text-muted-foreground">
                            {environment.envId} · {environment.packId}
                          </p>
                        </div>
                        <Badge variant={environment.state === "ready" ? "secondary" : "outline"}>
                          {environment.state}
                        </Badge>
                      </div>
                      <div className="mt-4 flex justify-end gap-2">
                        <Button
                          variant="outline"
                          size="sm"
                          onClick={() => void resetEnvironment(environment)}
                          disabled={busy || environment.state !== "ready"}
                        >
                          <RotateCcw />
                          Reset
                        </Button>
                        <Button
                          variant="ghost"
                          size="icon"
                          onClick={() => void deleteEnvironment(environment)}
                          disabled={busy || environment.state !== "ready"}
                          aria-label={`Delete ${environment.namespace}`}
                        >
                          <Trash2 />
                        </Button>
                      </div>
                    </div>
                  ))}
                </div>
              ) : (
                <div className="rounded-lg border border-dashed p-8 text-center text-sm text-muted-foreground">
                  No managed environment yet.
                </div>
              )}
              {message ? (
                <p className="text-sm text-muted-foreground" role="status">
                  {message}
                </p>
              ) : null}
            </CardContent>
          </Card>
        </div>
      </div>
    </main>
  )
}
