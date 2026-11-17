<script lang="ts">
  import { Header, Footer } from '$lib/components/navigation';
  import { Hero, WorkflowSteps, SupportedSources, FeatureHighlights } from '$lib/components/landing';
  import { UrlInput } from '$lib/components/input';
  import { ConversionSettings } from '$lib/components/settings';
  import { Badge } from '$lib/components/ui';
  import type { UrlDetectionResult } from '$lib/utils/url-detector';
  import {
    getDefaultConversionSettings,
    type ConversionSettingsConfig
  } from '$lib/types/settings';

  let isDark = $state(true);

  // Conversion entry state
  let inputUrl = $state('https://github.com/dayashimoga/web-repo-to-ai');
  let selectedFormat = $state<'markdown' | 'json' | 'xml' | 'txt' | 'jsonl' | 'zip'>('markdown');
  let isConverting = $state(false);
  let conversionFeedback = $state<string | null>(null);
  let feedbackType = $state<'success' | 'error' | 'info'>('info');
  let currentDetection = $state<UrlDetectionResult | null>(null);

  // Conversion settings state
  let settings = $state<ConversionSettingsConfig>(getDefaultConversionSettings());
  let isSettingsOpen = $state(false);

  let detectedSource = $derived(currentDetection?.label ?? 'Awaiting URL...');

  function handleUrlChange(detection: UrlDetectionResult) {
    currentDetection = detection;
    if (detection.suggestedFormat && !conversionFeedback) {
      selectedFormat = detection.suggestedFormat;
    }
  }

  function handleStartConversion(detection: UrlDetectionResult) {
    if (!detection.isValid || !detection.isSafe) {
      feedbackType = 'error';
      conversionFeedback = detection.errorMessage || 'Please provide a valid, safe URL to proceed.';
      return;
    }

    isConverting = true;
    conversionFeedback = null;

    setTimeout(() => {
      isConverting = false;
      feedbackType = 'success';
      const targetDesc = detection.githubMetadata
        ? `${detection.githubMetadata.owner}/${detection.githubMetadata.repo}`
        : detection.domain || detection.normalizedUrl;
      conversionFeedback = `Verified ${detection.label}: '${targetDesc}'. Destination validated against zero-trust SSRF policies. Configured for ${settings.selectedPreset} (${settings.maxTokenBudget.toLocaleString()} tokens ceiling) in ${selectedFormat.toUpperCase()} format. Ready for extraction pipeline.`;
    }, 500);
  }
</script>

<!-- Global Site Header -->
<Header bind:isDark />

<!-- Main Content Landmark -->
<main id="main-content" class="flex-1">
  <!-- Hero Section -->
  <Hero />

  <!-- URL Input & Conversion Entry Section -->
  <section
    id="convert"
    aria-labelledby="convert-heading"
    class="py-16 md:py-20 border-b border-theme-subtle bg-theme-panel/40 transition-colors duration-150"
  >
    <div class="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8">
      <div class="text-center mb-8">
        <Badge variant="brand" size="sm" class="mb-3">Content Ingestion</Badge>
        <h2
          id="convert-heading"
          class="text-3xl sm:text-4xl font-extrabold tracking-tight text-theme-primary"
        >
          Start Content Ingestion
        </h2>
        <p class="mt-3 text-base text-theme-secondary">
          Enter any public documentation URL or GitHub repository to extract and optimize AI context.
        </p>
      </div>

      <div class="p-6 sm:p-8 rounded-2xl bg-theme-panel border border-theme-subtle shadow-elevation-2 space-y-6">
        <!-- Dedicated UrlInput Component with Instant Source & SSRF Detection -->
        <UrlInput
          bind:value={inputUrl}
          loading={isConverting}
          onchange={handleUrlChange}
          onsubmit={handleStartConversion}
        />

        <!-- Advanced Conversion Settings & Token Budget Drawer -->
        <ConversionSettings bind:settings bind:isOpen={isSettingsOpen} />

        <!-- Target Output Format Selector -->
        <div class="space-y-2 pt-2 border-t border-theme-subtle">
          <div class="text-xs font-semibold uppercase tracking-wider text-theme-muted">
            Target Output Format
          </div>
          <div class="flex flex-wrap gap-2">
            {#each [
              { id: 'markdown', label: 'Markdown (.md)' },
              { id: 'json', label: 'JSON CDM (.json)' },
              { id: 'xml', label: 'Claude XML (.xml)' },
              { id: 'txt', label: 'Plaintext (.txt)' },
              { id: 'jsonl', label: 'JSONL (.jsonl)' },
              { id: 'zip', label: 'Archive (.zip)' }
            ] as const as fmt}
              <button
                type="button"
                class="text-xs font-medium px-3 py-1.5 rounded-lg border transition-all duration-150 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-brand-500 {selectedFormat ===
                fmt.id
                  ? 'bg-brand-600 text-white border-brand-600 shadow-elevation-1'
                  : 'bg-theme-subtle text-theme-secondary border-theme-subtle hover:text-theme-primary'}"
                onclick={() => (selectedFormat = fmt.id)}
              >
                {fmt.label}
              </button>
            {/each}
          </div>
        </div>

        <!-- Live Conversion / Verification Feedback Banner -->
        {#if conversionFeedback}
          <div
            role="alert"
            class="p-4 rounded-xl text-sm border flex flex-col sm:flex-row items-start sm:items-center justify-between gap-3 {feedbackType === 'success'
              ? 'bg-status-success/10 border-status-success/30 text-green-700 dark:text-green-300'
              : feedbackType === 'error'
                ? 'bg-status-error/10 border-status-error/30 text-red-700 dark:text-red-300'
                : 'bg-status-info/10 border-status-info/30 text-blue-700 dark:text-blue-300'}"
          >
            <div class="flex items-start gap-3">
              <span class="font-bold text-base" aria-hidden="true">
                {feedbackType === 'success' ? '✓' : feedbackType === 'error' ? '⚠' : 'ℹ'}
              </span>
              <span>{conversionFeedback}</span>
            </div>
            {#if feedbackType === 'success'}
              <a
                href={`/workspace?url=${encodeURIComponent(inputUrl)}&format=${selectedFormat}&preset=${settings.selectedPreset}&budget=${settings.maxTokenBudget}`}
                class="shrink-0 px-3.5 py-1.5 rounded-lg bg-brand-600 hover:bg-brand-500 text-white font-medium text-xs transition-colors shadow-elevation-1 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-brand-500"
              >
                Open Workspace →
              </a>
            {/if}
          </div>
        {/if}
      </div>
    </div>
  </section>

  <!-- How It Works: 3-Step Pipeline -->
  <WorkflowSteps />

  <!-- Supported Input Sources Matrix -->
  <SupportedSources />

  <!-- Architectural & Security Highlights -->
  <FeatureHighlights />
</main>

<!-- Global Site Footer -->
<Footer />
