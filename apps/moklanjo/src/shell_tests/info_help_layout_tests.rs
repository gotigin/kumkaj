#[cfg(test)]
mod info_help_layout_tests {
    use super::*;

    /// InfoHelp root stays Fill so the trail can absorb leftover row width.
    #[test]
    fn infohelp_default_is_fill_so_trail_absorbs() {
        let body = infohelp_default_body();
        let after = body
            .split("mod.widgets.InfoHelp = set_type_default()")
            .nth(1)
            .unwrap_or(body);
        let first_width = after
            .lines()
            .map(str::trim)
            .find(|l| l.starts_with("width:"))
            .unwrap_or("");
        assert_eq!(
            first_width, "width: Fill",
            "InfoHelp first width must be Fill so trail absorbs leftover, got {first_width:?}"
        );
    }

    /// Title is WrapLabel Fit — Fill title parks ℹ at the far end.
    #[test]
    fn infohelp_title_is_wraplabel_fit() {
        let body = infohelp_default_body();
        assert!(
            body.contains("title := WrapLabel{"),
            "InfoHelp title must be WrapLabel"
        );
        assert!(
            !body.contains("title := Label{"),
            "InfoHelp title must not be Label — Fill+wrap Label NaNs the turtle"
        );
        let title_block = body
            .split("title := WrapLabel{")
            .nth(1)
            .and_then(|s| s.split("tip_btn").next())
            .unwrap_or("");
        assert!(
            title_block.contains("width: Fit"),
            "title WrapLabel must be Fit so ℹ hugs the label, got {title_block:?}"
        );
        assert!(
            !title_block.contains("width: Fill"),
            "title WrapLabel must not be Fill — parks ℹ at the far end"
        );
    }

    #[test]
    fn infohelp_has_fill_trail_after_tip() {
        let body = infohelp_default_body();
        assert!(
            body.contains("trail := View{width: Fill height: Fit"),
            "InfoHelp must have Fill trail after tip so title+ℹ pack left"
        );
    }

    #[test]
    fn draw_walk_packs_title_with_tip_via_trail() {
        let src = include_str!("../info_help.rs");
        assert!(
            src.contains("Size::fill()") && src.contains("Size::fit()"),
            "info_help draw_walk must Size::fill() when titled and Size::fit() when tip-only"
        );
        assert!(
            src.contains("ids!(trail)"),
            "draw_walk must show/hide trail with the title"
        );
    }

    #[test]
    fn accordion_hdr_hit_btn_must_not_wrap() {
        let ui = include_str!("../ui_script.rs");
        let start = ui
            .find("let AccordionHdrHitBtn = Button{")
            .expect("AccordionHdrHitBtn");
        let end = ui[start..]
            .find("let QuietFitBtn = Button{")
            .or_else(|| ui[start..].find("\n    let "))
            .unwrap_or(800);
        let block = &ui[start..start + end];
        assert!(
            !block.contains("wrap: true"),
            "AccordionHdrHitBtn must not use wrap:true — breaks Fit caption+ℹ pack"
        );
    }

    #[test]
    fn tip_rows_use_hit_btn_not_wrapped_accordion_hdr() {
        let ui = include_str!("../ui_script.rs");
        assert!(
            !ui.contains("AccordionHdrBtn{width: Fit"),
            "AccordionHdrBtn{{width: Fit}} + wrap parks ℹ far — use AccordionHdrHitBtn"
        );
        assert!(
            ui.contains("AccordionHdrHitBtn{width: Fit"),
            "tip-row captions must use AccordionHdrHitBtn{{width: Fit}}"
        );
    }

    #[test]
    fn short_tip_labels_are_fit_wraplabel_not_titled_infohelp() {
        let ui = include_str!("../ui_script.rs");
        for id in ["server_pick_title", "lock_login_title"] {
            assert!(
                ui.contains(&format!("{id} := WrapLabel{{")),
                "{id} must be a sibling WrapLabel, not InfoHelp title"
            );
        }
        let pick = ui
            .split("server_pick_title := WrapLabel{")
            .nth(1)
            .and_then(|s| s.split("server_name_input").next())
            .unwrap_or("");
        assert!(
            pick.contains("width: Fit"),
            "server_pick_title WrapLabel must be width: Fit"
        );
        assert!(
            pick.contains("server_pick_help := InfoHelp{"),
            "server_pick_help must follow the Fit WrapLabel"
        );
        assert!(
            !pick.contains("title:"),
            "server_pick_help must be tip-only (no DSL title:)"
        );
    }

    /// SegBtn must not wrap — wrap inside squeezed cells makes irregular ovals.
    #[test]
    fn segbtn_does_not_wrap_label() {
        let ui = include_str!("../ui_script.rs");
        if !ui.contains("let SegBtn = Button{") {
            return;
        }
        let seg = ui
            .split("let SegBtn = Button{")
            .nth(1)
            .and_then(|s| s.split("let SegBtnOn = SegBtn{").next())
            .unwrap_or("");
        assert!(
            !seg.contains("wrap: true"),
            "SegBtn must use flow: Right (no wrap) so phone strips stay even"
        );
        assert!(seg.contains("flow: Right"), "SegBtn must declare flow: Right");
    }

