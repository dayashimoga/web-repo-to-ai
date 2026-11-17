<script lang="ts">
  import { colors, getTokenBudgetStatus } from '$lib/tokens';
  import { getTokenBudgetClasses } from '$lib/tokens/color-utils';
  import { Heading, Text, Code, Mono } from '$lib/components/typography';
  import { Container, Stack, Grid } from '$lib/components/layout';

  let isDark = $state(true);
  let sampleTokens = $state(75000);
  const maxTokens = 100000;

  let budgetStatus = $derived(getTokenBudgetStatus(sampleTokens, maxTokens));
  let budgetClasses = $derived(getTokenBudgetClasses(budgetStatus.ratio));

  function toggleTheme() {
    isDark = !isDark;
    if (typeof document !== 'undefined') {
      document.documentElement.classList.toggle('dark', isDark);
    }
  }
</script>

<header class="border-b border-theme-subtle bg-theme-panel px-6 py-4 flex items-center justify-between shadow-elevation-1">
  <div class="flex items-center space-x-3">
    <div class="h-8 w-8 rounded-lg bg-brand-500 flex items-center justify-center text-white font-bold text-sm shadow-elevation-1">
      AI
    </div>
    <span class="font-bold text-lg tracking-tight">web-repo-to-ai</span>
    <span class="text-xs px-2 py-0.5 rounded-full bg-brand-100 dark:bg-brand-950 text-brand-700 dark:text-brand-300 font-mono">
      v0.1.0-system
    </span>
  </div>
  <button
    type="button"
    onclick={toggleTheme}
    class="px-3 py-1.5 rounded-md text-sm border border-theme-subtle hover:bg-theme-subtle transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-brand-500"
    aria-label="Toggle color theme"
  >
    {isDark ? '☀️ Light' : '🌙 Dark'}
  </button>
</header>

<main class="flex-1 py-8">
  <Container maxWidth="xl">
    <Stack gap={8}>
      <!-- Page Header -->
      <Stack gap={2}>
        <Heading level={1}>Typography, Spacing & Color Systems</Heading>
        <Text size="lg" tone="secondary">
          Design system foundation for high-density, accessible AI context conversion interfaces.
        </Text>
      </Stack>

      <!-- Typography Scale Showcase -->
      <section class="bg-theme-panel p-6 rounded-xl border border-theme-subtle shadow-elevation-1 space-y-4">
        <Heading level={2} size="2xl">Typography Scale</Heading>
        <Stack gap={4}>
          <Heading level={3} size="xl">Heading 3 (20px / 1.25rem)</Heading>
          <Heading level={4} size="lg">Heading 4 (18px / 1.125rem)</Heading>
          <Text size="base">
            Body base text (<Code>text-base leading-6</Code>) with standard 16px size for dense repository manifests and article excerpts.
          </Text>
          <Text size="sm" tone="secondary">
            Body small text (<Code>text-sm leading-5</Code>) for file metadata, timestamps, and AST node descriptions.
          </Text>
          <div class="flex items-center space-x-3 text-sm">
            <span>Inline Monospace:</span>
            <Code>crates/tokenizer/src/bpe.rs:142</Code>
            <span>Token Counter:</span>
            <Mono size="sm" class="font-bold text-brand-600 dark:text-brand-400">128,450 tokens</Mono>
          </div>
        </Stack>
      </section>

      <!-- Layout Primitives Showcase -->
      <section class="bg-theme-panel p-6 rounded-xl border border-theme-subtle shadow-elevation-1 space-y-4">
        <Heading level={2} size="2xl">4px Grid Layout Primitives</Heading>
        <Text tone="secondary">
          Responsive CSS Grid and Flexbox Stacks adhering to the 4px baseline rhythm (<Code>gap-1</Code> to <Code>gap-12</Code>).
        </Text>

        <Grid cols={3} gap={4}>
          <div class="p-4 rounded-lg bg-theme-subtle border border-theme-subtle space-y-1">
            <Heading level={5} size="base">Grid Col 1</Heading>
            <Text size="sm" tone="secondary">SSRF URL Validation Gateway</Text>
          </div>
          <div class="p-4 rounded-lg bg-theme-subtle border border-theme-subtle space-y-1">
            <Heading level={5} size="base">Grid Col 2</Heading>
            <Text size="sm" tone="secondary">Playwright Browser Worker</Text>
          </div>
          <div class="p-4 rounded-lg bg-theme-subtle border border-theme-subtle space-y-1">
            <Heading level={5} size="base">Grid Col 3</Heading>
            <Text size="sm" tone="secondary">Canonical Model & Tokenizer</Text>
          </div>
        </Grid>
      </section>

      <!-- Interactive Token Budget Indicator -->
      <section class="bg-theme-panel p-6 rounded-xl border border-theme-subtle shadow-elevation-1 space-y-4">
        <div class="flex justify-between items-center">
          <Heading level={2} size="2xl">Interactive Context Budget Indicator</Heading>
          <span class="px-2.5 py-1 rounded text-xs font-semibold uppercase border {budgetClasses.badgeClass}">
            {budgetStatus.status} ({(budgetStatus.ratio * 100).toFixed(1)}%)
          </span>
        </div>

        <Text tone="secondary" size="sm">
          Dynamic context usage slider demonstrating automatic WCAG status transitions:
          <span class="text-status-success font-medium"> Safe (&le; 80%)</span>,
          <span class="text-status-warning font-medium"> Warning (80%–100%)</span>,
          <span class="text-status-error font-medium"> Critical (&gt; 100%)</span>.
        </Text>

        <div class="space-y-3 pt-2">
          <div class="flex justify-between items-center text-sm font-mono">
            <span>Consumed Tokens: <Mono class="font-bold">{sampleTokens.toLocaleString()}</Mono></span>
            <span>Target Budget: <Mono class="font-bold">{maxTokens.toLocaleString()}</Mono></span>
          </div>

          <input
            type="range"
            min="10000"
            max="150000"
            step="2500"
            bind:value={sampleTokens}
            class="w-full accent-brand-500 cursor-pointer h-2 bg-theme-subtle rounded-lg"
            aria-label="Token budget slider"
          />

          <div class="w-full bg-theme-subtle rounded-full h-3 overflow-hidden border border-theme-subtle">
            <div
              class="h-full transition-all duration-150 rounded-full"
              style="width: {Math.min(100, budgetStatus.ratio * 100)}%; background-color: {budgetStatus.color}"
            ></div>
          </div>
        </div>
      </section>
    </Stack>
  </Container>
</main>
