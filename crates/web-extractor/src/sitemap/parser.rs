//! # Sitemap Parser Engine
//!
//! High-performance, zero-external-dependency stream-like XML and plain-text parser supporting:
//! - Standard XML `<urlset>` with `<loc>`, `<lastmod>`, `<changefreq>`, and `<priority>`
//! - `<sitemapindex>` child sitemap references
//! - RSS 2.0 (`<item><link>`) and Atom (`<entry><link>`) feeds
//! - Gzip compressed `.xml.gz` de-compression with safety limits
//! - Plain text sitemaps (one URL per line)
//! - XML entity decoding and CDATA unwrapping
//! - Multi-format timestamp parsing (W3C Datetime, RFC 3339, RFC 2822, ISO 8601, date-only)

use std::io::Read;
use std::str::FromStr;

use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};
use flate2::read::GzDecoder;
use url::Url;

use super::model::{ChangeFrequency, ParsedSitemap, SitemapEntry, SitemapIndexEntry, SitemapType};
use crate::error::WebExtractorError;

/// Maximum allowable uncompressed sitemap size (50 MB) preventing decompression bombs.
const MAX_UNCOMPRESSED_SITEMAP_BYTES: usize = 50 * 1024 * 1024;

/// Parses raw sitemap content bytes (plain XML, gzipped XML, RSS, Atom, or text) into a [`ParsedSitemap`].
pub fn parse_sitemap(
    sitemap_url: &Url,
    raw_bytes: &[u8],
) -> Result<ParsedSitemap, WebExtractorError> {
    // 1. Decompress if Gzip magic header (0x1f 0x8b) is detected
    let uncompressed_bytes = if is_gzipped(raw_bytes) {
        decompress_gzip(raw_bytes)?
    } else {
        raw_bytes.to_vec()
    };

    let text = String::from_utf8_lossy(&uncompressed_bytes);

    // 2. Identify sitemap format
    let sitemap_type = detect_sitemap_type(&text);

    match sitemap_type {
        SitemapType::UrlSet => parse_urlset(sitemap_url, &text),
        SitemapType::SitemapIndex => parse_sitemap_index(sitemap_url, &text),
        SitemapType::RssFeed => parse_rss_feed(sitemap_url, &text),
        SitemapType::AtomFeed => parse_atom_feed(sitemap_url, &text),
        SitemapType::PlainText => parse_plain_text(sitemap_url, &text),
    }
}

/// Checks if byte slice begins with Gzip magic number (0x1f, 0x8b).
#[must_use]
pub fn is_gzipped(bytes: &[u8]) -> bool {
    bytes.len() >= 2 && bytes[0] == 0x1f && bytes[1] == 0x8b
}

/// Safely decompresses gzip payload with a strict maximum byte output ceiling.
fn decompress_gzip(compressed: &[u8]) -> Result<Vec<u8>, WebExtractorError> {
    let mut decoder = GzDecoder::new(compressed);
    let mut buffer = Vec::new();
    let mut chunk = [0u8; 8192];

    loop {
        match decoder.read(&mut chunk) {
            Ok(0) => break,
            Ok(n) => {
                if buffer.len() + n > MAX_UNCOMPRESSED_SITEMAP_BYTES {
                    return Err(WebExtractorError::DecompressionBomb {
                        ratio: (buffer.len() + n) as f64 / compressed.len().max(1) as f64,
                        max_ratio: 100.0,
                        compressed_bytes: compressed.len(),
                        decompressed_bytes: buffer.len() + n,
                    });
                }
                buffer.extend_from_slice(&chunk[..n]);
            }
            Err(e) => {
                return Err(WebExtractorError::DecompressionFailed {
                    encoding: "gzip".to_string(),
                    details: e.to_string(),
                });
            }
        }
    }

    Ok(buffer)
}

/// Heuristically detects the sitemap format from the textual content.
fn detect_sitemap_type(text: &str) -> SitemapType {
    let lower = text.to_ascii_lowercase();

    if lower.contains("<sitemapindex") || lower.contains(":sitemapindex") {
        SitemapType::SitemapIndex
    } else if lower.contains("<urlset") || lower.contains(":urlset") {
        SitemapType::UrlSet
    } else if (lower.contains("<rss") || lower.contains(":rss")) && lower.contains("<channel") {
        SitemapType::RssFeed
    } else if (lower.contains("<feed") || lower.contains(":feed")) && lower.contains("xmlns") {
        SitemapType::AtomFeed
    } else {
        SitemapType::PlainText
    }
}

