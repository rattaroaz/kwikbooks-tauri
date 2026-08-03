import { describe, expect, it } from "vitest";
import { amountMinorToWords } from "./checkAmountWords";

describe("amountMinorToWords", () => {
  it("formats zero dollars with cents", () => {
    expect(amountMinorToWords(45)).toBe("Zero and 45/100");
  });

  it("formats whole dollars", () => {
    expect(amountMinorToWords(10000)).toBe("One hundred and 00/100");
  });

  it("formats typical check amount", () => {
    expect(amountMinorToWords(12345)).toBe(
      "One hundred twenty-three and 45/100",
    );
  });

  it("formats zero-decimal currencies without a fractional part", () => {
    expect(amountMinorToWords(1234, "JPY")).toBe("One thousand two hundred thirty-four");
  });

  it("formats thousands", () => {
    expect(amountMinorToWords(1_005_01)).toBe(
      "One thousand five and 01/100",
    );
  });

  it("formats one trillion dollars", () => {
    expect(amountMinorToWords(100_000_000_000_000)).toBe(
      "One trillion and 00/100",
    );
  });

  it("rejects negative", () => {
    expect(() => amountMinorToWords(-1)).toThrow(/non-negative/i);
  });

  it("rejects unsafe integers", () => {
    expect(() => amountMinorToWords(Number.MAX_SAFE_INTEGER + 1)).toThrow(
      /safe integer/i,
    );
  });
});
