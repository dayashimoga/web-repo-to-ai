<script lang="ts">
  import { Badge, Button } from '$lib/components/ui';
  import { Mono } from '$lib/components/typography';
  import PreviewPanel from './PreviewPanel.svelte';
  import TokenBudgetBar from './TokenBudgetBar.svelte';
  import DownloadControls from './DownloadControls.svelte';
  import type { PreviewFormat } from '$lib/types/preview';
  import {
    type ConversionJob,
    type JobStatus,
    type PipelineStageInfo,
    calculateOverallProgress,
    createInitialJob,
    verifyCompletenessInvariant
  } from '$lib/types/job';

  interface Props {
    job?: ConversionJob;
    class?: string;
    oncancel?: (jobId: string) => void;
    onretry?: (job: ConversionJob) => void;
    onnavigateHome?: () => void;
  }

  let {
    job = $bindable(
      createInitialJob('https://github.com/dayashimoga/web-repo-to-ai', {
        detectedSource: 'GitHub Repository',
        targetFormat: 'markdown',
        modelPreset: 'claude-3.5-sonnet'
      })
    ),
    class: className = '',
    oncancel,
    onretry,
    onnavigateHome
  }: Props = $props();

  let isPreviewVisible = $state(true);
  let overallPct = $derived(job.overallProgress ?? calculateOverallProgress(job.stages));
  let isCompleteInvariant = $derived(verifyCompletenessInvariant(job.completeness));

  function getPreviewFormat(fmt: string): PreviewFormat {
    if (fmt === 'zip') return 'markdown';
    return (fmt as PreviewFormat) || 'markdown';
  }

  const statusVariantMap: Record<JobStatus, 'default' | 'brand' | 'success' | 'warning' | 'error' | 'info'> = {
    queued: 'default',
    leased: 'info',
    running: 'brand',
    completed: 'success',
    partial: 'warning',
    failed: 'error',
    cancelled: 'default'
  };

  function handleCancel() {
    if (job.status === 'running' || job.status === 'queued' || job.status === 'leased') {
      job.status = 'cancelled';
      job.logs = [
        ...job.logs,
        {
          id: `log-${Date.now()}`,
          timestamp: new Date().toISOString(),
          level: 'warn',
          phase: job.currentPhase,
          message: 'Job cancelled by user request.'
        }
      ];
      oncancel?.(job.id);
    }
  }

  function handleRetry() {
    job.status = 'running';
    job.currentPhase = 'acquisition';
    job.overallProgress = 10;
    job.error = undefined;
    job.stages = job.stages.map((st, i) =>
      i === 0
        ? { ...st, state: 'in_progress', progressPct: 20 }
        : { ...st, state: 'pending', progressPct: 0 }
    );
    job.logs = [
      ...job.logs,
      {
        id: `log-${Date.now()}`,
        timestamp: new Date().toISOString(),
        level: 'info',
        phase: 'acquisition',
        message: 'Restarting conversion pipeline...'
      }
    ];
    onretry?.(job);
  }

  function formatTime(isoString: string): string {
    try {
      const d = new Date(isoString);
      return d.toTimeString().split(' ')[0] || isoString;
    } catch {
      return isoString;
    }
  }
</script>

