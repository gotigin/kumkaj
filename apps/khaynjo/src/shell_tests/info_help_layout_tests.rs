#[cfg(test)]
mod info_help_layout_tests {
    /// InfoHelp must stay Fit so title+ℹ pack (gap = tip_btn.margin.left only).
    /// Fill parks ℹ at the row end (lock-screen regression, 2026-11).
    #[test]
    fn infohelp_default_is_fit_pack_not_fill_row_end() {
        let body = infohelp_default_body();
        assert!(
            body.contains("width: Fit"),
            "InfoHelp default must be Fit so title+ℹ pack as one cluster"
        );
        assert!(
            !body.contains("width: Fill\n")
                || body.find("width: Fit").unwrap() < body.find("title :=").unwrap_or(usize::MAX),
            "InfoHelp outer walk must not be Fill (that parks ℹ at the row end)"
        );
        // Outer width line specifically
        let after = body.split("mod.widgets.InfoHelp = set_type_default()").nth(1).unwrap_or(body);
        let first_width = after
            .lines()
            .map(str::trim)
            .find(|l| l.starts_with("width:"))
            .unwrap_or("");
        assert!(
            first_width.starts_with("width: Fit"),
            "InfoHelp first width must be Fit…, got {first_width:?}"
        );
    }

    /// Title is WrapLabel Fit{max:…} — never Label Fill+wrap (DEV bug 7 NaN).
    #[test]
    fn infohelp_title_is_wraplabel_fit_not_fill_label() {
        let body = infohelp_default_body();
        assert!(
            body.contains("title := WrapLabel{"),
            "InfoHelp title must be WrapLabel"
        );
        assert!(
            !body.contains("title := Label{"),
            "InfoHelp title must not be Label — Fill+wrap NaNs the turtle"
        );
        let title_block = body
            .split("title := WrapLabel{")
            .nth(1)
            .and_then(|s| s.split('}').next())
            .unwrap_or("");
        assert!(
            title_block.contains("width: Fit"),
            "title WrapLabel must be Fit (pack with ℹ), got {title_block:?}"
        );
        assert!(
            !title_block.contains("width: Fill"),
            "title WrapLabel must not be Fill (parks ℹ at row end)"
        );
    }

    /// set_title must not force Fill on the widget walk (that undoes Fit pack).
    #[test]
    fn set_title_never_forces_fill_walk() {
        let src = include_str!("../info_help.rs");
        assert!(
            !src.contains("Size::fill()") && !src.contains("Size::Fill"),
            "info_help.rs must not force Fill walk — breaks title+ℹ pack"
        );
    }

    fn infohelp_default_body() -> &'static str {
        let ui = include_str!("../ui_script.rs");
        let start = ui
            .find("mod.widgets.InfoHelp = set_type_default()")
            .expect("InfoHelp type default");
        &ui[start..start + 2800.min(ui.len() - start)]
    }
}
