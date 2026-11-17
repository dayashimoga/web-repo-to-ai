# Design System Specification (`docs/DESIGN_SYSTEM.md`)

**Project:** `web-repo-to-ai`  
**Status:** Baseline Design System v1.0  
**Associated Subphase:** `1.1.2 — Typography, spacing & color systems`

---

## 1. Design Principles & Goals

1. **Information Density & Clarity:** Designed specifically for developer and AI context operations. Data tables, token budgets, file trees, and markdown previews prioritize legible scanning and dense utility over decorative whitespace.
2. **Accessible by Default (WCAG 2.1 AA/AAA):** Text and UI controls meet strict contrast requirements across dark and light palettes. Every interactive component maintains a visible `2px` focus ring with keyboard accessibility.
3. **Zero Network Dependencies:** Typography utilizes system font stacks (`Inter`, `Segoe UI`, `JetBrains Mono`, `system-ui`). No external Google Fonts, Typekit, or CDN links are loaded, eliminating external tracking and offline failure modes.
4. **Deterministic 4px Rhythm:** All margins, paddings, gaps, and component heights adhere to a strict 4px grid.

---

## 2. Typography Scale & Font Stacks

### 2.1 Font Families

| Token | Stack | Usage |
| :--- | :--- | :--- |
| `--font-sans` | `'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif` | Headings, UI labels, buttons, navigation, and body copy. |
| `--font-mono` | `'JetBrains Mono', 'Fira Code', ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace` | Token counters, code previews, file paths, and AST manifests. |

### 2.2 Type Hierarchy

| Level / Token | Size | Line Height | Weight | Tailwind Class | Semantic HTML |
| :--- | :---: | :---: | :---: | :--- | :---: |
| **Display / H1** | `3.00rem` (48px) | `1.00` | `700` (Bold) | `text-4xl sm:text-5xl font-bold tracking-tight` | `<h1>` |
| **Section / H2** | `2.25rem` (36px) | `2.50rem` (40px) | `700` (Bold) | `text-3xl font-bold tracking-tight` | `<h2>` |
| **Subsection / H3** | `1.875rem` (30px) | `2.25rem` (36px) | `600` (Semibold) | `text-2xl font-semibold tracking-tight` | `<h3>` |
| **Group / H4** | `1.50rem` (24px) | `2.00rem` (32px) | `600` (Semibold) | `text-xl font-semibold` | `<h4>` |
| **Panel / H5** | `1.25rem` (20px) | `1.75rem` (28px) | `500` (Medium) | `text-lg font-medium` | `<h5>` |
| **Caption / H6** | `1.00rem` (16px) | `1.50rem` (24px) | `500` (Medium) | `text-sm font-medium text-theme-secondary` | `<h6>` |
| **Body Large** | `1.125rem` (18px) | `1.75rem` (28px) | `400` / `500` | `text-lg leading-7` | `<p>` |
| **Body Base** | `1.00rem` (16px) | `1.50rem` (24px) | `400` / `500` | `text-base leading-6` | `<p>`, `<span>` |
| **Body Small** | `0.875rem` (14px) | `1.25rem` (20px) | `400` / `500` | `text-sm leading-5` | `<p>`, `<label>` |
| **Micro / Meta** | `0.75rem` (12px) | `1.00rem` (16px) | `400` / `500` | `text-xs leading-4` | `<span>`, `<kbd>` |

---

## 3. Spacing Grid & Layout System

### 3.1 4px Baseline Grid

All spatial dimensions map directly to the 4px baseline grid formula: $\text{Size} = \text{Step} \times 4\text{px}$.

| Step | Pixel Value | Rem Value | Tailwind Spacing | Common Usage |
| :---: | :---: | :---: | :--- | :--- |
| `0` | `0px` | `0` | `p-0`, `m-0`, `gap-0` | Reset / inline flush |
| `1` | `4px` | `0.25rem` | `p-1`, `gap-1` | Tight badge padding, micro gaps |
| `2` | `8px` | `0.50rem` | `p-2`, `gap-2` | Button padding, icon spacing |
| `3` | `12px` | `0.75rem` | `p-3`, `gap-3` | Compact card padding, form input padding |
| `4` | `16px` | `1.00rem` | `p-4`, `gap-4` | Standard card padding, grid gaps |
| `5` | `20px` | `1.25rem` | `p-5`, `gap-5` | Modal header spacing |
| `6` | `24px` | `1.50rem` | `p-6`, `gap-6` | Section spacing, container padding |
| `8` | `32px` | `2.00rem` | `p-8`, `gap-8` | Major panel divisions |
| `10` | `40px` | `2.50rem` | `p-10`, `gap-10` | Page hero margins |
| `12` | `48px` | `3.00rem` | `p-12`, `gap-12` | Main content separation |
| `16` | `64px` | `4.00rem` | `p-16`, `gap-16` | Section headers |
| `20` | `80px` | `5.00rem` | `p-20`, `gap-20` | Hero section padding |
| `24` | `96px` | `6.00rem` | `p-24`, `gap-24` | Landing page layout blocks |

