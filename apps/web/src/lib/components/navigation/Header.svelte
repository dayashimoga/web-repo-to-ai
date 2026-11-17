<script lang="ts">
  import { Button, Badge } from '$lib/components/ui';

  interface Props {
    isDark?: boolean;
    class?: string;
    onToggleTheme?: () => void;
  }

  let {
    isDark = $bindable(true),
    class: className = '',
    onToggleTheme
  }: Props = $props();

  let mobileMenuOpen = $state(false);

  function handleThemeToggle() {
    if (onToggleTheme) {
      onToggleTheme();
    } else {
      isDark = !isDark;
      if (typeof document !== 'undefined') {
        document.documentElement.classList.toggle('dark', isDark);
      }
    }
  }

  function toggleMobileMenu() {
    mobileMenuOpen = !mobileMenuOpen;
  }

  function closeMobileMenu() {
    mobileMenuOpen = false;
  }
</script>

<header
  class="sticky top-0 z-40 w-full border-b border-theme-subtle bg-theme-panel/95 backdrop-blur supports-[backdrop-filter]:bg-theme-panel/85 shadow-elevation-1 transition-colors duration-150 {className}"
>
  <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
    <div class="flex items-center justify-between h-16">
      <!-- Logo and App Title -->
      <div class="flex items-center gap-3">
        <a
          href="/"
          class="flex items-center gap-2.5 group focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-brand-500 rounded-lg p-1"
          aria-label="web-repo-to-ai Home"
        >
          <div
            class="h-9 w-9 rounded-xl bg-gradient-to-tr from-brand-600 to-accent-500 flex items-center justify-center text-white font-bold text-sm shadow-elevation-1 tracking-wider select-none"
            aria-hidden="true"
          >
            AI
          </div>
          <span class="font-bold text-lg tracking-tight group-hover:text-brand-500 transition-colors">
            web-repo-to-ai
          </span>
        </a>
        <Badge variant="brand" size="sm" dot>v0.1.0</Badge>
      </div>

      <!-- Desktop Navigation Links -->
      <nav aria-label="Main Navigation" class="hidden md:flex items-center gap-6">
        <a
          href="#convert"
          class="text-sm font-medium text-theme-secondary hover:text-theme-primary transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-brand-500 rounded px-1.5 py-1"
        >
          Convert
        </a>
        <a
          href="#workflow"
          class="text-sm font-medium text-theme-secondary hover:text-theme-primary transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-brand-500 rounded px-1.5 py-1"
        >
          How It Works
        </a>
        <a
          href="#sources"
          class="text-sm font-medium text-theme-secondary hover:text-theme-primary transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-brand-500 rounded px-1.5 py-1"
        >
          Sources
        </a>
        <a
          href="#features"
          class="text-sm font-medium text-theme-secondary hover:text-theme-primary transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-brand-500 rounded px-1.5 py-1"
        >
          Features
        </a>
        <a
          href="https://github.com/dayashimoga/web-repo-to-ai#readme"
          target="_blank"
          rel="noopener noreferrer"
          class="text-sm font-medium text-theme-secondary hover:text-theme-primary transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-brand-500 rounded px-1.5 py-1 inline-flex items-center gap-1"
        >
          Docs
          <span class="text-xs" aria-hidden="true">↗</span>
        </a>
      </nav>

      <!-- Desktop Right Controls -->
      <div class="hidden md:flex items-center gap-3">
        <Button
          variant="ghost"
          size="sm"
          onclick={handleThemeToggle}
          ariaLabel={isDark ? 'Switch to light mode' : 'Switch to dark mode'}
        >
          {#if isDark}
            <span class="flex items-center gap-1.5">
              <span aria-hidden="true">☀️</span>
              <span>Light</span>
            </span>
          {:else}
            <span class="flex items-center gap-1.5">
              <span aria-hidden="true">🌙</span>
              <span>Dark</span>
            </span>
          {/if}
        </Button>

        <Button
          variant="secondary"
          size="sm"
          href="https://github.com/dayashimoga/web-repo-to-ai"
          target="_blank"
          rel="noopener noreferrer"
          ariaLabel="GitHub Repository (opens in new tab)"
        >
          <svg class="h-4 w-4 mr-1 fill-current" viewBox="0 0 24 24" aria-hidden="true">
            <path
              fill-rule="evenodd"
              clip-rule="evenodd"
              d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.53 1.032 1.53 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0112 6.844c.85.004 1.705.115 2.504.337 1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.019 10.019 0 0022 12.017C22 6.484 17.522 2 12 2z"
            />
          </svg>
          GitHub
        </Button>

        <Button variant="primary" size="sm" href="#convert">
          Start Ingestion
        </Button>
      </div>

      <!-- Mobile Hamburger Button & Theme Switcher -->
      <div class="flex items-center gap-2 md:hidden">
        <Button
          variant="ghost"
          size="sm"
          class="min-h-[44px] min-w-[44px] flex items-center justify-center"
          onclick={handleThemeToggle}
          ariaLabel={isDark ? 'Switch to light mode' : 'Switch to dark mode'}
        >
          {isDark ? '☀️' : '🌙'}
        </Button>
        <button
          type="button"
          class="p-2 min-h-[44px] min-w-[44px] flex items-center justify-center rounded-lg text-theme-secondary hover:text-theme-primary hover:bg-theme-subtle focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-brand-500"
          aria-expanded={mobileMenuOpen}
          aria-controls="mobile-navigation"
          aria-label="Toggle navigation menu"
          onclick={toggleMobileMenu}
        >
          <svg class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor" aria-hidden="true">
            {#if mobileMenuOpen}
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            {:else}
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16" />
            {/if}
          </svg>
        </button>
      </div>
    </div>
  </div>

  <!-- Mobile Dropdown Navigation -->
  {#if mobileMenuOpen}
    <div
      id="mobile-navigation"
      class="md:hidden border-t border-theme-subtle bg-theme-panel px-4 pt-2 pb-5 space-y-2 shadow-elevation-2 animate-fadeIn"
    >
      <nav aria-label="Mobile Navigation" class="flex flex-col space-y-1">
        <a
          href="#convert"
          class="px-3 py-2.5 rounded-lg text-sm font-medium text-theme-secondary hover:text-theme-primary hover:bg-theme-subtle transition-colors min-h-[44px] flex items-center"
          onclick={closeMobileMenu}
        >
          Convert
        </a>
        <a
          href="#workflow"
          class="px-3 py-2.5 rounded-lg text-sm font-medium text-theme-secondary hover:text-theme-primary hover:bg-theme-subtle transition-colors min-h-[44px] flex items-center"
          onclick={closeMobileMenu}
        >
          How It Works
        </a>
        <a
          href="#sources"
          class="px-3 py-2.5 rounded-lg text-sm font-medium text-theme-secondary hover:text-theme-primary hover:bg-theme-subtle transition-colors min-h-[44px] flex items-center"
          onclick={closeMobileMenu}
        >
          Supported Sources
        </a>
        <a
          href="#features"
          class="px-3 py-2.5 rounded-lg text-sm font-medium text-theme-secondary hover:text-theme-primary hover:bg-theme-subtle transition-colors min-h-[44px] flex items-center"
          onclick={closeMobileMenu}
        >
          Features
        </a>
        <a
          href="https://github.com/dayashimoga/web-repo-to-ai#readme"
          target="_blank"
          rel="noopener noreferrer"
          class="px-3 py-2.5 rounded-lg text-sm font-medium text-theme-secondary hover:text-theme-primary hover:bg-theme-subtle transition-colors min-h-[44px] flex items-center justify-between"
          onclick={closeMobileMenu}
        >
          Documentation
          <span class="text-xs" aria-hidden="true">↗</span>
        </a>
      </nav>
      <div class="pt-2 border-t border-theme-subtle flex flex-col gap-2">
        <Button
          variant="secondary"
          size="sm"
          fullWidth
          href="https://github.com/dayashimoga/web-repo-to-ai"
          target="_blank"
          rel="noopener noreferrer"
        >
          GitHub Repository ↗
        </Button>
        <Button variant="primary" size="sm" fullWidth href="#convert" onclick={closeMobileMenu}>
          Start Ingestion
        </Button>
      </div>
    </div>
  {/if}
</header>
