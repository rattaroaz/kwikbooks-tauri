// @vitest-environment happy-dom
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";
import { describe, expect, it, vi } from "vitest";
import { GlobalSearch } from "./GlobalSearch";

const { globalSearchMock, reportErrorMock } = vi.hoisted(() => ({
  globalSearchMock: vi.fn(),
  reportErrorMock: vi.fn(),
}));

vi.mock("../api/tauri", () => ({
  globalSearch: globalSearchMock,
}));

vi.mock("../lib/reportError", () => ({
  reportError: reportErrorMock,
}));

describe("GlobalSearch", () => {
  it("reports search failures to the host log pipeline", async () => {
    globalSearchMock.mockRejectedValue(new Error("search failed"));
    reportErrorMock.mockImplementation((_ctx, _err, notify) => {
      notify("search failed");
    });

    render(
      <MemoryRouter>
        <GlobalSearch />
      </MemoryRouter>,
    );

    fireEvent.click(screen.getByRole("button", { name: /search/i }));
    fireEvent.change(screen.getByPlaceholderText(/search accounts/i), {
      target: { value: "acme" },
    });

    await waitFor(() => {
      expect(reportErrorMock).toHaveBeenCalledWith(
        "GlobalSearch.search",
        expect.any(Error),
        expect.any(Function),
      );
    });
    expect(screen.getByText("search failed")).toBeDefined();
  });
});
