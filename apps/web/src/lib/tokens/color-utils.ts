/**
 * Color and Accessibility Utilities for web-repo-to-ai Design System
 */

import { getRelativeLuminance, getContrastRatio, colors } from "./index";

export { getRelativeLuminance, getContrastRatio };

/**
 * Check if a color pair passes WCAG 2.1 AA contrast requirements.
 * - Normal text requires >= 4.5:1
 * - Large text (>= 18pt / 24px or bold >= 14pt / 18.66px) or UI controls require >= 3.0:1
 */
export function isWcagAaCompliant(
  fgHex: string,
  bgHex: string,
  isLargeText: boolean = false
): boolean {
  const ratio = getContrastRatio(fgHex, bgHex);
  const threshold = isLargeText ? 3.0 : 4.5;
  return ratio >= threshold;
}

/**
 * Check if a color pair passes WCAG 2.1 AAA contrast requirements.
 * - Normal text requires >= 7.0:1
 * - Large text requires >= 4.5:1
 */
export function isWcagAaaCompliant(
  fgHex: string,
  bgHex: string,
  isLargeText: boolean = false
): boolean {
  const ratio = getContrastRatio(fgHex, bgHex);
  const threshold = isLargeText ? 4.5 : 7.0;
  return ratio >= threshold;
}

/**
 * Automatically determine whether light or dark text provides better accessible contrast
 * against a given background color.
 */
export function getAccessibleTextColor(
  bgHex: string,
  darkColor: string = colors.surface[950],
  lightColor: string = "#ffffff"
): string {
  const lightContrast = getContrastRatio(lightColor, bgHex);
  const darkContrast = getContrastRatio(darkColor, bgHex);
  return lightContrast >= darkContrast ? lightColor : darkColor;
}

/**
 * Map token budget usage ratio to semantic CSS class for styling badges and status counters.
 */
export function getTokenBudgetClasses(ratio: number): {
  textClass: string;
  bgClass: string;
  borderClass: string;
  badgeClass: string;
} {
  if (ratio > 1.0) {
    return {
      textClass: "text-status-error dark:text-status-error-light",
      bgClass: "bg-status-error-light dark:bg-status-error-dark/20",
      borderClass: "border-status-error",
      badgeClass:
        "bg-status-error-light text-status-error-dark dark:bg-status-error-dark/30 dark:text-red-200 border-status-error/40",
    };
  }
  if (ratio > 0.8) {
    return {
      textClass: "text-status-warning dark:text-status-warning-light",
      bgClass: "bg-status-warning-light dark:bg-status-warning-dark/20",
      borderClass: "border-status-warning",
      badgeClass:
        "bg-status-warning-light text-status-warning-dark dark:bg-status-warning-dark/30 dark:text-amber-200 border-status-warning/40",
    };
  }
  return {
    textClass: "text-status-success dark:text-status-success-light",
    bgClass: "bg-status-success-light dark:bg-status-success-dark/20",
    borderClass: "border-status-success",
    badgeClass:
      "bg-status-success-light text-status-success-dark dark:bg-status-success-dark/30 dark:text-emerald-200 border-status-success/40",
  };
}

/**
 * Universal visible focus ring classes enforcing WCAG 2.1 AA focus visible standards.
 */
export const focusVisibleClasses =
  "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-brand-500 focus-visible:ring-offset-2 dark:focus-visible:ring-brand-400 dark:focus-visible:ring-offset-surface-900";
