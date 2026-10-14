export type StatementSegment = {
  kind: "text" | "strong" | "code"
  text: string
}

const INLINE = /\*\*([^*\n]+)\*\*|`([^`\n]+)`/g

// Pack statements are untrusted: only **strong** and `code` are recognised, rendered as React text nodes.
export function statementBlocks(statement: string): StatementSegment[][] {
  return statement
    .split(/\n\s*\n/)
    .map((paragraph) => paragraph.trim())
    .filter(Boolean)
    .map((paragraph) => {
      const segments: StatementSegment[] = []
      let cursor = 0
      for (const match of paragraph.matchAll(INLINE)) {
        if (match.index > cursor) {
          segments.push({ kind: "text", text: paragraph.slice(cursor, match.index) })
        }
        segments.push(
          match[1] === undefined
            ? { kind: "code", text: match[2] }
            : { kind: "strong", text: match[1] }
        )
        cursor = match.index + match[0].length
      }
      if (cursor < paragraph.length) {
        segments.push({ kind: "text", text: paragraph.slice(cursor) })
      }
      return segments
    })
}
