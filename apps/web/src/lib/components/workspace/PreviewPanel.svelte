<script lang="ts">
  import { Badge, Button } from '$lib/components/ui';
  import { Mono } from '$lib/components/typography';
  import {
    type PreviewFile,
    type PreviewFormat,
    PREVIEW_FORMATS,
    formatContentForExport,
    generateSampleExtractedFiles,
    estimateTokens
  } from '$lib/types/preview';
  import { highlightLines, type HighlightedLine } from '$lib/utils/syntax-highlighter';
  import { defaultApiClient } from '$lib/api';

  interface Props {
    jobId?: string;
    files?: PreviewFile[];
    sourceUrl?: string;
    selectedFormat?: PreviewFormat;
    class?: string;
    onformatChange?: (format: PreviewFormat) => void;
    oncopy?: (format: PreviewFormat, text: string) => void;
  }

  let {
    jobId,
    files = $bindable(
      generateSampleExtractedFiles('https://github.com/dayashimoga/web-repo-to-ai')
    ),
    sourceUrl = 'https://github.com/dayashimoga/web-repo-to-ai',
    selectedFormat = $bindable<PreviewFormat>('markdown'),
    class: className = '',
    onformatChange,
    oncopy
  }: Props = $props();

  // Selection state
  let selectedFileId = $state<string | 'all'>('all');
  let searchQuery = $state('');
  let fileSearchQuery = $state('');
  let isCopied = $state(false);
  let showLineNumbers = $state(true);
  let wrapText = $state(false);
  let isSidebarOpen = $state(true);

  // Live backend preview state
  let isLoading = $state(false);
  let isBackendLive = $state(false);
  let isTruncated = $state(false);
  let serverTotalLines = $state<number | null>(null);
  let serverTotalTokens = $state<number | null>(null);
  let serverTotalBytes = $state<number | null>(null);
  let serverContent = $state<string | null>(null);

  // Filtered files for the sidebar
  let filteredFiles = $derived(
    files.filter((f) =>
      f.path.toLowerCase().includes(fileSearchQuery.toLowerCase())
    )
  );

  // Active files to format
  let activeFiles = $derived(
    selectedFileId === 'all'
      ? files
      : files.filter((f) => f.id === selectedFileId)
  );

  // Raw generated export text (uses server preview if loaded, else local fallback)
  let rawContent = $derived(
    serverContent !== null
      ? serverContent
      : formatContentForExport(activeFiles, selectedFormat, { sourceUrl })
  );

  // Filtered lines for preview search
  let highlightedLinesList = $derived.by<HighlightedLine[]>(() => {
    const lines = highlightLines(rawContent, selectedFormat);
    if (!searchQuery.trim()) {
      return lines;
    }
    const q = searchQuery.toLowerCase();
    return lines.filter(
      (l) =>
        l.html.toLowerCase().includes(q) ||
        String(l.lineNumber).includes(q)
    );
  });

  // Dynamic token and byte metrics
  let totalTokens = $derived(serverTotalTokens ?? estimateTokens(rawContent));
  let totalBytes = $derived(serverTotalBytes ?? new Blob([rawContent]).size);
  let totalLines = $derived(serverTotalLines ?? rawContent.split('\n').length);

  // Asynchronously query live preview from API when jobId is available
  async function fetchBackendPreview(targetJobId: string, format: PreviewFormat, fileId: string | 'all') {
    if (!targetJobId) return;
    isLoading = true;

    try {
      const targetFilePath = fileId !== 'all' ? files.find((f) => f.id === fileId)?.path : undefined;
      const res = await defaultApiClient.getJobPreview(targetJobId, {
        format,
        maxLines: 2000,
        filePath: targetFilePath
      });

      if (res && res.content) {
        serverContent = res.content;
        isTruncated = res.truncated;
        serverTotalLines = res.total_lines;
        serverTotalTokens = res.total_tokens;
        serverTotalBytes = res.total_bytes;
        isBackendLive = true;

        if (res.files && res.files.length > 0 && files.length <= 3) {
          files = res.files.map((rf) => ({
            id: rf.id,
            path: rf.path,
            language: rf.language,
            sizeBytes: rf.byte_size,
            tokenCount: rf.token_count,
            content: '',
            lineCount: rf.line_count
          }));
        }
      }
    } catch {
      // Graceful degradation to local preview
      serverContent = null;
      isBackendLive = false;
      isTruncated = false;
      serverTotalLines = null;
      serverTotalTokens = null;
      serverTotalBytes = null;
    } finally {
      isLoading = false;
    }
  }

  $effect(() => {
    if (jobId) {
      fetchBackendPreview(jobId, selectedFormat, selectedFileId);
    }
  });

  function handleSelectFormat(fmt: PreviewFormat) {
    selectedFormat = fmt;
    onformatChange?.(fmt);
    if (jobId) {
      fetchBackendPreview(jobId, fmt, selectedFileId);
    }
  }

  function handleSelectFile(fileId: string | 'all') {
    selectedFileId = fileId;
    if (jobId) {
      fetchBackendPreview(jobId, selectedFormat, fileId);
    }
  }

  async function handleCopy() {
    let success = false;
    try {
      if (typeof navigator !== 'undefined' && navigator.clipboard && navigator.clipboard.writeText) {
        await navigator.clipboard.writeText(rawContent);
        success = true;
      } else if (typeof document !== 'undefined') {
        const textArea = document.createElement('textarea');
        textArea.value = rawContent;
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
      console.warn('Copy to clipboard failed:', err);
    }

    if (success || typeof navigator === 'undefined') {
      isCopied = true;
      oncopy?.(selectedFormat, rawContent);
      setTimeout(() => {
        isCopied = false;
      }, 2000);
    }
  }

  function handleDownload() {
    if (typeof window === 'undefined') return;
    const currentOpt = PREVIEW_FORMATS.find((f) => f.id === selectedFormat);
    const ext = currentOpt?.extension || 'txt';
    const mime = currentOpt?.mime || 'text/plain';
    const filename = `context-export.${ext}`;

    if (jobId) {
      const downloadUrl = defaultApiClient.getArtifactUrl(jobId, selectedFormat);
      const a = document.createElement('a');
      a.href = downloadUrl;
      a.download = filename;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      return;
    }

    const blob = new Blob([rawContent], { type: mime });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = filename;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  }
</script>

<div
  role="region"
  aria-label="Content Preview Panel"
  class="w-full rounded-2xl border border-theme-subtle bg-theme-panel shadow-elevation-1 overflow-hidden {className}"
>
  <!-- Top Bar: Format Tablist & Action Toolbar -->
  <div
    class="p-4 sm:p-5 border-b border-theme-subtle bg-theme-panel/80 flex flex-col lg:flex-row lg:items-center justify-between gap-4"
  >
    <!-- Accessible Format Tablist -->
    <div
      role="tablist"
      aria-label="Export format preview tabs"
      class="flex flex-wrap items-center gap-1.5 p-1 bg-theme-subtle/60 rounded-xl border border-theme-subtle"
    >
      {#each PREVIEW_FORMATS as fmt}
        <button
          type="button"
          role="tab"
          id="tab-{fmt.id}"
          aria-selected={selectedFormat === fmt.id}
          aria-controls="panel-{fmt.id}"
          class="flex items-center gap-2 px-3 py-1.5 rounded-lg text-xs font-semibold transition-all duration-150 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-brand-500 {selectedFormat ===
          fmt.id
            ? 'bg-brand-600 text-white shadow-elevation-1'
            : 'text-theme-secondary hover:text-theme-primary hover:bg-theme-panel'}"
          onclick={() => handleSelectFormat(fmt.id)}
        >
          <span>{fmt.label}</span>
          <span
            class="px-1.5 py-0.2 rounded text-[10px] font-mono {selectedFormat ===
            fmt.id
              ? 'bg-white/20 text-white'
              : 'bg-theme-subtle text-theme-muted'}"
          >
            {fmt.badge}
          </span>
        </button>
      {/each}
    </div>

    <!-- Quick Actions Toolbar -->
    <div class="flex flex-wrap items-center gap-2">
      <!-- Search within code -->
      <div class="relative">
        <input
          type="text"
          placeholder="Filter preview..."
          aria-label="Filter preview content lines"
          bind:value={searchQuery}
          class="w-36 sm:w-44 text-xs px-2.5 py-1.5 rounded-lg border border-theme-subtle bg-theme-panel text-theme-primary placeholder-theme-muted focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-brand-500"
        />
        {#if searchQuery}
          <button
            type="button"
            aria-label="Clear preview search"
            class="absolute right-2 top-1.5 text-xs text-theme-muted hover:text-theme-primary"
            onclick={() => (searchQuery = '')}
          >
            ✕
          </button>
        {/if}
      </div>

      <!-- Toggle Line Numbers Button -->
      <button
        type="button"
        title="Toggle line numbers"
        aria-label="Toggle line numbers"
        aria-pressed={showLineNumbers}
        class="px-2.5 py-1.5 rounded-lg text-xs font-medium border border-theme-subtle transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-brand-500 {showLineNumbers
          ? 'bg-theme-subtle text-theme-primary'
          : 'bg-theme-panel text-theme-muted hover:text-theme-primary'}"
        onclick={() => (showLineNumbers = !showLineNumbers)}
      >
        #
      </button>

      <!-- Toggle Word Wrap Button -->
      <button
        type="button"
        title="Toggle word wrap"
        aria-label="Toggle word wrap"
        aria-pressed={wrapText}
        class="px-2.5 py-1.5 rounded-lg text-xs font-medium border border-theme-subtle transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-brand-500 {wrapText
          ? 'bg-theme-subtle text-theme-primary'
          : 'bg-theme-panel text-theme-muted hover:text-theme-primary'}"
        onclick={() => (wrapText = !wrapText)}
      >
        Wrap
      </button>

      <!-- Copy to Clipboard Button with Live Feedback -->
      <Button
        variant={isCopied ? 'secondary' : 'secondary'}
        size="sm"
        onclick={handleCopy}
        ariaLabel="Copy current preview to clipboard"
      >
        {#if isCopied}
          <span class="text-status-success font-semibold flex items-center gap-1">
            <span>✓</span>
            <span>Copied!</span>
          </span>
        {:else}
          <span class="flex items-center gap-1">
            <span>📋</span>
            <span>Copy</span>
          </span>
        {/if}
      </Button>

      <!-- Live Announcement Region for Screen Readers -->
      <div role="status" aria-live="polite" class="sr-only">
        {isCopied ? 'Preview text copied to clipboard successfully' : ''}
      </div>

      <!-- Download Button -->
      <Button
        variant="primary"
        size="sm"
        onclick={handleDownload}
        ariaLabel="Download context export file"
      >
        <span>Download {selectedFormat.toUpperCase()}</span>
      </Button>
    </div>
  </div>

  <!-- Content Metrics Banner -->
  <div
    class="px-5 py-2.5 bg-theme-subtle/30 border-b border-theme-subtle flex flex-wrap items-center justify-between gap-3 text-xs"
  >
    <div class="flex items-center gap-3">
      <!-- Sidebar Toggle on Mobile -->
      <button
        type="button"
        aria-label="Toggle files sidebar"
        aria-expanded={isSidebarOpen}
        class="text-xs text-theme-muted hover:text-theme-primary font-medium flex items-center gap-1 sm:hidden focus-visible:outline-none"
        onclick={() => (isSidebarOpen = !isSidebarOpen)}
      >
        <span>📁 Files ({files.length})</span>
        <span>{isSidebarOpen ? '▼' : '▶'}</span>
      </button>

      <span class="hidden sm:inline text-theme-muted font-medium">Context Metrics:</span>
      <Badge variant="brand" size="sm">
        ~{totalTokens.toLocaleString()} Tokens
      </Badge>
      <Badge variant="default" size="sm">
        {totalLines.toLocaleString()} Lines
      </Badge>
      <Badge variant="default" size="sm">
        {(totalBytes / 1024).toFixed(1)} KB
      </Badge>

      {#if isBackendLive}
        <Badge variant="success" size="sm" dot>
          Live API Preview
        </Badge>
      {:else if jobId}
        <Badge variant="default" size="sm">
          Local Preview
        </Badge>
      {/if}

      {#if isLoading}
        <span class="text-brand-500 font-mono text-[11px] animate-pulse">
          Loading preview...
        </span>
      {/if}
    </div>

    <!-- Active Selection Indicator -->
    <div class="flex items-center gap-2 text-theme-secondary font-mono text-[11px]">
      <span class="text-theme-muted">Scope:</span>
      <span class="font-semibold text-theme-primary">
        {selectedFileId === 'all'
          ? `All ${files.length} Files (Concatenated)`
          : files.find((f) => f.id === selectedFileId)?.path || 'File'}
      </span>
    </div>
  </div>

  <!-- Truncation Banner -->
  {#if isTruncated}
    <div
      role="status"
      aria-label="Preview truncation notice"
      class="px-5 py-2 bg-status-warning/15 border-b border-status-warning/30 text-status-warning text-xs flex items-center justify-between"
    >
      <div class="flex items-center gap-2 font-medium">
        <span>⚠</span>
        <span>
          Preview windowed to first {serverTotalLines ?? 2000} lines for smooth rendering. Download complete artifact to view full export.
        </span>
      </div>
    </div>
  {/if}

  <!-- Main Body: File Tree Sidebar + Syntax-Highlighted Code Editor -->
  <div class="grid grid-cols-1 md:grid-cols-12 min-h-[460px]">
    <!-- File Structure Sidebar (Left 3 or 4 cols) -->
    {#if isSidebarOpen}
      <aside
        aria-label="Extracted repository files list"
        class="md:col-span-4 lg:col-span-3 border-b md:border-b-0 md:border-r border-theme-subtle bg-theme-panel/50 flex flex-col"
      >
        <!-- Sidebar Search -->
        <div class="p-3 border-b border-theme-subtle">
          <input
            type="text"
            placeholder="Search files..."
            aria-label="Search extracted files"
            bind:value={fileSearchQuery}
            class="w-full text-xs px-2.5 py-1.5 rounded-lg border border-theme-subtle bg-theme-panel text-theme-primary placeholder-theme-muted focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-brand-500"
          />
        </div>

        <!-- File List Items -->
        <div class="flex-1 overflow-y-auto max-h-72 md:max-h-[500px] p-2 space-y-1">
          <!-- Option: All Files Combined -->
          <button
            type="button"
            class="w-full text-left px-2.5 py-2 rounded-lg text-xs font-semibold flex items-center justify-between transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-brand-500 {selectedFileId ===
            'all'
              ? 'bg-brand-600/10 text-brand-500 border border-brand-500/30'
              : 'text-theme-secondary hover:text-theme-primary hover:bg-theme-subtle'}"
            onclick={() => handleSelectFile('all')}
          >
            <span class="flex items-center gap-1.5 truncate">
              <span>📦</span>
              <span>All Files ({files.length})</span>
            </span>
            <span class="font-mono text-[10px] text-theme-muted">
              {files.reduce((a, b) => a + b.tokenCount, 0).toLocaleString()} t
            </span>
          </button>

          <!-- Individual Files -->
          {#each filteredFiles as file (file.id)}
            <button
              type="button"
              class="w-full text-left px-2.5 py-1.5 rounded-lg text-xs font-mono flex items-center justify-between transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-brand-500 {selectedFileId ===
              file.id
                ? 'bg-brand-600/10 text-brand-500 border border-brand-500/30 font-semibold'
                : 'text-theme-secondary hover:text-theme-primary hover:bg-theme-subtle'}"
              onclick={() => handleSelectFile(file.id)}
            >
              <span class="flex items-center gap-1.5 truncate">
                <span class="text-theme-muted text-[11px]">
                  {file.language === 'markdown'
                    ? '📝'
                    : file.language === 'rust'
                      ? '🦀'
                      : file.language === 'toml'
                        ? '⚙'
                        : '📄'}
                </span>
                <span class="truncate">{file.path}</span>
              </span>
              <span class="text-[10px] text-theme-muted shrink-0 ml-1">
                ~{file.tokenCount} t
              </span>
            </button>
          {/each}

          {#if filteredFiles.length === 0}
            <div class="p-3 text-center text-xs text-theme-muted">
              No files match "{fileSearchQuery}"
            </div>
          {/if}
        </div>
      </aside>
    {/if}

    <!-- Main Syntax-Highlighted Code Viewer (Right 8 or 9 cols) -->
    <div
      role="tabpanel"
      id="panel-{selectedFormat}"
      aria-labelledby="tab-{selectedFormat}"
      class="{isSidebarOpen
        ? 'md:col-span-8 lg:col-span-9'
        : 'md:col-span-12'} bg-surface-950 text-surface-100 flex flex-col overflow-hidden"
    >
      <!-- Scrollable Code Surface -->
      <div
        class="flex-1 overflow-x-auto overflow-y-auto max-h-[500px] p-4 font-mono text-xs select-text leading-relaxed"
      >
        {#if highlightedLinesList.length === 0}
          <div class="py-12 text-center text-surface-500">
            No lines match query "{searchQuery}"
          </div>
        {:else}
          <table class="w-full border-collapse">
            <tbody>
              {#each highlightedLinesList as line (line.lineNumber)}
                <tr class="hover:bg-surface-900/60 transition-colors">
                  <!-- Line number gutter -->
                  {#if showLineNumbers}
                    <td
                      aria-hidden="true"
                      class="pr-4 pl-1 text-right select-none text-surface-600 font-mono text-[11px] align-top w-12 border-r border-surface-800 shrink-0"
                    >
                      {line.lineNumber}
                    </td>
                  {/if}

                  <!-- Code Line content -->
                  <td
                    class="pl-4 align-top font-mono {wrapText
                      ? 'whitespace-pre-wrap break-all'
                      : 'whitespace-pre'}"
                  >
                    <!-- eslint-disable-next-line svelte/no-at-html-tags -->
                    {@html line.html}
                  </td>
                </tr>
              {/each}
            </tbody>
          </table>
        {/if}
      </div>
    </div>
  </div>
</div>
