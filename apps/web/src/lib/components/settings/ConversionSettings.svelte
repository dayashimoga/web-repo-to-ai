<script lang="ts">
  import { Badge, Toggle } from '$lib/components/ui';
  import {
    AI_MODEL_PRESETS,
    type AIModelPresetId,
    type ConversionSettingsConfig,
    getDefaultConversionSettings,
    validateConversionSettings
  } from '$lib/types/settings';

  interface Props {
    settings?: ConversionSettingsConfig;
    isOpen?: boolean;
    class?: string;
    onchange?: (settings: ConversionSettingsConfig) => void;
  }

  let {
    settings = $bindable(getDefaultConversionSettings()),
    isOpen = $bindable(false),
    class: className = '',
    onchange
  }: Props = $props();

  const panelId = `settings-panel-${Math.random().toString(36).substring(2, 9)}`;
  const headerId = `settings-header-${Math.random().toString(36).substring(2, 9)}`;

  let activePreset = $derived(AI_MODEL_PRESETS[settings.selectedPreset] || AI_MODEL_PRESETS.custom);
  let validation = $derived(validateConversionSettings(settings));

  // Compute context ratio relative to model's total window
  let contextRatio = $derived(
    Math.min(1, Math.max(0, settings.maxTokenBudget / activePreset.contextWindow))
  );

  function toggleOpen() {
    isOpen = !isOpen;
  }

  function selectPreset(presetId: AIModelPresetId) {
    settings.selectedPreset = presetId;
    settings.maxTokenBudget = AI_MODEL_PRESETS[presetId].defaultBudget;
    onchange?.(settings);
  }

  function handleSliderChange(e: Event) {
    const val = Number((e.target as HTMLInputElement).value);
    settings.maxTokenBudget = val;

    // If custom slider value doesn't match default budget, set to custom
    if (activePreset.id !== 'custom' && val !== activePreset.defaultBudget) {
      settings.selectedPreset = 'custom';
    }
    onchange?.(settings);
  }

  function notifyChange() {
    onchange?.(settings);
  }
</script>

