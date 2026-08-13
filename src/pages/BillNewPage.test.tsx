// @vitest-environment happy-dom
import { fireEvent, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi, beforeEach } from "vitest";
import { BillNewPage } from "./BillNewPage";
import { renderWithApp } from "../test/renderWithApp";
import * as api from "../api/tauri";

vi.mock("../api/tauri", () => ({
  listVendors: vi.fn(),
  accountList: vi.fn(),
  billCreate: vi.fn(),
}));

describe("BillNewPage", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(api.listVendors).mockResolvedValue([]);
    vi.mocked(api.accountList).mockResolvedValue([
      { id: 5, code: "5000", name: "Expenses", accountType: "expense" },
    ]);
    vi.mocked(api.billCreate).mockResolvedValue(3);
  });

  it("requires a vendor or payee before creating", async () => {
    renderWithApp(<BillNewPage />);
    await waitFor(() =>
      expect(screen.getByRole("option", { name: /5000/ })).toBeDefined(),
    );

    fireEvent.change(screen.getByLabelText(/^Bill #/i), {
      target: { value: "B-1" },
    });
    fireEvent.change(screen.getByLabelText(/^Amount/i), {
      target: { value: "900" },
    });
    fireEvent.click(screen.getByTestId("bill-new-save-draft"));

    expect(api.billCreate).not.toHaveBeenCalled();
  });

  it("rejects a non-positive amount", async () => {
    vi.mocked(api.listVendors).mockResolvedValue([
      { id: 2, displayName: "Office Co" },
    ]);
    renderWithApp(<BillNewPage />);
    await waitFor(() =>
      expect(screen.getByRole("option", { name: "Office Co" })).toBeDefined(),
    );

    fireEvent.change(screen.getByLabelText(/^Bill #/i), {
      target: { value: "B-2" },
    });
    fireEvent.change(screen.getByLabelText(/^Vendor/i), {
      target: { value: "2" },
    });
    fireEvent.change(screen.getByLabelText(/^Amount/i), {
      target: { value: "0" },
    });
    fireEvent.click(screen.getByTestId("bill-new-save-draft"));

    expect(api.billCreate).not.toHaveBeenCalled();
  });
});
