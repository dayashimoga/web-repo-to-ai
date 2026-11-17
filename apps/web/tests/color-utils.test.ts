import { describe, it, expect } from "vitest";
import {
  isWcagAaCompliant,
  isWcagAaaCompliant,
  getAccessibleTextColor,
  getTokenBudgetClasses,
  focusVisibleClasses,
} from "../src/lib/tokens/color-utils";
import { colors } from "../src/lib/tokens/index";

describe("Color & Accessibility Utilities", () => {
  describe("isWcagAaCompliant", () => {
    it("passes for high-contrast pairs (white on dark canvas)", () => {
      expect(isWcagAaCompliant("#ffffff", colors.surface[950])).toBe(true);
      expect(isWcagAaCompliant(colors.surface[950], "#ffffff")).toBe(true);
    });

    it("passes for brand primary against white with large text threshold", () => {
      expect(isWcagAaCompliant(colors.brand[500], "#ffffff", true)).toBe(true);
    });

    it("fails for low contrast pairs", () => {
      expect(isWcagAaCompliant("#888888", "#999999")).toBe(false);
      expect(isWcagAaCompliant(colors.surface[400], "#ffffff")).toBe(false);
    });
  });

  describe("isWcagAaaCompliant", () => {
    it("passes for black/white maximum contrast", () => {
      expect(isWcagAaaCompliant("#ffffff", "#000000")).toBe(true);
      expect(isWcagAaaCompliant("#ffffff", colors.surface[950])).toBe(true);
    });

    it("distinguishes normal vs large text requirements", () => {
      // Brand-600 (~6.28:1) passes large AAA (>=4.5), but fails normal AAA (>=7.0)
      const mockFg = colors.brand[600];
      const mockBg = "#ffffff";
      const isLargePass = isWcagAaaCompliant(mockFg, mockBg, true);
      const isNormalPass = isWcagAaaCompliant(mockFg, mockBg, false);
      expect(isLargePass).toBe(true);
      expect(isNormalPass).toBe(false);
    });
  });

  describe("getAccessibleTextColor", () => {
    it("picks white text for dark canvas background", () => {
      const textColor = getAccessibleTextColor(colors.surface[950]);
      expect(textColor).toBe("#ffffff");
    });

    it("picks dark text for light canvas background", () => {
      const textColor = getAccessibleTextColor(colors.surface[50]);
      expect(textColor).toBe(colors.surface[950]);
    });
  });

  describe("getTokenBudgetClasses", () => {
    it("returns success classes for <= 80% usage", () => {
      const classes = getTokenBudgetClasses(0.75);
      expect(classes.textClass).toContain("text-status-success");
      expect(classes.borderClass).toContain("border-status-success");
    });

    it("returns warning classes for 80% to 100% usage", () => {
      const classes = getTokenBudgetClasses(0.92);
      expect(classes.textClass).toContain("text-status-warning");
      expect(classes.borderClass).toContain("border-status-warning");
    });

    it("returns error classes for > 100% usage", () => {
      const classes = getTokenBudgetClasses(1.15);
      expect(classes.textClass).toContain("text-status-error");
      expect(classes.borderClass).toContain("border-status-error");
    });
  });

  describe("focusVisibleClasses", () => {
    it("contains essential focus ring and offset classes", () => {
      expect(focusVisibleClasses).toContain("focus-visible:outline-none");
      expect(focusVisibleClasses).toContain("focus-visible:ring-2");
      expect(focusVisibleClasses).toContain("focus-visible:ring-offset-2");
    });
  });
});
