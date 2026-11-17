<script lang="ts">
  import { colors, getTokenBudgetStatus } from '$lib/tokens';

  let isDark = $state(true);
  let sampleTokens = $state(75000);
  const maxTokens = 100000;

  let budgetStatus = $derived(getTokenBudgetStatus(sampleTokens, maxTokens));

  function toggleTheme() {
    isDark = !isDark;
    if (typeof document !== 'undefined') {
      document.documentElement.classList.toggle('dark', isDark);
    }
  }
</script>

<header class="border-b border-theme-subtle bg-theme-panel px-6 py-4 flex items-center justify-between">
  <div class="flex items-center space-x-3">
    <div class="h-8 w-8 rounded-lg bg-brand-500 flex items-center justify-center text-white font-bold text-sm shadow-elevation-1">
      AI
    </div>
    <span class="font-bold text-lg tracking-tight">web-repo-to-ai</span>
    <span class="text-xs px-2 py-0.5 rounded-full bg-brand-100 dark:bg-brand-950 text-brand-700 dark:text-brand-300 font-mono">
      v0.1.0-tokens
    </span>
  </div>
  <button
    type="button"
    onclick={toggleTheme}
    class="px-3 py-1.5 rounded-md text-sm border border-theme-subtle hover:bg-theme-subtle transition-colors focus-visible:outline-none"
    aria-label="Toggle dark mode"
  >
    {isDark ? '☀️ Light' : '🌙 Dark'}
  </button>
</header>

<main class="flex-1 max-w-5xl mx-auto w-full p-6 space-y-8">
  <section class="space-y-2">
    <h1 class="text-3xl font-bold tracking-tight">Visual Design Token System</h1>
    <p class="text-theme-secondary text-base">
      Foundational design tokens for the universal URL and repository to AI converter.
    </p>
  </section>

  <!-- Color Palette Overview -->
  <section class="bg-theme-panel p-6 rounded-xl border border-theme-subtle shadow-elevation-1 space-y-4">
    <h2 class="text-xl font-semibold">Color Palettes</h2>
    <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-6 gap-3">
      <div class="p-3 rounded-lg bg-brand-500 text-white text-xs font-mono text-center">
        brand-500<br/>#6366f1
      </div>
      <div class="p-3 rounded-lg bg-accent-500 text-white text-xs font-mono text-center">
        accent-500<br/>#06b6d4
      </div>
      <div class="p-3 rounded-lg bg-status-success text-white text-xs font-mono text-center">
        success<br/>#10b981
      </div>
      <div class="p-3 rounded-lg bg-status-warning text-white text-xs font-mono text-center">
        warning<br/>#f59e0b
      </div>
      <div class="p-3 rounded-lg bg-status-error text-white text-xs font-mono text-center">
        error<br/>#ef4444
      </div>
      <div class="p-3 rounded-lg bg-status-info text-white text-xs font-mono text-center">
        info<br/>#0284c7
      </div>
    </div>
  </section>

  <!-- Interactive Token Budget Indicator -->
  <section class="bg-theme-panel p-6 rounded-xl border border-theme-subtle shadow-elevation-1 space-y-4">
    <h2 class="text-xl font-semibold">Interactive Context Token Budget Indicator</h2>
    <p class="text-theme-secondary text-sm">
      Adjust the token slider to test visual warning thresholds (&gt;80% warning, &gt;100% critical).
    </p>

    <div class="space-y-3">
      <div class="flex justify-between items-center text-sm font-mono">
        <span>Current Tokens: {sampleTokens.toLocaleString()} / {maxTokens.toLocaleString()}</span>
        <span
          class="px-2 py-0.5 rounded text-xs font-semibold uppercase"
          style="background-color: {budgetStatus.color}22; color: {budgetStatus.color}"
        >
          {budgetStatus.status} ({(budgetStatus.ratio * 100).toFixed(1)}%)
        </span>
      </div>

      <input
        type="range"
        min="10000"
        max="150000"
        step="2500"
        bind:value={sampleTokens}
        class="w-full accent-brand-500 cursor-pointer"
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
</main>
