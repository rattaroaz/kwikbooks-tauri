import { afterEach, describe, expect, it, vi } from "vitest";
import { createRequestId } from "./correlation";

afterEach(() => {
  vi.unstubAllGlobals();
});

describe("createRequestId", () => {
  it("returns a non-empty stable-length token when crypto is available", () => {
    const a = createRequestId();
    const b = createRequestId();
    expect(a.length).toBeGreaterThan(4);
    expect(b.length).toBeGreaterThan(4);
    expect(a).not.toBe(b);
  });

  it("falls back when crypto.randomUUID is missing", () => {
    const prev = globalThis.crypto;
    vi.stubGlobal("crypto", {
      randomUUID: undefined,
    } as unknown as Crypto);
    try {
      const id = createRequestId();
      expect(id.length).toBeGreaterThan(4);
      expect(id).toContain("-");
    } finally {
      vi.stubGlobal("crypto", prev);
    }
  });
});
