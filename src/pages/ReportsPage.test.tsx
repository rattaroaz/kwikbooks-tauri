// @vitest-environment happy-dom
import { fireEvent, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi, beforeEach } from "vitest";
import { ReportsPage } from "./ReportsPage";
import { renderWithApp } from "../test/renderWithApp";
import * as api from "../api/tauri";

vi.mock("../api/tauri", () => ({
  reportProfitLoss: vi.fn(),
  reportBalanceSheet: vi.fn(),
  reportTrialBalance: vi.fn(),
  reportArOpen: vi.fn(),
  reportApOpen: vi.fn(),
  reportGeneralLedger: vi.fn(),
  accountList: vi.fn(),
}));

describe("ReportsPage", () => {
  beforeEach(() => {
    vi.mocked(api.reportProfitLoss).mockResolvedValue({
      incomeLines: [],
      expenseLines: [],
      netIncomeMinor: 0,
    });
    vi.mocked(api.reportArOpen).mockResolvedValue([
      { displayName: "Acme Corp", openMinor: 5100 },
    ]);
    vi.mocked(api.accountList).mockResolvedValue([
      { id: 1, code: "1000", name: "Cash" },
    ]);
  });

  it("loads profit and loss report", async () => {
    renderWithApp(<ReportsPage />);
    fireEvent.click(screen.getByRole("button", { name: "Run" }));
    await waitFor(() => {
      expect(api.reportProfitLoss).toHaveBeenCalled();
    });
    expect(await screen.findByText(/Net income/i)).toBeDefined();
  });

  it("loads AR summary on tab switch", async () => {
    renderWithApp(<ReportsPage />);
    fireEvent.click(screen.getByRole("button", { name: "AR summary" }));
    fireEvent.click(
      screen.getByRole("button", { name: "Load AR (open balances)" }),
    );
    await waitFor(() => {
      expect(api.reportArOpen).toHaveBeenCalled();
      expect(screen.getByText("$51.00")).toBeDefined();
    });
  });

  it("loads chart of accounts when the general ledger tab is opened", async () => {
    renderWithApp(<ReportsPage />);
    fireEvent.click(screen.getByRole("button", { name: "General ledger" }));
    await waitFor(() => {
      expect(api.accountList).toHaveBeenCalled();
      expect(screen.getByRole("option", { name: /1000/ })).toBeDefined();
    });
  });
});
