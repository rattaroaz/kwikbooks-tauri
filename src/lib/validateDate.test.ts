import { describe, expect, it } from "vitest";
import { requireValidISODate, requireValidISODateRange } from "./validateDate";

describe("requireValidISODate", () => {
  it("accepts valid dates", () => {
    expect(requireValidISODate("Date", "2026-01-15")).toBeNull();
  });

  it("rejects invalid dates", () => {
    expect(requireValidISODate("Date", "2026-13-40")).toMatch(/valid date/);
  });
});

describe("requireValidISODateRange", () => {
  it("accepts ordered dates", () => {
    expect(
      requireValidISODateRange("From", "2026-01-01", "To", "2026-01-31"),
    ).toBeNull();
  });

  it("rejects inverted ranges", () => {
    expect(
      requireValidISODateRange("From", "2026-02-01", "To", "2026-01-01"),
    ).toMatch(/on or before/i);
  });
});
