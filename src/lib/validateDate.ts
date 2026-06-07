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
