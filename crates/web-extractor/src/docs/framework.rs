//! # Documentation Framework Detection
//!
//! Identifies the documentation generator framework (Docusaurus, mdBook, Sphinx,
//! MkDocs, Rustdoc, Nextra, VitePress, GitBook) by inspecting meta generator tags,
//! structural DOM patterns, and framework-specific CSS class signatures.

use super::model::DocFramework;

/// Inspects a parsed `tl::VDom` to identify the underlying documentation generator.
#[must_use]
pub fn detect_framework(vdom: &tl::VDom<'_>) -> DocFramework {
    let parser = vdom.parser();

    // 1. Check <meta name="generator" content="...">
    if let Some(handles) = vdom.query_selector("meta[name=\"generator\"]") {
        for handle in handles {
            if let Some(tl::Node::Tag(tag)) = handle.get(parser) {
                if let Some(Some(content)) = tag.attributes().get("content") {
                    let cow = content.as_utf8_str();
                    let val = cow.to_ascii_lowercase();
                    if val.contains("docusaurus") {
                        return DocFramework::Docusaurus;
                    }
                    if val.contains("mdbook") {
                        return DocFramework::MdBook;
                    }
                    if val.contains("sphinx") {
                        return DocFramework::Sphinx;
                    }
                    if val.contains("mkdocs") {
                        return DocFramework::MkDocs;
                    }
                    if val.contains("rustdoc") {
                        return DocFramework::Rustdoc;
                    }
                    if val.contains("gitbook") {
                        return DocFramework::GitBook;
                    }
                    if val.contains("nextra") {
                        return DocFramework::Nextra;
                    }
                    if val.contains("vitepress") {
                        return DocFramework::VitePress;
                    }
                }
            }
        }
    }

    // 2. Check characteristic CSS classes and IDs
    // Docusaurus signatures
    if has_selector(vdom, ".theme-doc-markdown")
        || has_selector(vdom, ".theme-doc-sidebar-menu")
        || has_selector(vdom, ".docusaurus")
    {
        return DocFramework::Docusaurus;
    }

    // mdBook signatures
    if has_selector(vdom, "nav#sidebar.sidebar")
        || (has_selector(vdom, "div#content.content") && has_selector(vdom, "i.fa-angle-right"))
    {
        return DocFramework::MdBook;
    }

    // Sphinx signatures
    if has_selector(vdom, ".sphinxsidebar")
        || has_selector(vdom, ".sphinxsidebarwrapper")
        || has_selector(vdom, "div.documentwrapper")
    {
        return DocFramework::Sphinx;
    }

    // MkDocs / Material for MkDocs signatures
    if has_selector(vdom, ".md-content")
        || has_selector(vdom, ".md-sidebar")
        || has_selector(vdom, ".md-header")
    {
        return DocFramework::MkDocs;
    }

    // Rustdoc signatures
    if has_selector(vdom, "section#main-content.content")
        || has_selector(vdom, ".rustdoc")
        || has_selector(vdom, "div.main-heading")
    {
        return DocFramework::Rustdoc;
    }

    // VitePress signatures
    if has_selector(vdom, ".vp-doc")
        || has_selector(vdom, ".VPSidebar")
        || has_selector(vdom, ".VPNavBar")
    {
        return DocFramework::VitePress;
    }

    // Nextra signatures
    if has_selector(vdom, ".nextra-content") || has_selector(vdom, ".nextra-sidebar") {
        return DocFramework::Nextra;
    }

    // GitBook signatures
    if has_selector(vdom, ".gitbook-root") || has_selector(vdom, "[data-gitbook]") {
        return DocFramework::GitBook;
    }

    DocFramework::Generic
}

fn has_selector(vdom: &tl::VDom<'_>, selector: &str) -> bool {
    vdom.query_selector(selector)
        .is_some_and(|mut it| it.next().is_some())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_docusaurus_by_meta_and_classes() {
        let html = r#"<html><head><meta name="generator" content="Docusaurus v3.0"></head><body><div class="theme-doc-markdown"></div></body></html>"#;
        let vdom = tl::parse(html, tl::ParserOptions::default()).unwrap();
        assert_eq!(detect_framework(&vdom), DocFramework::Docusaurus);
    }

    #[test]
    fn test_detect_mdbook() {
        let html = r#"<html><head><meta name="generator" content="mdBook 0.4.35"></head><body><nav id="sidebar" class="sidebar"></nav></body></html>"#;
        let vdom = tl::parse(html, tl::ParserOptions::default()).unwrap();
        assert_eq!(detect_framework(&vdom), DocFramework::MdBook);
    }

    #[test]
    fn test_detect_generic() {
        let html = r#"<html><body><h1>Docs</h1><p>Welcome to the guide.</p></body></html>"#;
        let vdom = tl::parse(html, tl::ParserOptions::default()).unwrap();
        assert_eq!(detect_framework(&vdom), DocFramework::Generic);
    }
}
