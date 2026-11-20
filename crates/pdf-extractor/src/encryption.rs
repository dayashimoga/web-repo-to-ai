//! # Encrypted PDF Detection & Permission Analysis
//!
//! Inspects PDF encryption dictionaries (`/Encrypt`), parses user/owner authentication requirements,
//! and decodes standard permission flags (ISO 32000-1, Table 22).

use lopdf::{Document, Object};
use serde::{Deserialize, Serialize};

/// Decoded standard PDF permission flags (ISO 32000-1, Table 22).
///
/// Encoded as a 32-bit signed integer in the `/P` entry of the standard encryption dictionary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PdfPermissions {
    /// Raw permission bitmask integer from the `/P` entry.
    pub raw_flags: i64,

    /// Bit 3: Print the document (possibly at lower resolution).
    pub can_print: bool,

    /// Bit 4: Modify the contents of the document.
    pub can_modify: bool,

    /// Bit 5: Copy or otherwise extract text and graphics.
    pub can_copy: bool,

    /// Bit 6: Add or modify text annotations, fill interactive form fields.
    pub can_annotate: bool,

    /// Bit 9: Fill interactive form fields (even if `can_annotate` is false).
    pub can_fill_forms: bool,

    /// Bit 10: Extract text and graphics in support of accessibility.
    pub can_extract_for_accessibility: bool,

    /// Bit 11: Assemble the document (insert, rotate, delete pages).
    pub can_assemble: bool,

    /// Bit 12: Print to high-resolution / full fidelity representation.
    pub can_print_high_quality: bool,
}

impl PdfPermissions {
    /// Decodes permission flags from the standard signed 32-bit integer `P`.
    #[must_use]
    pub fn from_flags(flags: i64) -> Self {
        // According to ISO 32000-1, permissions are stored in bits 3..12
        let p = flags as i32;
        Self {
            raw_flags: flags,
            can_print: (p & (1 << 2)) != 0,
            can_modify: (p & (1 << 3)) != 0,
            can_copy: (p & (1 << 4)) != 0,
            can_annotate: (p & (1 << 5)) != 0,
            can_fill_forms: (p & (1 << 8)) != 0,
            can_extract_for_accessibility: (p & (1 << 9)) != 0,
            can_assemble: (p & (1 << 10)) != 0,
            can_print_high_quality: (p & (1 << 11)) != 0,
        }
    }

    /// Creates full-access permissions (all operations allowed).
    #[must_use]
    pub const fn all_allowed() -> Self {
        Self {
            raw_flags: -1,
            can_print: true,
            can_modify: true,
            can_copy: true,
            can_annotate: true,
            can_fill_forms: true,
            can_extract_for_accessibility: true,
            can_assemble: true,
            can_print_high_quality: true,
        }
    }
}

/// Structured metadata describing PDF document encryption.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EncryptionInfo {
    /// Whether the document is encrypted.
    pub is_encrypted: bool,

    /// The security handler name (e.g. `Standard`).
    pub handler: Option<String>,

    /// The algorithm version (`/V`: 1 = 40-bit RC4, 2 = 128-bit RC4/AES, 4 = AES-128, 5 = AES-256).
    pub version: Option<i64>,

    /// The revision of the standard security handler (`/R`: 2, 3, 4, 5, 6).
    pub revision: Option<i64>,

    /// Decoded user permissions if present.
    pub permissions: Option<PdfPermissions>,

    /// Whether a user password entry (`/U`) is present in the encryption dictionary.
    pub has_user_password: bool,

    /// Whether an owner password entry (`/O`) is present in the encryption dictionary.
    pub has_owner_password: bool,
}

impl EncryptionInfo {
    /// Returns true if content extraction/copying is permitted according to decoded permissions.
    /// If no permissions entry exists, returns true by default.
    #[must_use]
    pub fn allows_content_copying(&self) -> bool {
        self.permissions.as_ref().map_or(true, |p| p.can_copy)
    }

    /// Returns a human-readable description of the encryption algorithm.
    #[must_use]
    pub fn algorithm_description(&self) -> String {
        match (self.version, self.revision) {
            (Some(1), _) => "RC4 40-bit (Legacy PDF 1.1)".to_string(),
            (Some(2), _) => "RC4 128-bit (PDF 1.4)".to_string(),
            (Some(4), _) => "AES-128 / RC4 128-bit (PDF 1.5+)".to_string(),
            (Some(5), _) => "AES-256 (PDF 1.7 Extension 3 / PDF 2.0)".to_string(),
            _ => format!(
                "Standard Handler V={:?} R={:?}",
                self.version, self.revision
            ),
        }
    }
}

