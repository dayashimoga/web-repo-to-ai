<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    open?: boolean;
    title?: string;
    description?: string;
    maxWidth?: 'sm' | 'md' | 'lg' | 'xl';
    onclose?: () => void;
    children?: Snippet;
    footer?: Snippet;
  }

  let {
    open = $bindable(false),
    title,
    description,
    maxWidth = 'md',
    onclose,
    children,
    footer
  }: Props = $props();

  const maxWidthClasses: Record<string, string> = {
    sm: 'max-w-sm',
    md: 'max-w-md',
    lg: 'max-w-lg',
    xl: 'max-w-xl'
  };

  const modalId = `modal-${Math.random().toString(36).substring(2, 9)}`;

  function close() {
    open = false;
    onclose?.();
  }

  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget) {
      close();
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (open && e.key === 'Escape') {
      e.stopPropagation();
      close();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if open}
  <!-- Backdrop Overlay -->
  <div
    class="fixed inset-0 z-50 bg-black/60 backdrop-blur-sm flex items-center justify-center p-4 overflow-y-auto animate-in fade-in duration-150"
    onclick={handleBackdropClick}
    role="presentation"
  >
    <!-- Modal Dialog Content -->
    <div
      role="dialog"
      aria-modal="true"
      aria-labelledby={title ? `${modalId}-title` : undefined}
      aria-describedby={description ? `${modalId}-desc` : undefined}
      class="w-full {maxWidthClasses[maxWidth]} bg-theme-panel rounded-2xl border border-theme-subtle shadow-elevation-4 overflow-hidden transform transition-all duration-150 scale-100"
    >
      <!-- Header -->
      <div class="px-6 py-4 border-b border-theme-subtle flex items-center justify-between">
        <div>
          {#if title}
            <h2 id="{modalId}-title" class="text-lg font-semibold text-theme-primary">
              {title}
            </h2>
          {/if}
          {#if description}
            <p id="{modalId}-desc" class="text-xs text-theme-secondary mt-0.5">
              {description}
            </p>
          {/if}
        </div>
        <button
          type="button"
          onclick={close}
          class="p-1 rounded-lg text-theme-muted hover:text-theme-primary hover:bg-theme-subtle focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-brand-500"
          aria-label="Close dialog"
        >
          <svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <!-- Body -->
      <div class="p-6 text-sm text-theme-secondary">
        {@render children?.()}
      </div>

      <!-- Footer Actions -->
      {#if footer}
        <div class="px-6 py-3.5 bg-theme-subtle/50 border-t border-theme-subtle flex items-center justify-end gap-3">
          {@render footer()}
        </div>
      {/if}
    </div>
  </div>
{/if}
