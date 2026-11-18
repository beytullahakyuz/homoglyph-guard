//! Dil desteği: uyarı nedenleri yapılandırılmış kodlardır, metin burada üretilir.
//! Yeni bir dil eklemek için [`Lang`]'a varyant ekleyip `match`leri tamamlayın.

use crate::scripts::Script;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
pub enum Lang {
    Tr,
    #[default]
    En,
}

impl Lang {
    /// "tr_TR.UTF-8", "tr-TR", "en-US" gibi yerel ayar dizelerinden dil seçer.
    pub fn from_locale(locale: &str) -> Lang {
        if locale.to_ascii_lowercase().starts_with("tr") {
            Lang::Tr
        } else {
            Lang::En
        }
    }

    pub fn code(self) -> &'static str {
        match self {
            Lang::Tr => "tr",
            Lang::En => "en",
        }
    }

    pub fn from_code(code: &str) -> Option<Lang> {
        match code.trim() {
            "tr" => Some(Lang::Tr),
            "en" => Some(Lang::En),
            _ => None,
        }
    }
}

impl Script {
    pub fn name(self, lang: Lang) -> &'static str {
        if lang == Lang::Tr {
            return self.name_tr();
        }
        match self {
            Script::Common => "Common",
            Script::Latin => "Latin",
            Script::Greek => "Greek",
            Script::Cyrillic => "Cyrillic",
            Script::Armenian => "Armenian",
            Script::Hebrew => "Hebrew",
            Script::Arabic => "Arabic",
            Script::Thai => "Thai",
            Script::Han => "Han (Chinese)",
            Script::Hiragana => "Hiragana",
            Script::Katakana => "Katakana",
            Script::Hangul => "Hangul",
            Script::Fullwidth => "Fullwidth",
            Script::Other => "Other",
        }
    }
}

/// Bir bulgunun nedeni. Metne çevirmek için [`Reason::message`].
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "kind", rename_all = "snake_case"))]
pub enum Reason {
    BadPunycode,
    Fullwidth { label: String },
    MixedScript { label: String, scripts: Vec<Script> },
    WholeScript { label: String, script: Script },
    Lookalike { target: String },
    Typo { target: String },
    BrandEmbed { brand: String, actual: String },
    AtSign { shown: String, actual: String },
    Idn,
}

impl Reason {
    pub fn message(&self, lang: Lang) -> String {
        use Reason::*;
        match (self, lang) {
            (BadPunycode, Lang::Tr) => "Bozuk / geçersiz punycode etiketi".into(),
            (BadPunycode, Lang::En) => "Malformed / invalid punycode label".into(),

            (Fullwidth { label }, Lang::Tr) => format!("\"{label}\" tam genişlik (fullwidth) karakter içeriyor"),
            (Fullwidth { label }, Lang::En) => format!("\"{label}\" contains fullwidth characters"),

            (MixedScript { label, scripts }, l) => {
                let names = scripts.iter().map(|s| s.name(l)).collect::<Vec<_>>().join(" + ");
                match l {
                    Lang::Tr => format!("\"{label}\" etiketinde karışık alfabe: {names}"),
                    Lang::En => format!("Mixed scripts in \"{label}\": {names}"),
                }
            }

            (WholeScript { label, script }, Lang::Tr) => format!(
                "\"{label}\" tamamen Latin harflerine benzeyen {} harflerden oluşuyor",
                script.name(Lang::Tr)
            ),
            (WholeScript { label, script }, Lang::En) => format!(
                "\"{label}\" is made entirely of {} letters that look like Latin",
                script.name(Lang::En)
            ),

            (Lookalike { target }, Lang::Tr) => format!("\"{target}\" adresine görsel olarak birebir benziyor"),
            (Lookalike { target }, Lang::En) => format!("Visually identical to \"{target}\""),

            (Typo { target }, Lang::Tr) => format!("\"{target}\" adresinin yazım hatalı hali olabilir"),
            (Typo { target }, Lang::En) => format!("Possible misspelling of \"{target}\""),

            (BrandEmbed { brand, actual }, Lang::Tr) => {
                format!("\"{brand}\" markası ilgisiz bir alan adında geçiyor (gerçek alan: {actual})")
            }
            (BrandEmbed { brand, actual }, Lang::En) => {
                format!("Brand \"{brand}\" used in an unrelated domain (real domain: {actual})")
            }

            (AtSign { shown, actual }, Lang::Tr) => {
                format!("URL'de \"@\" hilesi: görünen \"{shown}\", gerçek hedef \"{actual}\"")
            }
            (AtSign { shown, actual }, Lang::En) => {
                format!("\"@\" trick in URL: shows \"{shown}\", actually goes to \"{actual}\"")
            }

            (Idn, Lang::Tr) => "Uluslararası alan adı (IDN / punycode)".into(),
            (Idn, Lang::En) => "Internationalized domain name (IDN / punycode)".into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locale_detection() {
        assert_eq!(Lang::from_locale("tr_TR.UTF-8"), Lang::Tr);
        assert_eq!(Lang::from_locale("tr-TR"), Lang::Tr);
        assert_eq!(Lang::from_locale("en-US"), Lang::En);
        assert_eq!(Lang::from_locale("de_DE"), Lang::En);
    }

    #[test]
    fn messages() {
        let r = Reason::MixedScript { label: "аpple".into(), scripts: vec![Script::Latin, Script::Cyrillic] };
        assert_eq!(r.message(Lang::Tr), "\"аpple\" etiketinde karışık alfabe: Latin + Kiril");
        assert_eq!(r.message(Lang::En), "Mixed scripts in \"аpple\": Latin + Cyrillic");
    }
}
