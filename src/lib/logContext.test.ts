import { describe, expect, it } from "vitest";
import { logContext } from "./logContext";

describe("logContext", () => {
  it("builds dotted component.action labels", () => {
    expect(logContext("SettingsPage", "backup")).toBe("SettingsPage.backup");
  });
});
