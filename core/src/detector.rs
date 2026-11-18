use std::collections::hash_map::RandomState;
use std::collections::{HashMap, HashSet};
use std::hash::BuildHasher;

use crate::confusables::Confusables;
use crate::extract::{extract_hosts, normalize_host};
use crate::i18n::{Lang, Reason};
use crate::punycode;
use crate::scripts::{Script, ScriptSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
pub enum Risk {
    Info,
    Medium,
    High,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Finding {
    pub raw_host: String,
    pub unicode_host: String,
    pub ascii_host: String,
    pub registrable: String,
    pub risk: Risk,
    pub reasons: Vec<Reason>,
    pub lookalike_of: Option<String>,
}

impl Finding {
    /// Nedenleri istenen dilde metne çevirir.
    pub fn messages(&self, lang: Lang) -> Vec<String> {
        self.reasons.iter().map(|r| r.message(lang)).collect()
    }
}

/// İkinci seviye genel sonekler. Tam doğruluk için Public Suffix List eklenebilir.
const TWO_LEVEL_SUFFIXES: &[&str] = &[
    "com.tr", "gov.tr", "org.tr", "net.tr", "edu.tr", "gen.tr", "bel.tr", "k12.tr", "av.tr",
    "dr.tr", "bbs.tr", "tv.tr", "web.tr", "info.tr", "biz.tr", "name.tr", "co.uk", "org.uk",
    "ac.uk", "gov.uk", "com.au", "net.au", "org.au", "co.jp", "ne.jp", "or.jp", "co.kr",
    "com.br", "com.cn", "com.mx", "co.nz", "co.za", "co.in", "com.sg", "com.hk", "com.ar",
    "com.de", "co.il", "com.ua",
];

/// Marka taklidi kuralında yanlış alarma yol açan genel kelimeler.
const GENERIC_TOKENS: &[&str] = &[
    "live", "mail", "news", "shop", "store", "online", "cloud", "office", "home", "info", "apps",
    "play", "drive", "docs", "maps", "blog", "help", "support", "login", "secure", "account",
    "bank", "site", "media", "video", "music", "tech", "sites", "static", "images", "files",
    "about", "world", "game", "games", "example",
];

pub const BUILTIN_POPULAR: &str = include_str!("../data/popular_domains.txt");
const TRANCO_MARKER: &str = "# --- Tranco";

/// Yerleşik Tranco bölümünden yazım hatası / marka taklidi kurallarına dahil edilen satır sayısı.
/// Ölçüm (Tranco 10.001–30.000 arası 20.000 gerçek site): 0 → orta risk yanlış alarm %0,23;
/// 1000 → %0,94; 10.000 → %2,66. Yüksek risk kuralları her durumda %0,01.
pub const BUILTIN_TRANCO_FULL: usize = 0;

/// Yazım hatası ve marka taklidi kuralları için en kısa ad uzunluğu.
const MIN_FUZZY_LEN: usize = 5;

/// Ülke uzantılarında sık kullanılan ikinci seviye etiketler: com.gt, co.th, gob.mx, ac.id …
const GENERIC_SECOND_LEVEL: &[&str] =
    &["com", "co", "org", "net", "gov", "gob", "edu", "ac", "or", "ne", "go", "mil", "nic", "info"];

pub fn registrable_of(host: &str) -> String {
    let labels: Vec<&str> = host.split('.').collect();
    if labels.len() <= 2 {
        return host.to_string();
    }
    let (sld, tld) = (labels[labels.len() - 2], labels[labels.len() - 1]);
    let last2 = format!("{sld}.{tld}");
    let cc_second_level = tld.len() == 2 && GENERIC_SECOND_LEVEL.contains(&sld);
    let take = if cc_second_level || TWO_LEVEL_SUFFIXES.contains(&last2.as_str()) { 3 } else { 2 };
    labels[labels.len() - take..].join(".")
}

/// "garantibbva.com.tr" -> "com.tr"
pub fn suffix_of(registrable: &str) -> &str {
    registrable.split_once('.').map_or("", |(_, s)| s)
}

/// `"garantibbva.com.tr"` → `"garantibbva"`
pub fn name_of(registrable: &str) -> &str {
    registrable.split('.').next().unwrap_or(registrable)
}

fn deletes(s: &str) -> impl Iterator<Item = String> + '_ {
    std::iter::once(s.to_string())
        .chain((0..s.len()).filter(|&i| s.is_char_boundary(i)).map(move |i| {
            let mut v = String::with_capacity(s.len());
            v.push_str(&s[..i]);
            let next = s[i..].chars().next().map_or(0, char::len_utf8);
            v.push_str(&s[i + next..]);
            v
        }))
}

/// Optimal string alignment mesafesi (bitişik yer değiştirme dahil).
pub fn osa_distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let (n, m) = (a.len(), b.len());
    let mut d = vec![vec![0usize; m + 1]; n + 1];
    for (i, row) in d.iter_mut().enumerate() {
        row[0] = i;
    }
    for j in 0..=m {
        d[0][j] = j;
    }
    for i in 1..=n {
        for j in 1..=m {
            let cost = usize::from(a[i - 1] != b[j - 1]);
            let mut v = (d[i - 1][j] + 1).min(d[i][j - 1] + 1).min(d[i - 1][j - 1] + cost);
            if i > 1 && j > 1 && a[i - 1] == b[j - 2] && a[i - 2] == b[j - 1] {
                v = v.min(d[i - 2][j - 2] + 1);
            }
            d[i][j] = v;
        }
    }
    d[n][m]
}


