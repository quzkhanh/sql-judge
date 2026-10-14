export type DraftPersistence = {
  enabled: () => boolean
  read: () => string | null
  write: (value: string | null) => void
}

function parseDrafts(raw: string | null) {
  let value: unknown
  try {
    value = raw ? JSON.parse(raw) : null
  } catch {
    return []
  }
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    return []
  }
  return Object.entries(value).filter(
    (entry): entry is [string, string] => typeof entry[1] === "string"
  )
}

export function createDraftStore(maxLength: number, persistence?: DraftPersistence) {
  const drafts = new Map<string, string>(
    persistence?.enabled() ? parseDrafts(persistence.read()) : []
  )
  return {
    get(problemId: string, fallback: string) {
      return drafts.get(problemId) ?? fallback
    },
    set(problemId: string, value: string) {
      if (problemId) {
        drafts.set(problemId, value.slice(0, maxLength))
        persistence?.write(
          persistence.enabled() ? JSON.stringify(Object.fromEntries(drafts)) : null
        )
      }
    },
  }
}

const STORAGE_KEY = "sql-judge-drafts"

export function localDraftPersistence(enabled: () => boolean): DraftPersistence {
  return {
    enabled,
    read() {
      try {
        return window.localStorage.getItem(STORAGE_KEY)
      } catch {
        return null
      }
    },
    write(value) {
      try {
        if (value === null) {
          window.localStorage.removeItem(STORAGE_KEY)
        } else {
          window.localStorage.setItem(STORAGE_KEY, value)
        }
      } catch {
        // Storage can be unavailable; drafts then live only for this session.
      }
    },
  }
}
