import { describe, expect, test } from "bun:test"

import {
  backupRestoreImpactMessage,
  backupProgressMessage,
  backupSizeError,
  backupSuccessMessage,
  readBackupError,
} from "./backup-errors"
import { serializeBackupSnapshot } from "./judge-api"

describe("backup messages", () => {
  test("shows bounded progress and success messages for each operation", () => {
    expect(backupProgressMessage("export")).toBe("Creating metadata backup...")
    expect(backupProgressMessage("restore")).toBe("Validating backup...")
    expect(backupSuccessMessage("export")).toBe(
      "Metadata and submission history exported without credentials."
    )
    expect(backupSuccessMessage("restore")).toBe(
      "Backup restored. PostgreSQL schemas were not modified."
    )
    expect(backupRestoreImpactMessage()).toBe(
      "Restoring will replace local metadata and submission history. PostgreSQL schemas and secrets will not be modified."
    )
  })

  test("rejects only backup files larger than the native limit", () => {
    expect(backupSizeError(4 * 1024 * 1024)).toBeNull()
    expect(backupSizeError(4 * 1024 * 1024 + 1)).toBe(
      "Backup exceeds the 4 MiB size limit."
    )
  })

  test("serializes restore IPC payloads as raw JSON", () => {
    const snapshot = {
      version: 1,
      environments: [],
      submissions: [],
    }
    expect(serializeBackupSnapshot(snapshot)).toBe(JSON.stringify(snapshot))
  })

  test("serializes an under-limit UTF-8 snapshot", () => {
    const snapshot = {
      version: 1,
      environments: [],
      submissions: [],
      note: "🔒".repeat(100),
    }
    expect(() => serializeBackupSnapshot(snapshot)).not.toThrow()
  })

  test("rejects an oversized UTF-8 snapshot before IPC", () => {
    const snapshot = {
      version: 1,
      environments: [],
      submissions: [],
      note: "🔒".repeat(1_100_000),
    }
    expect(() => serializeBackupSnapshot(snapshot)).toThrow(
      "Metadata backup exceeds the supported size limit."
    )
  })

  test("preserves only safe native errors and hides snapshot details", () => {
    expect(
      readBackupError("Managed environment state is busy.", "restore")
    ).toBe("Managed environment state is busy.")
    expect(
      readBackupError(
        "snapshot contains password=secret and /tmp/private.json",
        "restore"
      )
    ).toBe("Backup restore failed closed. Existing data was preserved.")
    expect(readBackupError(new Error("database path=/secret"), "export")).toBe(
      "Backup export failed."
    )
  })
})
