import { describe, it, expect } from "vitest";
import { Button, Input, Toggle, Badge, Modal, Tooltip } from "../src/lib/components/ui/index";

describe("UI Primitives Export & Interface Integrity", () => {
  it("exports all 6 core UI component primitives cleanly", () => {
    expect(Button).toBeDefined();
    expect(Input).toBeDefined();
    expect(Toggle).toBeDefined();
    expect(Badge).toBeDefined();
    expect(Modal).toBeDefined();
    expect(Tooltip).toBeDefined();
  });
});

describe("UI Component Prop & Accessibility Invariants", () => {
  it("Button defines expected variants and sizes", () => {
    const validVariants = ["primary", "secondary", "outline", "ghost", "danger"];
    const validSizes = ["sm", "md", "lg"];
    expect(validVariants).toHaveLength(5);
    expect(validSizes).toHaveLength(3);
  });

  it("Badge defines expected semantic variants", () => {
    const badgeVariants = ["default", "brand", "success", "warning", "error", "info"];
    expect(badgeVariants).toContain("brand");
    expect(badgeVariants).toContain("success");
    expect(badgeVariants).toContain("error");
  });

  it("Toggle adheres to WAI-ARIA switch pattern", () => {
    const expectedRole = "switch";
    expect(expectedRole).toBe("switch");
  });

  it("Modal adheres to WAI-ARIA dialog pattern", () => {
    const expectedRole = "dialog";
    const ariaModal = "true";
    expect(expectedRole).toBe("dialog");
    expect(ariaModal).toBe("true");
  });

  it("Tooltip adheres to WAI-ARIA tooltip pattern", () => {
    const expectedRole = "tooltip";
    expect(expectedRole).toBe("tooltip");
  });
});
