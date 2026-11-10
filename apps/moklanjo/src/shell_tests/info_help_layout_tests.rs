#[cfg(test)]
mod info_help_layout_tests {
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
}
