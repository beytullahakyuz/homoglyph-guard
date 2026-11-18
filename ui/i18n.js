// Uyarı penceresi metinleri. Uyarı nedenleri Rust tarafında seçili dilde hazırlanıp gelir.
window.I18N = {
  tr: {
    title: (n, high) => `Şüpheli alan adı: ${n}` + (high ? `  (${high} yüksek risk)` : ""),
    risk: { high: "YÜKSEK", medium: "ORTA", info: "BİLGİ" },
    trust: "Güvenilir",
    trustTip: (d) => `${d} için bundan sonra uyarı verilmesin`,
    lookalike: (d) => `Benzediği: ${d}`,
    close: "Kapat",
  },
  en: {
    title: (n, high) => `Suspicious domain${n > 1 ? "s" : ""}: ${n}` + (high ? `  (${high} high risk)` : ""),
    risk: { high: "HIGH", medium: "MEDIUM", info: "INFO" },
    trust: "Trust",
    trustTip: (d) => `Don't warn about ${d} again`,
    lookalike: (d) => `Looks like: ${d}`,
    close: "Close",
  },
};
