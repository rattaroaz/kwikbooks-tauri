import { describe, expect, it } from "vitest";
import {
  errorMessage,
  formatAppCommandError,
  parseAppCommandError,
} from "./errors";

describe("parseAppCommandError", () => {
  it("parses not_found", () => {
    const e = parseAppCommandError({
      code: "not_found",
      entity: "invoice",
      id: 3,
    });
    expect(e).toEqual({ code: "not_found", entity: "invoice", id: 3 });
  });

  it("parses conflict", () => {
    const e = parseAppCommandError({
      code: "conflict",
      message: "payment already posted",
    });
    expect(e?.code).toBe("conflict");
  });

  it("returns null for unknown shapes", () => {
    expect(parseAppCommandError("oops")).toBeNull();
    expect(parseAppCommandError({ code: "nope" })).toBeNull();
  });
});

describe("formatAppCommandError", () => {
  it("formats not_found", () => {
    const msg = formatAppCommandError({
      code: "not_found",
      entity: "customer_payment",
      id: 9,
    });
    expect(msg).toContain("customer payment");
    expect(msg).toContain("9");
  });

  it("formats already posted conflict", () => {
    const msg = formatAppCommandError({
      code: "conflict",
      message: "payment already posted",
    });
    expect(msg).toContain("already posted");
  });
});

describe("errorMessage", () => {
  it("uses structured formatter when possible", () => {
    expect(errorMessage({ code: "validation", message: "bad date" })).toBe(
      "bad date",
    );
  });
});
