const SAFE_CONNECTION_ERRORS = new Set([
  "Connection profile could not be read.",
  "Connection profile could not be saved.",
  "The connection profile is invalid.",
  "The connection secret cannot be empty.",
  "Credential vault is unavailable.",
  "Managed environment state is busy.",
])

export function readConnectionProfileError(reason: unknown): string {
  const message =
    reason instanceof Error
      ? reason.message
      : typeof reason === "string"
        ? reason
        : typeof reason === "object" && reason !== null && "message" in reason
          ? typeof (reason as { message?: unknown }).message === "string"
            ? (reason as { message: string }).message
            : ""
          : ""
  return SAFE_CONNECTION_ERRORS.has(message)
    ? message
    : "Connection profile could not be saved."
}
