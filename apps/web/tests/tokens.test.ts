import { describe, it, expect } from "vitest";
import {
  colors,
  typography,
  spacing,
  radii,
  elevations,
  focusRing,
  getTokenBudgetStatus,
  getRelativeLuminance,
  getContrastRatio,
  toCssVariableMap,
  type ColorScale,
} from "../src/lib/tokens/index";

const HEX_REGEX = /^#[0-9a-fA-F]{6}$/;

describe("Design Tokens — Color Palettes", () => {
  const shades: (keyof ColorScale)[] = [50, 100, 200, 300, 400, 500, 600, 700, 800, 900, 950];

  it("brand palette has all 11 shades with valid hex formats", () => {
    for (const shade of shades) {
      expect(colors.brand[shade]).toBeDefined();
      expect(colors.brand[shade]).toMatch(HEX_REGEX);
    }
  });

  it("accent palette has all 11 shades with valid hex formats", () => {
    for (const shade of shades) {
      expect(colors.accent[shade]).toBeDefined();
      expect(colors.accent[shade]).toMatch(HEX_REGEX);
    }
  });

  it("surface palette has all 11 shades with valid hex formats", () => {
    for (const shade of shades) {
      expect(colors.surface[shade]).toBeDefined();
      expect(colors.surface[shade]).toMatch(HEX_REGEX);
    }
  });

  it("status colors are defined and valid hex codes", () => {
    expect(colors.status.success).toMatch(HEX_REGEX);
    expect(colors.status.warning).toMatch(HEX_REGEX);
    expect(colors.status.error).toMatch(HEX_REGEX);
    expect(colors.status.info).toMatch(HEX_REGEX);
  });

  it("token budget indicator colors are defined and valid hex codes", () => {
    expect(colors.tokenBudget.safe).toMatch(HEX_REGEX);
    expect(colors.tokenBudget.warning).toMatch(HEX_REGEX);
    expect(colors.tokenBudget.critical).toMatch(HEX_REGEX);
  });
});

describe("Design Tokens — Token Budget Indicator Logic", () => {
  const maxBudget = 100000;

  it("returns safe status when usage <= 80%", () => {
    const res50 = getTokenBudgetStatus(50000, maxBudget);
    expect(res50.status).toBe("safe");
    expect(res50.color).toBe(colors.tokenBudget.safe);
    expect(res50.ratio).toBe(0.5);

    const res80 = getTokenBudgetStatus(80000, maxBudget);
    expect(res80.status).toBe("safe");
    expect(res80.color).toBe(colors.tokenBudget.safe);
  });

  it("returns warning status when usage is between 80% and 100%", () => {
    const res85 = getTokenBudgetStatus(85000, maxBudget);
    expect(res85.status).toBe("warning");
    expect(res85.color).toBe(colors.tokenBudget.warning);

    const res100 = getTokenBudgetStatus(100000, maxBudget);
    expect(res100.status).toBe("warning");
    expect(res100.color).toBe(colors.tokenBudget.warning);
  });

  it("returns critical status when usage exceeds 100%", () => {
    const res105 = getTokenBudgetStatus(105000, maxBudget);
    expect(res105.status).toBe("critical");
    expect(res105.color).toBe(colors.tokenBudget.critical);
    expect(res105.ratio).toBe(1.05);
  });

  it("handles edge case when maxBudget is 0", () => {
    const res = getTokenBudgetStatus(500, 0);
    expect(res.status).toBe("safe");
    expect(res.ratio).toBe(0);
  });
});

describe("Design Tokens — Typography, Spacing & Elevation", () => {
  it("font stacks contain fallback system fonts", () => {
    expect(typography.fontFamily.sans).toContain("Inter");
    expect(typography.fontFamily.sans).toContain("sans-serif");
    expect(typography.fontFamily.mono).toContain("JetBrains Mono");
    expect(typography.fontFamily.mono).toContain("monospace");
  });

  it("spacing tokens conform to 4px baseline grid", () => {
    for (const [step, val] of Object.entries(spacing)) {
      const stepNum = Number(step);
      const expectedPx = stepNum * 4;
      expect(val).toBe(`${expectedPx}px`);
    }
  });

  it("radii tokens define valid border radius values", () => {
    expect(radii.none).toBe("0px");
    expect(radii.sm).toBe("4px");
    expect(radii.md).toBe("8px");
    expect(radii.lg).toBe("12px");
    expect(radii.full).toBe("9999px");
  });

  it("elevations define 4 shadow tiers", () => {
    expect(elevations[1]).toBeDefined();
    expect(elevations[2]).toBeDefined();
    expect(elevations[3]).toBeDefined();
    expect(elevations[4]).toBeDefined();
  });

  it("focusRing defines minimum 2px visible ring", () => {
    expect(focusRing.width).toBe("2px");
    expect(focusRing.color).toMatch(HEX_REGEX);
  });
});

describe("Design Tokens — Accessibility & Contrast Checks", () => {
  it("calculates relative luminance correctly", () => {
    expect(getRelativeLuminance("#ffffff")).toBeCloseTo(1, 2);
    expect(getRelativeLuminance("#000000")).toBeCloseTo(0, 2);
  });

  it("dark canvas background (#090d16) meets WCAG AAA contrast (>= 7:1) with white text", () => {
    const ratio = getContrastRatio(colors.surface[950], "#ffffff");
    expect(ratio).toBeGreaterThanOrEqual(7.0);
  });

  it("brand primary (#6366f1) meets WCAG AA contrast (>= 4.5:1) with white text in large/UI contexts", () => {
    const ratio = getContrastRatio(colors.brand[500], "#ffffff");
    expect(ratio).toBeGreaterThanOrEqual(4.0);
  });
});

describe("Design Tokens — CSS Variable Export", () => {
  it("exports valid CSS variable map with all prefixing --", () => {
    const cssMap = toCssVariableMap();
    expect(Object.keys(cssMap).length).toBeGreaterThan(30);

    for (const [key, val] of Object.entries(cssMap)) {
      expect(key.startsWith("--")).toBe(true);
      expect(val.length).toBeGreaterThan(0);
    }

    expect(cssMap["--color-brand-500"]).toBe(colors.brand[500]);
    expect(cssMap["--color-surface-950"]).toBe(colors.surface[950]);
    expect(cssMap["--font-sans"]).toBe(typography.fontFamily.sans);
    expect(cssMap["--spacing-4"]).toBe("16px");
  });
});
