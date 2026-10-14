import { useEffect, useMemo, useState } from "react"
import { Award, BarChart3, CheckCircle2, RefreshCw, Target } from "lucide-react"

import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import {
  listSubmissionHistory,
  type SubmissionHistorySummary,
} from "@/lib/judge-api"
import { BackupActions } from "./backup-actions"
import { learningLoadErrorMessage } from "./load-state"

export function ProgressScreen() {
  const [records, setRecords] = useState<SubmissionHistorySummary[]>([])
  const [historyLoading, setHistoryLoading] = useState(true)
  const [historyError, setHistoryError] = useState<string | null>(null)
  const [historyReloadToken, setHistoryReloadToken] = useState(0)

  function retryHistoryLoad() {
    setHistoryLoading(true)
    setHistoryError(null)
    setHistoryReloadToken((current) => current + 1)
  }

  useEffect(() => {
    let active = true

    void listSubmissionHistory()
      .then((nextRecords) => {
        if (active) {
          setRecords(nextRecords)
        }
      })
      .catch(() => {
        if (active) {
          setRecords([])
          setHistoryError(learningLoadErrorMessage("history"))
        }
      })
      .finally(() => {
        if (active) {
          setHistoryLoading(false)
        }
      })

    return () => {
      active = false
    }
  }, [historyReloadToken])

  const stats = useMemo(() => {
    const accepted = records.filter(
      (record) => record.verdict === "Accepted"
    ).length
    const total = records.length
    return {
      accepted,
      total,
      rate: total === 0 ? 0 : Math.round((accepted / total) * 100),
    }
  }, [records])

  return (
    <main className="min-h-0 flex-1 overflow-y-auto p-4 lg:p-6">
      <div className="mx-auto flex max-w-6xl flex-col gap-6">
        <header className="border-b pb-5">
          <p className="text-xs font-semibold tracking-[0.18em] text-primary uppercase">
            Learning loop
          </p>
          <h2 className="mt-1 text-2xl font-semibold tracking-tight">
            Progress
          </h2>
          <div className="flex flex-col gap-3 sm:flex-row sm:items-end sm:justify-between">
            <p className="mt-1 text-sm text-muted-foreground">
              A lightweight view of your local judging activity. No cloud
              account required.
            </p>
            <BackupActions onRestored={retryHistoryLoad} />
          </div>
        </header>

        <div className="grid gap-4 sm:grid-cols-3">
          <StatCard icon={BarChart3} label="Submissions" value={stats.total} />
          <StatCard
            icon={CheckCircle2}
            label="Accepted"
            value={stats.accepted}
          />
          <StatCard
            icon={Target}
            label="Success rate"
            value={`${stats.rate}%`}
          />
        </div>

        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2 text-base">
              <Award className="size-4" /> Recent outcomes
            </CardTitle>
          </CardHeader>
          <CardContent>
            {historyLoading ? (
              <p
                className="rounded-lg border border-dashed p-8 text-center text-sm text-muted-foreground"
                role="status"
              >
                Loading local progress...
              </p>
            ) : historyError ? (
              <div
                className="rounded-lg border border-destructive/30 bg-destructive/5 p-8 text-center"
                role="alert"
              >
                <p className="text-sm text-destructive">{historyError}</p>
                <Button
                  className="mt-3"
                  variant="outline"
                  size="sm"
                  onClick={retryHistoryLoad}
                >
                  <RefreshCw />
                  Retry
                </Button>
              </div>
            ) : records.length > 0 ? (
              <div className="grid gap-2 sm:grid-cols-2 lg:grid-cols-3">
                {records.slice(0, 9).map((record) => (
                  <div
                    key={record.submissionId}
                    className="rounded-lg border p-3"
                  >
                    <div className="flex items-center justify-between gap-2">
                      <span className="truncate text-sm font-medium">
                        {record.packId}
                      </span>
                      <Badge
                        variant={
                          record.verdict === "Accepted" ? "default" : "outline"
                        }
                      >
                        {record.verdict}
                      </Badge>
                    </div>
                    <p className="mt-2 text-xs text-muted-foreground">
                      {record.createdAt}
                    </p>
                  </div>
                ))}
              </div>
            ) : (
              <p className="rounded-lg border border-dashed p-8 text-center text-sm text-muted-foreground">
                Complete a practice submission to see progress here.
              </p>
            )}
          </CardContent>
        </Card>
      </div>
    </main>
  )
}

function StatCard({
  icon: Icon,
  label,
  value,
}: {
  icon: typeof BarChart3
  label: string
  value: number | string
}) {
  return (
    <Card>
      <CardContent className="flex items-center gap-3 p-5">
        <div className="rounded-lg bg-primary/10 p-2.5 text-primary">
          <Icon className="size-5" />
        </div>
        <div>
          <p className="text-2xl font-semibold tracking-tight">{value}</p>
          <p className="text-xs text-muted-foreground">{label}</p>
        </div>
      </CardContent>
    </Card>
  )
}
