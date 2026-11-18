const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

const AUTO_HIDE_MS = 20000;
const card = document.getElementById("card");
const list = document.getElementById("list");
const title = document.getElementById("title");
let findings = [];
let lang = "en";
const t = () => window.I18N[lang] || window.I18N.en;
let hideTimer = null;
let hovering = false;


function el(tag, cls, text) {
  const e = document.createElement(tag);
  if (cls) e.className = cls;
  if (text != null) e.textContent = text;
  return e;
}

/** ASCII dışı karakterleri <mark> ile vurgular (textContent ile, HTML enjeksiyonu yok). */
function highlightHost(host) {
  const span = el("span", "host");
  span.title = host;
  for (const ch of host) {
    if (ch.codePointAt(0) >= 0x80) span.appendChild(el("mark", null, ch));
    else span.appendChild(document.createTextNode(ch));
  }
  return span;
}

function render() {
  list.replaceChildren();
  if (!findings.length) {
    card.classList.add("hidden");
    invoke("hide_alert");
    return;
  }
  const high = findings.filter((f) => f.risk === "high").length;
  card.classList.remove("hidden");
  card.classList.toggle("medium", high === 0);
  document.documentElement.lang = lang;
  document.getElementById("close").title = t().close;
  title.textContent = t().title(findings.length, high);

  for (const f of findings) {
    const li = el("li");
    const row = el("div", "row");
    row.append(el("span", `badge ${f.risk}`, t().risk[f.risk]), highlightHost(f.unicode_host));
    const btn = el("button", "trust", t().trust);
    btn.title = t().trustTip(f.registrable);
    btn.onclick = async () => {
      await invoke("trust", { registrable: f.registrable });
      findings = findings.filter((x) => x.registrable !== f.registrable);
      render();
      resize();
    };
    row.append(btn);
    li.append(row);
    if (f.ascii_host !== f.unicode_host) li.append(el("div", "ascii", f.ascii_host));
    if (f.lookalike_of) li.append(el("div", "like", t().lookalike(f.lookalike_of)));
    for (const r of f.messages) li.append(el("div", "reason", `• ${r}`));
    list.append(li);
  }
}

function resize() {
  if (!findings.length) return;
  invoke("resize_alert", { height: 64 + Math.min(findings.length, 4) * 104 });
}

function restartTimer() {
  clearTimeout(hideTimer);
  hideTimer = setTimeout(() => {
    if (!hovering) invoke("hide_alert");
  }, AUTO_HIDE_MS);
}

card.addEventListener("mouseenter", () => { hovering = true; clearTimeout(hideTimer); });
card.addEventListener("mouseleave", () => { hovering = false; restartTimer(); });
document.getElementById("close").onclick = () => invoke("hide_alert");
document.addEventListener("keydown", (e) => { if (e.key === "Escape") invoke("hide_alert"); });

function apply(p) {
  lang = p.lang;
  findings = p.findings;
}

listen("findings", (e) => {
  apply(e.payload);
  render();
  restartTimer();
});

invoke("current_findings").then((p) => {
  if (p.findings.length) { apply(p); render(); }
});
