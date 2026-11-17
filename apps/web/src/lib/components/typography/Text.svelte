<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    as?: 'p' | 'span' | 'div' | 'label';
    size?: 'xs' | 'sm' | 'base' | 'lg' | 'xl';
    weight?: 'regular' | 'medium' | 'semibold' | 'bold';
    tone?: 'primary' | 'secondary' | 'muted' | 'brand' | 'success' | 'warning' | 'error';
    class?: string;
    children?: Snippet;
  }

  let {
    as = 'p',
    size = 'base',
    weight = 'regular',
    tone = 'primary',
    class: className = '',
    children
  }: Props = $props();

  const sizeClasses: Record<string, string> = {
    xs: 'text-xs leading-4',
    sm: 'text-sm leading-5',
    base: 'text-base leading-6',
    lg: 'text-lg leading-7',
    xl: 'text-xl leading-7'
  };

  const weightClasses: Record<string, string> = {
    regular: 'font-normal',
    medium: 'font-medium',
    semibold: 'font-semibold',
    bold: 'font-bold'
  };

  const toneClasses: Record<string, string> = {
    primary: 'text-theme-primary',
    secondary: 'text-theme-secondary',
    muted: 'text-theme-muted',
    brand: 'text-brand-600 dark:text-brand-400',
    success: 'text-status-success dark:text-emerald-400',
    warning: 'text-status-warning dark:text-amber-400',
    error: 'text-status-error dark:text-rose-400'
  };

  let computedClasses = $derived(
    `${sizeClasses[size]} ${weightClasses[weight]} ${toneClasses[tone]} ${className}`.trim()
  );
</script>

{#if as === 'p'}
  <p class={computedClasses}>{@render children?.()}</p>
{:else if as === 'span'}
  <span class={computedClasses}>{@render children?.()}</span>
{:else if as === 'label'}
  <label class={computedClasses}>{@render children?.()}</label>
{:else}
  <div class={computedClasses}>{@render children?.()}</div>
{/if}
