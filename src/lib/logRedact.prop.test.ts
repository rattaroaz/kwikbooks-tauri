import fc from "fast-check";
import { describe, it } from "vitest";
import { redactForLog } from "./logRedact";

describe("logRedact (property)", () => {
  it("always redacts the email key regardless of string payload", () => {
    fc.assert(
      fc.property(fc.string(), (secret) => {
        const r = redactForLog({ email: secret }) as { email: unknown };
        return r.email === "[redacted]";
      }),
      { numRuns: 100 },
    );
  });

  it("preserves non-sensitive numeric ids", () => {
    fc.assert(
      fc.property(fc.integer({ min: 1, max: 10_000_000 }), (id) => {
        const r = redactForLog({
          customerId: id,
          memo: "x",
        }) as { customerId: number; memo: unknown };
        return r.customerId === id && r.memo === "[redacted]";
      }),
      { numRuns: 80 },
    );
  });
});
