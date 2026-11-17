<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    cols?: 1 | 2 | 3 | 4 | 6 | 12;
    gap?: 1 | 2 | 3 | 4 | 6 | 8;
    class?: string;
    children?: Snippet;
  }

  let {
    cols = 1,
    gap = 4,
    class: className = '',
    children
  }: Props = $props();

  const colsClasses: Record<number, string> = {
    1: 'grid-cols-1',
    2: 'grid-cols-1 sm:grid-cols-2',
    3: 'grid-cols-1 sm:grid-cols-2 lg:grid-cols-3',
    4: 'grid-cols-1 sm:grid-cols-2 lg:grid-cols-4',
    6: 'grid-cols-2 sm:grid-cols-3 lg:grid-cols-6',
    12: 'grid-cols-12'
  };

  const gapClasses: Record<number, string> = {
    1: 'gap-1',
    2: 'gap-2',
    3: 'gap-3',
    4: 'gap-4',
    6: 'gap-6',
    8: 'gap-8'
  };

  let computedClasses = $derived(
    `grid ${colsClasses[cols] ?? 'grid-cols-1'} ${gapClasses[gap] ?? 'gap-4'} ${className}`.trim()
  );
</script>

<div class={computedClasses}>
  {@render children?.()}
</div>
