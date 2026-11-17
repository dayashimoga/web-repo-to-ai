<script lang="ts">
  interface Props {
    checked?: boolean;
    label?: string;
    description?: string;
    disabled?: boolean;
    id?: string;
    name?: string;
    class?: string;
    onchange?: (checked: boolean) => void;
  }

  let {
    checked = $bindable(false),
    label,
    description,
    disabled = false,
    id,
    name,
    class: className = '',
    onchange
  }: Props = $props();

  let autoId = $derived(id || `toggle-${Math.random().toString(36).substring(2, 9)}`);

  function toggle() {
    if (disabled) return;
    checked = !checked;
    onchange?.(checked);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (disabled) return;
    if (e.key === ' ' || e.key === 'Enter') {
      e.preventDefault();
      toggle();
    }
  }
</script>

<div class="flex items-start justify-between gap-4 {className}">
  {#if label || description}
    <div class="flex flex-col">
      {#if label}
        <label id="{autoId}-label" for={autoId} class="text-sm font-medium text-theme-primary cursor-pointer select-none">
          {label}
        </label>
      {/if}
      {#if description}
        <span id="{autoId}-desc" class="text-xs text-theme-secondary select-none">
          {description}
        </span>
      {/if}
    </div>
  {/if}

  <button
    id={autoId}
    {name}
    type="button"
    role="switch"
    aria-checked={checked}
    aria-labelledby={label ? `${autoId}-label` : undefined}
    aria-describedby={description ? `${autoId}-desc` : undefined}
    disabled={disabled}
    onclick={toggle}
    onkeydown={handleKeydown}
    class={[
      'relative inline-flex h-6 w-11 shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out',
      'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-brand-500 focus-visible:ring-offset-2 dark:focus-visible:ring-offset-surface-900',
      checked ? 'bg-brand-600' : 'bg-surface-300 dark:bg-surface-700',
      disabled ? 'opacity-50 cursor-not-allowed' : ''
    ].join(' ')}
  >
    <span
      class={[
        'pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow-elevation-1 ring-0 transition duration-200 ease-in-out',
        checked ? 'translate-x-5' : 'translate-x-0'
      ].join(' ')}
      aria-hidden="true"
    ></span>
  </button>
</div>
