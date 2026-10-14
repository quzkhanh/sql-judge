import { describe, expect, test } from "bun:test"

import { MAX_BACKUP_BYTES } from "./backup-errors"
import { serializeBackupForDownload } from "./backup-export"

describe("backup export", () => {
  test("keeps a near-limit export compact and import-sized", () => {
    const snapshot = {
      version: 1,
      environments: [],
      submissions: [
        {
          submissionId: "near-limit",
          packId: "hr-sql-101",
          packVersion: "1.0.0",
          packHash: "x".repeat(MAX_BACKUP_BYTES - 512),
          engineVersion: "postgresql-managed",
          policyVersion: "policy-v1",
          verdict: "Accepted",
          createdAt: "2026-10-12T00:00:00Z",
          tests: [],
        },
      ],
    }
    const serialized = serializeBackupForDownload(snapshot)
    const bytes = new TextEncoder().encode(serialized).byteLength
    expect(bytes).toBeLessThanOrEqual(MAX_BACKUP_BYTES)
    expect(bytes).toBeGreaterThan(MAX_BACKUP_BYTES - 1024)
    expect(serialized).not.toContain("\n")
  })
})
