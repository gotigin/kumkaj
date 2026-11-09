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

    /// Title is WrapLabel Fit — Fill title parks ℹ at the far end (Day book).
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

    /// Fill trail after tip absorbs leftover width (ℹ stays next to title).
    #[test]
    fn infohelp_has_fill_trail_after_tip() {
        let body = infohelp_default_body();
        assert!(
            body.contains("trail := View{width: Fill height: Fit"),
            "InfoHelp must have Fill trail after tip so title+ℹ pack left"
        );
    }

    /// Tip-only → Fit; titled → Fill + trail visible.
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

    /// AccordionHdrHitBtn is used at width: Fit beside tip-only InfoHelp.
    /// Fit+wrap expands the caption and parks ℹ at the far end.
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

    /// Painted AccordionHdrBtn keeps wrap (alone Fill headers). Never use it
    /// at width: Fit beside InfoHelp — that is AccordionHdrHitBtn’s job.
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

    /// Short labels beside ℹ: sibling Fit WrapLabel + tip-only InfoHelp.
    #[test]
    fn short_tip_labels_are_fit_wraplabel_not_titled_infohelp() {
        let ui = include_str!("../ui_script.rs");
        assert!(
            ui.contains("server_pick_title := WrapLabel{"),
            "server_pick_title must be a sibling WrapLabel, not InfoHelp title"
        );
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

    /// Order types: 2×2 equal Fill rows (not Right{wrap}+Fill — turtle coerces).
    #[test]
    fn otype_strip_is_two_by_two_fill_rows() {
        let ui = include_str!("../ui_script.rs");
        let strip = ui
            .split("otype_strip := View{")
            .nth(1)
            .and_then(|s| s.split("// Ticket:").next())
            .unwrap_or("");
        assert!(
            strip.contains("flow: Down"),
            "otype_strip must be Down (2×2), not a single wrap row"
        );
        assert!(
            !strip.contains("wrap: true"),
            "otype_strip must not use Right{{wrap}}+Fill (turtle coerces)"
        );
        assert_eq!(
            strip.matches("otype_dinein := SegBtn").count()
                + strip.matches("otype_takeaway := SegBtn").count()
                + strip.matches("otype_delivery := SegBtn").count()
                + strip.matches("otype_counter := SegBtn").count(),
            4,
            "otype_strip must keep all four SegBtn faces"
        );
    }

    /// Lock PIN: smaller Fit prompt + tip-only InfoHelp (20pt Fill stacked PASSCOD/E).
    #[test]
    fn lock_pin_prompt_is_fit_smaller_type_beside_tip() {
        let ui = include_str!("../ui_script.rs");
        let pin = ui
            .split("pin_prompt := WrapLabel{")
            .nth(1)
            .and_then(|s| s.split("lock_pin_input").next())
            .unwrap_or("");
        assert!(
            pin.contains("width: Fit"),
            "pin_prompt must be Fit so ℹ hugs the label"
        );
        assert!(
            pin.contains("font_size: 15."),
            "pin_prompt must use 15pt so ENTER OWNER PASSCODE fits on phone"
        );
        assert!(
            pin.contains("pin_help := InfoHelp{"),
            "pin_help must be tip-only beside pin_prompt"
        );
        assert!(!pin.contains("title:"), "pin_help must be tip-only (no DSL title:)");
        let login = ui
            .split("lock_login_title := WrapLabel{")
            .nth(1)
            .and_then(|s| s.split("lock_login_help").next())
            .unwrap_or("");
        assert!(
            login.contains("width: Fill"),
            "lock_login_title must be Fill so or_unlock_login wraps"
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
