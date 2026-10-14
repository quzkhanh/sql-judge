import { useEffect, useState } from "react"
import { History, RefreshCw } from "lucide-react"

import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"
import {
  listSubmissionHistory,
  type SubmissionHistorySummary,
} from "@/lib/judge-api"
import { cn } from "@/lib/utils"
import { BackupActions } from "./backup-actions"

export function HistoryScreen() {
  const [records, setRecords] = useState<SubmissionHistorySummary[]>([])
  const [busy, setBusy] = useState(true)
  const [message, setMessage] = useState<string | null>(null)

  async function refresh() {
    setBusy(true)
    setRecords([])
    try {
      setRecords(await listSubmissionHistory())
      setMessage(null)
    } catch {
      setMessage("Submission history could not be loaded.")
    } finally {
      setBusy(false)
    }
  }

  useEffect(() => {
    let active = true
    void listSubmissionHistory()
      .then((nextRecords) => {
        if (active) {
          setRecords(nextRecords)
          setMessage(null)
        }
      })
      .catch(() => {
        if (active) {
          setRecords([])
          setMessage("Submission history could not be loaded.")
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

  return (
    <main className="min-h-0 flex-1 overflow-y-auto p-4 lg:p-6">
      <div className="mx-auto flex max-w-6xl flex-col gap-6">
        <header className="flex flex-col gap-2 border-b pb-5 sm:flex-row sm:items-end sm:justify-between">
          <div>
            <p className="text-xs font-semibold tracking-[0.18em] text-primary uppercase">
              Audit trail
            </p>
            <h2 className="mt-1 text-2xl font-semibold tracking-tight">
              History
            </h2>
            <p className="mt-1 text-sm text-muted-foreground">
              Review local submission outcomes and immutable pack metadata.
            </p>
          </div>
          <div className="flex flex-wrap justify-end gap-2">
            <BackupActions onRestored={() => void refresh()} />
            <Button
              variant="outline"
              onClick={() => void refresh()}
              disabled={busy}
            >
              <RefreshCw className={cn(busy && "animate-spin")} />
              Refresh
            </Button>
          </div>
        </header>

        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2 text-base">
              <History className="size-4" /> Submissions
            </CardTitle>
            <CardDescription>
              Passwords, connection strings, and raw SQL are never stored in
              this history.
            </CardDescription>
          </CardHeader>
          <CardContent>
            {busy ? (
              <p
                className="rounded-lg border border-dashed p-12 text-center text-sm text-muted-foreground"
                role="status"
              >
                Loading submission history...
              </p>
            ) : message ? (
              <div
                className="rounded-lg border border-destructive/30 bg-destructive/5 p-8 text-center"
                role="alert"
              >
                <p className="text-sm text-destructive">{message}</p>
                <Button
                  className="mt-3"
                  variant="outline"
                  size="sm"
                  onClick={() => void refresh()}
                >
                  <RefreshCw />
                  Retry
                </Button>
              </div>
            ) : records.length > 0 ? (
              <div className="overflow-x-auto rounded-lg border">
                <table className="w-full text-left text-sm">
                  <thead className="bg-muted/40 text-xs text-muted-foreground">
                    <tr>
                      <th className="px-4 py-3 font-medium">Created</th>
                      <th className="px-4 py-3 font-medium">Pack</th>
                      <th className="px-4 py-3 font-medium">Verdict</th>
                      <th className="px-4 py-3 font-medium">Engine</th>
                      <th className="px-4 py-3 font-medium">Policy</th>
                    </tr>
                  </thead>
                  <tbody>
                    {records.map((record) => (
                      <tr key={record.submissionId} className="border-t">
                        <td className="px-4 py-3 whitespace-nowrap text-muted-foreground">
                          {record.createdAt}
                        </td>
                        <td className="px-4 py-3">
                          <p className="font-medium">{record.packId}</p>
                          <p className="text-xs text-muted-foreground">
                            v{record.packVersion}
                          </p>
                        </td>
                        <td className="px-4 py-3">
                          <Badge
                            variant={
                              record.verdict === "Accepted"
                                ? "default"
                                : "outline"
                            }
                          >
                            {record.verdict}
                          </Badge>
                        </td>
                        <td className="px-4 py-3 font-mono text-xs text-muted-foreground">
                          {record.engineVersion}
                        </td>
                        <td className="px-4 py-3 font-mono text-xs text-muted-foreground">
                          {record.policyVersion}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            ) : (
              <div className="rounded-lg border border-dashed p-12 text-center">
                <History className="mx-auto size-8 text-muted-foreground/50" />
                <p className="mt-3 text-sm font-medium">No submissions yet</p>
                <p className="mt-1 text-sm text-muted-foreground">
                  Run or submit a solution from Practice to build your local
                  history.
                </p>
              </div>
            )}
          </CardContent>
        </Card>
      </div>
    </main>
  )
}
