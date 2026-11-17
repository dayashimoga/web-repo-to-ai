<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    block?: boolean;
    class?: string;
    children?: Snippet;
  }

  let {
    block = false,
    class: className = '',
    children
  }: Props = $props();
</script>

{#if block}
  <pre class="p-4 rounded-lg bg-surface-900 text-surface-50 font-mono text-xs sm:text-sm overflow-x-auto border border-theme-subtle shadow-elevation-1 {className}"><code>{@render children?.()}</code></pre>
{:else}
  <code class="px-1.5 py-0.5 rounded bg-theme-subtle text-brand-600 dark:text-brand-300 font-mono text-xs sm:text-sm border border-theme-subtle {className}">{@render children?.()}</code>
{/if}
