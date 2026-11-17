<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    maxWidth?: 'sm' | 'md' | 'lg' | 'xl' | '2xl' | 'full';
    padding?: boolean;
    class?: string;
    children?: Snippet;
  }

  let {
    maxWidth = 'xl',
    padding = true,
    class: className = '',
    children
  }: Props = $props();

  const maxWidthClasses: Record<string, string> = {
    sm: 'max-w-screen-sm',
    md: 'max-w-screen-md',
    lg: 'max-w-screen-lg',
    xl: 'max-w-screen-xl',
    '2xl': 'max-w-screen-2xl',
    full: 'max-w-full'
  };

  let computedClasses = $derived(
    `w-full mx-auto ${maxWidthClasses[maxWidth]} ${padding ? 'px-4 sm:px-6 lg:px-8' : ''} ${className}`.trim()
  );
</script>

<div class={computedClasses}>
  {@render children?.()}
</div>
