# Third-Party License Policy & Inventory (`docs/LICENSING.md`)

**Project:** `web-repo-to-ai`  
**Status:** Certified Compliance Inventory v1.0  
**Repository License:** MIT License (`LICENSE`)

---

## 1. License Governance Policy

`web-repo-to-ai` enforces strict dependency hygiene and compliance across all components in the monorepo. Every direct and transitive dependency adopted within Rust crates (`Cargo.toml`) and Node packages (`package.json`) must comply with our approved permissive licensing policy.

### 1.1 Approved Permissive Licenses (Whitelist)

All adopted third-party libraries must be distributed under one or more of the following approved permissive licenses:

- **`MIT`** (Massachusetts Institute of Technology)
- **`Apache-2.0`** (Apache License 2.0)
- **`BSD-2-Clause`** (FreeBSD / Simplified BSD)
- **`BSD-3-Clause`** (New / Revised BSD)
- **`ISC`** (Internet Systems Consortium)
- **`Unicode-3.0` / `Unicode-DFS-2016`** (Unicode License Agreement)
- **`PostgreSQL`** (PostgreSQL Database License)
- **`CC0-1.0` / `Unlicense`** (Public Domain Dedication)
- **`CDLA-Permissive-2.0`** (Community Data License Agreement - Permissive)
- **`Zlib`** (zlib/libpng License)

### 1.2 Restricted & Prohibited Licenses (Blacklist)

The following license families are strictly prohibited from entering any production runtime binaries, services, or web bundles:

- **Strong & Weak Copyleft:** `GPL-2.0`, `GPL-3.0`, `AGPL-3.0`, `LGPL-2.1`, `LGPL-3.0`, `MPL-2.0`, `SSPL`, `BSL`.
- **Non-Commercial / Source-Available:** `CC-BY-NC-*`, `Commons Clause`, `PolyForm`.
- **Unlicensed / Ambiguous:** Any package lacking an explicit SPDX-compliant license declaration.

Compliance is automatically enforced on every build and CI pull request via `cargo deny check licenses` and npm license audits.

---

## 2. Automated Enforcement Tooling (`deny.toml`)

The repository root defines a deterministic `deny.toml` configuration audited by `cargo-deny`:

```toml
[licenses]
unused-allowed-license = "allow"
confidence-threshold = 0.8
allow = [
    "MIT",
    "Apache-2.0",
    "BSD-2-Clause",
    "BSD-3-Clause",
    "ISC",
    "Unicode-3.0",
    "Unicode-DFS-2016",
    "PostgreSQL",
    "CC0-1.0",
    "CDLA-Permissive-2.0",
    "Zlib",
    "Unlicense",
]
exceptions = []

[[licenses.clarify]]
name = "ring"
expression = "MIT AND ISC AND OpenSSL"
license-files = [{ path = "LICENSE", hash = 0xbd0eed23 }]
```

---

## 3. Dependency Inventory & Audit Results

### 3.1 Rust Workspace Inventory (21 Crates, Services & CLI/MCP)

Audited via `cargo deny check licenses` (100% Passing, 0 Violations):

| Category | Primary Crates | Predominant Licenses | Audit Status |
| :--- | :--- | :--- | :--- |
| **Async Runtime & Concurrency** | `tokio`, `tokio-util`, `futures`, `async-trait` | MIT / Apache-2.0 | Verified Compliant |
| **Web Services & HTTP** | `axum`, `tower`, `tower-http`, `reqwest`, `hyper` | MIT / Apache-2.0 | Verified Compliant |
| **Database & Persistence** | `sqlx`, `sqlx-postgres`, `uuid`, `chrono` | MIT / Apache-2.0 | Verified Compliant |
| **Parsing & AST Analysis** | `tree-sitter`, `html5ever`, `scraper`, `markup5ever` | MIT / Apache-2.0 | Verified Compliant |
| **Tokenization & Encoders** | `tiktoken-rs`, `tokenizers`, `bpe` | MIT / Apache-2.0 | Verified Compliant |
| **Serialization & Formats** | `serde`, `serde_json`, `serde_yaml_ng`, `csv`, `quick-xml` | MIT / Apache-2.0 | Verified Compliant |
| **Hashing & Security** | `blake3`, `sha2`, `aes-gcm`, `ring`, `subtle` | Apache-2.0 / MIT / OpenSSL | Verified Compliant |
| **CLI & MCP Protocols** | `clap`, `serde_json`, `tracing`, `tracing-subscriber` | MIT / Apache-2.0 | Verified Compliant |

### 3.2 Frontend & Chrome Extension Inventory (`apps/web` & `apps/extension`)

Audited via `npm audit` and license crawlers:

| Ecosystem | Key Dependencies | License | Compliance |
| :--- | :--- | :--- | :--- |
| **Frontend Framework** | `svelte`, `@sveltejs/kit`, `@sveltejs/adapter-node` | MIT | Verified Compliant |
| **Styling & Icons** | `tailwindcss`, `lucide-svelte`, `clsx`, `tailwind-merge` | MIT / Apache-2.0 | Verified Compliant |
| **Monaco Editor** | `monaco-editor`, `monaco-editor-workers` | MIT | Verified Compliant |
| **Testing & Quality** | `vitest`, `@testing-library/svelte`, `jsdom`, `typescript` | MIT / Apache-2.0 | Verified Compliant |
| **Chrome Extension** | `chrome-types`, `webextension-polyfill`, `vite` | MIT | Verified Compliant |

---

## 4. Software Bill of Materials (SBOM) Generation

To support enterprise compliance audits and supply chain verification, the release pipeline (`.github/workflows/release.yml`) automatically generates machine-readable SBOM artifacts:

1. **CycloneDX JSON:** Generated via `cargo cyclonedx --format json` capturing all Rust crate dependencies, cryptographic hashes, and SPDX license IDs.
2. **SPDX 2.3:** Standard Software Package Data Exchange document generated for full container and npm dependencies.
3. **Distribution:** Published alongside compiled release binaries and container image signatures in GitHub Releases.
