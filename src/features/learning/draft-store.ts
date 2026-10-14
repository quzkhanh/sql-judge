export function createDraftStore(maxLength: number) {
  const drafts = new Map<string, string>()
  return {
    get(problemId: string, fallback: string) {
      return drafts.get(problemId) ?? fallback
    },
    set(problemId: string, value: string) {
      if (problemId) {
        drafts.set(problemId, value.slice(0, maxLength))
      }
    },
  }
}
