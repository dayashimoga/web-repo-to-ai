<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    content: string;
    position?: 'top' | 'bottom' | 'left' | 'right';
    class?: string;
    children?: Snippet;
  }

  let {
    content,
    position = 'top',
    class: className = '',
    children
  }: Props = $props();

  let visible = $state(false);
  const tooltipId = `tooltip-${Math.random().toString(36).substring(2, 9)}`;

  const positionClasses: Record<string, string> = {
    top: 'bottom-full left-1/2 -translate-x-1/2 mb-2',
    bottom: 'top-full left-1/2 -translate-x-1/2 mt-2',
    left: 'right-full top-1/2 -translate-y-1/2 mr-2',
    right: 'left-full top-1/2 -translate-y-1/2 ml-2'
  };
</script>

<span
  role="group"
  class="relative inline-flex items-center {className}"
  onmouseenter={() => (visible = true)}
  onmouseleave={() => (visible = false)}
  onfocusin={() => (visible = true)}
  onfocusout={() => (visible = false)}
  aria-describedby={visible ? tooltipId : undefined}
>
  {@render children?.()}

  {#if visible}
    <div
      id={tooltipId}
      role="tooltip"
      class="absolute z-40 px-2.5 py-1 text-xs font-medium rounded-md shadow-elevation-2 pointer-events-none whitespace-nowrap bg-surface-900 text-surface-50 dark:bg-surface-800 dark:text-surface-100 border border-theme-subtle {positionClasses[position]}"
    >
      {content}
    </div>
  {/if}
</span>
