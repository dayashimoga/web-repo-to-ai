<script lang="ts">
  import { Badge } from '$lib/components/ui';

  interface Props {
    class?: string;
  }

  let { class: className = '' }: Props = $props();

  const sources = [
    {
      title: 'Static Documentation & Articles',
      type: 'Web Extraction',
      badge: 'HTML5 Parser',
      badgeVariant: 'brand' as const,
      description:
        'Fast streaming extraction for documentation sites, blogs, and technical manuals. Strips headers, footers, navigation boilerplate, and ad scripts while preserving hierarchical headings and code blocks.',
      features: [
        'html5ever & scraper streaming parser',
        'Readability-based content isolation',
        'Semantic table and code block preservation',
        'Strict zero-trust SSRF destination checks'
      ]
    },
    {
      title: 'Dynamic JavaScript Single Page Apps',
      type: 'Dynamic Ingestion',
      badge: 'Chromium Worker',
      badgeVariant: 'info' as const,
      description:
        'Isolated headless Chromium browser fallback for client-rendered React, Vue, Angular, or Svelte applications with JavaScript-gated navigation and DOM hydration.',
      features: [
        'Headless Chromium container isolation',
        'Network-idle and DOM selector triggers',
        'Automated cookie / consent banner bypass',
        'Memory-bounded isolated sandboxing'
      ]
    },
    {
      title: 'Public GitHub Repositories',
      type: 'Code Ingestion',
      badge: 'Git Trees API',
      badgeVariant: 'success' as const,
      description:
        'Deep repository ingestion supporting full branch trees, recursive subdirectory traversal, commit subtrees, and binary filtering with zero local git clone overhead.',
      features: [
        'Git Trees API recursive tree traversal',
        'Commit subtree recovery for monorepos',
        'Automatic license and README inspection',
        'Binary asset and artifact exclusion'
      ]
    },
    {
      title: 'Private Repositories via GitHub Apps',
      type: 'Enterprise Auth',
      badge: 'Ephemeral Auth',
      badgeVariant: 'warning' as const,
      description:
        'Secure, least-privilege repository access using GitHub App installations. Authenticates with short-lived installation tokens with zero persistent credential storage.',
      features: [
        'Short-lived GitHub App installation JWTs',
        'Zero disk persistence for OAuth secrets',
        'Strict repository-scoped access control',
        'Audit-logged token lifecycles'
      ]
    }
  ];
</script>

<section
  id="sources"
  aria-labelledby="sources-heading"
  class="py-16 md:py-24 border-b border-theme-subtle transition-colors duration-150 {className}"
>
  <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
    <!-- Section Header -->
    <div class="text-center max-w-3xl mx-auto mb-16">
      <Badge variant="brand" size="sm" class="mb-3">Universal Sources</Badge>
      <h2
        id="sources-heading"
        class="text-3xl sm:text-4xl font-extrabold tracking-tight text-theme-primary"
      >
        Supported Input Sources
      </h2>
      <p class="mt-4 text-base sm:text-lg text-theme-secondary leading-relaxed">
        Ingest any technical content—from a single API guide to multi-gigabyte open-source monorepos.
      </p>
    </div>

    <!-- 4-Card Responsive Grid -->
    <div class="grid grid-cols-1 md:grid-cols-2 gap-8">
      {#each sources as source}
        <div
          class="flex flex-col p-6 lg:p-8 rounded-2xl bg-theme-panel border border-theme-subtle shadow-elevation-1 hover:border-brand-500/40 transition-all duration-200"
        >
          <div class="flex items-center justify-between mb-4">
            <span class="text-xs font-semibold uppercase tracking-wider text-theme-muted">
              {source.type}
            </span>
            <Badge variant={source.badgeVariant} size="sm">
              {source.badge}
            </Badge>
          </div>

          <h3 class="text-xl font-bold text-theme-primary mb-3">
            {source.title}
          </h3>

          <p class="text-sm text-theme-secondary leading-relaxed mb-6 flex-1">
            {source.description}
          </p>

          <div class="pt-4 border-t border-theme-subtle">
            <h4 class="text-xs font-semibold uppercase tracking-wider text-theme-muted mb-2">
              Capabilities
            </h4>
            <ul class="space-y-1.5 text-xs text-theme-secondary">
              {#each source.features as feat}
                <li class="flex items-start gap-2">
                  <span class="text-brand-500 font-bold" aria-hidden="true">&bull;</span>
                  <span>{feat}</span>
                </li>
              {/each}
            </ul>
          </div>
        </div>
      {/each}
    </div>
  </div>
</section>
