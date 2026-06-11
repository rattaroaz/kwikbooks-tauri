// @vitest-environment happy-dom
import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { PageLoading } from "./PageLoading";

describe("PageLoading", () => {
  it("shows label and busy state", () => {
    render(<PageLoading label="Loading accounts…" />);
    expect(screen.getByText("Loading accounts…")).toBeDefined();
    expect(document.querySelector("[aria-busy='true']")).toBeTruthy();
  });
});
