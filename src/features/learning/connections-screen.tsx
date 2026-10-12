import { useEffect, useState } from "react"
import { Cable, Check, LockKeyhole, Play, ShieldCheck } from "lucide-react"

import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import {
  getManagedRuntimeStatus,
  startManagedRuntime,
  type ManagedRuntimeStatus,
} from "@/lib/judge-api"

export function ConnectionsScreen() {
  const [status, setStatus] = useState<ManagedRuntimeStatus | null>(null)
  const [busy, setBusy] = useState(false)

  useEffect(() => {
    void getManagedRuntimeStatus().then(setStatus).catch(() => setStatus(null))
  }, [])

  async function start() {
    setBusy(true)
    try {
      setStatus(await startManagedRuntime())
    } catch {
      setStatus({
        available: false,
        state: "unavailable",
        message: "The managed PostgreSQL runtime could not be started.",
      })
    } finally {
      setBusy(false)
    }
  }

  return (
    <main className="min-h-0 flex-1 overflow-y-auto p-4 lg:p-6">
      <div className="mx-auto flex max-w-6xl flex-col gap-6">
        <header className="border-b pb-5">
          <p className="text-xs font-semibold tracking-[0.18em] text-primary uppercase">
            Infrastructure
          </p>
          <h2 className="mt-1 text-2xl font-semibold tracking-tight">Connections</h2>
          <p className="mt-1 text-sm text-muted-foreground">
            Inspect the local managed runner. External connections are intentionally not enabled
            in the judge workspace.
          </p>
        </header>

        <div className="grid gap-4 lg:grid-cols-[minmax(0,1.2fr)_minmax(260px,0.8fr)]">
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2 text-base">
                <Cable className="size-4" /> Managed PostgreSQL
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-5">
              <div className="flex items-center justify-between gap-4 rounded-lg border p-4">
                <div className="flex items-center gap-3">
                  <span
                    className={`size-3 rounded-full ${status?.available ? "bg-emerald-500" : "bg-amber-500"}`}
                    aria-hidden="true"
                  />
                  <div>
                    <p className="font-medium">
                      {status?.available ? "Connected and ready" : "Not connected"}
                    </p>
                    <p className="text-sm text-muted-foreground">
                      {status?.message ?? "Checking local runtime status..."}
                    </p>
                  </div>
                </div>
                <Badge variant={status?.available ? "default" : "outline"}>
                  {status?.state ?? "checking"}
                </Badge>
              </div>
              {status && !status.available ? (
                <Button onClick={() => void start()} disabled={busy}>
                  <Play />
                  Start runtime
                </Button>
              ) : null}
              <div className="grid gap-3 sm:grid-cols-2">
                <Capability icon={Check} label="Local Unix socket" />
                <Capability icon={ShieldCheck} label="Ownership verified" />
                <Capability icon={LockKeyhole} label="Runner role constrained" />
                <Capability icon={Check} label="Statement limits enabled" />
              </div>
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle className="text-base">Connection policy</CardTitle>
            </CardHeader>
            <CardContent className="space-y-3 text-sm leading-6 text-muted-foreground">
              <p>
                SQL Judge never uses the metadata database as a practice backend and never grants
                judge permissions to arbitrary external servers.
              </p>
              <p>
                External explorer connections will be read-only and credential-vault backed when
                that feature is enabled.
              </p>
            </CardContent>
          </Card>
        </div>
      </div>
    </main>
  )
}

function Capability({
  icon: Icon,
  label,
}: {
  icon: typeof Check
  label: string
}) {
  return (
    <div className="flex items-center gap-2 rounded-lg bg-muted/40 p-3 text-sm">
      <Icon className="size-4 text-emerald-600" />
      {label}
    </div>
  )
}
