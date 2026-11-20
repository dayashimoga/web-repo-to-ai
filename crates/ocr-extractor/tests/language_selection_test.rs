//! # OCR Language Selection and Validation Integration Tests
//!
//! Validates ISO code normalization, multi-language combination parsing,
//! Tesseract model validation, and fallback policy enforcement.

use web_repo_to_ai_ocr_extractor::{
    canonicalize_languages, is_cjk, is_rtl, normalize_language_code, parse_language_string,
    resolve_languages, LanguageFallbackPolicy, OcrError, OcrOptions,
};

#[test]
fn test_language_normalization() {
    assert_eq!(normalize_language_code("en"), "eng");
    assert_eq!(normalize_language_code("EN"), "eng");
    assert_eq!(normalize_language_code("English"), "eng");
    assert_eq!(normalize_language_code("fr"), "fra");
    assert_eq!(normalize_language_code("French"), "fra");
    assert_eq!(normalize_language_code("de"), "deu");
    assert_eq!(normalize_language_code("German"), "deu");
    assert_eq!(normalize_language_code("es"), "spa");
    assert_eq!(normalize_language_code("Spanish"), "spa");
    assert_eq!(normalize_language_code("zh"), "chi_sim");
    assert_eq!(normalize_language_code("Chinese"), "chi_sim");
    assert_eq!(normalize_language_code("zh-tw"), "chi_tra");
    assert_eq!(normalize_language_code("ja"), "jpn");
    assert_eq!(normalize_language_code("Japanese"), "jpn");
    assert_eq!(normalize_language_code("ko"), "kor");
    assert_eq!(normalize_language_code("ar"), "ara");
    assert_eq!(normalize_language_code("hi"), "hin");
    assert_eq!(normalize_language_code("osd"), "osd");
}

#[test]
fn test_multi_language_parsing_and_canonicalization() {
    assert_eq!(
        parse_language_string("english, french, german"),
        vec!["eng".to_string(), "fra".to_string(), "deu".to_string()]
    );
    assert_eq!(
        parse_language_string("eng+spa;fra"),
        vec!["eng".to_string(), "spa".to_string(), "fra".to_string()]
    );
    assert_eq!(
        parse_language_string("eng  eng  fra"),
        vec!["eng".to_string(), "fra".to_string()]
    );

    assert_eq!(
        canonicalize_languages("english, french, spanish"),
        "eng+fra+spa"
    );
}

#[test]
fn test_language_resolution_policies() {
    let supported = vec![
        "eng".to_string(),
        "fra".to_string(),
        "deu".to_string(),
        "osd".to_string(),
    ];

    // 1. All supported succeeds under all policies
    assert_eq!(
        resolve_languages(
            "eng+fra",
            &supported,
            LanguageFallbackPolicy::StrictError,
            "eng"
        )
        .unwrap(),
        "eng+fra"
    );

    // 2. StrictError rejects missing language
    let err = resolve_languages(
        "eng+spa",
        &supported,
        LanguageFallbackPolicy::StrictError,
        "eng",
    )
    .expect_err("Missing spa must fail under StrictError");

    match err {
        OcrError::UnsupportedLanguage {
            requested,
            available,
        } => {
            assert_eq!(requested, "spa");
            assert!(available.contains(&"eng".to_string()));
        }
        other => panic!("Expected UnsupportedLanguage, got {other:?}"),
    }

    // 3. FallbackToDefault reverts to default
    let res_fallback = resolve_languages(
        "eng+spa",
        &supported,
        LanguageFallbackPolicy::FallbackToDefault,
        "eng",
    )
    .unwrap();
    assert_eq!(res_fallback, "eng");

    // 4. FilterUnavailable keeps available languages
    let res_filter = resolve_languages(
        "eng+spa+fra",
        &supported,
        LanguageFallbackPolicy::FilterUnavailable,
        "eng",
    )
    .unwrap();
    assert_eq!(res_filter, "eng+fra");

    // 5. FilterUnavailable falls back to default if no requested languages available
    let res_filter_none = resolve_languages(
        "spa+ita",
        &supported,
        LanguageFallbackPolicy::FilterUnavailable,
        "eng",
    )
    .unwrap();
    assert_eq!(res_filter_none, "eng");
}

#[test]
fn test_script_characteristics() {
    assert!(is_cjk("chi_sim"));
    assert!(is_cjk("chi_tra"));
    assert!(is_cjk("jpn"));
    assert!(is_cjk("kor"));
    assert!(!is_cjk("eng"));
    assert!(!is_cjk("fra"));

    assert!(is_rtl("ara"));
    assert!(is_rtl("heb"));
    assert!(is_rtl("fas"));
    assert!(!is_rtl("eng"));
    assert!(!is_rtl("deu"));
}

#[test]
fn test_ocr_options_language_builder() {
    let opts = OcrOptions::default()
        .with_language("fra+deu")
        .with_language_fallback(LanguageFallbackPolicy::StrictError);

    assert_eq!(opts.language, "fra+deu");
    assert_eq!(opts.language_fallback, LanguageFallbackPolicy::StrictError);
}
