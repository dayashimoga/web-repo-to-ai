<script lang="ts">
  import { onMount } from 'svelte';
  import { Header, Footer } from '$lib/components/navigation';
  import { JobWorkspace } from '$lib/components/workspace';
  import { createInitialJob, type ConversionJob } from '$lib/types/job';
  import { detectUrlSource } from '$lib/utils/url-detector';

  let isDark = $state(true);
  let job = $state<ConversionJob>(
    createInitialJob('https://github.com/dayashimoga/web-repo-to-ai', {
      detectedSource: 'GitHub Repository',
      targetFormat: 'markdown',
      modelPreset: 'claude-3.5-sonnet'
    })
  );

  onMount(() => {
    if (typeof window !== 'undefined') {
      const params = new URLSearchParams(window.location.search);
      const urlParam = params.get('url');
      const formatParam = params.get('format') as
        | 'markdown'
        | 'json'
        | 'xml'
        | 'txt'
        | 'jsonl'
        | 'zip'
        | null;
      const presetParam = params.get('preset');
      const budgetParam = params.get('budget');

      if (urlParam) {
        const detection = detectUrlSource(urlParam);
        job = createInitialJob(urlParam, {
          detectedSource: detection.label,
          targetFormat: formatParam || 'markdown',
          modelPreset: presetParam || 'claude-3.5-sonnet',
          budgetLimit: budgetParam ? parseInt(budgetParam, 10) : 160000
        });
      }
    }
  });

  function handleCancel(jobId: string) {
    console.info(`[Workspace] Job cancelled: ${jobId}`);
  }

  function handleRetry(retriedJob: ConversionJob) {
    console.info(`[Workspace] Retrying job: ${retriedJob.id}`);
  }
</script>

<svelte:head>
  <title>Conversion Workspace - web-repo-to-ai</title>
  <meta
    name="description"
    content="Real-time multi-stage AI context conversion workspace with complete file accounting and telemetry."
  />
</svelte:head>

<!-- Global Site Header -->
<Header bind:isDark />

<!-- Main Conversion Workspace Content -->
<main id="main-content" class="flex-1 py-10 md:py-14">
  <div class="max-w-6xl mx-auto px-4 sm:px-6 lg:px-8">
    <JobWorkspace
      bind:job
      oncancel={handleCancel}
      onretry={handleRetry}
    />
  </div>
</main>

<!-- Global Site Footer -->
<Footer />
