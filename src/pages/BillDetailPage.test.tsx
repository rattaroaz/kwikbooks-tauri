// @vitest-environment happy-dom
import { screen } from "@testing-library/react";
import { describe, expect, it, vi, beforeEach } from "vitest";
import { BillDetailPage } from "./BillDetailPage";
import { renderWithApp } from "../test/renderWithApp";
import * as api from "../api/tauri";

vi.mock("../api/tauri", () => ({
  getBill: vi.fn(),
  billSetStatus: vi.fn(),
  billPost: vi.fn(),
}));

describe("BillDetailPage", () => {
  beforeEach(() => {
    vi.mocked(api.getBill).mockReset();
  });

  it("shows an error instead of spinning for an invalid id", async () => {
    renderWithApp(<BillDetailPage />, {
      route: "/bills/nope",
      path: "/bills/:id",
    });

    expect(await screen.findByText(/Invalid bill id/i)).toBeDefined();
    expect(screen.queryByText("Loading…")).toBeNull();
    expect(api.getBill).not.toHaveBeenCalled();
  });

  it("shows an error instead of spinning when load fails", async () => {
    vi.mocked(api.getBill).mockRejectedValue(new Error("missing"));

    renderWithApp(<BillDetailPage />, {
      route: "/bills/3",
      path: "/bills/:id",
    });

    expect(await screen.findByText(/Could not load this bill/i)).toBeDefined();
    expect(screen.queryByText("Loading…")).toBeNull();
  });
});
