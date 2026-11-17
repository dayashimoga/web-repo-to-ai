<script lang="ts">
  import { onMount } from 'svelte';
  import { Badge, Button } from '$lib/components/ui';
  import { Mono } from '$lib/components/typography';
  import {
    type PreviewFile,
    generateSampleExtractedFiles
  } from '$lib/types/preview';
  import {
    type ExportArtifact,
    buildExportArtifacts,
    generateChecksumsTxt,
    generateChecksumManifestJson,
    triggerDownload,
    triggerBlobDownload,
    deriveArtifactBaseName
  } from '$lib/types/download';
  import type { JobArtifactItem } from '$lib/types/job';

  interface Props {
    jobId?: string;
    artifacts?: JobArtifactItem[];
    files?: PreviewFile[];
    sourceUrl?: string;
    class?: string;
    ondownload?: (artifact: ExportArtifact) => void;
    ondownloadAll?: (count: number) => void;
  }

  let {
    jobId,
    artifacts: propArtifacts = [],
    files = $bindable(
      generateSampleExtractedFiles('https://github.com/dayashimoga/web-repo-to-ai')
    ),
    sourceUrl = 'https://github.com/dayashimoga/web-repo-to-ai',
    class: className = '',
    ondownload,
    ondownloadAll
  }: Props = $props();

  let artifacts = $state<ExportArtifact[]>([]);
  let copiedChecksumId = $state<string | null>(null);
  let liveStatus = $state<string | null>(null);
  let isManifestOpen = $state(false);

  // Asynchronously compute artifacts and SHA-256 digests
  async function refreshArtifacts() {
    artifacts = await buildExportArtifacts(files, sourceUrl, {
      jobId,
      backendArtifacts: propArtifacts
    });
  }

  onMount(() => {
    refreshArtifacts();
  });

  $effect(() => {
    // Recompute when files, sourceUrl, propArtifacts, or jobId change
    if (files || sourceUrl || propArtifacts || jobId) {
      refreshArtifacts();
    }
  });

  async function handleCopyChecksum(artifact: ExportArtifact) {
    let success = false;
    try {
      if (typeof navigator !== 'undefined' && navigator.clipboard && navigator.clipboard.writeText) {
        await navigator.clipboard.writeText(artifact.sha256Hex);
        success = true;
      } else if (typeof document !== 'undefined') {
        const textArea = document.createElement('textarea');
        textArea.value = artifact.sha256Hex;
        textArea.style.position = 'fixed';
        textArea.style.left = '-9999px';
        textArea.style.top = '0';
        document.body.appendChild(textArea);
        textArea.focus();
        textArea.select();
        success = document.execCommand('copy');
        document.body.removeChild(textArea);
      }
    } catch (err) {
      console.warn('Failed to copy checksum:', err);
    }

    if (success || typeof navigator === 'undefined') {
      copiedChecksumId = artifact.id;
      liveStatus = `Copied SHA-256 for ${artifact.filename}`;
      setTimeout(() => {
        copiedChecksumId = null;
      }, 2000);
    }
  }

  function handleDownloadArtifact(artifact: ExportArtifact) {
    liveStatus = `Downloading ${artifact.filename}...`;
    try {
      triggerDownload(artifact.filename, artifact.content, artifact.mimeType, artifact.downloadUrl);
      liveStatus = `Downloaded ${artifact.filename}`;
      ondownload?.(artifact);
    } catch (err) {
      liveStatus = `Failed to download ${artifact.filename}`;
      console.warn('Download error:', err);
    }
  }

  function handleDownloadChecksumsTxt() {
    const baseName = deriveArtifactBaseName(sourceUrl);
    const filename = `${baseName}-checksums.txt`;
    const backendChecksumUrl = jobId ? `/api/v1/jobs/${jobId}/artifacts/checksums` : undefined;
    const content = generateChecksumsTxt(artifacts, sourceUrl);
    triggerDownload(filename, content, 'text/plain', backendChecksumUrl);
    liveStatus = `Downloaded ${filename}`;
  }

  function handleDownloadManifestJson() {
    const baseName = deriveArtifactBaseName(sourceUrl);
    const filename = `${baseName}-manifest.json`;
    const backendManifestUrl = jobId ? `/api/v1/jobs/${jobId}/artifacts/manifest` : undefined;
    const content = generateChecksumManifestJson(artifacts, sourceUrl);
    triggerDownload(filename, content, 'application/json', backendManifestUrl);
    liveStatus = `Downloaded ${filename}`;
  }

  function handleDownloadAll() {
    // If ZIP package is available, prefer downloading the comprehensive ZIP bundle
    const zipArtifact = artifacts.find((a) => a.format === 'zip');
    if (zipArtifact && zipArtifact.downloadUrl) {
      handleDownloadArtifact(zipArtifact);
      liveStatus = `Started download of complete archive bundle (${zipArtifact.filename})`;
      ondownloadAll?.(1);
      return;
    }

    artifacts.forEach((art, idx) => {
      setTimeout(() => {
        triggerDownload(art.filename, art.content, art.mimeType, art.downloadUrl);
      }, idx * 200);
    });
    liveStatus = `Started download of all ${artifacts.length} export formats`;
    ondownloadAll?.(artifacts.length);
  }

  function truncateHash(hash: string): string {
    if (!hash || hash.length < 16) return hash;
    return `${hash.slice(0, 8)}...${hash.slice(-8)}`;
  }
