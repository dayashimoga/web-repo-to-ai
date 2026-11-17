<script lang="ts">
  import { Badge, Button } from '$lib/components/ui';
  import { detectUrlSource, type UrlDetectionResult } from '$lib/utils/url-detector';

  interface Props {
    value?: string;
    placeholder?: string;
    label?: string;
    disabled?: boolean;
    loading?: boolean;
    class?: string;
    onsubmit?: (result: UrlDetectionResult) => void;
    onchange?: (result: UrlDetectionResult) => void;
  }

  let {
    value = $bindable(''),
    placeholder = 'https://github.com/owner/repo or https://docs.example.com',
    label = 'Target URL or GitHub Repository',
    disabled = false,
    loading = false,
    class: className = '',
    onsubmit,
    onchange
  }: Props = $props();

  const inputId = `url-input-${Math.random().toString(36).substring(2, 9)}`;

  let detection = $derived(detectUrlSource(value));

  // Notify parent on detection change
  $effect(() => {
    if (onchange) {
      onchange(detection);
    }
  });

  function handleClear() {
    value = '';
  }

  function handleSubmit(e: Event) {
    e.preventDefault();
    if (detection.isValid && detection.isSafe && onsubmit) {
      onsubmit(detection);
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape' && value) {
      handleClear();
    }
  }
</script>

<form
  onsubmit={handleSubmit}
  class="w-full space-y-3 transition-colors duration-150 {className}"
  role="search"
  aria-label="URL Ingestion Form"
>
  <!-- Label & Dynamic Status Header -->
  <div class="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-2">
    <label for={inputId} class="text-sm font-semibold text-theme-primary select-none">
      {label}
    </label>

    <!-- Source Classification Badge with Accessible Live Region -->
    <div class="flex items-center gap-2">
      <!-- Screen reader announcement live region -->
      <span role="status" aria-live="polite" class="sr-only">
        {detection.label}
      </span>

      <Badge variant={detection.badgeVariant} size="sm" dot={detection.isValid}>
        {detection.label}
      </Badge>

      <!-- GitHub AST Context Preview -->
      {#if detection.githubMetadata}
        <span class="hidden sm:inline-flex items-center gap-1 text-xs font-mono text-theme-secondary">
          <span class="text-theme-muted">{detection.githubMetadata.owner}/</span>
          <span class="font-bold text-theme-primary">{detection.githubMetadata.repo}</span>
          {#if detection.githubMetadata.ref}
            <span class="text-brand-500 font-semibold">
              @{detection.githubMetadata.ref}
            </span>
          {/if}
        </span>
      {/if}
    </div>
  </div>

  <!-- Input Field Container -->
  <div class="flex flex-col sm:flex-row items-stretch sm:items-center gap-3">
    <div class="relative flex-1 flex items-center">
      <!-- Leading Icon -->
      <div class="absolute left-3.5 pointer-events-none text-theme-muted" aria-hidden="true">
        {#if detection.sourceType === 'github_repo' || detection.sourceType === 'github_tree' || detection.sourceType === 'github_blob'}
          <!-- GitHub Octocat Icon -->
          <svg class="h-5 w-5 fill-current" viewBox="0 0 24 24">
            <path
              fill-rule="evenodd"
              clip-rule="evenodd"
              d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.53 1.032 1.53 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0112 6.844c.85.004 1.705.115 2.504.337 1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.019 10.019 0 0022 12.017C22 6.484 17.522 2 12 2z"
            />
          </svg>
        {:else if detection.sourceType === 'sitemap'}
          <!-- Sitemap XML Icon -->
          <svg class="h-5 w-5 stroke-current" fill="none" viewBox="0 0 24 24">
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
            />
          </svg>
        {:else if detection.sourceType === 'ssrf_blocked' || detection.sourceType === 'disallowed_protocol'}
          <!-- Security Shield Blocked Icon -->
          <svg class="h-5 w-5 text-status-error" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
            />
          </svg>
        {:else}
          <!-- Globe / URL Icon -->
          <svg class="h-5 w-5 stroke-current" fill="none" viewBox="0 0 24 24">
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M21 12a9 9 0 01-9 9m9-9a9 9 0 00-9-9m9 9H3m9 9a9 9 0 01-9-9m9 9c1.657 0 3-4.03 3-9s-1.343-9-3-9m0 18c-1.657 0-3-4.03-3-9s1.343-9 3-9m-9 9a9 9 0 019-9"
            />
          </svg>
        {/if}
      </div>

      <!-- Main HTML Input -->
      <input
        id={inputId}
        type="url"
        {placeholder}
        {disabled}
        bind:value
        onkeydown={handleKeydown}
        aria-label={label}
        aria-invalid={!detection.isValid && detection.sourceType !== 'empty'}
        aria-describedby={detection.errorMessage ? `${inputId}-error` : undefined}
        class={[
          'w-full pl-11 pr-10 py-3 text-sm sm:text-base rounded-xl border transition-all duration-150',
          'bg-theme-panel text-theme-primary placeholder:text-theme-muted',
          'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-offset-1 dark:focus-visible:ring-offset-surface-900',
          !detection.isSafe || detection.sourceType === 'invalid_syntax'
            ? 'border-status-error focus-visible:ring-status-error'
            : detection.isValid
              ? 'border-brand-500/60 focus-visible:ring-brand-500 shadow-elevation-1'
              : 'border-theme-subtle hover:border-theme-strong focus-visible:ring-brand-500',
          disabled ? 'opacity-50 cursor-not-allowed bg-theme-subtle' : ''
        ]
          .filter(Boolean)
          .join(' ')}
      />

      <!-- Clear Button -->
      {#if value && !disabled}
        <button
          type="button"
          onclick={handleClear}
          aria-label="Clear URL input"
          class="absolute right-3 p-1.5 rounded-lg text-theme-muted hover:text-theme-primary hover:bg-theme-subtle focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-brand-500 transition-colors"
        >
          <svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      {/if}
    </div>

    <!-- Submit Ingestion Button -->
    <Button
      type="submit"
      variant="primary"
      size="md"
      loading={loading}
      disabled={disabled || !detection.isValid || !detection.isSafe}
      ariaLabel="Start Ingestion Job"
      class="sm:h-[50px] whitespace-nowrap min-w-[150px]"
    >
      {loading ? 'Verifying...' : 'Convert URL'}
    </Button>
  </div>

  <!-- Client-Side Security Error / Invalidation Banner -->
  {#if detection.errorMessage}
    <div
      id="{inputId}-error"
      role="alert"
      class="p-3.5 rounded-xl text-xs sm:text-sm border flex items-start gap-2.5 bg-status-error/10 border-status-error/30 text-red-700 dark:text-red-300 animate-fadeIn"
    >
      <span class="font-bold text-base leading-none" aria-hidden="true">⚠</span>
      <span class="flex-1 leading-relaxed">{detection.errorMessage}</span>
    </div>
  {/if}
</form>
