type BackupOperation = "export" | "restore"

export const MAX_BACKUP_BYTES = 4 * 1024 * 1024

const SAFE_BACKUP_ERRORS = new Set([
  "Managed environment state is busy.",
  "Metadata backup could not be exported.",
  "Metadata backup could not be encoded.",
  "Metadata backup exceeds the supported size limit.",
  "Metadata backup failed validation or restore.",
  "Restored environment metadata could not be read.",
  "Restored environment metadata failed policy validation.",
])

export function backupProgressMessage(operation: BackupOperation): string {
  return operation === "export"
    ? "Creating metadata backup..."
    : "Validating backup..."
}

export function backupSuccessMessage(operation: BackupOperation): string {
  return operation === "export"
    ? "Metadata and submission history exported without credentials."
    : "Backup restored. PostgreSQL schemas were not modified."
}

export function backupRestoreImpactMessage(): string {
  return "Restoring will replace local metadata and submission history. PostgreSQL schemas and secrets will not be modified."
}

export function backupSizeError(fileSize: number): string | null {
  return fileSize > MAX_BACKUP_BYTES
    ? "Backup exceeds the 4 MiB size limit."
    : null
}

export function readBackupError(
  reason: unknown,
  operation: BackupOperation
): string {
  const message = nativeMessage(reason)
  if (SAFE_BACKUP_ERRORS.has(message)) {
    return message
  }
  return operation === "export"
    ? "Backup export failed."
    : "Backup restore failed closed. Existing data was preserved."
}

function nativeMessage(reason: unknown): string {
  if (reason instanceof Error) {
    return reason.message
  }
  if (typeof reason === "string") {
    return reason
  }
  if (typeof reason === "object" && reason !== null && "message" in reason) {
    const message = (reason as { message?: unknown }).message
    return typeof message === "string" ? message : ""
  }
  return ""
}
