<script lang="ts">
  import { colors, getTokenBudgetStatus } from '$lib/tokens';
  import { getTokenBudgetClasses } from '$lib/tokens/color-utils';
  import { Heading, Text, Code, Mono } from '$lib/components/typography';
  import { Container, Stack, Grid } from '$lib/components/layout';
  import { Button, Input, Toggle, Badge, Modal, Tooltip } from '$lib/components/ui';

  let isDark = $state(true);
  let sampleTokens = $state(75000);
  const maxTokens = 100000;

  let testInput = $state('https://github.com/dayashimoga/web-repo-to-ai');
  let testToggle = $state(true);
  let isModalOpen = $state(false);
  let isButtonLoading = $state(false);

  let budgetStatus = $derived(getTokenBudgetStatus(sampleTokens, maxTokens));
  let budgetClasses = $derived(getTokenBudgetClasses(budgetStatus.ratio));

  function toggleTheme() {
    isDark = !isDark;
    if (typeof document !== 'undefined') {
      document.documentElement.classList.toggle('dark', isDark);
    }
  }

  function simulateLoad() {
    isButtonLoading = true;
    setTimeout(() => {
      isButtonLoading = false;
    }, 1500);
  }
</script>

<header class="border-b border-theme-subtle bg-theme-panel px-6 py-4 flex items-center justify-between shadow-elevation-1">
  <div class="flex items-center space-x-3">
    <div class="h-8 w-8 rounded-lg bg-brand-500 flex items-center justify-center text-white font-bold text-sm shadow-elevation-1">
      AI
    </div>
    <span class="font-bold text-lg tracking-tight">web-repo-to-ai</span>
    <Badge variant="brand" size="sm" dot>Stage 1 Design System</Badge>
  </div>
  <Button variant="secondary" size="sm" onclick={toggleTheme} ariaLabel="Toggle color theme">
    {isDark ? '☀️ Light Mode' : '🌙 Dark Mode'}
  </Button>
</header>

<main class="flex-1 py-8">
  <Container maxWidth="xl">
    <Stack gap={8}>
      <!-- Page Header -->
      <Stack gap={2}>
        <Heading level={1}>Design System & UI Component Primitives</Heading>
        <Text size="lg" tone="secondary">
          Accessible, keyboard-navigable components built with Svelte 5 and Tailwind CSS.
        </Text>
      </Stack>

      <!-- UI Component Primitives Showcase -->
      <section class="bg-theme-panel p-6 rounded-xl border border-theme-subtle shadow-elevation-1 space-y-6">
        <Heading level={2} size="2xl">Interactive UI Primitives</Heading>

        <!-- Buttons -->
        <Stack gap={3}>
          <Heading level={4} size="lg">Buttons & States</Heading>
          <div class="flex flex-wrap items-center gap-3">
            <Button variant="primary">Primary Button</Button>
            <Button variant="secondary">Secondary</Button>
            <Button variant="outline">Outline</Button>
            <Button variant="ghost">Ghost</Button>
            <Button variant="danger">Danger</Button>
            <Button variant="primary" loading={isButtonLoading} onclick={simulateLoad}>
              {isButtonLoading ? 'Processing...' : 'Click to Load'}
            </Button>
            <Button variant="secondary" disabled>Disabled</Button>
          </div>
        </Stack>

        <!-- Badges -->
        <Stack gap={3}>
          <Heading level={4} size="lg">Status Badges</Heading>
          <div class="flex flex-wrap items-center gap-2">
            <Badge variant="default">Default</Badge>
            <Badge variant="brand" dot>Brand AI</Badge>
            <Badge variant="success" dot>Extracted</Badge>
            <Badge variant="warning" dot>Budget &gt;80%</Badge>
            <Badge variant="error" dot>SSRF Blocked</Badge>
            <Badge variant="info" dot>Playwright</Badge>
          </div>
        </Stack>

        <!-- Form Inputs & Toggles -->
        <Grid cols={2} gap={6}>
          <Stack gap={3}>
            <Heading level={4} size="lg">Accessible Inputs</Heading>
            <Input
              label="Source URL / GitHub Repository"
              placeholder="https://github.com/org/repo"
              bind:value={testInput}
              helperText="Valid URLs are checked against the SSRF security policy."
              clearable
              required
            />
          </Stack>

          <Stack gap={3}>
            <Heading level={4} size="lg">Switch Toggles & Tooltips</Heading>
            <div class="p-4 rounded-lg bg-theme-subtle border border-theme-subtle space-y-4">
              <Toggle
                label="Dynamic JavaScript Rendering"
                description="Fall back to isolated Chromium browser worker when static HTML is incomplete."
                bind:checked={testToggle}
              />
              <div class="flex items-center gap-2 text-sm text-theme-secondary">
                <span>Hover or tab for details:</span>
                <Tooltip content="SSRF protection revalidates DNS on every redirect hop.">
                  <Badge variant="brand" size="sm">Hover Info</Badge>
                </Tooltip>
              </div>
            </div>
          </Stack>
        </Grid>

        <!-- Modal Trigger -->
        <Stack gap={3}>
          <Heading level={4} size="lg">Accessible Dialog Modal</Heading>
          <div>
            <Button variant="primary" onclick={() => (isModalOpen = true)}>
              Open Sample Modal Dialog
            </Button>
          </div>
        </Stack>
      </section>

      <!-- Token Budget Indicator Showcase -->
      <section class="bg-theme-panel p-6 rounded-xl border border-theme-subtle shadow-elevation-1 space-y-4">
        <div class="flex justify-between items-center">
          <Heading level={2} size="2xl">AI Context Token Budget Indicator</Heading>
          <span class="px-2.5 py-1 rounded text-xs font-semibold uppercase border {budgetClasses.badgeClass}">
            {budgetStatus.status} ({(budgetStatus.ratio * 100).toFixed(1)}%)
          </span>
        </div>

        <Text tone="secondary" size="sm">
          Dynamic context window tracker adhering to WCAG 2.1 contrast guidelines.
        </Text>

        <div class="space-y-3 pt-2">
          <div class="flex justify-between items-center text-sm font-mono">
            <span>Tokens: <Mono class="font-bold">{sampleTokens.toLocaleString()}</Mono></span>
            <span>Ceiling: <Mono class="font-bold">{maxTokens.toLocaleString()}</Mono></span>
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

<!-- Sample Dialog Modal -->
<Modal
  bind:open={isModalOpen}
  title="Conversion Parameters"
  description="Configure CDM Canonical Context Model settings for extraction."
>
  <Stack gap={4}>
    <Text size="sm">
      Adjust target model context parameters. The extraction worker will automatically normalize whitespace, strip tracking tags, and generate labeled token maps.
    </Text>
    <Input
      label="Max Token Budget"
      type="number"
      value="128000"
      helperText="Context window ceiling for selected AI model."
    />
  </Stack>

  {#snippet footer()}
    <Button variant="outline" size="sm" onclick={() => (isModalOpen = false)}>Cancel</Button>
    <Button variant="primary" size="sm" onclick={() => (isModalOpen = false)}>Save Settings</Button>
  {/snippet}
</Modal>
