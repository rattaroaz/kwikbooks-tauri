import * as fc from "fast-check";
import { describe, expect, it } from "vitest";
import {
  asSafeMinor,
  formatMoneyMinor,
  lineTotalMinor,
  parseMinorInt,
  sumMinor,
} from "./money";

describe("parseMinorInt", () => {
  it("parses integers and strips commas", () => {
    expect(parseMinorInt("  42 ")).toBe(42);
    expect(parseMinorInt("1,000")).toBe(1000);
  });

  it("rejects fractions", () => {
    expect(() => parseMinorInt("10.5")).toThrow(/whole number/i);
  });

  it("treats empty and hyphen as zero", () => {
    expect(parseMinorInt("")).toBe(0);
    expect(parseMinorInt("   ")).toBe(0);
    expect(parseMinorInt("-")).toBe(0);
  });

  it("rejects values exceeding safe integer range", () => {
    const tooBig = String(Number.MAX_SAFE_INTEGER + 1);
    expect(() => parseMinorInt(tooBig)).toThrow(/too large for safe integer/i);
  });
});

describe("sumMinor", () => {
  it("sums safely with bigint", () => {
    expect(sumMinor([1, 2, 3])).toBe(6);
  });

  it("throws on overflow range", () => {
    expect(() =>
      sumMinor([Number.MAX_SAFE_INTEGER, Number.MAX_SAFE_INTEGER]),
    ).toThrow(/overflow/);
  });
});

describe("formatMoneyMinor", () => {
  it("formats with fixed cents", () => {
    expect(formatMoneyMinor(12345, "USD", "en-US")).toMatch(/\$123\.45/);
  });

  it("formats zero-decimal currencies without dividing by 100", () => {
    expect(formatMoneyMinor(1234, "JPY", "en-US")).toMatch(/1,234/);
  });

  it("rejects unsafe integers", () => {
    expect(() => formatMoneyMinor(Number.MAX_SAFE_INTEGER + 1)).toThrow(
      /safe integer/i,
    );
  });
});

describe("lineTotalMinor", () => {
  it("matches integer-safe qty × unit", () => {
    expect(lineTotalMinor(2, 1250)).toBe(2500);
    expect(lineTotalMinor(0.1, 33)).toBe(3);
    expect(lineTotalMinor(1.5, 100)).toBe(150);
  });
});

describe("asSafeMinor", () => {
  it("accepts safe integers", () => {
    expect(asSafeMinor(42)).toBe(42);
  });
});

describe("parseMinorInt (property)", () => {
  it("round-trips valid integer strings (with optional commas)", () => {
    fc.assert(
      fc.property(fc.integer({ min: -1_000_000, max: 1_000_000 }), (n) => {
        const withCommas = n.toLocaleString("en-US");
        expect(parseMinorInt(withCommas)).toBe(n);
        expect(parseMinorInt(String(n))).toBe(n);
      }),
      { numRuns: 300 },
    );
  });

  it("returns 0 for empty/hyphen whitespace", () => {
    fc.assert(
      fc.property(fc.constantFrom("", " ", "-", "  -  ", "\t"), (s) => {
        expect(parseMinorInt(s)).toBe(0);
      }),
    );
  });
});

describe("sumMinor (property)", () => {
  it("never overflows on safe inputs and matches naive sum", () => {
    fc.assert(
      fc.property(
        fc.array(fc.integer({ min: -1_000_000, max: 1_000_000 }), {
          maxLength: 50,
        }),
        (arr) => {
          const expected = arr.reduce((a, b) => a + b, 0);
          if (
            expected > Number.MAX_SAFE_INTEGER ||
            expected < Number.MIN_SAFE_INTEGER
          ) {
            expect(() => sumMinor(arr)).toThrow(/overflow/);
          } else {
            expect(sumMinor(arr)).toBe(expected);
          }
        },
      ),
      { numRuns: 200 },
    );
  });
});
