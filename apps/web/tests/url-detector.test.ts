import { describe, it, expect } from "vitest";
import {
  detectUrlSource,
  isBlockedHost,
  type UrlDetectionResult,
} from "../src/lib/utils/url-detector";

describe("SSRF Host Blocklist Detection (isBlockedHost)", () => {
  it("identifies localhost and loopback IPv4 addresses as blocked", () => {
    expect(isBlockedHost("localhost")).toBe(true);
    expect(isBlockedHost("LOCALHOST")).toBe(true);
    expect(isBlockedHost("127.0.0.1")).toBe(true);
    expect(isBlockedHost("127.0.1.1")).toBe(true);
    expect(isBlockedHost("127.255.255.254")).toBe(true);
    expect(isBlockedHost("app.localhost")).toBe(true);
  });

  it("identifies IPv6 loopback and internal addresses as blocked", () => {
    expect(isBlockedHost("::1")).toBe(true);
    expect(isBlockedHost("[::1]")).toBe(true);
    expect(isBlockedHost("::")).toBe(true);
    expect(isBlockedHost("fe80::1")).toBe(true);
    expect(isBlockedHost("fc00::1")).toBe(true);
  });

  it("identifies RFC 1918 private subnets as blocked", () => {
    // 10.0.0.0/8
    expect(isBlockedHost("10.0.0.1")).toBe(true);
    expect(isBlockedHost("10.254.10.5")).toBe(true);

    // 172.16.0.0/12
    expect(isBlockedHost("172.16.0.1")).toBe(true);
    expect(isBlockedHost("172.31.255.255")).toBe(true);
    expect(isBlockedHost("172.32.0.1")).toBe(false); // public

    // 192.168.0.0/16
    expect(isBlockedHost("192.168.1.1")).toBe(true);
    expect(isBlockedHost("192.168.0.254")).toBe(true);
  });

  it("identifies cloud metadata endpoints as blocked", () => {
    expect(isBlockedHost("169.254.169.254")).toBe(true);
    expect(isBlockedHost("metadata.google.internal")).toBe(true);
    expect(isBlockedHost("instance-data")).toBe(true);
  });

  it("permits valid public domain hostnames", () => {
    expect(isBlockedHost("github.com")).toBe(false);
    expect(isBlockedHost("docs.rs")).toBe(false);
    expect(isBlockedHost("developer.mozilla.org")).toBe(false);
    expect(isBlockedHost("8.8.8.8")).toBe(false);
    expect(isBlockedHost("1.1.1.1")).toBe(false);
  });
});