<div class="rounded-2xl border border-theme-subtle bg-theme-panel/70 overflow-hidden transition-colors duration-150 {className}">
  <!-- Collapsible Header / Toggle Trigger -->
  <button
    type="button"
    id={headerId}
    onclick={toggleOpen}
    aria-expanded={isOpen}
    aria-controls={panelId}
    class="w-full px-5 py-4 flex items-center justify-between text-left hover:bg-theme-subtle/50 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-brand-500 transition-colors"
  >
    <div class="flex items-center gap-3">
      <div class="p-2 rounded-lg bg-theme-subtle text-brand-500" aria-hidden="true">
        <svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M12 6V4m0 2a2 2 0 100 4m0-4a2 2 0 110 4m-6 8a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4m6 6v10m6-2a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4"
          />
        </svg>
      </div>
      <div>
        <div class="text-sm font-semibold text-theme-primary">
          Conversion Settings &amp; Token Budget
        </div>
        <div class="text-xs text-theme-secondary mt-0.5 flex items-center gap-1.5">
          <span>Target: <span class="font-medium text-theme-primary">{activePreset.name}</span></span>
          <span>&bull;</span>
          <span>Ceiling: <span class="font-mono font-semibold text-brand-500">{settings.maxTokenBudget.toLocaleString()} tokens</span></span>
        </div>
      </div>
    </div>

    <div class="flex items-center gap-2">
      <Badge variant="brand" size="sm">
        {activePreset.badge}
      </Badge>
      <span
        class="text-theme-muted transition-transform duration-200 {isOpen ? 'rotate-180' : ''}"
        aria-hidden="true"
      >
        <svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
        </svg>
      </span>
    </div>
  </button>

  <!-- Expandable Settings Region -->
  {#if isOpen}
    <div
      id={panelId}
      role="region"
      aria-labelledby={headerId}
      class="px-5 pt-2 pb-6 border-t border-theme-subtle space-y-6 animate-fadeIn"
    >
      <!-- 1. Target AI Model Context Presets -->
      <div class="space-y-3">
        <div class="text-xs font-semibold uppercase tracking-wider text-theme-muted">
          Target AI Model Presets
        </div>
        <div class="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-6 gap-2.5">
          {#each Object.values(AI_MODEL_PRESETS) as preset}
            <button
              type="button"
              onclick={() => selectPreset(preset.id)}
              class="p-2.5 rounded-xl border text-left flex flex-col justify-between transition-all duration-150 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-brand-500 {settings.selectedPreset ===
              preset.id
                ? 'bg-brand-500/10 border-brand-500 text-theme-primary shadow-elevation-1'
                : 'bg-theme-subtle/50 border-theme-subtle text-theme-secondary hover:text-theme-primary hover:border-theme-strong'}"
            >
              <div class="text-xs font-bold leading-tight truncate">{preset.name}</div>
              <div class="text-[10px] text-theme-muted mt-1">{preset.provider}</div>
              <div class="text-xs font-mono font-semibold text-brand-500 mt-2">
                {preset.badge}
              </div>
            </button>
          {/each}
        </div>
      </div>

      <!-- 2. Token Budget Slider -->
      <div class="p-4 rounded-xl bg-theme-subtle/40 border border-theme-subtle space-y-3">
        <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-1 text-sm">
          <label for="token-budget-slider" class="font-semibold text-theme-primary">
            Context Window Token Ceiling
          </label>
          <div class="font-mono text-xs text-theme-secondary">
            <span>Allocated: </span>
            <span class="font-bold text-brand-500 text-sm">
              {settings.maxTokenBudget.toLocaleString()}
            </span>
            <span> / {activePreset.contextWindow.toLocaleString()} tokens ({(contextRatio * 100).toFixed(0)}%)</span>
          </div>
        </div>

        <input
          id="token-budget-slider"
          type="range"
          min="10000"
          max="2000000"
          step="5000"
          value={settings.maxTokenBudget}
          oninput={handleSliderChange}
          class="w-full accent-brand-500 cursor-pointer h-2 bg-theme-subtle rounded-lg"
          aria-label="Target token budget ceiling slider"
        />

        <div class="w-full bg-theme-subtle rounded-full h-2 overflow-hidden border border-theme-subtle">
          <div
            class="h-full bg-brand-500 transition-all duration-150 rounded-full"
            style="width: {contextRatio * 100}%"
          ></div>
        </div>
      </div>

      <!-- 3. CDM Content Normalization & Inclusion Filters -->
      <div class="space-y-3">
        <div class="text-xs font-semibold uppercase tracking-wider text-theme-muted">
          Content Normalization &amp; Deduplication
        </div>
        <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
          <div class="p-3.5 rounded-xl bg-theme-subtle/40 border border-theme-subtle">
            <Toggle
              label="Strip Boilerplate &amp; Navigation"
              description="Eliminates headers, footers, sidebars, and tracking script elements from web pages."
              bind:checked={settings.stripBoilerplate}
              onchange={notifyChange}
            />
          </div>

          <div class="p-3.5 rounded-xl bg-theme-subtle/40 border border-theme-subtle">
            <Toggle
              label="Strip Non-Doc Code Comments"
              description="Removes inline and implementation comments while preserving API docstrings."
              bind:checked={settings.stripCodeComments}
              onchange={notifyChange}
            />
          </div>

          <div class="p-3.5 rounded-xl bg-theme-subtle/40 border border-theme-subtle">
            <Toggle
              label="Exclude Test Files &amp; Suites"
              description="Excludes tests/, spec/, __tests__/, and test fixtures from repository extraction."
              bind:checked={settings.excludeTests}
              onchange={notifyChange}
            />
          </div>

          <div class="p-3.5 rounded-xl bg-theme-subtle/40 border border-theme-subtle">
            <Toggle
              label="MinHash Semantic Deduplication"
              description="Detects and eliminates duplicate documentation sections and code blocks."
              bind:checked={settings.semanticDeduplication}
              onchange={notifyChange}
            />
          </div>
        </div>
      </div>

      <!-- 4. Dynamic Ingestion & Sandbox Limits -->
      <div class="space-y-3">
        <div class="text-xs font-semibold uppercase tracking-wider text-theme-muted">
          Dynamic Ingestion &amp; Boundaries
        </div>
        <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
          <div class="p-3.5 rounded-xl bg-theme-subtle/40 border border-theme-subtle flex flex-col justify-between">
            <Toggle
              label="Chromium Dynamic Rendering Fallback"
              description="Fall back to headless Chromium worker when static HTML parser detects dynamic hydration."
              bind:checked={settings.enableDynamicRendering}
              onchange={notifyChange}
            />
          </div>

          <div class="p-3.5 rounded-xl bg-theme-subtle/40 border border-theme-subtle space-y-2">
            <label for="max-file-size-input" class="text-xs font-semibold text-theme-primary block">
              Max Individual File Size (KB)
            </label>
            <input
              id="max-file-size-input"
              type="number"
              min="64"
              max="10240"
              step="64"
              bind:value={settings.maxFileSizeKb}
              oninput={notifyChange}
              class="w-full px-3 py-1.5 text-sm rounded-lg border border-theme-subtle bg-theme-panel text-theme-primary focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-brand-500"
              aria-label="Max individual file size in kilobytes"
            />
            <p class="text-[11px] text-theme-muted">
              Files exceeding {settings.maxFileSizeKb} KB will be indexed by symbol metadata rather than raw content.
            </p>
          </div>
        </div>
      </div>

      <!-- Validation Error Feedback -->
      {#if !validation.isValid}
        <div
          role="alert"
          class="p-3 rounded-xl border border-status-error/30 bg-status-error/10 text-xs text-status-error space-y-1"
        >
          <div class="font-bold">Settings Configuration Error:</div>
          <ul class="list-disc list-inside">
            {#each validation.errors as err}
              <li>{err}</li>
            {/each}
          </ul>
        </div>
      {/if}
    </div>
  {/if}
</div>
