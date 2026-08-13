// @vitest-environment happy-dom
import { fireEvent, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi, beforeEach } from "vitest";
import { ReceivePaymentPage } from "./ReceivePaymentPage";
import { renderWithApp } from "../test/renderWithApp";
import * as api from "../api/tauri";

vi.mock("../api/tauri", () => ({
  listCustomers: vi.fn(),
  accountList: vi.fn(),
  customerPaymentCreate: vi.fn(),
}));

describe("ReceivePaymentPage", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(api.listCustomers).mockResolvedValue([
      { id: 1, displayName: "Acme" },
    ]);
    vi.mocked(api.accountList).mockResolvedValue([
      { id: 10, code: "1000", name: "Cash", isBankCash: true },
    ]);
    vi.mocked(api.customerPaymentCreate).mockResolvedValue(99);
  });

  async function ready() {
    renderWithApp(<ReceivePaymentPage />);
    await waitFor(() =>
      expect(screen.getByRole("option", { name: "Acme" })).toBeDefined(),
    );
  }

  it("rejects dollar-style decimals that would under-count cents", async () => {
    await ready();
    fireEvent.change(screen.getByLabelText(/^Amount/i), {
      target: { value: "100.00" },
    });
    fireEvent.click(screen.getByTestId("receive-payment-submit"));
    expect(api.customerPaymentCreate).not.toHaveBeenCalled();
  });

  it("rejects a non-integer invoice id", async () => {
    await ready();
    fireEvent.change(screen.getByLabelText(/^Amount/i), {
      target: { value: "2500" },
    });
    fireEvent.change(screen.getByLabelText(/invoice id/i), {
      target: { value: "1.5" },
    });
    fireEvent.click(screen.getByTestId("receive-payment-submit"));
    expect(api.customerPaymentCreate).not.toHaveBeenCalled();
  });

  it("records a payment once even if submit is clicked twice", async () => {
    let resolveCreate: ((id: number) => void) | undefined;
    vi.mocked(api.customerPaymentCreate).mockImplementation(
      () =>
        new Promise((resolve) => {
          resolveCreate = resolve;
        }),
    );

    await ready();
    fireEvent.change(screen.getByLabelText(/^Amount/i), {
      target: { value: "2500" },
    });
    const submit = screen.getByTestId("receive-payment-submit");
    fireEvent.click(submit);
    fireEvent.click(submit);

    await waitFor(() =>
      expect(api.customerPaymentCreate).toHaveBeenCalledTimes(1),
    );
    resolveCreate?.(99);
    await waitFor(() =>
      expect(screen.getByTestId("receive-payment-submit")).not.toBeDisabled(),
    );
  });
});
