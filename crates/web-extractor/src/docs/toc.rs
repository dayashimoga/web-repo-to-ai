//! # Table of Contents and Section Hierarchy Extraction
//!
//! Extracts heading structures (`h1`–`h6`), parses anchor IDs, strips self-referential
//! anchor symbols (`#`, `¶`, `§`), and constructs nested table-of-contents trees.

use super::model::{DocSection, TocItem};

/// Discovers headings in the document and builds a nested `TocItem` hierarchy.
#[must_use]
pub fn extract_table_of_contents(
    content_handle: tl::NodeHandle,
    parser: &tl::Parser<'_>,
) -> (Vec<TocItem>, Vec<DocSection>) {
    let mut flat_items: Vec<FlatHeading> = Vec::new();
    traverse_headings(content_handle, parser, &mut flat_items);

    let toc_tree = build_toc_tree(&flat_items);
    let sections = build_doc_sections(&flat_items);

    (toc_tree, sections)
}

struct FlatHeading {
    level: u8,
    title: String,
    anchor_id: Option<String>,
    content_text: String,
}

fn traverse_headings(
    handle: tl::NodeHandle,
    parser: &tl::Parser<'_>,
    headings: &mut Vec<FlatHeading>,
) {
    let node = match handle.get(parser) {
        Some(n) => n,
        None => return,
    };

    if let tl::Node::Tag(tag) = node {
        let name = tag.name().as_utf8_str();
        let name_lower = name.to_ascii_lowercase();

        if let Some(level) = heading_level(&name_lower) {
            let (title, anchor_id) = extract_heading_details(tag, parser);
            if !title.is_empty() {
                headings.push(FlatHeading {
                    level,
                    title,
                    anchor_id,
                    content_text: String::new(),
                });
            }
        }

        for child in tag.children().top().iter() {
            traverse_headings(*child, parser, headings);
        }
    }
}

fn heading_level(tag_lower: &str) -> Option<u8> {
    match tag_lower {
        "h1" => Some(1),
        "h2" => Some(2),
        "h3" => Some(3),
        "h4" => Some(4),
        "h5" => Some(5),
        "h6" => Some(6),
        _ => None,
    }
}

fn extract_heading_details(
    tag: &tl::HTMLTag<'_>,
    parser: &tl::Parser<'_>,
) -> (String, Option<String>) {
    let attrs = tag.attributes();

    // 1. Direct id attribute on heading e.g. <h2 id="install">
    let mut anchor_id = attrs
        .get("id")
        .and_then(|v| v)
        .map(|b| b.as_utf8_str().trim().to_string())
        .filter(|s| !s.is_empty());

    // 2. Child anchor if id was missing
    if anchor_id.is_none() {
        for child_handle in tag.children().top().iter() {
            if let Some(tl::Node::Tag(child_tag)) = child_handle.get(parser) {
                let c_name = child_tag.name().as_utf8_str();
                if c_name.eq_ignore_ascii_case("a") {
                    let c_attrs = child_tag.attributes();
                    if let Some(Some(id)) = c_attrs.get("id").or_else(|| c_attrs.get("name")) {
                        let id_str = id.as_utf8_str().trim().to_string();
                        if !id_str.is_empty() {
                            anchor_id = Some(id_str);
                            break;
                        }
                    }
                    if let Some(Some(href)) = c_attrs.get("href") {
                        let href_str = href.as_utf8_str();
                        if let Some(stripped) = href_str.strip_prefix('#') {
                            let s = stripped.trim().to_string();
                            if !s.is_empty() {
                                anchor_id = Some(s);
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    // Extract title text, stripping anchor symbol glyphs (#, ¶, §)
    let raw_text = tag.inner_text(parser);
    let title = clean_heading_text(raw_text.as_ref());

    (title, anchor_id)
}

/// Cleans heading text by stripping self-referential anchor symbols.
fn clean_heading_text(raw: &str) -> String {
    let mut cleaned = raw
        .replace(['#', '¶', '§'], "")
        .replace("Direct link to this heading", "")
        .replace("Permalink", "")
        .trim()
        .to_string();

    // Remove excessive inner whitespace
    while cleaned.contains("  ") {
        cleaned = cleaned.replace("  ", " ");
    }
    cleaned
}

/// Transforms a flat list of headings into a nested `TocItem` tree.
fn build_toc_tree(flat: &[FlatHeading]) -> Vec<TocItem> {
    let mut root_items: Vec<TocItem> = Vec::new();

    for item in flat {
        let toc_node = TocItem {
            level: item.level,
            title: item.title.clone(),
            anchor_id: item.anchor_id.clone(),
            children: Vec::new(),
        };

        insert_into_tree(&mut root_items, toc_node);
    }

    root_items
}

fn insert_into_tree(tree: &mut Vec<TocItem>, node: TocItem) {
    if let Some(last) = tree.last_mut() {
        if node.level > last.level {
            insert_into_tree(&mut last.children, node);
            return;
        }
    }
    tree.push(node);
}

/// Builds document sections from flat headings.
fn build_doc_sections(flat: &[FlatHeading]) -> Vec<DocSection> {
    flat.iter()
        .map(|h| DocSection {
            heading: h.title.clone(),
            level: h.level,
            anchor_id: h.anchor_id.clone(),
            content_text: h.content_text.clone(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_table_of_contents_tree() {
        let html = r##"
        <main>
            <h1 id="overview">Overview #</h1>
            <p>Some text</p>
            <h2 id="getting-started">Getting Started <a class="hash-link" href="#getting-started">#</a></h2>
            <p>Installation guide</p>
            <h3 id="prerequisites">Prerequisites</h3>
            <p>Requirements</p>
            <h2 id="configuration">Configuration ¶</h2>
            <p>Settings</p>
        </main>
        "##;

        let vdom = tl::parse(html, tl::ParserOptions::default()).unwrap();
        let parser = vdom.parser();
        let handle = vdom.query_selector("main").unwrap().next().unwrap();

        let (toc, sections) = extract_table_of_contents(handle, parser);

        assert_eq!(toc.len(), 1);
        assert_eq!(toc[0].title, "Overview");
        assert_eq!(toc[0].anchor_id.as_deref(), Some("overview"));
        assert_eq!(toc[0].children.len(), 2);

        assert_eq!(toc[0].children[0].title, "Getting Started");
        assert_eq!(
            toc[0].children[0].anchor_id.as_deref(),
            Some("getting-started")
        );
        assert_eq!(toc[0].children[0].children.len(), 1);
        assert_eq!(toc[0].children[0].children[0].title, "Prerequisites");

        assert_eq!(toc[0].children[1].title, "Configuration");
        assert_eq!(
            toc[0].children[1].anchor_id.as_deref(),
            Some("configuration")
        );

        assert_eq!(sections.len(), 4);
        assert_eq!(sections[0].heading, "Overview");
        assert_eq!(sections[1].heading, "Getting Started");
    }
}
