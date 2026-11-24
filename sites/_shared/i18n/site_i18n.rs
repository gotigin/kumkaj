//! Website language runtime shared by the Kumkaj Makepad sites (khaynjo,
//! moklanjo, kirano, tabiyat, kumkaj). Each site includes this file —
//!
//! ```ignore
//! #[path = "../../_shared/i18n/site_i18n.rs"]
//! mod site_i18n;
//! ```
//!
//! — and owns its words: `src/locale.rs` (`en`, the English source keyed by
//! a stable id) and `translations/<code>.tsv` (one `key<TAB>text` line per
//! `en` key for every table language, the same 110 stems as the apps). The
//! sites never depend on an app crate (docs/WEBSITES.md §3); this file only
//! shares the catalogue and the machinery.
//!
//! **How a language reaches the page.** `lang-nav.js` (the HTML 115-language
//! picker) stores the choice and writes `#l=<code>` into the URL hash; the
//! wasm reads that hash on Startup and on every hashchange (it arrives as
//! `Event::Signal`) — no web-sys, which would pull wasm-bindgen imports
//! Makepad's bridge does not provide. The pack for that language is fetched
//! as `/i18n/<code>.tsv` (staged next to the wasm); it is never baked into
//! the wasm, so an English visitor downloads nothing extra.
//!
//! **How the words change.** `SiteLang::apply` walks the widget tree. The
//! first time it meets a Label / Markdown / Button it records which `en`
//! key the widget's (English) text is; from then on that widget shows
//! `t(lang, key)`. Text that is not an `en` value — dish names, numbers,
//! mock tickets — is left as drawn. Text the site sets from code goes
//! through the site's `tr()` and is repainted after a language change.
//!
//! **Fonts.** Makepad loads every member of a font family the first time it
//! draws with it, so the sites' UI families stay Latin-only (fast first
//! visit). Hidden prototype labels in `ui.rs` carry one family per script
//! (Latin UI face first, then the Noto face for that script); keyed widgets
//! switch to it for a non-Latin language and get their own family back for
//! a Latin one. Script faces are subset at stage time to the characters the
//! packs use (`subset-fonts.sh` reads `translations/*.tsv`).
//!
//! **Right to left.** Makepad shapes and bidi-orders the glyphs; keyed
//! labels and paragraphs are right-aligned while an RTL language is shown
//! (the apps do the same, `align_static_texts`), every paragraph starts
//! with a RIGHT-TO-LEFT MARK so Makepad lays it out right to left
//! (`rtl_marks`), and lang-nav.js flips `dir` on the HTML pages.
//!
//! **Line breaks.** Makepad wraps at Unicode (UAX #14) break opportunities
//! (vendor patch `line_break_uax14`): punctuation of every script stays with
//! its word and no-break spaces hold. Thai, Lao, Khmer and Burmese packs
//! carry a ZERO WIDTH SPACE between words, where their rows may break
//! (`checks::words_are_marked_in_scripts_without_spaces`).

#![allow(dead_code)]

use makepad_widgets::*;
use makepad_widgets::shader::draw_text::FontFamily;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy, Debug)]
pub struct LanguageInfo {
    pub code: &'static str,
    pub name_en: &'static str,
    pub native_name: &'static str,
    pub region: &'static str,
}

include!("languages.rs");

/// The catalogue entry for `code`, matched case-insensitively (`zh-hant`
/// from a hand-typed URL is `zh-Hant`).
pub fn language_by_code(code: &str) -> Option<&'static LanguageInfo> {
    let code = code.trim();
    LANGUAGES.iter().find(|l| l.code.eq_ignore_ascii_case(code))
}

/// Catalogue languages that read another language's table (same rule as
/// the apps): regional variants share their parent's words, Cantonese reads
/// Traditional Chinese and Goan Konkani reads Konkani.
pub fn table_language(lang: &str) -> &str {
    match lang {
        "pt-BR" => "pt",
        "es-MX" => "es",
        "yue" => "zh-Hant",
        "gom" => "kok",
        other => other,
    }
}

/// Languages written right to left (same list as the apps' `is_rtl`).
pub fn is_rtl(lang: &str) -> bool {
    let primary = lang
        .split(['-', '_'])
        .next()
        .unwrap_or(lang)
        .to_ascii_lowercase();
    matches!(
        primary.as_str(),
        "ar" | "ur" | "fa" | "he" | "dv" | "ps" | "sd" | "ks" | "yi" | "ckb" | "ug"
    )
}

const RLM: char = '\u{200F}';

/// Copy for a right-to-left language, marked for Makepad's text layout.
///
/// A RIGHT-TO-LEFT MARK starts every paragraph, so Makepad lays it out right
/// to left on every row (vendor patch `rtl_paragraph_direction`): Latin words
/// and numbers keep their place in the sentence, punctuation at either end of
/// a row stays with its words, and the space that ends a wrapped row hangs at
/// its left end, past the right-aligned text. Without the mark Makepad keeps
/// a left-to-right base, as it does for the apps' user data.
///
/// The mark goes after a Markdown block marker (`+ `, `- `, `# `, `> `,
/// `1. `), which must stay first to keep its meaning; blank lines stay blank
/// (they separate Markdown paragraphs).
pub fn rtl_marks(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 3 * (1 + text.matches('\n').count()));
    for (i, line) in text.split('\n').enumerate() {
        if i > 0 {
            out.push('\n');
        }
        let at = block_marker_len(line);
        out.push_str(&line[..at]);
        if !line.trim().is_empty() && !line[at..].starts_with(RLM) {
            out.push(RLM);
        }
        out.push_str(&line[at..]);
    }
    out
}

/// Bytes of `line` taken by Markdown block markers (`# `, `> `, `- `, `+ `,
/// `* `, `1. `, `1) `) with their indentation, nested ones included; 0 when
/// the line starts with none.
fn block_marker_len(line: &str) -> usize {
    let bytes = line.as_bytes();
    let (mut at, mut end) = (0, 0);
    loop {
        while bytes.get(at) == Some(&b' ') {
            at += 1;
        }
        let rest = &line[at..];
        let marker = match rest.bytes().next() {
            Some(b'#') => rest.bytes().take_while(|&b| b == b'#').count(),
            Some(b'>' | b'-' | b'+' | b'*') => 1,
            Some(b'0'..=b'9') => {
                let digits = rest.bytes().take_while(u8::is_ascii_digit).count();
                if rest[digits..].starts_with(['.', ')']) {
                    digits + 1
                } else {
                    0
                }
            }
            _ => 0,
        };
        if marker == 0 || !rest[marker..].starts_with(' ') {
            return end;
        }
        at += marker + 1;
        end = at;
    }
}

