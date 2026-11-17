<script lang="ts">
  import { Badge, Button } from '$lib/components/ui';
  import { Mono } from '$lib/components/typography';
  import { AI_MODEL_PRESETS, type AIModelPreset, type AIModelPresetId } from '$lib/types/settings';
  import type { JobTokenMetrics } from '$lib/types/job';
  import {
    evaluateBudgetUtilization,
    type BudgetUtilizationReport,
    type BudgetStatusLevel
  } from '$lib/types/token-budget';

  interface Props {
    metrics?: JobTokenMetrics;
    modelPreset?: string;
    class?: string;
    onpresetChange?: (presetId: string, budget: number) => void;
  }

  let {
    metrics = $bindable<JobTokenMetrics>({
      rawTokens: 74500,
      deduplicatedTokens: 52200,
      finalTokens: 48900,
      budgetLimit: 160000,
      savingsPct: 34,
      budgetRatio: 0.31
    }),
    modelPreset = $bindable<string>('claude-3.5-sonnet'),
    class: className = '',
    onpresetChange
  }: Props = $props();

  let report = $derived<BudgetUtilizationReport>(
    evaluateBudgetUtilization(metrics, modelPreset)
  );

  let activePreset = $derived<AIModelPreset>(
    AI_MODEL_PRESETS[modelPreset as AIModelPresetId] || AI_MODEL_PRESETS['claude-3.5-sonnet']
  );

  // Bar segment percentages
  let finalPct = $derived(Math.min(100, report.utilizationPct));
  let savingsRatioOfBudget = $derived(
    report.budgetLimit > 0
      ? Math.round((report.savingsTokens / report.budgetLimit) * 100)
      : 0
  );
  let ghostSavingsPct = $derived(
    Math.min(Math.max(0, 100 - finalPct), savingsRatioOfBudget)
  );
  let headroomPct = $derived(
    Math.max(0, 100 - finalPct - ghostSavingsPct)
  );

  const statusVariantMap: Record<
    BudgetStatusLevel,
    'success' | 'warning' | 'error'
  > = {
    optimal: 'success',
    warning: 'warning',
    exceeded: 'error'
  };

  function handleSelectPreset(presetId: string) {
    modelPreset = presetId;
    const selected = AI_MODEL_PRESETS[presetId as AIModelPresetId];
    if (selected) {
      metrics.budgetLimit = selected.defaultBudget;
      onpresetChange?.(presetId, selected.defaultBudget);
    }
  }
</script>

<div
  role="region"
  aria-label="Token Statistics and Context Budget Bar"
  class="w-full rounded-2xl border border-theme-subtle bg-theme-panel p-6 shadow-elevation-1 space-y-6 {className}"
