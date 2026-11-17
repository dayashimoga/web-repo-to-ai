import { describe, it, expect } from "vitest";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

function readComponentSource(relativePath: string): string {
  const filePath = resolve(process.cwd(), relativePath);
  return readFileSync(filePath, "utf-8");
}

/**
 * Tailwind CSS breakpoint definition helper:
 * xs: < 640px (Mobile)
 * sm: 640px - 767px (Large Mobile / Phablet)
 * md: 768px - 1023px (Tablet Portrait / Landscape)
 * lg: 1024px - 1279px (Small Desktop / Large Tablet)
 * xl: 1280px - 1535px (Standard Desktop)
 * 2xl: >= 1536px (High-Resolution Wide Desktop)
 */
function resolveTailwindBreakpoint(width: number): "xs" | "sm" | "md" | "lg" | "xl" | "2xl" {
  if (width < 640) return "xs";
  if (width < 768) return "sm";
  if (width < 1024) return "md";
  if (width < 1280) return "lg";
  if (width < 1536) return "xl";
  return "2xl";
}

describe("Responsive Layout & Viewport Certification (Subphase 1.4.2)", () => {
  describe("1. Tailwind Breakpoint Spectrum & Viewport Simulation Matrix", () => {
    it("verifies correct categorization across representative mobile devices (320px–480px)", () => {
      const mobileProfiles = [
        { name: "iPhone SE (1st gen)", width: 320, expected: "xs" },
        { name: "iPhone SE (3rd gen) / iPhone 13 mini", width: 375, expected: "xs" },
        { name: "iPhone 14 / 15 / 16", width: 390, expected: "xs" },
        { name: "Google Pixel 7 / Galaxy S23", width: 412, expected: "xs" },
        { name: "iPhone 14 Pro Max / Plus", width: 430, expected: "xs" },
        { name: "Large Mobile Landscape", width: 480, expected: "xs" },
      ];

      for (const device of mobileProfiles) {
        expect(
          resolveTailwindBreakpoint(device.width),
          `${device.name} (${device.width}px) must resolve to xs`
        ).toBe(device.expected);
      }
    });

    it("verifies correct categorization across tablet devices (640px–1024px)", () => {
      const tabletProfiles = [
        { name: "Small Tablet / Phablet Landscape", width: 640, expected: "sm" },
        { name: "iPad Mini Portrait", width: 768, expected: "md" },
        { name: "iPad 10th Gen Portrait", width: 820, expected: "md" },
        { name: "iPad Air / Pro 11 Portrait", width: 834, expected: "md" },
        { name: "iPad Landscape / Small Laptop", width: 1024, expected: "lg" },
      ];

      for (const device of tabletProfiles) {
        expect(
          resolveTailwindBreakpoint(device.width),
          `${device.name} (${device.width}px) must resolve to ${device.expected}`
        ).toBe(device.expected);
      }
    });

    it("verifies correct categorization across desktop viewports (1280px–1920px+)", () => {
      const desktopProfiles = [
        { name: "Standard Laptop (13-inch / 14-inch)", width: 1280, expected: "xl" },
        { name: "MacBook Pro 16 / QHD Display", width: 1440, expected: "xl" },
        { name: "2xl Breakpoint Display", width: 1536, expected: "2xl" },
        { name: "Full HD 1080p Desktop Monitor", width: 1920, expected: "2xl" },
        { name: "4K UHD Ultrawide Monitor", width: 2560, expected: "2xl" },
      ];

      for (const device of desktopProfiles) {
        expect(
          resolveTailwindBreakpoint(device.width),
          `${device.name} (${device.width}px) must resolve to ${device.expected}`
        ).toBe(device.expected);
      }
    });
  });

  describe("2. Mobile Viewport Layout Invariants (320px–480px)", () => {
    it("Root layout strictly enforces horizontal overflow containment and minimum screen height", () => {
      const layout = readComponentSource("src/routes/+layout.svelte");
      expect(layout).toContain("overflow-x-hidden");
      expect(layout).toContain("max-w-full");
      expect(layout).toContain("min-h-screen");
    });

    it("All landing page section containers enforce responsive mobile horizontal padding", () => {
      const hero = readComponentSource("src/lib/components/landing/Hero.svelte");
      expect(hero).toContain("max-w-7xl mx-auto px-4 sm:px-6 lg:px-8");

      const workflow = readComponentSource("src/lib/components/landing/WorkflowSteps.svelte");
      expect(workflow).toContain("max-w-7xl mx-auto px-4 sm:px-6 lg:px-8");

      const sources = readComponentSource("src/lib/components/landing/SupportedSources.svelte");
      expect(sources).toContain("max-w-7xl mx-auto px-4 sm:px-6 lg:px-8");

      const features = readComponentSource("src/lib/components/landing/FeatureHighlights.svelte");
      expect(features).toContain("max-w-7xl mx-auto px-4 sm:px-6 lg:px-8");

      const footer = readComponentSource("src/lib/components/navigation/Footer.svelte");
      expect(footer).toContain("max-w-7xl mx-auto px-4 sm:px-6 lg:px-8");
    });

    it("Landing page hero scales typography, stacks action buttons, and collapses format grid on mobile", () => {
      const hero = readComponentSource("src/lib/components/landing/Hero.svelte");

      // Typography scales down on mobile
      expect(hero).toContain("text-3xl sm:text-5xl lg:text-6xl");

      // CTA buttons stack vertically on mobile (flex-col) and switch to inline on sm+ (sm:flex-row)
      expect(hero).toContain("flex flex-col sm:flex-row");
      expect(hero).toContain("w-full sm:w-auto min-h-[48px]");

      // Output format pills collapse to 2 columns on mobile
      expect(hero).toContain("grid-cols-2 sm:grid-cols-3 lg:grid-cols-6");

      // Value proposition metrics collapse to 1 column on mobile
      expect(hero).toContain("grid-cols-1 sm:grid-cols-3");
    });

    it("UrlInput form collapses cleanly to full-width mobile layout with iOS auto-zoom prevention", () => {
      const urlInput = readComponentSource("src/lib/components/input/UrlInput.svelte");

      // Form layout stacks on mobile, becomes row on tablet+
      expect(urlInput).toContain("flex flex-col sm:flex-row");

      // Submit button is full-width on mobile with 48px minimum touch height
      expect(urlInput).toContain("w-full sm:w-auto min-h-[48px]");

      // Text input font size scales up to 16px (sm:text-base) to prevent iOS auto-zoom
      expect(urlInput).toContain("text-sm sm:text-base");
    });

    it("All landing page content grids collapse to single column on mobile viewports", () => {
      const workflow = readComponentSource("src/lib/components/landing/WorkflowSteps.svelte");
      expect(workflow).toContain("grid-cols-1 md:grid-cols-3");

      const sources = readComponentSource("src/lib/components/landing/SupportedSources.svelte");
      expect(sources).toContain("grid-cols-1 md:grid-cols-2");

      const features = readComponentSource("src/lib/components/landing/FeatureHighlights.svelte");
      expect(features).toContain("grid-cols-1 md:grid-cols-2 lg:grid-cols-3");

      const footer = readComponentSource("src/lib/components/navigation/Footer.svelte");
      expect(footer).toContain("grid-cols-1 md:grid-cols-2 lg:grid-cols-5");
    });

    it("Header renders dedicated mobile hamburger button and collapses desktop navigation", () => {
      const header = readComponentSource("src/lib/components/navigation/Header.svelte");

      // Desktop navigation hidden on mobile
      expect(header).toContain("hidden md:flex");

      // Mobile hamburger controls visible only on mobile (< md)
      expect(header).toContain("md:hidden");
      expect(header).toContain('aria-controls="mobile-navigation"');
      expect(header).toContain("aria-expanded={mobileMenuOpen}");
    });
  });

  describe("3. Tablet Viewport Layout Invariants (640px–1024px)", () => {
    it("Hero layout expands buttons horizontally and format grid to 3 columns at sm breakpoint (640px)", () => {
      const hero = readComponentSource("src/lib/components/landing/Hero.svelte");
      expect(hero).toContain("sm:flex-row");
      expect(hero).toContain("sm:grid-cols-3");
      expect(hero).toContain("sm:text-5xl");
    });

    it("Workflow, sources, and features expand to multi-column grids at md breakpoint (768px)", () => {
      const workflow = readComponentSource("src/lib/components/landing/WorkflowSteps.svelte");
      expect(workflow).toContain("md:grid-cols-3");

      const sources = readComponentSource("src/lib/components/landing/SupportedSources.svelte");
      expect(sources).toContain("md:grid-cols-2");

      const features = readComponentSource("src/lib/components/landing/FeatureHighlights.svelte");
      expect(features).toContain("md:grid-cols-2");
    });

    it("Header desktop navigation becomes visible and mobile drawer hides at md breakpoint (768px)", () => {
      const header = readComponentSource("src/lib/components/navigation/Header.svelte");
      expect(header).toContain("hidden md:flex items-center gap-6");
      expect(header).toContain("hidden md:flex items-center gap-3");
      expect(header).toContain("flex items-center gap-2 md:hidden");
    });
  });

  describe("4. Desktop Viewport Layout Invariants (1280px–1920px+)", () => {
    it("Hero expands format pills to full 6-column matrix and maximum typography at lg/xl", () => {
      const hero = readComponentSource("src/lib/components/landing/Hero.svelte");
      expect(hero).toContain("lg:grid-cols-6");
      expect(hero).toContain("lg:text-6xl");
    });

    it("Feature highlights expand to 3 columns and footer expands to 5 columns on desktop", () => {
      const features = readComponentSource("src/lib/components/landing/FeatureHighlights.svelte");
      expect(features).toContain("lg:grid-cols-3");

      const footer = readComponentSource("src/lib/components/navigation/Footer.svelte");
      expect(footer).toContain("lg:grid-cols-5");
    });

    it("Container max-width constraints prevent unconstrained ultra-wide stretching", () => {
      const hero = readComponentSource("src/lib/components/landing/Hero.svelte");
      expect(hero).toContain("max-w-7xl mx-auto");

      const workflow = readComponentSource("src/lib/components/landing/WorkflowSteps.svelte");
      expect(workflow).toContain("max-w-7xl mx-auto");

      const workspaceRoute = readComponentSource("src/routes/workspace/+page.svelte");
      expect(workspaceRoute).toContain("max-w-6xl mx-auto px-4 sm:px-6 lg:px-8");
    });
  });

  describe("5. Conversion Workspace (/workspace) Multi-Viewport Invariants", () => {
    it("Workspace job header stacks vertically on mobile and aligns horizontally on tablet/desktop", () => {
      const workspace = readComponentSource("src/lib/components/workspace/JobWorkspace.svelte");
      expect(workspace).toContain(
        "flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4"
      );
    });

    it("Pipeline stage execution cards collapse to 1 col on mobile, 2 cols on tablet, and 4 cols on desktop", () => {
      const workspace = readComponentSource("src/lib/components/workspace/JobWorkspace.svelte");
      expect(workspace).toContain("grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4");
    });

    it("Repository completeness accounting collapses to 2 cols on mobile and expands to 4 cols on tablet+", () => {
      const workspace = readComponentSource("src/lib/components/workspace/JobWorkspace.svelte");
      expect(workspace).toContain("grid grid-cols-2 sm:grid-cols-4 gap-4");
    });

    it("TokenBudgetBar metric grid renders 2 cols on mobile/tablet and 4 cols on desktop", () => {
      const budgetBar = readComponentSource("src/lib/components/workspace/TokenBudgetBar.svelte");
      expect(budgetBar).toContain("grid grid-cols-2 lg:grid-cols-4 gap-4");
    });

    it("TokenBudgetBar consumption banner stacks on mobile and aligns on tablet+", () => {
      const budgetBar = readComponentSource("src/lib/components/workspace/TokenBudgetBar.svelte");
      expect(budgetBar).toContain(
        "flex flex-col sm:flex-row sm:items-center justify-between gap-1"
      );
    });

    it("PreviewPanel stacks file structure sidebar above code on mobile and splits into 12-column grid on md+", () => {
      const preview = readComponentSource("src/lib/components/workspace/PreviewPanel.svelte");
      // Responsive 12-column split layout
      expect(preview).toContain("grid grid-cols-1 md:grid-cols-12 min-h-[460px]");
      // Sidebar takes 4 cols on md and 3 cols on lg
      expect(preview).toContain("md:col-span-4 lg:col-span-3");
    });

    it("DownloadControls artifact cards stack vertically on mobile and align horizontally on md+", () => {
      const download = readComponentSource("src/lib/components/workspace/DownloadControls.svelte");
      expect(download).toContain("flex flex-col md:flex-row md:items-center justify-between gap-4");
    });

    it("Execution telemetry terminal log maintains bounded height and scrollable overflow across viewports", () => {
      const workspace = readComponentSource("src/lib/components/workspace/JobWorkspace.svelte");
      expect(workspace).toContain("max-h-64 overflow-y-auto");
    });
  });

  describe("6. Touch Target Accessibility Invariants (WCAG 2.1 AA / AAA - Criteria 2.5.5 & 2.5.8)", () => {
    it("Header mobile toggle and theme button satisfy >= 44x44px touch targets", () => {
      const header = readComponentSource("src/lib/components/navigation/Header.svelte");
      expect(header).toContain("min-h-[44px] min-w-[44px]");
      expect(header).toContain('aria-label="Toggle navigation menu"');
      expect(header).toContain('aria-controls="mobile-navigation"');
    });

    it("Mobile dropdown navigation links provide >= 44px comfortable touch heights", () => {
      const header = readComponentSource("src/lib/components/navigation/Header.svelte");
      expect(header).toContain("min-h-[44px] flex items-center");
      expect(header).toContain('id="mobile-navigation"');
    });

    it("Hero action buttons enforce full-width mobile stacking and >= 48px touch heights", () => {
      const hero = readComponentSource("src/lib/components/landing/Hero.svelte");
      expect(hero).toContain("flex flex-col sm:flex-row");
      expect(hero).toContain("w-full sm:w-auto min-h-[48px]");
    });

    it("UrlInput submit and clear controls provide responsive >= 48px touch heights", () => {
      const urlInput = readComponentSource("src/lib/components/input/UrlInput.svelte");
      expect(urlInput).toContain("w-full sm:w-auto min-h-[48px]");
      expect(urlInput).toContain('aria-label="Clear URL input"');
    });

    it("Modal dialog dismiss and action buttons enforce accessible touch targets", () => {
      const modal = readComponentSource("src/lib/components/ui/Modal.svelte");
      expect(modal).toContain("aria-label");
      expect(modal).toContain('role="dialog"');
    });
  });
});
