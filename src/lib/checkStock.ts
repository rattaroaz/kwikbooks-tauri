/** Check stock presets: brand labels map to voucher band positions on US Letter. */

export type CheckLayout =
  | "voucher_top"
  | "voucher_middle"
  | "voucher_bottom"
  | "generic";

export type CheckStockPreset = {
  id: string;
  label: string;
  layout: CheckLayout;
  help: string;
};

export const CHECK_STOCK_PRESETS: readonly CheckStockPreset[] = [
  {
    id: "generic",
    label: "Generic (alignment guides)",
    layout: "generic",
    help: "Dashed field boxes for aligning any blank stock. MICR stays pre-printed.",
  },
  {
    id: "deluxe_qb_top",
    label: "Deluxe / QuickBooks-compatible (check on top)",
    layout: "voucher_top",
    help: "Standard business voucher: check band at the top of the page.",
  },
  {
    id: "checks_unlimited_middle",
    label: "Checks Unlimited / middle voucher",
    layout: "voucher_middle",
    help: "Voucher stock with the check band in the middle of the page.",
  },
  {
    id: "bottom_quicken",
    label: "Bottom voucher / Quicken-style",
    layout: "voucher_bottom",
    help: "Voucher stock with the check band at the bottom of the page.",
  },
] as const;

export function layoutFromStyle(style: string): CheckLayout {
  if (
    style === "voucher_top" ||
    style === "voucher_middle" ||
    style === "voucher_bottom" ||
    style === "generic"
  ) {
    return style;
  }
  return "voucher_top";
}

export function presetForLayout(layout: CheckLayout): CheckStockPreset {
  return (
    CHECK_STOCK_PRESETS.find((p) => p.layout === layout) ??
    CHECK_STOCK_PRESETS[1]!
  );
}

export function presetById(id: string): CheckStockPreset {
  return (
    CHECK_STOCK_PRESETS.find((p) => p.id === id) ?? CHECK_STOCK_PRESETS[1]!
  );
}
