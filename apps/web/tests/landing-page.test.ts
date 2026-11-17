import { describe, it, expect } from "vitest";
import { Header, Footer } from "../src/lib/components/navigation/index";
import {
  Hero,
  WorkflowSteps,
  SupportedSources,
  FeatureHighlights,
} from "../src/lib/components/landing/index";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

describe("Navigation Components Integrity", () => {
  it("exports Header and Footer navigation components cleanly", () => {
    expect(Header).toBeDefined();
    expect(Footer).toBeDefined();
  });

  it("Header component contains required semantic landmarks and navigation links", () => {
    const headerPath = resolve(process.cwd(), "src/lib/components/navigation/Header.svelte");
    const headerContent = readFileSync(headerPath, "utf-8");

    // Semantic landmark header
    expect(headerContent).toContain("<header");
    expect(headerContent).toContain("</header>");

    // Desktop nav with ARIA label
    expect(headerContent).toContain('aria-label="Main Navigation"');

    // Section anchor links
    expect(headerContent).toContain('href="#convert"');
    expect(headerContent).toContain('href="#workflow"');
    expect(headerContent).toContain('href="#sources"');
    expect(headerContent).toContain('href="#features"');

    // Secure external docs and repo links
    expect(headerContent).toContain('href="https://github.com/dayashimoga/web-repo-to-ai#readme"');
    expect(headerContent).toContain('target="_blank"');
    expect(headerContent).toContain('rel="noopener noreferrer"');

    // Theme toggle accessibility
    expect(headerContent).toContain("ariaLabel");
    expect(headerContent).toContain("handleThemeToggle");

    // Mobile drawer accessibility
    expect(headerContent).toContain("aria-expanded={mobileMenuOpen}");
    expect(headerContent).toContain('aria-controls="mobile-navigation"');
  });

  it("Footer component contains required architectural references and compliance badges", () => {
    const footerPath = resolve(process.cwd(), "src/lib/components/navigation/Footer.svelte");
    const footerContent = readFileSync(footerPath, "utf-8");

    // Semantic landmark footer
    expect(footerContent).toContain("<footer");
    expect(footerContent).toContain("</footer>");

    // Architecture documentation specifications links
    expect(footerContent).toContain("docs/ARCHITECTURE.md");
    expect(footerContent).toContain("docs/SECURITY.md");
    expect(footerContent).toContain("docs/DATA_MODEL.md");
    expect(footerContent).toContain("docs/PERFORMANCE.md");

    // Open source and resource ceiling indicators
    expect(footerContent).toContain("MIT License");
    expect(footerContent).toContain("CPU-Only 16 GB");
    expect(footerContent).toContain("Podman &lt;= 30 GB");
    expect(footerContent).toContain("Zero API Costs");
    expect(footerContent).toContain("Zero Telemetry");
    expect(footerContent).toContain("Zero Data Retention");
  });
});

