<script lang="ts">
  import { Button, Badge } from '$lib/components/ui';

  interface Props {
    class?: string;
  }

  let { class: className = '' }: Props = $props();

  const formats = [
    { ext: '.md', label: 'Markdown', desc: 'LLM Prompt Context' },
    { ext: '.json', label: 'JSON CDM', desc: 'Canonical AST & Meta' },
    { ext: '.xml', label: 'Claude XML', desc: 'Structured Documents' },
    { ext: '.txt', label: 'Plaintext', desc: 'Raw Normalized Text' },
    { ext: '.jsonl', label: 'JSONL', desc: 'Fine-tuning & RAG' },
    { ext: '.zip', label: 'Archive ZIP', desc: 'Deterministic Bundle' }
  ];
</script>

<section
  aria-labelledby="hero-title"
  class="relative overflow-hidden pt-12 pb-16 md:pt-20 md:pb-24 border-b border-theme-subtle transition-colors duration-150 {className}"
>
  <!-- Background radial subtle ambient glow -->
  <div
    class="pointer-events-none absolute -top-40 left-1/2 -translate-x-1/2 w-[800px] h-[400px] bg-gradient-to-b from-brand-500/10 via-brand-500/5 to-transparent blur-3xl opacity-50 dark:opacity-40"
    aria-hidden="true"
  ></div>

  <div class="relative max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 text-center">
    <!-- Top Pill Badges -->
    <div class="inline-flex flex-wrap items-center justify-center gap-2 mb-6">
      <Badge variant="brand" dot>Self-Hostable Rust Core</Badge>
      <Badge variant="success" dot>Zero Cloud Dependencies</Badge>
      <Badge variant="info" dot>Open Source (MIT)</Badge>
    </div>

    <!-- Main Headline -->
    <h1
      id="hero-title"
      class="text-4xl sm:text-5xl lg:text-6xl font-extrabold tracking-tight text-theme-primary max-w-4xl mx-auto leading-tight"
    >
      Universal URL &amp; GitHub Repository
      <span class="block bg-gradient-to-r from-brand-600 via-brand-500 to-accent-500 bg-clip-text text-transparent">
        → AI-Ready Content Converter
      </span>
    </h1>

    <!-- Subtitle / Value Proposition -->
    <p class="mt-6 text-lg sm:text-xl text-theme-secondary max-w-3xl mx-auto leading-relaxed">
      Transform complex web documentation, dynamic web applications, and multi-language GitHub repositories into
      canonical, deduplicated, token-budgeted context for Gemini, Claude, OpenAI, and local LLMs.
    </p>

    <!-- Call to Action Buttons -->
    <div class="mt-8 flex flex-wrap items-center justify-center gap-4">
      <Button variant="primary" size="lg" href="#convert">
        Start Conversion
      </Button>
      <Button variant="outline" size="lg" href="#workflow">
        Explore How It Works
      </Button>
    </div>

    <!-- Output Format Pills -->
    <div class="mt-14 pt-8 border-t border-theme-subtle/80 max-w-4xl mx-auto">
      <p class="text-xs font-semibold uppercase tracking-wider text-theme-muted mb-4">
        Multi-Format Deterministic Exporters
      </p>
      <div class="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-6 gap-3">
        {#each formats as fmt}
          <div
            class="flex flex-col items-center p-3 rounded-xl bg-theme-panel border border-theme-subtle shadow-elevation-1 hover:border-brand-500/50 transition-colors"
          >
            <span class="text-xs font-mono font-bold text-brand-500 dark:text-brand-400">{fmt.ext}</span>
            <span class="text-sm font-semibold text-theme-primary mt-1">{fmt.label}</span>
            <span class="text-xs text-theme-muted mt-0.5 text-center">{fmt.desc}</span>
          </div>
        {/each}
      </div>
    </div>

    <!-- Metric Badges Bar -->
    <div class="mt-12 grid grid-cols-1 sm:grid-cols-3 gap-6 max-w-3xl mx-auto text-center">
      <div class="p-4 rounded-xl bg-theme-panel/60 border border-theme-subtle">
        <div class="text-2xl font-bold text-theme-primary">CPU-Only 16 GB</div>
        <div class="text-xs text-theme-secondary mt-1">Zero GPU requirements, self-hostable anywhere</div>
      </div>
      <div class="p-4 rounded-xl bg-theme-panel/60 border border-theme-subtle">
        <div class="text-2xl font-bold text-theme-primary">&le; 30 GB Podman</div>
        <div class="text-xs text-theme-secondary mt-1">Strict bounded storage ceiling enforced by CI</div>
      </div>
      <div class="p-4 rounded-xl bg-theme-panel/60 border border-theme-subtle">
        <div class="text-2xl font-bold text-theme-primary">$0 / Zero API Keys</div>
        <div class="text-xs text-theme-secondary mt-1">100% offline-capable heuristic token optimization</div>
      </div>
    </div>
  </div>
</section>
