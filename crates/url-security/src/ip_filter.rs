//! # IP Filtering and Host Security
//!
//! Enforces zero-trust IP range validation and host restrictions to mitigate SSRF
//! (Server-Side Request Forgery) attacks, including loopback, RFC 1918 private ranges,
//! link-local, cloud metadata services, and alternative IP address encodings.

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use crate::error::UrlSecurityError;
use crate::policy::UrlSecurityPolicy;

/// Evaluates whether an IP address belongs to a prohibited network or address space.
///
/// Unless explicitly permitted by the `allow_private_ips` policy flag, this function rejects:
/// - IPv4 loopback (`127.0.0.0/8`)
/// - IPv4 private networks (`10.0.0.0/8`, `172.16.0.0/12`, `192.168.0.0/16`)
/// - IPv4 link-local & cloud metadata (`169.254.0.0/16`)
/// - IPv4 broadcast, multicast, unspecified (`0.0.0.0/8`, `224.0.0.0/4`, `240.0.0.0/4`, `255.255.255.255`)
/// - IPv4 CGNAT / Shared Address Space (`100.64.0.0/10`)
/// - IPv4 documentation / benchmarking ranges (`192.0.2.0/24`, `198.51.100.0/24`, `203.0.113.0/24`, `198.18.0.0/15`)
/// - IPv6 loopback (`::1`) and unspecified (`::`)
/// - IPv6 unique-local addresses (`fc00::/7`)
/// - IPv6 link-local unicast (`fe80::/10`)
/// - IPv6 multicast (`ff00::/8`)
/// - IPv6 documentation (`2001:db8::/32`)
/// - IPv4-mapped, IPv4-compatible, 6to4, and Teredo IPv6 tunnels targeting prohibited IPv4s
pub fn is_blocked_ip(ip: IpAddr, policy: &UrlSecurityPolicy) -> Result<(), UrlSecurityError> {
    if policy.allow_private_ips {
        // If private IPs are permitted (e.g. in test suites), only cloud metadata is checked if enabled
        if policy.block_cloud_metadata && is_cloud_metadata_ip(ip) {
            return Err(UrlSecurityError::BlockedIp {
                ip: ip.to_string(),
                category: "Cloud Instance Metadata Service (169.254.169.254)".to_string(),
            });
        }
        return Ok(());
    }

    match ip {
        IpAddr::V4(v4) => check_ipv4(v4, policy),
        IpAddr::V6(v6) => check_ipv6(v6, policy),
    }
}

/// Helper evaluating an `Ipv4Addr` against restricted CIDRs.
fn check_ipv4(ip: Ipv4Addr, policy: &UrlSecurityPolicy) -> Result<(), UrlSecurityError> {
    let octets = ip.octets();

    // Loopback: 127.0.0.0/8
    if octets[0] == 127 {
        return Err(UrlSecurityError::BlockedIp {
            ip: ip.to_string(),
            category: "IPv4 Loopback (127.0.0.0/8)".to_string(),
        });
    }

    // Unspecified: 0.0.0.0/8
    if octets[0] == 0 {
        return Err(UrlSecurityError::BlockedIp {
            ip: ip.to_string(),
            category: "IPv4 Unspecified (0.0.0.0/8)".to_string(),
        });
    }

    // RFC 1918 Private: 10.0.0.0/8
    if octets[0] == 10 {
        return Err(UrlSecurityError::BlockedIp {
            ip: ip.to_string(),
            category: "IPv4 RFC1918 Private (10.0.0.0/8)".to_string(),
        });
    }

    // RFC 1918 Private: 172.16.0.0/12 (172.16.0.0 - 172.31.255.255)
    if octets[0] == 172 && (16..=31).contains(&octets[1]) {
        return Err(UrlSecurityError::BlockedIp {
            ip: ip.to_string(),
            category: "IPv4 RFC1918 Private (172.16.0.0/12)".to_string(),
        });
    }

    // RFC 1918 Private: 192.168.0.0/16
    if octets[0] == 192 && octets[1] == 168 {
        return Err(UrlSecurityError::BlockedIp {
            ip: ip.to_string(),
            category: "IPv4 RFC1918 Private (192.168.0.0/16)".to_string(),
        });
    }

    // Link-Local / Cloud Metadata: 169.254.0.0/16 (RFC 3927)
    if octets[0] == 169 && octets[1] == 254 {
        let category = if octets[2] == 169 && octets[3] == 254 {
            "Cloud Instance Metadata Service (169.254.169.254)"
        } else {
            "IPv4 Link-Local (169.254.0.0/16)"
        };
        return Err(UrlSecurityError::BlockedIp {
            ip: ip.to_string(),
            category: category.to_string(),
        });
    }

    // Shared Address Space / CGNAT: 100.64.0.0/10 (100.64.0.0 - 100.127.255.255)
    if octets[0] == 100 && (64..=127).contains(&octets[1]) {
        return Err(UrlSecurityError::BlockedIp {
            ip: ip.to_string(),
            category: "IPv4 CGNAT / Shared Address Space (100.64.0.0/10)".to_string(),
        });
    }

    // Multicast: 224.0.0.0/4
    if (224..=239).contains(&octets[0]) {
        return Err(UrlSecurityError::BlockedIp {
            ip: ip.to_string(),
            category: "IPv4 Multicast (224.0.0.0/4)".to_string(),
        });
    }

    // Reserved / Future Use: 240.0.0.0/4
    if octets[0] >= 240 {
        return Err(UrlSecurityError::BlockedIp {
            ip: ip.to_string(),
            category: "IPv4 Reserved / Broadcast (240.0.0.0/4)".to_string(),
        });
    }

    // Documentation: TEST-NET-1 (192.0.2.0/24), TEST-NET-2 (198.51.100.0/24), TEST-NET-3 (203.0.113.0/24)
    if (octets[0] == 192 && octets[1] == 0 && octets[2] == 2)
        || (octets[0] == 198 && octets[1] == 51 && octets[2] == 100)
        || (octets[0] == 203 && octets[1] == 0 && octets[2] == 113)
    {
        return Err(UrlSecurityError::BlockedIp {
            ip: ip.to_string(),
            category: "IPv4 Documentation Range".to_string(),
        });
    }

    // Benchmarking: 198.18.0.0/15 (198.18.0.0 - 198.19.255.255)
    if octets[0] == 198 && (octets[1] == 18 || octets[1] == 19) {
        return Err(UrlSecurityError::BlockedIp {
            ip: ip.to_string(),
            category: "IPv4 Benchmarking (198.18.0.0/15)".to_string(),
        });
    }

    let _ = policy;
    Ok(())
}

