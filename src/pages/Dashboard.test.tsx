// @vitest-environment happy-dom
import { screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi, beforeEach } from "vitest";
import { Dashboard } from "./Dashboard";
import { renderWithApp } from "../test/renderWithApp";
import * as api from "../api/tauri";

vi.mock("../api/tauri", () => ({
  listInvoices: vi.fn(),
  listBills: vi.fn(),
  reportArOpen: vi.fn(),
}));

describe("Dashboard", () => {
  beforeEach(() => {
    vi.mocked(api.listInvoices).mockResolvedValue([{ id: 1 }, { id: 2 }]);
    vi.mocked(api.listBills).mockResolvedValue([{ id: 10 }]);
    vi.mocked(api.reportArOpen).mockResolvedValue([
      { customerName: "Acme", openMinor: 100 },
      { customerName: "Beta", openMinor: 200 },
    ]);
  });

  it("loads summary counts into cards", async () => {
    renderWithApp(<Dashboard />);

    await waitFor(() => {
      const values = screen.getAllByText(/^[12]$/);
      expect(values).toHaveLength(3);
    });
    expect(screen.getByText("Invoices").parentElement).toHaveTextContent("2");
    expect(screen.getByText("Bills").parentElement).toHaveTextContent("1");
    expect(screen.getByText("Open AR (customers)").parentElement).toHaveTextContent(
      "2",
    );
    expect(api.listInvoices).toHaveBeenCalled();
    expect(api.listBills).toHaveBeenCalled();
    expect(api.reportArOpen).toHaveBeenCalled();
  });

  it("shows getting started when there are no invoices or bills", async () => {
    vi.mocked(api.listInvoices).mockResolvedValue([]);
    vi.mocked(api.listBills).mockResolvedValue([]);

    renderWithApp(<Dashboard />);

    expect(await screen.findByText(/Getting started/i)).toBeDefined();
  });

  it("reports load failures without crashing", async () => {
    vi.mocked(api.listInvoices).mockRejectedValue(new Error("db offline"));

    renderWithApp(<Dashboard />);

    await waitFor(() => {
      expect(screen.getAllByText("—")).toHaveLength(3);
    });
    expect(api.listInvoices).toHaveBeenCalled();
  });
});
