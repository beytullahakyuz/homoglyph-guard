<div align="center">

<img src="src-tauri/icons/128x128.png" width="96" alt="Homoglyph Guard logo" />

# Homoglyph Guard

**Detects fake domain names in URLs you copy, before you paste them into a browser.**

🇬🇧 English · [🇹🇷 Türkçe](README.tr.md)

![Platforms](https://img.shields.io/badge/platforms-Linux%20%7C%20Windows%20%7C%20macOS-informational)
![Rust](https://img.shields.io/badge/Rust-Tauri%202-orange)
![License](https://img.shields.io/badge/license-MIT-green)

</div>

---

## What is it?

Phishing links often use domains that look identical to real ones:

| Looks like | Actually is | Trick |
|---|---|---|
| `apple.com` | `аpple.com` | The first "а" is a **Cyrillic** letter |
| `paypal.com` | `paypa1.com` | The "l" is the digit **1** |
| `microsoft.com` | `rnicrosoft.com` | **r + n** looks like **m** |
| `isbank.com.tr` | `ısbank.com.tr` | Turkish dotless **ı** |
| `garantibbva.com.tr` | `https://garantibbva.com.tr@evil.site` | Everything before **@** is ignored |

Homoglyph Guard runs quietly in the system tray and watches the clipboard. When you copy text containing URLs, it checks **only the domain part** of each URL. If a domain looks fake, a warning appears in the **bottom-right corner** of the screen showing the real characters, the punycode form and the site it imitates.

- 🔒 **Fully offline.** Nothing you copy ever leaves your computer.
- ⚡ **Fast.** 5,000 URLs are checked in about 15 ms.
- 🪶 **Lightweight.** Written in Rust, runs in the background with low CPU usage.
- 🌍 **Turkish and English** interface, following the system language.

## Detection rules

| # | Rule | Risk | Example |
|---|---|---|---|
| 1 | **Punycode** (`xn--`) is decoded; malformed punycode | High | `xn--pple-43d.com` → `аpple.com` |
| 2 | **Mixed scripts** in one label (Latin + Cyrillic/Greek…) | High | `akbаnk.com` |
| 3 | Label made **entirely** of Cyrillic/Greek letters that look Latin | High | `аррӏе.com` |
| 4 | **Fullwidth** characters | High | `ｇｏｏｇｌｅ.com` |
| 5 | **Visual twin** of a popular domain ([UTS #39](https://www.unicode.org/reports/tr39/) skeleton: `0→o`, `1→l`, `rn→m`, `ı→i`…) | High | `paypa1.com`, `g00gle.com` |
| 6 | **Typosquatting:** one edit away from a protected domain | Medium | `gooogle.com`, `trendoyl.com` |
| 7 | **Brand impersonation:** a brand name inside an unrelated domain | Medium | `paypal-login.com`, `apple.com.secure-verify.net` |
| 8 | **"@" trick** in the URL | Medium | `https://bank.com@evil.site/` |

Rules 1–4 and 8 work for **any** domain, whether or not it is on a list. Rules 5–7 compare against protected domain lists.

Paths and query parameters are never inspected. `https://google.com/?next=paypa1.com` is treated as clean.

### Accuracy

The rules were measured against 20,000 real websites (Tranco ranks 10,001–30,000):

| Rules | False alarm rate |
|---|---|
| High risk (homoglyph, punycode, visual twin) | **0.01%** |
| Medium risk (typosquatting, brand impersonation) | **0.23%** |

## Installation

Download the package for your system from the [**Releases**](../../releases) page:

| System | File |
|---|---|
| Windows 10/11 | `Homoglyph.Guard_x.y.z_x64-setup.exe` or `.msi` |
| macOS 10.15+ (Intel and Apple Silicon) | `Homoglyph.Guard_x.y.z_universal.dmg` |
| Linux | `.AppImage` (any distribution), `.deb` (Debian/Ubuntu), `.rpm` (Fedora/openSUSE) |

> **The packages are not code-signed.**
> - **Windows** may show a SmartScreen warning: click *More info → Run anyway*.
> - **macOS:** on first launch, right-click the app in Finder and choose *Open*, or run:
>   `xattr -dr com.apple.quarantine "/Applications/Homoglyph Guard.app"`
> - **Linux AppImage:** `chmod +x Homoglyph*.AppImage` then run it.

## Usage

1. Launch the app. It has no main window; a shield icon appears in the system tray (menu bar on macOS).
2. Copy any text containing URLs, from a single link to a list of hundreds.
3. If a suspicious domain is found, a warning card appears in the bottom-right corner. It closes by itself after 20 seconds, stays open while the mouse is over it, and closes with **Esc**.

On the warning card:
- Non-ASCII characters are **highlighted in red**.
- The punycode form and the real site being imitated are shown.
- **Trust** stops warnings for that domain from then on.

Tray menu: **Pause monitoring**, **Show last alert**, **Language** (Automatic / Türkçe / English), **About**, **Quit**.

Only one copy of the app runs at a time. Launching it again brings up the running copy.

**Quick test:** copy `https://xn--pple-43d.com/login`. A high-risk warning should appear.

## Configuration

Optional files in the configuration folder:

| System | Folder |
|---|---|
| Linux | `~/.config/com.homoglyphguard.desktop/` |
| macOS | `~/Library/Application Support/com.homoglyphguard.desktop/` |
| Windows | `%APPDATA%\com.homoglyphguard.desktop\` |

| File | Purpose |
|---|---|
| `priority_domains.txt` | **Critical targets** (your bank, your company's domains…). All rules apply, including typosquatting across different TLDs. Keep it to a few hundred entries. |
| `popular_domains.txt` | **Large lists** such as [Tranco](https://tranco-list.eu). Used for the "is it popular?" and "visual twin?" checks. One million domains load in about 1 second. |
| `whitelist.txt` | Domains marked **Trust**. Written by the app; can be edited by hand. |
| `language.txt` | `auto`, `tr` or `en` |
| `confusables.txt` | Optional full Unicode [confusables table](https://www.unicode.org/Public/security/latest/confusables.txt) |

One domain per line. Lines starting with `#` are ignored. Restart the app after editing.

**Loading the full Tranco list (excluding `.cn`):**
```bash
curl -LO https://tranco-list.eu/top-1m.csv.zip
unzip -p top-1m.csv.zip | tr -d '\r' | cut -d, -f2 | grep -viE '\.cn$' \
  > ~/.config/com.homoglyphguard.desktop/popular_domains.txt
```

The app ships with two built-in lists: about 100 hand-picked critical targets (Turkish banks, payment services, email providers; all rules apply) and the top 10,000 Tranco domains (excluding `.cn`; visual twin check only).

## Building from source

**Requirements:** Rust 1.77+ and the Tauri CLI (`cargo install tauri-cli --version "^2" --locked`).

| System | Extra packages |
|---|---|
| Arch Linux | `sudo pacman -S webkit2gtk-4.1 libayatana-appindicator librsvg fuse2` |
| Debian/Ubuntu | `sudo apt install libwebkit2gtk-4.1-dev libayatana-appindicator3-dev librsvg2-dev libxdo-dev build-essential` |
| macOS | Xcode Command Line Tools |
| Windows | Visual Studio Build Tools ("Desktop development with C++") |

```bash
cargo test -p homoglyph_core     # detection engine tests
cd src-tauri
cargo tauri dev                  # run in development mode
cargo tauri build                # build installers
```

### Builds for all platforms (GitHub Actions)

`.github/workflows/build.yml` builds Windows, macOS and Linux packages on GitHub's machines:

- **Manual:** Actions → *build* → *Run workflow*. Packages appear under the run's **Artifacts**.
- **Release:** `git tag v0.1.0 && git push --tags`. Packages are attached to a draft GitHub Release.

## Project structure

```
core/         Detection engine: a pure Rust library with no dependencies, plus tests
  src/punycode.rs      RFC 3492 Punycode
  src/scripts.rs       Unicode script detection
  src/confusables.rs   Confusable characters and UTS #39 skeletons
  src/extract.rs       Extracts hosts from text (single pass, no regex)
  src/detector.rs      Detection rules and two-tier index
  src/i18n.rs          Warning messages (TR / EN)
  data/                Built-in domain lists
src-tauri/    Desktop app: clipboard watcher, tray, windows
ui/           Warning card and About window (plain HTML/CSS/JS)
```

## Privacy

Homoglyph Guard makes **no network connections**. Clipboard contents are analyzed in memory and never stored or sent anywhere. The only files written are the settings files in the configuration folder (`whitelist.txt` and `language.txt`).

## Known limitations

- **Wayland** does not let apps position their own windows, so the warning may not appear in the bottom-right corner. A system notification is sent as well.
- The clipboard is checked every 400 ms; the same content is not checked twice.
- ASCII-only imitations (`paypa1.com`) are detected only for domains on a protected list. Imitations using other scripts (`аpple.com`) are detected for any domain.

## License

[MIT](LICENSE) © 2026 Beytullah
