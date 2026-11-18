//! "Hakkında" penceresinin içeriği. Geliştirici bilgilerini buradan düzenleyin.

use homoglyph_core::Lang;
use serde::Serialize;

// ---- Düzenlenecek alanlar ----------------------------------------------
const DEVELOPER: &str = "Beytullah Akyüz";
const EMAIL: &str = "beytullahakyuz@hotmail.com.tr";
const WEBSITE: &str = "https://github.com/beytullahakyuz/homoglyph-guard";
const LICENSE: &str = "MIT";
const YEAR: &str = "2026";
// -------------------------------------------------------------------------

#[derive(Serialize, Clone)]
pub struct AboutInfo {
    pub lang: Lang,
    pub name: &'static str,
    pub version: &'static str,
    pub description: &'static str,
    pub developer: &'static str,
    pub email: &'static str,
    pub website: &'static str,
    pub license: &'static str,
    pub copyright: String,
    pub popular_count: usize,
}

pub fn info(lang: Lang, popular_count: usize) -> AboutInfo {
    let description = match lang {
        Lang::Tr => {
            "Panoya kopyalanan URL'lerdeki sahte alan adlarını tespit eder: homoglyph \
             (benzer karakter), punycode, karışık alfabe, yazım hatası taklidi ve marka \
             taklidi. Tüm denetim cihazınızda çevrimdışı yapılır; hiçbir veri dışarı gönderilmez."
        }
        Lang::En => {
            "Detects fake domain names in URLs copied to the clipboard: homoglyphs \
             (look-alike characters), punycode, mixed scripts, typosquatting and brand \
             impersonation. All checks run offline on your device; no data is ever sent out."
        }
    };
    AboutInfo {
        lang,
        name: "Homoglyph Guard",
        version: env!("CARGO_PKG_VERSION"),
        description,
        developer: DEVELOPER,
        email: EMAIL,
        website: WEBSITE,
        license: LICENSE,
        copyright: format!("© {YEAR} {DEVELOPER}"),
        popular_count,
    }
}
