//! # Robots.txt Sitemap Discovery
//!
//! Parses `robots.txt` documents conforming to RFC 9309 and web conventions, extracting
//! all declared `Sitemap: <url>` directives while stripping comments and normalizing URLs.

use url::Url;

/// Parses a `robots.txt` text document, extracting all valid sitemap URLs declared via `Sitemap:` directives.
#[must_use]
pub fn parse_robots_txt(robots_content: &str, base_url: &Url) -> Vec<Url> {
    let mut sitemaps = Vec::new();

    for line in robots_content.lines() {
        // Strip comments starting with '#'
        let clean_line = if let Some(hash_pos) = line.find('#') {
            &line[..hash_pos]
        } else {
            line
        };

        let trimmed = clean_line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Directives are case-insensitive key:value pairs
        if let Some(colon_pos) = trimmed.find(':') {
            let key = trimmed[..colon_pos].trim().to_ascii_lowercase();
            let value = trimmed[colon_pos + 1..].trim();

            if key == "sitemap" && !value.is_empty() {
                // Parse or resolve against base URL
                let parsed_url = if value.starts_with("http://") || value.starts_with("https://") {
                    Url::parse(value).ok()
                } else {
                    base_url.join(value).ok()
                };

                if let Some(url) = parsed_url {
                    if (url.scheme() == "http" || url.scheme() == "https")
                        && !sitemaps.contains(&url)
                    {
                        sitemaps.push(url);
                    }
                }
            }
        }
    }

    sitemaps
}
