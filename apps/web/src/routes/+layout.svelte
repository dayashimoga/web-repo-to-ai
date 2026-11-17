<script lang="ts">
  import '../lib/styles/app.css';

  let { children } = $props();
</script>

<div class="min-h-screen bg-theme-canvas text-theme-primary flex flex-col font-sans transition-colors duration-150">
  {@render children?.()}
</div>