/// Helper evaluating an `Ipv6Addr` against restricted IPv6 ranges and encapsulated IPv4 addresses.
fn check_ipv6(ip: Ipv6Addr, policy: &UrlSecurityPolicy) -> Result<(), UrlSecurityError> {
    let segments = ip.segments();
    let octets = ip.octets();

    // Loopback: ::1
    if ip.is_loopback() {
        return Err(UrlSecurityError::BlockedIp {
            ip: ip.to_string(),
            category: "IPv6 Loopback (::1)".to_string(),
        });
    }

    // Unspecified: ::
    if ip.is_unspecified() {
        return Err(UrlSecurityError::BlockedIp {
            ip: ip.to_string(),
            category: "IPv6 Unspecified (::)".to_string(),
        });
    }

    // Unique-Local: fc00::/7 (first byte has top 7 bits matching 1111110)
    if (octets[0] & 0xfe) == 0xfc {
        return Err(UrlSecurityError::BlockedIp {
            ip: ip.to_string(),
            category: "IPv6 Unique-Local (fc00::/7)".to_string(),
        });
    }

    // Link-Local: fe80::/10 (first byte 0xfe, second byte top 2 bits 10)
    if octets[0] == 0xfe && (octets[1] & 0xc0) == 0x80 {
        return Err(UrlSecurityError::BlockedIp {
            ip: ip.to_string(),
            category: "IPv6 Link-Local (fe80::/10)".to_string(),
        });
    }

    // Multicast: ff00::/8
    if octets[0] == 0xff {
        return Err(UrlSecurityError::BlockedIp {
            ip: ip.to_string(),
            category: "IPv6 Multicast (ff00::/8)".to_string(),
        });
    }

    // Documentation: 2001:db8::/32
    if segments[0] == 0x2001 && segments[1] == 0x0db8 {
        return Err(UrlSecurityError::BlockedIp {
            ip: ip.to_string(),
            category: "IPv6 Documentation (2001:db8::/32)".to_string(),
        });
    }

    // IPv4-mapped IPv6 address: ::ffff:w.x.y.z (or ::ffff:0:0/96)
    // Extract underlying IPv4 address and evaluate
    if segments[0] == 0
        && segments[1] == 0
        && segments[2] == 0
        && segments[3] == 0
        && segments[4] == 0
        && segments[5] == 0xffff
    {
        let mapped_v4 = Ipv4Addr::new(octets[12], octets[13], octets[14], octets[15]);
        return check_ipv4(mapped_v4, policy).map_err(|_| UrlSecurityError::BlockedIp {
            ip: ip.to_string(),
            category: format!("IPv4-mapped IPv6 address targeting restricted IPv4 ({mapped_v4})"),
        });
    }

    // IPv4-compatible IPv6 (deprecated): ::w.x.y.z
    if segments[0] == 0
        && segments[1] == 0
        && segments[2] == 0
        && segments[3] == 0
        && segments[4] == 0
        && segments[5] == 0
        && !(segments[6] == 0 && segments[7] == 1)
    // not ::1
    {
        let compat_v4 = Ipv4Addr::new(octets[12], octets[13], octets[14], octets[15]);
        return check_ipv4(compat_v4, policy).map_err(|_| UrlSecurityError::BlockedIp {
            ip: ip.to_string(),
            category: format!(
                "IPv4-compatible IPv6 address targeting restricted IPv4 ({compat_v4})"
            ),
        });
    }

    // 6to4 encapsulation: 2002::/16
    // Bytes 2..6 represent the encapsulated IPv4 address
    if segments[0] == 0x2002 {
        let v4 = Ipv4Addr::new(octets[2], octets[3], octets[4], octets[5]);
        return check_ipv4(v4, policy).map_err(|_| UrlSecurityError::BlockedIp {
            ip: ip.to_string(),
            category: format!("6to4 encapsulated restricted IPv4 ({v4})"),
        });
    }

    // Teredo tunneling: 2001:0000::/32
    // Bytes 12..16 represent the client IPv4 address inverted (XOR 0xffffffff)
    if segments[0] == 0x2001 && segments[1] == 0 {
        let v4 = Ipv4Addr::new(
            octets[12] ^ 0xff,
            octets[13] ^ 0xff,
            octets[14] ^ 0xff,
            octets[15] ^ 0xff,
        );
        return check_ipv4(v4, policy).map_err(|_| UrlSecurityError::BlockedIp {
            ip: ip.to_string(),
            category: format!("Teredo tunnel restricted IPv4 ({v4})"),
        });
    }

    Ok(())
}

