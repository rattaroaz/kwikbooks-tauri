// @vitest-environment happy-dom
import { fireEvent, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi, beforeEach } from "vitest";
import { InvoiceNewPage } from "./InvoiceNewPage";
import { renderWithApp } from "../test/renderWithApp";
import * as api from "../api/tauri";

const navigateMock = vi.fn();

vi.mock("react-router-dom", async (importOriginal) => {
  const actual = await importOriginal<typeof import("react-router-dom")>();
  return {
    ...actual,
    useNavigate: () => navigateMock,
  };
});

vi.mock("../api/tauri", () => ({
  listCustomers: vi.fn(),
  invoiceCreate: vi.fn(),
}));

describe("InvoiceNewPage", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    navigateMock.mockReset();
    vi.mocked(api.listCustomers).mockResolvedValue([
      { id: 7, displayName: "Acme Corp" },
    ]);
    vi.mocked(api.invoiceCreate).mockResolvedValue(42);
  });

  it("loads customers into the form", async () => {
    renderWithApp(<InvoiceNewPage />);

    await waitFor(() => {
      expect(screen.getByRole("option", { name: "Acme Corp" })).toBeDefined();
    });
    expect(api.listCustomers).toHaveBeenCalled();
  });

  it("creates a draft invoice and navigates to detail", async () => {
    renderWithApp(<InvoiceNewPage />);
    await waitFor(() =>
      expect(screen.getByRole("option", { name: "Acme Corp" })).toBeDefined(),
    );

    fireEvent.change(screen.getByLabelText(/^Invoice #/i), {
      target: { value: "INV-100" },
    });
    fireEvent.change(screen.getByLabelText(/^Unit price/i), {
      target: { value: "2500" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Save draft" }));

    await waitFor(() => {
      expect(api.invoiceCreate).toHaveBeenCalledWith(
        expect.objectContaining({
          customerId: 7,
          number: "INV-100",
          lines: [
            expect.objectContaining({
              description: "Line 1",
              quantity: 1,
              unitPriceMinor: 2500,
            }),
          ],
        }),
      );
    });
    expect(navigateMock).toHaveBeenCalledWith("/invoices/42");
  });

  it("requires a customer before creating an invoice", async () => {
    vi.mocked(api.listCustomers).mockResolvedValue([]);

    renderWithApp(<InvoiceNewPage />);
    await waitFor(() => expect(api.listCustomers).toHaveBeenCalled());

    fireEvent.change(screen.getByLabelText(/^Invoice #/i), {
      target: { value: "INV-1" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Save draft" }));

    expect(api.invoiceCreate).not.toHaveBeenCalled();
    expect(navigateMock).not.toHaveBeenCalled();
  });

  it("rejects invalid quantity", async () => {
    renderWithApp(<InvoiceNewPage />);
    await waitFor(() =>
      expect(screen.getByRole("option", { name: "Acme Corp" })).toBeDefined(),
    );

    fireEvent.change(screen.getByLabelText(/^Invoice #/i), {
      target: { value: "INV-2" },
    });
    fireEvent.change(screen.getByLabelText(/^Qty/i), {
      target: { value: "0" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Save draft" }));

    expect(api.invoiceCreate).not.toHaveBeenCalled();
    expect(navigateMock).not.toHaveBeenCalled();
  });

  it("rejects negative tax or unit price", async () => {
    renderWithApp(<InvoiceNewPage />);
    await waitFor(() =>
      expect(screen.getByRole("option", { name: "Acme Corp" })).toBeDefined(),
    );

    fireEvent.change(screen.getByLabelText(/^Invoice #/i), {
      target: { value: "INV-3" },
    });
    fireEvent.change(screen.getByLabelText(/^Tax/i), {
      target: { value: "-10" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Save draft" }));

    expect(api.invoiceCreate).not.toHaveBeenCalled();
  });
});
