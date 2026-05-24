export const WINWS_BUSY_PREFIX = "WINWS_BUSY:";

export function splitInvokeError(e: unknown): {
  error: string | null;
  winwsBusyHint: string | null;
} {
  const msg = String(e);
  if (msg.startsWith(WINWS_BUSY_PREFIX)) {
    return {
      error: null,
      winwsBusyHint: msg.slice(WINWS_BUSY_PREFIX.length),
    };
  }
  return { error: msg, winwsBusyHint: null };
}