/// The writing system a language's site copy is drawn in — picks the
/// prototype font family (`ScriptGroup::proto_id`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ScriptGroup {
    Latin,
    Cyrillic,
    Greek,
    Devanagari,
    Bengali,
    Gurmukhi,
    Gujarati,
    Oriya,
    Tamil,
    Telugu,
    Kannada,
    Malayalam,
    Sinhala,
    Arabic,
    Hebrew,
    Thaana,
    Tibetan,
    OlChiki,
    MeeteiMayek,
    Thai,
    Lao,
    Khmer,
    Myanmar,
    Ethiopic,
    Armenian,
    Georgian,
    /// Simplified Chinese.
    CjkSc,
    /// Traditional Chinese (and Cantonese).
    CjkTc,
    CjkJp,
    CjkKr,
    /// Latin languages whose letters the UI faces lack (Noto Sans covers them).
    LatinExt,
}

impl ScriptGroup {
    /// Every group but Latin, in the order the prototypes appear in `ui.rs`.
    pub const NON_LATIN: [ScriptGroup; 30] = [
        ScriptGroup::Cyrillic,
        ScriptGroup::Greek,
        ScriptGroup::Devanagari,
        ScriptGroup::Bengali,
        ScriptGroup::Gurmukhi,
        ScriptGroup::Gujarati,
        ScriptGroup::Oriya,
        ScriptGroup::Tamil,
        ScriptGroup::Telugu,
        ScriptGroup::Kannada,
        ScriptGroup::Malayalam,
        ScriptGroup::Sinhala,
        ScriptGroup::Arabic,
        ScriptGroup::Hebrew,
        ScriptGroup::Thaana,
        ScriptGroup::Tibetan,
        ScriptGroup::OlChiki,
        ScriptGroup::MeeteiMayek,
        ScriptGroup::Thai,
        ScriptGroup::Lao,
        ScriptGroup::Khmer,
        ScriptGroup::Myanmar,
        ScriptGroup::Ethiopic,
        ScriptGroup::Armenian,
        ScriptGroup::Georgian,
        ScriptGroup::CjkSc,
        ScriptGroup::CjkTc,
        ScriptGroup::CjkJp,
        ScriptGroup::CjkKr,
        ScriptGroup::LatinExt,
    ];

    /// Id of the hidden prototype label (`i18n_fonts.<id>` in `ui.rs`) whose
    /// font family draws this script.
    pub fn proto_id(self) -> LiveId {
        match self {
            ScriptGroup::Latin => live_id!(font_latin),
            ScriptGroup::Cyrillic => live_id!(font_cyrillic),
            ScriptGroup::Greek => live_id!(font_greek),
            ScriptGroup::Devanagari => live_id!(font_devanagari),
            ScriptGroup::Bengali => live_id!(font_bengali),
            ScriptGroup::Gurmukhi => live_id!(font_gurmukhi),
            ScriptGroup::Gujarati => live_id!(font_gujarati),
            ScriptGroup::Oriya => live_id!(font_oriya),
            ScriptGroup::Tamil => live_id!(font_tamil),
            ScriptGroup::Telugu => live_id!(font_telugu),
            ScriptGroup::Kannada => live_id!(font_kannada),
            ScriptGroup::Malayalam => live_id!(font_malayalam),
            ScriptGroup::Sinhala => live_id!(font_sinhala),
            ScriptGroup::Arabic => live_id!(font_arabic),
            ScriptGroup::Hebrew => live_id!(font_hebrew),
            ScriptGroup::Thaana => live_id!(font_thaana),
            ScriptGroup::Tibetan => live_id!(font_tibetan),
            ScriptGroup::OlChiki => live_id!(font_olchiki),
            ScriptGroup::MeeteiMayek => live_id!(font_meetei),
            ScriptGroup::Thai => live_id!(font_thai),
            ScriptGroup::Lao => live_id!(font_lao),
            ScriptGroup::Khmer => live_id!(font_khmer),
            ScriptGroup::Myanmar => live_id!(font_myanmar),
            ScriptGroup::Ethiopic => live_id!(font_ethiopic),
            ScriptGroup::Armenian => live_id!(font_armenian),
            ScriptGroup::Georgian => live_id!(font_georgian),
            ScriptGroup::CjkSc => live_id!(font_cjk_sc),
            ScriptGroup::CjkTc => live_id!(font_cjk_tc),
            ScriptGroup::CjkJp => live_id!(font_cjk_jp),
            ScriptGroup::CjkKr => live_id!(font_cjk_kr),
            ScriptGroup::LatinExt => live_id!(font_latin_ext),
        }
    }
}

/// Script of a catalogue language's site copy (by its table language).
pub fn script_group(lang: &str) -> ScriptGroup {
    match table_language(lang) {
        "hi" | "mr" | "ne" | "mai" | "sa" | "kok" | "doi" | "brx" | "bho" | "awa" | "mag"
        | "hne" | "raj" => ScriptGroup::Devanagari,
        "bn" | "as" | "trp" => ScriptGroup::Bengali,
        "pa" => ScriptGroup::Gurmukhi,
        "gu" => ScriptGroup::Gujarati,
        "or" => ScriptGroup::Oriya,
        "ta" => ScriptGroup::Tamil,
        "te" => ScriptGroup::Telugu,
        "kn" | "tcy" => ScriptGroup::Kannada,
        "ml" => ScriptGroup::Malayalam,
        "si" => ScriptGroup::Sinhala,
        "ar" | "ur" | "fa" | "ps" | "sd" | "ks" => ScriptGroup::Arabic,
        "he" => ScriptGroup::Hebrew,
        "dv" => ScriptGroup::Thaana,
        "bo" => ScriptGroup::Tibetan,
        "sat" => ScriptGroup::OlChiki,
        "mni" => ScriptGroup::MeeteiMayek,
        "th" => ScriptGroup::Thai,
        "lo" => ScriptGroup::Lao,
        "km" => ScriptGroup::Khmer,
        "my" => ScriptGroup::Myanmar,
        "am" => ScriptGroup::Ethiopic,
        "hy" => ScriptGroup::Armenian,
        "ka" => ScriptGroup::Georgian,
        "zh" => ScriptGroup::CjkSc,
        "zh-Hant" => ScriptGroup::CjkTc,
        "ja" => ScriptGroup::CjkJp,
        "ko" => ScriptGroup::CjkKr,
        // Hausa, Igbo, Yoruba, Guarani, Mizo: letters such as ɓ ƙ ị ẹ ẽ ṭ.
        "ha" | "ig" | "yo" | "gn" | "lus" => ScriptGroup::LatinExt,
        "ru" | "uk" | "bg" | "sr" | "mk" | "be" | "kk" | "ky" | "tg" | "mn" => {
            ScriptGroup::Cyrillic
        }
        "el" => ScriptGroup::Greek,
        _ => ScriptGroup::Latin,
    }
}

