//! Metindeki URL'lerden yalnızca host (alan adı) kısmını çıkarır.
//! Path, sorgu ve parametreler hiç incelenmez. Regex kullanılmaz; tek geçişli tarayıcıdır.

use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtractedHost {
    pub host: String,
    /// `https://paypal.com@evil.com` hilesinde '@' öncesinde görünen kısım.
    pub userinfo_decoy: Option<String>,
}

/// Şemasız metinde dosya adlarını (main.rs, x.json) alan adı sanmamak için.
const KNOWN_GTLDS: &[&str] = &[
    "com", "net", "org", "info", "biz", "io", "app", "dev", "ai", "xyz", "online", "site",
    "shop", "store", "tech", "cloud", "gov", "edu", "mil", "int", "pro", "club", "top", "live",
    "news", "blog", "page", "link", "click", "mobi", "name", "asia", "tel", "travel", "jobs",
    "bank", "lol", "fun", "icu", "vip", "win", "work", "art", "space", "website", "email",
    "support", "today", "world", "digital", "services", "security", "center", "network",
    "finance", "money", "pay", "wallet", "crypto", "account", "login", "help", "host", "zip",
    "mov",
];

fn is_ideographic_dot(c: char) -> bool {
    matches!(c, '\u{3002}' | '\u{FF0E}' | '\u{FF61}')
}

fn percent_decode(s: &str) -> String {
    fn hex(b: u8) -> Option<u8> {
        (b as char).to_digit(16).map(|v| v as u8)
    }
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%' && i + 2 < b.len() {
            if let (Some(h), Some(l)) = (hex(b[i + 1]), hex(b[i + 2])) {
                out.push(h * 16 + l);
                i += 3;
                continue;
            }
        }
        out.push(b[i]);
        i += 1;
    }
    String::from_utf8(out).unwrap_or_else(|_| s.to_string())
}

fn is_trim_char(c: char) -> bool {
    c.is_whitespace() || ".,;:!?()[]{}'\"<>".contains(c)
}

fn is_ipv4(h: &str) -> bool {
    let parts: Vec<&str> = h.split('.').collect();
    parts.len() == 4 && parts.iter().all(|p| !p.is_empty() && p.len() <= 3 && p.bytes().all(|b| b.is_ascii_digit()))
}

/// Host'u normalleştirir: ideografik noktalar, küçük harf, %-çözme, kenar noktalama.
pub fn normalize_host(raw: &str) -> Option<String> {
    let mut h: String = raw.chars().map(|c| if is_ideographic_dot(c) { '.' } else { c }).collect();
    if h.contains('%') {
        h = percent_decode(&h);
    }
    let h = h.to_lowercase();
    let h = h.trim_matches(is_trim_char);
    if !h.contains('.') || h.len() > 253 || is_ipv4(h) {
        return None;
    }
    if h.split('.').any(|l| l.is_empty() || l.chars().count() > 63) {
        return None;
    }
    Some(h.to_string())
}

fn is_scheme_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'+' || b == b'.' || b == b'-'
}

fn is_authority_end(c: char) -> bool {
    c.is_whitespace() || "/?#<>\"'`\\".contains(c)
}

fn is_domain_char(c: char) -> bool {
    c.is_alphanumeric() || c == '-' || c == '.' || is_ideographic_dot(c)
}

