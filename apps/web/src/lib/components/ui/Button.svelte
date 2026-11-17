<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    variant?: 'primary' | 'secondary' | 'outline' | 'ghost' | 'danger';
    size?: 'sm' | 'md' | 'lg';
    type?: 'button' | 'submit' | 'reset';
    disabled?: boolean;
    loading?: boolean;
    fullWidth?: boolean;
    class?: string;
    ariaLabel?: string;
    href?: string;
    target?: string;
    rel?: string;
    onclick?: (e: MouseEvent) => void;
    children?: Snippet;
  }

  let {
    variant = 'primary',
    size = 'md',
    type = 'button',
    disabled = false,
    loading = false,
    fullWidth = false,
    class: className = '',
    ariaLabel,
    href,
    target,
    rel,
    onclick,
    children
  }: Props = $props();

  const variantClasses: Record<string, string> = {
    primary: 'bg-brand-600 hover:bg-brand-700 active:bg-brand-800 text-white shadow-elevation-1 focus-visible:ring-brand-500 border border-transparent',
    secondary: 'bg-theme-subtle hover:bg-surface-200 dark:hover:bg-surface-700 text-theme-primary border border-theme-subtle focus-visible:ring-brand-500',
    outline: 'bg-transparent hover:bg-theme-subtle text-theme-primary border border-theme-strong focus-visible:ring-brand-500',
    ghost: 'bg-transparent hover:bg-theme-subtle text-theme-secondary hover:text-theme-primary focus-visible:ring-brand-500 border border-transparent',
    danger: 'bg-status-error hover:bg-red-600 active:bg-red-700 text-white shadow-elevation-1 focus-visible:ring-red-500 border border-transparent'
  };

  const sizeClasses: Record<string, string> = {
    sm: 'text-xs px-2.5 py-1.5 rounded-md gap-1.5',
    md: 'text-sm px-4 py-2 rounded-lg gap-2',
    lg: 'text-base px-5 py-2.5 rounded-xl gap-2.5'
  };

  let isDisabled = $derived(disabled || loading);

  let computedClasses = $derived(
    [
      'inline-flex items-center justify-center font-medium select-none',
      'transition-all duration-150',
      'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-offset-2 dark:focus-visible:ring-offset-surface-900',
      'disabled:opacity-50 disabled:cursor-not-allowed disabled:pointer-events-none',
      variantClasses[variant],
      sizeClasses[size],
      fullWidth ? 'w-full' : '',
      className
    ]
      .filter(Boolean)
      .join(' ')
  );
</script>

{#if href}
  <a
    {href}
    {target}
    rel={target === '_blank' && !rel ? 'noopener noreferrer' : rel}
    aria-label={ariaLabel}
    class={computedClasses}
    {onclick}
  >
    {#if loading}
      <svg
        class="animate-spin -ml-0.5 h-4 w-4 text-current"
        xmlns="http://www.w3.org/2000/svg"
        fill="none"
        viewBox="0 0 24 24"
        aria-hidden="true"
      >
        <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
        <path
          class="opacity-75"
          fill="currentColor"
          d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
        ></path>
      </svg>
    {/if}
    {@render children?.()}
  </a>
{:else}
  <button
    {type}
    disabled={isDisabled}
    aria-disabled={isDisabled}
    aria-busy={loading}
    aria-label={ariaLabel}
    class={computedClasses}
    {onclick}
  >
    {#if loading}
      <svg
        class="animate-spin -ml-0.5 h-4 w-4 text-current"
        xmlns="http://www.w3.org/2000/svg"
        fill="none"
        viewBox="0 0 24 24"
        aria-hidden="true"
      >
        <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
        <path
          class="opacity-75"
          fill="currentColor"
          d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
        ></path>
      </svg>
    {/if}
    {@render children?.()}
  </button>
{/if}