/// Every table language a site must ship a pack for: the catalogue minus
/// English and the languages that read another table.
pub fn table_languages() -> impl Iterator<Item = &'static str> {
    LANGUAGES
        .iter()
        .map(|l| l.code)
        .filter(|c| *c != "en" && table_language(c) == *c)
}

/// Hash segments (`#menus`, `#auth=LO`, `#l=hi&auth=LO`).
pub fn hash_segments(hash: &str) -> impl Iterator<Item = &str> {
    hash.trim_start_matches('#')
        .split('&')
        .map(str::trim)
        .filter(|p| !p.is_empty())
}

/// `true` for hash segments that are not section names (`l=`, `auth=`, …).
pub fn is_bridge_segment(part: &str) -> bool {
    let lower = part.to_ascii_lowercase();
    lower.starts_with("l=")
        || lower.starts_with("auth=")
        || lower == "lang"
        || lower == "authmenu"
}

/// Catalogue language named by `#l=<code>` (unknown codes → None).
pub fn lang_from_hash(hash: &str) -> Option<&'static str> {
    hash_segments(hash).find_map(|part| {
        let lower = part.to_ascii_lowercase();
        let raw = lower.strip_prefix("l=")?;
        // The code keeps its own case in the catalogue (`zh-Hant`, `pt-BR`).
        let raw = &part[part.len() - raw.len()..];
        language_by_code(raw).map(|l| l.code)
    })
}

/// One `key<TAB>text` line per key; `#` lines are comments. `\n` in a text
/// is a line break (Markdown paragraphs); `\t` a tab; `\\` a backslash.
pub fn parse_pack(text: &str) -> HashMap<String, String> {
    text.lines()
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .filter_map(|line| line.split_once('\t'))
        .map(|(k, v)| (k.trim().to_string(), unescape(v)))
        .collect()
}

/// A pack as the page uses it: scaffold stand-ins dropped, so an unfinished
/// row shows the English (never "@@EN@@ …") in a local preview.
fn runtime_pack(text: &str) -> HashMap<String, String> {
    let mut pack = parse_pack(text);
    pack.retain(|_, v| !v.starts_with(STAND_IN));
    pack
}

pub fn unescape(v: &str) -> String {
    let mut out = String::with_capacity(v.len());
    let mut chars = v.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('n') => out.push('\n'),
            Some('t') => out.push('\t'),
            Some('\\') => out.push('\\'),
            Some(other) => {
                out.push('\\');
                out.push(other);
            }
            None => out.push('\\'),
        }
    }
    out
}

/// `{placeholder}` names in a text, in order (the packs must keep them).
pub fn placeholders(text: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut rest = text;
    while let Some(open) = rest.find('{') {
        let after = &rest[open + 1..];
        let Some(close) = after.find('}') else { break };
        let name = &after[..close];
        if !name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            out.push(name);
        }
        rest = &after[close + 1..];
    }
    out
}

/// Marker the scaffold puts before English in a row nobody has translated
/// yet (`scripts/scaffold-site-translations.sh`); the tests fail on it.
pub const STAND_IN: &str = "@@EN@@ ";

/// Where a language's pack is served from (staged by preview / deploy).
pub fn pack_web_path(lang: &str) -> String {
    format!("/i18n/{}.tsv", table_language(lang))
}

/// Original look of a keyed widget, restored for Latin languages.
struct Original {
    families: Vec<FontFamily>,
    align_x: Option<f64>,
}

/// A site's language state: current language, loaded packs, and which
/// widget shows which `en` key.
#[derive(Default)]
pub struct SiteLang {
    /// Language the page shows now (`en` until a pack has arrived).
    pub shown: String,
    /// Language asked for (hash / picker); may be waiting for its pack.
    pub wanted: String,
    /// Packs by table language.
    packs: HashMap<String, HashMap<String, String>>,
    /// In-flight pack request (id, table language).
    pending: Option<(LiveId, String)>,
    /// Table languages whose pack failed to load (do not retry every frame).
    failed: HashSet<String>,
    /// Widgets met so far → their `en` key (None = not site copy).
    keys: HashMap<WidgetUid, Option<&'static str>>,
    /// Look of each keyed widget before any script font / RTL alignment.
    originals: HashMap<WidgetUid, Original>,
    /// Hash language last seen (to notice a picker change on Signal).
    pub last_hash_lang: Option<String>,
}

impl SiteLang {
    /// `true` while the page shows English.
    pub fn is_english(&self) -> bool {
        self.shown.is_empty() || self.shown == "en"
    }

    /// Current language code (`en` before any choice).
    pub fn code(&self) -> &str {
        if self.shown.is_empty() {
            "en"
        } else {
            &self.shown
        }
    }

    /// The pack for `lang`, if loaded (English has none).
    pub fn pack(&self, lang: &str) -> Option<&HashMap<String, String>> {
        self.packs.get(table_language(lang))
    }

    pub fn has_pack(&self, lang: &str) -> bool {
        lang == "en" || self.packs.contains_key(table_language(lang))
    }

    pub fn insert_pack(&mut self, lang: &str, text: &str) {
        self.packs
            .insert(table_language(lang).to_string(), runtime_pack(text));
    }