/// Inspects a parsed `lopdf::Document` to extract encryption metadata.
#[must_use]
pub fn extract_encryption_info(doc: &Document) -> Option<EncryptionInfo> {
    let encrypt_obj = match doc.trailer.get(b"Encrypt") {
        Ok(Object::Reference(id)) => doc.get_object(*id).ok()?,
        Ok(obj) => obj,
        Err(_) => return None,
    };

    let dict = encrypt_obj.as_dict().ok()?;
    let handler = dict
        .get(b"Filter")
        .ok()
        .and_then(|obj| obj.as_name().ok())
        .map(|n| String::from_utf8_lossy(n).to_string());
    let version = dict.get(b"V").ok().and_then(|obj| obj.as_i64().ok());
    let revision = dict.get(b"R").ok().and_then(|obj| obj.as_i64().ok());
    let permissions = dict
        .get(b"P")
        .ok()
        .and_then(|obj| obj.as_i64().ok())
        .map(PdfPermissions::from_flags);
    let has_user_password = dict.get(b"U").is_ok();
    let has_owner_password = dict.get(b"O").is_ok();

    Some(EncryptionInfo {
        is_encrypted: true,
        handler,
        version,
        revision,
        permissions,
        has_user_password,
        has_owner_password,
    })
}

/// Inspects raw PDF bytes to locate and parse the `/Encrypt` dictionary if available,
/// even when lopdf cross-reference parsing fails due to encryption locks.
#[must_use]
pub fn extract_raw_encryption_info(bytes: &[u8]) -> Option<EncryptionInfo> {
    // Search for "/Encrypt" in trailer or catalog
    let encrypt_pos = bytes.windows(8).position(|w| w == b"/Encrypt")?;
    let slice = &bytes[encrypt_pos..bytes.len().min(encrypt_pos + 1024)];
    let slice_str = String::from_utf8_lossy(slice);

    // If /Encrypt is an indirect reference (e.g. "/Encrypt 7 0 R"), locate the referenced object
    let dict_str = if let Some(dict_start) = slice_str.find("<<") {
        slice_str[dict_start..].to_string()
    } else {
        let after_enc = slice_str.trim_start_matches("/Encrypt").trim_start();
        let obj_num = after_enc.split_whitespace().next().unwrap_or("");
        if !obj_num.is_empty() && obj_num.chars().all(|c| c.is_ascii_digit()) {
            let obj_pattern = format!("{obj_num} 0 obj");
            if let Some(pos) = bytes
                .windows(obj_pattern.len())
                .position(|w| w == obj_pattern.as_bytes())
            {
                let obj_slice = &bytes[pos..bytes.len().min(pos + 2048)];
                String::from_utf8_lossy(obj_slice).to_string()
            } else {
                slice_str.to_string()
            }
        } else if let Some(pos) = bytes
            .windows(16)
            .position(|w| w.starts_with(b"/Filter /Standard") || w.starts_with(b"/Filter/Standard"))
        {
            let start = pos.saturating_sub(64);
            let obj_slice = &bytes[start..bytes.len().min(pos + 2048)];
            String::from_utf8_lossy(obj_slice).to_string()
        } else {
            slice_str.to_string()
        }
    };

    // Look for permission flag /P <number>
    let mut permissions = None;
    if let Some(p_idx) = dict_str.find("/P ") {
        let after_p = &dict_str[p_idx + 3..];
        let token = after_p
            .split(|c: char| c.is_whitespace() || c == '/' || c == '>' || c == ']')
            .next()
            .unwrap_or("");
        if let Ok(flags) = token.parse::<i64>() {
            permissions = Some(PdfPermissions::from_flags(flags));
        }
    }

    // Look for /V <version>
    let mut version = None;
    if let Some(v_idx) = dict_str.find("/V ") {
        let after_v = &dict_str[v_idx + 3..];
        let token = after_v
            .split(|c: char| c.is_whitespace() || c == '/' || c == '>' || c == ']')
            .next()
            .unwrap_or("");
        if let Ok(v) = token.parse::<i64>() {
            version = Some(v);
        }
    }

    // Look for /R <revision>
    let mut revision = None;
    if let Some(r_idx) = dict_str.find("/R ") {
        let after_r = &dict_str[r_idx + 3..];
        let token = after_r
            .split(|c: char| c.is_whitespace() || c == '/' || c == '>' || c == ']')
            .next()
            .unwrap_or("");
        if let Ok(r) = token.parse::<i64>() {
            revision = Some(r);
        }
    }

    let has_user_password =
        dict_str.contains("/U ") || dict_str.contains("/U<") || dict_str.contains("/U(");
    let has_owner_password =
        dict_str.contains("/O ") || dict_str.contains("/O<") || dict_str.contains("/O(");

    Some(EncryptionInfo {
        is_encrypted: true,
        handler: Some("Standard".to_string()),
        version,
        revision,
        permissions,
        has_user_password,
        has_owner_password,
    })
}
