import * as fc from "fast-check";
import { describe, expect, it } from "vitest";
import { isValidISODate, todayISODate } from "./dates";

describe("todayISODate", () => {
  it("returns YYYY-MM-DD", () => {
    expect(todayISODate()).toMatch(/^\d{4}-\d{2}-\d{2}$/);
  });
});

describe("isValidISODate", () => {
  it("accepts valid date strings", () => {
    expect(isValidISODate("2026-01-31")).toBe(true);
    expect(isValidISODate("2024-02-29")).toBe(true);
  });

  it("rejects malformed strings", () => {
    expect(isValidISODate("2026-1-1")).toBe(false);
    expect(isValidISODate("01-31-2026")).toBe(false);
    expect(isValidISODate("not-a-date")).toBe(false);
  });

  it("rejects impossible calendar dates", () => {
    expect(isValidISODate("2026-02-30")).toBe(false);
    expect(isValidISODate("2025-13-01")).toBe(false);
    expect(isValidISODate("2025-00-10")).toBe(false);
  });
});

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
          const dt = new Date(Date.UTC(y, m - 1, d));
          const isReal =
            dt.getUTCFullYear() === y && dt.getUTCMonth() === m - 1 && dt.getUTCDate() === d;
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