    /// Ask for `lang`: returns the table language whose pack must be fetched
    /// first (None = ready to apply now, or a request is already running).
    pub fn want(&mut self, lang: &str) -> Option<String> {
        self.wanted = lang.to_string();
        if self.has_pack(lang) {
            return None;
        }
        let table = table_language(lang).to_string();
        if self.failed.contains(&table) {
            return None;
        }
        if self
            .pending
            .as_ref()
            .is_some_and(|(_, pending)| *pending == table)
        {
            return None;
        }
        Some(table)
    }

    pub fn set_pending(&mut self, id: LiveId, table: String) {
        self.pending = Some((id, table));
    }

    /// A network response for our pack request: store it. Returns true when
    /// it was ours (the caller then applies `wanted`).
    pub fn take_response(&mut self, id: LiveId, body: Option<&[u8]>) -> bool {
        let Some((pending_id, table)) = self.pending.clone() else {
            return false;
        };
        if pending_id != id {
            return false;
        }
        self.pending = None;
        match body.and_then(|b| std::str::from_utf8(b).ok()) {
            Some(text) if text.contains('\t') => {
                self.packs.insert(table, runtime_pack(text));
            }
            _ => {
                log!("site i18n: /i18n/{table}.tsv did not load — staying in English");
                self.failed.insert(table);
            }
        }
        true
    }

    /// `en` text for `key` in the shown language (English when the pack lacks
    /// the key — the tests forbid that from shipping).
    pub fn text(&self, key: &str, english: &str) -> String {
        if self.is_english() {
            return english.to_string();
        }
        let text = self
            .pack(&self.shown)
            .and_then(|p| p.get(key))
            .cloned()
            .unwrap_or_else(|| english.to_string());
        if is_rtl(&self.shown) {
            rtl_marks(&text)
        } else {
            text
        }
    }

    /// Show `lang` on every keyed widget under `root`.
    ///
    /// `key_of(english)` is the site's reverse lookup (`locale::key_for`),
    /// `en_of(key)` its English; `script_font` the family for `lang`'s
    /// script (None for Latin languages). `skip` lists widgets the site
    /// paints from code (their text changes at run time).
    pub fn apply(
        &mut self,
        cx: &mut Cx,
        root: &WidgetRef,
        lang: &str,
        key_of: &dyn Fn(&str) -> Option<&'static str>,
        en_of: &dyn Fn(&str) -> &'static str,
        script_font: Option<&FontFamily>,
        skip: &HashSet<WidgetUid>,
    ) {
        self.shown = lang.to_string();
        let rtl = is_rtl(lang);
        let pack = if lang == "en" {
            None
        } else {
            self.packs.get(table_language(lang)).cloned()
        };
        let mut visit = |w: &WidgetRef, this: &mut SiteLang| {
            let uid = w.widget_uid();
            if skip.contains(&uid) {
                return;
            }
            let key = *this
                .keys
                .entry(uid)
                .or_insert_with(|| key_of(&w.text()));
            let Some(key) = key else { return };
            let english = en_of(key);
            let want = pack
                .as_ref()
                .and_then(|p| p.get(key).cloned())
                .unwrap_or_else(|| english.to_string());
            let want = if rtl { rtl_marks(&want) } else { want };
            if w.text() != want {
                w.set_text(cx, &want);
            }
            this.style(w, uid, script_font, rtl);
        };
        walk(root, &mut |w| visit(w, self));
        root.redraw(cx);
    }

    /// Switch one widget (keyed or painted from code) to `script_font` /
    /// RTL alignment, keeping its own look to restore later.
    pub fn style_widget(&mut self, w: &WidgetRef, script_font: Option<&FontFamily>, rtl: bool) {
        let uid = w.widget_uid();
        self.style(w, uid, script_font, rtl);
    }

    fn style(&mut self, w: &WidgetRef, uid: WidgetUid, script_font: Option<&FontFamily>, rtl: bool) {
        if !self.originals.contains_key(&uid) {
            if script_font.is_none() && !rtl {
                return; // never changed, nothing to restore
            }
            let Some(orig) = original_look(w) else { return };
            self.originals.insert(uid, orig);
        }
        let orig = &self.originals[&uid];
        let families: Vec<FontFamily> = match script_font {
            Some(f) => vec![f.clone(); orig.families.len()],
            None => orig.families.clone(),
        };
        let align_x = if rtl { Some(1.0) } else { orig.align_x };
        set_look(w, &families, align_x);
    }
}

/// Visit every Label, Markdown and Button under `root` (depth first).
pub fn walk(root: &WidgetRef, f: &mut dyn FnMut(&WidgetRef)) {
    root.children(&mut |_, child| {
        if child.borrow::<Markdown>().is_some()
            || child.borrow::<Label>().is_some()
            || child.borrow::<Button>().is_some()
        {
            f(&child);
        }
        walk(&child, &mut *f);
    });
}

fn original_look(w: &WidgetRef) -> Option<Original> {
    if let Some(md) = w.borrow::<Markdown>() {
        return Some(Original {
            families: vec![
                md.draw_text.text_style.font_family.clone(),
                md.text_style_normal.font_family.clone(),
                md.text_style_bold.font_family.clone(),
                md.text_style_italic.font_family.clone(),
                md.text_style_bold_italic.font_family.clone(),
            ],
            align_x: Some(md.paragraph_align_x),
        });
    }
    if let Some(label) = w.borrow::<Label>() {
        return Some(Original {
            families: vec![label.draw_text.text_style.font_family.clone()],
            align_x: Some(label.align.x),
        });
    }
    if let Some(button) = w.borrow::<Button>() {
        return Some(Original {
            families: vec![button.draw_text.text_style.font_family.clone()],
            align_x: None,
        });
    }
    None
}

fn set_look(w: &WidgetRef, families: &[FontFamily], align_x: Option<f64>) {
    if let Some(mut md) = w.borrow_mut::<Markdown>() {
        if families.len() >= 5 {
            md.draw_text.text_style.font_family = families[0].clone();
            md.text_style_normal.font_family = families[1].clone();
            md.text_style_bold.font_family = families[2].clone();
            md.text_style_italic.font_family = families[3].clone();
            md.text_style_bold_italic.font_family = families[4].clone();
        }
        if let Some(x) = align_x {
            md.paragraph_align_x = x;
        }
        return;
    }
    if let Some(mut label) = w.borrow_mut::<Label>() {
        if let Some(f) = families.first() {
            label.draw_text.text_style.font_family = f.clone();
        }
        if let Some(x) = align_x {
            label.align.x = x;
        }
        return;
    }
    if let Some(mut button) = w.borrow_mut::<Button>() {
        if let Some(f) = families.first() {
            button.draw_text.text_style.font_family = f.clone();
        }
    }
}

