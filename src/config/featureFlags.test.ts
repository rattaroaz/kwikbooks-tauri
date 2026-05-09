import { describe, expect, it } from "vitest";
import { featureFlags } from "./featureFlags";

describe("featureFlags", () => {
  it("exposes boolean toggles", () => {
    expect(typeof featureFlags.experimentalUi).toBe("boolean");
  });
});