/// Tekilleştirilmiş host listesi. `max_hosts` çok büyük panolar için üst sınırdır.
pub fn extract_hosts(text: &str, max_hosts: usize) -> Vec<ExtractedHost> {
    let mut order: Vec<String> = Vec::new();
    let mut seen: HashMap<String, Option<String>> = HashMap::new();
    let mut push = |h: String, decoy: Option<String>, order: &mut Vec<String>| {
        match seen.get_mut(&h) {
            Some(d) => {
                if d.is_none() && decoy.is_some() {
                    *d = decoy;
                }
            }
            None => {
                order.push(h.clone());
                seen.insert(h, decoy);
            }
        }
    };

    // 1) Şemalı URL'ler: "xxx://authority"
    let bytes = text.as_bytes();
    let mut covered: Vec<(usize, usize)> = Vec::new();
    let mut search = 0;
    while let Some(off) = text[search..].find("://") {
        let sep = search + off;
        search = sep + 3;
        let mut start = sep;
        while start > 0 && is_scheme_char(bytes[start - 1]) {
            start -= 1;
        }
        // Şema harfle başlamalı
        while start < sep && !bytes[start].is_ascii_alphabetic() {
            start += 1;
        }
        if start == sep {
            continue;
        }
        let auth_start = sep + 3;
        let auth_len = text[auth_start..].find(is_authority_end).unwrap_or(text.len() - auth_start);
        let end = auth_start + auth_len;
        // Path/sorgu dahil tüm URL'yi kapsa: parametrelerdeki alan adları incelenmez.
        let url_len = text[end..]
            .find(|c: char| c.is_whitespace() || "<>\"'`".contains(c))
            .unwrap_or(text.len() - end);
        covered.push((start, end + url_len));
        search = (end + url_len).max(search);

        let mut auth = &text[auth_start..end];
        let mut decoy = None;
        if let Some(at) = auth.rfind('@') {
            let d = &auth[..at];
            if d.contains('.') {
                decoy = Some(d.to_string());
            }
            auth = &auth[at + 1..];
        }
        if auth.starts_with('[') {
            continue; // IPv6
        }
        if let Some(colon) = auth.find(':') {
            auth = &auth[..colon];
        }
        if let Some(h) = normalize_host(auth) {
            push(h, decoy, &mut order);
            if order.len() >= max_hosts {
                break;
            }
        }
    }

    // 2) Şemasız alan adları (e-posta adreslerindeki alan adları dahil)
    let mut ci = 0;
    let mut i = 0;
    while i < text.len() && order.len() < max_hosts {
        while ci < covered.len() && covered[ci].1 <= i {
            ci += 1;
        }
        if ci < covered.len() && covered[ci].0 <= i {
            i = covered[ci].1;
            continue;
        }
        let c = text[i..].chars().next().unwrap();
        if !is_domain_char(c) {
            i += c.len_utf8();
            continue;
        }
        let tok_start = i;
        let mut tok_end = i;
        for (o, ch) in text[i..].char_indices() {
            if !is_domain_char(ch) {
                break;
            }
            let pos = i + o;
            if ci < covered.len() && pos >= covered[ci].0 {
                break;
            }
            tok_end = pos + ch.len_utf8();
        }
        i = tok_end;
        let prev = text[..tok_start].chars().next_back();
        if prev == Some('/') {
            continue; // path parçası
        }
        let Some(h) = normalize_host(&text[tok_start..tok_end]) else { continue };
        if h.split('.').any(|l| l.starts_with('-') || l.ends_with('-')) {
            continue;
        }
        let tld = h.rsplit('.').next().unwrap();
        let tld_ok = tld.starts_with("xn--")
            || (tld.chars().all(char::is_alphabetic)
                && (tld.chars().count() == 2 || KNOWN_GTLDS.contains(&tld) || !tld.is_ascii()));
        if tld_ok {
            push(h, None, &mut order);
        }
    }

    order
        .into_iter()
        .map(|h| {
            let decoy = seen.get(&h).cloned().flatten();
            ExtractedHost { host: h, userinfo_decoy: decoy }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hosts(t: &str) -> Vec<String> {
        extract_hosts(t, 10_000).into_iter().map(|e| e.host).collect()
    }

    #[test]
    fn only_hosts_deduped() {
        let h = hosts(
            "https://www.Google.com/search?q=evil.com\n http://google.com:8080/a/b.html\n\
             www.google.com, bak: trendyol.com. dosya main.dart ve x.json\n\
             mail: ali@garantibbva.com.tr\n https://paypal.com@evil.example/login",
        );
        assert_eq!(
            h,
            ["www.google.com", "google.com", "evil.example", "trendyol.com", "garantibbva.com.tr"]
        );
    }

    #[test]
    fn decoy() {
        let e = extract_hosts("https://paypal.com@evil.example/x", 10);
        assert_eq!(e[0].userinfo_decoy.as_deref(), Some("paypal.com"));
    }

    #[test]
    fn unicode_and_percent() {
        assert_eq!(hosts("https://аpple.com/x"), ["аpple.com"]);
        assert_eq!(hosts("https://%D0%B0pple.com/"), ["аpple.com"]);
        assert_eq!(hosts("apple。com"), ["apple.com"]);
        assert!(hosts("1.2.3.4 ve 3.14").is_empty());
    }
}
