const SAFE_PACK_ERRORS = new Set([
  "A local pack path is required.",
  "Pack import is only available in the desktop app.",
  "Pack source could not be loaded.",
  "Pack failed structural validation.",
  "Managed environment state is busy.",
  "Managed PostgreSQL runtime could not be started.",
  "Managed PostgreSQL runtime is unavailable.",
  "Managed PostgreSQL runtime ownership could not be verified.",
  "Pack runtime validation or publication failed.",
])

const FALLBACK_PACK_ERROR =
  "Pack import failed closed. No unvalidated pack was published."

export function readPackImportError(reason: unknown): string {
  const message = nativeMessage(reason)
  return SAFE_PACK_ERRORS.has(message) ? message : FALLBACK_PACK_ERROR
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
