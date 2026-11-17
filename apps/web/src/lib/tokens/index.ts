/**
 * Visual Design Tokens for web-repo-to-ai
 *
 * Strict atomic tokens defining colors, typography, spacing, elevations,
 * borders, and accessibility focus states across dark and light modes.
 */

export interface ColorScale {
  50: string;
  100: string;
  200: string;
  300: string;
  400: string;
  500: string;
  600: string;
  700: string;
  800: string;
  900: string;
  950: string;
}

export interface StatusColors {
  success: string;
  successLight: string;
  successDark: string;
  warning: string;
  warningLight: string;
  warningDark: string;
  error: string;
  errorLight: string;
  errorDark: string;
  info: string;
  infoLight: string;
  infoDark: string;
}

export interface TokenBudgetColors {
  safe: string;
  warning: string;
  critical: string;
}

export interface FontSizeToken {
  size: string;
  lineHeight: string;
}

export const colors = {
  brand: {
    50: "#eef2ff",
    100: "#e0e7ff",
    200: "#c7d2fe",
    300: "#a5b4fc",
    400: "#818cf8",
    500: "#6366f1", // Primary AI violet accent
    600: "#4f46e5",
    700: "#4338ca",
    800: "#3730a3",
    900: "#312e81",
    950: "#1e1b4b",
  } satisfies ColorScale,

  accent: {
    50: "#ecfeff",
    100: "#cffafe",
    200: "#a5f3fc",
    300: "#67e8f9",
    400: "#22d3ee",
    500: "#06b6d4", // Cyan for real-time SSE streaming indicators
    600: "#0891b2",
    700: "#0e7490",
    800: "#155e75",
    900: "#164e63",
    950: "#083344",
  } satisfies ColorScale,

  surface: {
    50: "#f8fafc",
    100: "#f1f5f9",
    200: "#e2e8f0",
    300: "#cbd5e1",
    400: "#94a3b8",
    500: "#64748b",
    600: "#475569",
    700: "#334155",
    800: "#1e293b",
    900: "#0f172a",
    950: "#090d16", // Primary dark canvas background
  } satisfies ColorScale,

  status: {
    success: "#10b981",
    successLight: "#d1fae5",
    successDark: "#065f46",
    warning: "#f59e0b",
    warningLight: "#fef3c7",
    warningDark: "#92400e",
    error: "#ef4444",
    errorLight: "#fee2e2",
    errorDark: "#991b1b",
    info: "#0284c7",
    infoLight: "#e0f2fe",
    infoDark: "#075985",
  } satisfies StatusColors,

  tokenBudget: {
    safe: "#10b981", // <= 80% of model token budget
    warning: "#f59e0b", // > 80% and <= 100% of budget
    critical: "#ef4444", // > 100% of budget (exceeded/truncated)
  } satisfies TokenBudgetColors,
} as const;

export const typography = {
  fontFamily: {
    sans: "'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif",
    mono: "'JetBrains Mono', 'Fira Code', ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace",
  },
  fontSize: {
    xs: { size: "0.75rem", lineHeight: "1rem" } as FontSizeToken, // 12px
    sm: { size: "0.875rem", lineHeight: "1.25rem" } as FontSizeToken, // 14px
    base: { size: "1rem", lineHeight: "1.5rem" } as FontSizeToken, // 16px
    lg: { size: "1.125rem", lineHeight: "1.75rem" } as FontSizeToken, // 18px
    xl: { size: "1.25rem", lineHeight: "1.75rem" } as FontSizeToken, // 20px
    "2xl": { size: "1.5rem", lineHeight: "2rem" } as FontSizeToken, // 24px
    "3xl": { size: "1.875rem", lineHeight: "2.25rem" } as FontSizeToken, // 30px
    "4xl": { size: "2.25rem", lineHeight: "2.5rem" } as FontSizeToken, // 36px
    "5xl": { size: "3rem", lineHeight: "1" } as FontSizeToken, // 48px
  },
  fontWeight: {
    regular: "400",
    medium: "500",
    semibold: "600",
    bold: "700",
  },
} as const;

export const spacing = {
  0: "0px",
  1: "4px",
  2: "8px",
  3: "12px",
  4: "16px",
  5: "20px",
  6: "24px",
  8: "32px",
  10: "40px",
  12: "48px",
  16: "64px",
  20: "80px",
  24: "96px",
} as const;

export const radii = {
  none: "0px",
  sm: "4px",
  md: "8px",
  lg: "12px",
  xl: "16px",
  "2xl": "24px",
  full: "9999px",
} as const;

