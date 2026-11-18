#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod about;
mod clipboard;
mod texts;

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, RwLock};

use homoglyph_core::{Confusables, Detector, Finding, Lang, Risk};
use serde::Serialize;
use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, Wry, WindowEvent};
use tauri_plugin_notification::NotificationExt;

use texts::T;

const ALERT: &str = "alert";
const ABOUT: &str = "about";
const ALERT_WIDTH: f64 = 420.0;
const MARGIN: f64 = 16.0;

/// Kullanıcının dil tercihi. `None` = sistem diline göre otomatik.
type LangPref = Option<Lang>;

struct AppState {
    detector: RwLock<Detector>,
    last: Mutex<Vec<Finding>>,
    paused: Arc<AtomicBool>,
    config_dir: PathBuf,
    lang_pref: Mutex<LangPref>,
}

impl AppState {
    fn lang(&self) -> Lang {
        self.lang_pref.lock().unwrap().unwrap_or_else(system_lang)
    }
}

/// Tepsi menüsündeki öğeler — dil değişince metinleri güncellenir.
struct TrayItems {
    pause: MenuItem<Wry>,
    last: MenuItem<Wry>,
    about: MenuItem<Wry>,
    quit: MenuItem<Wry>,
    lang_auto: CheckMenuItem<Wry>,
    lang_tr: CheckMenuItem<Wry>,
    lang_en: CheckMenuItem<Wry>,
}

/// Arayüze giden bulgu: nedenler seçili dilde metne çevrilmiş halde.
#[derive(Serialize, Clone)]
struct FindingView {
    unicode_host: String,
    ascii_host: String,
    registrable: String,
    risk: Risk,
    messages: Vec<String>,
    lookalike_of: Option<String>,
}

#[derive(Serialize, Clone)]
struct AlertPayload {
    lang: Lang,
    findings: Vec<FindingView>,
}

fn system_lang() -> Lang {
    sys_locale::get_locale().map(|l| Lang::from_locale(&l)).unwrap_or_default()
}

fn is_wayland() -> bool {
    cfg!(target_os = "linux") && std::env::var_os("WAYLAND_DISPLAY").is_some()
}

fn payload(state: &AppState) -> AlertPayload {
    let lang = state.lang();
    let findings = state
        .last
        .lock()
        .unwrap()
        .iter()
        .map(|f| FindingView {
            unicode_host: f.unicode_host.clone(),
            ascii_host: f.ascii_host.clone(),
            registrable: f.registrable.clone(),
            risk: f.risk,
            messages: f.messages(lang),
            lookalike_of: f.lookalike_of.clone(),
        })
        .collect();
    AlertPayload { lang, findings }
}

// ---------------- Yapılandırma dosyaları ----------------
//   priority_domains.txt kritik hedefler: tüm kurallar (yazım hatası, marka taklidi dahil)
//   popular_domains.txt  büyük listeler (Tranco): yalnızca görsel ikiz kontrolü
//   confusables.txt      Unicode'un tam tablosu
//   whitelist.txt        "Güvenilir" işaretlenenler (uygulama yazar)
//   language.txt         auto | tr | en

fn build_detector(dir: &PathBuf) -> Detector {
    let mut conf = Confusables::builtin();
    if let Ok(t) = fs::read_to_string(dir.join("confusables.txt")) {
        conf.merge_unicode_data(&t);
    }
    let mut det = Detector::with_builtin();
    det.confusables = conf;
    if let Ok(t) = fs::read_to_string(dir.join("priority_domains.txt")) {
        det.add_priority(t.lines());
    }
    if let Ok(t) = fs::read_to_string(dir.join("popular_domains.txt")) {
        det.add_popular(t.lines());
    }
    if let Ok(t) = fs::read_to_string(dir.join("whitelist.txt")) {
        for l in t.lines().map(str::trim).filter(|l| !l.is_empty()) {
            det.trust(l);
        }
    }
    det
}

fn load_lang_pref(dir: &PathBuf) -> LangPref {
    fs::read_to_string(dir.join("language.txt")).ok().and_then(|s| Lang::from_code(&s))
}

fn save_lang_pref(dir: &PathBuf, pref: LangPref) {
    let _ = fs::create_dir_all(dir);
    let _ = fs::write(dir.join("language.txt"), pref.map_or("auto", Lang::code));
}

// ---------------- Arayüz komutları ----------------

#[tauri::command]
fn current_findings(state: tauri::State<AppState>) -> AlertPayload {
    payload(&state)
}