### 3.2 Responsive Container Breakpoints

| Breakpoint | Minimum Width | Container Class | Max Width |
| :--- | :---: | :--- | :---: |
| `sm` | `640px` | `Container maxWidth="sm"` | `640px` |
| `md` | `768px` | `Container maxWidth="md"` | `768px` |
| `lg` | `1024px` | `Container maxWidth="lg"` | `1024px` |
| `xl` | `1280px` | `Container maxWidth="xl"` | `1280px` |
| `2xl` | `1536px` | `Container maxWidth="2xl"` | `1536px` |

---

## 4. Color Palette & Surface Contrast

### 4.1 Palette Overview

- **Brand Primary (AI Accent):** `#6366f1` (`brand-500`) with scale from `50` (`#eef2ff`) to `950` (`#1e1b4b`).
- **Telemetry Accent (Streaming):** `#06b6d4` (`accent-500`) for real-time SSE progress indicators and live status nodes.
- **Surface Neutrals (Slate):** Full 11-step scale from `#f8fafc` (`surface-50`, light canvas) to `#090d16` (`surface-950`, dark canvas).

### 4.2 Semantic Feedback Palette

| State | Hex Code | Light Surface Tint | Dark Surface Tint |
| :--- | :---: | :---: | :---: |
| **Success** | `#10b981` (Emerald) | `#d1fae5` | `#065f46` |
| **Warning** | `#f59e0b` (Amber) | `#fef3c7` | `#92400e` |
| **Error** | `#ef4444` (Rose) | `#fee2e2` | `#991b1b` |
| **Info** | `#0284c7` (Sky) | `#e0f2fe` | `#075985` |

### 4.3 AI Context Token Budget System

Token budgets represent model-specific context window consumption (e.g. 128k Claude, 1M Gemini, 32k local models):

```
Token Consumption Range:
[======================= 0% to 80% =======================] -> Safe (Emerald)
[======================== 80% to 100% ====================] -> Warning (Amber)
[======================== > 100% =========================] -> Critical (Rose)
```

| Budget Status | Usage Ratio | Hex Color | Badge Class |
| :--- | :---: | :---: | :--- |
| **Safe** | $\le 80\%$ | `#10b981` | `bg-status-success-light text-status-success-dark` |
| **Warning** | $> 80\% \text{ and } \le 100\%$ | `#f59e0b` | `bg-status-warning-light text-status-warning-dark` |
| **Critical** | $> 100\%$ | `#ef4444` | `bg-status-error-light text-status-error-dark` |

---

## 5. Accessibility & Contrast Certification (WCAG 2.1)

| Foreground Color | Background Color | Contrast Ratio | WCAG Compliance Level |
| :--- | :--- | :---: | :---: |
| Text White (`#ffffff`) | Dark Canvas (`#090d16`) | **18.9 : 1** | **Passes WCAG AAA** ($\ge 7.0:1$) |
| Text White (`#ffffff`) | Dark Panel (`#0f172a`) | **16.1 : 1** | **Passes WCAG AAA** ($\ge 7.0:1$) |
| Text Slate-900 (`#0f172a`) | Light Canvas (`#f8fafc`) | **16.9 : 1** | **Passes WCAG AAA** ($\ge 7.0:1$) |
| Brand-600 (`#4f46e5`) | Text White (`#ffffff`) | **6.3 : 1** | **Passes WCAG AA & Large AAA** ($\ge 4.5:1$) |
| Brand-500 (`#6366f1`) | Text White (`#ffffff`) | **4.5 : 1** | **Passes WCAG AA (Large / UI)** ($\ge 3.0:1$) |
| Status Success (`#065f46`) | Success Tint (`#d1fae5`) | **5.4 : 1** | **Passes WCAG AA** ($\ge 4.5:1$) |
| Status Error (`#991b1b`) | Error Tint (`#fee2e2`) | **6.1 : 1** | **Passes WCAG AA** ($\ge 4.5:1$) |

### 5.1 Universal Keyboard Focus Ring
All interactive elements require visible focus ring styles:
```css
:focus-visible {
  outline: 2px solid var(--focus-ring-color);
  outline-offset: 2px;
}
```
Utility class: `focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-brand-500 focus-visible:ring-offset-2`.
