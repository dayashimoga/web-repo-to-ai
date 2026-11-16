# User Guide & Onboarding (`docs/USER_GUIDE.md`)

**Project:** `web-repo-to-ai`

---

## 1. Converting a Webpage

1. Paste any public `http://` or `https://` URL into the main input bar on the landing page.
2. The platform automatically detects a **Webpage** source.
3. Choose an **AI Context Profile** (default: `General AI` or `Lossless`) and your target tokenizer/model budget.
4. Enable **Dynamic Browser Rendering** only if the target page requires client-side JavaScript execution (auto-fallback is enabled by default).
5. Click **Convert** to watch real-time progress, preview the extracted Markdown/sections, inspect token counts, and copy or download the output package.

---

## 2. Converting a Public GitHub Repository

1. Paste a GitHub repository URL (e.g., `https://github.com/owner/repo`, or a specific `/tree/branch/subdir` or `/blob/main/file.rs`).
2. Select the target **Branch**, **Tag**, or **Commit SHA**, and optionally refine the interactive file/folder filter.
3. Select the desired **Context Profile** (`General AI`, `Code Review`, `Security Review`, `Debugging`, `Documentation`, or `RAG`).
4. Review the **Completeness Manifest** showing every discovered, extracted, and excluded file along with exact exclusion reasons.
5. Copy the AI-ready Markdown directly or download a `.zip` bundle containing the formatted context, `manifest.json`, `provenance.json`, `quality-report.json`, and `checksums.sha256`.

---

## 3. Converting a Private GitHub Repository

1. Click **Connect GitHub** to authorize the `web-repo-to-ai` GitHub App on selected repositories with strictly **Read-Only** (`Contents: Read-only`, `Metadata: Read-only`) permissions.
2. Pick an authorized private repository from the selector or paste its URL.
3. Convert and download with the same completeness guarantees—your credentials are encrypted server-side, never written into logs or exports, and can be revoked at any time.
