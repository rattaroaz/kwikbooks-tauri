import { describe, expect, it } from "vitest";
import { requireValidISODate } from "./validateDate";

describe("requireValidISODate", () => {
  it("accepts valid dates", () => {
    expect(requireValidISODate("Date", "2026-01-15")).toBeNull();
  });

  it("rejects invalid dates", () => {
    expect(requireValidISODate("Date", "2026-13-40")).toMatch(/valid date/);
  });
});