export const elevations = {
  1: "0 1px 3px 0 rgb(0 0 0 / 0.1), 0 1px 2px -1px rgb(0 0 0 / 0.1)",
  2: "0 4px 6px -1px rgb(0 0 0 / 0.1), 0 2px 4px -2px rgb(0 0 0 / 0.1)",
  3: "0 10px 15px -3px rgb(0 0 0 / 0.1), 0 4px 6px -4px rgb(0 0 0 / 0.1)",
  4: "0 20px 25px -5px rgb(0 0 0 / 0.1), 0 8px 10px -6px rgb(0 0 0 / 0.1)",
} as const;

export const focusRing = {
  width: "2px",
  offset: "2px",
  color: colors.brand[500],
  darkColor: colors.brand[400],
} as const;

export const transitions = {
  default: "150ms cubic-bezier(0.4, 0, 0.2, 1)",
  fast: "100ms cubic-bezier(0.4, 0, 0.2, 1)",
  smooth: "250ms cubic-bezier(0.4, 0, 0.2, 1)",
} as const;

/**
 * Return semantic token budget status key and hex color based on token consumption ratio
 */
export function getTokenBudgetStatus(
  usedTokens: number,
  maxBudget: number
): { status: "safe" | "warning" | "critical"; color: string; ratio: number } {
  if (maxBudget <= 0) {
    return { status: "safe", color: colors.tokenBudget.safe, ratio: 0 };
  }
  const ratio = usedTokens / maxBudget;
  if (ratio > 1.0) {
    return { status: "critical", color: colors.tokenBudget.critical, ratio };
  }
  if (ratio > 0.8) {
    return { status: "warning", color: colors.tokenBudget.warning, ratio };
  }
  return { status: "safe", color: colors.tokenBudget.safe, ratio };
}

/**
 * Compute the relative luminance of a hex color for WCAG contrast calculations.
 */
export function getRelativeLuminance(hex: string): number {
  const cleanHex = hex.replace("#", "");
  const r = parseInt(cleanHex.substring(0, 2), 16) / 255;
  const g = parseInt(cleanHex.substring(2, 4), 16) / 255;
  const b = parseInt(cleanHex.substring(4, 6), 16) / 255;

  const toLinear = (c: number) => (c <= 0.03928 ? c / 12.92 : Math.pow((c + 0.055) / 1.055, 2.4));

  return 0.2126 * toLinear(r) + 0.7152 * toLinear(g) + 0.0722 * toLinear(b);
}

/**
 * Compute WCAG 2.1 contrast ratio between two hex colors.
 */
export function getContrastRatio(hex1: string, hex2: string): number {
  const lum1 = getRelativeLuminance(hex1);
  const lum2 = getRelativeLuminance(hex2);
  const brightest = Math.max(lum1, lum2);
  const darkest = Math.min(lum1, lum2);
  return (brightest + 0.05) / (darkest + 0.05);
}

/**
 * Export all design tokens as flat CSS variable map.
 */
export function toCssVariableMap(): Record<string, string> {
  const map: Record<string, string> = {};

  for (const [shade, val] of Object.entries(colors.brand)) {
    map[`--color-brand-${shade}`] = val;
  }
  for (const [shade, val] of Object.entries(colors.accent)) {
    map[`--color-accent-${shade}`] = val;
  }
  for (const [shade, val] of Object.entries(colors.surface)) {
    map[`--color-surface-${shade}`] = val;
  }
  for (const [key, val] of Object.entries(colors.status)) {
    map[`--color-status-${key}`] = val;
  }
  for (const [key, val] of Object.entries(colors.tokenBudget)) {
    map[`--color-budget-${key}`] = val;
  }
  for (const [step, val] of Object.entries(spacing)) {
    map[`--spacing-${step}`] = val;
  }
  for (const [name, val] of Object.entries(radii)) {
    map[`--radius-${name}`] = val;
  }
  for (const [level, val] of Object.entries(elevations)) {
    map[`--elevation-${level}`] = val;
  }

  map["--font-sans"] = typography.fontFamily.sans;
  map["--font-mono"] = typography.fontFamily.mono;
  map["--focus-ring-width"] = focusRing.width;
  map["--focus-ring-offset"] = focusRing.offset;
  map["--focus-ring-color"] = focusRing.color;

  return map;
}

export const designTokens = {
  colors,
  typography,
  spacing,
  radii,
  elevations,
  focusRing,
  transitions,
  getTokenBudgetStatus,
  getRelativeLuminance,
  getContrastRatio,
  toCssVariableMap,
} as const;

export default designTokens;
