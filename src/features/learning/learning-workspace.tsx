import { useEffect, useMemo, useState } from "react"
import {
  CheckCircle2,
  Database,
  Filter,
  Play,
  RefreshCw,
  Send,
  Table2,
  Trash2,
  XCircle,
} from "lucide-react"

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
import { cn } from "@/lib/utils"
import {
  getManagedRuntimeStatus,
  cancelJudge,
  createManagedEnvironment,
  deleteManagedEnvironment,
  listManagedEnvironments,
  listSubmissionHistory,
  resetManagedEnvironment,
  runJudge,
  startManagedRuntime,
  type JudgeRunResult,
  type ManagedEnvironmentSummary,
  type ManagedRuntimeStatus,
  type SubmissionHistorySummary,
} from "@/lib/judge-api"
import {
  filterProblems,
  learningProblems,
  type LearningProblem,
  type ProblemDifficulty,
  type ProblemDialect,
} from "./catalog"

const difficultyLabels: Record<ProblemDifficulty, string> = {
  easy: "Easy",
  medium: "Medium",
  hard: "Hard",
}

export function LearningWorkspace() {
  const [query, setQuery] = useState("")
  const [dialect, setDialect] = useState<ProblemDialect | "all">("all")
  const [difficulty, setDifficulty] = useState<ProblemDifficulty | "all">("all")
  const [selectedId, setSelectedId] = useState(learningProblems[0].id)
  const [drafts, setDrafts] = useState<Record<string, string>>({})
  const [result, setResult] = useState<JudgeRunResult | null>(null)
  const [busy, setBusy] = useState(false)
  const [activeRequestId, setActiveRequestId] = useState<string | null>(null)
  const [runtimeStatus, setRuntimeStatus] =
    useState<ManagedRuntimeStatus | null>(null)
  const [runtimeBusy, setRuntimeBusy] = useState(false)
  const [environments, setEnvironments] = useState<ManagedEnvironmentSummary[]>(
    []
  )
  const [environmentNamespace, setEnvironmentNamespace] =
    useState("sj_practice")
  const [environmentBusy, setEnvironmentBusy] = useState(false)
  const [environmentMessage, setEnvironmentMessage] = useState<string | null>(
    null
  )
  const [history, setHistory] = useState<SubmissionHistorySummary[]>([])

  useEffect(() => {
    let active = true

    void getManagedRuntimeStatus()
      .then((status) => {
        if (active) {
          setRuntimeStatus(status)
        }
      })
      .catch(() => {
        if (active) {
          setRuntimeStatus({
            available: false,
            state: "unavailable",
            message: "The managed PostgreSQL runtime status could not be read.",
          })
        }
      })

    return () => {
      active = false
    }
  }, [])

  useEffect(() => {
    let active = true

    void listSubmissionHistory()
      .then((records) => {
        if (active) {
          setHistory(records)
        }
      })
      .catch(() => undefined)

    return () => {
      active = false
    }
  }, [])

  useEffect(() => {
    let active = true

    void listManagedEnvironments()
      .then((records) => {
        if (active) {
          setEnvironments(records)
        }
      })
      .catch(() => {
        if (active) {
          setEnvironmentMessage("Managed environments could not be loaded.")
        }
      })

    return () => {
      active = false
    }
  }, [])

  const filteredProblems = useMemo(
    () => filterProblems(learningProblems, query, dialect, difficulty),
    [difficulty, dialect, query]
  )
  const selectedProblem =
    learningProblems.find((problem) => problem.id === selectedId) ??
    filteredProblems[0] ??
    learningProblems[0]
  const draft = drafts[selectedProblem.id] ?? selectedProblem.starterSql

  async function run(mode: "run" | "submit") {
    const requestId = crypto.randomUUID()
    setBusy(true)
    setActiveRequestId(requestId)
    setResult(null)
    try {
      const nextResult = await runJudge({
        problemId: selectedProblem.id,
        sql: draft,
        mode,
        requestId,
      })
      setResult(nextResult)
      if (mode === "submit") {
        setHistory(await listSubmissionHistory())
      }
    } catch {
      setResult({
        status: "error",
        durationMs: 0,
        message:
          "The native judge returned an unavailable or malformed response.",
      })
    } finally {
      setBusy(false)
      setActiveRequestId(null)
    }
  }

  async function cancelActiveJudge() {
    if (activeRequestId) {
      await cancelJudge(activeRequestId).catch(() => undefined)
    }
  }

  async function startRuntime() {
    setRuntimeBusy(true)
    try {
      setRuntimeStatus(await startManagedRuntime())
    } catch {
      setRuntimeStatus({
        available: false,
        state: "unavailable",
        message: "The managed PostgreSQL runtime could not be started.",
      })
    } finally {
      setRuntimeBusy(false)
    }
  }

  async function refreshEnvironments() {
    setEnvironmentBusy(true)
    try {
      setEnvironments(await listManagedEnvironments())
      setEnvironmentMessage(null)
    } catch {
      setEnvironmentMessage("Managed environments could not be loaded.")
    } finally {
      setEnvironmentBusy(false)
    }
  }

  async function createPracticeEnvironment() {
    setEnvironmentBusy(true)
    try {
      await createManagedEnvironment({
        envId: "practice",
        packId: "hr-sql-101",
        namespace: environmentNamespace,
      })
      await refreshEnvironments()
      setEnvironmentMessage("Practice environment is ready.")
    } catch {
      setEnvironmentMessage("Practice environment could not be provisioned.")
    } finally {
      setEnvironmentBusy(false)
    }
  }

  async function resetEnvironment(environment: ManagedEnvironmentSummary) {
    const namespace = environmentNamespace.trim()
    if (
      !namespace ||
      !window.confirm(`Replace ${environment.namespace} with ${namespace}?`)
    ) {
      return
    }
    setEnvironmentBusy(true)
    try {
      await resetManagedEnvironment({
        envId: environment.envId,
        instanceId: environment.instanceId,
        namespace,
        confirmed: true,
      })
      await refreshEnvironments()
      setEnvironmentMessage("Environment rebuilt and old schema cleaned up.")
    } catch {
      setEnvironmentMessage("Environment rebuild failed closed.")
    } finally {
      setEnvironmentBusy(false)
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
    setEnvironmentBusy(true)
    try {
      await deleteManagedEnvironment({
        envId: environment.envId,
        instanceId: environment.instanceId,
        confirmed: true,
      })
      await refreshEnvironments()
      setEnvironmentMessage("Environment deleted.")
    } catch {
      setEnvironmentMessage("Environment deletion failed closed.")
    } finally {
      setEnvironmentBusy(false)
    }
  }

  return (
    <div className="flex min-h-0 flex-1 flex-col gap-4 p-4 lg:p-6">
      <header className="flex flex-col gap-3 border-b pb-4 lg:flex-row lg:items-end lg:justify-between">
        <div>
          <p className="text-xs font-semibold tracking-[0.18em] text-primary uppercase">
            Practice room
          </p>
          <h2 className="mt-1 text-2xl font-semibold tracking-tight">
            SQL Judge
          </h2>
          <p className="mt-1 max-w-2xl text-sm text-muted-foreground">
            Work against clean fixtures, inspect the schema, then submit a typed
            result for judging.
          </p>
        </div>
        <div className="flex flex-wrap items-center justify-end gap-2 text-xs text-muted-foreground">
          <span
            className={cn(
              "size-2 rounded-full",
              runtimeStatus?.available ? "bg-emerald-500" : "bg-amber-500"
            )}
            aria-hidden="true"
          />
          <span>
            {runtimeStatus?.available
              ? "Native runner ready"
              : (runtimeStatus?.message ?? "Checking native runner...")}
          </span>
          {runtimeStatus && !runtimeStatus.available ? (
            <Button
              variant="outline"
              size="sm"
              onClick={() => void startRuntime()}
              disabled={runtimeBusy}
            >
              {runtimeBusy ? "Starting..." : "Start runtime"}
            </Button>
          ) : null}
        </div>
      </header>

      <div className="grid min-h-0 flex-1 gap-4 xl:grid-cols-[280px_minmax(0,1fr)_300px]">
        <Card className="min-h-0 xl:flex">
          <CardHeader className="gap-3 border-b">
            <div className="flex items-center justify-between">
              <CardTitle className="text-sm">Problems</CardTitle>
              <Badge variant="secondary">{filteredProblems.length}</Badge>
            </div>
            <Label className="sr-only" htmlFor="problem-search">
              Search problems
            </Label>
            <Input
              id="problem-search"
              value={query}
              onChange={(event) => setQuery(event.target.value)}
              placeholder="Search topics or title"
              aria-describedby="problem-filter-help"
            />
            <p
              id="problem-filter-help"
              className="text-[11px] text-muted-foreground"
            >
              Filter by title, statement, or topic.
            </p>
            <div className="grid grid-cols-2 gap-2">
              <label className="flex flex-col gap-1 text-[11px] font-medium text-muted-foreground">
                Dialect
                <select
                  className="h-8 rounded-lg border border-input bg-background px-2 text-xs text-foreground outline-none focus-visible:ring-3 focus-visible:ring-ring/50"
                  value={dialect}
                  onChange={(event) =>
                    setDialect(event.target.value as ProblemDialect | "all")
                  }
                  aria-label="Filter by dialect"
                >
                  <option value="all">All</option>
                  <option value="postgresql">PostgreSQL</option>
                  <option value="sqlserver">SQL Server</option>
                </select>
              </label>
              <label className="flex flex-col gap-1 text-[11px] font-medium text-muted-foreground">
                Level
                <select
                  className="h-8 rounded-lg border border-input bg-background px-2 text-xs text-foreground outline-none focus-visible:ring-3 focus-visible:ring-ring/50"
                  value={difficulty}
                  onChange={(event) =>
                    setDifficulty(
                      event.target.value as ProblemDifficulty | "all"
                    )
                  }
                  aria-label="Filter by difficulty"
                >
                  <option value="all">All</option>
                  <option value="easy">Easy</option>
                  <option value="medium">Medium</option>
                  <option value="hard">Hard</option>
                </select>
              </label>
            </div>
          </CardHeader>
          <CardContent className="min-h-0 flex-1 overflow-y-auto p-2">
            {filteredProblems.length === 0 ? (
              <div className="rounded-lg border border-dashed p-4 text-center text-xs text-muted-foreground">
                No problems match these filters.
              </div>
            ) : (
              <div
                className="grid gap-1"
                role="listbox"
                aria-label="SQL problems"
              >
                {filteredProblems.map((problem) => (
                  <ProblemListItem
                    key={problem.id}
                    problem={problem}
                    selected={problem.id === selectedProblem.id}
                    onSelect={() => {
                      setSelectedId(problem.id)
                      setResult(null)
                    }}
                  />
                ))}
              </div>
            )}
          </CardContent>
        </Card>

        <Card className="min-h-0 overflow-hidden">
          <CardHeader className="border-b bg-muted/20">
            <div className="flex flex-wrap items-start justify-between gap-3">
              <div>
                <CardTitle className="text-base">
                  {selectedProblem.title}
                </CardTitle>
                <CardDescription className="mt-1">
                  Write one read-only statement. Drafts stay with this problem.
                </CardDescription>
              </div>
              <div className="flex gap-2">
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => void run("run")}
                  disabled={busy}
                >
                  <Play />
                  Run sample
                </Button>
                <Button
                  size="sm"
                  onClick={() => void run("submit")}
                  disabled={busy}
                >
                  <Send />
                  Submit
                </Button>
                {busy && activeRequestId ? (
                  <Button
                    variant="destructive"
                    size="sm"
                    onClick={() => void cancelActiveJudge()}
                  >
                    <XCircle />
                    Cancel
                  </Button>
                ) : null}
              </div>
            </div>
          </CardHeader>
          <CardContent className="flex min-h-0 flex-1 flex-col gap-3 p-3">
            <Label htmlFor="sql-editor">SQL editor</Label>
            <textarea
              id="sql-editor"
              value={draft}
              onChange={(event) =>
                setDrafts((current) => ({
                  ...current,
                  [selectedProblem.id]: event.target.value,
                }))
              }
              spellCheck={false}
              className="min-h-52 flex-1 resize-none rounded-lg border border-input bg-[#10151d] p-4 font-mono text-[13px] leading-6 text-slate-100 shadow-inner outline-none focus-visible:ring-3 focus-visible:ring-ring/50"
              aria-label={`SQL editor for ${selectedProblem.title}`}
            />
            <div className="flex flex-wrap items-center justify-between gap-2 text-[11px] text-muted-foreground">
              <span>UTF-8 · read-only policy · 64 KiB maximum</span>
              <span>{draft.length.toLocaleString()} characters</span>
            </div>
            {result ? <ResultBanner result={result} /> : null}
          </CardContent>
        </Card>

        <Card className="min-h-0 overflow-y-auto">
          <CardHeader className="border-b">
            <CardTitle className="flex items-center gap-2 text-sm">
              <Table2 className="size-4" /> Schema & prompt
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-5 p-4">
            <section
              aria-labelledby="environment-heading"
              className="rounded-lg border bg-muted/20 p-3"
            >
              <div className="flex items-center justify-between gap-2">
                <div>
                  <h3
                    id="environment-heading"
                    className="text-xs font-semibold tracking-wide text-muted-foreground uppercase"
                  >
                    Managed environments
                  </h3>
                  <p className="mt-1 text-[11px] text-muted-foreground">
                    Owned schemas only. External connections never appear here.
                  </p>
                </div>
                <Button
                  variant="ghost"
                  size="icon"
                  onClick={() => void refreshEnvironments()}
                  disabled={environmentBusy}
                  aria-label="Refresh managed environments"
                >
                  <RefreshCw
                    className={cn("size-4", environmentBusy && "animate-spin")}
                  />
                </Button>
              </div>
              <div className="mt-3 flex gap-2">
                <Input
                  value={environmentNamespace}
                  onChange={(event) =>
                    setEnvironmentNamespace(event.target.value)
                  }
                  aria-label="Managed environment namespace"
                  placeholder="sj_practice"
                  className="h-8 font-mono text-xs"
                />
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => void createPracticeEnvironment()}
                  disabled={environmentBusy}
                >
                  Create
                </Button>
              </div>
              {environments.length > 0 ? (
                <div className="mt-3 space-y-2">
                  {environments.map((environment) => (
                    <div
                      key={environment.envId}
                      className="rounded-md border bg-background p-2"
                    >
                      <div className="flex items-start justify-between gap-2">
                        <div className="min-w-0">
                          <p className="truncate font-mono text-xs font-semibold">
                            {environment.namespace}
                          </p>
                          <p className="mt-1 text-[11px] text-muted-foreground">
                            {environment.envId} · {environment.state}
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
                      <div className="mt-2 flex justify-end gap-1">
                        <Button
                          variant="ghost"
                          size="sm"
                          onClick={() => void resetEnvironment(environment)}
                          disabled={
                            environmentBusy || environment.state !== "ready"
                          }
                        >
                          Reset
                        </Button>
                        <Button
                          variant="ghost"
                          size="icon"
                          onClick={() => void deleteEnvironment(environment)}
                          disabled={
                            environmentBusy || environment.state !== "ready"
                          }
                          aria-label={`Delete ${environment.namespace}`}
                        >
                          <Trash2 className="size-4" />
                        </Button>
                      </div>
                    </div>
                  ))}
                </div>
              ) : (
                <p className="mt-3 text-xs text-muted-foreground">
                  No managed environment yet.
                </p>
              )}
              {environmentMessage ? (
                <p className="mt-2 text-xs text-muted-foreground" role="status">
                  {environmentMessage}
                </p>
              ) : null}
            </section>
            <section aria-labelledby="statement-heading">
              <h3
                id="statement-heading"
                className="text-xs font-semibold tracking-wide text-muted-foreground uppercase"
              >
                Statement
              </h3>
              <p className="mt-2 text-sm leading-6">
                {selectedProblem.statement}
              </p>
            </section>
            <section
              aria-labelledby="history-heading"
              className="rounded-lg border bg-muted/20 p-3"
            >
              <div className="flex items-center justify-between gap-2">
                <h3
                  id="history-heading"
                  className="text-xs font-semibold tracking-wide text-muted-foreground uppercase"
                >
                  Recent submissions
                </h3>
                <Badge variant="secondary">{history.length}</Badge>
              </div>
              {history.length > 0 ? (
                <div className="mt-2 space-y-1.5">
                  {history.slice(0, 3).map((submission) => (
                    <div
                      key={submission.submissionId}
                      className="flex items-center justify-between gap-2 text-xs"
                    >
                      <span className="truncate text-muted-foreground">
                        {submission.createdAt}
                      </span>
                      <Badge
                        variant={
                          submission.verdict === "Accepted"
                            ? "default"
                            : "outline"
                        }
                      >
                        {submission.verdict}
                      </Badge>
                    </div>
                  ))}
                </div>
              ) : (
                <p className="mt-2 text-xs text-muted-foreground">
                  No submissions yet.
                </p>
              )}
            </section>
            <section aria-labelledby="schema-heading">
              <h3
                id="schema-heading"
                className="flex items-center gap-2 text-xs font-semibold tracking-wide text-muted-foreground uppercase"
              >
                <Database className="size-3.5" /> Tables
              </h3>
              <div className="mt-2 space-y-2">
                {selectedProblem.schema.map((table) => (
                  <div
                    key={table.name}
                    className="rounded-lg border bg-muted/20 p-3"
                  >
                    <p className="font-mono text-xs font-semibold">
                      {table.name}
                    </p>
                    <ul className="mt-2 space-y-1.5 text-xs text-muted-foreground">
                      {table.columns.map((column) => (
                        <li
                          key={column.name}
                          className="flex items-center justify-between gap-2"
                        >
                          <span className="font-mono text-foreground">
                            {column.name}
                          </span>
                          <span>
                            {column.type}
                            {column.nullable ? " · nullable" : ""}
                          </span>
                        </li>
                      ))}
                    </ul>
                  </div>
                ))}
              </div>
            </section>
            <section aria-labelledby="topics-heading">
              <h3
                id="topics-heading"
                className="text-xs font-semibold tracking-wide text-muted-foreground uppercase"
              >
                Topics
              </h3>
              <div className="mt-2 flex flex-wrap gap-1.5">
                {selectedProblem.topics.map((topic) => (
                  <Badge key={topic} variant="outline">
                    {topic}
                  </Badge>
                ))}
              </div>
            </section>
            <section aria-labelledby="sample-heading">
              <h3
                id="sample-heading"
                className="text-xs font-semibold tracking-wide text-muted-foreground uppercase"
              >
                Public sample output
              </h3>
              <div className="mt-2 overflow-x-auto rounded-lg border">
                <table className="w-full text-left text-xs">
                  <thead className="bg-muted/40 text-muted-foreground">
                    <tr>
                      {Object.keys(selectedProblem.sampleRows[0]).map((key) => (
                        <th key={key} className="px-2 py-2 font-medium">
                          {key}
                        </th>
                      ))}
                    </tr>
                  </thead>
                  <tbody>
                    {selectedProblem.sampleRows.map((row, index) => (
                      <tr key={index} className="border-t">
                        {Object.values(row).map((value, valueIndex) => (
                          <td key={valueIndex} className="px-2 py-2 font-mono">
                            {value ?? "NULL"}
                          </td>
                        ))}
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </section>
          </CardContent>
        </Card>
      </div>
    </div>
  )
}

function ProblemListItem({
  problem,
  selected,
  onSelect,
}: {
  problem: LearningProblem
  selected: boolean
  onSelect: () => void
}) {
  return (
    <button
      type="button"
      role="option"
      aria-selected={selected}
      onClick={onSelect}
      className={cn(
        "w-full rounded-lg border border-transparent p-3 text-left transition-colors focus-visible:ring-3 focus-visible:ring-ring/50",
        selected ? "border-primary/30 bg-primary/10" : "hover:bg-muted/60"
      )}
    >
      <div className="flex items-start justify-between gap-2">
        <span className="text-sm font-medium">{problem.title}</span>
        <Badge variant={selected ? "default" : "secondary"}>
          {difficultyLabels[problem.difficulty]}
        </Badge>
      </div>
      <p className="mt-1 line-clamp-2 text-xs leading-5 text-muted-foreground">
        {problem.statement}
      </p>
      <div className="mt-2 flex items-center gap-1.5 text-[10px] text-muted-foreground">
        <Filter className="size-3" /> {problem.dialect} ·{" "}
        {problem.topics.length} topics
      </div>
    </button>
  )
}

function ResultBanner({ result }: { result: JudgeRunResult }) {
  const isSuccess =
    result.status === "completed" && result.verdict === "Accepted"
  return (
    <div
      className={cn(
        "rounded-lg border p-3 text-sm",
        isSuccess
          ? "border-emerald-500/30 bg-emerald-500/10"
          : "border-amber-500/30 bg-amber-500/10"
      )}
      role="status"
    >
      <div className="flex items-start gap-2">
        {isSuccess ? (
          <CheckCircle2 className="mt-0.5 size-4 text-emerald-600" />
        ) : (
          <span
            className="mt-0.5 size-2 rounded-full bg-amber-500"
            aria-hidden="true"
          />
        )}
        <div>
          <p className="font-medium">
            {result.verdict ?? "Runner unavailable"}
          </p>
          <p className="mt-1 text-xs text-muted-foreground">
            {result.message} · {result.durationMs} ms
          </p>
        </div>
      </div>
    </div>
  )
}
