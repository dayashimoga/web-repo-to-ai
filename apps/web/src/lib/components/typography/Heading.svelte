<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    level?: 1 | 2 | 3 | 4 | 5 | 6;
    size?: 'base' | 'lg' | 'xl' | '2xl' | '3xl' | '4xl' | '5xl';
    id?: string;
    class?: string;
    children?: Snippet;
  }

  let {
    level = 1,
    size,
    id,
    class: className = '',
    children
  }: Props = $props();

  const defaultSizeClasses: Record<1 | 2 | 3 | 4 | 5 | 6, string> = {
    1: 'text-3xl sm:text-4xl lg:text-5xl font-bold tracking-tight',
    2: 'text-2xl sm:text-3xl font-bold tracking-tight',
    3: 'text-xl sm:text-2xl font-semibold tracking-tight',
    4: 'text-lg sm:text-xl font-semibold',
    5: 'text-base sm:text-lg font-medium',
    6: 'text-sm sm:text-base font-medium text-theme-secondary'
  };

  const overrideSizeClasses: Record<string, string> = {
    base: 'text-base font-medium',
    lg: 'text-lg font-semibold',
    xl: 'text-xl font-semibold',
    '2xl': 'text-2xl font-bold tracking-tight',
    '3xl': 'text-3xl font-bold tracking-tight',
    '4xl': 'text-4xl font-bold tracking-tight',
    '5xl': 'text-5xl font-bold tracking-tight'
  };

  let computedClasses = $derived(
    `${size ? overrideSizeClasses[size] : defaultSizeClasses[level]} text-theme-primary ${className}`.trim()
  );
</script>

{#if level === 1}
  <h1 {id} class={computedClasses}>{@render children?.()}</h1>
{:else if level === 2}
  <h2 {id} class={computedClasses}>{@render children?.()}</h2>
{:else if level === 3}
  <h3 {id} class={computedClasses}>{@render children?.()}</h3>
{:else if level === 4}
  <h4 {id} class={computedClasses}>{@render children?.()}</h4>
{:else if level === 5}
  <h5 {id} class={computedClasses}>{@render children?.()}</h5>
{:else if level === 6}
  <h6 {id} class={computedClasses}>{@render children?.()}</h6>
{/if}
