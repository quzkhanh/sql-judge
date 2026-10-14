import { useEffect, useMemo, useRef, useState } from "react"
import Editor from "@monaco-editor/react"
import {
  CheckCircle2,
  ChevronLeft,
  ChevronRight,
  Database,
  Filter,
  LoaderCircle,
  Play,
  RefreshCw,
  Send,
  Table2,
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
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { cn } from "@/lib/utils"
import {
  cancelJudge,
  listInstalledPacks,
  runJudge,
  type JudgeRunResult,
  type InstalledPack,
} from "@/lib/judge-api"
import {
  catalogProblems,
  catalogProblemKey,
  canJudgeProblem,
  filterProblems,
  hasSampleData,
  sampleColumns,
  type LearningProblem,
  type ProblemDifficulty,
  type ProblemDialect,
} from "./catalog"
import {
  beginJudgeRequest,
  isCurrentJudgeRequest,
  type JudgeRequest,
} from "./judge-state"
import { learningLoadErrorMessage } from "./load-state"
import {
  pageForProblem,
  paginateProblems,
  problemsForPack,
  stopVisibleAfter,
} from "./practice-workspace-state"
import { sqlCompletionItems } from "./sql-completions"

const difficultyLabels: Record<ProblemDifficulty, string> = {
  easy: "Easy",
  medium: "Medium",
  hard: "Hard",
}

const MAX_SQL_LENGTH = 65_536
const MAX_RENDERED_ROWS = 100
const MAX_RENDERED_CELL_LENGTH = 512

export function LearningWorkspace() {
  const [packs, setPacks] = useState<InstalledPack[]>([])
  const [query, setQuery] = useState("")
  const [jumpQuery, setJumpQuery] = useState("")
  const [dialect, setDialect] = useState<ProblemDialect | "all">("all")
  const [difficulty, setDifficulty] = useState<ProblemDifficulty | "all">("all")
  const [selectedPackId, setSelectedPackId] = useState("")
  const [selectedId, setSelectedId] = useState("")
  const [drafts, setDrafts] = useState<Record<string, string>>({})
  const [result, setResult] = useState<JudgeRunResult | null>(null)
  const [busy, setBusy] = useState(false)
  const [activeRequestId, setActiveRequestId] = useState<string | null>(null)
  const [activeMode, setActiveMode] = useState<"run" | "submit" | null>(null)
  const [page, setPage] = useState(0)
  const [elapsedBusyMs, setElapsedBusyMs] = useState(0)
  const [catalogLoading, setCatalogLoading] = useState(true)
  const [catalogError, setCatalogError] = useState<string | null>(null)
  const [catalogReloadToken, setCatalogReloadToken] = useState(0)
  const selectedProblemIdRef = useRef("")
  const judgeGenerationRef = useRef(0)
  const activeJudgeRequestRef = useRef<JudgeRequest | null>(null)
  const activeRequestIdRef = useRef<string | null>(null)
  const completionItemsRef = useRef<ReturnType<typeof sqlCompletionItems>>([])
  const completionProviderRef = useRef<{ dispose(): void } | null>(null)

  function invalidateJudgeRequests() {
    judgeGenerationRef.current += 1
    activeJudgeRequestRef.current = null
    activeRequestIdRef.current = null
    setBusy(false)
    setActiveRequestId(null)
    setActiveMode(null)
    setResult(null)
  }

  function selectProblem(problemId: string) {
    if (selectedProblemIdRef.current !== problemId) {
      invalidateJudgeRequests()
      selectedProblemIdRef.current = problemId
    }
    setSelectedId(problemId)
  }

  function retryCatalogLoad() {
    invalidateJudgeRequests()
    setCatalogLoading(true)
    setCatalogError(null)
    setCatalogReloadToken((current) => current + 1)
  }

  useEffect(() => {
    let active = true

    function setLoadedProblem(problemId: string) {
      if (selectedProblemIdRef.current !== problemId) {
        judgeGenerationRef.current += 1
        activeJudgeRequestRef.current = null
        activeRequestIdRef.current = null
        selectedProblemIdRef.current = problemId
        setBusy(false)
        setActiveRequestId(null)
        setResult(null)
      }
      setSelectedId(problemId)
    }

    void listInstalledPacks()
      .then((installedPacks) => {
        if (!active) {
          return
        }
        setPacks(installedPacks)
        setSelectedPackId((current) =>
          installedPacks.some((pack) => pack.packId === current)
            ? current
            : (installedPacks[0]?.packId ?? "")
        )
        const firstProblem = installedPacks[0]?.problems[0]
        const availableIds = new Set(
          installedPacks.flatMap((pack) =>
            pack.problems.map((problem) => catalogProblemKey(pack, problem.id))
          )
        )
        const nextSelectedId =
          selectedProblemIdRef.current &&
          availableIds.has(selectedProblemIdRef.current)
            ? selectedProblemIdRef.current
            : firstProblem
              ? catalogProblemKey(installedPacks[0], firstProblem.id)
              : ""
        setLoadedProblem(nextSelectedId)
      })
      .catch(() => {
        if (!active) {
          return
        }
        setPacks([])
        setLoadedProblem("")
        setCatalogError(learningLoadErrorMessage("catalog"))
      })
      .finally(() => {
        if (active) {
          setCatalogLoading(false)
        }
      })

    return () => {
      active = false
    }
  }, [catalogReloadToken])

  const learningProblems = useMemo(() => catalogProblems(packs), [packs])
  const packProblems = useMemo(
    () => problemsForPack(learningProblems, selectedPackId),
    [learningProblems, selectedPackId]
  )
  const filteredProblems = useMemo(
    () => filterProblems(packProblems, query, dialect, difficulty),
    [difficulty, dialect, packProblems, query]
  )
  const paginatedProblems = useMemo(
    () => paginateProblems(filteredProblems, page),
    [filteredProblems, page]
  )
  const selectedProblem = learningProblems.find(
    (problem) => problem.id === selectedId
  ) ??
    filteredProblems[0] ?? {
      id: "",
      problemId: "",
      title: "No imported problem pack",
      difficulty: "easy" as const,
      dialect: "postgresql" as const,
      topics: [],
      statement: "Import a validated pack to begin practicing.",
      packId: "",
      packVersion: "",
      packDigest: "",
      schema: [],
      sampleRows: [],
      starterSql: "",
    }
  const draft = drafts[selectedProblem.id] ?? selectedProblem.starterSql
  const judgeable = canJudgeProblem(selectedProblem)
  const completionItems = useMemo(
    () => sqlCompletionItems(selectedProblem.dialect, selectedProblem.schema),
    [selectedProblem.dialect, selectedProblem.schema]
  )

  useEffect(() => {
    completionItemsRef.current = completionItems
  }, [completionItems])

  useEffect(() => {
    if (!busy) {
      return
    }
    const startedAt = Date.now()
    const timer = window.setInterval(() => setElapsedBusyMs(Date.now() - startedAt), 100)
    return () => window.clearInterval(timer)
  }, [busy])

  function updateDraft(value: string | undefined) {
    setDrafts((current) => ({
      ...current,
      [selectedProblem.id]: (value ?? "").slice(0, MAX_SQL_LENGTH),
    }))
  }

  async function run(mode: "run" | "submit") {
    if (!selectedProblem.id || !judgeable) {
      return
    }
    const problemIdentity = selectedProblem.id
    const request = beginJudgeRequest(
      judgeGenerationRef.current,
      problemIdentity
    )
    judgeGenerationRef.current = request.generation
    const requestId = crypto.randomUUID()
    activeJudgeRequestRef.current = request
    activeRequestIdRef.current = requestId
    setBusy(true)
    setElapsedBusyMs(0)
    setActiveRequestId(requestId)
    setActiveMode(mode)
    setResult(null)
    try {
      const nextResult = await runJudge({
        problemId: selectedProblem.problemId,
        packId: selectedProblem.packId,
        packVersion: selectedProblem.packVersion,
        packDigest: selectedProblem.packDigest,
        sql: draft,
        mode,
        requestId,
      })
      if (
        isCurrentJudgeRequest(
          request,
          judgeGenerationRef.current,
          selectedProblemIdRef.current
        )
      ) {
        setResult(nextResult)
      }
    } catch {
      if (
        isCurrentJudgeRequest(
          request,
          judgeGenerationRef.current,
          selectedProblemIdRef.current
        )
      ) {
        setResult({
          status: "error",
          durationMs: 0,
          message:
            "The native judge returned an unavailable or malformed response.",
        })
      }
    } finally {
      if (
        isCurrentJudgeRequest(
          request,
          judgeGenerationRef.current,
          selectedProblemIdRef.current
        )
      ) {
        activeJudgeRequestRef.current = null
        activeRequestIdRef.current = null
        setBusy(false)
        setActiveRequestId(null)
        setActiveMode(null)
      }
    }
  }

  async function cancelActiveJudge() {
    const request = activeJudgeRequestRef.current
    const requestId = activeRequestIdRef.current
    if (request && requestId) {
      await cancelJudge(requestId).catch(() => undefined)
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
            Select a pack, solve one problem, and submit against isolated fixtures.
          </p>
        </div>
        <label className="flex flex-col gap-1 text-xs font-medium text-muted-foreground">
          Active pack
          <select
            className="h-9 min-w-56 rounded-lg border border-input bg-background px-2 text-sm text-foreground outline-none focus-visible:ring-3 focus-visible:ring-ring/50"
            value={selectedPackId}
            onChange={(event) => {
              const packId = event.target.value
              setSelectedPackId(packId)
              setPage(0)
              const firstProblem = problemsForPack(learningProblems, packId)[0]
              if (firstProblem) {
                selectProblem(firstProblem.id)
              }
            }}
            disabled={catalogLoading || packs.length === 0}
          >
            {packs.map((pack) => (
              <option key={`${pack.packId}-${pack.version}`} value={pack.packId}>
                {pack.title} · {pack.version}
              </option>
            ))}
          </select>
        </label>
      </header>

      <div className="grid min-h-0 flex-1 gap-4 xl:grid-cols-[280px_minmax(0,1fr)_300px]">
        <Card className="min-h-0 xl:flex">
          <CardHeader className="gap-3 border-b">
            <div className="flex items-center justify-between">
              <CardTitle className="text-sm">Problems</CardTitle>
              <Badge variant="secondary">{packProblems.length}</Badge>
            </div>
            <Label className="sr-only" htmlFor="problem-search">
              Search problems
            </Label>
            <Input
              id="problem-search"
              value={query}
              onChange={(event) => {
                setQuery(event.target.value)
                setPage(0)
              }}
              placeholder="Search topics or title"
              aria-describedby="problem-filter-help"
            />
            <p
              id="problem-filter-help"
              className="text-[11px] text-muted-foreground"
            >
              Filter by title, statement, or topic.
            </p>
            <Label className="sr-only" htmlFor="problem-jump">
              Jump to a problem
            </Label>
            <Input
              id="problem-jump"
              value={jumpQuery}
              list="pack-problem-options"
              onChange={(event) => {
                const nextValue = event.target.value
                setJumpQuery(nextValue)
                const problem = packProblems.find(
                  (candidate) => candidate.title === nextValue
                )
                if (problem) {
                  selectProblem(problem.id)
                  setPage(pageForProblem(filteredProblems, problem.id))
                }
              }}
              placeholder="Jump to a problem"
            />
            <datalist id="pack-problem-options">
              {packProblems.map((problem) => (
                <option key={problem.id} value={problem.title} />
              ))}
            </datalist>
            <div className="grid grid-cols-2 gap-2">
              <label className="flex flex-col gap-1 text-[11px] font-medium text-muted-foreground">
                Dialect
                <select
                  className="h-8 rounded-lg border border-input bg-background px-2 text-xs text-foreground outline-none focus-visible:ring-3 focus-visible:ring-ring/50"
                  value={dialect}
                  onChange={(event) => {
                    setDialect(event.target.value as ProblemDialect | "all")
                    setPage(0)
                  }}
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
                  onChange={(event) => {
                    setDifficulty(event.target.value as ProblemDifficulty | "all")
                    setPage(0)
                  }}
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
            {catalogLoading ? (
              <p className="rounded-lg border border-dashed p-4 text-center text-xs text-muted-foreground">
                Loading installed problem packs...
              </p>
            ) : catalogError ? (
              <div
                className="rounded-lg border border-destructive/30 bg-destructive/5 p-4 text-center text-xs"
                role="alert"
              >
                <p className="text-destructive">{catalogError}</p>
                <Button
                  className="mt-3"
                  variant="outline"
                  size="sm"
                  onClick={retryCatalogLoad}
                >
                  <RefreshCw />
                  Retry
                </Button>
              </div>
            ) : learningProblems.length === 0 ? (
              <div className="rounded-lg border border-dashed p-4 text-center text-xs text-muted-foreground">
                No complete problem metadata is available. Import a validated
                pack with problem manifests to begin practicing.
              </div>
            ) : filteredProblems.length === 0 ? (
              <div className="rounded-lg border border-dashed p-4 text-center text-xs text-muted-foreground">
                No problems match these filters.
              </div>
            ) : (
              <div
                className="grid gap-1"
                role="listbox"
                aria-label="SQL problems"
              >
                {paginatedProblems.items.map((problem) => (
                  <ProblemListItem
                    key={problem.id}
                    problem={problem}
                    selected={problem.id === selectedProblem.id}
                    onSelect={() => {
                      selectProblem(problem.id)
                      setPage(pageForProblem(filteredProblems, problem.id))
                    }}
                  />
                ))}
              </div>
            )}
            {filteredProblems.length > 0 ? (
              <div className="mt-3 flex items-center justify-between border-t pt-3 text-xs text-muted-foreground">
                <span>
                  {paginatedProblems.start + 1}-{paginatedProblems.end} of {filteredProblems.length}
                </span>
                <div className="flex gap-1">
                  <Button variant="outline" size="icon-sm" aria-label="Previous problems" disabled={paginatedProblems.page === 0} onClick={() => setPage((current) => current - 1)}>
                    <ChevronLeft />
                  </Button>
                  <Button variant="outline" size="icon-sm" aria-label="Next problems" disabled={paginatedProblems.page >= paginatedProblems.totalPages - 1} onClick={() => setPage((current) => current + 1)}>
                    <ChevronRight />
                  </Button>
                </div>
              </div>
            ) : null}
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
                  disabled={
                    busy ||
                    catalogLoading ||
                    catalogError !== null ||
                    !selectedProblem.id ||
                    !judgeable
                  }
                  title={
                    judgeable
                      ? undefined
                      : "An installed problem pack is required"
                  }
                >
                  {busy && activeMode === "run" ? <LoaderCircle className="animate-spin" /> : <Play />}
                  {busy && activeMode === "run" ? "Running" : "Run sample"}
                </Button>
                <Button
                  size="sm"
                  onClick={() => void run("submit")}
                  disabled={
                    busy ||
                    catalogLoading ||
                    catalogError !== null ||
                    !selectedProblem.id ||
                    !judgeable
                  }
                  title={
                    judgeable
                      ? undefined
                      : "An installed problem pack is required"
                  }
                >
                  {busy && activeMode === "submit" ? <LoaderCircle className="animate-spin" /> : <Send />}
                  {busy && activeMode === "submit" ? "Submitting" : "Submit"}
                </Button>
                {busy && activeRequestId && stopVisibleAfter(elapsedBusyMs) ? (
                  <Button
                    variant="destructive"
                    size="sm"
                    onClick={() => void cancelActiveJudge()}
                  >
                    <XCircle />
                    Stop
                  </Button>
                ) : null}
              </div>
            </div>
          </CardHeader>
          <CardContent className="flex min-h-0 flex-1 flex-col gap-3 p-3">
            <Label htmlFor="sql-editor">SQL editor</Label>
            <div className="min-h-52 flex-1 overflow-hidden rounded-lg border border-input bg-[#10151d] shadow-inner">
              <Editor
                height="100%"
                language="sql"
                theme="vs-dark"
                value={draft}
                onChange={updateDraft}
                onMount={(_editor, monaco) => {
                  completionProviderRef.current?.dispose()
                  completionProviderRef.current = monaco.languages.registerCompletionItemProvider("sql", {
                    provideCompletionItems: () => ({
                      suggestions: completionItemsRef.current.map((item) => ({
                        label: item.label,
                        detail: item.detail,
                        insertText: item.insertText,
                        insertTextRules: item.kind === "snippet" ? monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet : undefined,
                        kind: item.kind === "table" ? monaco.languages.CompletionItemKind.Struct : item.kind === "column" ? monaco.languages.CompletionItemKind.Field : item.kind === "snippet" ? monaco.languages.CompletionItemKind.Snippet : monaco.languages.CompletionItemKind.Keyword,
                      })),
                    }),
                  })
                }}
                options={{
                  ariaLabel: `SQL editor for ${selectedProblem.title}`,
                  automaticLayout: true,
                  fontSize: 13,
                  lineNumbers: "on",
                  minimap: { enabled: false },
                  padding: { top: 12, bottom: 12 },
                  scrollBeyondLastLine: false,
                  tabSize: 2,
                  wordWrap: "on",
                }}
              />
            </div>
            <div className="flex flex-wrap items-center justify-between gap-2 text-[11px] text-muted-foreground">
              <span>UTF-8 · read-only policy · 64 KiB maximum</span>
              <span>{draft.length.toLocaleString()} characters</span>
            </div>
            {result ? (
              <>
                <ResultBanner result={result} />
                <ResultTable result={result} />
              </>
            ) : null}
          </CardContent>
        </Card>

        <Card className="min-h-0 overflow-y-auto xl:col-start-3 xl:row-start-1">
          <CardHeader className="border-b">
            <CardTitle className="flex items-center gap-2 text-sm">
              <Table2 className="size-4" /> Problem context
            </CardTitle>
          </CardHeader>
          <CardContent className="p-4">
            <Tabs defaultValue="statement">
              <TabsList variant="line" className="w-full justify-start border-b">
                <TabsTrigger value="statement">Statement</TabsTrigger>
                <TabsTrigger value="schema">Schema</TabsTrigger>
                <TabsTrigger value="sample">Sample</TabsTrigger>
              </TabsList>
              <TabsContent value="statement" className="space-y-5 pt-4">
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
            <section aria-labelledby="topics-heading">
              <h3 id="topics-heading" className="text-xs font-semibold tracking-wide text-muted-foreground uppercase">Topics</h3>
              <div className="mt-2 flex flex-wrap gap-1.5">
                {selectedProblem.topics.map((topic) => <Badge key={topic} variant="outline">{topic}</Badge>)}
              </div>
            </section>
              </TabsContent>
              <TabsContent value="schema" className="pt-4">
            <section aria-labelledby="schema-heading">
              <h3
                id="schema-heading"
                className="flex items-center gap-2 text-xs font-semibold tracking-wide text-muted-foreground uppercase"
              >
                <Database className="size-3.5" /> Tables
              </h3>
              <div className="mt-2 space-y-2">
                {selectedProblem.schema.length === 0 ? (
                  <p className="rounded-lg border border-dashed p-3 text-xs text-muted-foreground">
                    Schema metadata is unavailable for this problem.
                  </p>
                ) : (
                  selectedProblem.schema.map((table) => (
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
                  ))
                )}
              </div>
            </section>
              </TabsContent>
              <TabsContent value="sample" className="pt-4">
            <section aria-labelledby="sample-heading">
              <h3
                id="sample-heading"
                className="text-xs font-semibold tracking-wide text-muted-foreground uppercase"
              >
                Public sample output
              </h3>
              {hasSampleData(selectedProblem) ? (
                <div className="mt-2 overflow-x-auto rounded-lg border">
                  <table className="w-full text-left text-xs">
                    <thead className="bg-muted/40 text-muted-foreground">
                      <tr>
                        {sampleColumns(selectedProblem).map((key) => (
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
                            <td
                              key={valueIndex}
                              className="px-2 py-2 font-mono"
                            >
                              {value ?? "NULL"}
                            </td>
                          ))}
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
              ) : (
                <p className="mt-2 rounded-lg border border-dashed p-3 text-xs text-muted-foreground">
                  Public sample output is unavailable for this problem.
                </p>
              )}
            </section>
              </TabsContent>
            </Tabs>
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

function ResultTable({ result }: { result: JudgeRunResult }) {
  if (result.status !== "completed" || !result.columns?.length) {
    return null
  }

  const columns = result.columns
  const rows = result.rows ?? []
  const visibleRows = rows.slice(0, MAX_RENDERED_ROWS)
  const rowsTruncated = rows.length > visibleRows.length

  return (
    <section
      aria-label="Query result"
      className="min-w-0 overflow-hidden rounded-lg border"
    >
      <div className="flex items-center justify-between gap-3 border-b bg-muted/30 px-3 py-2 text-xs">
        <span className="font-medium">Result preview</span>
        <span className="text-muted-foreground">
          {rowsTruncated
            ? `Showing ${visibleRows.length} of ${rows.length} rows`
            : `${rows.length} row${rows.length === 1 ? "" : "s"}`}
        </span>
      </div>
      {visibleRows.length > 0 ? (
        <div className="max-h-72 overflow-auto">
          <table className="w-full text-left text-xs">
            <thead className="sticky top-0 bg-background text-muted-foreground">
              <tr>
                {columns.map((column) => (
                  <th
                    key={column.name}
                    className="border-b px-3 py-2 font-medium whitespace-nowrap"
                  >
                    <span>{column.name}</span>
                    <span className="ml-1 font-normal opacity-60">
                      {column.type}
                    </span>
                  </th>
                ))}
              </tr>
            </thead>
            <tbody>
              {visibleRows.map((row, rowIndex) => (
                <tr key={rowIndex} className="border-t align-top">
                  {columns.map((column, columnIndex) => {
                    const value = formatResultValue(row[columnIndex])
                    const displayValue = value.slice(
                      0,
                      MAX_RENDERED_CELL_LENGTH
                    )
                    return (
                      <td
                        key={`${column.name}-${columnIndex}`}
                        className="max-w-64 px-3 py-2 font-mono break-words"
                        title={value}
                      >
                        {displayValue}
                        {value.length > displayValue.length ? "…" : ""}
                      </td>
                    )
                  })}
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      ) : (
        <p className="p-4 text-center text-xs text-muted-foreground">
          Query returned no rows.
        </p>
      )}
    </section>
  )
}

function formatResultValue(
  value: string | number | boolean | null | undefined
) {
  if (value === null || value === undefined) {
    return "NULL"
  }
  if (typeof value === "boolean") {
    return value ? "true" : "false"
  }
  return String(value)
}
