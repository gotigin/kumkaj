/// Designed (left-to-right) `align.x` of labels/inputs, captured the
/// first time a right-to-left language right-aligns them, so switching
/// back restores the exact original layout.
static ORIG_TEXT_ALIGN: std::sync::LazyLock<
    std::sync::Mutex<std::collections::HashMap<String, f64>>,
> = std::sync::LazyLock::new(|| std::sync::Mutex::new(std::collections::HashMap::new()));

/// Sideways-scrolling tab strips: (wrapper holding the ‹ › chevrons, the
/// scroll view itself).
const TAB_STRIPS: [(&[LiveId], &[LiveId]); 4] = [
    (ids!(feat_tabs), ids!(feat_tabs.tabs)),
    (ids!(lock_role_tabs), ids!(lock_role_tabs.tabs)),
    (ids!(cat_strip), ids!(cat_strip.tabs)),
    (ids!(bottom_nav_tabs), ids!(bottom_nav_scroll)),
];

#[derive(Script, ScriptHook)]
pub struct App {
    #[live]
    ui: WidgetRef,
}

/// Escape Markdown markers so product/customer strings never italicize or become headings.
fn plain_display_text(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '*' | '_' | '`' | '#' => {
                out.push('\\');
                out.push(c);
            }
            _ => out.push(c),
        }
    }
    out
}