/// Identifies whether the IP address matches cloud metadata service IPs.
#[must_use]
pub fn is_cloud_metadata_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => v4 == Ipv4Addr::new(169, 254, 169, 254),
        IpAddr::V6(v6) => {
            // Check for IPv4-mapped 169.254.169.254
            let octets = v6.octets();
            let segments = v6.segments();
            if segments[0] == 0
                && segments[1] == 0
                && segments[2] == 0
                && segments[3] == 0
                && segments[4] == 0
                && segments[5] == 0xffff
            {
                octets[12] == 169 && octets[13] == 254 && octets[14] == 169 && octets[15] == 254
            } else {
                false
            }
        }
    }
}

/// Evaluates whether a hostname string targets a prohibited or internal domain or IP.
///
/// This checks:
/// 1. Host existence (non-empty).
/// 2. Direct policy blocked host matches.
/// 3. Standard local domain suffixes (`.localhost`, `.local`, `.internal`, `.lan`, `.localdomain`).
/// 4. Known cloud metadata service hostnames (`metadata.google.internal`, etc.).
/// 5. Standard IPv4/IPv6 address literals.
/// 6. Alternative IP representations (decimal, hex, octal, dotted abbreviations like `127.1`).
pub fn is_blocked_host(host_str: &str, policy: &UrlSecurityPolicy) -> Result<(), UrlSecurityError> {
    let clean = host_str.trim().trim_end_matches('.');
    if clean.is_empty() {
        return Err(UrlSecurityError::MissingHost);
    }

    let lower = clean.to_ascii_lowercase();

    // 1. Direct match in policy blocked_hosts
    if policy
        .blocked_hosts
        .iter()
        .any(|h| h.eq_ignore_ascii_case(&lower))
    {
        return Err(UrlSecurityError::BlockedHost {
            host: lower.clone(),
            reason: "Host is explicitly blocked by policy".to_string(),
        });
    }

    // 2. Loopback / local hostnames
    if lower == "localhost"
        || lower.ends_with(".localhost")
        || lower.ends_with(".local")
        || lower.ends_with(".internal")
        || lower.ends_with(".localdomain")
        || lower.ends_with(".lan")
        || lower.ends_with(".home.arpa")
    {
        return Err(UrlSecurityError::BlockedHost {
            host: lower,
            reason: "Local or internal domain name prohibited".to_string(),
        });
    }

    // 3. Cloud metadata hostnames
    if lower == "metadata"
        || lower == "instance-data"
        || lower.contains("metadata.google")
        || lower.contains("169.254.169.254")
    {
        return Err(UrlSecurityError::BlockedHost {
            host: lower,
            reason: "Cloud metadata service hostname prohibited".to_string(),
        });
    }

    // 4. Check if host parses directly as a standard IP address
    // (stripping optional IPv6 RFC 6874 zone index e.g. %eth0 or %25eth0)
    let unbracketed = lower.trim_start_matches('[').trim_end_matches(']');
    let ip_str = if let Some((base, _)) = unbracketed.split_once('%') {
        base
    } else {
        unbracketed
    };
    if let Ok(ip) = ip_str.parse::<IpAddr>() {
        return is_blocked_ip(ip, policy);
    }

    // 5. Check alternative IPv4 formats (decimal integer, hex, octal, multi-base dotted)
    if let Some(parsed_v4) = parse_alternative_ipv4(&lower) {
        return is_blocked_ip(IpAddr::V4(parsed_v4), policy);
    }

    // 6. Check percent-encoded hostname obfuscation (e.g. %31%32%37%2E%30%2E%30%2E%31)
    if lower.contains('%') {
        if let Some(decoded) = decode_percent_host(&lower) {
            let dec_clean = decoded.trim().trim_end_matches('.');
            if let Ok(ip) = dec_clean.parse::<IpAddr>() {
                return is_blocked_ip(ip, policy);
            }
            if let Some(parsed_v4) = parse_alternative_ipv4(dec_clean) {
                return is_blocked_ip(IpAddr::V4(parsed_v4), policy);
            }
        }
    }

    Ok(())
}

