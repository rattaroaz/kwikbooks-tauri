// @vitest-environment happy-dom
import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { UpdateDialog } from "./UpdateDialog";
import {
  closeUpdateDialog,
  openUpdateDialog,
  setUpdateDialog,
} from "../stores/updateDialogStore";

describe("UpdateDialog", () => {
  afterEach(() => {
    cleanup();
    closeUpdateDialog();
  });

  it("renders checking title and message", () => {
    openUpdateDialog();
    render(<UpdateDialog />);
    expect(screen.getByRole("dialog")).toBeDefined();
    expect(
      screen.getByRole("heading", { name: "Checking for updates" }),
    ).toBeDefined();
    expect(screen.getByTestId("update-dialog-message").textContent).toMatch(
      /Checking for updates/,
    );
  });

  it("shows close when not busy", () => {
    openUpdateDialog();
    setUpdateDialog({ phase: "up_to_date", message: "Up to date" });
    render(<UpdateDialog />);
    expect(screen.getByTestId("update-dialog-close")).toBeDefined();
  });
});