</script>

<div
  role="region"
  aria-label="Format Selection and Download Controls"
  class="w-full rounded-2xl border border-theme-subtle bg-theme-panel p-6 shadow-elevation-1 space-y-6 {className}"
>
  <!-- Top Bar: Header and Batch Actions -->
  <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
    <div>
      <div class="flex items-center gap-2">
        <h3 class="text-base sm:text-lg font-bold text-theme-primary">
          Export Artifacts & Download Controls
        </h3>
        <Badge variant="brand" size="sm">
          {artifacts.length} Formats Ready
        </Badge>
        {#if jobId}
          <Badge variant="success" size="sm" dot>
            Verified API Streams
          </Badge>
        {/if}
      </div>
      <p class="text-xs text-theme-secondary mt-1">
        Cryptographically verified context packages with SHA-256 integrity checksums.
      </p>
    </div>

    <!-- Batch Download Buttons -->
    <div class="flex flex-wrap items-center gap-2.5">
      {#if artifacts.some((a) => a.format === 'zip')}
        <Button
          variant="primary"
          size="sm"
          onclick={() => {
            const zip = artifacts.find((a) => a.format === 'zip');
            if (zip) handleDownloadArtifact(zip);
          }}
          ariaLabel="Download complete ZIP package"
        >
          <span class="flex items-center gap-1.5">
            <span>📦</span>
            <span>Download ZIP Package</span>
          </span>
        </Button>
      {/if}

      <Button
        variant="secondary"
        size="sm"
        onclick={handleDownloadChecksumsTxt}
        ariaLabel="Download SHA-256 checksums file"
      >
        <span>Checksums (.txt)</span>
      </Button>

      <Button
        variant={artifacts.some((a) => a.format === 'zip') ? 'secondary' : 'primary'}
        size="sm"
        onclick={handleDownloadAll}
        ariaLabel="Download all available format artifacts"
      >
        <span>Download All Formats</span>
      </Button>
    </div>
  </div>

  <!-- Screen Reader Live Status Region -->
  <div role="status" aria-live="polite" class="sr-only">
    {liveStatus || ''}
  </div>

  <!-- Artifacts List / Grid -->
  <div class="grid grid-cols-1 gap-3">
    {#each artifacts as artifact (artifact.id)}
      <div
        class="p-4 rounded-xl border border-theme-subtle bg-theme-panel/70 hover:border-brand-500/50 transition-colors flex flex-col md:flex-row md:items-center justify-between gap-4"
      >
        <!-- Format Info & Description -->
        <div class="space-y-1 md:max-w-md">
          <div class="flex items-center gap-2">
            <span class="font-bold text-sm text-theme-primary">
              {artifact.label}
            </span>
            <span
              class="px-2 py-0.5 rounded text-[11px] font-mono font-bold bg-theme-subtle text-theme-primary border border-theme-subtle"
            >
              {artifact.badge}
            </span>
            <span class="text-xs font-mono text-theme-muted">
              {artifact.filename}
            </span>
          </div>
          <p class="text-xs text-theme-secondary leading-relaxed">
            {artifact.description}
          </p>
        </div>

        <!-- Middle: Size & SHA-256 Checksum -->
        <div class="flex flex-wrap items-center gap-3 text-xs font-mono">
          <span class="px-2 py-1 rounded bg-theme-subtle/60 text-theme-primary font-semibold">
            {(artifact.byteSize / 1024).toFixed(1)} KB
          </span>

          <!-- Checksum pill with copy button -->
          <div
            class="flex items-center gap-1.5 px-2.5 py-1 rounded-lg bg-surface-950 text-surface-200 border border-surface-800 text-[11px]"
          >
            <span class="text-surface-500">SHA-256:</span>
            <span title={artifact.sha256Hex}>
              {truncateHash(artifact.sha256Hex)}
            </span>
            <button
              type="button"
              class="ml-1 text-surface-400 hover:text-white transition-colors focus-visible:outline-none"
              title="Copy full SHA-256 hash"
              aria-label={`Copy SHA-256 checksum for ${artifact.filename}`}
              onclick={() => handleCopyChecksum(artifact)}
            >
              {#if copiedChecksumId === artifact.id}
                <span class="text-status-success font-bold">✓</span>
              {:else}
                <span>📋</span>
              {/if}
            </button>
          </div>
        </div>

        <!-- Right: Direct Download Button -->
        <div class="shrink-0 self-end md:self-center">
          <Button
            variant="secondary"
            size="sm"
            onclick={() => handleDownloadArtifact(artifact)}
            ariaLabel={`Download ${artifact.filename}`}
          >
            <span class="flex items-center gap-1">
              <span>⬇</span>
              <span>Download</span>
            </span>
          </Button>
        </div>
      </div>
    {/each}
  </div>

  <!-- Checksum Manifest Drawer Trigger & Content -->
  <div class="pt-3 border-t border-theme-subtle space-y-3">
    <div class="flex items-center justify-between text-xs">
      <button
        type="button"
        class="text-xs font-medium text-brand-500 hover:text-brand-400 flex items-center gap-1 focus-visible:outline-none focus-visible:underline"
        aria-expanded={isManifestOpen}
        onclick={() => (isManifestOpen = !isManifestOpen)}
      >
        <span>{isManifestOpen ? 'Hide Checksum Manifest ▲' : 'View Checksum Manifest (JSON/TXT) ▼'}</span>
      </button>

      <div class="flex items-center gap-2">
        <button
          type="button"
          class="text-xs text-theme-muted hover:text-theme-primary transition-colors focus-visible:outline-none focus-visible:underline"
          onclick={handleDownloadManifestJson}
        >
          Download manifest.json
        </button>
      </div>
    </div>

    {#if isManifestOpen}
      <div class="p-4 rounded-xl bg-surface-950 text-surface-200 border border-surface-800 font-mono text-xs overflow-x-auto space-y-2">
        <div class="text-[11px] text-surface-400 font-semibold uppercase">
          checksums.txt (Standard BSD / Coreutils Format)
        </div>
        <pre class="text-[11px] leading-relaxed text-surface-300">{generateChecksumsTxt(artifacts, sourceUrl)}</pre>
      </div>
    {/if}
  </div>
</div>
