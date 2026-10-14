import {
  useEffect,
  useRef,
  useState,
  type FormEvent,
  type ReactNode,
} from "react"
import {
  Cable,
  Check,
  Database,
  KeyRound,
  LockKeyhole,
  Play,
  Plus,
  RefreshCw,
  ShieldCheck,
  Table2,
  Trash2,
} from "lucide-react"

import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import {
  deleteConnectionProfile,
  executeExternalSelect,
  executeExternalSqlServerSelect,
  getManagedRuntimeStatus,
  listExternalMetadata,
  listExternalSqlServerMetadata,
  listConnectionProfiles,
  saveConnectionProfile,
  startManagedRuntime,
  testExternalConnection,
  testExternalSqlServer,
  type ConnectionProfileSummary,
  type ManagedRuntimeStatus,
  type ExternalConnectionInfo,
  type ExternalMetadata,
  type ExternalQueryResult,
  type SaveConnectionProfileRequest,
} from "@/lib/judge-api"
import {
  normalizeTlsMode,
  parseConnectionUrl,
  supportedTlsModes,
  suggestConnectionProfile,
} from "@/lib/connection-url"
import { readConnectionProfileError } from "@/lib/connection-errors"
import { readExternalError } from "@/lib/external-errors"
import { isReadOnlyVerified, readOnlyStatusLabel } from "@/lib/external-access"
import {
  beginExplorerRequest,
  clearExplorerOutput,
  isCurrentExplorerRequest,
  type ExplorerRequest,
} from "./connections-state"
import { learningLoadErrorMessage, learningLoadPhase } from "./load-state"

const EMPTY_DRAFT: SaveConnectionProfileRequest = {
  profileId: "",
  displayName: "",
  engine: "postgresql",
  host: "",
  port: 5432,
  database: "",
  username: "",
  tlsMode: "verify_full",
  secret: "",
}

const SAMPLE_QUERIES = {
  postgresql: "SELECT * FROM public.sample LIMIT 50",
  sqlserver: "SELECT TOP (50) * FROM dbo.sample",
} as const