/// Font family of the hidden prototype label `i18n_fonts.<group>` (None for
/// Latin, or when the site does not carry that prototype).
pub fn script_family(cx: &mut Cx, ui: &WidgetRef, group: ScriptGroup) -> Option<FontFamily> {
    if group == ScriptGroup::Latin {
        return None;
    }
    let proto = ui.widget(cx, &[live_id!(i18n_fonts), group.proto_id()]);
    let label = proto.borrow::<Label>()?;
    Some(label.draw_text.text_style.font_family.clone())
}

// ---- Fitting translated copy to its width ------------------------------------
//
// A language's words can be far longer than the English the page was laid out
// for: a German or Finnish compound wider than a phone (Makepad then breaks it
// inside the word), a Tibetan or Khmer button label wider than the screen.
// These passes measure the text in the font it draws with and shrink it —
// never below `FIT_MIN_SCALE` of the size the page asked for — until it fits
// its own width. Call them after a draw (widths are known then); they return
// whether a size changed, so the caller redraws only then.

/// Smallest a fit pass shrinks text, as a share of the size asked for.
pub const FIT_MIN_SCALE: f32 = 0.7;

/// Markdown at least this size (as the page sets it for the screen) is a
/// headline: `fit_headlines` fits its widest word.
pub const HEADLINE_MIN_PT: f32 = 22.0;

/// Per widget: the font size the page asked for and the one a fit pass set.
/// A current size that is neither means the page set a new base (a tier
/// change rescaled it), which then becomes the base.
pub type FitSizes = HashMap<WidgetUid, (f32, f32)>;

/// What `fit_markdown` makes fit.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Fit {
    /// The whole text on one line (pills, tile words).
    Line,
    /// The widest space-separated word (headlines: they wrap between words).
    Words,
    /// Nothing: put the size the page asked for back.
    Restore,
}

/// Languages whose rows break between syllables or characters, not words:
/// Chinese, Japanese (and the languages that read their tables) and Tibetan
/// (after each tsheg). There is no widest word to fit (`Fit::Restore`).
/// Thai, Lao, Khmer and Burmese are not among them: their packs mark words
/// with U+200B, which `Fit::Words` splits at.
pub fn writes_without_spaces(code: &str) -> bool {
    matches!(table_language(code), "bo" | "zh" | "zh-Hant" | "ja")
}

/// Width of one line of `text` in `draw_text`'s font and size (logical px).
pub fn text_width(cx: &mut Cx, draw_text: &DrawText, text: &str) -> f64 {
    draw_text
        .layout(cx, 0.0, 0.0, None, false, Align::default(), text)
        .size_in_lpxs
        .width as f64
}

fn fit_base(sizes: &FitSizes, uid: WidgetUid, current: f32) -> f32 {
    match sizes.get(&uid) {
        Some(&(base, applied)) if (applied - current).abs() < 0.01 => base,
        _ => current,
    }
}

/// The size that makes text `need` px wide at `base` fit `avail` px.
pub fn fit_size(base: f32, need: f64, avail: f64) -> f32 {
    if need <= avail || need <= 0.0 || avail <= 0.0 {
        base
    } else {
        (base * (avail / need) as f32).max(base * FIT_MIN_SCALE)
    }
}

/// Fit a Markdown into `avail` px as `fit` says. True when its size changed.
pub fn fit_markdown(cx: &mut Cx, w: &WidgetRef, avail: f64, fit: Fit, sizes: &mut FitSizes) -> bool {
    let text = w.text();
    let uid = w.widget_uid();
    let Some(mut md) = w.borrow_mut::<Markdown>() else {
        return false;
    };
    let current = md.text_style_normal.font_size;
    let base = fit_base(sizes, uid, current);
    // Plain body text draws with the normal style: measure in that face.
    let saved = md.draw_text.text_style.clone();
    md.draw_text.text_style = md.text_style_normal.clone();
    md.draw_text.text_style.font_size = base;
    let need = match fit {
        Fit::Line => text_width(cx, &md.draw_text, &text),
        Fit::Words => text
            .split(|c: char| c.is_whitespace() || c == '\u{200B}')
            .filter(|word| !word.is_empty())
            .map(|word| text_width(cx, &md.draw_text, word))
            .fold(0.0, f64::max),
        Fit::Restore => 0.0,
    };
    md.draw_text.text_style = saved;
    let size = fit_size(base, need, avail);
    sizes.insert(uid, (base, size));
    if (size - current).abs() < 0.01 {
        return false;
    }
    md.font_size = size;
    md.draw_text.text_style.font_size = size;
    md.text_style_normal.font_size = size;
    md.text_style_bold.font_size = size;
    md.text_style_italic.font_size = size;
    md.text_style_bold_italic.font_size = size;
    true
}

/// Fit a Button so its whole width fits `avail` px; its padding is what it
/// drew beyond its text. True when its size changed.
pub fn fit_button(cx: &mut Cx, w: &WidgetRef, avail: f64, sizes: &mut FitSizes) -> bool {
    let drawn_w = w.area().rect(cx).size.x;
    let text = w.text();
    let uid = w.widget_uid();
    let Some(mut b) = w.borrow_mut::<Button>() else {
        return false;
    };
    let current = b.draw_text.text_style.font_size;
    let chrome = if drawn_w > 0.0 {
        (drawn_w - text_width(cx, &b.draw_text, &text)).max(0.0)
    } else {
        0.0
    };
    let base = fit_base(sizes, uid, current);
    b.draw_text.text_style.font_size = base;
    let need = text_width(cx, &b.draw_text, &text);
    let size = fit_size(base, need, avail - chrome);
    b.draw_text.text_style.font_size = size;
    sizes.insert(uid, (base, size));
    (size - current).abs() >= 0.01
}