    /// Footer ScrollX must own whitespace lane presses (touch h-scroll).
    #[test]
    fn bottom_nav_scroll_captures_lane_for_hscroll() {
        let ui = include_str!("../ui_script.rs");
        if !ui.contains("bottom_nav_scroll := ScrollXView{") {
            return;
        }
        let nav = ui
            .split("bottom_nav_scroll := ScrollXView{")
            .nth(1)
            .and_then(|s| s.split("nav_books_slot :=").next())
            .unwrap_or("");
        assert!(
            nav.contains("mouse_capture_overload: true"),
            "bottom_nav_scroll needs mouse_capture_overload so lane drags scroll"
        );
        assert!(
            nav.contains("axis_lock: true"),
            "bottom_nav_scroll needs axis_lock so vertical page scroll does not steal"
        );
        assert!(
            nav.contains("bounce_at_start: false") && nav.contains("bounce_at_end: false"),
            "footer bounce fights chip tap targets"
        );
    }

    fn infohelp_default_body() -> &'static str {
        let ui = include_str!("../ui_script.rs");
        let start = ui
            .find("mod.widgets.InfoHelp = set_type_default()")
            .expect("InfoHelp type default");
        // Through the type-default close — fixed byte windows truncate mid
        // `trail :=` when the tip_btn shader grows (UTF-8 comments).
        let end = ui[start..]
            .find("\n    let InfoHelp =")
            .map(|i| start + i)
            .unwrap_or(ui.len());
        &ui[start..end]
    }

    /// Brand title+id must be Fill Down stacks — Fit+Fit Right cropped
    /// "Khaynjo (2282(" on phone chrome (header / Lock / DeviceSetup).
    #[test]
    fn brand_title_stacks_are_fill_not_fit_right_pairs() {
        let ui = include_str!("../ui_script.rs");
        for id in [
            "brand_title := WrapLabel{",
            "lock_brand_title := WrapLabel{",
            "setup_brand_title := WrapLabel{",
        ] {
            let block = ui
                .split(id)
                .nth(1)
                .and_then(|s| s.split('}').next())
                .unwrap_or("");
            assert!(
                block.contains("width: Fill"),
                "{id} must be Fill so long shop names wrap, got {block:?}"
            );
            assert!(
                !block.lines().take(6).any(|l| l.trim() == "width: Fit"),
                "{id} must not be Fit beside a peer id (crops on 375)"
            );
        }
        // No nested Fit+Fit Right row wrapping brand_title again.
        assert!(
            !ui.contains("brand_title := WrapLabel{\n                                                    width: Fit"),
            "brand_title must not sit Fit in a Right peer row"
        );
    }

    /// Word buttons must grow with ink — fixed 34/40px faces clip Gujarati / phone.
    #[test]
    fn text_chip_and_seg_buttons_use_fit_height() {
        let ui = include_str!("../ui_script.rs");
        let chip = ui
            .split("let ChipBtn = Button{")
            .nth(1)
            .and_then(|s| s.split("let ChipBtnActive").next())
            .unwrap_or("");
        assert!(
            chip.contains("height: Fit"),
            "ChipBtn must use Fit height"
        );
        assert!(
            !chip.contains("height: 34"),
            "ChipBtn must not pin height 34"
        );
        // SegBtn: Moklanjo keeps one-line segments (segbtn_does_not_wrap_label).
        assert!(
            !ui.contains("height: 40 flow: Overlay"),
            "never clip SegBtn/ChipBtn in height:40 overlay shells"
        );
    }

    /// Form ScrollX under headings gets vertical margin; ChipStrip does not
    /// (Overlay Lock tabs would float).
    #[test]
    fn form_scroll_x_has_vertical_margin_chipstrip_does_not() {
        let ui = include_str!("../ui_script.rs");
        let form = ui
            .split("let FormScrollX = ScrollXView{")
            .nth(1)
            .and_then(|s| s.split("let ").next())
            .unwrap_or("");
        assert!(
            form.contains("top: 8.") && form.contains("bottom: 8."),
            "FormScrollX must pad top/bottom 8 so strips are not flush"
        );
        let chip = ui
            .split("let ChipStrip = ScrollXView{")
            .nth(1)
            .and_then(|s| s.split("let ").next())
            .unwrap_or("");
        assert!(
            !chip.contains("margin:"),
            "ChipStrip must stay marginless (Overlay chevrons)"
        );
        // Moklanjo has no FormScrollX strip yet; the shared definition stays
        // ready (Kirano / Tabiyat used it undefined until 2026-11-14).
        assert!(ui.contains("let FormScrollX = ScrollXView{"), "FormScrollX must be defined");
    }

    /// Run-on figure lines split into rows; section titles and the words
    /// over figures become headings; a closing sentence stays a line.
    #[test]
    fn report_figures_never_run_on() {
        let rows = parse_report(
            "Cash ₹120.00 · UPI ₹0.00 · Card ₹5.00\nExpenses ₹50.00 (2)\n\nParty outstanding (aging)\nNo balances.\n\nAssets\nBank ₹0.00\nRavi · Customer · ₹500.00 · 3d · current\n\nNo day close recorded yet.",
        );
        assert_eq!(
            rows,
            vec![
                ReportRow::Kv("Cash".into(), "₹120.00".into()),
                ReportRow::Kv("UPI".into(), "₹0.00".into()),
                ReportRow::Kv("Card".into(), "₹5.00".into()),
                ReportRow::Kv("Expenses".into(), "₹50.00 (2)".into()),
                ReportRow::Gap,
                ReportRow::Head("Party outstanding (aging)".into()),
                ReportRow::Line("No balances.".into()),
                ReportRow::Gap,
                ReportRow::Head("Assets".into()),
                ReportRow::Kv("Bank".into(), "₹0.00".into()),
                ReportRow::Line("Ravi · Customer · ₹500.00 · 3d · current".into()),
                ReportRow::Gap,
                ReportRow::Line("No day close recorded yet.".into()),
            ]
        );
    }
}
