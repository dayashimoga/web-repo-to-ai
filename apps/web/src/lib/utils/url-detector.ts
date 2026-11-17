/**
 * URL Source Classification & Client-Side SSRF Pre-validation Engine
 *
 * Implements real-time detection for:
 * 1. Public GitHub Repositories, Branches, Subtrees, and Files.
 * 2. XML Sitemaps and Sitemap Indexes.
 * 3. Generic Web Documentation and Technical Articles.
 * 4. Client-side SSRF safety checks (private CIDRs, loopbacks, cloud metadata, forbidden protocols).
 */

export type DetectedSourceType =
  | "empty"
  | "github_repo"
  | "github_tree"
  | "github_blob"
  | "sitemap"
  | "web_documentation"
  | "disallowed_protocol"
  | "ssrf_blocked"
  | "invalid_syntax";

export interface GitHubUrlMetadata {
  owner: string;
  repo: string;
  ref?: string;
  subpath?: string;
  isBlob?: boolean;
  isTree?: boolean;
}

export interface UrlDetectionResult {
  rawInput: string;
  normalizedUrl: string | null;
  sourceType: DetectedSourceType;
  isValid: boolean;
  isSafe: boolean;
  label: string;
  badgeVariant: "brand" | "success" | "warning" | "error" | "default" | "info";
  errorMessage?: string;
  githubMetadata?: GitHubUrlMetadata;
  domain?: string;
  suggestedFormat?: "markdown" | "json" | "xml" | "txt" | "jsonl" | "zip";
}

/**
 * Checks whether an IP or hostname is private, loopback, or cloud metadata.
 */
export function isBlockedHost(hostname: string): boolean {
  const host = hostname
    .toLowerCase()
    .replace(/^\[|\]$/g, "")
    .trim();

  // Known dangerous hostnames
  if (
    host === "localhost" ||
    host === "127.0.0.1" ||
    host === "::1" ||
    host === "0.0.0.0" ||
    host === "metadata.google.internal" ||
    host === "instance-data" ||
    host.endsWith(".localhost") ||
    host.endsWith(".local") ||
    host.endsWith(".internal")
  ) {
    return true;
  }

  // IPv4 CIDR matching
  const ipv4Match = host.match(/^(\d{1,3})\.(\d{1,3})\.(\d{1,3})\.(\d{1,3})$/);
  if (ipv4Match) {
    const [, s1, s2, s3, s4] = ipv4Match;
    const o1 = Number(s1);
    const o2 = Number(s2);
    const o3 = Number(s3);
    const o4 = Number(s4);

    if (o1 > 255 || o2 > 255 || o3 > 255 || o4 > 255) {
      return true;
    }

    // 0.0.0.0/8 (Current network)
    if (o1 === 0) return true;
    // 127.0.0.0/8 (Loopback)
    if (o1 === 127) return true;
    // 10.0.0.0/8 (Private RFC 1918)
    if (o1 === 10) return true;
    // 172.16.0.0/12 (Private RFC 1918)
    if (o1 === 172 && o2 >= 16 && o2 <= 31) return true;
    // 192.168.0.0/16 (Private RFC 1918)
    if (o1 === 192 && o2 === 168) return true;
    // 169.254.0.0/16 (Link Local / Cloud Metadata 169.254.169.254)
    if (o1 === 169 && o2 === 254) return true;
    // 100.64.0.0/10 (Carrier-Grade NAT)
    if (o1 === 100 && o2 >= 64 && o2 <= 127) return true;
  }

  // IPv6 loopback, link-local, unique-local
  if (
    host === "::1" ||
    host === "::" ||
    host.startsWith("fe80:") ||
    host.startsWith("fc00:") ||
    host.startsWith("fd")
  ) {
    return true;
  }

  return false;
}

/**
 * Parses and classifies an input URL with security validation and GitHub AST metadata.
 */
