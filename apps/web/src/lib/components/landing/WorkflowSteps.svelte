<script lang="ts">
  import { Badge } from '$lib/components/ui';

  interface Props {
    class?: string;
  }

  let { class: className = '' }: Props = $props();

  const steps = [
    {
      step: '01',
      title: 'Acquisition & SSRF Defense',
      badge: 'Zero-Trust Ingestion',
      badgeVariant: 'brand' as const,
      description:
        'Fetches static HTML via streaming parsers or invokes isolated Playwright Chromium workers for dynamic SPAs. Enforces strict zero-trust SSRF IP validation on every redirect hop.',
      details: [
        'Private CIDR, loopback & cloud metadata blocking',
        'Streaming HTML parser with dynamic browser fallback',
        'Git Trees API deep repository subtree traversal'
      ]
    },
    {
      step: '02',
      title: 'CDM Canonicalization & Optimization',
      badge: 'Context Intelligence',
      badgeVariant: 'info' as const,
      description:
        'Transforms heterogeneous inputs into the Canonical Context Document Model (CDM). Runs structural normalization, SHA-256 exact & MinHash semantic deduplication, and AST code intelligence.',
      details: [
        'Tree-sitter AST symbol & declaration parsing',
        'MinHash LSH near-duplicate elimination',
        'Whitespace and boilerplate normalization'
      ]
    },
    {
      step: '03',
      title: 'Multi-Format Export & Token Fit',
      badge: 'Guaranteed Context',
      badgeVariant: 'success' as const,
      description:
        'Packages normalized documentation into target LLM formats with deterministic token budget verification. Export as Markdown, Claude XML, JSON, or ZIP archives with zero data retention.',
      details: [
        'Pre-flight tiktoken, BPE, and Gemini token budgeting',
        'Dynamic context window threshold indicators',
        'Ephemeral job execution with zero disk retention'
      ]
    }
  ];
</script>

<section
  id="workflow"
  aria-labelledby="workflow-heading"
  class="py-16 md:py-24 border-b border-theme-subtle transition-colors duration-150 {className}"
>
  <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
    <!-- Section Header -->
    <div class="text-center max-w-3xl mx-auto mb-16">
      <Badge variant="brand" size="sm" class="mb-3">Deterministic Pipeline</Badge>
      <h2
        id="workflow-heading"
        class="text-3xl sm:text-4xl font-extrabold tracking-tight text-theme-primary"
      >
        How It Works: 3-Step Canonical Pipeline
      </h2>
      <p class="mt-4 text-base sm:text-lg text-theme-secondary leading-relaxed">
        From raw network bytes to verified, model-tailored LLM context in three deterministic, verifiable stages.
      </p>
    </div>

    <!-- 3-Column Card Layout -->
    <div class="grid grid-cols-1 md:grid-cols-3 gap-8">
      {#each steps as item}
        <div
          class="relative flex flex-col p-6 lg:p-8 rounded-2xl bg-theme-panel border border-theme-subtle shadow-elevation-1 hover:shadow-elevation-2 hover:border-brand-500/40 transition-all duration-200"
        >
          <!-- Step Number & Badge -->
          <div class="flex items-center justify-between mb-6">
            <span class="text-3xl font-mono font-extrabold text-brand-500/80 dark:text-brand-400/80">
              {item.step}
            </span>
            <Badge variant={item.badgeVariant} size="sm">
              {item.badge}
            </Badge>
          </div>

          <!-- Step Title -->
          <h3 class="text-xl font-bold text-theme-primary mb-3">
            {item.title}
          </h3>

          <!-- Step Description -->
          <p class="text-sm text-theme-secondary leading-relaxed mb-6 flex-1">
            {item.description}
          </p>

          <!-- Key Details List -->
          <ul class="space-y-2 pt-4 border-t border-theme-subtle text-xs text-theme-secondary font-medium">
            {#each item.details as detail}
              <li class="flex items-start gap-2">
                <span class="text-status-success font-bold mt-0.5" aria-hidden="true">✓</span>
                <span>{detail}</span>
              </li>
            {/each}
          </ul>
        </div>
      {/each}
    </div>
  </div>
</section>