/// Decodes percent-encoded character sequences (%XX) in hostname strings.
fn decode_percent_host(s: &str) -> Option<String> {
    let mut decoded = Vec::with_capacity(s.len());
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            let hex_str = std::str::from_utf8(&bytes[i + 1..i + 3]).ok()?;
            if let Ok(byte) = u8::from_str_radix(hex_str, 16) {
                decoded.push(byte);
                i += 3;
                continue;
            }
        }
        decoded.push(bytes[i]);
        i += 1;
    }
    String::from_utf8(decoded).ok()
}

/// Parses non-standard IPv4 notations (POSIX `inet_aton` syntax) used in SSRF bypass attempts.
///
/// Handles:
/// - Single 32-bit integer: decimal (`2130706433`), hex (`0x7f000001`), octal (`017700000001`) -> `127.0.0.1`
/// - Two parts: `127.1` -> `127.0.0.1`
/// - Three parts: `127.0.1` -> `127.0.0.1`
/// - Four parts with hex or octal: `0x7f.0.0.1`, `0177.0.0.1`
#[must_use]
pub fn parse_alternative_ipv4(host: &str) -> Option<Ipv4Addr> {
    let parts: Vec<&str> = host.split('.').collect();
    if parts.is_empty() || parts.len() > 4 {
        return None;
    }

    let mut numbers = Vec::with_capacity(parts.len());
    for part in parts {
        let n = parse_number(part)?;
        numbers.push(n);
    }

    match numbers.len() {
        1 => {
            // Single 32-bit integer
            let val = numbers[0];
            if val <= u32::MAX as u64 {
                let u = val as u32;
                Some(Ipv4Addr::from(u))
            } else {
                None
            }
        }
        2 => {
            // a.b: a is 8-bit, b is 24-bit
            let a = numbers[0];
            let b = numbers[1];
            if a <= 255 && b <= 0x00ff_ffff {
                let u = ((a as u32) << 24) | (b as u32);
                Some(Ipv4Addr::from(u))
            } else {
                None
            }
        }
        3 => {
            // a.b.c: a is 8-bit, b is 8-bit, c is 16-bit
            let a = numbers[0];
            let b = numbers[1];
            let c = numbers[2];
            if a <= 255 && b <= 255 && c <= 0xffff {
                let u = ((a as u32) << 24) | ((b as u32) << 16) | (c as u32);
                Some(Ipv4Addr::from(u))
            } else {
                None
            }
        }
        4 => {
            // Standard 4 octets (each up to 255)
            let a = numbers[0];
            let b = numbers[1];
            let c = numbers[2];
            let d = numbers[3];
            if a <= 255 && b <= 255 && c <= 255 && d <= 255 {
                Some(Ipv4Addr::new(a as u8, b as u8, c as u8, d as u8))
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Parses a string part as a decimal, hex (`0x...`), or octal (`0...`) integer.
fn parse_number(s: &str) -> Option<u64> {
    if s.is_empty() {
        return None;
    }
    if let Some(stripped) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        u64::from_str_radix(stripped, 16).ok()
    } else if s.len() > 1 && s.starts_with('0') {
        // Octal notation (POSIX inet_aton)
        u64::from_str_radix(s, 8).ok()
    } else {
        s.parse::<u64>().ok()
    }
}
