import { afterEach, describe, expect, it, vi } from "vitest";
import { downloadTextFile, escapeCsvCell, rowsToCsv } from "./csv";

describe("escapeCsvCell", () => {
  it("quotes cells with commas/newlines/quotes", () => {
    expect(escapeCsvCell("a,b")).toBe('"a,b"');
    expect(escapeCsvCell("line1\nline2")).toBe('"line1\nline2"');
    expect(escapeCsvCell('a"b')).toBe('"a""b"');
  });

  it("returns empty for nullish values", () => {
    expect(escapeCsvCell(null)).toBe("");
    expect(escapeCsvCell(undefined)).toBe("");
  });
});

describe("rowsToCsv", () => {
  it("joins rows with CRLF", () => {
    const out = rowsToCsv([
      ["name", "amount"],
      ["Alice", "100"],
      ["Bob", "250"],
    ]);
    expect(out).toBe("name,amount\r\nAlice,100\r\nBob,250");
  });

  it("applies CSV escaping per cell", () => {
    const out = rowsToCsv([
      ["name", "note"],
      ["ACME", 'hello,"world"'],
    ]);
    expect(out).toBe('name,note\r\nACME,"hello,""world"""');
  });
});

describe("downloadTextFile", () => {
  const origCreate = URL.createObjectURL;
  const origRevoke = URL.revokeObjectURL;
  const origDocument = globalThis.document;

  afterEach(() => {
    URL.createObjectURL = origCreate;
    URL.revokeObjectURL = origRevoke;
    Object.assign(globalThis, { document: origDocument });
    vi.restoreAllMocks();
  });

  it("creates an anchor, clicks, and revokes object URL", () => {
    const click = vi.fn();
    const anchor = {
      href: "",
      download: "",
      click,
    } as unknown as HTMLAnchorElement;
    const createElement = vi.fn(() => anchor);
    Object.assign(globalThis, {
      document: { createElement } as unknown as Document,
    });
    URL.createObjectURL = vi.fn(() => "blob:abc");
    URL.revokeObjectURL = vi.fn();

    downloadTextFile("x.csv", "a,b", "text/csv");

    expect(createElement).toHaveBeenCalledWith("a");
    expect(anchor.href).toBe("blob:abc");
    expect(anchor.download).toBe("x.csv");
    expect(click).toHaveBeenCalledOnce();
    expect(URL.revokeObjectURL).toHaveBeenCalledWith("blob:abc");
  });
});
