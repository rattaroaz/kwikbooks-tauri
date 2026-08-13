// @vitest-environment happy-dom
import { fireEvent, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi, beforeEach } from "vitest";
import { PayBillPage } from "./PayBillPage";
import { renderWithApp } from "../test/renderWithApp";
import * as api from "../api/tauri";

vi.mock("../api/tauri", () => ({
  listVendors: vi.fn(),
  accountList: vi.fn(),
  vendorPaymentCreate: vi.fn(),
}));

describe("PayBillPage", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(api.listVendors).mockResolvedValue([
      { id: 2, displayName: "Office Co" },
    ]);
    vi.mocked(api.accountList).mockResolvedValue([
      { id: 10, code: "1000", name: "Cash", isBankCash: true },
    ]);
    vi.mocked(api.vendorPaymentCreate).mockResolvedValue(88);
  });

  async function ready() {
    renderWithApp(<PayBillPage />);
    await waitFor(() =>
      expect(screen.getByRole("option", { name: "Office Co" })).toBeDefined(),
    );
  }

  it("rejects a zero or negative amount", async () => {
    await ready();
    fireEvent.change(screen.getByLabelText(/^Amount/i), {
      target: { value: "0" },
    });
    fireEvent.click(screen.getByTestId("pay-bill-submit"));
    expect(api.vendorPaymentCreate).not.toHaveBeenCalled();
  });

  it("rejects a non-positive bill id", async () => {
    await ready();
    fireEvent.change(screen.getByLabelText(/^Amount/i), {
      target: { value: "1200" },
    });
    fireEvent.change(screen.getByLabelText(/bill id/i), {
      target: { value: "0" },
    });
    fireEvent.click(screen.getByTestId("pay-bill-submit"));
    expect(api.vendorPaymentCreate).not.toHaveBeenCalled();
  });
});