<div class="w-full space-y-8 {className}">
  <!-- Top Navigation & Job Header Card -->
  <div class="p-6 rounded-2xl bg-theme-panel border border-theme-subtle shadow-elevation-1 space-y-4">
    <div class="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4">
      <!-- Left side: Breadcrumb & Title -->
      <div class="space-y-1">
        <div class="flex items-center gap-2 text-xs text-theme-muted">
          {#if onnavigateHome}
            <button
              type="button"
              onclick={onnavigateHome}
              class="hover:text-theme-primary transition-colors flex items-center gap-1 focus-visible:outline-none focus-visible:underline"
            >
              <span>← Home</span>
            </button>
          {:else}
            <a href="/" class="hover:text-theme-primary transition-colors flex items-center gap-1">
              <span>← Home</span>
            </a>
          {/if}
          <span>/</span>
          <span class="text-theme-secondary font-medium">Conversion Workspace</span>
        </div>

        <div class="flex flex-wrap items-center gap-2.5 pt-1">
          <h1 class="text-xl sm:text-2xl font-extrabold tracking-tight text-theme-primary">
            Conversion Job
          </h1>
          <Mono class="text-xs px-2 py-0.5 rounded bg-theme-subtle border border-theme-subtle font-bold">
            {job.id}
          </Mono>
          <Badge variant={statusVariantMap[job.status]} size="sm" dot={job.status === 'running'}>
            {job.status.toUpperCase()}
          </Badge>
        </div>

        <!-- Target Source URL -->
        <p class="text-xs sm:text-sm text-theme-secondary break-all flex items-center gap-1.5 pt-1">
          <span class="text-theme-muted font-medium">Source:</span>
          <a
            href={job.sourceUrl}
            target="_blank"
            rel="noopener noreferrer"
            class="underline hover:text-brand-500 font-mono text-xs transition-colors"
          >
            {job.sourceUrl}
          </a>
        </p>
      </div>

      <!-- Right side: Quick Actions -->
      <div class="flex flex-wrap items-center gap-2.5 sm:self-center">
        <Badge variant="brand" size="sm">
          Format: {job.targetFormat.toUpperCase()}
        </Badge>
        <Badge variant="default" size="sm">
          {job.modelPreset}
        </Badge>

        {#if job.status === 'running' || job.status === 'queued' || job.status === 'leased'}
          <Button variant="danger" size="sm" onclick={handleCancel} ariaLabel="Cancel active conversion job">
            Cancel Job
          </Button>
        {:else if job.status === 'failed' || job.status === 'cancelled'}
          <Button variant="primary" size="sm" onclick={handleRetry} ariaLabel="Retry failed conversion job">
            Retry Job
          </Button>
        {/if}
      </div>
    </div>

    <!-- Overall Progress Bar Landmark -->
    <div class="pt-4 border-t border-theme-subtle space-y-2">
      <div class="flex items-center justify-between text-sm">
        <span class="font-semibold text-theme-primary flex items-center gap-2">
          <span>Overall Pipeline Progress</span>
          {#if job.status === 'running'}
            <span class="h-2 w-2 rounded-full bg-brand-500 animate-pulse" aria-hidden="true"></span>
          {/if}
        </span>
        <span class="font-mono font-bold text-brand-500 text-base" aria-live="polite">
          {overallPct}%
        </span>
      </div>

      <div
        role="progressbar"
        aria-valuenow={overallPct}
        aria-valuemin={0}
        aria-valuemax={100}
        aria-label="Overall conversion pipeline progress"
        class="w-full bg-theme-subtle rounded-full h-3 overflow-hidden border border-theme-subtle shadow-inner"
      >
        <div
          class="h-full bg-gradient-to-r from-brand-600 to-accent-500 transition-all duration-300 rounded-full"
          style="width: {overallPct}%"
        ></div>
      </div>
    </div>
  </div>

  <!-- 4-Stage Pipeline Tracker Cards -->
  <div class="space-y-3">
    <div class="text-xs font-semibold uppercase tracking-wider text-theme-muted">
      Pipeline Stage Execution
    </div>
    <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
      {#each job.stages as stage, index}
        <div
          class="p-5 rounded-2xl border transition-all duration-150 flex flex-col justify-between {stage.state ===
          'in_progress'
            ? 'bg-theme-panel border-brand-500 shadow-elevation-2'
            : stage.state === 'completed'
              ? 'bg-theme-panel border-status-success/40'
              : stage.state === 'failed'
                ? 'bg-theme-panel border-status-error/40'
                : 'bg-theme-panel/50 border-theme-subtle opacity-75'}"
        >
          <!-- Stage Card Top -->
          <div>
            <div class="flex items-center justify-between mb-3">
              <span class="text-xs font-mono font-bold text-theme-muted">
                0{index + 1}
              </span>
              <Badge
                variant={stage.state === 'completed'
                  ? 'success'
                  : stage.state === 'in_progress'
                    ? 'brand'
                    : stage.state === 'failed'
                      ? 'error'
                      : 'default'}
                size="sm"
                dot={stage.state === 'in_progress'}
              >
                {stage.state === 'in_progress'
                  ? 'Running'
                  : stage.state === 'completed'
                    ? 'Completed'
                    : stage.state === 'failed'
                      ? 'Failed'
                      : 'Pending'}
              </Badge>
            </div>

            <h3 class="text-sm font-bold text-theme-primary mb-1">
              {stage.name}
            </h3>
            <p class="text-xs text-theme-secondary leading-relaxed">
              {stage.description}
            </p>
          </div>

          <!-- Stage Progress or Status Details -->
          <div class="mt-4 pt-3 border-t border-theme-subtle/60">
            {#if stage.state === 'in_progress'}
              <div class="space-y-1">
                <div class="flex justify-between text-[11px] font-mono text-brand-500 font-semibold">
                  <span>In Progress</span>
                  <span>{stage.progressPct}%</span>
                </div>
                <div class="w-full bg-theme-subtle rounded-full h-1.5 overflow-hidden">
                  <div
                    class="h-full bg-brand-500 transition-all duration-150 rounded-full"
                    style="width: {stage.progressPct}%"
                  ></div>
                </div>
              </div>
            {:else if stage.state === 'completed'}
              <div class="text-[11px] font-semibold text-status-success flex items-center gap-1">
                <span>✓ Stage Verified</span>
              </div>
            {:else if stage.state === 'failed'}
              <div class="text-[11px] font-semibold text-status-error flex items-center gap-1">
                <span>⚠ Stage Failed</span>
              </div>
            {:else}
              <div class="text-[11px] text-theme-muted">
                <span>Awaiting prior stages...</span>
              </div>
            {/if}
          </div>
        </div>
      {/each}
    </div>
  </div>

  <!-- Token Statistics & Context Window Budget Utilization -->
  <TokenBudgetBar
    bind:metrics={job.tokenMetrics}
    bind:modelPreset={job.modelPreset}
  />

  <!-- Completeness File Accounting Matrix -->
  <div class="p-6 rounded-2xl bg-theme-panel border border-theme-subtle space-y-4 shadow-elevation-1">
    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-2">
      <div>
        <h3 class="text-sm font-bold text-theme-primary">
          Repository Completeness Accounting
        </h3>
        <p class="text-xs text-theme-secondary mt-0.5">
          Guaranteed invariant: <Mono>discovered = extracted + excluded + failed</Mono>
        </p>
      </div>

      <Badge variant={isCompleteInvariant ? 'success' : 'error'} size="sm" dot>
        {isCompleteInvariant ? 'Invariant Verified' : 'Accounting Discrepancy'}
      </Badge>
    </div>

    <div class="grid grid-cols-2 sm:grid-cols-4 gap-4 pt-2">
      <div class="p-4 rounded-xl bg-theme-subtle/50 border border-theme-subtle">
        <div class="text-xs text-theme-muted uppercase font-semibold">Discovered</div>
        <div class="text-xl font-mono font-bold text-theme-primary mt-1">
          {job.completeness.discoveredCount}
        </div>
        <div class="text-[10px] text-theme-muted mt-0.5">Total eligible files</div>
      </div>

      <div class="p-4 rounded-xl bg-theme-subtle/50 border border-theme-subtle">
        <div class="text-xs text-status-success uppercase font-semibold">Extracted</div>
        <div class="text-xl font-mono font-bold text-status-success mt-1">
          {job.completeness.extractedCount}
        </div>
        <div class="text-[10px] text-theme-muted mt-0.5">Canonical CDM units</div>
      </div>

      <div class="p-4 rounded-xl bg-theme-subtle/50 border border-theme-subtle">
        <div class="text-xs text-status-warning uppercase font-semibold">Excluded</div>
        <div class="text-xl font-mono font-bold text-status-warning mt-1">
          {job.completeness.excludedCount}
        </div>
        <div class="text-[10px] text-theme-muted mt-0.5">Filtered by rule</div>
      </div>

      <div class="p-4 rounded-xl bg-theme-subtle/50 border border-theme-subtle">
        <div class="text-xs text-status-error uppercase font-semibold">Failed</div>
        <div class="text-xl font-mono font-bold text-status-error mt-1">
          {job.completeness.failedCount}
        </div>
        <div class="text-[10px] text-theme-muted mt-0.5">Unparseable / timeout</div>
      </div>
    </div>
  </div>

  <!-- Extracted Content Multi-Format Preview Section -->
  <div class="space-y-3">
    <div class="flex items-center justify-between">
      <div class="text-xs font-semibold uppercase tracking-wider text-theme-muted">
        Extracted Content & Multi-Format Preview
      </div>
      <button
        type="button"
        class="text-xs font-medium text-brand-500 hover:text-brand-400 focus-visible:outline-none focus-visible:underline"
        onclick={() => (isPreviewVisible = !isPreviewVisible)}
      >
        {isPreviewVisible ? 'Hide Preview ▲' : 'Show Preview ▼'}
      </button>
    </div>

    {#if isPreviewVisible}
      <PreviewPanel
        jobId={job.id}
        sourceUrl={job.sourceUrl}
        selectedFormat={getPreviewFormat(job.targetFormat)}
      />
    {/if}
  </div>

  <!-- Export Artifacts & Checksum Download Controls -->
  <DownloadControls
    jobId={job.id}
    artifacts={job.artifacts}
    sourceUrl={job.sourceUrl}
  />

  <!-- Live Execution Log Terminal Stream -->
  <div class="rounded-2xl border border-surface-800 bg-surface-950 text-surface-100 overflow-hidden shadow-elevation-2">
    <div class="px-5 py-3 border-b border-surface-800 flex items-center justify-between bg-surface-900/60">
      <div class="flex items-center gap-2">
        <div class="h-3 w-3 rounded-full bg-red-500/80"></div>
        <div class="h-3 w-3 rounded-full bg-yellow-500/80"></div>
        <div class="h-3 w-3 rounded-full bg-green-500/80"></div>
        <span class="text-xs font-mono text-surface-400 ml-2 font-semibold">
          Execution Telemetry Log
        </span>
      </div>
      <span class="text-xs font-mono text-surface-500">
        {job.logs.length} events
      </span>
    </div>

    <!-- Scrollable windowed log window -->
    <div
      role="log"
      aria-live="polite"
      aria-label="Real-time conversion job logs"
      class="p-4 max-h-64 overflow-y-auto font-mono text-xs space-y-1.5 select-text"
    >
      {#each job.logs.slice(-200) as entry (entry.id)}
        <div class="flex items-start gap-2.5 leading-relaxed">
          <span class="text-surface-500 shrink-0">
            [{formatTime(entry.timestamp)}]
          </span>
          <span
            class="px-1.5 py-0.2 rounded text-[10px] uppercase font-bold shrink-0 {entry.level ===
            'success'
              ? 'bg-status-success/20 text-green-400'
              : entry.level === 'warn'
                ? 'bg-status-warning/20 text-yellow-400'
                : entry.level === 'error'
                  ? 'bg-status-error/20 text-red-400'
                  : 'bg-brand-500/20 text-brand-300'}"
          >
            {entry.phase}
          </span>
          <span
            class="flex-1 {entry.level === 'error'
              ? 'text-red-300 font-semibold'
              : entry.level === 'warn'
                ? 'text-yellow-300'
                : entry.level === 'success'
                  ? 'text-green-300'
                  : 'text-surface-200'}"
          >
            {entry.message}
          </span>
        </div>
      {/each}
    </div>
  </div>

  <!-- Failure Recovery Banner -->
  {#if job.status === 'failed' || job.error}
    <div
      role="alert"
      class="p-5 rounded-2xl border border-status-error/40 bg-status-error/10 text-sm text-status-error space-y-3"
    >
      <div class="flex items-center gap-2 font-bold text-base">
        <span>⚠</span>
        <span>Conversion Failed ({job.error?.code || 'UNKNOWN_ERROR'})</span>
      </div>
      <p class="text-xs sm:text-sm text-theme-secondary leading-relaxed">
        {job.error?.message || 'An error occurred during extraction. Check the telemetry log for details.'}
      </p>
      <div>
        <Button variant="danger" size="sm" onclick={handleRetry}>
          Retry Conversion Pipeline
        </Button>
      </div>
    </div>
  {/if}
</div>
