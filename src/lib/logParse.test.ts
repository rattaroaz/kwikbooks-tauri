import { describe, expect, it } from "vitest";
import {
  filterLogLinesBySearch,
  lineMatchesCorrelationSearch,
  parseLogTimestamp,
  sortLogLinesByTimestamp,
} from "./logParse";

describe("parseLogTimestamp", () => {
  it("reads bracketed plugin-log timestamps", () => {
    const ms = parseLogTimestamp(
      "[2026-03-15][09:30:00][INFO] invoke_ok seq=1 command=company_get",
    );
    expect(ms).toBe(Date.parse("2026-03-15T09:30:00"));
  });

  it("returns null when no timestamp is present", () => {
    expect(parseLogTimestamp("invoke_ok seq=2 rid=abc")).toBeNull();
  });
});

describe("sortLogLinesByTimestamp", () => {
  it("orders lines chronologically regardless of input order", () => {
    const lines = [
      { line: "[2026-01-01][12:00:01][INFO] second" },
      { line: "[2026-01-01][12:00:00][INFO] first" },
      { line: "no timestamp stays last" },
    ];
    const sorted = sortLogLinesByTimestamp(lines);
    expect(sorted.map((l) => l.line)).toEqual([
      "[2026-01-01][12:00:00][INFO] first",
      "[2026-01-01][12:00:01][INFO] second",
      "no timestamp stays last",
    ]);
  });
});

describe("lineMatchesCorrelationSearch", () => {
  it("matches seq and rid tokens", () => {
    const line =
      "[2026-01-01][12:00:00][INFO] invoke_ok seq=42 rid=req-abc command=logs_read";
    expect(lineMatchesCorrelationSearch(line, "42")).toBe(true);
    expect(lineMatchesCorrelationSearch(line, "req-abc")).toBe(true);
    expect(lineMatchesCorrelationSearch(line, "req-ab")).toBe(true);
    expect(lineMatchesCorrelationSearch(line, "999")).toBe(false);
  });
});

describe("filterLogLinesBySearch", () => {
  it("filters to lines matching the correlation query", () => {
    const lines = [
      { line: "invoke_ok seq=1 rid=aaa" },
      { line: "invoke_ok seq=2 rid=bbb" },
    ];
    expect(filterLogLinesBySearch(lines, "bbb")).toHaveLength(1);
  });
});
