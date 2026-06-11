// @vitest-environment happy-dom
import { fireEvent, screen, waitFor, within } from "@testing-library/react";
import { describe, expect, it, vi, beforeEach } from "vitest";
import { AccountsPage } from "./AccountsPage";
import { renderWithApp } from "../test/renderWithApp";
import * as api from "../api/tauri";

vi.mock("../api/tauri", () => ({
  accountList: vi.fn(),
  accountCreate: vi.fn(),
  accountUpdate: vi.fn(),
  accountDeactivate: vi.fn(),
}));

describe("AccountsPage", () => {
  beforeEach(() => {
    vi.mocked(api.accountList).mockResolvedValue([
      {
        id: 1,
        code: "5000",
        name: "Expenses",
        accountType: "expense",
        isBankCash: false,
        isActive: true,
      },
    ]);
  });

  it("loads and lists accounts", async () => {
    renderWithApp(<AccountsPage />);
    await waitFor(() => {
      expect(screen.getByText("Expenses")).toBeDefined();
    });
    expect(api.accountList).toHaveBeenCalled();
  });

  it("creates account on submit", async () => {
    vi.mocked(api.accountCreate).mockResolvedValue(2);
    renderWithApp(<AccountsPage />);
    await waitFor(() => expect(screen.getByText("Expenses")).toBeDefined());
    const form = screen
      .getByRole("heading", { name: "Add account" })
      .closest("form")!;
    fireEvent.change(within(form).getByLabelText(/^Code/i), {
      target: { value: "6000" },
    });
    fireEvent.change(within(form).getByLabelText(/^Name/i), {
      target: { value: "Utilities" },
    });
    fireEvent.click(within(form).getByRole("button", { name: "Create" }));
    await waitFor(() => {
      expect(api.accountCreate).toHaveBeenCalled();
    });
  });
});
