<script lang="ts">
  import { Header, Footer } from '$lib/components/navigation';
  import { Hero, WorkflowSteps, SupportedSources, FeatureHighlights } from '$lib/components/landing';
  import { Button, Input, Badge } from '$lib/components/ui';

  let isDark = $state(true);

  // Conversion quick entry state (expanded in Subphase 1.2.2)
  let inputUrl = $state('https://github.com/dayashimoga/web-repo-to-ai');
  let selectedFormat = $state<'markdown' | 'json' | 'xml' | 'txt' | 'jsonl' | 'zip'>('markdown');
  let isConverting = $state(false);
  let conversionFeedback = $state<string | null>(null);
  let feedbackType = $state<'success' | 'error' | 'info'>('info');

  let detectedSource = $derived.by(() => {
    const trimmed = inputUrl.trim();
    if (!trimmed) return 'Awaiting URL...';
    if (trimmed.includes('github.com')) return 'GitHub Repository';
    if (trimmed.startsWith('http://') || trimmed.startsWith('https://')) return 'Web Documentation / Article';
    return 'Invalid Protocol';
  });

  function handleStartConversion(e?: Event) {
    if (e) e.preventDefault();
    const trimmed = inputUrl.trim();
    if (!trimmed) {
      feedbackType = 'error';
      conversionFeedback = 'Please enter a valid HTTP(S) documentation URL or GitHub repository.';
      return;
    }
    if (!trimmed.startsWith('https://') && !trimmed.startsWith('http://')) {
      feedbackType = 'error';
      conversionFeedback = 'URL must begin with http:// or https:// for zero-trust security validation.';
      return;
    }

    isConverting = true;
    conversionFeedback = null;

    setTimeout(() => {
      isConverting = false;
      feedbackType = 'success';
      conversionFeedback = `URL verified (${detectedSource}). Target format set to ${selectedFormat.toUpperCase()}. Prepared for canonical extraction pipeline.`;
    }, 500);
  }
</script>

<!-- Global Site Header -->
<Header bind:isDark />

<!-- Main Content Landmark -->
<main id="main-content" class="flex-1">
  <!-- Hero Section -->
  <Hero />

  <!-- Quick Conversion Entry Section -->
  <section
    id="convert"
    aria-labelledby="convert-heading"
    class="py-16 md:py-20 border-b border-theme-subtle bg-theme-panel/40 transition-colors duration-150"
  >
    <div class="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8">
      <div class="text-center mb-8">
        <Badge variant="brand" size="sm" class="mb-3">Quick Conversion Entry</Badge>
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

      <form
        onsubmit={handleStartConversion}
        class="p-6 sm:p-8 rounded-2xl bg-theme-panel border border-theme-subtle shadow-elevation-2 space-y-6"
      >
        <div class="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-2">
          <label for="url-input" class="text-sm font-semibold text-theme-primary">
            Target URL or Repository
          </label>
          <span class="text-xs font-mono text-theme-secondary flex items-center gap-1.5">
            <span>Detected:</span>
            <Badge
              variant={detectedSource === 'Invalid Protocol'
                ? 'error'
                : detectedSource === 'Awaiting URL...'
                  ? 'default'
                  : 'success'}
              size="sm"
            >
              {detectedSource}
            </Badge>
          </span>
        </div>

        <div class="flex flex-col sm:flex-row gap-3">
          <div class="flex-1">
            <Input
              id="url-input"
              placeholder="https://github.com/owner/repo or https://docs.example.com"
              bind:value={inputUrl}
              clearable
              required
              ariaLabel="Target URL or GitHub Repository"
            />
          </div>
          <Button
            type="submit"
            variant="primary"
            size="md"
            loading={isConverting}
            ariaLabel="Start Conversion Job"
            class="sm:self-start whitespace-nowrap min-w-[140px]"
          >
            {isConverting ? 'Verifying...' : 'Convert URL'}
          </Button>
        </div>

        <!-- Format Selector Options -->
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

        <!-- Interactive Feedback Banner -->
        {#if conversionFeedback}
          <div
            role="alert"
            class="p-4 rounded-xl text-sm border flex items-start gap-3 {feedbackType === 'success'
              ? 'bg-status-success/10 border-status-success/30 text-green-700 dark:text-green-300'
              : feedbackType === 'error'
                ? 'bg-status-error/10 border-status-error/30 text-red-700 dark:text-red-300'
                : 'bg-status-info/10 border-status-info/30 text-blue-700 dark:text-blue-300'}"
          >
            <span class="font-bold text-base" aria-hidden="true">
              {feedbackType === 'success' ? '✓' : feedbackType === 'error' ? '⚠' : 'ℹ'}
            </span>
            <span class="flex-1">{conversionFeedback}</span>
          </div>
        {/if}
      </form>
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
