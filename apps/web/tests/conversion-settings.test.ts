import { describe, it, expect } from "vitest";
import { ConversionSettings } from "../src/lib/components/settings/index";
import {
  AI_MODEL_PRESETS,
  getDefaultConversionSettings,
  validateConversionSettings,
  type ConversionSettingsConfig,
} from "../src/lib/types/settings";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

describe("Conversion Settings Contracts & Model Presets", () => {
  it("defines calibrated presets for all major AI models", () => {
    expect(AI_MODEL_PRESETS["gemini-1.5-pro"]).toBeDefined();
    expect(AI_MODEL_PRESETS["gemini-1.5-pro"].contextWindow).toBe(2000000);

    expect(AI_MODEL_PRESETS["gemini-1.5-flash"]).toBeDefined();
    expect(AI_MODEL_PRESETS["gemini-1.5-flash"].contextWindow).toBe(1000000);

    expect(AI_MODEL_PRESETS["claude-3.5-sonnet"]).toBeDefined();
    expect(AI_MODEL_PRESETS["claude-3.5-sonnet"].contextWindow).toBe(200000);

    expect(AI_MODEL_PRESETS["gpt-4o"]).toBeDefined();
    expect(AI_MODEL_PRESETS["gpt-4o"].contextWindow).toBe(128000);

    expect(AI_MODEL_PRESETS["local-128k"]).toBeDefined();
    expect(AI_MODEL_PRESETS["local-128k"].contextWindow).toBe(128000);

    expect(AI_MODEL_PRESETS["local-32k"]).toBeDefined();
    expect(AI_MODEL_PRESETS["local-32k"].contextWindow).toBe(32768);

    expect(AI_MODEL_PRESETS["custom"]).toBeDefined();
  });

  it("generates valid default configuration", () => {
    const defaults = getDefaultConversionSettings();
    expect(defaults.selectedPreset).toBe("claude-3.5-sonnet");
    expect(defaults.maxTokenBudget).toBe(160000);
    expect(defaults.stripBoilerplate).toBe(true);
    expect(defaults.stripCodeComments).toBe(false);
    expect(defaults.excludeTests).toBe(true);
    expect(defaults.enableDynamicRendering).toBe(false);
    expect(defaults.maxFileSizeKb).toBe(1024);
    expect(defaults.exactDeduplication).toBe(true);
    expect(defaults.semanticDeduplication).toBe(true);

    const validation = validateConversionSettings(defaults);
    expect(validation.isValid).toBe(true);
    expect(validation.errors).toHaveLength(0);
  });

  it("validates token budget boundaries", () => {
    const lowBudget: ConversionSettingsConfig = {
      ...getDefaultConversionSettings(),
      maxTokenBudget: 500,
    };
    const lowRes = validateConversionSettings(lowBudget);
    expect(lowRes.isValid).toBe(false);
    expect(lowRes.errors[0]).toContain("at least 1,000 tokens");

    const highBudget: ConversionSettingsConfig = {
      ...getDefaultConversionSettings(),
      maxTokenBudget: 5000000,
    };
    const highRes = validateConversionSettings(highBudget);
    expect(highRes.isValid).toBe(false);
    expect(highRes.errors[0]).toContain("cannot exceed 2,000,000 tokens");
  });

  it("validates maximum file size boundaries", () => {
    const lowFileSize: ConversionSettingsConfig = {
      ...getDefaultConversionSettings(),
      maxFileSizeKb: 32,
    };
    const lowRes = validateConversionSettings(lowFileSize);
    expect(lowRes.isValid).toBe(false);
    expect(lowRes.errors[0]).toContain("at least 64 KB");

    const highFileSize: ConversionSettingsConfig = {
      ...getDefaultConversionSettings(),
      maxFileSizeKb: 20000,
    };
    const highRes = validateConversionSettings(highFileSize);
    expect(highRes.isValid).toBe(false);
    expect(highRes.errors[0]).toContain("cannot exceed 10,240 KB");
  });
});

describe("ConversionSettings Component Integrity & Accessibility", () => {
  it("exports ConversionSettings component cleanly from settings module", () => {
    expect(ConversionSettings).toBeDefined();
  });

  it("defines accessible disclosure trigger and collapsible region landmarks", () => {
    const componentPath = resolve(
      process.cwd(),
      "src/lib/components/settings/ConversionSettings.svelte"
    );
    const content = readFileSync(componentPath, "utf-8");

    // Disclosure trigger
    expect(content).toContain("aria-expanded={isOpen}");
    expect(content).toContain("aria-controls={panelId}");

    // Region landmark
    expect(content).toContain('role="region"');
    expect(content).toContain("aria-labelledby={headerId}");

    // Token range slider
    expect(content).toContain('type="range"');
    expect(content).toContain('aria-label="Target token budget ceiling slider"');

    // Normalization and filter switches
    expect(content).toContain("settings.stripBoilerplate");
    expect(content).toContain("settings.stripCodeComments");
    expect(content).toContain("settings.excludeTests");
    expect(content).toContain("settings.semanticDeduplication");
    expect(content).toContain("settings.enableDynamicRendering");

    // Validation alert
    expect(content).toContain('role="alert"');

    // Svelte 5 runes
    expect(content).toContain("$bindable");
    expect(content).toContain("$derived");
    expect(content).toContain("$props");
  });
});
