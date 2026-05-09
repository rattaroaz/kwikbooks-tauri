import * as fc from "fast-check";
import { describe, expect, it } from "vitest";
import { parseMinorInt, sumMinor } from "./money";

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
      fc.property(fc.array(fc.integer({ min: -1_000_000, max: 1_000_000 }), { maxLength: 50 }), (arr) => {
        const expected = arr.reduce((a, b) => a + b, 0);
        if (expected > Number.MAX_SAFE_INTEGER || expected < Number.MIN_SAFE_INTEGER) {
          expect(() => sumMinor(arr)).toThrow(/overflow/);
        } else {
          expect(sumMinor(arr)).toBe(expected);
        }
      }),
      { numRuns: 200 },
    );
  });
});
