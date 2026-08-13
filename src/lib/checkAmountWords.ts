import { currencyFractionDigits } from "./money";

const ONES = [
  "",
  "one",
  "two",
  "three",
  "four",
  "five",
  "six",
  "seven",
  "eight",
  "nine",
  "ten",
  "eleven",
  "twelve",
  "thirteen",
  "fourteen",
  "fifteen",
  "sixteen",
  "seventeen",
  "eighteen",
  "nineteen",
];

const TENS = [
  "",
  "",
  "twenty",
  "thirty",
  "forty",
  "fifty",
  "sixty",
  "seventy",
  "eighty",
  "ninety",
];

function underThousand(n: number): string {
  if (n === 0) {
    return "";
  }
  if (n < 20) {
    return ONES[n] ?? "";
  }
  if (n < 100) {
    const t = Math.floor(n / 10);
    const o = n % 10;
    const tens = TENS[t] ?? "";
    return o === 0 ? tens : `${tens}-${ONES[o] ?? ""}`;
  }
  const h = Math.floor(n / 100);
  const rest = n % 100;
  const hundreds = ONES[h] ?? "";
  return rest === 0
    ? `${hundreds} hundred`
    : `${hundreds} hundred ${underThousand(rest)}`;
}

function integerToWords(n: number): string {
  if (n === 0) {
    return "zero";
  }
  const parts: string[] = [];
  const trillions = Math.floor(n / 1_000_000_000_000);
  const billions = Math.floor((n % 1_000_000_000_000) / 1_000_000_000);
  const millions = Math.floor((n % 1_000_000_000) / 1_000_000);
  const thousands = Math.floor((n % 1_000_000) / 1000);
  const rest = n % 1000;
  if (trillions) {
    parts.push(`${underThousand(trillions)} trillion`);
  }
  if (billions) {
    parts.push(`${underThousand(billions)} billion`);
  }
  if (millions) {
    parts.push(`${underThousand(millions)} million`);
  }
  if (thousands) {
    parts.push(`${underThousand(thousands)} thousand`);
  }
  if (rest) {
    parts.push(underThousand(rest));
  }
  return parts.join(" ");
}

function capitalize(s: string): string {
  if (!s) {
    return s;
  }
  return s.charAt(0).toUpperCase() + s.slice(1);
}

/**
 * Minor units → check amount line.
 * Uses the currency's ISO fraction digits (USD → "… and 45/100").
 */
export function amountMinorToWords(
  minor: number,
  currencyCode: string = "USD",
): string {
  if (!Number.isFinite(minor) || !Number.isInteger(minor) || minor < 0) {
    throw new Error("Amount must be a non-negative integer of minor units.");
  }
  if (!Number.isSafeInteger(minor)) {
    throw new Error("Amount exceeds safe integer range.");
  }
  const digits = currencyFractionDigits(currencyCode);
  const scale = 10 ** digits;
  const major = digits === 0 ? minor : Math.floor(minor / scale);
  const frac = digits === 0 ? 0 : minor % scale;
  const majorWords = major === 0 ? "zero" : integerToWords(major);
  if (digits === 0) {
    return capitalize(majorWords);
  }
  const fracPart = `${String(frac).padStart(digits, "0")}/${scale}`;
  return capitalize(`${majorWords} and ${fracPart}`);
}