/// Fit every Button inside `row` to the row's width — each on its own, as
/// rows stack on phones.
pub fn fit_buttons_in(cx: &mut Cx, row: &WidgetRef, sizes: &mut FitSizes) -> bool {
    let row_w = row.area().rect(cx).size.x;
    if row_w < 1.0 {
        return false;
    }
    let mut buttons = Vec::new();
    collect_buttons(row, &mut buttons);
    let mut changed = false;
    for b in &buttons {
        changed |= fit_button(cx, b, row_w, sizes);
    }
    changed
}

fn collect_buttons(w: &WidgetRef, out: &mut Vec<WidgetRef>) {
    w.children(&mut |_, child| {
        if child.borrow::<Button>().is_some() {
            out.push(child.clone());
        } else {
            collect_buttons(&child, out);
        }
    });
}

/// Fit every headline-sized Markdown under `root` (`HEADLINE_MIN_PT`) to its
/// own width by its widest word — or put its size back (`Fit::Restore`) for a
/// language written without spaces.
pub fn fit_headlines(cx: &mut Cx, root: &WidgetRef, fit: Fit, sizes: &mut FitSizes) -> bool {
    let mut found = Vec::new();
    collect_headlines(root, sizes, &mut found);
    let mut changed = false;
    for md in &found {
        let w = md.area().rect(cx).size.x;
        if w > 1.0 {
            changed |= fit_markdown(cx, md, w, fit, sizes);
        }
    }
    changed
}

fn collect_headlines(w: &WidgetRef, sizes: &FitSizes, out: &mut Vec<WidgetRef>) {
    w.children(&mut |_, child| {
        let big = child
            .borrow::<Markdown>()
            .map(|md| fit_base(sizes, child.widget_uid(), md.text_style_normal.font_size));
        match big {
            Some(base) if base >= HEADLINE_MIN_PT => out.push(child.clone()),
            Some(_) => {}
            None => collect_headlines(&child, sizes, out),
        }
    });
}

#[cfg(test)]
mod fit_tests {
    use super::*;

    #[test]
    fn fits_shrink_only_as_far_as_needed_and_never_below_the_floor() {
        assert_eq!(fit_size(20.0, 100.0, 200.0), 20.0);
        assert!((fit_size(20.0, 200.0, 180.0) - 18.0).abs() < 1e-4);
        assert!((fit_size(20.0, 400.0, 100.0) - 20.0 * FIT_MIN_SCALE).abs() < 1e-4);
        assert_eq!(fit_size(20.0, 0.0, 100.0), 20.0);
        assert_eq!(fit_size(20.0, 100.0, 0.0), 20.0);
    }

    #[test]
    fn languages_without_spaces_are_the_catalogue_codes() {
        for code in ["bo", "zh", "zh-Hant", "ja"] {
            assert!(writes_without_spaces(code), "{code}");
            assert!(table_languages().any(|c| c == code), "{code} is not a pack");
        }
        // Cantonese reads the Traditional Chinese table.
        assert!(writes_without_spaces("yue"));
        // Spaced or word-marked (U+200B): their widest word is fitted.
        for code in ["en", "hi", "de", "fi", "ko", "ar", "th", "lo", "km", "my"] {
            assert!(!writes_without_spaces(code), "{code}");
        }
    }
}

#[cfg(test)]
mod rtl_mark_tests {
    use super::*;

    const R: char = RLM;

    #[test]
    fn marks_never_change_the_visible_text() {
        for s in [
            "ہر پارسل بک کریں، روانہ کریں اور حساب مکمل کریں — آپ کے اپنے آلات پر ایک کوریئر OS۔",
            "₹ {amount} ادا ہو گئے",
            "Android، Windows اور Linux پر مفت۔",
            "שלום עולם.",
            "انٹرنیٹ بند ہو گیا۔\nکاؤنٹر نہیں رکا۔",
            "+ نقد فروخت",
        ] {
            let marked = rtl_marks(s);
            assert_eq!(marked.replace(R, ""), s);
            // One mark per paragraph, nothing between the words.
            assert_eq!(marked.matches(R).count(), s.split('\n').count(), "{marked:?}");
            assert_eq!(rtl_marks(&marked), marked, "marking twice changes nothing");
        }
    }

    #[test]
    fn every_paragraph_starts_with_the_mark() {
        assert_eq!(rtl_marks("שלום עולם."), format!("{R}שלום עולם."));
        // Latin or a placeholder first: the mark still makes the line RTL.
        assert_eq!(rtl_marks("Kumkaj OS ایک"), format!("{R}Kumkaj OS ایک"));
        assert_eq!(rtl_marks("{min} منٹ"), format!("{R}{{min}} منٹ"));
        assert_eq!(
            rtl_marks("انٹرنیٹ بند ہو گیا۔\nکاؤنٹر نہیں رکا۔"),
            format!("{R}انٹرنیٹ بند ہو گیا۔\n{R}کاؤنٹر نہیں رکا۔")
        );
        // Blank lines separate Markdown paragraphs: they stay blank.
        assert_eq!(rtl_marks("ایک\n\nدو"), format!("{R}ایک\n\n{R}دو"));
        assert_eq!(rtl_marks(""), "");
    }

    #[test]
    fn markdown_block_markers_stay_first() {
        assert_eq!(rtl_marks("+ نقد فروخت"), format!("+ {R}نقد فروخت"));
        assert_eq!(rtl_marks("## عنوان"), format!("## {R}عنوان"));
        assert_eq!(rtl_marks("> - اقتباس"), format!("> - {R}اقتباس"));
        assert_eq!(rtl_marks("1. پہلا"), format!("1. {R}پہلا"));
        // Not markers: a signed number, emphasis, a price.
        assert_eq!(rtl_marks("-5 °C"), format!("{R}-5 °C"));
        assert_eq!(rtl_marks("*نیا* مینو"), format!("{R}*نیا* مینو"));
        assert_eq!(rtl_marks("1,240 ₹"), format!("{R}1,240 ₹"));
    }
}

#[cfg(test)]
pub mod checks {
    //! Pack checks every site runs from its own `locale.rs` tests.
    use super::*;

    /// Read `translations/<code>.tsv` of the site crate at `manifest_dir`.
    pub fn read_pack(manifest_dir: &str, code: &str) -> Option<HashMap<String, String>> {
        let path = std::path::Path::new(manifest_dir)
            .join("translations")
            .join(format!("{code}.tsv"));
        std::fs::read_to_string(path).ok().map(|t| parse_pack(&t))
    }

