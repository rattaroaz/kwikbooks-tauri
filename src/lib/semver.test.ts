import { describe, expect, it } from "vitest";
import { isVersionNewer, parseSemver } from "./semver";

describe("parseSemver", () => {
  it("parses three-part versions", () => {
    expect(parseSemver("1.2.3")).toEqual([1, 2, 3]);
  });

  it("parses v-prefix and two-part as patch 0", () => {
    expect(parseSemver("v1.2")).toEqual([1, 2, 0]);
  });
});

describe("isVersionNewer", () => {
  it("1.2.0 > 1.1.9", () => {
    expect(isVersionNewer("1.2.0", "1.1.9")).toBe(true);
  });

  it("1.1.0 is not newer than 1.1.0", () => {
    expect(isVersionNewer("1.1.0", "1.1.0")).toBe(false);
  });

  it("v1.2.0 > 1.1.0", () => {
    expect(isVersionNewer("v1.2.0", "1.1.0")).toBe(true);
  });

  it("1.2 parses as 1.2.0 and compares", () => {
    expect(isVersionNewer("1.2.1", "1.2")).toBe(true);
  });
});