export function detectUrlSource(input: string): UrlDetectionResult {
  const rawInput = input;
  const trimmed = input.trim();

  if (!trimmed) {
    return {
      rawInput,
      normalizedUrl: null,
      sourceType: "empty",
      isValid: false,
      isSafe: true,
      label: "Awaiting URL...",
      badgeVariant: "default",
    };
  }

  // Explicit forbidden protocols check
  const protocolMatch = trimmed.match(/^([a-zA-Z0-9+.-]+):/);
  if (protocolMatch) {
    const scheme = protocolMatch[1].toLowerCase();
    if (scheme !== "http" && scheme !== "https") {
      return {
        rawInput,
        normalizedUrl: null,
        sourceType: "disallowed_protocol",
        isValid: false,
        isSafe: false,
        label: `Disallowed Protocol (${scheme}:)`,
        badgeVariant: "error",
        errorMessage: `Only HTTP and HTTPS protocols are permitted. Protocol '${scheme}:' is blocked for security.`,
      };
    }
  }

  // Auto-prepend https:// if no scheme is provided for user convenience
  let urlCandidate = trimmed;
  if (!trimmed.includes("://")) {
    urlCandidate = `https://${trimmed}`;
  }

  let parsed: URL;
  try {
    parsed = new URL(urlCandidate);
  } catch {
    return {
      rawInput,
      normalizedUrl: null,
      sourceType: "invalid_syntax",
      isValid: false,
      isSafe: false,
      label: "Invalid URL Format",
      badgeVariant: "error",
      errorMessage: "Please enter a valid, well-formed URL (e.g. https://github.com/owner/repo).",
    };
  }

  // Validate hostname exists and contains a dot or valid structure
  if (!parsed.hostname || parsed.hostname.length < 1) {
    return {
      rawInput,
      normalizedUrl: null,
      sourceType: "invalid_syntax",
      isValid: false,
      isSafe: false,
      label: "Invalid Hostname",
      badgeVariant: "error",
      errorMessage: "The URL does not contain a valid hostname.",
    };
  }

  // Client-side SSRF Pre-validation
  if (isBlockedHost(parsed.hostname)) {
    return {
      rawInput,
      normalizedUrl: parsed.href,
      sourceType: "ssrf_blocked",
      isValid: false,
      isSafe: false,
      label: "SSRF Blocked (Private / Localhost)",
      badgeVariant: "error",
      domain: parsed.hostname,
      errorMessage: `Disallowed destination: '${parsed.hostname}' is a loopback, private CIDR, or cloud metadata address blocked by zero-trust SSRF policy.`,
    };
  }

  const hostname = parsed.hostname.toLowerCase();
  const pathname = parsed.pathname;

  // 1. GitHub Repository Classification
  if (hostname === "github.com" || hostname === "www.github.com") {
    const parts = pathname.split("/").filter(Boolean);

    if (parts.length === 0) {
      return {
        rawInput,
        normalizedUrl: parsed.href,
        sourceType: "invalid_syntax",
        isValid: false,
        isSafe: true,
        label: "Incomplete GitHub URL",
        badgeVariant: "warning",
        domain: parsed.hostname,
        errorMessage: "Please specify the repository owner and name (e.g. github.com/owner/repo).",
      };
    }

    if (parts.length === 1) {
      return {
        rawInput,
        normalizedUrl: parsed.href,
        sourceType: "invalid_syntax",
        isValid: false,
        isSafe: true,
        label: "GitHub User / Org Profile",
        badgeVariant: "warning",
        domain: parsed.hostname,
        errorMessage: `Path '${parts[0]}' is a profile. Please provide a specific repository URL.`,
      };
    }

    const owner = parts[0];
    const repo = parts[1].replace(/\.git$/, "");

    // Whole repository root
    if (parts.length === 2) {
      return {
        rawInput,
        normalizedUrl: `https://github.com/${owner}/${repo}`,
        sourceType: "github_repo",
        isValid: true,
        isSafe: true,
        label: "GitHub Repository",
        badgeVariant: "brand",
        domain: parsed.hostname,
        suggestedFormat: "markdown",
        githubMetadata: {
          owner,
          repo,
        },
      };
    }

    const action = parts[2].toLowerCase();

    // Branch / Directory subtree: /owner/repo/tree/:branch(/:subpath*)
    if (action === "tree" && parts.length >= 4) {
      const ref = parts[3];
      const subpath = parts.slice(4).join("/") || undefined;
      return {
        rawInput,
        normalizedUrl: parsed.href,
        sourceType: "github_tree",
        isValid: true,
        isSafe: true,
        label: subpath ? `GitHub Subtree (${subpath})` : `GitHub Branch (${ref})`,
        badgeVariant: "brand",
        domain: parsed.hostname,
        suggestedFormat: "markdown",
        githubMetadata: {
          owner,
          repo,
          ref,
          subpath,
          isTree: true,
        },
      };
    }

    // Specific file blob: /owner/repo/blob/:branch(/:subpath*)
    if (action === "blob" && parts.length >= 4) {
      const ref = parts[3];
      const subpath = parts.slice(4).join("/") || undefined;
      return {
        rawInput,
        normalizedUrl: parsed.href,
        sourceType: "github_blob",
        isValid: true,
        isSafe: true,
        label: `GitHub File (${subpath || ref})`,
        badgeVariant: "info",
        domain: parsed.hostname,
        suggestedFormat: "markdown",
        githubMetadata: {
          owner,
          repo,
          ref,
          subpath,
          isBlob: true,
        },
      };
    }

    // Fallback for other repository paths (e.g. issues, wiki, commit)
    return {
      rawInput,
      normalizedUrl: parsed.href,
      sourceType: "github_repo",
      isValid: true,
      isSafe: true,
      label: "GitHub Repository Path",
      badgeVariant: "brand",
      domain: parsed.hostname,
      suggestedFormat: "markdown",
      githubMetadata: {
        owner,
        repo,
        subpath: parts.slice(2).join("/"),
      },
    };
  }

  // 2. XML Sitemap Classification
  const lowerPath = pathname.toLowerCase();
  if (
    lowerPath.endsWith(".xml") ||
    lowerPath.includes("sitemap") ||
    lowerPath.includes("sitemap_index") ||
    lowerPath.includes("sitemap-index")
  ) {
    return {
      rawInput,
      normalizedUrl: parsed.href,
      sourceType: "sitemap",
      isValid: true,
      isSafe: true,
      label: "XML Sitemap Index",
      badgeVariant: "info",
      domain: parsed.hostname,
      suggestedFormat: "jsonl",
    };
  }

  // 3. Generic Web Documentation / Technical Article
  return {
    rawInput,
    normalizedUrl: parsed.href,
    sourceType: "web_documentation",
    isValid: true,
    isSafe: true,
    label: "Web Documentation / Article",
    badgeVariant: "success",
    domain: parsed.hostname,
    suggestedFormat: "markdown",
  };
}