    /// Keys of a pack in file order.
    pub fn pack_keys(manifest_dir: &str, code: &str) -> Vec<String> {
        let path = std::path::Path::new(manifest_dir)
            .join("translations")
            .join(format!("{code}.tsv"));
        std::fs::read_to_string(path)
            .unwrap_or_default()
            .lines()
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .filter_map(|l| l.split_once('\t').map(|(k, _)| k.trim().to_string()))
            .collect()
    }

    /// Every table language has a pack with every key, in `keys` order, with
    /// the same `{placeholders}`, and no text that is empty.
    pub fn every_language_translates_every_key(
        manifest_dir: &str,
        keys: &[&str],
        en_of: &dyn Fn(&str) -> &'static str,
    ) {
        let mut problems = Vec::new();
        for code in table_languages() {
            let Some(pack) = read_pack(manifest_dir, code) else {
                problems.push(format!("{code}: translations/{code}.tsv missing"));
                continue;
            };
            let order = pack_keys(manifest_dir, code);
            let wanted: Vec<String> = keys.iter().map(|k| k.to_string()).collect();
            if order != wanted {
                let missing: Vec<_> = keys.iter().filter(|k| !pack.contains_key(**k)).collect();
                let extra: Vec<_> = order.iter().filter(|k| !keys.contains(&k.as_str())).collect();
                problems.push(format!(
                    "{code}: keys differ from en() order (missing {missing:?}, extra {extra:?})"
                ));
                continue;
            }
            let stand_ins = keys
                .iter()
                .filter(|k| pack[**k].starts_with(STAND_IN))
                .count();
            if stand_ins > 0 {
                problems.push(format!("{code}: {stand_ins} rows still hold the scaffold's English stand-in"));
                continue;
            }
            for key in keys {
                let value = &pack[*key];
                if value.trim().is_empty() && !en_of(key).trim().is_empty() {
                    problems.push(format!("{code}/{key}: empty"));
                }
                if placeholders(value) != placeholders(en_of(key)) {
                    problems.push(format!("{code}/{key}: placeholders differ from English"));
                }
            }
        }
        assert!(problems.is_empty(), "site packs:\n{}", problems.join("\n"));
    }

    /// A text with real words must not be the same English in more than a
    /// quarter of the packs (a stand-in left by the scaffold).
    pub fn no_text_is_left_in_english(
        manifest_dir: &str,
        keys: &[&str],
        en_of: &dyn Fn(&str) -> &'static str,
        allow: &[&str],
    ) {
        let packs: Vec<(String, HashMap<String, String>)> = table_languages()
            .filter_map(|c| read_pack(manifest_dir, c).map(|p| (c.to_string(), p)))
            .collect();
        let mut problems = Vec::new();
        for key in keys {
            if allow.contains(key) {
                continue;
            }
            let english = en_of(key);
            let words = english
                .split(|c: char| !c.is_alphabetic())
                .filter(|w| w.chars().count() >= 3)
                .count();
            if words < 2 {
                continue;
            }
            let same = packs
                .iter()
                .filter(|(_, p)| p.get(*key).map(String::as_str) == Some(english))
                .count();
            if same * 4 > packs.len() {
                problems.push(format!("{key}: English in {same} of {} packs", packs.len()));
            }
        }
        assert!(problems.is_empty(), "English stand-ins:\n{}", problems.join("\n"));
    }

    /// Every character of every pack has a glyph in the font files the site
    /// draws that language with (`files_for(group)` → crate-relative paths).
    pub fn every_pack_draws_with_its_fonts(
        manifest_dir: &str,
        files_for: &dyn Fn(ScriptGroup) -> Vec<&'static str>,
    ) {
        let mut cache: HashMap<&'static str, Vec<u8>> = HashMap::new();
        let mut missing = std::collections::BTreeMap::<String, std::collections::BTreeSet<char>>::new();
        for code in table_languages() {
            let Some(pack) = read_pack(manifest_dir, code) else { continue };
            let files = files_for(script_group(code));
            for f in &files {
                cache.entry(f).or_insert_with(|| {
                    std::fs::read(std::path::Path::new(manifest_dir).join(f))
                        .unwrap_or_else(|e| panic!("{f}: {e}"))
                });
            }
            let faces: Vec<_> = files
                .iter()
                .map(|f| rustybuzz::ttf_parser::Face::parse(&cache[f], 0).unwrap())
                .collect();
            for value in pack.values() {
                for c in value.chars() {
                    if c.is_whitespace()
                        || c.is_control()
                        || matches!(c, '\u{200b}'..='\u{200f}' | '\u{2060}' | '\u{fe00}'..='\u{fe0f}')
                    {
                        continue;
                    }
                    if !faces.iter().any(|f| f.glyph_index(c).is_some()) {
                        missing.entry(code.to_string()).or_default().insert(c);
                    }
                }
            }
        }
        assert!(missing.is_empty(), "characters the site fonts cannot draw: {missing:?}");
    }

    /// Thai, Lao, Khmer and Burmese copy carries U+200B between words, where
    /// Makepad breaks its rows (vendor patch `line_break_uax14`; the marks
    /// come from `node scripts/mark-words.mjs <code> translations/<code>.tsv`).
    /// A run of more than 24 letters of the script with no mark is copy
    /// written without them.
    pub fn words_are_marked_in_scripts_without_spaces(manifest_dir: &str) {
        let scripts: [(&str, fn(char) -> bool); 4] = [
            ("th", |c| ('\u{0E00}'..='\u{0E7F}').contains(&c)),
            ("lo", |c| ('\u{0E80}'..='\u{0EFF}').contains(&c)),
            ("km", |c| matches!(c, '\u{1780}'..='\u{17FF}' | '\u{19E0}'..='\u{19FF}')),
            ("my", |c| {
                matches!(c, '\u{1000}'..='\u{109F}' | '\u{A9E0}'..='\u{A9FF}' | '\u{AA60}'..='\u{AA7F}')
            }),
        ];
        let mut problems = Vec::new();
        for (code, letter) in scripts {
            let Some(pack) = read_pack(manifest_dir, code) else { continue };
            for (key, value) in &pack {
                let mut run = 0;
                let longest = value
                    .chars()
                    .map(|c| {
                        run = if letter(c) { run + 1 } else { 0 };
                        run
                    })
                    .max()
                    .unwrap_or(0);
                if longest > 24 {
                    problems.push(format!("{code} {key}: {longest} letters without a word mark"));
                }
            }
        }
        problems.sort();
        assert!(
            problems.is_empty(),
            "mark them with `node scripts/mark-words.mjs <code> translations/<code>.tsv`:\n{}",
            problems.join("\n")
        );
    }

