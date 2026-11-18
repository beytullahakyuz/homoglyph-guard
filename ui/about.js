const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

const LABELS = {
  tr: {
    version: (v) => `Sürüm ${v}`,
    developer: "Geliştirici",
    email: "E-posta",
    website: "Web sitesi",
    license: "Lisans",
    protected: "Korunan alan adı",
    close: "Kapat",
  },
  en: {
    version: (v) => `Version ${v}`,
    developer: "Developer",
    email: "Email",
    website: "Website",
    license: "License",
    protected: "Protected domains",
    close: "Close",
  },
};

const $ = (id) => document.getElementById(id);

async function render() {
  const i = await invoke("about_info");
  const l = LABELS[i.lang] || LABELS.en;
  document.documentElement.lang = i.lang;
  $("name").textContent = i.name;
  $("version").textContent = l.version(i.version);
  $("description").textContent = i.description;
  for (const k of ["developer", "email", "website", "license", "protected"]) $(`l-${k}`).textContent = l[k];
  $("developer").textContent = i.developer;
  $("email").textContent = i.email;
  $("website").textContent = i.website;
  $("license").textContent = i.license;
  $("protected").textContent = i.popular_count.toLocaleString(i.lang);
  $("copyright").textContent = i.copyright;
  $("close").textContent = l.close;
}

$("close").onclick = () => invoke("hide_about");
document.addEventListener("keydown", (e) => { if (e.key === "Escape") invoke("hide_about"); });
listen("lang-changed", render);
render();
