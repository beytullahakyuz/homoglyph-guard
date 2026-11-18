//! Alan adları için yeterli Unicode alfabe (script) sınıflandırması.

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
pub enum Script {
    Common,
    Latin,
    Greek,
    Cyrillic,
    Armenian,
    Hebrew,
    Arabic,
    Thai,
    Han,
    Hiragana,
    Katakana,
    Hangul,
    Fullwidth,
    Other,
}

impl Script {
    pub fn name_tr(self) -> &'static str {
        match self {
            Script::Common => "Ortak",
            Script::Latin => "Latin",
            Script::Greek => "Yunan",
            Script::Cyrillic => "Kiril",
            Script::Armenian => "Ermeni",
            Script::Hebrew => "İbrani",
            Script::Arabic => "Arap",
            Script::Thai => "Tay",
            Script::Han => "Çin (Han)",
            Script::Hiragana => "Hiragana",
            Script::Katakana => "Katakana",
            Script::Hangul => "Hangul",
            Script::Fullwidth => "Tam genişlik",
            Script::Other => "Diğer",
        }
    }
    fn bit(self) -> u32 {
        1 << (self as u32)
    }
}

pub fn script_of(c: char) -> Script {
    let cp = c as u32;
    match cp {
        0x2D | 0x2E | 0x30..=0x39 => Script::Common,
        0x41..=0x5A | 0x61..=0x7A => Script::Latin,
        0xD7 | 0xF7 => Script::Common,
        0xC0..=0x2AF | 0x1D00..=0x1D7F | 0x1E00..=0x1EFF => Script::Latin,
        0x300..=0x36F => Script::Common, // birleşik işaretler
        0x370..=0x3FF | 0x1F00..=0x1FFF => Script::Greek,
        0x400..=0x52F | 0x2DE0..=0x2DFF | 0xA640..=0xA69F => Script::Cyrillic,
        0x530..=0x58F => Script::Armenian,
        0x590..=0x5FF => Script::Hebrew,
        0x600..=0x6FF | 0x750..=0x77F => Script::Arabic,
        0xE00..=0xE7F => Script::Thai,
        0x3400..=0x4DBF | 0x4E00..=0x9FFF => Script::Han,
        0x3040..=0x309F => Script::Hiragana,
        0x30A0..=0x30FF => Script::Katakana,
        0x1100..=0x11FF | 0xAC00..=0xD7AF => Script::Hangul,
        0xFF01..=0xFF5E => Script::Fullwidth,
        _ => Script::Other,
    }
}

/// Bir etiketteki (Common hariç) alfabeler — bit kümesi olarak.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ScriptSet(u32);

impl ScriptSet {
    pub fn of(label: &str) -> Self {
        let mut s = 0;
        for c in label.chars() {
            let sc = script_of(c);
            if sc != Script::Common {
                s |= sc.bit();
            }
        }
        ScriptSet(s)
    }
    pub fn len(self) -> u32 {
        self.0.count_ones()
    }
    pub fn is_empty(self) -> bool {
        self.0 == 0
    }
    pub fn contains(self, s: Script) -> bool {
        self.0 & s.bit() != 0
    }
    pub fn single(self) -> Option<Script> {
        if self.len() != 1 {
            return None;
        }
        ALL.iter().copied().find(|s| self.contains(*s))
    }
    pub fn to_vec(self) -> Vec<Script> {
        ALL.iter().copied().filter(|s| self.contains(*s)).collect()
    }
    pub fn names_tr(self) -> String {
        ALL.iter()
            .filter(|s| self.contains(**s))
            .map(|s| s.name_tr())
            .collect::<Vec<_>>()
            .join(" + ")
    }

    /// UTS #39 "Highly Restrictive" seviyesine yakın: Japonca ve Korece kombinasyonlarına izin ver.
    pub fn is_mixed(self) -> bool {
        if self.len() <= 1 {
            return false;
        }
        let jpan = Script::Latin.bit() | Script::Han.bit() | Script::Hiragana.bit() | Script::Katakana.bit();
        let kore = Script::Latin.bit() | Script::Han.bit() | Script::Hangul.bit();
        !(self.0 & !jpan == 0 || self.0 & !kore == 0)
    }
}

const ALL: [Script; 14] = [
    Script::Common,
    Script::Latin,
    Script::Greek,
    Script::Cyrillic,
    Script::Armenian,
    Script::Hebrew,
    Script::Arabic,
    Script::Thai,
    Script::Han,
    Script::Hiragana,
    Script::Katakana,
    Script::Hangul,
    Script::Fullwidth,
    Script::Other,
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mixed() {
        assert!(ScriptSet::of("аpple").is_mixed());
        assert!(!ScriptSet::of("türkiye").is_mixed());
        assert!(!ScriptSet::of("яндекс").is_mixed());
        assert!(!ScriptSet::of("東京tokyo").is_mixed());
        assert_eq!(ScriptSet::of("аpple").names_tr(), "Latin + Kiril");
    }
}