    /// Quoted strings after `body:`, `text:` and `empty_text:` in a `ui.rs`
    /// script (the page's visible copy), with `\"` / `\\` unescaped.
    pub fn ui_strings(ui: &str) -> Vec<String> {
        let mut out = Vec::new();
        for prop in ["body:", "text:", "empty_text:"] {
            let mut rest = ui;
            while let Some(at) = rest.find(prop) {
                let before = rest[..at].chars().last();
                rest = &rest[at + prop.len()..];
                // `title.body:` counts; `draw_text:` / `my_body:` do not.
                if before.is_some_and(|c| c.is_alphanumeric() || c == '_') {
                    continue;
                }
                let trimmed = rest.trim_start();
                let Some(body) = trimmed.strip_prefix('"') else { continue };
                let mut s = String::new();
                let mut chars = body.chars();
                while let Some(c) = chars.next() {
                    match c {
                        '"' => break,
                        '\\' => match chars.next() {
                            Some('"') => s.push('"'),
                            Some('\\') => s.push('\\'),
                            // The script tokenizer turns these into the real character.
                            Some('n') => s.push('\n'),
                            Some('t') => s.push('\t'),
                            Some(other) => {
                                s.push('\\');
                                s.push(other);
                            }
                            None => break,
                        },
                        c => s.push(c),
                    }
                }
                out.push(s);
            }
        }
        out
    }

    /// Text that never needs a key: blank, separators, numbers / money /
    /// times, and Font Awesome icon escapes.
    pub fn is_plain(s: &str) -> bool {
        s.contains("\\u{")
            || s.chars().all(|c| {
                c.is_whitespace()
                    || c.is_ascii_digit()
                    || "·•,.:;+-−×%/₹$()#".contains(c)
            })
    }

    /// Every visible `ui.rs` string is an `en` value (`key_of`) or in `skip`.
    pub fn every_ui_string_is_keyed_or_skipped(
        ui: &str,
        key_of: &dyn Fn(&str) -> Option<&'static str>,
        skip: &[&str],
    ) {
        let mut unkeyed: Vec<String> = ui_strings(ui)
            .into_iter()
            .filter(|s| !is_plain(s) && !skip.contains(&s.as_str()) && key_of(s).is_none())
            .collect();
        unkeyed.sort();
        unkeyed.dedup();
        assert!(
            unkeyed.is_empty(),
            "ui.rs copy with no key in locale.rs EN (add it, or to UI_SKIP if it is a name / mock-up):\n{}",
            unkeyed.join("\n")
        );
    }

    fn decode_html(s: &str) -> String {
        s.replace("&amp;", "&")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&quot;", "\"")
            .replace("&#39;", "'")
            .replace("&nbsp;", "\u{a0}")
    }

    fn squash(s: &str) -> String {
        s.split_whitespace().collect::<Vec<_>>().join(" ")
    }

    /// `data-i18n="key"` elements in the site's HTML pages and `T("key",
    /// "English")` calls in its scripts: every key is in `en`, and the
    /// English written there is exactly `en(key)` (so the page, the packs and
    /// the fallback never drift apart).
    pub fn html_and_script_copy_match_en(
        manifest_dir: &str,
        html: &[&str],
        scripts: &[&str],
        en_of: &dyn Fn(&str) -> &'static str,
    ) {
        let mut problems = Vec::new();
        let dir = std::path::Path::new(manifest_dir);
        for file in html {
            let text = std::fs::read_to_string(dir.join(file))
                .unwrap_or_else(|e| panic!("{file}: {e}"));
            let mut rest = text.as_str();
            while let Some(at) = rest.find("data-i18n") {
                rest = &rest[at + "data-i18n".len()..];
                let (attr_kind, after) = match rest.split_once("=\"") {
                    Some((kind, after)) if kind.len() < 20 => (kind.to_string(), after),
                    _ => continue,
                };
                if attr_kind == "-en" || attr_kind.ends_with("-en") {
                    continue;
                }
                let Some((key, after_key)) = after.split_once('"') else { continue };
                let english = en_of(key);
                if english.is_empty() {
                    problems.push(format!("{file}: data-i18n{attr_kind}=\"{key}\" is not in EN"));
                    continue;
                }
                if attr_kind.is_empty() {
                    // Element text: from the end of the opening tag to the next tag.
                    let Some(gt) = after_key.find('>') else { continue };
                    let body = &after_key[gt + 1..];
                    let inner = &body[..body.find('<').unwrap_or(body.len())];
                    if squash(&decode_html(inner)) != squash(english) {
                        problems.push(format!(
                            "{file}: data-i18n=\"{key}\" text {:?} differs from EN {:?}",
                            squash(&decode_html(inner)),
                            english
                        ));
                    }
                }
            }
        }
        for file in scripts {
            let text = std::fs::read_to_string(dir.join(file))
                .unwrap_or_else(|e| panic!("{file}: {e}"));
            let mut rest = text.as_str();
            while let Some(at) = rest.find("T(\"") {
                rest = &rest[at + 3..];
                let Some((key, after)) = rest.split_once('"') else { break };
                let after = after.trim_start();
                let Some(after) = after.strip_prefix(',') else { continue };
                let Some(after) = after.trim_start().strip_prefix('"') else { continue };
                let Some((english, _)) = after.split_once('"') else { continue };
                if en_of(key).is_empty() {
                    problems.push(format!("{file}: T(\"{key}\") is not in EN"));
                } else if en_of(key) != english {
                    problems.push(format!("{file}: T(\"{key}\") English {english:?} differs from EN {:?}", en_of(key)));
                }
            }
        }
        assert!(problems.is_empty(), "HTML / script copy vs EN:\n{}", problems.join("\n"));
    }
}
