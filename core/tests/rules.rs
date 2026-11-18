use homoglyph_core::{punycode, Detector, Lang, Reason, Risk};
use std::time::Instant;

fn d() -> Detector {
    Detector::with_builtin()
}

fn risk(h: &str) -> Option<Risk> {
    d().analyze_host(h, None).map(|f| f.risk)
}

fn lookalike(h: &str) -> Option<String> {
    d().analyze_host(h, None).and_then(|f| f.lookalike_of)
}

#[test]
fn real_domains_are_clean() {
    for h in [
        "google.com", "mail.google.com", "www.garantibbva.com.tr", "github.com", "trendyol.com",
        "bücher.de", "türkiye.com.tr", "яндекс.рф", "site123.example.com", "1password.com",
    ] {
        assert_eq!(d().analyze_host(h, None).map(|f| f.risk).filter(|r| *r > Risk::Info), None, "{h}");
    }
}

#[test]
fn punycode_mixed_script() {
    let f = d().analyze_host("xn--pple-43d.com", None).unwrap();
    assert_eq!(f.risk, Risk::High);
    assert_eq!(f.unicode_host, "аpple.com");
    assert_eq!(f.lookalike_of.as_deref(), Some("apple.com"));
    assert!(f.reasons.iter().any(|r| matches!(r, Reason::MixedScript { .. })));
    let tr = f.messages(Lang::Tr).join(" | ");
    let en = f.messages(Lang::En).join(" | ");
    assert!(tr.contains("Latin + Kiril"), "{tr}");
    assert!(en.contains("Latin + Cyrillic") && en.contains("Visually identical to \"apple.com\""), "{en}");
}

#[test]
fn whole_script_cyrillic() {
    assert_eq!(risk("аррӏе.com"), Some(Risk::High));
    assert_eq!(risk(&punycode::to_ascii("аррӏе.com")), Some(Risk::High));
}

#[test]
fn skeleton_hits() {
    assert_eq!(lookalike("paypa1.com").as_deref(), Some("paypal.com"));
    assert_eq!(lookalike("g00gle.com").as_deref(), Some("google.com"));
    assert_eq!(lookalike("rnicrosoft.com").as_deref(), Some("microsoft.com"));
    assert_eq!(lookalike("ısbank.com.tr").as_deref(), Some("isbank.com.tr"));
    assert_eq!(lookalike("ｇｏｏｇｌｅ.com").as_deref(), Some("google.com"));
    assert_eq!(risk("paypa1.com"), Some(Risk::High));
}

#[test]
fn typosquatting() {
    assert_eq!(risk("gooogle.com"), Some(Risk::Medium));
    assert_eq!(lookalike("trendoyl.com").as_deref(), Some("trendyol.com"));
    assert_eq!(lookalike("hepsiburda.com").as_deref(), Some("hepsiburada.com"));
}

#[test]
fn brand_embedding() {
    assert_eq!(risk("paypal-login.com"), Some(Risk::Medium));
    assert_eq!(risk("apple.com.secure-verify.net"), Some(Risk::Medium));
    assert_eq!(risk("live-stream.tv"), None);
}

#[test]
fn at_sign_trick() {
    let f = d().analyze_text("https://garantibbva.com.tr@kotu.site/giris");
    assert_eq!(f.len(), 1);
    assert!(f[0].reasons.iter().any(|r| matches!(r, Reason::AtSign { .. })));
}

#[test]
fn query_params_ignored() {
    assert!(d().analyze_text("https://google.com/?next=https://paypa1.com").is_empty());
}

#[test]
fn whitelist() {
    let mut det = d();
    det.trust("gooogle.com");
    assert!(det.analyze_host("gooogle.com", None).is_none());
}

#[test]
fn performance_thousands_of_urls() {
    let det = d();
    let mut text = String::new();
    for i in 0..5000 {
        text.push_str(&format!("https://site{i}.example.com/path/{i}?q={i}\n"));
        if i % 100 == 0 {
            text.push_str("https://xn--pple-43d.com/login\nhttps://paypa1.com/x\n");
        }
    }
    let t = Instant::now();
    let f = det.analyze_text(&text);
    let ms = t.elapsed().as_millis();
    println!("5000 URL: {ms} ms, {} bulgu", f.len());
    assert_eq!(f.len(), 2);
    assert!(ms < 500, "{ms} ms");
}

#[test]
fn tiered_index() {
    let mut det = d();
    // Liste başı: tüm kurallar
    det.add_popular_with_limit(["1,examplebank.com"], usize::MAX);
    // Liste sonu: yalnızca görsel ikiz kontrolü
    det.add_popular_with_limit(["999999,tailbrand.com"], 0);

    assert_eq!(det.analyze_host("exarnplebank.com", None).map(|f| f.risk), Some(Risk::High));
    assert_eq!(det.analyze_host("examplebnak.com", None).map(|f| f.risk), Some(Risk::Medium));

    assert_eq!(det.analyze_host("taiIbrand.com", None).map(|f| f.risk), None); // I → i, ikiz değil
    assert_eq!(det.analyze_host("tai1brand.com", None).map(|f| f.risk), Some(Risk::High));
    assert_eq!(det.analyze_host("tailbrnad.com", None), None); // yazım hatası kuralı uygulanmaz
    assert!(det.analyze_host("tailbrand.com", None).is_none());
}

#[test]
fn priority_vs_popular() {
    let mut det = d();
    det.add_popular(["yerelmarket.com"]);
    det.add_priority(["yerelbankam.com.tr"]);
    // Büyük liste: yalnızca görsel ikiz
    assert_eq!(det.analyze_host("yere1market.com", None).map(|f| f.risk), Some(Risk::High));
    assert_eq!(det.analyze_host("yerelmakret.com", None), None);
    // Öncelikli hedef: yazım hatası, farklı uzantıda bile
    assert_eq!(det.analyze_host("yerelbnakam.com", None).map(|f| f.risk), Some(Risk::Medium));
    assert_eq!(det.analyze_host("yerelbankam-giris.com", None).map(|f| f.risk), Some(Risk::Medium));
}

#[test]
fn country_second_level_suffixes() {
    for h in ["google.com.gt", "google.com.qa", "amazon.co.th", "mail.microsoft", "live-stream.tv"] {
        assert!(d().analyze_host(h, None).map_or(true, |f| f.risk == Risk::Info), "{h}");
    }
}
