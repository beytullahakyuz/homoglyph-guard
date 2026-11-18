//! Tepsi menüsü ve bildirim metinleri. Uyarı nedenlerinin metinleri core/src/i18n.rs'de,
//! uyarı penceresinin metinleri ui/i18n.js'de.

use homoglyph_core::Lang;

pub struct T {
    pub pause: &'static str,
    pub resume: &'static str,
    pub show_last: &'static str,
    pub quit: &'static str,
    pub about: &'static str,
    pub lang_auto: &'static str,
    pub tooltip: &'static str,
    pub notify_title: &'static str,
}

const TR: T = T {
    pause: "İzlemeyi duraklat",
    resume: "İzlemeyi sürdür",
    show_last: "Son uyarıyı göster",
    quit: "Çıkış",
    about: "Hakkında",
    lang_auto: "Otomatik (sistem dili)",
    tooltip: "Homoglyph Guard: pano izleniyor",
    notify_title: "Şüpheli alan adı",
};

const EN: T = T {
    pause: "Pause monitoring",
    resume: "Resume monitoring",
    show_last: "Show last alert",
    quit: "Quit",
    about: "About",
    lang_auto: "Automatic (system language)",
    tooltip: "Homoglyph Guard: watching clipboard",
    notify_title: "Suspicious domain",
};

impl T {
    pub fn get(lang: Lang) -> &'static T {
        match lang {
            Lang::Tr => &TR,
            Lang::En => &EN,
        }
    }
}