#[tauri::command]
fn trust(state: tauri::State<AppState>, registrable: String) -> Result<(), String> {
    let mut det = state.detector.write().unwrap();
    det.trust(&registrable);
    let mut list: Vec<String> = det.whitelist().cloned().collect();
    list.sort();
    fs::create_dir_all(&state.config_dir).map_err(|e| e.to_string())?;
    fs::write(state.config_dir.join("whitelist.txt"), list.join("\n") + "\n").map_err(|e| e.to_string())?;
    state.last.lock().unwrap().retain(|f| f.registrable != registrable);
    Ok(())
}

#[tauri::command]
fn hide_alert(app: AppHandle) {
    if let Some(w) = app.get_webview_window(ALERT) {
        let _ = w.hide();
    }
}

#[tauri::command]
fn about_info(state: tauri::State<AppState>) -> about::AboutInfo {
    about::info(state.lang(), state.detector.read().unwrap().popular_count())
}

#[tauri::command]
fn hide_about(app: AppHandle) {
    if let Some(w) = app.get_webview_window(ABOUT) {
        let _ = w.hide();
    }
}

fn show_about(app: &AppHandle) {
    if let Some(w) = app.get_webview_window(ABOUT) {
        let _ = w.emit("lang-changed", ());
        let _ = w.center();
        let _ = w.show();
        let _ = w.set_focus();
    }
}

#[tauri::command]
fn resize_alert(app: AppHandle, height: f64) {
    if let Some(w) = app.get_webview_window(ALERT) {
        place_bottom_right(&w, height);
    }
}

// ---------------- Uyarı penceresi ----------------

fn alert_height(n: usize) -> f64 {
    64.0 + n.min(4) as f64 * 104.0
}

fn place_bottom_right(win: &tauri::WebviewWindow, logical_height: f64) {
    let scale = win.scale_factor().unwrap_or(1.0);
    let (w, h) = ((ALERT_WIDTH * scale) as u32, (logical_height * scale) as u32);
    let m = (MARGIN * scale) as i32;
    let _ = win.set_size(PhysicalSize::new(w, h));
    let monitor = win.current_monitor().ok().flatten().or_else(|| win.primary_monitor().ok().flatten());
    if let Some(mon) = monitor {
        let area = mon.work_area(); // görev çubuğu / Dock hariç alan
        let x = area.position.x + area.size.width as i32 - w as i32 - m;
        let y = area.position.y + area.size.height as i32 - h as i32 - m;
        let _ = win.set_position(PhysicalPosition::new(x, y));
    }
}

fn show_alert(app: &AppHandle) {
    let state = app.state::<AppState>();
    let p = payload(&state);
    if p.findings.is_empty() {
        return;
    }
    if is_wayland() {
        // Wayland pencere konumlandırmaya izin vermez; sistem bildirimi de gönder.
        let t = T::get(p.lang);
        let f = &p.findings[0];
        let _ = app
            .notification()
            .builder()
            .title(format!("{} ({})", t.notify_title, p.findings.len()))
            .body(format!("{}\n{}", f.unicode_host, f.messages.first().cloned().unwrap_or_default()))
            .show();
    }
    let Some(win) = app.get_webview_window(ALERT) else { return };
    place_bottom_right(&win, alert_height(p.findings.len()));
    let _ = win.emit("findings", &p);
    let _ = win.set_always_on_top(true);
    let _ = win.show();
}

fn on_clipboard(app: &AppHandle, text: String) {
    let state = app.state::<AppState>();
    let findings: Vec<Finding> = state
        .detector
        .read()
        .unwrap()
        .analyze_text(&text)
        .into_iter()
        .filter(|f| f.risk >= Risk::Medium)
        .collect();
    if findings.is_empty() {
        return;
    }
    *state.last.lock().unwrap() = findings;
    show_alert(app);
}

// ---------------- Dil ----------------

fn apply_lang(app: &AppHandle) {
    let state = app.state::<AppState>();
    let pref = *state.lang_pref.lock().unwrap();
    let t = T::get(state.lang());
    let items = app.state::<TrayItems>();
    let _ = items.pause.set_text(if state.paused.load(Ordering::Relaxed) { t.resume } else { t.pause });
    let _ = items.last.set_text(t.show_last);
    let _ = items.about.set_text(t.about);
    let _ = items.quit.set_text(t.quit);
    let _ = items.lang_auto.set_text(t.lang_auto);
    let _ = items.lang_auto.set_checked(pref.is_none());
    let _ = items.lang_tr.set_checked(pref == Some(Lang::Tr));
    let _ = items.lang_en.set_checked(pref == Some(Lang::En));
    if let Some(tray) = app.tray_by_id("main") {
        let _ = tray.set_tooltip(Some(t.tooltip));
    }
    // Açık bir uyarı varsa yeni dilde yeniden çiz.
    if let Some(win) = app.get_webview_window(ALERT) {
        let _ = win.emit("findings", payload(&state));
    }
    if let Some(win) = app.get_webview_window(ABOUT) {
        let _ = win.emit("lang-changed", ());
    }
}

