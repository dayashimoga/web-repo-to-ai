<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    value?: string;
    type?: 'text' | 'url' | 'search' | 'email' | 'password' | 'number';
    placeholder?: string;
    label?: string;
    ariaLabel?: string;
    helperText?: string;
    error?: string;
    disabled?: boolean;
    required?: boolean;
    id?: string;
    name?: string;
    clearable?: boolean;
    class?: string;
    prefix?: Snippet;
    oninput?: (e: Event) => void;
    onkeydown?: (e: KeyboardEvent) => void;
  }

  let {
    value = $bindable(''),
    type = 'text',
    placeholder,
    label,
    ariaLabel,
    helperText,
    error,
    disabled = false,
    required = false,
    id,
    name,
    clearable = false,
    class: className = '',
    prefix,
    oninput,
    onkeydown
  }: Props = $props();

  let autoId = $derived(id || `input-${Math.random().toString(36).substring(2, 9)}`);
  let describedBy = $derived(
    error ? `${autoId}-error` : helperText ? `${autoId}-helper` : undefined
  );

  function handleClear() {
    value = '';
  }
</script>

<div class="w-full space-y-1.5 {className}">
  {#if label}
    <label for={autoId} class="block text-sm font-medium text-theme-primary">
      {label}
      {#if required}
        <span class="text-status-error" aria-hidden="true">*</span>
      {/if}
    </label>
  {/if}

  <div class="relative flex items-center">
    {#if prefix}
      <div class="absolute left-3 flex items-center pointer-events-none text-theme-muted">
        {@render prefix()}
      </div>
    {/if}

    <input
      id={autoId}
      {name}
      {type}
      {placeholder}
      {disabled}
      {required}
      bind:value
      aria-label={ariaLabel}
      aria-invalid={!!error}
      aria-describedby={describedBy}
      class={[
        'w-full px-3.5 py-2 text-sm rounded-lg border transition-colors',
        'bg-theme-panel text-theme-primary placeholder:text-theme-muted',
        'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-offset-1 dark:focus-visible:ring-offset-surface-900',
        error
          ? 'border-status-error focus-visible:ring-status-error'
          : 'border-theme-subtle hover:border-theme-strong focus-visible:ring-brand-500',
        disabled ? 'opacity-50 cursor-not-allowed bg-theme-subtle' : '',
        prefix ? 'pl-9' : '',
        clearable && value ? 'pr-9' : ''
      ]
        .filter(Boolean)
        .join(' ')}
      {oninput}
      {onkeydown}
    />

    {#if clearable && value && !disabled}
      <button
        type="button"
        onclick={handleClear}
        class="absolute right-2.5 p-1 rounded-md text-theme-muted hover:text-theme-primary focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-brand-500"
        aria-label="Clear input"
      >
        <svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
        </svg>
      </button>
    {/if}
  </div>

  {#if error}
    <p id="{autoId}-error" class="text-xs text-status-error font-medium" role="alert">
      {error}
    </p>
  {:else if helperText}
    <p id="{autoId}-helper" class="text-xs text-theme-muted">
      {helperText}
    </p>
  {/if}
</div>