describe("Landing Page Sections Integrity", () => {
  it("exports all 4 core landing section components cleanly", () => {
    expect(Hero).toBeDefined();
    expect(WorkflowSteps).toBeDefined();
    expect(SupportedSources).toBeDefined();
    expect(FeatureHighlights).toBeDefined();
  });

  it("Hero component specifies all 6 export formats and value proposition", () => {
    const heroPath = resolve(process.cwd(), "src/lib/components/landing/Hero.svelte");
    const heroContent = readFileSync(heroPath, "utf-8");

    // Landmark heading
    expect(heroContent).toContain('aria-labelledby="hero-title"');
    expect(heroContent).toContain('id="hero-title"');
    expect(heroContent).toContain("Universal URL &amp; GitHub Repository");
    expect(heroContent).toContain("AI-Ready Content Converter");

    // Formats supported
    expect(heroContent).toContain(".md");
    expect(heroContent).toContain(".json");
    expect(heroContent).toContain(".xml");
    expect(heroContent).toContain(".txt");
    expect(heroContent).toContain(".jsonl");
    expect(heroContent).toContain(".zip");

    // Operational bounds
    expect(heroContent).toContain("CPU-Only 16 GB");
    expect(heroContent).toContain("&le; 30 GB Podman");
    expect(heroContent).toContain("$0 / Zero API Keys");
  });

  it("WorkflowSteps component defines the 3-step canonical pipeline", () => {
    const stepsPath = resolve(process.cwd(), "src/lib/components/landing/WorkflowSteps.svelte");
    const stepsContent = readFileSync(stepsPath, "utf-8");

    expect(stepsContent).toContain('id="workflow"');
    expect(stepsContent).toContain('aria-labelledby="workflow-heading"');
    expect(stepsContent).toContain("Acquisition & SSRF Defense");
    expect(stepsContent).toContain("CDM Canonicalization & Optimization");
    expect(stepsContent).toContain("Multi-Format Export & Token Fit");
    expect(stepsContent).toContain("01");
    expect(stepsContent).toContain("02");
    expect(stepsContent).toContain("03");
  });

  it("SupportedSources component details all 4 ingestion modalities", () => {
    const sourcesPath = resolve(
      process.cwd(),
      "src/lib/components/landing/SupportedSources.svelte"
    );
    const sourcesContent = readFileSync(sourcesPath, "utf-8");

    expect(sourcesContent).toContain('id="sources"');
    expect(sourcesContent).toContain('aria-labelledby="sources-heading"');
    expect(sourcesContent).toContain("Static Documentation & Articles");
    expect(sourcesContent).toContain("Dynamic JavaScript Single Page Apps");
    expect(sourcesContent).toContain("Public GitHub Repositories");
    expect(sourcesContent).toContain("Private Repositories via GitHub Apps");
    expect(sourcesContent).toContain("HTML5 Parser");
    expect(sourcesContent).toContain("Chromium Worker");
    expect(sourcesContent).toContain("Git Trees API");
    expect(sourcesContent).toContain("Ephemeral Auth");
  });

  it("FeatureHighlights component encapsulates 6 enterprise architectural pillars", () => {
    const featuresPath = resolve(
      process.cwd(),
      "src/lib/components/landing/FeatureHighlights.svelte"
    );
    const featuresContent = readFileSync(featuresPath, "utf-8");

    expect(featuresContent).toContain('id="features"');
    expect(featuresContent).toContain('aria-labelledby="features-heading"');
    expect(featuresContent).toContain("Zero-Trust SSRF Defense in Depth");
    expect(featuresContent).toContain("AST-Aware Code Intelligence");
    expect(featuresContent).toContain("Zero Mandatory Paid AI APIs");
    expect(featuresContent).toContain("Strict <= 30 GB Podman Ceiling");
    expect(featuresContent).toContain("Deterministic Token Budget Allocation");
    expect(featuresContent).toContain("Privacy-First & Self-Hostable");
  });
});

describe("Root Landing Page Route Assembly", () => {
  it("+page.svelte integrates navigation, hero, conversion entry, pipeline, sources, features, and footer", () => {
    const pagePath = resolve(process.cwd(), "src/routes/+page.svelte");
    const pageContent = readFileSync(pagePath, "utf-8");

    // Landmarks and core component tags
    expect(pageContent).toContain("<Header");
    expect(pageContent).toContain('<main id="main-content"');
    expect(pageContent).toContain("<Hero />");
    expect(pageContent).toContain('id="convert"');
    expect(pageContent).toContain("<WorkflowSteps />");
    expect(pageContent).toContain("<SupportedSources />");
    expect(pageContent).toContain("<FeatureHighlights />");
    expect(pageContent).toContain("<Footer />");

    // Interactive conversion entry state
    expect(pageContent).toContain("inputUrl");
    expect(pageContent).toContain("selectedFormat");
    expect(pageContent).toContain("detectedSource");
    expect(pageContent).toContain("handleStartConversion");
    expect(pageContent).toContain("conversionFeedback");
  });
});
