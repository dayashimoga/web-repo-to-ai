<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    variant?: 'default' | 'brand' | 'success' | 'warning' | 'error' | 'info';
    size?: 'sm' | 'md';
    dot?: boolean;
    class?: string;
    children?: Snippet;
  }

  let {
    variant = 'default',
    size = 'md',
    dot = false,
    class: className = '',
    children
  }: Props = $props();

  const variantClasses: Record<string, string> = {
    default: 'bg-theme-subtle text-theme-secondary border-theme-subtle',
    brand: 'bg-brand-50 text-brand-700 border-brand-200 dark:bg-brand-950/60 dark:text-brand-300 dark:border-brand-800/60',
    success: 'bg-status-success-light text-status-success-dark border-status-success/30 dark:bg-status-success-dark/20 dark:text-emerald-300 dark:border-status-success/40',
    warning: 'bg-status-warning-light text-status-warning-dark border-status-warning/30 dark:bg-status-warning-dark/20 dark:text-amber-300 dark:border-status-warning/40',
    error: 'bg-status-error-light text-status-error-dark border-status-error/30 dark:bg-status-error-dark/20 dark:text-rose-300 dark:border-status-error/40',
    info: 'bg-status-info-light text-status-info-dark border-status-info/30 dark:bg-status-info-dark/20 dark:text-sky-300 dark:border-status-info/40'
  };

  const dotClasses: Record<string, string> = {
    default: 'bg-surface-400',
    brand: 'bg-brand-500',
    success: 'bg-status-success',
    warning: 'bg-status-warning',
    error: 'bg-status-error',
    info: 'bg-status-info'
  };

  const sizeClasses: Record<string, string> = {
    sm: 'text-xs px-2 py-0.5 rounded gap-1 font-mono',
    md: 'text-xs px-2.5 py-1 rounded-md gap-1.5 font-medium'
  };

  let computedClasses = $derived(
    `inline-flex items-center font-medium border select-none ${variantClasses[variant]} ${sizeClasses[size]} ${className}`.trim()
  );
</script>

<span class={computedClasses}>
  {#if dot}
    <span class="h-1.5 w-1.5 rounded-full {dotClasses[variant]}" aria-hidden="true"></span>
  {/if}
  {@render children?.()}
</span>
