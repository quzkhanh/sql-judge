import { useEffect, useRef, useState } from "react"
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
import { AccessibleModal } from "@/components/ui/accessible-modal"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import {
  createManagedEnvironment,
  deleteManagedEnvironment,
  getManagedRuntimeStatus,
  listInstalledPacks,
  listManagedEnvironments,
  resetManagedEnvironment,
  startManagedRuntime,
  type ManagedEnvironmentSummary,
  type InstalledPack,
  type ManagedRuntimeStatus,
} from "@/lib/judge-api"
import { cn } from "@/lib/utils"
import {
  describeEnvironmentImpact,
  isDestructiveActionAvailable,
  type EnvironmentAction,
} from "./environment-impact"
import { learningLoadErrorMessage, learningLoadPhase } from "./load-state"

type PendingAction = {
  action: EnvironmentAction
  environment: ManagedEnvironmentSummary
  replacementNamespace?: string
}

function packKey(pack: InstalledPack): string {
  return `${pack.packId}@${pack.version}#${pack.digest}`
}

export function EnvironmentsScreen() {
  const [runtimeStatus, setRuntimeStatus] =
    useState<ManagedRuntimeStatus | null>(null)
  const [packs, setPacks] = useState<InstalledPack[]>([])
  const [selectedPackKey, setSelectedPackKey] = useState("")
  const [environments, setEnvironments] = useState<ManagedEnvironmentSummary[]>(
    []
  )
  const [namespace, setNamespace] = useState("sj_practice")
  const [busy, setBusy] = useState(true)
  const [message, setMessage] = useState<string | null>(null)
  const [environmentLoading, setEnvironmentLoading] = useState(true)
  const [environmentError, setEnvironmentError] = useState<string | null>(null)
  const [environmentReloadToken, setEnvironmentReloadToken] = useState(0)
  const [pendingAction, setPendingAction] = useState<PendingAction | null>(null)

  async function refresh(): Promise<boolean> {
    setEnvironmentLoading(true)
    setEnvironmentError(null)
    setBusy(true)
    try {
      const [status, records, installedPacks] = await Promise.all([
        getManagedRuntimeStatus(),
        listManagedEnvironments(),
        listInstalledPacks(),
      ])
      setRuntimeStatus(status)
      setEnvironments(records)
      setPacks(installedPacks)
      setSelectedPackKey(
        (current) =>
          current || (installedPacks[0] ? packKey(installedPacks[0]) : "")
      )
      setEnvironmentError(null)
      setMessage(null)
      return true
    } catch {
      setRuntimeStatus(null)
      setEnvironments([])
      setPacks([])
      setSelectedPackKey("")
      setPendingAction(null)
      setEnvironmentError(learningLoadErrorMessage("environments"))
      setMessage("Managed environments could not be loaded.")
      return false
    } finally {
      setEnvironmentLoading(false)
      setBusy(false)
    }
  }

  function retryEnvironmentLoad() {
    setEnvironmentLoading(true)
    setEnvironmentError(null)
    setBusy(true)
    setEnvironmentReloadToken((current) => current + 1)
  }

  useEffect(() => {
    let active = true
    void Promise.all([
      getManagedRuntimeStatus(),
      listManagedEnvironments(),
      listInstalledPacks(),
    ])
      .then(([status, records, installedPacks]) => {
        if (active) {
          setRuntimeStatus(status)
          setEnvironments(records)
          setPacks(installedPacks)
          setSelectedPackKey(
            (current) =>
              current || (installedPacks[0] ? packKey(installedPacks[0]) : "")
          )
          setEnvironmentError(null)
          setMessage(null)
        }
      })
      .catch(() => {
        if (active) {
          setRuntimeStatus(null)
          setEnvironments([])
          setPacks([])
          setSelectedPackKey("")
          setPendingAction(null)
          setEnvironmentError(learningLoadErrorMessage("environments"))
        }
      })
      .finally(() => {
        if (active) {
          setEnvironmentLoading(false)
          setBusy(false)
        }
      })
    return () => {
      active = false
    }
  }, [environmentReloadToken])

  const selectedPack =
    packs.find((pack) => packKey(pack) === selectedPackKey) ?? null
  const environmentPhase = learningLoadPhase(
    environmentLoading,
    environmentError,
    environments.length > 0
  )

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
    if (!selectedPack) {
      setMessage("Import the managed pack before creating an environment.")
      return
    }
    setBusy(true)
    try {
      await createManagedEnvironment({
        envId: "practice",
        packId: selectedPack.packId,
        packVersion: selectedPack.version,
        packDigest: selectedPack.digest,
        namespace: nextNamespace,
      })
      const refreshed = await refresh()
      if (!refreshed) {
        setMessage(
          "Practice environment was created, but the environment list could not be refreshed. Retry to verify its state."
        )
        return
      }
      setMessage("Practice environment is ready.")
    } catch {
      setMessage("Practice environment could not be provisioned.")
      setBusy(false)
    }
  }

  function requestReset(environment: ManagedEnvironmentSummary) {
    const nextNamespace = namespace.trim()
    if (!nextNamespace) {
      setMessage("Enter a managed schema namespace first.")
      return
    }
    setPendingAction({
      action: "reset",
      environment,
      replacementNamespace: nextNamespace,
    })
  }

  function requestDelete(environment: ManagedEnvironmentSummary) {
    setPendingAction({ action: "delete", environment })
  }

  async function confirmPendingAction() {
    if (!pendingAction) {
      return
    }
    setBusy(true)
    try {
      if (pendingAction.action === "reset") {
        await resetManagedEnvironment({
          envId: pendingAction.environment.envId,
          instanceId: pendingAction.environment.instanceId,
          namespace: pendingAction.replacementNamespace ?? namespace.trim(),
          confirmed: true,
        })
      } else {
        await deleteManagedEnvironment({
          envId: pendingAction.environment.envId,
          instanceId: pendingAction.environment.instanceId,
          confirmed: true,
        })
      }
      const refreshed = await refresh()
      if (!refreshed) {
        setMessage(
          pendingAction.action === "reset"
            ? "Environment rebuild completed, but the environment list could not be refreshed. Retry to verify its state."
            : "Environment deletion completed, but the environment list could not be refreshed. Retry to verify its state."
        )
        return
      }
      setMessage(
        pendingAction.action === "reset"
          ? "Environment rebuilt and old schema cleaned up."
          : "Environment deleted."
      )
    } catch {
      setMessage(
        pendingAction.action === "reset"
          ? "Environment rebuild failed closed."
          : "Environment deletion failed closed."
      )
      setBusy(false)
    } finally {
      setPendingAction(null)
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
          <Button
            variant="outline"
            onClick={() => void refresh()}
            disabled={busy}
          >
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
                The managed runner is local, socket-only, and isolated from
                external databases.
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
                    {runtimeStatus?.available
                      ? "Runtime ready"
                      : "Runtime unavailable"}
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
                Destructive actions require a managed ownership marker, matching
                instance identity, and explicit confirmation in the desktop app.
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
                  <Label htmlFor="environment-pack">Pack</Label>
                  <select
                    id="environment-pack"
                    className="h-9 w-full rounded-md border border-input bg-background px-3 text-sm"
                    value={selectedPackKey}
                    onChange={(event) => setSelectedPackKey(event.target.value)}
                    disabled={
                      busy ||
                      environmentLoading ||
                      environmentError !== null ||
                      packs.length === 0
                    }
                  >
                    {packs.length === 0 ? (
                      <option value="">No imported packs</option>
                    ) : null}
                    {packs.map((pack) => (
                      <option key={packKey(pack)} value={packKey(pack)}>
                        {pack.title} v{pack.version}
                      </option>
                    ))}
                  </select>
                </div>
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
                <Button
                  onClick={() => void createEnvironment()}
                  disabled={
                    busy || environmentLoading || environmentError !== null
                  }
                >
                  <Plus />
                  Create
                </Button>
              </div>
              <p className="text-xs text-muted-foreground">
                {selectedPack
                  ? `Pinned pack ${selectedPack.version} (${selectedPack.digest.slice(0, 12)}...)`
                  : "The managed pack is not installed."}
              </p>

              {pendingAction && environmentPhase === "ready" ? (
                <ImpactReview
                  pendingAction={pendingAction}
                  busy={busy}
                  onCancel={() => setPendingAction(null)}
                  onConfirm={() => void confirmPendingAction()}
                />
              ) : null}

              {environmentPhase === "loading" ? (
                <p
                  className="rounded-lg border border-dashed p-8 text-center text-sm text-muted-foreground"
                  role="status"
                >
                  Loading managed environments...
                </p>
              ) : environmentPhase === "error" ? (
                <div
                  className="rounded-lg border border-destructive/30 bg-destructive/5 p-8 text-center"
                  role="alert"
                >
                  <p className="text-sm text-destructive">{environmentError}</p>
                  <Button
                    className="mt-3"
                    variant="outline"
                    size="sm"
                    onClick={retryEnvironmentLoad}
                  >
                    <RefreshCw />
                    Retry
                  </Button>
                </div>
              ) : environmentPhase === "ready" ? (
                <div className="grid gap-3 md:grid-cols-2">
                  {environments.map((environment) => (
                    <div
                      key={environment.envId}
                      className="rounded-lg border p-3"
                    >
                      <div className="flex items-start justify-between gap-3">
                        <div className="min-w-0">
                          <p className="truncate font-mono text-sm font-semibold">
                            {environment.namespace}
                          </p>
                          <p className="mt-1 text-xs text-muted-foreground">
                            {environment.envId} · {environment.packId}
                          </p>
                        </div>
                        <Badge
                          variant={
                            environment.state === "ready"
                              ? "secondary"
                              : "outline"
                          }
                        >
                          {environment.state}
                        </Badge>
                      </div>
                      <div className="mt-4 flex justify-end gap-2">
                        <Button
                          variant="outline"
                          size="sm"
                          onClick={() => requestReset(environment)}
                          disabled={
                            environmentPhase !== "ready" ||
                            !isDestructiveActionAvailable(
                              "reset",
                              environment,
                              busy
                            )
                          }
                        >
                          <RotateCcw />
                          Reset
                        </Button>
                        <Button
                          variant="ghost"
                          size="icon"
                          onClick={() => requestDelete(environment)}
                          disabled={
                            environmentPhase !== "ready" ||
                            !isDestructiveActionAvailable(
                              "delete",
                              environment,
                              busy
                            )
                          }
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

function ImpactReview({
  pendingAction,
  busy,
  onCancel,
  onConfirm,
}: {
  pendingAction: PendingAction
  busy: boolean
  onCancel: () => void
  onConfirm: () => void
}) {
  const cancelRef = useRef<HTMLButtonElement>(null)
  const impact = describeEnvironmentImpact(
    pendingAction.action,
    pendingAction.environment
  )

  return (
    <AccessibleModal
      open
      title="Review environment impact"
      description={impact.summary}
      busy={busy}
      initialFocusRef={cancelRef}
      onCancel={onCancel}
    >
      <dl className="mt-3 grid gap-2 text-xs sm:grid-cols-2">
        <div>
          <dt className="text-muted-foreground">Environment</dt>
          <dd className="font-mono font-medium">{impact.environmentId}</dd>
        </div>
        <div>
          <dt className="text-muted-foreground">Managed namespace</dt>
          <dd className="font-mono font-medium">
            {pendingAction.action === "reset"
              ? `${impact.namespace} -> ${pendingAction.replacementNamespace}`
              : impact.namespace}
          </dd>
        </div>
      </dl>
      <ul className="mt-3 list-disc space-y-1 pl-4 text-xs leading-5 text-muted-foreground">
        {impact.details.map((detail) => (
          <li key={detail}>{detail}</li>
        ))}
      </ul>
      <div className="mt-4 flex justify-end gap-2">
        <Button
          ref={cancelRef}
          variant="outline"
          size="sm"
          onClick={onCancel}
          disabled={busy}
        >
          Cancel
        </Button>
        <Button
          variant="destructive"
          size="sm"
          onClick={onConfirm}
          disabled={busy}
        >
          {pendingAction.action === "reset"
            ? "Confirm rebuild"
            : "Confirm delete"}
        </Button>
      </div>
    </AccessibleModal>
  )
}
