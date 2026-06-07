import * as fc from "fast-check";
import { describe, expect, it } from "vitest";
import { isValidISODate } from "./dates";

describe("isValidISODate (property)", () => {
  it("accepts only well-formed calendar dates", () => {
    fc.assert(
      fc.property(
        fc.integer({ min: 1900, max: 2100 }),
        fc.integer({ min: 1, max: 12 }),
        fc.integer({ min: 1, max: 31 }),
        (y, m, d) => {
          const s = `${y}-${String(m).padStart(2, "0")}-${String(d).padStart(2, "0")}`;
          const result = isValidISODate(s);
          // If the date is impossible (e.g. Feb 30), result must be false
          const dt = new Date(Date.UTC(y, m - 1, d));
          const isReal =
            dt.getUTCFullYear() === y &&
            dt.getUTCMonth() === m - 1 &&
            dt.getUTCDate() === d;
          expect(result).toBe(isReal);
        },
      ),
      { numRuns: 200 },
    );
  });

  it("rejects malformed strings", () => {
    fc.assert(
      fc.property(fc.string(), (s) => {
        if (!/^\d{4}-\d{2}-\d{2}$/.test(s)) {
          expect(isValidISODate(s)).toBe(false);
        }
      }),
      { numRuns: 100 },
    );
  });
});