describe("URL Source Classification (detectUrlSource)", () => {
  it("returns empty result for blank or whitespace inputs", () => {
    const empty1 = detectUrlSource("");
    expect(empty1.sourceType).toBe("empty");
    expect(empty1.isValid).toBe(false);
    expect(empty1.isSafe).toBe(true);

    const empty2 = detectUrlSource("   ");
    expect(empty2.sourceType).toBe("empty");
  });

  it("detects and blocks disallowed non-HTTP protocols", () => {
    const file = detectUrlSource("file:///etc/passwd");
    expect(file.sourceType).toBe("disallowed_protocol");
    expect(file.isValid).toBe(false);
    expect(file.isSafe).toBe(false);
    expect(file.errorMessage).toContain("Protocol 'file:' is blocked");

    const js = detectUrlSource("javascript:alert(1)");
    expect(js.sourceType).toBe("disallowed_protocol");

    const data = detectUrlSource("data:text/html,<script>alert(1)</script>");
    expect(data.sourceType).toBe("disallowed_protocol");

    const ftp = detectUrlSource("ftp://files.example.com/archive.zip");
    expect(ftp.sourceType).toBe("disallowed_protocol");
  });

  it("detects and flags SSRF targets immediately in client pre-check", () => {
    const localhost = detectUrlSource("http://localhost:3000/api");
    expect(localhost.sourceType).toBe("ssrf_blocked");
    expect(localhost.isValid).toBe(false);
    expect(localhost.isSafe).toBe(false);
    expect(localhost.errorMessage).toContain("zero-trust SSRF policy");

    const privateIp = detectUrlSource("https://192.168.1.100/admin");
    expect(privateIp.sourceType).toBe("ssrf_blocked");

    const cloudMeta = detectUrlSource("http://169.254.169.254/latest/meta-data/");
    expect(cloudMeta.sourceType).toBe("ssrf_blocked");
  });

  it("correctly classifies standard public GitHub repository roots", () => {
    const res = detectUrlSource("https://github.com/dayashimoga/web-repo-to-ai");
    expect(res.sourceType).toBe("github_repo");
    expect(res.isValid).toBe(true);
    expect(res.isSafe).toBe(true);
    expect(res.label).toBe("GitHub Repository");
    expect(res.badgeVariant).toBe("brand");
    expect(res.githubMetadata?.owner).toBe("dayashimoga");
    expect(res.githubMetadata?.repo).toBe("web-repo-to-ai");

    // With .git suffix
    const resGit = detectUrlSource("https://github.com/rust-lang/cargo.git");
    expect(resGit.githubMetadata?.repo).toBe("cargo");

    // Without scheme (auto-prepended https://)
    const resNoScheme = detectUrlSource("github.com/torvalds/linux");
    expect(resNoScheme.sourceType).toBe("github_repo");
    expect(resNoScheme.githubMetadata?.owner).toBe("torvalds");
    expect(resNoScheme.githubMetadata?.repo).toBe("linux");
  });

  it("correctly classifies GitHub branches and directory subtrees", () => {
    const res = detectUrlSource("https://github.com/rust-lang/rust/tree/master/library/std");
    expect(res.sourceType).toBe("github_tree");
    expect(res.isValid).toBe(true);
    expect(res.githubMetadata?.owner).toBe("rust-lang");
    expect(res.githubMetadata?.repo).toBe("rust");
    expect(res.githubMetadata?.ref).toBe("master");
    expect(res.githubMetadata?.subpath).toBe("library/std");
    expect(res.githubMetadata?.isTree).toBe(true);
    expect(res.label).toContain("GitHub Subtree (library/std)");
  });

  it("correctly classifies specific GitHub file blobs", () => {
    const res = detectUrlSource("https://github.com/torvalds/linux/blob/master/Makefile");
    expect(res.sourceType).toBe("github_blob");
    expect(res.isValid).toBe(true);
    expect(res.githubMetadata?.owner).toBe("torvalds");
    expect(res.githubMetadata?.repo).toBe("linux");
    expect(res.githubMetadata?.ref).toBe("master");
    expect(res.githubMetadata?.subpath).toBe("Makefile");
    expect(res.githubMetadata?.isBlob).toBe(true);
    expect(res.badgeVariant).toBe("info");
  });

  it("flags incomplete GitHub user or org URLs with helpful feedback", () => {
    const res = detectUrlSource("https://github.com/dayashimoga");
    expect(res.sourceType).toBe("invalid_syntax");
    expect(res.isValid).toBe(false);
    expect(res.errorMessage).toContain("is a profile. Please provide a specific repository URL");
  });

  it("correctly classifies XML sitemaps and sitemap indexes", () => {
    const res1 = detectUrlSource("https://docs.rs/sitemap.xml");
    expect(res1.sourceType).toBe("sitemap");
    expect(res1.isValid).toBe(true);
    expect(res1.badgeVariant).toBe("info");
    expect(res1.suggestedFormat).toBe("jsonl");

    const res2 = detectUrlSource("https://example.com/sitemap_index.xml");
    expect(res2.sourceType).toBe("sitemap");
  });

  it("correctly classifies generic web documentation and technical articles", () => {
    const res = detectUrlSource("https://docs.rs/tokio/latest/tokio/");
    expect(res.sourceType).toBe("web_documentation");
    expect(res.isValid).toBe(true);
    expect(res.isSafe).toBe(true);
    expect(res.label).toBe("Web Documentation / Article");
    expect(res.badgeVariant).toBe("success");
    expect(res.domain).toBe("docs.rs");
    expect(res.suggestedFormat).toBe("markdown");
  });
});
