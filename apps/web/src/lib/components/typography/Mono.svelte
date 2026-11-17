<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    as?: 'span' | 'code' | 'div';
    size?: 'xs' | 'sm' | 'base' | 'lg';
    class?: string;
    children?: Snippet;
  }

  let {
    as = 'span',
    size = 'sm',
    class: className = '',
    children
  }: Props = $props();

  const sizeClasses: Record<string, string> = {
    xs: 'text-xs',
    sm: 'text-sm',
    base: 'text-base',
    lg: 'text-lg'
  };

  let computedClasses = $derived(
    `font-mono tabular-nums ${sizeClasses[size]} ${className}`.trim()
  );
</script>

{#if as === 'code'}
  <code class={computedClasses}>{@render children?.()}</code>
{:else if as === 'div'}
  <div class={computedClasses}>{@render children?.()}</div>
{:else}
  <span class={computedClasses}>{@render children?.()}</span>
{/if}
