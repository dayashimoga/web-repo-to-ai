<script lang="ts">
  import { Badge } from '$lib/components/ui';

  interface Props {
    class?: string;
  }

  let { class: className = '' }: Props = $props();

  const currentYear = new Date().getFullYear();
</script>

<footer
  class="border-t border-theme-subtle bg-theme-panel mt-auto transition-colors duration-150 {className}"
>
  <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-12 lg:py-16">
    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-5 gap-8 lg:gap-12">
      <!-- Col 1: Brand & Badges (2 cols wide on large screens) -->
      <div class="lg:col-span-2 space-y-4">
        <div class="flex items-center gap-3">
          <div
            class="h-8 w-8 rounded-xl bg-gradient-to-tr from-brand-600 to-accent-500 flex items-center justify-center text-white font-bold text-sm shadow-elevation-1 select-none"
            aria-hidden="true"
          >
            AI
          </div>
          <span class="font-bold text-lg tracking-tight">web-repo-to-ai</span>
          <Badge variant="brand" size="sm">Open Source</Badge>
        </div>
        <p class="text-sm text-theme-secondary leading-relaxed max-w-sm">
          Universal URL & GitHub Repository → AI-Ready Content Converter. Transforming complex web docs and codebases
          into canonical, deduplicated, token-budgeted context for LLMs.
        </p>
        <div class="flex flex-wrap gap-2 pt-2">
          <Badge variant="default" size="sm">MIT License</Badge>
          <Badge variant="success" size="sm">CPU-Only 16 GB</Badge>
          <Badge variant="info" size="sm">Podman &lt;= 30 GB</Badge>
          <Badge variant="brand" size="sm">Zero API Costs</Badge>
        </div>
      </div>

      <!-- Col 2: Ingestion & Core -->
      <div class="space-y-3">
        <h3 class="text-xs font-semibold uppercase tracking-wider text-theme-primary">
          Ingestion Engine
        </h3>
        <ul class="space-y-2 text-sm text-theme-secondary">
          <li>
            <a href="#sources" class="hover:text-theme-primary transition-colors">Static Web Scraping</a>
          </li>
          <li>
            <a href="#sources" class="hover:text-theme-primary transition-colors">Chromium Dynamic Rendering</a>
          </li>
          <li>
            <a href="#sources" class="hover:text-theme-primary transition-colors">Git Trees API Traversal</a>
          </li>
          <li>
            <a href="#sources" class="hover:text-theme-primary transition-colors">GitHub App OAuth</a>
          </li>
        </ul>
      </div>

      <!-- Col 3: Optimization & Export -->
      <div class="space-y-3">
        <h3 class="text-xs font-semibold uppercase tracking-wider text-theme-primary">
          Context & Export
        </h3>
        <ul class="space-y-2 text-sm text-theme-secondary">
          <li>
            <a href="#workflow" class="hover:text-theme-primary transition-colors">Canonical Document Model</a>
          </li>
          <li>
            <a href="#workflow" class="hover:text-theme-primary transition-colors">MinHash Deduplication</a>
          </li>
          <li>
            <a href="#workflow" class="hover:text-theme-primary transition-colors">AST Code Intelligence</a>
          </li>
          <li>
            <a href="#workflow" class="hover:text-theme-primary transition-colors">Multi-Format Exporters</a>
          </li>
        </ul>
      </div>

      <!-- Col 4: Architecture & Specs -->
      <div class="space-y-3">
        <h3 class="text-xs font-semibold uppercase tracking-wider text-theme-primary">
          Specifications
        </h3>
        <ul class="space-y-2 text-sm text-theme-secondary">
          <li>
            <a
              href="https://github.com/dayashimoga/web-repo-to-ai/blob/main/docs/ARCHITECTURE.md"
              target="_blank"
              rel="noopener noreferrer"
              class="hover:text-theme-primary transition-colors inline-flex items-center gap-1"
            >
              Architecture
              <span class="text-xs" aria-hidden="true">↗</span>
            </a>
          </li>
          <li>
            <a
              href="https://github.com/dayashimoga/web-repo-to-ai/blob/main/docs/SECURITY.md"
              target="_blank"
              rel="noopener noreferrer"
              class="hover:text-theme-primary transition-colors inline-flex items-center gap-1"
            >
              SSRF Defense
              <span class="text-xs" aria-hidden="true">↗</span>
            </a>
          </li>
          <li>
            <a
              href="https://github.com/dayashimoga/web-repo-to-ai/blob/main/docs/DATA_MODEL.md"
              target="_blank"
              rel="noopener noreferrer"
              class="hover:text-theme-primary transition-colors inline-flex items-center gap-1"
            >
              CDM Data Model
              <span class="text-xs" aria-hidden="true">↗</span>
            </a>
          </li>
          <li>
            <a
              href="https://github.com/dayashimoga/web-repo-to-ai/blob/main/docs/PERFORMANCE.md"
              target="_blank"
              rel="noopener noreferrer"
              class="hover:text-theme-primary transition-colors inline-flex items-center gap-1"
            >
              Performance Targets
              <span class="text-xs" aria-hidden="true">↗</span>
            </a>
          </li>
        </ul>
      </div>
    </div>

    <!-- Bottom Bar -->
    <div class="mt-12 pt-8 border-t border-theme-subtle flex flex-col sm:flex-row items-center justify-between gap-4 text-xs text-theme-muted">
      <div>
        &copy; {currentYear} <span class="font-semibold text-theme-secondary">web-repo-to-ai</span>. Released under the
        <a
          href="https://github.com/dayashimoga/web-repo-to-ai/blob/main/LICENSE"
          target="_blank"
          rel="noopener noreferrer"
          class="underline hover:text-theme-primary transition-colors ml-1"
        >
          MIT License
        </a>.
      </div>
      <div class="flex items-center gap-4">
        <span>Self-Hostable</span>
        <span>•</span>
        <span>Zero Telemetry</span>
        <span>•</span>
        <span>Zero Data Retention</span>
      </div>
    </div>
  </div>
</footer>
