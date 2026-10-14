import { useEffect, useState } from "react"
import {
  FileArchive,
  FolderOpen,
  LockKeyhole,
  RefreshCw,
  Sparkles,
} from "lucide-react"

import { Badge } from "@/components/ui/badge"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import {
  importPack,
  listInstalledPacks,
  pickPackSource,
  type InstalledPack,
} from "@/lib/judge-api"
import { readPackImportError } from "@/lib/pack-errors"
import { catalogProblems, hasInstalledProblems } from "./catalog"
import { learningLoadErrorMessage, learningLoadPhase } from "./load-state"

export function PacksScreen() {
  const [sourcePath, setSourcePath] = useState("")
  const [busy, setBusy] = useState(false)
  const [message, setMessage] = useState<string | null>(null)
  const [packs, setPacks] = useState<InstalledPack[]>([])
  const [catalogLoading, setCatalogLoading] = useState(true)
  const [catalogError, setCatalogError] = useState<string | null>(null)
  const [catalogReloadToken, setCatalogReloadToken] = useState(0)
  const problems = catalogProblems(packs)
  const topics = new Set(problems.flatMap((problem) => problem.topics))
  const catalogPhase = learningLoadPhase(
    catalogLoading,
    catalogError,
    packs.length > 0
  )

  function retryCatalogLoad() {
    setCatalogLoading(true)
    setCatalogError(null)
    setCatalogReloadToken((current) => current + 1)
  }

  useEffect(() => {
    let active = true
    void listInstalledPacks()
      .then((installedPacks) => {
        if (active) {
          setPacks(installedPacks)
        }
      })
      .catch(() => {
        if (active) {
          setPacks([])
          setCatalogError(learningLoadErrorMessage("catalog"))
        }
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

  async function handleImport() {
    const path = sourcePath.trim()
    if (!path) {
      setMessage("Enter a local pack directory or .sqljudge.zip path.")
      return
    }
    setBusy(true)
    setMessage(
      "Validating pack, running reference tests, and publishing safely..."
    )
    try {
      const result = await importPack(path)
      const status =
        result.status === "already_installed"
          ? "already installed"
          : "published"
      setCatalogLoading(true)
      setCatalogError(null)
      let catalogRefreshFailed = false
      try {
        setPacks(await listInstalledPacks())
      } catch {
        catalogRefreshFailed = true
        setCatalogError(learningLoadErrorMessage("catalog"))
      } finally {
        setCatalogLoading(false)
      }
      setMessage(
        `${result.message} ${result.packId} v${result.version} (${status}).${
          catalogRefreshFailed
            ? " The catalog refresh failed; retry it below."
            : ""
        }`
      )
      setSourcePath("")
    } catch (reason) {
      setMessage(readPackImportError(reason))
    } finally {
      setBusy(false)
    }
  }

  async function handlePick(kind: "archive" | "directory") {
    setBusy(true)
    setMessage(null)
    try {
      const path = await pickPackSource(kind)
      if (path) {
        setSourcePath(path)
      }
    } catch {
      setMessage(
        "Native pack selection is unavailable. Enter a local path instead."
      )
    } finally {
      setBusy(false)
    }
  }

  return (
    <main className="min-h-0 flex-1 overflow-y-auto p-4 lg:p-6">
      <div className="mx-auto flex max-w-6xl flex-col gap-6">
        <header className="border-b pb-5">
          <p className="text-xs font-semibold tracking-[0.18em] text-primary uppercase">
            Content library
          </p>
          <h2 className="mt-1 text-2xl font-semibold tracking-tight">Packs</h2>
          <p className="mt-1 text-sm text-muted-foreground">
            Installed practice content available offline in this workspace.
          </p>
        </header>

        <Card>
          <CardHeader>
            <CardTitle className="text-base">Import a pack</CardTitle>
            <p className="text-sm text-muted-foreground">
              Use a local pack directory or a bounded `.sqljudge.zip` archive.
              The native runtime validates structure and reference SQL before
              publication.
            </p>
          </CardHeader>
          <CardContent className="space-y-3">
            <div className="flex flex-col gap-2 sm:flex-row">
              <Input
                aria-label="Local pack path"
                value={sourcePath}
                onChange={(event) => setSourcePath(event.target.value)}
                placeholder="/path/to/pack.sqljudge.zip"
                className="font-mono"
                disabled={busy}
              />
              <Button onClick={() => void handleImport()} disabled={busy}>
                Import pack
              </Button>
            </div>
            <div className="flex flex-wrap gap-2">
              <Button
                variant="outline"
                onClick={() => void handlePick("archive")}
                disabled={busy}
              >
                <FileArchive />
                Choose archive
              </Button>
              <Button
                variant="outline"
                onClick={() => void handlePick("directory")}
                disabled={busy}
              >
                <FolderOpen />
                Choose folder
              </Button>
            </div>
            {message ? (
              <p
                className="text-sm text-muted-foreground"
                role="status"
                aria-live="polite"
              >
                {message}
              </p>
            ) : null}
          </CardContent>
        </Card>

        {catalogPhase === "loading" ? (
          <Card>
            <CardContent className="p-8 text-center text-sm text-muted-foreground">
              <p role="status">Loading installed pack catalog...</p>
            </CardContent>
          </Card>
        ) : catalogPhase === "error" ? (
          <Card>
            <CardContent
              className="border-destructive/30 bg-destructive/5 p-8 text-center"
              role="alert"
            >
              <p className="text-sm text-destructive">{catalogError}</p>
              <Button
                className="mt-3"
                variant="outline"
                size="sm"
                onClick={retryCatalogLoad}
              >
                <RefreshCw />
                Retry
              </Button>
            </CardContent>
          </Card>
        ) : catalogPhase === "empty" ? (
          <Card>
            <CardContent className="p-8 text-center">
              <p className="text-sm font-medium">No installed packs yet</p>
              <p className="mt-1 text-sm text-muted-foreground">
                Import a validated pack to make practice content available.
              </p>
            </CardContent>
          </Card>
        ) : (
          <>
            {packs.map((pack) => (
              <Card
                key={`${pack.packId}@${pack.version}#${pack.digest}`}
                className="border-primary/20 bg-primary/[0.04]"
              >
                <CardContent className="flex flex-col gap-4 p-5 sm:flex-row sm:items-center sm:justify-between">
                  <div className="flex items-start gap-3">
                    <div className="rounded-lg bg-primary/10 p-2.5 text-primary">
                      <FolderOpen className="size-5" />
                    </div>
                    <div>
                      <div className="flex flex-wrap items-center gap-2">
                        <h3 className="font-semibold">{pack.title}</h3>
                        <Badge>Installed</Badge>
                      </div>
                      <p className="mt-1 text-sm text-muted-foreground">
                        {pack.packId}
                      </p>
                    </div>
                  </div>
                  <div className="flex flex-wrap gap-2 text-xs text-muted-foreground">
                    <span className="rounded-full border px-2.5 py-1">
                      {pack.problems.length} problems
                    </span>
                    <span className="rounded-full border px-2.5 py-1">
                      {pack.dialect}
                    </span>
                    <span className="rounded-full border px-2.5 py-1">
                      v{pack.version}
                    </span>
                  </div>
                </CardContent>
                {!hasInstalledProblems(pack) ? (
                  <CardContent className="border-t pt-4 text-sm text-muted-foreground">
                    This installed pack contains no problems yet. Import a pack
                    with validated problem manifests to start practicing.
                  </CardContent>
                ) : null}
              </Card>
            ))}

            <div className="grid gap-4 lg:grid-cols-[minmax(0,1.4fr)_minmax(260px,0.6fr)]">
              <Card>
                <CardHeader>
                  <CardTitle className="text-base">
                    Problems in this pack
                  </CardTitle>
                </CardHeader>
                <CardContent className="grid gap-3 sm:grid-cols-2">
                  {problems.length === 0 ? (
                    <p className="rounded-lg border border-dashed p-4 text-sm text-muted-foreground sm:col-span-2">
                      No problem metadata is available in the installed packs.
                    </p>
                  ) : (
                    problems.map((problem) => (
                      <div key={problem.id} className="rounded-lg border p-4">
                        <div className="flex items-start justify-between gap-3">
                          <h3 className="text-sm font-medium">
                            {problem.title}
                          </h3>
                          <Badge variant="outline">{problem.difficulty}</Badge>
                        </div>
                        <p className="mt-2 line-clamp-3 text-xs leading-5 text-muted-foreground">
                          {problem.statement}
                        </p>
                        <div className="mt-3 flex flex-wrap gap-1.5">
                          {problem.topics.map((topic) => (
                            <Badge
                              key={topic}
                              variant="secondary"
                              className="text-[10px]"
                            >
                              {topic}
                            </Badge>
                          ))}
                        </div>
                      </div>
                    ))
                  )}
                </CardContent>
              </Card>

              <Card>
                <CardHeader>
                  <CardTitle className="flex items-center gap-2 text-base">
                    <Sparkles className="size-4" /> Pack details
                  </CardTitle>
                </CardHeader>
                <CardContent className="space-y-4 text-sm">
                  <Detail
                    label="Dialect"
                    value={
                      packs.map((pack) => pack.dialect).join(", ") || "None"
                    }
                  />
                  <Detail label="Topics" value={`${topics.size} covered`} />
                  <Detail label="Execution" value="Managed runtime" />
                  <div className="rounded-lg bg-muted/40 p-3 text-xs leading-5 text-muted-foreground">
                    <LockKeyhole className="mb-2 size-4" />
                    Pack setup is validated before publication and runs only in
                    disposable, owned environments.
                  </div>
                </CardContent>
              </Card>
            </div>
          </>
        )}
      </div>
    </main>
  )
}

function Detail({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex items-center justify-between gap-4 border-b pb-3 last:border-0 last:pb-0">
      <span className="text-muted-foreground">{label}</span>
      <span className="font-medium">{value}</span>
    </div>
  )
}
