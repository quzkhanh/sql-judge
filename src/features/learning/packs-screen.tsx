import { FolderOpen, LockKeyhole, Sparkles } from "lucide-react"

import { Badge } from "@/components/ui/badge"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { learningProblems } from "./catalog"

export function PacksScreen() {
  const dialects = new Set(learningProblems.map((problem) => problem.dialect))
  const topics = new Set(learningProblems.flatMap((problem) => problem.topics))

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

        <Card className="border-primary/20 bg-primary/[0.04]">
          <CardContent className="flex flex-col gap-4 p-5 sm:flex-row sm:items-center sm:justify-between">
            <div className="flex items-start gap-3">
              <div className="rounded-lg bg-primary/10 p-2.5 text-primary">
                <FolderOpen className="size-5" />
              </div>
              <div>
                <div className="flex flex-wrap items-center gap-2">
                  <h3 className="font-semibold">HR SQL 101</h3>
                  <Badge>Installed</Badge>
                </div>
                <p className="mt-1 text-sm text-muted-foreground">
                  Core SQL patterns using isolated PostgreSQL fixtures.
                </p>
              </div>
            </div>
            <div className="flex flex-wrap gap-2 text-xs text-muted-foreground">
              <span className="rounded-full border px-2.5 py-1">
                {learningProblems.length} problems
              </span>
              <span className="rounded-full border px-2.5 py-1">
                {dialects.size} dialect
              </span>
              <span className="rounded-full border px-2.5 py-1">v1.0.0</span>
            </div>
          </CardContent>
        </Card>

        <div className="grid gap-4 lg:grid-cols-[minmax(0,1.4fr)_minmax(260px,0.6fr)]">
          <Card>
            <CardHeader>
              <CardTitle className="text-base">Problems in this pack</CardTitle>
            </CardHeader>
            <CardContent className="grid gap-3 sm:grid-cols-2">
              {learningProblems.map((problem) => (
                <div key={problem.id} className="rounded-lg border p-4">
                  <div className="flex items-start justify-between gap-3">
                    <h3 className="text-sm font-medium">{problem.title}</h3>
                    <Badge variant="outline">{problem.difficulty}</Badge>
                  </div>
                  <p className="mt-2 line-clamp-3 text-xs leading-5 text-muted-foreground">
                    {problem.statement}
                  </p>
                  <div className="mt-3 flex flex-wrap gap-1.5">
                    {problem.topics.map((topic) => (
                      <Badge key={topic} variant="secondary" className="text-[10px]">
                        {topic}
                      </Badge>
                    ))}
                  </div>
                </div>
              ))}
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2 text-base">
                <Sparkles className="size-4" /> Pack details
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-4 text-sm">
              <Detail label="Dialect" value="PostgreSQL" />
              <Detail label="Topics" value={`${topics.size} covered`} />
              <Detail label="Execution" value="Managed runtime" />
              <div className="rounded-lg bg-muted/40 p-3 text-xs leading-5 text-muted-foreground">
                <LockKeyhole className="mb-2 size-4" />
                Pack setup is validated before publication and runs only in disposable, owned
                environments.
              </div>
            </CardContent>
          </Card>
        </div>
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
