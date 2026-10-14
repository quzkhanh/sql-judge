import { serializeBackupSnapshot, type BackupSnapshot } from "./judge-api"

export function serializeBackupForDownload(snapshot: BackupSnapshot): string {
  return serializeBackupSnapshot(snapshot)
}
