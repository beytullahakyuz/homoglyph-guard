//! Karıştırılabilir karakter tablosu ve UTS #39 "iskelet" (skeleton) üretimi.
//!
//! Yerleşik tablo en sık kullanılan sahtecilik karakterlerini kapsar. Tam liste için
//! <https://www.unicode.org/Public/security/latest/confusables.txt> dosyası
//! [`Confusables::merge_unicode_data`] ile eklenebilir.

use std::collections::HashMap;

/// hedef ASCII → ona benzeyen karakterler
const BUILTIN: &[(&str, &str)] = &[
    ("a", "аɑαàáâãäåāăąǎȁȃạảấầẩẫậắằẳẵặ"),
    ("b", "Ьɓḃḅ"),
    ("c", "сϲçćĉċčƈ"),
    ("d", "ԁďđɗḋḍ"),
    ("e", "еεèéêëēĕėęěȅȇẹẻẽếềểễệҽ"),
    ("g", "ɡĝğġģǥǧ"),
    ("h", "һĥħḣḥ"),
    ("i", "іιıɩɪìíîïĩīĭįǐȉȋịỉї¡"),
    ("j", "јĵʝ"),
    ("k", "κкķƙḳ"),
    ("l", "ӏłĺļľŀƚ|1"),
    ("m", "мṁṃ"),
    ("n", "ηпñńņňŉṅṇ"),
    ("o", "оοσօòóôõöøōŏőǒȍȏọỏốồổỗộớờởỡợ0"),
    ("p", "рρṗ"),
    ("q", "ԛզ"),
    ("r", "гŕŗřȑȓṙṛ"),
    ("s", "ѕśŝşšșṡṣ5"),
    ("t", "τтţťŧțṫṭ"),
    ("u", "υսùúûüũūŭůűųǔȕȗụủứừửữự"),
    ("v", "νѵ"),
    ("w", "ԝѡŵẁẃẅ"),
    ("x", "хχ"),
    ("y", "уүýÿŷỳỵỷỹ"),
    ("z", "źżžẓ"),
];

#[derive(Debug, Clone)]
pub struct Confusables {
    map: HashMap<char, String>,
}

impl Default for Confusables {
    fn default() -> Self {
        Self::builtin()
    }
}

impl Confusables {
    pub fn builtin() -> Self {
        let mut map = HashMap::new();
        for (target, sources) in BUILTIN {
            for c in sources.chars() {
                map.insert(c, (*target).to_string());
            }
        }
        Confusables { map }
    }

    /// `confusables.txt` biçimi: `0430 ;\t0061 ;\tMA\t# ...`
    pub fn merge_unicode_data(&mut self, text: &str) -> usize {
        let mut added = 0;
        for line in text.lines() {
            let line = line.split('#').next().unwrap_or("");
            let mut parts = line.split(';');
            let (Some(src), Some(dst)) = (parts.next(), parts.next()) else { continue };
            let src = src.trim();
            if src.is_empty() || src.contains(' ') {
                continue;
            }
            let Some(sc) = u32::from_str_radix(src, 16).ok().and_then(char::from_u32) else { continue };
            let target: String = dst
                .split_whitespace()
                .filter_map(|h| u32::from_str_radix(h, 16).ok().and_then(char::from_u32))
                .collect();
            if target.is_empty() {
                continue;
            }
            if let std::collections::hash_map::Entry::Vacant(e) = self.map.entry(sc) {
                e.insert(target.to_lowercase());
                added += 1;
            }
        }
        added
    }

    pub fn is_mapped(&self, c: char) -> bool {
        self.map.contains_key(&c)
    }

    /// Görsel olarak aynı görünen iki dizenin iskeleti eşittir.
    pub fn skeleton(&self, s: &str) -> String {
        let mut out = String::with_capacity(s.len());
        for mut c in s.chars() {
            // Tam genişlik ASCII → normal ASCII
            if ('\u{FF01}'..='\u{FF5E}').contains(&c) {
                c = char::from_u32(c as u32 - 0xFEE0).unwrap_or(c);
            }
            match self.map.get(&c) {
                Some(m) => out.push_str(m),
                None => out.push(c),
            }
        }
        out.to_lowercase().replace("rn", "m").replace("vv", "w")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skeletons() {
        let c = Confusables::builtin();
        assert_eq!(c.skeleton("аррӏе.com"), "apple.com");
        assert_eq!(c.skeleton("paypa1.com"), "paypal.com");
        assert_eq!(c.skeleton("rnicrosoft.com"), "microsoft.com");
        assert_eq!(c.skeleton("ｇｏｏｇｌｅ.com"), "google.com");
        assert_eq!(c.skeleton("ısbank.com.tr"), "isbank.com.tr");
    }

    #[test]
    fn merge() {
        let mut c = Confusables::builtin();
        let n = c.merge_unicode_data("13A0 ;\t0044 ;\tMA\t# ( Ꭰ → D )\n0430 ;\t0061 ;\tMA\n");
        assert_eq!(n, 1);
        assert_eq!(c.skeleton("Ꭰ"), "d");
    }
}
