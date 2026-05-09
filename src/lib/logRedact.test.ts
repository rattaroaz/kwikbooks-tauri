import { describe, expect, it } from "vitest";
import { redactForLog, summarizeInvokePayload } from "./logRedact";

describe("logRedact", () => {
  it("redacts known sensitive keys recursively", () => {
    const input = {
      customerId: 1,
      displayName: "Acme",
      nested: { Email: "x@y.com", safe: 2 },
    };
    const r = redactForLog(input) as Record<string, unknown>;
    expect(r.customerId).toBe(1);
    expect(r.displayName).toBe("[redacted]");
    expect((r.nested as Record<string, unknown>).Email).toBe("[redacted]");
    expect((r.nested as Record<string, unknown>).safe).toBe(2);
  });

  it("summarizeInvokePayload produces stable JSON", () => {
    const s = summarizeInvokePayload({
      number: "INV-1",
      memo: "secret text",
    });
    expect(s).toContain('"memo":"[redacted]"');
    expect(s).toContain('"number":"INV-1"');
  });

  it("passes primitives through unchanged", () => {
    expect(redactForLog(0)).toBe(0);
    expect(redactForLog(true)).toBe(true);
  });

  it("marks object cycles without hanging", () => {
    const o: Record<string, unknown> = { x: 1 };
    o.loop = o;
    const r = redactForLog(o) as Record<string, unknown>;
    expect(r.loop).toBe("[cycle]");
  });

  it("summarizeInvokePayload catches stringify failures", () => {
    const big = BigInt(1);
    expect(summarizeInvokePayload({ big })).toBe("[payload_not_serializable]");
  });
});
