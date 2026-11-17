<script lang="ts">
  import { Badge } from '$lib/components/ui';

  interface Props {
    class?: string;
  }

  let { class: className = '' }: Props = $props();

  const features = [
    {
      title: 'Zero-Trust SSRF Defense in Depth',
      category: 'Security',
      badge: 'Hardened',
      badgeVariant: 'brand' as const,
      description:
        'Guards against internal network scans, loopback accesses (127.0.0.0/8, ::1), RFC 1918 subnets, and cloud metadata endpoints (169.254.169.254). DNS resolution is strictly re-validated on every single HTTP redirect.'
    },
    {
      title: 'AST-Aware Code Intelligence',
      category: 'Code Intelligence',
      badge: 'Tree-sitter',
      badgeVariant: 'info' as const,
      description:
        'Parses source code into abstract syntax trees to extract function signatures, type declarations, and module dependencies while stripping comments and formatting overhead when context budgets are constrained.'
    },
    {
      title: 'Zero Mandatory Paid AI APIs',
      category: 'Cost & Freedom',
      badge: 'Open Source',
      badgeVariant: 'success' as const,
      description:
        'Entirely independent of commercial LLM APIs. Deduplication, normalization, and token allocation run on fast, deterministic CPU heuristics, saving substantial recurring API extraction costs.'
    },
    {
      title: 'Strict <= 30 GB Podman Ceiling',
      category: 'DevOps & Reliability',
      badge: 'Enforced',
      badgeVariant: 'warning' as const,
      description:
        'Engineered from day one for resource-constrained environments. All containers, ephemeral volumes, and build caches operate within a strictly enforced 30 GB local disk storage budget.'
    },
    {
      title: 'Deterministic Token Budget Allocation',
      category: 'Context Optimization',
      badge: 'Accurate',
      badgeVariant: 'brand' as const,
      description:
        'Pre-flight token estimation calibrated to OpenAI tiktoken, Claude BPE, and Gemini tokenizer models. Prioritizes primary content over boilerplates to ensure zero context window overflow.'
    },
    {
      title: 'Privacy-First & Self-Hostable',
      category: 'Data Governance',
      badge: 'Zero Retention',
      badgeVariant: 'success' as const,
      description:
        'Zero third-party tracking, zero external telemetry pings, and ephemeral job processing. Content is processed in-memory or transient storage and purged immediately upon job completion.'
    }
  ];
</script>

<section
  id="features"
  aria-labelledby="features-heading"
  class="py-16 md:py-24 transition-colors duration-150 {className}"
>
  <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
    <!-- Section Header -->
    <div class="text-center max-w-3xl mx-auto mb-16">
      <Badge variant="brand" size="sm" class="mb-3">Architectural Highlights</Badge>
      <h2
        id="features-heading"
        class="text-3xl sm:text-4xl font-extrabold tracking-tight text-theme-primary"
      >
        Engineered for Reliability, Privacy &amp; Performance
      </h2>
      <p class="mt-4 text-base sm:text-lg text-theme-secondary leading-relaxed">
        Built with Rust and modern web standards for enterprise-grade security and zero-overhead execution.
      </p>
    </div>

    <!-- 6-Item Responsive Grid -->
    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-8">
      {#each features as feat}
        <div
          class="flex flex-col p-6 rounded-2xl bg-theme-panel border border-theme-subtle shadow-elevation-1 hover:shadow-elevation-2 hover:border-brand-500/40 transition-all duration-200"
        >
          <div class="flex items-center justify-between mb-4">
            <span class="text-xs font-semibold uppercase tracking-wider text-theme-muted">
              {feat.category}
            </span>
            <Badge variant={feat.badgeVariant} size="sm">
              {feat.badge}
            </Badge>
          </div>

          <h3 class="text-lg font-bold text-theme-primary mb-2">
            {feat.title}
          </h3>

          <p class="text-sm text-theme-secondary leading-relaxed flex-1">
            {feat.description}
          </p>
        </div>
      {/each}
    </div>
  </div>
</section>
