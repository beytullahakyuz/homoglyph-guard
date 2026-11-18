//! Pano izleyici: ayrı bir thread'de panoyu okur, içerik değiştiğinde geri çağırır.
//!
//! Neden yoklama (polling)? macOS'ta pano için olay API'si yok (yalnızca changeCount),
//! Wayland'da arka plan uygulamalarına olay gelmeyebilir. 400 ms'lik yoklama ve hash
//! karşılaştırması üç platformda da güvenilir ve CPU kullanımı ihmal edilebilir düzeyde.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

const INTERVAL: Duration = Duration::from_millis(400);
const MAX_LEN: usize = 2 * 1024 * 1024;

fn hash(s: &str) -> u64 {
    let mut h = DefaultHasher::new();
    s.hash(&mut h);
    h.finish()
}

pub fn spawn_watcher<F>(paused: Arc<AtomicBool>, on_text: F)
where
    F: Fn(String) + Send + 'static,
{
    thread::Builder::new()
        .name("clipboard-watcher".into())
        .spawn(move || {
            let mut cb = loop {
                match arboard::Clipboard::new() {
                    Ok(c) => break c,
                    Err(e) => {
                        eprintln!("Panoya erişilemedi, tekrar denenecek: {e}");
                        thread::sleep(Duration::from_secs(2));
                    }
                }
            };
            // Açılıştaki mevcut içerik için uyarı verme.
            let mut last = cb.get_text().ok().map(|t| hash(&t));
            loop {
                thread::sleep(INTERVAL);
                if paused.load(Ordering::Relaxed) {
                    continue;
                }
                let Ok(mut text) = cb.get_text() else { continue };
                if text.is_empty() {
                    continue;
                }
                let h = hash(&text);
                if last == Some(h) {
                    continue;
                }
                last = Some(h);
                if text.len() > MAX_LEN {
                    let mut cut = MAX_LEN;
                    while !text.is_char_boundary(cut) {
                        cut -= 1;
                    }
                    text.truncate(cut);
                }
                on_text(text);
            }
        })
        .expect("pano thread'i başlatılamadı");
}
