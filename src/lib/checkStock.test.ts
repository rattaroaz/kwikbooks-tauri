import { describe, expect, it } from "vitest";
import {
  CHECK_STOCK_PRESETS,
  layoutFromStyle,
  presetById,
  presetForLayout,
} from "./checkStock";

describe("checkStock", () => {
  it("includes generic and three voucher layouts", () => {
    const layouts = new Set(CHECK_STOCK_PRESETS.map((p) => p.layout));
    expect(layouts.has("generic")).toBe(true);
    expect(layouts.has("voucher_top")).toBe(true);
    expect(layouts.has("voucher_middle")).toBe(true);
    expect(layouts.has("voucher_bottom")).toBe(true);
  });

  it("maps company style strings to layouts", () => {
    expect(layoutFromStyle("voucher_middle")).toBe("voucher_middle");
    expect(layoutFromStyle("unknown")).toBe("voucher_top");
  });

  it("resolves preset by layout and id", () => {
    expect(presetForLayout("voucher_bottom").layout).toBe("voucher_bottom");
    expect(presetById("deluxe_qb_top").layout).toBe("voucher_top");
    expect(presetById("missing").layout).toBe("voucher_top");
  });
});
