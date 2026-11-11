//! Product privacy policy — one Markdown file for app and website.
//!
//! Source of truth: `apps/khaynjo/docs/PRIVACY.md`
//! - App: Settings → Privacy (`WrapLabel` / Makepad Markdown presentation)
//! - Site: `https://khaynjo.com/privacy` (formatted HTML from the same file)
//! Future apps: each owns `apps/<app>/docs/PRIVACY.md` the same way.

/// Full policy text (Markdown). Edit only `docs/PRIVACY.md`.
pub const PRIVACY_MD: &str = include_str!("../docs/PRIVACY.md");
