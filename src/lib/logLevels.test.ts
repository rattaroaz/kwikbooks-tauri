import { describe, expect, it } from "vitest";
import { defaultEnabledLevels, filterLogLines } from "./logLevels";

describe("logLevels", () => {
  it("defaultEnabledLevels includes every level", () => {
    expect(defaultEnabledLevels().size).toBe(6);
  });

  it("filters lines by selected level", () => {
    const lines = [
      { level: "info", line: "a" },
      { level: "warn", line: "b" },
    ];
    expect(filterLogLines(lines, "info")).toEqual([
      { level: "info", line: "a" },
    ]);
  });

  it("returns all lines when filter is all", () => {
    const lines = [
      { level: "info", line: "a" },
      { level: "warn", line: "b" },
    ];
    expect(filterLogLines(lines, "all")).toEqual(lines);
  });
});
