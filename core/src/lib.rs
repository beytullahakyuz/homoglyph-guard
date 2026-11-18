//! Homoglyph / punycode / typosquatting alan adı tespit motoru.
//!
//! ```
//! use homoglyph_core::{Detector, Risk};
//! let d = Detector::with_builtin();
//! let f = d.analyze_text("bak: https://xn--pple-43d.com/login?x=1");
//! assert_eq!(f[0].risk, Risk::High);
//! assert_eq!(f[0].lookalike_of.as_deref(), Some("apple.com"));
//! ```

pub mod confusables;
pub mod detector;
pub mod extract;
pub mod i18n;
pub mod punycode;
pub mod scripts;

pub use confusables::Confusables;
pub use detector::{Detector, Finding, Risk};
pub use extract::{extract_hosts, ExtractedHost};
pub use i18n::{Lang, Reason};
