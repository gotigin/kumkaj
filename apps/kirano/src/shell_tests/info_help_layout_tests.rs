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

    /// Order types: ScrollX + Fit row + Fixed cells — phone pans, desk one line.
    #[test]
    fn otype_strip_is_scrollx_fit_fixed_row() {
        let ui = include_str!("../ui_script.rs");
        if !ui.contains("otype_scroll := ScrollXView{") {
            return;
        }
        assert!(
            ui.contains("otype_strip := View{"),
            "otype_scroll must wrap otype_strip"
        );
        let strip = ui
            .split("otype_strip := View{")
            .nth(1)
            .and_then(|s| s.split("// Ticket:").next())
            .unwrap_or("");
        assert!(
            strip.contains("width: Fit height: Fit"),
            "otype_strip must be Fit so ScrollX content is intrinsic"
        );
        assert!(
            strip.contains("Fixed("),
            "otype cells must be Fixed so labels are not crushed"
        );
        assert!(strip.contains("flow: Right"), "otype_strip must be one Right row");
        assert!(
            !strip.contains("wrap: true"),
            "otype_strip must not use Right{{wrap}}+Fill"
        );
    }

    /// Pay methods: ScrollX + Fit + Fixed — no Cas/h on phone, one line on desk.
    #[test]
    fn pm_strip_is_scrollx_fit_fixed_row() {
        let ui = include_str!("../ui_script.rs");
        if !ui.contains("pm_scroll := ScrollXView{") {
            return;
        }
        let strip = ui
            .split("pm_strip := View{")
            .nth(1)
            .and_then(|s| s.split("split_panel := View{").next())
            .unwrap_or("");
        assert!(strip.contains("width: Fit height: Fit"));
        assert!(strip.contains("Fixed("));
        assert!(strip.contains("flow: Right"));
        for name in ["pm_cash", "pm_upi", "pm_card", "pm_credit", "pm_split"] {
            assert!(
                strip.contains(&format!("{name} := SegBtn")),
                "pm_strip missing {name}"
            );
        }
    }

    /// Reports facets: ScrollX + Fit + Fixed — even stadiums, desk one line.
    #[test]
    fn reports_books_btns_is_scrollx_fit_fixed_row() {
        let ui = include_str!("../ui_script.rs");
        if !ui.contains("reports_books_scroll := ScrollXView{") {
            return;
        }
        let strip = ui
            .split("reports_books_btns := View{")
            .nth(1)
            .and_then(|s| s.split("reports_label :=").next())
            .unwrap_or("");
        assert!(strip.contains("width: Fit height: Fit"));
        assert!(strip.contains("Fixed("));
        assert!(strip.contains("flow: Right"));
        for name in [
            "reports_today",
            "reports_month",
            "reports_aging",
            "reports_balance_sheet",
        ] {
            assert!(
                strip.contains(&format!("{name} := SegBtn")),
                "reports_books_btns missing {name}"
            );
        }
    }

    /// Settings pay methods: same ScrollX + Fit + Fixed as Order pm_strip.
    #[test]
    fn pay_m_strip_is_scrollx_fit_fixed_row() {
        let ui = include_str!("../ui_script.rs");
        if !ui.contains("pay_m_scroll := ScrollXView{") {
            return;
        }
        let strip = ui
            .split("pay_m_strip := View{")
            .nth(1)
            .and_then(|s| s.split("FormCard{").next())
            .unwrap_or("");
        assert!(strip.contains("width: Fit height: Fit"));
        assert!(strip.contains("Fixed("));
        assert!(strip.contains("flow: Right"));
    }

    /// Staff roles: ScrollX + Fit + Fixed — phone pans, desk one line.
    #[test]
    fn staff_role_strip_is_scrollx_fit_fixed_row() {
        let ui = include_str!("../ui_script.rs");
        if !ui.contains("staff_role_scroll := ScrollXView{") {
            return;
        }
        let strip = ui
            .split("staff_role_strip := View{")
            .nth(1)
            .and_then(|s| s.split("staff_copy_hire_input :=").next())
            .unwrap_or("");
        assert!(strip.contains("width: Fit height: Fit"));
        assert!(strip.contains("Fixed("));
        assert!(strip.contains("flow: Right"));
        for name in [
            "staff_role_waiter",
            "staff_role_cashier",
            "staff_role_kitchen",
            "staff_role_manager",
            "staff_role_admin",
        ] {
            assert!(
                strip.contains(&format!("{name} := SegBtn")),
                "staff_role_strip missing {name}"
            );
        }
    }

    /// SegBtn labels wrap on a Fit-height face (25fcce2) — a fixed or
    /// no-wrap face clipped Gujarati / long labels on phone.
    #[test]
    fn segbtn_wraps_on_a_fit_height_face() {
        let ui = include_str!("../ui_script.rs");
        let seg = ui
            .split("let SegBtn = Button{")
            .nth(1)
            .and_then(|s| s.split("let KotRowBtn = Button{").next())
            .unwrap_or("");
        assert!(
            seg.contains("flow: Right{wrap: true}"),
            "SegBtn must wrap labels when squeezed"
        );
        assert!(seg.contains("height: Fit"), "SegBtn must grow with its label");
        assert!(
            !ui.contains("height: 40 flow: Overlay"),
            "never clip SegBtn/ChipBtn in height:40 overlay shells"
        );
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

    /// CRM guest segments must not stack Down as ActionStack SegBtns (desk multi-line).
    #[test]
    fn crm_seg_is_scrollx_fit_fixed_row() {
        let ui = include_str!("../ui_script.rs");
        if !ui.contains("crm_seg_scroll := ScrollXView{") {
            return;
        }
        let strip = ui
            .split("crm_seg_strip := View{")
            .nth(1)
            .and_then(|s| s.split("crm_preview :=").next())
            .unwrap_or("");
        assert!(strip.contains("width: Fit height: Fit"));
        assert!(strip.contains("Fixed("));
        assert!(!strip.contains("wrap: true"));
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
        assert!(
            ui.contains("buy_pay_mode_scroll := FormScrollX{"),
            "Buy pay-mode chips must scroll (not Fill-share SegBtns)"
        );
        // FormScrollX was used but never defined here until 2026-11-14 — a
        // runtime DSL error that silently dropped both strips.
        assert!(ui.contains("let FormScrollX = ScrollXView{"), "FormScrollX must be defined");
        assert!(
            ui.contains("part_methods_scroll := FormScrollX{"),
            "PartsList expense chips must scroll (not Fill-share SegBtns)"
        );
        assert!(
            ui.contains("till_pay_scroll := FormScrollX{"),
            "Take payment chips must scroll on phone widths"
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
        let seg = ui
            .split("let SegBtn = Button{")
            .nth(1)
            .and_then(|s| s.split("let KotRowBtn").next())
            .unwrap_or("");
        assert!(
            seg.contains("flow: Right{wrap: true}"),
            "SegBtn must wrap labels when squeezed"
        );
        assert!(
            !ui.contains("height: 40 flow: Overlay"),
            "never clip SegBtn/ChipBtn in height:40 overlay shells"
        );
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

    /// Cart line Hold / In print / Mark: fixed 34px height clipped labels on phone.
    #[test]
    fn ticket_hold_btn_is_fit_height() {
        let ui = include_str!("../ui_script.rs");
        let block = ui
            .split("let TicketHoldBtn = Button{")
            .nth(1)
            .and_then(|s| s.split("let CartRow").next())
            .unwrap_or("");
        assert!(
            block.contains("height: Fit"),
            "TicketHoldBtn must be Fit height, not fixed 34"
        );
        assert!(
            !block.lines().any(|l| l.trim() == "width: Fit height: 34"),
            "TicketHoldBtn must not pin height: 34"
        );
        let cart = ui
            .split("let CartRow = RoundedView{")
            .nth(1)
            .and_then(|s| s.split("details :=").next())
            .unwrap_or("");
        assert!(
            cart.contains("width: Fill height: Fit")
                && cart.contains("hold_send_btn := TicketHoldBtn"),
            "cart action chip row must Fill-wrap inside the line column"
        );
    }
}
