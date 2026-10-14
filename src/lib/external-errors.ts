export function readExternalError(reason: unknown): string {
  if (reason instanceof Error && reason.message.trim()) {
    return reason.message
  }
  if (typeof reason === "object" && reason !== null && "message" in reason) {
    const message = (reason as { message?: unknown }).message
    if (typeof message === "string" && message.trim()) {
      return message
    }
  }
  return "The external database operation failed."
}
