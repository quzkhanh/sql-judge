import { useRef, useState } from "react"
import { Download, Upload } from "lucide-react"

import { AccessibleModal } from "@/components/ui/accessible-modal"
import { Button } from "@/components/ui/button"
import {
  exportBackup,
  restoreBackup,
  type BackupSnapshot,
} from "@/lib/judge-api"
import { serializeBackupForDownload } from "@/lib/backup-export"
import {
  backupProgressMessage,
  backupRestoreImpactMessage,
  backupSizeError,
  backupSuccessMessage,
  readBackupError,
} from "@/lib/backup-errors"

export function BackupActions({ onRestored }: { onRestored: () => void }) {
  const inputRef = useRef<HTMLInputElement>(null)
  const restoreButtonRef = useRef<HTMLButtonElement>(null)
  const [busy, setBusy] = useState(false)
  const [message, setMessage] = useState<string | null>(null)
  const [pendingFile, setPendingFile] = useState<File | null>(null)

  function clearFileInput() {
    if (inputRef.current) {
      inputRef.current.value = ""
    }
  }

  async function handleExport() {
    setBusy(true)
    setMessage(backupProgressMessage("export"))
    try {
      const snapshot = await exportBackup()
      const blob = new Blob([serializeBackupForDownload(snapshot)], {
        type: "application/json",
      })
      const url = URL.createObjectURL(blob)
      const link = document.createElement("a")
      link.href = url
      link.download = "sql-judge-backup.json"
      link.click()
      URL.revokeObjectURL(url)
      setMessage(backupSuccessMessage("export"))
    } catch (reason) {
      setMessage(readBackupError(reason, "export"))
    } finally {
      setBusy(false)
    }
  }

  function handleImport(file: File | undefined) {
    if (!file) {
      return
    }
    const sizeError = backupSizeError(file.size)
    if (sizeError) {
      setMessage(sizeError)
      clearFileInput()
      return
    }
    setPendingFile(file)
    setMessage(null)
    clearFileInput()
  }

  async function confirmRestore() {
    if (!pendingFile) {
      return
    }
    setBusy(true)
    setMessage(backupProgressMessage("restore"))
    try {
      const snapshot = JSON.parse(await pendingFile.text()) as BackupSnapshot
      await restoreBackup(snapshot)
      setMessage(backupSuccessMessage("restore"))
      setPendingFile(null)
      onRestored()
    } catch (reason) {
      setMessage(readBackupError(reason, "restore"))
      setPendingFile(null)
    } finally {
      setBusy(false)
      clearFileInput()
    }
  }

  return (
    <div className="relative">
      <div
        className="flex flex-wrap items-center justify-end gap-2"
        aria-hidden={pendingFile ? true : undefined}
        inert={pendingFile ? true : undefined}
      >
        <input
          ref={inputRef}
          type="file"
          accept="application/json,.json"
          className="sr-only"
          onChange={(event) => void handleImport(event.target.files?.[0])}
          disabled={busy || pendingFile !== null}
        />
        <Button
          ref={restoreButtonRef}
          variant="outline"
          size="sm"
          onClick={() => inputRef.current?.click()}
          disabled={busy || pendingFile !== null}
        >
          <Upload />
          Restore backup
        </Button>
        <Button
          variant="outline"
          size="sm"
          onClick={() => void handleExport()}
          disabled={busy || pendingFile !== null}
        >
          <Download />
          Export backup
        </Button>
      </div>
      {message ? (
        <p
          className="mt-2 text-right text-xs text-muted-foreground"
          role="status"
        >
          {message}
        </p>
      ) : null}
      {pendingFile ? (
        <AccessibleModal
          open
          title="Confirm backup restore"
          description={backupRestoreImpactMessage()}
          restoreFocusRef={restoreButtonRef}
          onCancel={() => setPendingFile(null)}
          busy={busy}
        >
          <p
            className="mt-2 truncate font-mono text-xs"
            title={pendingFile.name}
          >
            {pendingFile.name}
          </p>
          <div className="mt-3 flex justify-end gap-2">
            <Button
              variant="outline"
              size="sm"
              onClick={() => setPendingFile(null)}
              disabled={busy}
            >
              Cancel
            </Button>
            <Button
              variant="destructive"
              size="sm"
              onClick={() => void confirmRestore()}
              disabled={busy}
            >
              Confirm restore
            </Button>
          </div>
        </AccessibleModal>
      ) : null}
    </div>
  )
}