/// Parses a standard `<urlset>` XML document.
fn parse_urlset(sitemap_url: &Url, text: &str) -> Result<ParsedSitemap, WebExtractorError> {
    let mut entries = Vec::new();
    let mut warnings = Vec::new();

    let tokens = tokenize_xml(text);
    let mut current_loc: Option<String> = None;
    let mut current_lastmod: Option<DateTime<Utc>> = None;
    let mut current_changefreq: Option<ChangeFrequency> = None;
    let mut current_priority: Option<f32> = None;
    let mut inside_url = false;
    let mut current_tag: Option<String> = None;

    for token in tokens {
        match token {
            XmlToken::OpenTag(tag, _attrs) => {
                let local_name = strip_namespace(&tag);
                if local_name == "url" {
                    inside_url = true;
                    current_loc = None;
                    current_lastmod = None;
                    current_changefreq = None;
                    current_priority = None;
                } else if inside_url {
                    current_tag = Some(local_name.to_string());
                }
            }
            XmlToken::CloseTag(tag) => {
                let local_name = strip_namespace(&tag);
                if local_name == "url" && inside_url {
                    if let Some(loc_str) = current_loc.take() {
                        match resolve_and_parse_url(sitemap_url, &loc_str) {
                            Ok(valid_url) => {
                                let mut entry = SitemapEntry::new(valid_url)
                                    .with_source_sitemap(sitemap_url.clone());
                                if let Some(dt) = current_lastmod {
                                    entry = entry.with_last_modified(dt);
                                }
                                if let Some(freq) = current_changefreq {
                                    entry = entry.with_change_frequency(freq);
                                }
                                if let Some(pri) = current_priority {
                                    entry = entry.with_priority(pri);
                                }
                                entries.push(entry);
                            }
                            Err(warn) => warnings.push(warn),
                        }
                    }
                    inside_url = false;
                } else if inside_url {
                    current_tag = None;
                }
            }
            XmlToken::Text(text_content) => {
                if inside_url {
                    if let Some(ref tag) = current_tag {
                        let clean_text = text_content.trim();
                        if !clean_text.is_empty() {
                            match tag.as_str() {
                                "loc" => current_loc = Some(clean_text.to_string()),
                                "lastmod" => current_lastmod = parse_datetime(clean_text),
                                "changefreq" => {
                                    current_changefreq = ChangeFrequency::from_str(clean_text).ok()
                                }
                                "priority" => {
                                    if let Ok(pri) = clean_text.parse::<f32>() {
                                        current_priority = Some(pri.clamp(0.0, 1.0));
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(ParsedSitemap {
        sitemap_url: sitemap_url.clone(),
        sitemap_type: SitemapType::UrlSet,
        entries,
        child_sitemaps: Vec::new(),
        warnings,
    })
}

/// Parses a `<sitemapindex>` XML document referencing child sitemaps.
fn parse_sitemap_index(sitemap_url: &Url, text: &str) -> Result<ParsedSitemap, WebExtractorError> {
    let mut child_sitemaps = Vec::new();
    let mut warnings = Vec::new();

    let tokens = tokenize_xml(text);
    let mut current_loc: Option<String> = None;
    let mut current_lastmod: Option<DateTime<Utc>> = None;
    let mut inside_sitemap = false;
    let mut current_tag: Option<String> = None;

    for token in tokens {
        match token {
            XmlToken::OpenTag(tag, _attrs) => {
                let local_name = strip_namespace(&tag);
                if local_name == "sitemap" {
                    inside_sitemap = true;
                    current_loc = None;
                    current_lastmod = None;
                } else if inside_sitemap {
                    current_tag = Some(local_name.to_string());
                }
            }
            XmlToken::CloseTag(tag) => {
                let local_name = strip_namespace(&tag);
                if local_name == "sitemap" && inside_sitemap {
                    if let Some(loc_str) = current_loc.take() {
                        match resolve_and_parse_url(sitemap_url, &loc_str) {
                            Ok(valid_url) => {
                                child_sitemaps.push(SitemapIndexEntry {
                                    sitemap_url: valid_url,
                                    last_modified: current_lastmod,
                                });
                            }
                            Err(warn) => warnings.push(warn),
                        }
                    }
                    inside_sitemap = false;
                } else if inside_sitemap {
                    current_tag = None;
                }
            }
            XmlToken::Text(text_content) => {
                if inside_sitemap {
                    if let Some(ref tag) = current_tag {
                        let clean_text = text_content.trim();
                        if !clean_text.is_empty() {
                            match tag.as_str() {
                                "loc" => current_loc = Some(clean_text.to_string()),
                                "lastmod" => current_lastmod = parse_datetime(clean_text),
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(ParsedSitemap {
        sitemap_url: sitemap_url.clone(),
        sitemap_type: SitemapType::SitemapIndex,
        entries: Vec::new(),
        child_sitemaps,
        warnings,
    })
}

/// Parses an RSS 2.0 XML syndication feed for URL entries.
fn parse_rss_feed(sitemap_url: &Url, text: &str) -> Result<ParsedSitemap, WebExtractorError> {
    let mut entries = Vec::new();
    let mut warnings = Vec::new();

    let tokens = tokenize_xml(text);
    let mut current_link: Option<String> = None;
    let mut current_pub_date: Option<DateTime<Utc>> = None;
    let mut inside_item = false;
    let mut current_tag: Option<String> = None;

    for token in tokens {
        match token {
            XmlToken::OpenTag(tag, _attrs) => {
                let local = strip_namespace(&tag);
                if local == "item" {
                    inside_item = true;
                    current_link = None;
                    current_pub_date = None;
                } else if inside_item {
                    current_tag = Some(local.to_string());
                }
            }
            XmlToken::CloseTag(tag) => {
                let local = strip_namespace(&tag);
                if local == "item" && inside_item {
                    if let Some(link_str) = current_link.take() {
                        match resolve_and_parse_url(sitemap_url, &link_str) {
                            Ok(valid_url) => {
                                let mut entry = SitemapEntry::new(valid_url)
                                    .with_source_sitemap(sitemap_url.clone());
                                if let Some(dt) = current_pub_date {
                                    entry = entry.with_last_modified(dt);
                                }
                                entries.push(entry);
                            }
                            Err(warn) => warnings.push(warn),
                        }
                    }
                    inside_item = false;
                } else if inside_item {
                    current_tag = None;
                }
            }
            XmlToken::Text(text_content) => {
                if inside_item {
                    if let Some(ref tag) = current_tag {
                        let clean = text_content.trim();
                        if !clean.is_empty() {
                            match tag.as_str() {
                                "link" => current_link = Some(clean.to_string()),
                                "pubdate" | "date" => current_pub_date = parse_datetime(clean),
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(ParsedSitemap {
        sitemap_url: sitemap_url.clone(),
        sitemap_type: SitemapType::RssFeed,
        entries,
        child_sitemaps: Vec::new(),
        warnings,
    })
}

/// Parses an Atom XML syndication feed for URL entries.
fn parse_atom_feed(sitemap_url: &Url, text: &str) -> Result<ParsedSitemap, WebExtractorError> {
    let mut entries = Vec::new();
    let mut warnings = Vec::new();

    let tokens = tokenize_xml(text);
    let mut current_link: Option<String> = None;
    let mut current_updated: Option<DateTime<Utc>> = None;
    let mut inside_entry = false;
    let mut current_tag: Option<String> = None;

    for token in tokens {
        match token {
            XmlToken::OpenTag(tag, attrs) => {
                let local = strip_namespace(&tag);
                if local == "entry" {
                    inside_entry = true;
                    current_link = None;
                    current_updated = None;
                } else if inside_entry {
                    if local == "link" {
                        // In Atom, <link href="..." rel="alternate"/>
                        if let Some(href) = attrs.get("href") {
                            let rel = attrs.get("rel").map(String::as_str).unwrap_or("alternate");
                            if rel == "alternate" || rel.is_empty() {
                                current_link = Some(href.clone());
                            }
                        }
                    }
                    current_tag = Some(local.to_string());
                }
            }
            XmlToken::CloseTag(tag) => {
                let local = strip_namespace(&tag);
                if local == "entry" && inside_entry {
                    if let Some(link_str) = current_link.take() {
                        match resolve_and_parse_url(sitemap_url, &link_str) {
                            Ok(valid_url) => {
                                let mut entry = SitemapEntry::new(valid_url)
                                    .with_source_sitemap(sitemap_url.clone());
                                if let Some(dt) = current_updated {
                                    entry = entry.with_last_modified(dt);
                                }
                                entries.push(entry);
                            }
                            Err(warn) => warnings.push(warn),
                        }
                    }
                    inside_entry = false;
                } else if inside_entry {
                    current_tag = None;
                }
            }
            XmlToken::Text(text_content) => {
                if inside_entry {
                    if let Some(ref tag) = current_tag {
                        let clean = text_content.trim();
                        if !clean.is_empty() {
                            match tag.as_str() {
                                "link" => {
                                    if current_link.is_none() {
                                        current_link = Some(clean.to_string());
                                    }
                                }
                                "updated" | "published" => {
                                    current_updated = parse_datetime(clean);
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(ParsedSitemap {
        sitemap_url: sitemap_url.clone(),
        sitemap_type: SitemapType::AtomFeed,
        entries,
        child_sitemaps: Vec::new(),
        warnings,
    })
}

/// Parses a plain-text sitemap listing one URL per line.
fn parse_plain_text(sitemap_url: &Url, text: &str) -> Result<ParsedSitemap, WebExtractorError> {
    let mut entries = Vec::new();
    let mut warnings = Vec::new();

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        match resolve_and_parse_url(sitemap_url, trimmed) {
            Ok(url) => {
                entries.push(SitemapEntry::new(url).with_source_sitemap(sitemap_url.clone()))
            }
            Err(warn) => warnings.push(warn),
        }
    }

    Ok(ParsedSitemap {
        sitemap_url: sitemap_url.clone(),
        sitemap_type: SitemapType::PlainText,
        entries,
        child_sitemaps: Vec::new(),
        warnings,
    })
}

/// Resolves a candidate URL string against base sitemap URL and validates HTTP/HTTPS scheme.
fn resolve_and_parse_url(base_url: &Url, raw: &str) -> Result<Url, String> {
    let trimmed = raw.trim();
    let url = if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        Url::parse(trimmed).map_err(|e| format!("Invalid URL '{trimmed}': {e}"))?
    } else {
        base_url.join(trimmed).map_err(|e| {
            format!("Could not join relative URL '{trimmed}' to base '{base_url}': {e}")
        })?
    };

    if url.scheme() != "http" && url.scheme() != "https" {
        return Err(format!("Unsupported scheme for sitemap URL: '{url}'"));
    }

    Ok(url)
}

/// Strips namespace prefixes (e.g. `sitemap:loc` -> `loc`, `image:image` -> `image`).
#[must_use]
pub fn strip_namespace(tag: &str) -> &str {
    if let Some(pos) = tag.find(':') {
        &tag[pos + 1..]
    } else {
        tag
    }
}

/// Parses multi-format sitemap datetimes (RFC 3339, RFC 2822, ISO 8601, and date-only).
#[must_use]
pub fn parse_datetime(s: &str) -> Option<DateTime<Utc>> {
    let clean = s.trim();

    // 1. Standard RFC 3339 / ISO 8601 (e.g. 2024-11-19T10:00:00Z, 2024-11-19T10:00:00+02:00)
    if let Ok(dt) = DateTime::parse_from_rfc3339(clean) {
        return Some(dt.with_timezone(&Utc));
    }

    // 2. RFC 2822 (standard in RSS feeds, e.g. "Tue, 19 Nov 2024 10:00:00 GMT")
    if let Ok(dt) = DateTime::parse_from_rfc2822(clean) {
        return Some(dt.with_timezone(&Utc));
    }

    // 3. Date only: YYYY-MM-DD
    if let Ok(date) = NaiveDate::parse_from_str(clean, "%Y-%m-%d") {
        return date.and_hms_opt(0, 0, 0).map(|dt| dt.and_utc());
    }

    // 4. ISO without timezone: YYYY-MM-DDTHH:MM:SS
    if let Ok(ndt) = NaiveDateTime::parse_from_str(clean, "%Y-%m-%dT%H:%M:%S") {
        return Some(ndt.and_utc());
    }

    // 5. Month only: YYYY-MM
    if let Ok(date) = NaiveDate::parse_from_str(&format!("{clean}-01"), "%Y-%m-%d") {
        return date.and_hms_opt(0, 0, 0).map(|dt| dt.and_utc());
    }

    // 6. Year only: YYYY
    if clean.len() == 4 {
        if let Ok(date) = NaiveDate::parse_from_str(&format!("{clean}-01-01"), "%Y-%m-%d") {
            return date.and_hms_opt(0, 0, 0).map(|dt| dt.and_utc());
        }
    }

    None
}

/// Token generated by the lightweight streaming XML tokenizer.
#[derive(Debug, PartialEq, Eq)]
enum XmlToken {
    /// Opening tag with element name and attribute map.
    OpenTag(String, std::collections::HashMap<String, String>),
    /// Closing tag with element name.
    CloseTag(String),
    /// Inner text or decoded CDATA content.
    Text(String),
}

/// Lightweight, memory-bounded XML tokenizer.
fn tokenize_xml(text: &str) -> Vec<XmlToken> {
    let mut tokens = Vec::new();
    let bytes = text.as_bytes();
    let mut i = 0;
    let len = bytes.len();

    while i < len {
        if bytes[i] == b'<' {
            // Check for XML comments: <!-- ... -->
            if text[i..].starts_with("<!--") {
                if let Some(end) = text[i..].find("-->") {
                    i += end + 3;
                    continue;
                } else {
                    break;
                }
            }

            // Check for CDATA: <![CDATA[ ... ]]>
            if text[i..].starts_with("<![CDATA[") {
                if let Some(end) = text[i..].find("]]>") {
                    let cdata = &text[i + 9..i + end];
                    tokens.push(XmlToken::Text(cdata.to_string()));
                    i += end + 3;
                    continue;
                } else {
                    break;
                }
            }

            // Check for DOCTYPE or processing instructions <?xml ... ?>
            if text[i..].starts_with("<xml")
                || text[i..].starts_with("<?")
                || text[i..].starts_with("<!")
            {
                if let Some(end) = text[i..].find('>') {
                    i += end + 1;
                    continue;
                } else {
                    break;
                }
            }

            // Find closing '>'
            if let Some(tag_end_offset) = text[i..].find('>') {
                let tag_end = i + tag_end_offset;
                let tag_slice = &text[i + 1..tag_end].trim();
                i = tag_end + 1;

                if let Some(stripped) = tag_slice.strip_prefix('/') {
                    // Closing tag: </tag>
                    let tag_name = stripped.trim().to_ascii_lowercase();
                    tokens.push(XmlToken::CloseTag(tag_name));
                } else {
                    // Opening tag, possibly self-closing: <tag attr="val"/>
                    let (clean_tag, is_self_closing) =
                        if let Some(stripped) = tag_slice.strip_suffix('/') {
                            (stripped.trim(), true)
                        } else {
                            (*tag_slice, false)
                        };

                    let (name, attrs) = parse_tag_name_and_attrs(clean_tag);
                    let name_lower = name.to_ascii_lowercase();
                    tokens.push(XmlToken::OpenTag(name_lower.clone(), attrs));
                    if is_self_closing {
                        tokens.push(XmlToken::CloseTag(name_lower));
                    }
                }
            } else {
                break;
            }
        } else {
            // Text between tags
            if let Some(next_tag) = text[i..].find('<') {
                let text_slice = &text[i..i + next_tag];
                i += next_tag;
                let decoded = decode_xml_entities(text_slice);
                if !decoded.trim().is_empty() {
                    tokens.push(XmlToken::Text(decoded));
                }
            } else {
                let text_slice = &text[i..];
                i = len;
                let decoded = decode_xml_entities(text_slice);
                if !decoded.trim().is_empty() {
                    tokens.push(XmlToken::Text(decoded));
                }
            }
        }
    }

    tokens
}

/// Parses the tag name and key-value attributes from inside `<tag-name k="v" ...>`.
fn parse_tag_name_and_attrs(tag_str: &str) -> (String, std::collections::HashMap<String, String>) {
    let mut attrs = std::collections::HashMap::new();
    let parts: Vec<&str> = tag_str.split_whitespace().collect();
    if parts.is_empty() {
        return (String::new(), attrs);
    }

    let tag_name = parts[0].to_string();
    let remainder = tag_str[parts[0].len()..].trim();

    // Parse attributes: attr="val" or attr='val'
    let mut chars = remainder.char_indices().peekable();
    while let Some((_, _)) = chars.peek() {
        // Skip whitespace
        while let Some(&(_, c)) = chars.peek() {
            if c.is_whitespace() {
                chars.next();
            } else {
                break;
            }
        }

        // Read attribute name
        let mut attr_name = String::new();
        while let Some(&(_, c)) = chars.peek() {
            if c == '=' || c.is_whitespace() {
                break;
            }
            attr_name.push(c);
            chars.next();
        }

        // Look for '='
        while let Some(&(_, c)) = chars.peek() {
            if c.is_whitespace() {
                chars.next();
            } else {
                break;
            }
        }

        if let Some(&(_, '=')) = chars.peek() {
            chars.next(); // Consume '='
        } else {
            continue;
        }

        // Look for opening quote
        while let Some(&(_, c)) = chars.peek() {
            if c.is_whitespace() {
                chars.next();
            } else {
                break;
            }
        }

        let quote_char = match chars.peek() {
            Some(&(_, '"')) => {
                chars.next();
                '"'
            }
            Some(&(_, '\'')) => {
                chars.next();
                '\''
            }
            _ => continue,
        };

        // Read value up to matching quote
        let mut attr_val = String::new();
        for (_, c) in chars.by_ref() {
            if c == quote_char {
                break;
            }
            attr_val.push(c);
        }

        if !attr_name.is_empty() {
            attrs.insert(
                attr_name.to_ascii_lowercase(),
                decode_xml_entities(&attr_val),
            );
        }
    }

    (tag_name, attrs)
}

/// Decodes standard XML and HTML character entities.
#[must_use]
pub fn decode_xml_entities(input: &str) -> String {
    if !input.contains('&') {
        return input.to_string();
    }

    let mut result = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '&' {
            let mut entity = String::new();
            let mut found_semicolon = false;

            for next_ch in chars.by_ref() {
                if next_ch == ';' {
                    found_semicolon = true;
                    break;
                }
                entity.push(next_ch);
                if entity.len() > 10 {
                    break;
                }
            }

            if found_semicolon {
                match entity.as_str() {
                    "amp" => result.push('&'),
                    "lt" => result.push('<'),
                    "gt" => result.push('>'),
                    "quot" => result.push('"'),
                    "apos" => result.push('\''),
                    // Decimal numeric entity: &#123;
                    dec if dec.starts_with('#')
                        && !dec.starts_with("#x")
                        && !dec.starts_with("#X") =>
                    {
                        if let Ok(code) = dec[1..].parse::<u32>() {
                            if let Some(ch) = char::from_u32(code) {
                                result.push(ch);
                            } else {
                                result.push_str("&#");
                                result.push_str(dec);
                                result.push(';');
                            }
                        } else {
                            result.push_str("&#");
                            result.push_str(dec);
                            result.push(';');
                        }
                    }
                    // Hexadecimal numeric entity: &#x7B;
                    hex if hex.starts_with("#x") || hex.starts_with("#X") => {
                        if let Ok(code) = u32::from_str_radix(&hex[2..], 16) {
                            if let Some(ch) = char::from_u32(code) {
                                result.push(ch);
                            } else {
                                result.push_str("&#x");
                                result.push_str(&hex[2..]);
                                result.push(';');
                            }
                        } else {
                            result.push_str("&#x");
                            result.push_str(&hex[2..]);
                            result.push(';');
                        }
                    }
                    other => {
                        result.push('&');
                        result.push_str(other);
                        result.push(';');
                    }
                }
            } else {
                result.push('&');
                result.push_str(&entity);
            }
        } else {
            result.push(c);
        }
    }

    result
}
