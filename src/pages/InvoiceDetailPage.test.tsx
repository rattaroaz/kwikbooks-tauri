// @vitest-environment happy-dom
import { screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi, beforeEach } from "vitest";
import { InvoiceDetailPage } from "./InvoiceDetailPage";
import { renderWithApp } from "../test/renderWithApp";
import * as api from "../api/tauri";

vi.mock("../api/tauri", () => ({
  getInvoice: vi.fn(),
  invoiceSetStatus: vi.fn(),
  invoicePost: vi.fn(),
}));

describe("InvoiceDetailPage", () => {
  beforeEach(() => {
    vi.mocked(api.getInvoice).mockReset();
  });

  it("shows an error instead of spinning for an invalid id", async () => {
    renderWithApp(<InvoiceDetailPage />, {
      route: "/invoices/abc",
      path: "/invoices/:id",
    });

    expect(await screen.findByText(/Invalid invoice id/i)).toBeDefined();
    expect(screen.queryByText("Loading…")).toBeNull();
    expect(api.getInvoice).not.toHaveBeenCalled();
  });

  it("shows an error instead of spinning when load fails", async () => {
    vi.mocked(api.getInvoice).mockRejectedValue(new Error("not found"));

    renderWithApp(<InvoiceDetailPage />, {
      route: "/invoices/99",
      path: "/invoices/:id",
    });

    expect(
      await screen.findByText(/Could not load this invoice/i),
    ).toBeDefined();
    expect(screen.queryByText("Loading…")).toBeNull();
  });

  it("renders a loaded invoice", async () => {
    vi.mocked(api.getInvoice).mockResolvedValue({
      header: {
        number: "INV-1",
        customerName: "Acme",
        status: "draft",
        totalMinor: 2500,
      },
      lines: [
        {
          lineNumber: 1,
          description: "Work",
          quantity: 1,
          lineTotalMinor: 2500,
        },
      ],
    });

    renderWithApp(<InvoiceDetailPage />, {
      route: "/invoices/7",
      path: "/invoices/:id",
    });

    expect(await screen.findByText("Invoice INV-1")).toBeDefined();
    await waitFor(() => expect(api.getInvoice).toHaveBeenCalledWith(7));
  });
});
