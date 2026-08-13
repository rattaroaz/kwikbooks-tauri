/**
 * All monetary amounts in the database are **integer minor units** (e.g. cents).
 * Use `bigint` for sums and products; use `formatMoneyMinor` only for **display** (uses `Intl`, not for accounting logic).
 */

/** Quantity scale matching Rust `domain::money::line_total_minor` (6 decimal places). */
const QTY_SCALE = 1_000_000n;

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

/** Coerce JSON / unknown values to a safe integer minor amount for display. */
export function asSafeMinor(value: unknown): number {
  if (typeof value === "bigint") {
    if (
      value > BigInt(Number.MAX_SAFE_INTEGER) ||
      value < BigInt(Number.MIN_SAFE_INTEGER)
    ) {
      throw new Error("Amount exceeds safe integer range.");
    }
    return Number(value);
  }
  const n = typeof value === "number" ? value : Number(value);
  if (!Number.isFinite(n) || !Number.isInteger(n)) {
    throw new Error("Amount must be an integer of minor units.");
  }
  if (!Number.isSafeInteger(n)) {
    throw new Error("Amount exceeds safe integer range.");
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

/**
 * Line total in minor units — mirrors Rust `line_total_minor`
 * (qty quantized to 6 dp, half-away-from-zero, clamp ≥ 0).
 */
export function lineTotalMinor(qty: number, unitMinor: number): number {
  if (!Number.isFinite(qty) || !Number.isFinite(unitMinor)) {
    return 0;
  }
  const qtyScaled = BigInt(Math.round(qty * Number(QTY_SCALE)));
  const product = qtyScaled * BigInt(Math.trunc(unitMinor));
  const half = QTY_SCALE / 2n;
  const rounded =
    product >= 0n ? (product + half) / QTY_SCALE : (product - half) / QTY_SCALE;
  if (rounded <= 0n) {
    return 0;
  }
  if (rounded > BigInt(Number.MAX_SAFE_INTEGER)) {
    return Number.MAX_SAFE_INTEGER;
  }
  return Number(rounded);
}

/** ISO 4217 fraction digits for a currency (fallback 2). */
export function currencyFractionDigits(
  currencyCode: string,
  locale: string = typeof navigator !== "undefined"
    ? navigator.language
    : "en-US",
): number {
  try {
    const digits = new Intl.NumberFormat(locale, {
      style: "currency",
      currency: currencyCode,
    }).resolvedOptions().maximumFractionDigits;
    return typeof digits === "number" ? digits : 2;
  } catch {
    return 2;
  }
}

/** Major-unit label for checks (e.g. USD → "Dollars"). */
export function currencyMajorLabel(
  currencyCode: string,
  locale: string = typeof navigator !== "undefined"
    ? navigator.language
    : "en-US",
): string {
  const code = currencyCode.trim().toUpperCase() || "USD";
  if (code === "USD") {
    return "Dollars";
  }
  try {
    return (
      new Intl.DisplayNames([locale], { type: "currency" }).of(code) ?? code
    );
  } catch {
    return code;
  }
}

export function formatMoneyMinor(
  minor: number | unknown,
  currencyCode: string = "USD",
  locale: string = typeof navigator !== "undefined"
    ? navigator.language
    : "en-US",
): string {
  const safe = asSafeMinor(minor);
  const digits = currencyFractionDigits(currencyCode, locale);
  const divisor = 10 ** digits;
  const major = digits === 0 ? safe : safe / divisor;
  return new Intl.NumberFormat(locale, {
    style: "currency",
    currency: currencyCode,
    minimumFractionDigits: digits,
    maximumFractionDigits: digits,
  }).format(major);
}