export function ConnectionsScreen() {
  const [status, setStatus] = useState<ManagedRuntimeStatus | null>(null)
  const [profiles, setProfiles] = useState<ConnectionProfileSummary[]>([])
  const [profilesLoading, setProfilesLoading] = useState(true)
  const [profilesError, setProfilesError] = useState<string | null>(null)
  const [profilesReloadToken, setProfilesReloadToken] = useState(0)
  const [draft, setDraft] = useState<SaveConnectionProfileRequest>(EMPTY_DRAFT)
  const [connectionUrl, setConnectionUrl] = useState("")
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [selectedProfileId, setSelectedProfileId] = useState<string | null>(
    null
  )
  const [connectionInfo, setConnectionInfo] =
    useState<ExternalConnectionInfo | null>(null)
  const [metadata, setMetadata] = useState<ExternalMetadata | null>(null)
  const [query, setQuery] = useState<string>(SAMPLE_QUERIES.postgresql)
  const [queryResult, setQueryResult] = useState<ExternalQueryResult | null>(
    null
  )
  const [explorerBusy, setExplorerBusy] = useState(false)
  const [explorerError, setExplorerError] = useState<string | null>(null)
  const [connectionTestBusy, setConnectionTestBusy] = useState(false)
  const [connectionTestError, setConnectionTestError] = useState<string | null>(
    null
  )
  const selectedProfileIdRef = useRef<string | null>(null)
  const explorerGenerationRef = useRef(0)

  function clearExplorerData() {
    const cleared = clearExplorerOutput({ metadata, queryResult })
    setMetadata(cleared.metadata)
    setQueryResult(cleared.queryResult)
  }

  function selectProfile(profileId: string | null) {
    selectedProfileIdRef.current = profileId
    setSelectedProfileId(profileId)
  }

  function invalidateExplorerRequests() {
    explorerGenerationRef.current += 1
    setExplorerBusy(false)
    setConnectionTestBusy(false)
    clearExplorerData()
  }

  function retryProfileLoad() {
    setProfilesLoading(true)
    setProfilesError(null)
    invalidateExplorerRequests()
    selectProfile(null)
    setConnectionInfo(null)
    setMetadata(null)
    setQueryResult(null)
    setConnectionTestError(null)
    setExplorerError(null)
    setProfilesReloadToken((current) => current + 1)
  }

  useEffect(() => {
    let active = true
    void Promise.all([getManagedRuntimeStatus(), listConnectionProfiles()])
      .then(([runtime, connectionProfiles]) => {
        if (active) {
          setStatus(runtime)
          setProfiles(connectionProfiles)
        }
      })
      .catch(() => {
        if (active) {
          setStatus(null)
          setProfiles([])
          explorerGenerationRef.current += 1
          selectedProfileIdRef.current = null
          setSelectedProfileId(null)
          setConnectionInfo(null)
          setMetadata(null)
          setQueryResult(null)
          setProfilesError(learningLoadErrorMessage("profiles"))
        }
      })
      .finally(() => {
        if (active) {
          setProfilesLoading(false)
        }
      })
    return () => {
      active = false
    }
  }, [profilesReloadToken])

  async function start() {
    setBusy(true)
    setError(null)
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

  async function saveProfile(event: FormEvent<HTMLFormElement>) {
    event.preventDefault()
    setBusy(true)
    setError(null)
    try {
      const saved = await saveConnectionProfile({
        ...draft,
        secret: draft.secret?.trim() ? draft.secret : undefined,
      })
      setProfiles((current) =>
        [
          ...current.filter((profile) => profile.profileId !== saved.profileId),
          saved,
        ].sort((left, right) => left.profileId.localeCompare(right.profileId))
      )
      invalidateExplorerRequests()
      selectProfile(saved.profileId)
      setConnectionInfo(null)
      setMetadata(null)
      setQueryResult(null)
      setExplorerError(null)
      setConnectionTestError(null)
      setDraft((current) => ({ ...current, secret: "" }))
    } catch (reason) {
      setError(readConnectionProfileError(reason))
    } finally {
      setBusy(false)
    }
  }

  function importConnectionUrl() {
    try {
      const parsed = parseConnectionUrl(connectionUrl)
      const suggestions = suggestConnectionProfile(parsed)
      setDraft((current) => ({
        ...current,
        ...parsed,
        profileId: current.profileId || suggestions.profileId,
        displayName: current.displayName || suggestions.displayName,
      }))
      setQuery(SAMPLE_QUERIES[parsed.engine])
      setConnectionUrl("")
      setError(null)
    } catch (reason) {
      setError(
        reason instanceof Error
          ? reason.message
          : "Connection URL could not be parsed."
      )
    }
  }

  async function removeProfile(profile: ConnectionProfileSummary) {
    if (
      !window.confirm(`Delete connection profile "${profile.displayName}"?`)
    ) {
      return
    }
    setBusy(true)
    setError(null)
    try {
      await deleteConnectionProfile(profile.profileId)
      setProfiles((current) =>
        current.filter((item) => item.profileId !== profile.profileId)
      )
      if (draft.profileId === profile.profileId) {
        setDraft(EMPTY_DRAFT)
        setConnectionUrl("")
      }
      if (selectedProfileId === profile.profileId) {
        invalidateExplorerRequests()
        selectProfile(null)
        setConnectionInfo(null)
        setMetadata(null)
        setQueryResult(null)
        setExplorerError(null)
        setConnectionTestError(null)
      }
    } catch (reason) {
      setError(
        reason instanceof Error
          ? reason.message
          : "Profile could not be deleted."
      )
    } finally {
      setBusy(false)
    }
  }

  function editProfile(profile: ConnectionProfileSummary) {
    setDraft({
      profileId: profile.profileId,
      displayName: profile.displayName,
      engine: profile.engine,
      host: profile.host,
      port: profile.port,
      database: profile.database,
      username: profile.username,
      tlsMode: normalizeTlsMode(profile.engine, profile.tlsMode),
      secret: "",
    })
    setConnectionUrl("")
    setQuery(SAMPLE_QUERIES[profile.engine])
    invalidateExplorerRequests()
    selectProfile(profile.profileId)
    setConnectionInfo(null)
    setMetadata(null)
    setQueryResult(null)
    setExplorerError(null)
    setConnectionTestError(null)
    setError(null)
  }

  const selectedProfile = profiles.find(
    (profile) => profile.profileId === selectedProfileId
  )
  const profilesPhase = learningLoadPhase(
    profilesLoading,
    profilesError,
    profiles.length > 0
  )
  const connectionTestPhase = learningLoadPhase(
    connectionTestBusy,
    connectionTestError,
    connectionInfo !== null
  )

  async function testExplorerConnection() {
    if (!selectedProfileId || !selectedProfile) {
      return
    }
    const profileId = selectedProfileId
    const request: ExplorerRequest = beginExplorerRequest(
      explorerGenerationRef.current,
      profileId
    )
    explorerGenerationRef.current = request.generation
    setConnectionTestBusy(true)
    setConnectionInfo(null)
    setMetadata(null)
    setQueryResult(null)
    setConnectionTestError(null)
    setExplorerError(null)
    try {
      const info = await (selectedProfile.engine === "sqlserver"
        ? testExternalSqlServer(profileId)
        : testExternalConnection(profileId))
      if (
        isCurrentExplorerRequest(
          request,
          explorerGenerationRef.current,
          selectedProfileIdRef.current
        )
      ) {
        setConnectionInfo(info)
        setMetadata(null)
        setQueryResult(null)
        if (!isReadOnlyVerified(info.readOnly)) {
          setConnectionTestError(
            "The backend did not verify a read-only connection. Metadata and SELECT actions are disabled."
          )
        }
      }
    } catch (reason) {
      if (
        isCurrentExplorerRequest(
          request,
          explorerGenerationRef.current,
          selectedProfileIdRef.current
        )
      ) {
        setConnectionTestError(readExternalError(reason))
      }
    } finally {
      if (
        isCurrentExplorerRequest(
          request,
          explorerGenerationRef.current,
          selectedProfileIdRef.current
        )
      ) {
        setConnectionTestBusy(false)
      }
    }
  }

  async function loadExplorerMetadata() {
    if (!selectedProfileId || !selectedProfile) {
      return
    }
    if (!isReadOnlyVerified(connectionInfo?.readOnly)) {
      setExplorerError(
        "Metadata is unavailable until the backend verifies read-only access."
      )
      return
    }
    const profileId = selectedProfileId
    const request: ExplorerRequest = beginExplorerRequest(
      explorerGenerationRef.current,
      profileId
    )
    explorerGenerationRef.current = request.generation
    setExplorerBusy(true)
    setExplorerError(null)
    clearExplorerData()
    try {
      const result = await (selectedProfile.engine === "sqlserver"
        ? listExternalSqlServerMetadata(profileId)
        : listExternalMetadata(profileId))
      if (
        !isCurrentExplorerRequest(
          request,
          explorerGenerationRef.current,
          selectedProfileIdRef.current
        )
      ) {
        return
      }
      setMetadata(result)
    } catch (reason) {
      if (
        isCurrentExplorerRequest(
          request,
          explorerGenerationRef.current,
          selectedProfileIdRef.current
        )
      ) {
        clearExplorerData()
        setExplorerError(readExternalError(reason))
      }
    } finally {
      if (
        isCurrentExplorerRequest(
          request,
          explorerGenerationRef.current,
          selectedProfileIdRef.current
        )
      ) {
        setExplorerBusy(false)
      }
    }
  }

  async function runExplorerQuery(event: FormEvent<HTMLFormElement>) {
    event.preventDefault()
    if (!selectedProfileId || !selectedProfile) {
      return
    }
    if (!isReadOnlyVerified(connectionInfo?.readOnly)) {
      setExplorerError(
        "SELECT is unavailable until the backend verifies read-only access."
      )
      return
    }
    const profileId = selectedProfileId
    const request: ExplorerRequest = beginExplorerRequest(
      explorerGenerationRef.current,
      profileId
    )
    explorerGenerationRef.current = request.generation
    setExplorerBusy(true)
    setExplorerError(null)
    clearExplorerData()
    try {
      const result = await (selectedProfile.engine === "sqlserver"
        ? executeExternalSqlServerSelect({
            profileId,
            sql: query,
            statementTimeoutMs: 3_000,
            maxRows: 100,
            maxBytes: 256 * 1024,
          })
        : executeExternalSelect({
            profileId,
            sql: query,
            statementTimeoutMs: 3_000,
            maxRows: 100,
            maxBytes: 256 * 1024,
          }))
      if (
        !isCurrentExplorerRequest(
          request,
          explorerGenerationRef.current,
          selectedProfileIdRef.current
        )
      ) {
        return
      }
      setQueryResult(result)
    } catch (reason) {
      if (
        isCurrentExplorerRequest(
          request,
          explorerGenerationRef.current,
          selectedProfileIdRef.current
        )
      ) {
        clearExplorerData()
        setExplorerError(readExternalError(reason))
      }
    } finally {
      if (
        isCurrentExplorerRequest(
          request,
          explorerGenerationRef.current,
          selectedProfileIdRef.current
        )
      ) {
        setExplorerBusy(false)
      }
    }
  }

  return (
    <main className="min-h-0 flex-1 overflow-y-auto p-4 lg:p-6">
      <div className="mx-auto flex max-w-6xl flex-col gap-6">
        <header className="border-b pb-5">
          <p className="text-xs font-semibold tracking-[0.18em] text-primary uppercase">
            Infrastructure
          </p>
          <h2 className="mt-1 text-2xl font-semibold tracking-tight">
            Connections
          </h2>
          <p className="mt-1 max-w-2xl text-sm text-muted-foreground">
            Save read-only explorer profiles without putting credentials in app
            storage or backup files.
          </p>
        </header>

        {error ? (
          <div
            role="alert"
            className="rounded-lg border border-destructive/30 bg-destructive/10 p-3 text-sm text-destructive"
          >
            {error}
          </div>
        ) : null}

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
                      {status?.available
                        ? "Connected and ready"
                        : "Not connected"}
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
                <Capability
                  icon={LockKeyhole}
                  label="Runner role constrained"
                />
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
                External profiles are Explorer-only. They never receive managed
                runner or reset permissions.
              </p>
              <p>
                Passwords are stored in the OS credential vault when available
                and never returned to the UI.
              </p>
            </CardContent>
          </Card>
        </div>

        <div className="grid gap-4 lg:grid-cols-[minmax(0,0.85fr)_minmax(0,1.15fr)]">
          <Card>
            <CardHeader className="flex-row items-center justify-between">
              <CardTitle className="text-base">Saved profiles</CardTitle>
              <Button
                size="sm"
                variant="outline"
                onClick={() => {
                  setDraft(EMPTY_DRAFT)
                  setConnectionUrl("")
                  setQuery(SAMPLE_QUERIES.postgresql)
                  invalidateExplorerRequests()
                  selectProfile(null)
                  setConnectionInfo(null)
                  setMetadata(null)
                  setQueryResult(null)
                  setExplorerError(null)
                  setConnectionTestError(null)
                }}
              >
                <Plus /> New
              </Button>
            </CardHeader>
            <CardContent className="space-y-2">
              {profilesPhase === "loading" ? (
                <p
                  className="rounded-lg border border-dashed p-4 text-sm text-muted-foreground"
                  role="status"
                >
                  Loading saved connection profiles...
                </p>
              ) : profilesPhase === "error" ? (
                <div
                  className="rounded-lg border border-destructive/30 bg-destructive/5 p-4"
                  role="alert"
                >
                  <p className="text-sm text-destructive">{profilesError}</p>
                  <Button
                    className="mt-3"
                    variant="outline"
                    size="sm"
                    onClick={retryProfileLoad}
                  >
                    <RefreshCw />
                    Retry
                  </Button>
                </div>
              ) : profiles.length === 0 ? (
                <p className="rounded-lg border border-dashed p-4 text-sm text-muted-foreground">
                  No external profiles yet.
                </p>
              ) : (
                profiles.map((profile) => (
                  <div
                    key={profile.profileId}
                    className="flex items-center justify-between gap-3 rounded-lg border p-3"
                  >
                    <button
                      type="button"
                      className="min-w-0 flex-1 text-left"
                      onClick={() => editProfile(profile)}
                      aria-pressed={selectedProfileId === profile.profileId}
                    >
                      <p className="truncate font-medium">
                        {profile.displayName}
                      </p>
                      <p className="truncate text-xs text-muted-foreground">
                        {profile.username}@{profile.host}:{profile.port}/
                        {profile.database}
                      </p>
                    </button>
                    <div className="flex items-center gap-2">
                      {profile.hasSecret ? (
                        <KeyRound
                          className="size-4 text-emerald-600"
                          aria-label="Credential stored"
                        />
                      ) : null}
                      <Button
                        size="icon-xs"
                        variant="ghost"
                        aria-label={`Delete ${profile.displayName}`}
                        onClick={() => void removeProfile(profile)}
                        disabled={busy}
                      >
                        <Trash2 />
                      </Button>
                    </div>
                  </div>
                ))
              )}
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle className="text-base">Profile details</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="mb-5 rounded-lg border bg-muted/20 p-3">
                <Label htmlFor="connection-url">Import connection URL</Label>
                <div className="mt-2 flex flex-col gap-2 sm:flex-row">
                  <Input
                    id="connection-url"
                    type="text"
                    inputMode="url"
                    autoComplete="off"
                    value={connectionUrl}
                    onChange={(event) => setConnectionUrl(event.target.value)}
                    placeholder="postgresql://user:password@host:5432/database"
                    disabled={busy}
                  />
                  <Button
                    type="button"
                    variant="outline"
                    onClick={importConnectionUrl}
                    disabled={busy || !connectionUrl.trim()}
                  >
                    Fill fields
                  </Button>
                </div>
                <p className="mt-2 text-xs text-muted-foreground">
                  Parsed locally. The URL is not stored or written to debug
                  logs.
                </p>
              </div>
              <form
                className="grid gap-4 sm:grid-cols-2"
                onSubmit={(event) => void saveProfile(event)}
              >
                <Field label="Profile ID" htmlFor="profile-id">
                  <Input
                    id="profile-id"
                    required
                    value={draft.profileId}
                    onChange={(event) =>
                      setDraft({ ...draft, profileId: event.target.value })
                    }
                  />
                </Field>
                <Field label="Display name" htmlFor="display-name">
                  <Input
                    id="display-name"
                    required
                    value={draft.displayName}
                    onChange={(event) =>
                      setDraft({ ...draft, displayName: event.target.value })
                    }
                  />
                </Field>
                <Field label="Engine" htmlFor="engine">
                  <select
                    id="engine"
                    className="h-8 w-full rounded-lg border border-input bg-transparent px-2.5 text-sm"
                    value={draft.engine}
                    onChange={(event) =>
                      (() => {
                        const engine = event.target
                          .value as SaveConnectionProfileRequest["engine"]
                        setDraft({
                          ...draft,
                          engine,
                          port: engine === "sqlserver" ? 1433 : 5432,
                          tlsMode: normalizeTlsMode(engine, draft.tlsMode),
                        })
                        setQuery(SAMPLE_QUERIES[engine])
                      })()
                    }
                  >
                    <option value="postgresql">PostgreSQL</option>
                    <option value="sqlserver">SQL Server</option>
                  </select>
                </Field>
                <Field label="TLS policy" htmlFor="tls-mode">
                  <select
                    id="tls-mode"
                    className="h-8 w-full rounded-lg border border-input bg-transparent px-2.5 text-sm"
                    value={draft.tlsMode}
                    onChange={(event) =>
                      setDraft({
                        ...draft,
                        tlsMode: event.target
                          .value as SaveConnectionProfileRequest["tlsMode"],
                      })
                    }
                  >
                    {supportedTlsModes(draft.engine).map((mode) => (
                      <option key={mode} value={mode}>
                        {mode === "verify_full"
                          ? "Verify full"
                          : mode === "verify_ca"
                            ? "Verify CA"
                            : "Require TLS"}
                      </option>
                    ))}
                  </select>
                </Field>
                <Field label="Host" htmlFor="host">
                  <Input
                    id="host"
                    required
                    value={draft.host}
                    onChange={(event) =>
                      setDraft({ ...draft, host: event.target.value })
                    }
                  />
                </Field>
                <Field label="Port" htmlFor="port">
                  <Input
                    id="port"
                    required
                    min={1}
                    max={65535}
                    type="number"
                    value={draft.port}
                    onChange={(event) =>
                      setDraft({ ...draft, port: Number(event.target.value) })
                    }
                  />
                </Field>
                <Field label="Database" htmlFor="database">
                  <Input
                    id="database"
                    required
                    value={draft.database}
                    onChange={(event) =>
                      setDraft({ ...draft, database: event.target.value })
                    }
                  />
                </Field>
                <Field label="Username" htmlFor="username">
                  <Input
                    id="username"
                    required
                    value={draft.username}
                    onChange={(event) =>
                      setDraft({ ...draft, username: event.target.value })
                    }
                  />
                </Field>
                <Field
                  label="Password or token (optional on update)"
                  htmlFor="secret"
                  className="sm:col-span-2"
                >
                  <Input
                    id="secret"
                    type="password"
                    autoComplete="new-password"
                    value={draft.secret ?? ""}
                    onChange={(event) =>
                      setDraft({ ...draft, secret: event.target.value })
                    }
                  />
                </Field>
                <div className="flex items-center justify-between gap-3 border-t pt-4 sm:col-span-2">
                  <p className="text-xs text-muted-foreground">
                    Secrets go directly to the native vault and are never backed
                    up.
                  </p>
                  <Button type="submit" disabled={busy}>
                    Save profile
                  </Button>
                </div>
              </form>
            </CardContent>
          </Card>
        </div>

        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2 text-base">
              <Database className="size-4" /> External Explorer
            </CardTitle>
            <p className="text-sm text-muted-foreground">
              Select a saved profile, then explicitly test before loading
              metadata or running a bounded read-only SELECT.
            </p>
          </CardHeader>
          <CardContent className="space-y-4">
            {!selectedProfile ? (
              <p className="rounded-lg border border-dashed p-4 text-sm text-muted-foreground">
                Choose a saved profile to begin. External connections are never
                probed automatically.
              </p>
            ) : (
              <>
                <div className="flex flex-wrap items-center gap-2">
                  <Button
                    onClick={() => void testExplorerConnection()}
                    disabled={explorerBusy || connectionTestBusy}
                  >
                    <ShieldCheck /> Test connection
                  </Button>
                  <Button
                    variant="outline"
                    onClick={() => void loadExplorerMetadata()}
                    disabled={
                      explorerBusy ||
                      connectionTestBusy ||
                      !isReadOnlyVerified(connectionInfo?.readOnly)
                    }
                  >
                    <RefreshCw /> Load metadata
                  </Button>
                  {connectionInfo ? (
                    <Badge
                      variant={
                        isReadOnlyVerified(connectionInfo.readOnly)
                          ? "default"
                          : "destructive"
                      }
                    >
                      {readOnlyStatusLabel(connectionInfo.readOnly)}
                    </Badge>
                  ) : null}
                </div>
                {connectionTestPhase === "loading" ? (
                  <p
                    className="rounded-lg border border-dashed p-3 text-sm text-muted-foreground"
                    role="status"
                  >
                    Testing the connection and verifying effective read-only
                    access...
                  </p>
                ) : connectionTestPhase === "error" ? (
                  <div
                    role="alert"
                    className="rounded-lg border border-destructive/30 bg-destructive/10 p-3 text-sm text-destructive"
                  >
                    <p>{connectionTestError}</p>
                    <Button
                      className="mt-3"
                      variant="outline"
                      size="sm"
                      onClick={() => void testExplorerConnection()}
                    >
                      <RefreshCw />
                      Retry test
                    </Button>
                  </div>
                ) : connectionTestPhase === "empty" ? (
                  <p
                    className="rounded-lg border border-dashed p-3 text-sm text-muted-foreground"
                    role="status"
                  >
                    Test the connection to verify read-only access before
                    loading metadata or running a SELECT.
                  </p>
                ) : null}
                {connectionInfo && connectionTestPhase === "ready" ? (
                  <div className="grid gap-3 rounded-lg border bg-muted/20 p-3 text-sm sm:grid-cols-3">
                    <span>
                      <strong>Database:</strong> {connectionInfo.database}
                    </span>
                    <span>
                      <strong>User:</strong> {connectionInfo.username}
                    </span>
                    <span>
                      <strong>Server:</strong> {connectionInfo.serverVersion}
                    </span>
                  </div>
                ) : null}
                {explorerError ? (
                  <div
                    role="alert"
                    className="rounded-lg border border-destructive/30 bg-destructive/10 p-3 text-sm text-destructive"
                  >
                    {explorerError}
                  </div>
                ) : null}
                {metadata ? <MetadataPanel metadata={metadata} /> : null}
                <form
                  className="space-y-2 border-t pt-4"
                  onSubmit={(event) => void runExplorerQuery(event)}
                >
                  <Field
                    label={`Read-only ${selectedProfile.engine === "sqlserver" ? "T-SQL SELECT" : "SELECT"}`}
                    htmlFor="external-sql"
                  >
                    <textarea
                      id="external-sql"
                      className="min-h-28 w-full rounded-lg border border-input bg-transparent p-2.5 font-mono text-sm outline-none focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50"
                      value={query}
                      onChange={(event) => setQuery(event.target.value)}
                      spellCheck={false}
                    />
                  </Field>
                  <div className="flex items-center justify-between gap-3">
                    <p className="text-xs text-muted-foreground">
                      One SELECT, 100 rows, 256 KiB, 3 seconds.
                    </p>
                    <Button
                      type="submit"
                      disabled={
                        explorerBusy ||
                        !isReadOnlyVerified(connectionInfo?.readOnly) ||
                        !query.trim()
                      }
                    >
                      <Play /> Run SELECT
                    </Button>
                  </div>
                </form>
                {queryResult ? <QueryResultPanel result={queryResult} /> : null}
              </>
            )}
          </CardContent>
        </Card>
      </div>
    </main>
  )
}

function Field({
  label,
  htmlFor,
  className,
  children,
}: {
  label: string
  htmlFor: string
  className?: string
  children: ReactNode
}) {
  return (
    <div className={`grid gap-1.5 ${className ?? ""}`}>
      <Label htmlFor={htmlFor}>{label}</Label>
      {children}
    </div>
  )
}

function MetadataPanel({ metadata }: { metadata: ExternalMetadata }) {
  return (
    <section aria-labelledby="metadata-heading" className="space-y-2">
      <div className="flex items-center justify-between gap-3">
        <h3 id="metadata-heading" className="font-medium">
          Database metadata
        </h3>
        {metadata.truncated ? <Badge variant="outline">limited</Badge> : null}
      </div>
      {metadata.tables.length === 0 ? (
        <p className="rounded-lg border border-dashed p-4 text-sm text-muted-foreground">
          No visible tables were returned.
        </p>
      ) : (
        <div className="grid gap-2 sm:grid-cols-2">
          {metadata.tables.map((table) => (
            <div
              key={`${table.schema}.${table.name}`}
              className="rounded-lg border p-3"
            >
              <p className="flex items-center gap-2 text-sm font-medium">
                <Table2 className="size-4" />
                {table.schema}.{table.name}
              </p>
              <ul className="mt-2 space-y-1 text-xs text-muted-foreground">
                {table.columns.map((column) => (
                  <li key={column.name}>
                    {column.name}: {column.dataType}
                  </li>
                ))}
              </ul>
            </div>
          ))}
        </div>
      )}
    </section>
  )
}

function QueryResultPanel({ result }: { result: ExternalQueryResult }) {
  return (
    <section
      aria-labelledby="external-result-heading"
      className="space-y-2 border-t pt-4"
    >
      <h3 id="external-result-heading" className="font-medium">
        SELECT result
      </h3>
      <div className="overflow-x-auto rounded-lg border">
        <table className="w-full text-left text-sm">
          <thead className="bg-muted/50 text-xs uppercase">
            <tr>
              {result.columns.map((column) => (
                <th key={column.name} className="px-3 py-2 font-medium">
                  {column.name}
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {result.rows.map((row, rowIndex) => (
              <tr key={rowIndex} className="border-t">
                {row.map((value, columnIndex) => (
                  <td
                    key={`${rowIndex}-${columnIndex}`}
                    className="px-3 py-2 align-top"
                  >
                    {value === null ? (
                      <span className="text-muted-foreground">NULL</span>
                    ) : (
                      String(value)
                    )}
                  </td>
                ))}
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      {result.rows.length === 0 ? (
        <p className="text-sm text-muted-foreground">
          The query returned no rows.
        </p>
      ) : null}
    </section>
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
