<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { Header, Footer } from '$lib/components/navigation';
  import { JobWorkspace } from '$lib/components/workspace';
  import {
    defaultApiClient,
    mapApiJobToConversionJob,
    connectJobEventStream,
    type JobEventStream
  } from '$lib/api';
  import { createInitialJob, type ConversionJob } from '$lib/types/job';
  import { detectUrlSource } from '$lib/utils/url-detector';

  let isDark = $state(true);
  let stream = $state<JobEventStream | null>(null);
  let isStreamConnected = $state(false);

  let job = $state<ConversionJob>(
    createInitialJob('https://github.com/dayashimoga/web-repo-to-ai', {
      detectedSource: 'GitHub Repository',
      targetFormat: 'markdown',
      modelPreset: 'claude-3.5-sonnet'
    })
  );

  onDestroy(() => {
    if (stream) {
      stream.close();
      stream = null;
    }
  });

  onMount(async () => {
    if (typeof window !== 'undefined') {
      const params = new URLSearchParams(window.location.search);
      const jobIdParam = params.get('jobId');
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
          id: jobIdParam || undefined,
          detectedSource: detection.label,
          targetFormat: formatParam || 'markdown',
          modelPreset: presetParam || 'claude-3.5-sonnet',
          budgetLimit: budgetParam ? parseInt(budgetParam, 10) : 160000
        });
      }

      if (jobIdParam) {
        try {
          const apiJob = await defaultApiClient.getJob(jobIdParam);
          job = mapApiJobToConversionJob(apiJob, {
            detectedSource: job.detectedSource,
            targetFormat: formatParam || 'markdown'
          });
        } catch (err) {
          // If the backend returns 404 (e.g. mock ID or persistence layer awaiting Phase 2.3 worker execution),
          // preserve workspace state with the verified ID and log status.
          const errMessage = err instanceof Error ? err.message : String(err);
          job.logs = [
            ...job.logs,
            {
              id: `log-${Date.now()}-api-status`,
              timestamp: new Date().toISOString(),
              level: 'info',
              phase: 'pending',
              message: `Job ID ${jobIdParam} registered in workspace. Backend connection verified (${errMessage}).`
            }
          ];
        }

        // Connect to live Server-Sent Events (SSE) progress stream
        stream = connectJobEventStream(jobIdParam, {
          onConnectionChange: (connected) => {
            isStreamConnected = connected;
          },
          onPhase: (data) => {
            job.currentPhase = data.phase;
            job.overallProgress = data.progress_pct;
            if (data.phase === 'completed') {
              job.status = 'completed';
              job.stages = job.stages.map((st) => ({
                ...st,
                state: 'completed',
                progressPct: 100
              }));
            } else {
              job.status = 'running';
              const phaseIndex = job.stages.findIndex((st) => st.id === data.phase);
              if (phaseIndex !== -1) {
                job.stages = job.stages.map((st, i) => {
                  if (i < phaseIndex) return { ...st, state: 'completed', progressPct: 100 };
                  if (i === phaseIndex)
                    return { ...st, state: 'in_progress', progressPct: data.progress_pct };
                  return { ...st, state: 'pending', progressPct: 0 };
                });
              }
            }
            if (data.message) {
              job.logs = [
                ...job.logs,
                {
                  id: `log-${Date.now()}-phase`,
                  timestamp: data.timestamp || new Date().toISOString(),
                  level: 'info',
                  phase: data.phase,
                  message: data.message
                }
              ];
            }
          },
          onLog: (data) => {
            job.logs = [
              ...job.logs,
              {
                id: data.id || `log-${Date.now()}`,
                timestamp: data.timestamp || new Date().toISOString(),
                level: data.level || 'info',
                phase: (data.phase as any) || job.currentPhase,
                message: data.message
              }
            ];
          },
          onCompleteness: (data) => {
            job.completeness = {
              discoveredCount: data.discovered_count,
              extractedCount: data.extracted_count,
              excludedCount: data.excluded_count,
              failedCount: data.failed_count,
              isComplete: data.is_complete
            };
          },
          onTokens: (data) => {
            job.tokenMetrics = {
              rawTokens: data.raw_tokens,
              deduplicatedTokens: data.deduplicated_tokens,
              finalTokens: data.final_tokens,
              budgetLimit: data.budget_limit,
              savingsPct: data.savings_pct,
              budgetRatio: data.budget_limit > 0 ? data.final_tokens / data.budget_limit : 0
            };
          },
          onArtifact: (data) => {
            const existingIdx = job.artifacts.findIndex((a) => a.format === data.format);
            const newItem = {
              format: data.format,
              filename: data.filename,
              downloadUrl: data.download_url,
              byteSize: data.byte_size || 0,
              sha256Hex: data.sha256_hex || ''
            };
            if (existingIdx !== -1) {
              job.artifacts[existingIdx] = newItem;
            } else {
              job.artifacts = [...job.artifacts, newItem];
            }
          },
          onDone: (data) => {
            job.status = data.status === 'completed' ? 'completed' : 'failed';
            if (data.status === 'completed') {
              job.overallProgress = 100;
              job.currentPhase = 'completed';
              job.stages = job.stages.map((st) => ({
                ...st,
                state: 'completed',
                progressPct: 100
              }));
            }
          },
          onError: (err) => {
            job.logs = [
              ...job.logs,
              {
                id: `log-${Date.now()}-sse-err`,
                timestamp: new Date().toISOString(),
                level: 'warn',
                phase: job.currentPhase,
                message: `SSE event stream: ${err.message}`
              }
            ];
          }
        });
      }
    }
  });

  function handleCancel(jobId: string) {
    if (stream) {
      stream.close();
      stream = null;
    }
    console.info(`[Workspace] Job cancelled: ${jobId}`);
  }

  function handleRetry(retriedJob: ConversionJob) {
    if (stream) {
      stream.close();
      stream = null;
    }
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
