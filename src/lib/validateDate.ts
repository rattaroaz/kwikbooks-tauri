import { isValidISODate } from "./dates";

/** Returns an error message suitable for toasts, or `null` when valid. */
export function requireValidISODate(
  label: string,
  value: string,
): string | null {
  const trimmed = value.trim();
  if (!isValidISODate(trimmed)) {
    return `${label} must be a valid date (YYYY-MM-DD).`;
  }
  return null;
}

/** Returns an error when either date is invalid or `from` is after `to`. */
export function requireValidISODateRange(
  fromLabel: string,
  from: string,
  toLabel: string,
  to: string,
): string | null {
  const fromErr = requireValidISODate(fromLabel, from);
  if (fromErr) {
    return fromErr;
  }
  const toErr = requireValidISODate(toLabel, to);
  if (toErr) {
    return toErr;
  }
  if (from.trim() > to.trim()) {
    return `${fromLabel} must be on or before ${toLabel}.`;
  }
  return null;
}