fn set_lang(app: &AppHandle, pref: LangPref) {
    let state = app.state::<AppState>();
    *state.lang_pref.lock().unwrap() = pref;
    save_lang_pref(&state.config_dir, pref);
    apply_lang(app);
}

// ---------------- Uygulama ----------------

fn main() {
    tauri::Builder::default()
        // Tek kopya: ilk eklenti olmalı. Uygulama zaten açıksa yeni kopya kapanır,
        // açık olan kopya son uyarıyı (yoksa "Hakkında" penceresini) gösterir.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            let has_alert = !app.state::<AppState>().last.lock().unwrap().is_empty();
            if has_alert {
                show_alert(app);
            } else {
                show_about(app);
            }
        }))
        .plugin(tauri_plugin_notification::init())
        .invoke_handler(tauri::generate_handler![current_findings, trust, hide_alert, resize_alert, about_info, hide_about])
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory); // Dock'ta görünme

            let config_dir = app.path().app_config_dir()?;
            let paused = Arc::new(AtomicBool::new(false));
            let detector = build_detector(&config_dir);
            let lang_pref = load_lang_pref(&config_dir);
            println!(
                "Homoglyph Guard: {} popular domains, config: {}",
                detector.popular_count(),
                config_dir.display()
            );
            app.manage(AppState {
                detector: RwLock::new(detector),
                last: Mutex::new(Vec::new()),
                paused: paused.clone(),
                config_dir,
                lang_pref: Mutex::new(lang_pref),
            });

            // Sistem tepsisi (metinler apply_lang ile doldurulur)
            let t = T::get(app.state::<AppState>().lang());
            let items = TrayItems {
                pause: MenuItem::with_id(app, "pause", t.pause, true, None::<&str>)?,
                last: MenuItem::with_id(app, "last", t.show_last, true, None::<&str>)?,
                about: MenuItem::with_id(app, "about", t.about, true, None::<&str>)?,
                quit: MenuItem::with_id(app, "quit", t.quit, true, None::<&str>)?,
                lang_auto: CheckMenuItem::with_id(app, "lang_auto", t.lang_auto, true, lang_pref.is_none(), None::<&str>)?,
                lang_tr: CheckMenuItem::with_id(app, "lang_tr", "Türkçe", true, lang_pref == Some(Lang::Tr), None::<&str>)?,
                lang_en: CheckMenuItem::with_id(app, "lang_en", "English", true, lang_pref == Some(Lang::En), None::<&str>)?,
            };
            let lang_menu = Submenu::with_id_and_items(
                app,
                "lang",
                "Dil / Language",
                true,
                &[&items.lang_auto, &items.lang_tr, &items.lang_en],
            )?;
            let menu = Menu::with_items(
                app,
                &[
                    &items.pause,
                    &items.last,
                    &PredefinedMenuItem::separator(app)?,
                    &lang_menu,
                    &items.about,
                    &PredefinedMenuItem::separator(app)?,
                    &items.quit,
                ],
            )?;
            app.manage(items);

            TrayIconBuilder::with_id("main")
                .icon(app.default_window_icon().cloned().expect("icon missing"))
                .tooltip(t.tooltip)
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "pause" => {
                        let st = app.state::<AppState>();
                        st.paused.store(!st.paused.load(Ordering::Relaxed), Ordering::Relaxed);
                        apply_lang(app);
                    }
                    "last" => show_alert(app),
                    "about" => show_about(app),
                    "lang_auto" => set_lang(app, None),
                    "lang_tr" => set_lang(app, Some(Lang::Tr)),
                    "lang_en" => set_lang(app, Some(Lang::En)),
                    "quit" => app.exit(0),
                    _ => {}
                })
                .build(app)?;

            // Pano izleyici
            let handle = app.handle().clone();
            clipboard::spawn_watcher(paused, move |text| on_clipboard(&handle, text));
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .run(tauri::generate_context!())
        .expect("failed to start Tauri application");
}