/// İki kademeli indeks:
/// - Tüm liste: "popüler mi?" ve "görsel ikiz mi?" (hash'lenmiş, az bellek)
/// - Listenin başı: yazım hatası ve marka taklidi (silme-varyantı indeksi)
pub struct Detector {
    pub confusables: Confusables,
    /// Rastgele anahtarlı SipHash: dışarıdan hash çakışması üretilemez.
    hasher: RandomState,
    popular: HashSet<u64>,
    /// iskelet hash'i → arena'daki alan adı indeksi
    skeleton_index: HashMap<u64, u32>,
    arena: String,
    spans: Vec<(u32, u16)>,
    whitelist: HashSet<String>,
    name_index: HashMap<String, Vec<String>>,
    delete_index: HashMap<String, Vec<String>>,
    full_indexed: usize,
    /// Elle seçilmiş hedefler: yazım hatası kuralı uzantı farkına bakmadan uygulanır.
    priority: HashSet<String>,
}

impl Detector {
    pub fn new<'a>(popular: impl IntoIterator<Item = &'a str>, confusables: Confusables) -> Self {
        let mut d = Detector {
            confusables,
            hasher: RandomState::new(),
            popular: HashSet::new(),
            skeleton_index: HashMap::new(),
            arena: String::new(),
            spans: Vec::new(),
            whitelist: HashSet::new(),
            name_index: HashMap::new(),
            delete_index: HashMap::new(),
            full_indexed: 0,
            priority: HashSet::new(),
        };
        // Yerleşik liste her zaman tam indekslenir ve sınıra sayılmaz.
        d.add_popular_with_limit(popular, usize::MAX);
        d.full_indexed = 0;
        d.priority = d.name_index.keys().cloned().collect();
        d
    }

    /// Yerleşik liste: elle seçilmiş hedefler (bankalar, ödeme, e-posta…) tüm kurallarla,
    /// Tranco bölümü ise ilk [`BUILTIN_TRANCO_FULL`] satırı tüm kurallarla, gerisi yalnızca
    /// görsel ikiz kontrolüyle kullanılır.
    pub fn with_builtin() -> Self {
        Self::with_builtin_full_limit(BUILTIN_TRANCO_FULL)
    }

    pub fn with_builtin_full_limit(tranco_full: usize) -> Self {
        let (curated, tranco) = match BUILTIN_POPULAR.find(TRANCO_MARKER) {
            Some(i) => BUILTIN_POPULAR.split_at(i),
            None => (BUILTIN_POPULAR, ""),
        };
        let mut d = Self::new(curated.lines(), Confusables::builtin());
        d.add_popular_with_limit(tranco.lines(), tranco_full);
        d.full_indexed = 0;
        d
    }

    fn h(&self, s: &str) -> u64 {
        self.hasher.hash_one(s)
    }

    fn arena_get(&self, idx: u32) -> &str {
        let (start, len) = self.spans[idx as usize];
        &self.arena[start as usize..start as usize + len as usize]
    }

    /// Büyük popüler listeler (Tranco vb.): yalnızca "popüler mi?" ve "görsel ikiz mi?"
    /// kontrollerinde kullanılır. Satır başına bir alan adı; "1,google.com" da kabul edilir.
    pub fn add_popular<'a>(&mut self, domains: impl IntoIterator<Item = &'a str>) {
        let before = self.full_indexed;
        self.add_popular_with_limit(domains, before);
    }

    /// Kritik hedefler (banka, ödeme, e-posta…): yazım hatası ve marka taklidi dahil tüm kurallar,
    /// farklı uzantıdaki yazım hataları da yakalanır.
    pub fn add_priority<'a>(&mut self, domains: impl IntoIterator<Item = &'a str>) {
        let before: HashSet<String> = self.name_index.keys().cloned().collect();
        self.add_popular_with_limit(domains, usize::MAX);
        let new: Vec<String> = self.name_index.keys().filter(|k| !before.contains(*k)).cloned().collect();
        self.priority.extend(new);
    }

    /// `full_index_limit`: yazım hatası / marka taklidi kurallarına dahil edilecek en fazla
    /// alan adı (bu çağrılar boyunca toplam). Geri kalanlar yalnızca görsel ikiz kontrolünde kullanılır.
    pub fn add_popular_with_limit<'a>(
        &mut self,
        domains: impl IntoIterator<Item = &'a str>,
        full_index_limit: usize,
    ) {
        for line in domains {
            let line = line.trim();
            let dom = line.rsplit(',').next().unwrap_or(line).trim().to_lowercase();
            if dom.is_empty() || dom.starts_with('#') || !dom.contains('.') || dom.len() > 253 {
                continue;
            }
            let reg = registrable_of(&dom);
            if !self.popular.insert(self.h(&reg)) {
                continue;
            }
            let idx = self.spans.len() as u32;
            self.spans.push((self.arena.len() as u32, reg.len() as u16));
            self.arena.push_str(&reg);
            let sk = self.h(&self.confusables.skeleton(&reg));
            self.skeleton_index.entry(sk).or_insert(idx);

            if self.full_indexed >= full_index_limit {
                continue;
            }
            self.full_indexed += 1;
            let name = name_of(&reg).to_string();
            self.name_index.entry(name.clone()).or_default().push(reg.clone());
            if name.chars().count() >= MIN_FUZZY_LEN {
                for v in deletes(&name) {
                    let e = self.delete_index.entry(v).or_default();
                    if !e.contains(&name) {
                        e.push(name.clone());
                    }
                }
            }
        }
    }

    pub fn popular_count(&self) -> usize {
        self.popular.len()
    }

    pub fn trust(&mut self, registrable: &str) {
        self.whitelist.insert(registrable.to_lowercase());
    }

    pub fn whitelist(&self) -> impl Iterator<Item = &String> {
        self.whitelist.iter()
    }

    /// Yazım hatası eşleşmesi: (benzediği ad, benzediği kayıtlı alan adı)
    fn typo_match(&self, name: &str, suffix: &str) -> Option<String> {
        if name.chars().count() < MIN_FUZZY_LEN || self.name_index.contains_key(name) {
            return None;
        }
        let bare = name.replace('-', "");
        for v in deletes(name) {
            let Some(cands) = self.delete_index.get(&v) else { continue };
            for c in cands {
                // Yalnızca tire farkı (live-stream / livestream) yazım hatası sayılmaz.
                if c == name || c.replace('-', "") == bare || osa_distance(c, name) != 1 {
                    continue;
                }
                let regs = &self.name_index[c];
                // Uzantılar arası eşleşme (stape.io ~ state.gov) yalnızca öncelikli hedeflerde.
                let target = regs
                    .iter()
                    .find(|r| suffix_of(r) == suffix)
                    .or_else(|| self.priority.contains(c).then(|| &regs[0]));
                if let Some(t) = target {
                    return Some(t.clone());
                }
            }
        }
        None
    }

    /// Metindeki tüm alan adlarını denetler; riskliden az riskliye sıralı döner.
    pub fn analyze_text(&self, text: &str) -> Vec<Finding> {
        let mut out: Vec<Finding> = extract_hosts(text, 5000)
            .into_iter()
            .filter_map(|e| self.analyze_host(&e.host, e.userinfo_decoy.as_deref()))
            .collect();
        out.sort_by(|a, b| b.risk.cmp(&a.risk));
        out
    }

    pub fn analyze_host(&self, raw_host: &str, userinfo_decoy: Option<&str>) -> Option<Finding> {
        let host = normalize_host(raw_host)?;
        let mut reasons: Vec<Reason> = Vec::new();
        let mut risk = Risk::Info;
        let mut lookalike: Option<String> = None;
        let mut add = |r: Risk, reason: Reason, reasons: &mut Vec<Reason>| {
            reasons.push(reason);
            if r > risk {
                risk = r;
            }
        };

        // 1) Punycode çözümü
        let unicode = match punycode::to_unicode(&host) {
            Ok(u) => u,
            Err(_) => {
                add(Risk::High, Reason::BadPunycode, &mut reasons);
                host.clone()
            }
        };
        let ascii = punycode::to_ascii(&unicode);
        let reg = registrable_of(&unicode);
        let reg_ascii = registrable_of(&ascii);

        if self.whitelist.contains(&reg_ascii) || self.whitelist.contains(&reg) {
            return None;
        }
        let is_popular = self.popular.contains(&self.h(&reg_ascii));

        // 2) Karışık alfabe, tam genişlik, tüm-alfabe taklidi
        for label in unicode.split('.') {
            let set = ScriptSet::of(label);
            if set.contains(Script::Fullwidth) {
                add(Risk::High, Reason::Fullwidth { label: label.to_string() }, &mut reasons);
            }
            if set.is_mixed() {
                add(Risk::High, Reason::MixedScript { label: label.to_string(), scripts: set.to_vec() }, &mut reasons);
            } else if let Some(s @ (Script::Cyrillic | Script::Greek)) = set.single() {
                if label.chars().all(|c| c.is_ascii() || self.confusables.is_mapped(c)) {
                    add(Risk::High, Reason::WholeScript { label: label.to_string(), script: s }, &mut reasons);
                }
            }
        }

        if !is_popular {
            // 3) Confusable iskelet: popüler bir alan adının görsel ikizi mi?
            let sk = self.h(&self.confusables.skeleton(&reg));
            if let Some(&idx) = self.skeleton_index.get(&sk) {
                let hit = self.arena_get(idx);
                if hit != reg_ascii {
                    lookalike = Some(hit.to_string());
                    add(Risk::High, Reason::Lookalike { target: hit.to_string() }, &mut reasons);
                }
            }

            // 4) Typosquatting
            let name = name_of(&reg_ascii);
            if lookalike.is_none() && !name.starts_with("xn--") {
                if let Some(target) = self.typo_match(name, suffix_of(&reg_ascii)) {
                    add(Risk::Medium, Reason::Typo { target: target.clone() }, &mut reasons);
                    lookalike = Some(target);
                }
            }

            // 5) Marka taklidi: paypal-login.com, apple.com.evil.net
            let tld = ascii.rsplit('.').next().unwrap_or("");
            for t in ascii.split(['.', '-']) {
                // t == tld: .microsoft, .google gibi marka uzantıları
                if t.len() < MIN_FUZZY_LEN || t == name || t == tld || GENERIC_TOKENS.contains(&t) {
                    continue;
                }
                if let Some(brand) = self.name_index.get(t) {
                    if lookalike.is_none() {
                        lookalike = Some(brand[0].clone());
                    }
                    add(
                        Risk::Medium,
                        Reason::BrandEmbed { brand: t.to_string(), actual: reg_ascii.clone() },
                        &mut reasons,
                    );
                    break;
                }
            }
        }

        // 6) '@' hilesi
        if let Some(d) = userinfo_decoy {
            add(Risk::Medium, Reason::AtSign { shown: d.to_string(), actual: host.clone() }, &mut reasons);
        }

        if reasons.is_empty() && ascii.contains("xn--") {
            add(Risk::Info, Reason::Idn, &mut reasons);
        }
        if reasons.is_empty() {
            return None;
        }
        Some(Finding {
            raw_host: raw_host.to_string(),
            unicode_host: unicode,
            ascii_host: ascii,
            registrable: reg_ascii,
            risk,
            reasons,
            lookalike_of: lookalike,
        })
    }
}
