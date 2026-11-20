//! PDF encryption and permission enforcement test suite.

use lopdf::{Dictionary, Document, Object, Stream};
use web_repo_to_ai_pdf_extractor::encryption::{
    extract_raw_encryption_info, EncryptionInfo, PdfPermissions,
};
use web_repo_to_ai_pdf_extractor::error::PdfError;
use web_repo_to_ai_pdf_extractor::parser::PdfParser;

/// Creates a synthetic encrypted PDF document with configurable permission flags.
fn create_synthetic_encrypted_pdf(perm_flags: i64) -> Vec<u8> {
    let mut doc = Document::with_version("1.6");
    let pages_id = doc.new_object_id();

    let font_id = doc.add_object(Dictionary::from_iter(vec![
        ("Type", Object::Name(b"Font".to_vec())),
        ("Subtype", Object::Name(b"Type1".to_vec())),
        ("BaseFont", Object::Name(b"Helvetica".to_vec())),
    ]));

    let resources_id = doc.add_object(Dictionary::from_iter(vec![(
        "Font",
        Object::Dictionary(Dictionary::from_iter(vec![(
            "F1",
            Object::Reference(font_id),
        )])),
    )]));

    let content = b"BT /F1 12 Tf 72 700 Td (Confidential encrypted report text) Tj ET";
    let content_id = doc.add_object(Stream::new(Dictionary::new(), content.to_vec()));

    let page_id = doc.add_object(Dictionary::from_iter(vec![
        ("Type", Object::Name(b"Page".to_vec())),
        ("Parent", Object::Reference(pages_id)),
        ("Contents", Object::Reference(content_id)),
        ("Resources", Object::Reference(resources_id)),
        (
            "MediaBox",
            Object::Array(vec![
                Object::Integer(0),
                Object::Integer(0),
                Object::Integer(612),
                Object::Integer(792),
            ]),
        ),
    ]));

    let pages = Dictionary::from_iter(vec![
        ("Type", Object::Name(b"Pages".to_vec())),
        ("Kids", Object::Array(vec![Object::Reference(page_id)])),
        ("Count", Object::Integer(1)),
    ]);
    doc.objects.insert(pages_id, Object::Dictionary(pages));

    let catalog_id = doc.add_object(Dictionary::from_iter(vec![
        ("Type", Object::Name(b"Catalog".to_vec())),
        ("Pages", Object::Reference(pages_id)),
    ]));
    doc.trailer.set("Root", Object::Reference(catalog_id));

    // Standard encryption dictionary
    let mut encrypt_dict = Dictionary::new();
    encrypt_dict.set("Filter", Object::Name(b"Standard".to_vec()));
    encrypt_dict.set("V", Object::Integer(4));
    encrypt_dict.set("R", Object::Integer(4));
    encrypt_dict.set("P", Object::Integer(perm_flags));
    encrypt_dict.set(
        "O",
        Object::String(vec![0xAA; 32], lopdf::StringFormat::Hexadecimal),
    );
    encrypt_dict.set(
        "U",
        Object::String(vec![0xBB; 32], lopdf::StringFormat::Hexadecimal),
    );
    let encrypt_id = doc.add_object(encrypt_dict);

    doc.trailer.set("Encrypt", Object::Reference(encrypt_id));

    let mut bytes = Vec::new();
    doc.save_to(&mut bytes)
        .expect("Failed to serialize encrypted PDF");
    bytes
}

#[test]
fn test_permission_flags_decoding() {
    // Flag with print allowed (bit 3), copy disallowed (bit 5 = 0), annotate allowed (bit 6)
    // Bit 3 = 4, Bit 6 = 32 -> 36
    let perms = PdfPermissions::from_flags(36);
    assert!(perms.can_print);
    assert!(!perms.can_modify);
    assert!(!perms.can_copy, "Bit 5 is not set, copying should be false");
    assert!(perms.can_annotate);

    // Full permissions (standard default is -4 or -64 with top bits set)
    // -4 as 32-bit = 0xFFFFFFFC (bits 2..31 all 1)
    let full_perms = PdfPermissions::from_flags(-4);
    assert!(full_perms.can_print);
    assert!(full_perms.can_modify);
    assert!(full_perms.can_copy);
    assert!(full_perms.can_annotate);
    assert!(full_perms.can_fill_forms);
    assert!(full_perms.can_extract_for_accessibility);
    assert!(full_perms.can_assemble);
    assert!(full_perms.can_print_high_quality);

    // Disallowed copy flag test: 0xFFFFFFEC (bit 5 cleared)
    let no_copy_flags = -4 & !(1 << 4);
    let restricted = PdfPermissions::from_flags(no_copy_flags);
    assert!(restricted.can_print);
    assert!(!restricted.can_copy);
}

#[test]
fn test_encryption_info_helpers() {
    let mut info = EncryptionInfo {
        is_encrypted: true,
        handler: Some("Standard".to_string()),
        version: Some(4),
        revision: Some(4),
        permissions: Some(PdfPermissions::from_flags(-4)),
        has_user_password: true,
        has_owner_password: true,
    };

    assert!(info.allows_content_copying());
    assert_eq!(
        info.algorithm_description(),
        "AES-128 / RC4 128-bit (PDF 1.5+)"
    );

    // Restrict copy permission
    info.permissions = Some(PdfPermissions::from_flags(0));
    assert!(!info.allows_content_copying());
}

#[test]
fn test_encrypted_pdf_rejection_without_password() {
    // Create an encrypted PDF with copy allowed
    let pdf_bytes = create_synthetic_encrypted_pdf(-4);
    let parser = PdfParser::new();

    let err = parser
        .parse(&pdf_bytes)
        .expect_err("Should reject encrypted PDF when no password is provided");

    assert!(
        matches!(err, PdfError::EncryptedDocument),
        "Expected EncryptedDocument error, got {err:?}"
    );
}

#[test]
fn test_encrypted_pdf_raw_trailer_extraction() {
    let pdf_bytes = create_synthetic_encrypted_pdf(-4);
    let raw_info = extract_raw_encryption_info(&pdf_bytes)
        .expect("Should locate /Encrypt dictionary in raw bytes");

    assert!(raw_info.is_encrypted);
    assert_eq!(raw_info.handler.as_deref(), Some("Standard"));
    assert_eq!(raw_info.version, Some(4));
    assert_eq!(raw_info.revision, Some(4));
    assert!(raw_info.permissions.is_some());
    assert!(raw_info.has_user_password);
    assert!(raw_info.has_owner_password);
}