>
  <!-- Header: Title and Model Context Window Selector -->
  <div class="flex flex-col lg:flex-row lg:items-center justify-between gap-4">
    <div>
      <div class="flex items-center gap-2">
        <h3 class="text-base sm:text-lg font-bold text-theme-primary">
          Token Statistics & Context Window Budget
        </h3>
        <Badge variant={statusVariantMap[report.status]} size="sm" dot>
          {report.status.toUpperCase()}
        </Badge>
      </div>
      <p class="text-xs text-theme-secondary mt-1">
        Context window consumption for <strong class="text-theme-primary">{activePreset.name}</strong> ({activePreset.contextWindow.toLocaleString()} tokens ceiling).
      </p>
    </div>

    <!-- Quick Model Preset Switcher -->
    <div class="flex flex-wrap items-center gap-1.5 p-1 bg-theme-subtle/60 rounded-xl border border-theme-subtle">
      {#each [
        { id: 'gemini-1.5-pro', label: 'Gemini 2M' },
        { id: 'gemini-1.5-flash', label: 'Gemini 1M' },
        { id: 'claude-3.5-sonnet', label: 'Claude 200k' },
        { id: 'gpt-4o', label: 'GPT-4o 128k' },
        { id: 'local-32k', label: 'Local 32k' }
      ] as opt}
        <button
          type="button"
          class="px-2.5 py-1 rounded-lg text-xs font-semibold transition-all duration-150 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-brand-500 {modelPreset ===
          opt.id
            ? 'bg-brand-600 text-white shadow-elevation-1'
            : 'text-theme-secondary hover:text-theme-primary hover:bg-theme-panel'}"
          onclick={() => handleSelectPreset(opt.id)}
        >
          {opt.label}
        </button>
      {/each}
    </div>
  </div>

  <!-- 4-Card Token Statistics Metric Grid -->
  <div class="grid grid-cols-2 lg:grid-cols-4 gap-4">
    <!-- Card 1: Raw Ingested -->
    <div class="p-4 rounded-xl bg-theme-subtle/40 border border-theme-subtle">
      <div class="text-[11px] font-semibold uppercase tracking-wider text-theme-muted">
        Raw Ingested Tokens
      </div>
      <div class="text-xl sm:text-2xl font-mono font-bold text-theme-primary mt-1">
        {report.rawTokens.toLocaleString()}
      </div>
      <div class="text-[11px] text-theme-muted mt-0.5">
        Pre-deduplication text
      </div>
    </div>

    <!-- Card 2: Deduplicated Tokens -->
    <div class="p-4 rounded-xl bg-theme-subtle/40 border border-theme-subtle">
      <div class="text-[11px] font-semibold uppercase tracking-wider text-theme-muted">
        Deduplicated Tokens
      </div>
      <div class="text-xl sm:text-2xl font-mono font-bold text-brand-500 mt-1">
        {report.deduplicatedTokens.toLocaleString()}
      </div>
      <div class="text-[11px] text-theme-muted mt-0.5">
        MinHash LSH reduced
      </div>
    </div>

    <!-- Card 3: Final Context Tokens -->
    <div class="p-4 rounded-xl bg-theme-subtle/40 border border-theme-subtle">
      <div class="text-[11px] font-semibold uppercase tracking-wider text-theme-muted">
        Final Context Output
      </div>
      <div class="text-xl sm:text-2xl font-mono font-bold text-status-success mt-1">
        {report.finalTokens.toLocaleString()}
      </div>
      <div class="text-[11px] text-theme-muted mt-0.5">
        Normalized AI prompt size
      </div>
    </div>

    <!-- Card 4: Savings Percentage -->
    <div class="p-4 rounded-xl bg-theme-subtle/40 border border-theme-subtle">
      <div class="text-[11px] font-semibold uppercase tracking-wider text-theme-muted">
        Token Optimization Savings
      </div>
      <div class="flex items-center gap-2 mt-1">
        <span class="text-xl sm:text-2xl font-mono font-bold text-status-success">
          {report.savingsPct}%
        </span>
        <Badge variant="success" size="sm">
          -{report.savingsTokens.toLocaleString()} t
        </Badge>
      </div>
      <div class="text-[11px] text-theme-muted mt-0.5">
        Redundancy eliminated
      </div>
    </div>
  </div>

  <!-- Visual Context Budget Bar -->
  <div class="space-y-2.5 pt-2 border-t border-theme-subtle">
    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-1 text-xs sm:text-sm">
      <div class="flex items-center gap-2 font-medium text-theme-primary">
        <span>Budget Consumption:</span>
        <Mono class="font-bold">
          {report.finalTokens.toLocaleString()} / {report.budgetLimit.toLocaleString()} tokens
        </Mono>
      </div>

      <div class="flex items-center gap-2 font-mono text-xs">
        <span class="text-theme-muted">Utilized:</span>
        <span
          class="font-bold {report.status === 'optimal'
            ? 'text-status-success'
            : report.status === 'warning'
              ? 'text-status-warning'
              : 'text-status-error'}"
        >
          {report.utilizationPct}%
        </span>
        <span class="text-theme-muted">|</span>
        <span class="text-theme-secondary">
          +{report.headroomTokens.toLocaleString()} tokens headroom
        </span>
      </div>
    </div>

    <!-- Layered Progress Bar Landmark -->
    <div
      role="progressbar"
      aria-valuenow={report.utilizationPct}
      aria-valuemin={0}
      aria-valuemax={100}
      aria-label="Context window token budget utilization"
      class="w-full bg-theme-subtle rounded-full h-4 overflow-hidden border border-theme-subtle shadow-inner flex relative"
    >
      <!-- Final Tokens Segment -->
      <div
        class="h-full transition-all duration-300 rounded-l-full {report.status ===
        'optimal'
          ? 'bg-gradient-to-r from-green-500 to-emerald-400'
          : report.status === 'warning'
            ? 'bg-gradient-to-r from-amber-500 to-yellow-400'
            : 'bg-gradient-to-r from-red-600 to-rose-500'}"
        style="width: {finalPct}%"
        title="Final Context: {report.finalTokens.toLocaleString()} tokens ({finalPct}%)"
      ></div>

      <!-- Ghost / Savings Segment (Demonstrating deduplication impact) -->
      {#if ghostSavingsPct > 0}
        <div
          class="h-full bg-brand-500/25 border-r border-brand-500/40 transition-all duration-300"
          style="width: {ghostSavingsPct}%"
          title="Deduplication Savings: {report.savingsTokens.toLocaleString()} tokens"
        ></div>
      {/if}

      <!-- 80% Safety Margin Guideline Marker -->
      <div
        class="absolute top-0 bottom-0 w-0.5 bg-theme-primary/40 pointer-events-none"
        style="left: 80%"
        title="80% recommended maximum prompt budget"
      ></div>
    </div>

    <!-- Legend & Context Headroom Callout -->
    <div class="flex flex-wrap items-center justify-between gap-3 text-[11px] text-theme-muted pt-1">
      <div class="flex items-center gap-4">
        <span class="flex items-center gap-1.5">
          <span
            class="h-2 w-2 rounded-full {report.status === 'optimal'
              ? 'bg-status-success'
              : report.status === 'warning'
                ? 'bg-status-warning'
                : 'bg-status-error'}"
          ></span>
          <span class="text-theme-secondary font-medium">Final Context ({finalPct}%)</span>
        </span>

        {#if ghostSavingsPct > 0}
          <span class="flex items-center gap-1.5">
            <span class="h-2 w-2 rounded-full bg-brand-500/50"></span>
            <span class="text-theme-secondary font-medium">Saved via Deduplication (~{ghostSavingsPct}%)</span>
          </span>
        {/if}

        <span class="flex items-center gap-1.5">
          <span class="h-2 w-2 rounded-full bg-theme-subtle border border-theme-subtle"></span>
          <span class="text-theme-secondary font-medium">Headroom Buffer ({headroomPct}%)</span>
        </span>
      </div>

      <div class="text-[11px] font-mono text-theme-secondary">
        80% safety margin: {(report.budgetLimit * 0.8).toLocaleString()} t
      </div>
    </div>
  </div>

  <!-- Status Advisory Banner -->
  {#if report.status === 'exceeded'}
    <div
      role="alert"
      class="p-4 rounded-xl border border-status-error/40 bg-status-error/10 text-xs text-status-error flex flex-col sm:flex-row sm:items-center justify-between gap-3"
    >
      <div class="flex items-start gap-2.5">
        <span class="font-bold text-sm shrink-0">⚠</span>
        <div>
          <strong class="font-bold">Context Window Limit Exceeded:</strong>
          <span> {report.statusMessage}</span>
        </div>
      </div>
      <Button
        variant="danger"
        size="sm"
        onclick={() => handleSelectPreset('gemini-1.5-pro')}
        ariaLabel="Switch model preset to Gemini 1.5 Pro"
      >
        Switch to Gemini 2M
      </Button>
    </div>
  {:else if report.status === 'warning'}
    <div
      role="status"
      class="p-4 rounded-xl border border-status-warning/40 bg-status-warning/10 text-xs text-yellow-800 dark:text-yellow-200 flex items-start gap-2.5"
    >
      <span class="font-bold text-sm shrink-0">⚡</span>
      <div>
        <strong class="font-bold">High Context Utilization:</strong>
        <span> {report.statusMessage}</span>
      </div>
    </div>
  {:else}
    <div
      class="p-3 rounded-xl border border-status-success/30 bg-status-success/5 text-xs text-green-800 dark:text-green-300 flex items-center gap-2"
    >
      <span class="font-bold">✓</span>
      <span>
        Optimal fit for <strong>{activePreset.name}</strong>. {report.headroomTokens.toLocaleString()} tokens remain for multi-turn conversations and reasoning.
      </span>
    </div>
  {/if}
</div>
