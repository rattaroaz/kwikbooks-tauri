/**
 * All monetary amounts in the database are **integer minor units** (e.g. cents).
 * Use `bigint` for sums and products; use `formatMoneyMinor` only for **display** (uses `Intl`, not for accounting logic).
 */

export function parseMinorInt(raw: string): number {
  const t = raw.trim().replace(/,/g, "");
  if (t === "" || t === "-") {
    return 0;
  }
  if (!/^-?\d+$/.test(t)) {
    throw new Error("Enter a whole number of minor units (e.g. cents).");
  }
  const n = Number(t);
  if (!Number.isFinite(n) || !Number.isInteger(n)) {
    throw new Error("Enter a whole number of minor units (e.g. cents).");
  }
  if (n > Number.MAX_SAFE_INTEGER || n < Number.MIN_SAFE_INTEGER) {
    throw new Error("Amount is too large for safe integer arithmetic.");
  }
  return n;
}

export function sumMinor(values: readonly number[]): number {
  let t = 0n;
  for (const v of values) {
    t += BigInt(Math.trunc(v));
  }
  if (
    t > BigInt(Number.MAX_SAFE_INTEGER) ||
    t < BigInt(Number.MIN_SAFE_INTEGER)
  ) {
    throw new Error("sum overflow");
  }
  return Number(t);
}

export function formatMoneyMinor(
  minor: number,
  currencyCode: string = "USD",
  locale: string = typeof navigator !== "undefined"
    ? navigator.language
    : "en-US",
): string {
  const major = minor / 100;
  return new Intl.NumberFormat(locale, {
    style: "currency",
    currency: currencyCode,
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  }).format(major);
}
