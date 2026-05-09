import { describe, expect, it } from "vitest";
import { env, parsePositiveMs } from "./env";

describe("parsePositiveMs", () => {
  it("falls back to default for invalid or empty input", () => {
    expect(parsePositiveMs(undefined, 1500)).toBe(1500);
    expect(parsePositiveMs("", 99)).toBe(99);
    expect(parsePositiveMs("x", 10)).toBe(10);
    expect(parsePositiveMs("-5", 10)).toBe(10);
  });

  it("parses positive integers", () => {
    expect(parsePositiveMs("2500", 1)).toBe(2500);
    expect(parsePositiveMs("  42  ", 1)).toBe(42);
  });
});

describe("env.parseBool", () => {
  it("returns default for empty inputs", () => {
    expect(env.parseBool(undefined, true)).toBe(true);
    expect(env.parseBool("", false)).toBe(false);
  });

  it("accepts canonical truthy values", () => {
    expect(env.parseBool("1", false)).toBe(true);
    expect(env.parseBool("true", false)).toBe(true);
    expect(env.parseBool("yes", false)).toBe(true);
    expect(env.parseBool("on", false)).toBe(true);
    expect(env.parseBool("TRUE", false)).toBe(true);
  });

  it("treats other values as false", () => {
    expect(env.parseBool("0", true)).toBe(false);
    expect(env.parseBool("false", true)).toBe(false);
    expect(env.parseBool("no", true)).toBe(false);
    expect(env.parseBool("off", true)).toBe(false);
    expect(env.parseBool("random", true)).toBe(false);
  });
});
