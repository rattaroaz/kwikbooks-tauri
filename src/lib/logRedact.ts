/** Keys (case-insensitive) and path fragments treated as sensitive for log previews. */
const REDACT_KEY_FRAGMENTS = [
  "password",
  "secret",
  "token",
  "email",
  "phone",
  "memo",
  "displayname",
  "legalname",
  "payeename",
  "taxid",
  "ssn",
  "routing",
  "accountnumber",
  "iban",
  "creditcard",
  "cardnumber",
  "cvv",
  "authorization",
  "bearer",
] as const;

function keyLooksSensitive(key: string): boolean {
  const k = key.replace(/_/g, "").toLowerCase();
  return REDACT_KEY_FRAGMENTS.some((frag) => k.includes(frag));
}

/** Returns a deep clone with sensitive string/primitive fields replaced by `[redacted]`. */
export function redactForLog(
  value: unknown,
  seen = new WeakSet<object>(),
): unknown {
  if (value === null || value === undefined) {
    return value;
  }
  if (Array.isArray(value)) {
    return value.map((x) => redactForLog(x, seen));
  }
  if (typeof value === "object") {
    const obj = value as object;
    if (seen.has(obj)) {
      return "[cycle]";
    }
    seen.add(obj);
    const out: Record<string, unknown> = {};
    for (const [key, v] of Object.entries(value as Record<string, unknown>)) {
      if (keyLooksSensitive(key)) {
        out[key] = "[redacted]";
      } else {
        out[key] = redactForLog(v, seen);
      }
    }
    return out;
  }
  return value;
}

/** Compact JSON for IPC debug logs (no raw PII when keys match redaction rules). */
export function summarizeInvokePayload(args: Record<string, unknown>): string {
  try {
    return JSON.stringify(redactForLog(args));
  } catch {
    return "[payload_not_serializable]";
  }
}
