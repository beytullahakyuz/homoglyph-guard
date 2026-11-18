<div align="center">

<img src="src-tauri/icons/128x128.png" width="96" alt="Homoglyph Guard logosu" />

# Homoglyph Guard

**Kopyaladığınız URL'lerdeki sahte alan adlarını, tarayıcıya yapıştırmadan önce yakalar.**

[🇬🇧 English](README.md) · 🇹🇷 Türkçe

![Platformlar](https://img.shields.io/badge/platform-Linux%20%7C%20Windows%20%7C%20macOS-informational)
![Rust](https://img.shields.io/badge/Rust-Tauri%202-orange)
![Lisans](https://img.shields.io/badge/lisans-MIT-green)

</div>

---

## Nedir?

Oltalama (phishing) bağlantıları çoğu zaman gerçek bir alan adıyla birebir aynı görünen adresler kullanır:

| Görünen | Gerçekte olan | Hile |
|---|---|---|
| `apple.com` | `аpple.com` | İlk "а" harfi **Kiril** alfabesinden |
| `paypal.com` | `paypa1.com` | "l" yerine **1** rakamı |
| `microsoft.com` | `rnicrosoft.com` | **r + n**, **m** gibi görünüyor |
| `isbank.com.tr` | `ısbank.com.tr` | Türkçe noktasız **ı** |
| `garantibbva.com.tr` | `https://garantibbva.com.tr@kotu.site` | **@** öncesindeki her şey yok sayılır |

Homoglyph Guard sistem tepsisinde sessizce çalışır ve panoyu izler. URL içeren bir metin kopyaladığınızda her URL'nin **yalnızca alan adı** kısmını denetler. Bir alan adı sahte görünüyorsa ekranın **sağ alt köşesinde** bir uyarı çıkar; uyarıda gerçek karakterler, punycode hali ve taklit edilen site gösterilir.

- 🔒 **Tamamen çevrimdışı.** Kopyaladığınız hiçbir şey bilgisayarınızdan dışarı çıkmaz.
- ⚡ **Hızlı.** 5.000 URL yaklaşık 15 ms'de denetlenir.
- 🪶 **Hafif.** Rust ile yazıldı; arka planda düşük CPU kullanımıyla çalışır.
- 🌍 **Türkçe ve İngilizce** arayüz; sistem diline göre otomatik seçilir.

## Tespit kuralları

| # | Kural | Risk | Örnek |
|---|---|---|---|
| 1 | **Punycode** (`xn--`) çözülür; bozuk punycode | Yüksek | `xn--pple-43d.com` → `аpple.com` |
| 2 | Bir etikette **karışık alfabe** (Latin + Kiril/Yunan…) | Yüksek | `akbаnk.com` |
| 3 | **Tamamı** Latin'e benzeyen Kiril/Yunan harflerden oluşan etiket | Yüksek | `аррӏе.com` |
| 4 | **Tam genişlik** (fullwidth) karakterler | Yüksek | `ｇｏｏｇｌｅ.com` |
| 5 | Popüler bir alan adının **görsel ikizi** ([UTS #39](https://www.unicode.org/reports/tr39/) iskeleti: `0→o`, `1→l`, `rn→m`, `ı→i`…) | Yüksek | `paypa1.com`, `g00gle.com` |
| 6 | **Yazım hatası (typosquatting):** korunan bir alan adından tek harf farklı | Orta | `gooogle.com`, `trendoyl.com` |
| 7 | **Marka taklidi:** ilgisiz bir alan adında marka adı | Orta | `paypal-login.com`, `apple.com.secure-verify.net` |
| 8 | URL'de **"@" hilesi** | Orta | `https://banka.com@kotu.site/` |

1–4 ve 8 numaralı kurallar, listede olsun olmasın **her** alan adı için çalışır. 5–7 numaralı kurallar korunan alan adı listeleriyle karşılaştırma yapar.

Path ve sorgu parametreleri hiç incelenmez. `https://google.com/?next=paypa1.com` temiz sayılır.

### Doğruluk

Kurallar 20.000 gerçek web sitesi üzerinde ölçüldü (Tranco'nun 10.001–30.000 arası sıraları):

| Kurallar | Yanlış alarm oranı |
|---|---|
| Yüksek risk (homoglyph, punycode, görsel ikiz) | **%0,01** |
| Orta risk (yazım hatası, marka taklidi) | **%0,23** |

## Kurulum

Sisteminize uygun paketi [**Releases**](../../releases) sayfasından indirin:

| Sistem | Dosya |
|---|---|
| Windows 10/11 | `Homoglyph.Guard_x.y.z_x64-setup.exe` veya `.msi` |
| macOS 10.15+ (Intel ve Apple Silicon) | `Homoglyph.Guard_x.y.z_universal.dmg` |
| Linux | `.AppImage` (tüm dağıtımlar), `.deb` (Debian/Ubuntu), `.rpm` (Fedora/openSUSE) |

> **Paketler kod imzalı değildir.**
> - **Windows** SmartScreen uyarısı gösterebilir: *Ek bilgi → Yine de çalıştır*'a tıklayın.
> - **macOS:** İlk açılışta Finder'da uygulamaya sağ tıklayıp *Aç*'ı seçin, ya da şu komutu çalıştırın:
>   `xattr -dr com.apple.quarantine "/Applications/Homoglyph Guard.app"`
> - **Linux AppImage:** `chmod +x Homoglyph*.AppImage` komutundan sonra çalıştırın.

## Kullanım

1. Uygulamayı başlatın. Ana penceresi yoktur; sistem tepsisinde (macOS'ta menü çubuğunda) bir kalkan simgesi görünür.
2. URL içeren herhangi bir metni kopyalayın; tek bir bağlantı da olabilir, yüzlerce bağlantılık bir liste de.
3. Şüpheli bir alan adı bulunursa sağ alt köşede bir uyarı kartı açılır. Kart 20 saniye sonra kendiliğinden kapanır, fare üzerindeyken açık kalır, **Esc** ile de kapatılabilir.

Uyarı kartında:
- ASCII dışı karakterler **kırmızıyla vurgulanır**.
- Punycode hali ve taklit edilen gerçek site gösterilir.
- **Güvenilir** butonu, o alan adı için bundan sonra uyarı verilmesini engeller.

Tepsi menüsü: **İzlemeyi duraklat**, **Son uyarıyı göster**, **Dil / Language** (Otomatik / Türkçe / English), **Hakkında**, **Çıkış**.

Uygulamanın aynı anda yalnızca bir kopyası çalışır. Yeniden başlatıldığında açık olan kopya öne gelir.

**Hızlı deneme:** `https://xn--pple-43d.com/login` metnini kopyalayın. Yüksek risk uyarısı çıkmalı.

## Yapılandırma

Yapılandırma klasöründeki isteğe bağlı dosyalar:

| Sistem | Klasör |
|---|---|
| Linux | `~/.config/com.homoglyphguard.desktop/` |
| macOS | `~/Library/Application Support/com.homoglyphguard.desktop/` |
| Windows | `%APPDATA%\com.homoglyphguard.desktop\` |

| Dosya | Amacı |
|---|---|
| `priority_domains.txt` | **Kritik hedefler** (bankanız, şirketinizin alan adları…). Farklı uzantıdaki yazım hataları dahil tüm kurallar uygulanır. Birkaç yüz alan adını geçmemesi önerilir. |
| `popular_domains.txt` | [Tranco](https://tranco-list.eu) gibi **büyük listeler**. "Popüler mi?" ve "görsel ikiz mi?" kontrollerinde kullanılır. 1 milyon alan adı yaklaşık 1 saniyede yüklenir. |
| `whitelist.txt` | **Güvenilir** olarak işaretlenen alan adları. Uygulama kendisi yazar; elle de düzenlenebilir. |
| `language.txt` | `auto`, `tr` veya `en` |
| `confusables.txt` | İsteğe bağlı tam Unicode [karıştırılabilir karakter tablosu](https://www.unicode.org/Public/security/latest/confusables.txt) |

Satır başına bir alan adı yazılır; `#` ile başlayan satırlar yok sayılır. Düzenledikten sonra uygulamayı yeniden başlatın.

**Tranco listesinin tamamını yüklemek (`.cn` hariç):**
```bash
curl -LO https://tranco-list.eu/top-1m.csv.zip
unzip -p top-1m.csv.zip | tr -d '\r' | cut -d, -f2 | grep -viE '\.cn$' \
  > ~/.config/com.homoglyphguard.desktop/popular_domains.txt
```

Uygulamanın içinde iki yerleşik liste gelir: elle seçilmiş yaklaşık 100 kritik hedef (Türk bankaları, ödeme servisleri, e-posta sağlayıcıları; tüm kurallar uygulanır) ve Tranco'nun en popüler 10.000 alan adı (`.cn` hariç; yalnızca görsel ikiz kontrolü).

## Kaynak koddan derleme

**Gereksinimler:** Rust 1.77+ ve Tauri CLI (`cargo install tauri-cli --version "^2" --locked`).

| Sistem | Ek paketler |
|---|---|
| Arch Linux | `sudo pacman -S webkit2gtk-4.1 libayatana-appindicator librsvg fuse2` |
| Debian/Ubuntu | `sudo apt install libwebkit2gtk-4.1-dev libayatana-appindicator3-dev librsvg2-dev libxdo-dev build-essential` |
| macOS | Xcode Command Line Tools |
| Windows | Visual Studio Build Tools ("Desktop development with C++") |

```bash
cargo test -p homoglyph_core     # tespit motoru testleri
cd src-tauri
cargo tauri dev                  # geliştirme modunda çalıştır
cargo tauri build                # kurulum paketlerini üret
```

### Tüm platformlar için derleme (GitHub Actions)

`.github/workflows/build.yml`, Windows, macOS ve Linux paketlerini GitHub'ın makinelerinde derler:

- **Elle:** Actions → *build* → *Run workflow*. Paketler çalışmanın **Artifacts** bölümünde görünür.
- **Sürüm:** `git tag v0.1.0 && git push --tags`. Paketler taslak bir GitHub Release'e eklenir.

## Proje yapısı

```
core/         Tespit motoru: bağımlılıksız, saf Rust kütüphanesi ve testleri
  src/punycode.rs      RFC 3492 Punycode
  src/scripts.rs       Unicode alfabe tespiti
  src/confusables.rs   Karıştırılabilir karakterler ve UTS #39 iskeleti
  src/extract.rs       Metinden host çıkarma (tek geçiş, regex yok)
  src/detector.rs      Tespit kuralları ve iki kademeli indeks
  src/i18n.rs          Uyarı mesajları (TR / EN)
  data/                Yerleşik alan adı listeleri
src-tauri/    Masaüstü uygulaması: pano izleyici, sistem tepsisi, pencereler
ui/           Uyarı kartı ve Hakkında penceresi (düz HTML/CSS/JS)
```

## Gizlilik

Homoglyph Guard **hiçbir ağ bağlantısı kurmaz**. Pano içeriği bellekte incelenir; hiçbir yere kaydedilmez veya gönderilmez. Uygulamanın yazdığı tek dosyalar, yapılandırma klasöründeki ayar dosyalarıdır (`whitelist.txt` ve `language.txt`).

## Bilinen sınırlamalar

- **Wayland**, uygulamaların kendi pencerelerinin konumunu belirlemesine izin vermez; bu yüzden uyarı sağ alt köşede görünmeyebilir. Uyarıyla birlikte bir sistem bildirimi de gönderilir.
- Pano 400 ms'de bir denetlenir; aynı içerik ikinci kez denetlenmez.
- Yalnızca ASCII karakterlerle yapılan taklitler (`paypa1.com`), yalnızca korunan listelerdeki alan adları için yakalanır. Başka alfabeden harf kullanan taklitler (`аpple.com`) her alan adı için yakalanır.

## Lisans

[MIT](LICENSE) © 2026 Beytullah
