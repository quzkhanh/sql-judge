export type ExplorerRequest = {
  generation: number
  profileId: string
}

export function beginExplorerRequest(
  generation: number,
  profileId: string
): ExplorerRequest {
  return { generation: generation + 1, profileId }
}

export function isCurrentExplorerRequest(
  request: ExplorerRequest,
  generation: number,
  profileId: string | null
): boolean {
  return request.generation === generation && request.profileId === profileId
}

export function shouldApplyProfileLoad(
  loadMutationGeneration: number,
  currentMutationGeneration: number
): boolean {
  return loadMutationGeneration === currentMutationGeneration
}

export function isConnectionProfileEditorDisabled(
  desktopAvailable: boolean,
  busy: boolean
): boolean {
  return !desktopAvailable || busy
}

export function clearExplorerOutput<
  T extends { metadata: unknown; queryResult: unknown },
>(
  state: T
): Omit<T, "metadata" | "queryResult"> & {
  metadata: null
  queryResult: null
} {
  return {
    ...state,
    metadata: null,
    queryResult: null,
  }
}
