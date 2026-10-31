// Makepad script_mod! UI DSL — one macro invocation; cannot be include!-split inside the block.
script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    // Multi-script UI fonts: Makepad falls through FontFamily members when a
    // glyph is missing. Cover Indian scheduled scripts + famous world scripts.
    // `icons` (Font Awesome) supplies ✕ ⌫ ✓ etc. that Noto Sans lacks.
    // CJK (zh/ja/ko) comes from Makepad's LXGW WenKai.
    let WorldFonts = FontFamily{
        latin := FontMember{res: crate_resource("self:resources/fonts/NotoSans-Regular.ttf") asc: -0.1 desc: 0.0}
        devanagari := FontMember{res: crate_resource("self:resources/fonts/NotoSansDevanagari-Regular.ttf") asc: 0.0 desc: 0.0}
        bengali := FontMember{res: crate_resource("self:resources/fonts/NotoSansBengali-Regular.ttf") asc: 0.0 desc: 0.0}
        tamil := FontMember{res: crate_resource("self:resources/fonts/NotoSansTamil-Regular.ttf") asc: 0.0 desc: 0.0}
        telugu := FontMember{res: crate_resource("self:resources/fonts/NotoSansTelugu-Regular.ttf") asc: 0.0 desc: 0.0}
        gujarati := FontMember{res: crate_resource("self:resources/fonts/NotoSansGujarati-Regular.ttf") asc: 0.0 desc: 0.0}
        kannada := FontMember{res: crate_resource("self:resources/fonts/NotoSansKannada-Regular.ttf") asc: 0.0 desc: 0.0}
        malayalam := FontMember{res: crate_resource("self:resources/fonts/NotoSansMalayalam-Regular.ttf") asc: 0.0 desc: 0.0}
        gurmukhi := FontMember{res: crate_resource("self:resources/fonts/NotoSansGurmukhi-Regular.ttf") asc: 0.0 desc: 0.0}
        odia := FontMember{res: crate_resource("self:resources/fonts/NotoSansOriya-Regular.ttf") asc: 0.0 desc: 0.0}
        arabic := FontMember{res: crate_resource("self:resources/fonts/NotoSansArabic-Regular.ttf") asc: 0.0 desc: 0.0}
        hebrew := FontMember{res: crate_resource("self:resources/fonts/NotoSansHebrew-Regular.ttf") asc: 0.0 desc: 0.0}
        thai := FontMember{res: crate_resource("self:resources/fonts/NotoSansThai-Regular.ttf") asc: 0.0 desc: 0.0}
        sinhala := FontMember{res: crate_resource("self:resources/fonts/NotoSansSinhala-Regular.ttf") asc: 0.0 desc: 0.0}
        myanmar := FontMember{res: crate_resource("self:resources/fonts/NotoSansMyanmar-Regular.ttf") asc: 0.0 desc: 0.0}
        khmer := FontMember{res: crate_resource("self:resources/fonts/NotoSansKhmer-Regular.ttf") asc: 0.0 desc: 0.0}
        lao := FontMember{res: crate_resource("self:resources/fonts/NotoSansLao-Regular.ttf") asc: 0.0 desc: 0.0}
        tibetan := FontMember{res: crate_resource("self:resources/fonts/NotoSansTibetan-Regular.ttf") asc: 0.0 desc: 0.0}
        ethiopic := FontMember{res: crate_resource("self:resources/fonts/NotoSansEthiopic-Regular.ttf") asc: 0.0 desc: 0.0}
        thaana := FontMember{res: crate_resource("self:resources/fonts/NotoSansThaana-Regular.ttf") asc: 0.0 desc: 0.0}
        georgian := FontMember{res: crate_resource("self:resources/fonts/NotoSansGeorgian-Regular.ttf") asc: 0.0 desc: 0.0}
        armenian := FontMember{res: crate_resource("self:resources/fonts/NotoSansArmenian-Regular.ttf") asc: 0.0 desc: 0.0}
        meetei := FontMember{res: crate_resource("self:resources/fonts/NotoSansMeeteiMayek-Regular.ttf") asc: 0.0 desc: 0.0}
        olchiki := FontMember{res: crate_resource("self:resources/fonts/NotoSansOlChiki-Regular.ttf") asc: 0.0 desc: 0.0}
        // CJK / icons / emoji: same files Makepad widgets already ships — use
        // that crate so Android/desktop packages do not double-bundle ~48MB.
        chinese := FontMember{res: crate_resource("makepad_widgets:resources/LXGWWenKaiRegular.ttf") asc: 0.0 desc: 0.0}
        icons := FontMember{res: crate_resource("makepad_widgets:resources/fa-solid-900.ttf") asc: 0.0 desc: 0.0}
        emoji := FontMember{res: crate_resource("makepad_widgets:resources/NotoColorEmoji.ttf") asc: 0.0 desc: 0.0}
    }
    let WorldFontsBold = FontFamily{
        latin := FontMember{res: crate_resource("self:resources/fonts/NotoSans-Regular.ttf") asc: -0.1 desc: 0.0}
        devanagari := FontMember{res: crate_resource("self:resources/fonts/NotoSansDevanagari-Regular.ttf") asc: 0.0 desc: 0.0}
        bengali := FontMember{res: crate_resource("self:resources/fonts/NotoSansBengali-Regular.ttf") asc: 0.0 desc: 0.0}
        tamil := FontMember{res: crate_resource("self:resources/fonts/NotoSansTamil-Regular.ttf") asc: 0.0 desc: 0.0}
        telugu := FontMember{res: crate_resource("self:resources/fonts/NotoSansTelugu-Regular.ttf") asc: 0.0 desc: 0.0}
        gujarati := FontMember{res: crate_resource("self:resources/fonts/NotoSansGujarati-Regular.ttf") asc: 0.0 desc: 0.0}
        kannada := FontMember{res: crate_resource("self:resources/fonts/NotoSansKannada-Regular.ttf") asc: 0.0 desc: 0.0}
        malayalam := FontMember{res: crate_resource("self:resources/fonts/NotoSansMalayalam-Regular.ttf") asc: 0.0 desc: 0.0}
        gurmukhi := FontMember{res: crate_resource("self:resources/fonts/NotoSansGurmukhi-Regular.ttf") asc: 0.0 desc: 0.0}
        odia := FontMember{res: crate_resource("self:resources/fonts/NotoSansOriya-Regular.ttf") asc: 0.0 desc: 0.0}
        arabic := FontMember{res: crate_resource("self:resources/fonts/NotoSansArabic-Regular.ttf") asc: 0.0 desc: 0.0}
        hebrew := FontMember{res: crate_resource("self:resources/fonts/NotoSansHebrew-Regular.ttf") asc: 0.0 desc: 0.0}
        thai := FontMember{res: crate_resource("self:resources/fonts/NotoSansThai-Regular.ttf") asc: 0.0 desc: 0.0}
        sinhala := FontMember{res: crate_resource("self:resources/fonts/NotoSansSinhala-Regular.ttf") asc: 0.0 desc: 0.0}
        myanmar := FontMember{res: crate_resource("self:resources/fonts/NotoSansMyanmar-Regular.ttf") asc: 0.0 desc: 0.0}
        khmer := FontMember{res: crate_resource("self:resources/fonts/NotoSansKhmer-Regular.ttf") asc: 0.0 desc: 0.0}
        lao := FontMember{res: crate_resource("self:resources/fonts/NotoSansLao-Regular.ttf") asc: 0.0 desc: 0.0}
        tibetan := FontMember{res: crate_resource("self:resources/fonts/NotoSansTibetan-Regular.ttf") asc: 0.0 desc: 0.0}
        ethiopic := FontMember{res: crate_resource("self:resources/fonts/NotoSansEthiopic-Regular.ttf") asc: 0.0 desc: 0.0}
        thaana := FontMember{res: crate_resource("self:resources/fonts/NotoSansThaana-Regular.ttf") asc: 0.0 desc: 0.0}
        georgian := FontMember{res: crate_resource("self:resources/fonts/NotoSansGeorgian-Regular.ttf") asc: 0.0 desc: 0.0}
        armenian := FontMember{res: crate_resource("self:resources/fonts/NotoSansArmenian-Regular.ttf") asc: 0.0 desc: 0.0}
        meetei := FontMember{res: crate_resource("self:resources/fonts/NotoSansMeeteiMayek-Regular.ttf") asc: 0.0 desc: 0.0}
        olchiki := FontMember{res: crate_resource("self:resources/fonts/NotoSansOlChiki-Regular.ttf") asc: 0.0 desc: 0.0}
        chinese := FontMember{res: crate_resource("makepad_widgets:resources/LXGWWenKaiBold.ttf") asc: 0.0 desc: 0.0}
        icons := FontMember{res: crate_resource("makepad_widgets:resources/fa-solid-900.ttf") asc: 0.0 desc: 0.0}
        emoji := FontMember{res: crate_resource("makepad_widgets:resources/NotoColorEmoji.ttf") asc: 0.0 desc: 0.0}
    }
    mod.theme.font_regular.font_family = WorldFonts
    mod.theme.font_label.font_family = WorldFonts
    mod.theme.font_bold.font_family = WorldFontsBold
    mod.theme.font_italic.font_family = WorldFonts
    mod.theme.font_bold_italic.font_family = WorldFontsBold
    // Widget classes copy the theme face when they are defined, which is
    // before this script runs. Point the classes themselves at WorldFonts
    // so every label, button, and field actually falls through to the
    // script fonts. A theme-only assignment leaves IBM Plex in place and
    // Gujarati, Devanagari, and the other scripts draw as garbage.
    mod.widgets.Label.draw_text.text_style.font_family = WorldFonts
    mod.widgets.Button.draw_text.text_style.font_family = WorldFonts
    mod.widgets.TextInput.draw_text.text_style.font_family = WorldFonts
    mod.widgets.TextInputFlat.draw_text.text_style.font_family = WorldFonts
    // Contrast law: never pair two lights or two darks — theme OR local styles.
    // Palette from apps/moklanjo/assets/moklanjo-app-icon.png (navy C, sky blue, orange, yellow).
    // Light face → dark ink (#0B1220 / #3A4554 / #56575D) in EVERY state.
    // Navy face (#072058) → white ink in EVERY state.
    // Makepad mixes color→color_hover→color_down; mismatched lightness = wash.
    // Orange (#C44E10) is accent ink/fill only — never pale peach on white.
    mod.theme.color_bg_app = #xD6E0EC
    mod.theme.color_app_caption_bar = #x072058
    mod.theme.color_text = #x0B1220
    mod.theme.color_text_hover = #x0B1220
    mod.theme.color_text_focus = #x0B1220
    mod.theme.color_text_down = #x0B1220
    mod.theme.color_text_hl = #x0B1220
    mod.theme.color_text_disabled = #x56575D
    mod.theme.color_text_val = #x3A4554
    mod.theme.color_text_meta = #x56575D
    mod.theme.color_label_inner = #x0B1220
    mod.theme.color_label_inner_hover = #x0B1220
    mod.theme.color_label_inner_down = #x0B1220
    mod.theme.color_label_inner_focus = #x0B1220
    mod.theme.color_label_inner_active = #x0B1220
    mod.theme.color_text_placeholder = #x3A4554
    mod.theme.color_text_placeholder_hover = #x0B1220
    // Caret must be dark on cool/white wells (stock theme uses white = invisible).
    mod.theme.color_text_cursor = #x02030B
    mod.theme.color_cursor = #x02030B
    mod.theme.color_cursor_focus = #x02030B
    mod.theme.color_cursor_empty = #x02030B
    mod.theme.color_cursor_border = #x02030B

    // Stock TextInput caret + placeholder colors are baked from theme at
    // widget-def time (often white on gray wells). Re-bind after our theme so
    // EVERY TextInput shows a visible caret and readable placeholders.
    // Do NOT use set_type_default() here (Android Animator UAF).
    mod.widgets.TextInputFlat = mod.widgets.TextInputFlat{
        blink_speed: 0.5
        draw_cursor +: {
            color: #x02030B
        }
        draw_text +: {
            color: #x0B1220
            color_hover: #x0B1220
            color_focus: #x0B1220
            color_down: #x0B1220
            color_empty: #x56575D
            color_empty_hover: #x3A4554
            color_empty_focus: #x3A4554
        }
    }
    mod.widgets.TextInput = mod.widgets.TextInput{
        blink_speed: 0.5
        draw_cursor +: {
            color: #x02030B
        }
        draw_text +: {
            color: #x0B1220
            color_hover: #x0B1220
            color_focus: #x0B1220
            color_down: #x0B1220
            color_empty: #x56575D
            color_empty_hover: #x3A4554
            color_empty_focus: #x3A4554
        }
    }
    mod.theme.color_icon = #x0B1220
    mod.theme.color_icon_active = #x0B1220
    mod.theme.color_icon_inactive = #x56575D
    mod.theme.color_icon_disabled = #xA8B4C4
    // Cool inset wells — mid gray, never near-white cream on white cards.
    // Inset = white wells on cool canvas (readable placeholders). Never match color_bg_app.
    mod.theme.color_inset = #xffffff
    mod.theme.color_inset_hover = #xEEF3F8
    mod.theme.color_inset_down = #xE0E8F0
    mod.theme.color_inset_focus = #xffffff
    mod.theme.color_inset_empty = #xffffff
    // Solid light faces — color_2 must match color (never transparent wash).
    mod.theme.color_outset = #xffffff
    mod.theme.color_outset_hover = #xEEF3F8
    mod.theme.color_outset_down = #xE0E8F0
    mod.theme.color_outset_focus = #xffffff
    mod.theme.color_outset_1 = #xffffff
    mod.theme.color_outset_1_hover = #xEEF3F8
    mod.theme.color_outset_1_down = #xE0E8F0
    mod.theme.color_outset_1_focus = #xffffff
    // Same stop as color_outset — transparent color_2 washes light-on-light.
    mod.theme.color_outset_2 = #xffffff
    mod.theme.color_outset_2_hover = #xEEF3F8
    mod.theme.color_outset_2_down = #xE0E8F0
    mod.theme.color_outset_2_focus = #xffffff
    // Selection band: opaque mid-blue on light page (plan: visible, not pale wash).
    // Dark ink (#x0B1220) stays readable on this band.
    mod.theme.color_selection = #x2E5BADFF
    mod.theme.color_selection_hover = #x2E5BADFF
    mod.theme.color_selection_down = #x02339EFF
    mod.theme.color_selection_focus = #x2E5BADFF
    mod.theme.color_selection_empty = #x2E5BADFF
    mod.theme.color_selection_disabled = #xE0E8F0

    // Default Label ink. Keep stock flow — forcing Down drops glyphs on Android GLES.
    mod.widgets.Label = set_type_default() do mod.widgets.Label{
        height: Fit
        draw_text +: {
            color: #x0B1220
        }
    }

    // Default Button: light face + dark ink (identical in every pointer state).
    // Set color_2* = color* so the bevel gradient never washes to a second light.
    // Label inset: Makepad 2 Fill + Ellipsis can pin ink to the face edge when only
    // layout padding is set — keep padding *and* label_walk.margin on every face.
    mod.widgets.Button = set_type_default() do mod.widgets.Button{
        align: Center
        padding: Inset{left: 16. right: 16. top: 10. bottom: 10.}
        label_walk: Walk{
            width: Fit
            height: Fit
            margin: Inset{left: 16. right: 16.}
        }
        icon_walk: Walk{width: 0. height: 0.}
        draw_bg +: {
            color: #xffffff
            color_hover: #x072058
            color_down: #x02030B
            color_focus: #xffffff
            color_disabled: #xE0E8F0
            color_2: #xffffff
            color_2_hover: #x072058
            color_2_down: #x02030B
            color_2_focus: #xffffff
            color_2_disabled: #xE0E8F0
            border_color: #xA8B4C4
            border_color_hover: #x072058
            border_color_down: #x02030B
            border_color_focus: #xA8B4C4
            border_radius: 12.
            border_size: 1.
        }
        draw_text +: {
            color: #x0B1220
            color_hover: #xffffff
            color_down: #xffffff
            color_focus: #x0B1220
            color_disabled: #x56575D
        }
    }

    // NEVER call set_type_default on TextInput — it rebuilds Animator apply
    // objects and Android then panics with GenVec use-after-free on NextFrame,
    // freezing the whole UI (PIN keypad included). Style fields via theme +
    // per-instance blink_speed/padding only (see FieldInput).

    // Makepad 2 ScrollYView: wheel/trackpad + app keyboard (↑/↓, PgUp/PgDn).
    // Pages swipe-scroll on touch; a selectable WrapLabel takes its own drag.
    // The page's scrollbar must be seen: the stock handle colour is almost
    // the page colour. A thin handle that darkens on hover / drag.
    // Right padding clears the bar (bar_size 10 + gap) so inputs never sit
    // flush against the scrollbar.
    mod.widgets.ScrollYView = mod.widgets.ViewBase {
        padding: Inset{right: 16.}
        scroll_bars: mod.widgets.ScrollBars {
            show_scroll_x: false show_scroll_y: true
            // Swipe-to-scroll stays on (tablets, phones) — it is what pages
            // always did; the stock definition was the one in effect.
            scroll_bar_y: mod.widgets.ScrollBar{
                drag_scrolling: true
                bounce_at_start: false
                bounce_at_end: false
                bar_size: 10.
                min_handle_size: 40.
                draw_bg +: {
                    size: 6.
                    border_radius: 3.
                    color: #xA8B4C4
                    color_hover: #x072058
                    color_drag: #x02030B
                }
            }
        }
    }
    // Pages say `ScrollYView{…}`: bind the name to the definition above, not
    // the stock one imported earlier (whose scrollbar is nearly invisible).
    let ScrollYView = mod.widgets.ScrollYView

    // Horizontal chip/tab strip. Mouse drag-scroll starts on whitespace
    // around the chips (Makepad: no capture overload for mouse), so a press
    // on a chip stays a tap or nav drag-to-swap; touch still scrolls from a
    // chip. Vertical pages stay touch-only (`drag_scroll_touch_only`).
    mod.widgets.ScrollXView = mod.widgets.ViewBase {
        scroll_bars: mod.widgets.ScrollBars {
            show_scroll_x: true show_scroll_y: false
            scroll_bar_x.drag_scrolling: true
            scroll_bar_x.bar_size: 4.
        }
    }

    // Custom window chrome (Wayland / Windows): stock Makepad uses black
    // min/max/close glyphs on a light caption — invisible on our navy UI.
    mod.widgets.Window = set_type_default() do mod.widgets.Window{
        pass.clear_color: #xD6E0EC
        caption_bar +: {
            draw_bg.color: #x072058
            caption_label +: {
                label +: {
                    draw_text.color: #xffffff
                }
            }
            windows_buttons +: {
                min +: {
                    draw_bg +: {
                        color: #xffffff
                        color_hover: #xffffff
                        color_down: #xD0DAE6
                        bg_color: #00000000
                        bg_color_hover: #x0A3A8C
                        bg_color_down: #x02030B
                    }
                }
                max +: {
                    draw_bg +: {
                        color: #xffffff
                        color_hover: #xffffff
                        color_down: #xD0DAE6
                        bg_color: #00000000
                        bg_color_hover: #x0A3A8C
                        bg_color_down: #x02030B
                    }
                }
                close +: {
                    draw_bg +: {
                        color: #xffffff
                        color_hover: #xffffff
                        color_down: #xffffff
                        bg_color: #00000000
                        bg_color_hover: #xE81123
                        // Dark red — never light pink (#F1707A) under white glyph
                        bg_color_down: #x93000a
                    }
                }
            }
            web_fullscreen +: {
                fullscreen +: {
                    draw_bg +: {
                        color: #xffffff
                        color_hover: #xffffff
                        color_down: #xD0DAE6
                    }
                }
            }
        }
    }

    // Face language (intent first — same job → same look):
    //   Navigate hub jump → HubDestBtn + mark
    //   Select in place → DeskTab / SegBtn / ChipBtn
    //   Submit / do → one RowPrimary / StackPrimary (or PayBtn on ticket)
    //   Quiet alt → RowSecondary / StackSecondary / NavBtn (outlined);
    //                QuietFitBtn for inline Fit actions (Show PIN, NDR reasons)
    //   Destroy → DangerBtn
    // Never put Row*/Stack* (Fill + wrap) at width:Fit — wrap collapses to
    // one letter per line. Use QuietFitBtn instead.
    // Full-width quiet action — wrap, never ellipsis. Not a shouty primary;
    // use RowPrimary/StackPrimary for the single form submit.
    let NavBtn = Button{
        width: Fill height: Fit
        margin: Inset{left: 0. right: 0. top: 0. bottom: 0.}
        padding: Inset{left: 16. right: 16. top: 12. bottom: 12.}
        align: Align{x: 0.5 y: 0.5}
        flow: Right{wrap: true}
        label_walk: Walk{
            width: Fit
            height: Fit
            margin: Inset{left: 16. right: 16.}
        }
        icon_walk: Walk{width: 0. height: 0.}
        draw_bg +: {
            color: #xffffff
            color_hover: #xE0E8F0
            color_down: #xD0DAE6
            color_focus: #xffffff
            color_disabled: #xF0F4F8
            color_2: #xffffff
            color_2_hover: #xE0E8F0
            color_2_down: #xD0DAE6
            color_2_focus: #xffffff
            color_2_disabled: #xF0F4F8
            border_color: #xA8B4C4
            border_color_hover: #x072058
            border_color_down: #x02030B
            border_color_focus: #xA8B4C4
            border_radius: 10.
            border_size: 1.
        }
        draw_text +: {
            color: #x0B1220
            color_hover: #x0B1220
            color_down: #x0B1220
            color_focus: #x0B1220
            color_disabled: #x6B7F9A
            ink_centered: true
            text_style +: {font_size: 15.}
        }
    }

    // Compact back control for More child screens (+ header).
    let BackBtn = Button{
        width: 128 height: 40
        align: Center
        padding: Inset{left: 14. right: 14. top: 8. bottom: 8.}
        label_walk: Walk{
            width: Fit
            height: Fit
            margin: Inset{left: 14. right: 14.}
        }
        icon_walk: Walk{width: 0. height: 0.}
        draw_bg +: {
            color: #x072058
            color_hover: #x0A3A8C
            color_down: #x02030B
            color_focus: #x072058
            color_2: #x072058
            color_2_hover: #x0A3A8C
            color_2_down: #x02030B
            color_2_focus: #x072058
            border_color: #x072058
            border_radius: 12.
            border_size: 0.
        }
        draw_text +: {
            color: #xffffff
            color_hover: #xffffff
            color_down: #xffffff
            color_focus: #xffffff
            text_style +: {font_size: 14.}
        }
    }

    // Side-rail faces. Makepad 2 Button has NO selected state — only
    // hover/down/focus animators. Selection = TabBtnActive visible via Overlay
    // swap. Idle TabBtn must NEVER use the selected navy fill on hover/focus
    // (that made previous + current tabs look active at once).
    // Wrap + no Ellipsis: long/translated labels grow the row instead of clipping.
    let TabBtn = Button{
        width: Fill height: Fit
        align: Align{x: 0.0 y: 0.5}
        flow: Right{wrap: true}
        grab_key_focus: false
        reset_hover_on_click: true
        margin: Inset{top: 0. right: 0. bottom: 0. left: 0.}
        // Left pad clears the colored PNG overlaid at x≈10 (22px mark + gap).
        padding: Inset{left: 40. right: 12. top: 12. bottom: 12.}
        label_walk: Walk{
            width: Fit
            height: Fit
            margin: Inset{left: 0. right: 8.}
        }
        icon_walk: Walk{width: 0. height: 0.}
        draw_bg +: {
            color: #x00000000
            color_hover: #x072058
            color_down: #x02030B
            color_focus: #x00000000
            color_disabled: #x00000000
            color_2: #x00000000
            color_2_hover: #x072058
            color_2_down: #x02030B
            color_2_focus: #x00000000
            border_color: #x00000000
            border_radius: 12.
            border_size: 0.
        }
        draw_text +: {
            color: #x8BB8E8
            color_hover: #xffffff
            color_down: #xffffff
            color_focus: #x8BB8E8
            color_disabled: #x56575D
            text_style +: {font_size: 13.}
        }
    }
    // Hide / show the whole side rail (tablet/desktop chrome only).
    let RailToggleBtn = Button{
        width: Fit height: 36
        align: Align{x: 0.5 y: 0.5}
        grab_key_focus: false
        reset_hover_on_click: true
        margin: Inset{top: 0. right: 0. bottom: 0. left: 0.}
        padding: Inset{left: 10. right: 10. top: 6. bottom: 6.}
        label_walk: Walk{
            width: Fit
            height: Fit
            margin: Inset{left: 0. right: 0.}
        }
        icon_walk: Walk{width: 0. height: 0.}
        draw_bg +: {
            color: #x00000000
            color_hover: #x072058
            color_down: #x02030B
            color_focus: #x00000000
            color_disabled: #x00000000
            color_2: #x00000000
            color_2_hover: #x072058
            color_2_down: #x02030B
            color_2_focus: #x00000000
            border_color: #x00000000
            border_radius: 8.
            border_size: 0.
        }
        draw_text +: {
            color: #x8BB8E8
            color_hover: #xffffff
            color_down: #xffffff
            color_focus: #x8BB8E8
            color_disabled: #x56575D
            text_style +: {font_size: 16.}
        }
    }
    // Bottom-nav chips for expanded More peers (scroll row; Fit width).
    // Fit min/max must be FitBound objects — bare numbers only work on Fill{min:}.
    let NavChip = Button{
        width: Fit{min: FitBound.Abs(96.)} height: 56
        align: Align{x: 0.0 y: 0.5}
        grab_key_focus: false
        reset_hover_on_click: true
        margin: Inset{top: 0. right: 0. bottom: 0. left: 0.}
        padding: Inset{left: 36. right: 12. top: 8. bottom: 8.}
        label_walk: Walk{
            width: Fit
            height: Fit
            margin: Inset{left: 0. right: 2.}
        }
        icon_walk: Walk{width: 0. height: 0.}
        draw_bg +: {
            color: #x00000000
            color_hover: #x072058
            color_down: #x02030B
            color_focus: #x00000000
            color_disabled: #x00000000
            color_2: #x00000000
            color_2_hover: #x072058
            color_2_down: #x02030B
            color_2_focus: #x00000000
            border_color: #x00000000
            border_radius: 12.
            border_size: 0.
        }
        draw_text +: {
            color: #x8BB8E8
            color_hover: #xffffff
            color_down: #xffffff
            color_focus: #x8BB8E8
            color_disabled: #x56575D
            text_style +: {font_size: 12.}
        }
    }
    let NavChipActive = Button{
        width: Fit{min: FitBound.Abs(96.)} height: 56
        align: Align{x: 0.0 y: 0.5}
        grab_key_focus: false
        reset_hover_on_click: true
        margin: Inset{top: 0. right: 0. bottom: 0. left: 0.}
        padding: Inset{left: 36. right: 12. top: 8. bottom: 8.}
        label_walk: Walk{
            width: Fit
            height: Fit
            margin: Inset{left: 0. right: 2.}
        }
        icon_walk: Walk{width: 0. height: 0.}
        draw_bg +: {
            color: #x0A3A8C
            color_hover: #x0A3A8C
            color_down: #x072058
            color_focus: #x0A3A8C
            color_disabled: #x56575D
            color_2: #x0A3A8C
            color_2_hover: #x0A3A8C
            color_2_down: #x072058
            color_2_focus: #x0A3A8C
            border_color: #x00000000
            border_radius: 12.
            border_size: 0.
        }
        draw_text +: {
            color: #xffffff
            color_hover: #xffffff
            color_down: #xffffff
            color_focus: #xffffff
            color_disabled: #xC5DDF5
            text_style +: {font_size: 12.}
        }
    }
    let TabBtnActive = Button{
        width: Fill height: Fit
        align: Align{x: 0.0 y: 0.5}
        flow: Right{wrap: true}
        grab_key_focus: false
        reset_hover_on_click: true
        margin: Inset{top: 0. right: 0. bottom: 0. left: 0.}
        padding: Inset{left: 40. right: 12. top: 12. bottom: 12.}
        label_walk: Walk{
            width: Fit
            height: Fit
            margin: Inset{left: 0. right: 8.}
        }
        icon_walk: Walk{width: 0. height: 0.}
        draw_bg +: {
            color: #x072058
            color_hover: #x072058
            color_down: #x072058
            color_focus: #x072058
            color_disabled: #x072058
            color_2: #x072058
            color_2_hover: #x072058
            color_2_down: #x072058
            color_2_focus: #x072058
            color_2_disabled: #x072058
            border_color: #x072058
            border_radius: 12.
            border_size: 0.
        }
        draw_text +: {
            color: #xffffff
            color_hover: #xffffff
            color_down: #xffffff
            color_focus: #xffffff
            color_disabled: #x56575D
            text_style +: {font_size: 13.}
        }
    }

    // Ticket take-payment only — taller filled CTA; not Amend/Void twin and
    // not a form RowPrimary (those stay r14 / 15px on ActionStacks).
    let PayBtn = Button{
        width: Fill height: Fit
        margin: Inset{left: 0. right: 0. top: 0. bottom: 0.}
        padding: Inset{left: 16. right: 16. top: 16. bottom: 16.}
        align: Align{x: 0.5 y: 0.5}
        flow: Right{wrap: true}
        label_walk: Walk{
            width: Fit
            height: Fit
            margin: Inset{left: 16. right: 16.}
        }
        icon_walk: Walk{width: 0. height: 0.}
        draw_bg +: {
            color: #x0A3A8C
            color_hover: #x072058
            color_down: #x02030B
            color_focus: #x0A3A8C
            color_disabled: #x6B7F9A
            color_2: #x0A3A8C
            color_2_hover: #x072058
            color_2_down: #x02030B
            color_2_focus: #x0A3A8C
            color_2_disabled: #x6B7F9A
            border_color: #xFE8357
            border_color_hover: #xFE8357
            border_color_down: #xC44E10
            border_color_focus: #xFE8357
            border_radius: 12.
            border_size: 2.
        }
        draw_text +: {
            color: #xffffff
            color_hover: #xffffff
            color_down: #xffffff
            color_focus: #xffffff
            color_disabled: #xE0E8F0
            ink_centered: true
            text_style +: {font_size: 16.}
        }
    }

    // Full-width primary CTA (onboard Done / pair) — not a digit pad key.
    let PinOkKey = Button{
        width: Fill height: 56
        margin: Inset{left: 0. right: 0. top: 0. bottom: 0.}
        padding: Inset{left: 8. right: 8. top: 8. bottom: 8.}
        align: Center
        label_walk: Walk{
            width: Fit
            height: Fit
            margin: Inset{left: 8. right: 8.}
        }
        icon_walk: Walk{width: 0. height: 0.}
        draw_bg +: {
            color: #x072058
            color_hover: #x0A3A8C
            color_down: #x02030B
            color_focus: #x072058
            color_2: #x072058
            color_2_hover: #x0A3A8C
            color_2_down: #x02030B
            color_2_focus: #x072058
            border_color: #x072058
            border_radius: 8.
            border_size: 0.
        }
        draw_text +: {
            color: #xffffff
            color_hover: #xffffff
            color_down: #xffffff
            color_focus: #xffffff
            ink_centered: true
            max_lines: 1
            text_style +: {font_size: 18.}
        }
    }

    // Side-by-side row actions — same height/margins so mixed primary/secondary
    // never sit on different baselines (Makepad Fill+theme margin misaligns).
    // Defined as full Button (not PinOkKey alias) so padding always applies.
    // Prefer ActionStack when labels are longer than ~4 chars at 320px width.
    // Row faces wrap (no ellipsis) so half-width ActionRow still shows full text.
    let RowPrimaryBtn = Button{
        width: Fill height: Fit
        margin: Inset{left: 0. right: 0. top: 0. bottom: 0.}
        padding: Inset{left: 16. right: 16. top: 12. bottom: 12.}
        align: Align{x: 0.5 y: 0.5}
        flow: Right{wrap: true}
        label_walk: Walk{
            width: Fit
            height: Fit
            margin: Inset{left: 4. right: 4.}
        }
        icon_walk: Walk{width: 0. height: 0.}
        draw_bg +: {
            color: #x072058
            color_hover: #x0A3A8C
            color_down: #x02030B
            color_focus: #x072058
            color_2: #x072058
            color_2_hover: #x0A3A8C
            color_2_down: #x02030B
            color_2_focus: #x072058
            border_color: #x072058
            border_radius: 14.
            border_size: 0.
        }
        draw_text +: {
            color: #xffffff
            color_hover: #xffffff
            color_down: #xffffff
            color_focus: #xffffff
            ink_centered: true
            text_style +: {font_size: 15.}
        }
    }
    let RowSecondaryBtn = Button{
        width: Fill height: Fit
        margin: Inset{left: 0. right: 0. top: 0. bottom: 0.}
        padding: Inset{left: 16. right: 16. top: 12. bottom: 12.}
        align: Align{x: 0.5 y: 0.5}
        flow: Right{wrap: true}
        label_walk: Walk{
            width: Fit
            height: Fit
            margin: Inset{left: 4. right: 4.}
        }
        icon_walk: Walk{width: 0. height: 0.}
        draw_bg +: {
            color: #xE0E8F0
            color_hover: #x072058
            color_down: #x02030B
            color_focus: #xE0E8F0
            color_2: #xE0E8F0
            color_2_hover: #x072058
            color_2_down: #x02030B
            color_2_focus: #xE0E8F0
            border_color: #xA8B4C4
            border_radius: 8.
            border_size: 1.
        }
        draw_text +: {
            color: #x0B1220
            color_hover: #xffffff
            color_down: #xffffff
            color_focus: #x0B1220
            ink_centered: true
            text_style +: {font_size: 15.}
        }
    }
    // Icon-only eye next to a money figure (or the screen-level header eye).
    let EyeIconBtn = Button{
        width: Fit height: Fit
        margin: Inset{left: 0. right: 0. top: 0. bottom: 0.}
        padding: Inset{left: 6. right: 6. top: 4. bottom: 4.}
        align: Center
        grab_key_focus: false
        reset_hover_on_click: true
        label_walk: Walk{width: Fit height: Fit margin: Inset{left: 0. right: 0.}}
        icon_walk: Walk{width: 0. height: 0.}
        draw_bg +: {
            color: #x00000000
            color_hover: #xE0E8F0
            color_down: #xC8D4E0
            color_focus: #x00000000
            color_2: #x00000000
            color_2_hover: #xE0E8F0
            color_2_down: #xC8D4E0
            color_2_focus: #x00000000
            border_color: #x00000000
            border_radius: 6.
            border_size: 0.
        }
        draw_text +: {
            color: #x072058
            color_hover: #x072058
            color_down: #x072058
            color_focus: #x072058
            text_style +: {font_size: 14.}
        }
        text: "\u{f06e}"
    }
    // Books accordion section header — FA chevron (›/˅) + title.
    // Glyph comes from WorldFonts `icons` (fa-solid); Noto has no ▸/▾.
    // label_walk must be Fit: with width: Fit (title+ℹ rows) a Fill label
    // collapses to zero and the caption disappears (chevron-only row).
    let AccordionHdrBtn = Button{
        width: Fill height: Fit
        margin: Inset{left: 0. right: 0. top: 0. bottom: 0.}
        padding: Inset{left: 6. right: 8. top: 10. bottom: 10.}
        align: Align{x: 0.0 y: 0.5}
        grab_key_focus: false
        reset_hover_on_click: true
        label_walk: Walk{width: Fit height: Fit margin: Inset{left: 0. right: 0.}}
        icon_walk: Walk{width: 0. height: 0.}
        draw_bg +: {
            color: #x00000000
            color_hover: #xE0E8F0
            color_down: #xC8D4E0
            color_focus: #x00000000
            color_2: #x00000000
            color_2_hover: #xE0E8F0
            color_2_down: #xC8D4E0
            color_2_focus: #x00000000
            border_color: #x00000000
            border_radius: 8.
            border_size: 0.
        }
        draw_text +: {
            color: #x072058
            color_hover: #x02030B
            color_down: #x02030B
            color_focus: #x072058
            text_style +: {font_size: 14.}
        }
        text: "TODAY"
    }
    // Inline quiet action — Fit width, NO wrap (Fill+wrap at Fit stacks
    // letters vertically). Same colours as RowSecondary; not a ChipBtn filter.
    let QuietFitBtn = Button{
        width: Fit height: Fit
        margin: Inset{left: 0. right: 0. top: 0. bottom: 0.}
        padding: Inset{left: 14. right: 14. top: 8. bottom: 8.}
        align: Center
        label_walk: Walk{
            width: Fit
            height: Fit
            margin: Inset{left: 4. right: 4.}
        }
        icon_walk: Walk{width: 0. height: 0.}
        draw_bg +: {
            color: #xE0E8F0
            color_hover: #x072058
            color_down: #x02030B
            color_focus: #xE0E8F0
            color_2: #xE0E8F0
            color_2_hover: #x072058
            color_2_down: #x02030B
            color_2_focus: #xE0E8F0
            border_color: #xA8B4C4
            border_radius: 8.
            border_size: 1.
        }
        draw_text +: {
            color: #x0B1220
            color_hover: #xffffff
            color_down: #xffffff
            color_focus: #x0B1220
            ink_centered: true
            max_lines: 1
            text_style +: {font_size: 13.}
        }
    }
    // Destroy — void / refund / delete. Outline danger; never solid primary.
    let DangerBtn = Button{
        width: Fill height: Fit
        margin: Inset{left: 0. right: 0. top: 0. bottom: 0.}
        padding: Inset{left: 16. right: 16. top: 12. bottom: 12.}
        align: Align{x: 0.5 y: 0.5}
        flow: Right{wrap: true}
        label_walk: Walk{
            width: Fit
            height: Fit
            margin: Inset{left: 16. right: 16.}
        }
        icon_walk: Walk{width: 0. height: 0.}
        draw_bg +: {
            color: #xffffff
            color_hover: #xFDECE4
            color_down: #xF8D5C8
            color_focus: #xffffff
            color_2: #xffffff
            color_2_hover: #xFDECE4
            color_2_down: #xF8D5C8
            color_2_focus: #xffffff
            border_color: #xC44E10
            border_color_hover: #xA33D0A
            border_color_down: #x822F08
            border_color_focus: #xC44E10
            border_radius: 8.
            border_size: 1.5
        }
        draw_text +: {
            color: #xC44E10
            color_hover: #xA33D0A
            color_down: #x822F08
            color_focus: #xC44E10
            ink_centered: true
            text_style +: {font_size: 15.}
        }
    }
    // Day-book (and similar hub) destination link — left-aligned label with
    // room for a colored nav mark. Not a stadium chip / SegBtn lookalike.
    let HubDestBtn = Button{
        width: Fill height: Fit
        margin: Inset{left: 0. right: 0. top: 0. bottom: 0.}
        padding: Inset{left: 44. right: 16. top: 12. bottom: 12.}
        align: Align{x: 0.0 y: 0.5}
        flow: Right{wrap: true}
        label_walk: Walk{
            width: Fit
            height: Fit
            margin: Inset{left: 0. right: 8.}
        }
        icon_walk: Walk{width: 0. height: 0.}
        draw_bg +: {
            color: #xffffff
            color_hover: #xE0E8F0
            color_down: #xD0DAE6
            color_focus: #xffffff
            color_2: #xffffff
            color_2_hover: #xE0E8F0
            color_2_down: #xD0DAE6
            color_2_focus: #xffffff
            border_color: #xD0DAE6
            border_radius: 10.
            border_size: 1.
        }
        draw_text +: {
            color: #x0B1220
            color_hover: #x0B1220
            color_down: #x0B1220
            color_focus: #x0B1220
            ink_centered: false
            text_style +: {font_size: 15.}
        }
    }
    // Prefer ActionStack for CTAs. ActionRow only for 2 ultra-short labels
    // (≤4 chars) — Fill NavBtn/PayBtn in a Right row becomes "..." at ~320px.
    let ActionRow = View{
        width: Fill height: Fit
        flow: Right spacing: 8
        align: Align{y: 0.5}
    }
    // Full-width stacked actions — required when labels are longer than ~4 chars.
    let ActionStack = View{
        width: Fill height: Fit
        flow: Down spacing: 14
    }
    // Equal-width fields only (price|unit, etc.). NEVER put NavBtn / Stack*
    // CTAs here — Fit+wrap beside Fill TextInputs collapses to a vertical
    // letter stack ("A d d r a t e"). Put CTAs in ActionStack under the row.
    let FormFieldRow = View{
        width: Fill height: Fit
        flow: Right spacing: 8
        align: Align{y: 0.5}
    }
    // A section of a form: white card, roomy padding, one job per card.
    // Navy-adjacent border (#E0E8F0) — matches StackSecondary / settings chips.
    let FormCard = RoundedView{
        width: Fill height: Fit
        flow: Down spacing: 14
        padding: Inset{left: 18. right: 18. top: 16. bottom: 18.}
        draw_bg.color: #xffffff
        draw_bg.border_radius: 16.
        draw_bg.border_size: 1.
        draw_bg.border_color: #xE0E8F0
    }
    // ActionStack faces — same navy/gray as RowPrimary/Secondary, but height Fit
    // + wrap (no ellipsis). Use for any multi-word stacked CTA (language source,
    // Server/Client Mode, etc.). Never put those labels in ChipBtn.
    let StackPrimaryBtn = Button{
        width: Fill height: Fit
        margin: Inset{left: 0. right: 0. top: 0. bottom: 0.}
        padding: Inset{left: 16. right: 16. top: 12. bottom: 12.}
        align: Align{x: 0.5 y: 0.5}
        flow: Right{wrap: true}
        label_walk: Walk{
            width: Fit
            height: Fit
            margin: Inset{left: 4. right: 4.}
        }
        icon_walk: Walk{width: 0. height: 0.}
        draw_bg +: {
            color: #x072058
            color_hover: #x0A3A8C
            color_down: #x02030B
            color_focus: #x072058
            color_2: #x072058
            color_2_hover: #x0A3A8C
            color_2_down: #x02030B
            color_2_focus: #x072058
            border_color: #x072058
            border_radius: 14.
            border_size: 0.
        }
        draw_text +: {
            color: #xffffff
            color_hover: #xffffff
            color_down: #xffffff
            color_focus: #xffffff
            ink_centered: true
            text_style +: {font_size: 15.}
        }
    }
    let StackSecondaryBtn = Button{
        width: Fill height: Fit
        margin: Inset{left: 0. right: 0. top: 0. bottom: 0.}
        padding: Inset{left: 16. right: 16. top: 12. bottom: 12.}
        align: Align{x: 0.5 y: 0.5}
        flow: Right{wrap: true}
        label_walk: Walk{
            width: Fit
            height: Fit
            margin: Inset{left: 4. right: 4.}
        }
        icon_walk: Walk{width: 0. height: 0.}
        draw_bg +: {
            color: #xE0E8F0
            color_hover: #x072058
            color_down: #x02030B
            color_focus: #xE0E8F0
            color_2: #xE0E8F0
            color_2_hover: #x072058
            color_2_down: #x02030B
            color_2_focus: #xE0E8F0
            border_color: #xA8B4C4
            border_radius: 8.
            border_size: 1.
        }
        draw_text +: {
            color: #x0B1220
            color_hover: #xffffff
            color_down: #xffffff
            color_focus: #x0B1220
            ink_centered: true
            text_style +: {font_size: 15.}
        }
    }

    // FILTER CHIP — Fit width in a ChipStrip (kg/lb, cash/UPI…). Soft tag,
    // not a page tab (DeskTab) and not a radio segment (SegBtn).
    // NEVER use for multi-word CTAs — ActionStack + StackPrimary/Secondary.
    let ChipBtn = Button{
        width: Fit{max: FitBound.Abs(160.)} height: 34
        margin: Inset{left: 0. right: 0. top: 0. bottom: 0.}
        padding: Inset{left: 12. right: 12. top: 6. bottom: 6.}
        align: Center
        label_walk: Walk{
            width: Fit
            height: Fit
            margin: Inset{left: 4. right: 4.}
        }
        icon_walk: Walk{width: 0. height: 0.}
        draw_bg +: {
            color: #xEEF3F8
            color_hover: #xD6E0EC
            color_down: #xC8D2E0
            color_focus: #xEEF3F8
            color_2: #xEEF3F8
            color_2_hover: #xD6E0EC
            color_2_down: #xC8D2E0
            color_2_focus: #xEEF3F8
            border_color: #x00000000
            border_radius: 16.
            border_size: 0.
        }
        draw_text +: {
            color: #x3A4554
            color_hover: #x0B1220
            color_down: #x0B1220
            color_focus: #x3A4554
            ink_centered: true
            max_lines: 1
            text_overflow: TextOverflow.Ellipsis
            text_style +: {font_size: 12.}
        }
    }
    // RADIO SEGMENT — equal Fill share in a Right row (Domestic|International,
    // Cash|UPI|…). Stadium outline; selected = filled. Not a page tab.
    let SegBtn = Button{
        width: Fill height: Fit
        margin: Inset{left: 0. right: 0. top: 0. bottom: 0.}
        padding: Inset{left: 8. right: 8. top: 10. bottom: 10.}
        align: Align{x: 0.5 y: 0.5}
        flow: Right{wrap: true}
        label_walk: Walk{width: Fit height: Fit}
        icon_walk: Walk{width: 0. height: 0.}
        draw_bg +: {
            color: #xffffff
            color_hover: #xEEF3F8
            color_down: #xE0E8F0
            color_focus: #xffffff
            color_2: #xffffff
            color_2_hover: #xEEF3F8
            color_2_down: #xE0E8F0
            color_2_focus: #xffffff
            border_color: #x072058
            border_radius: 22.
            border_size: 1.5
        }
        draw_text +: {
            color: #x072058
            color_hover: #x072058
            color_down: #x072058
            color_focus: #x072058
            ink_centered: true
            text_style +: {font_size: 13.}
        }
    }
    let SegBtnOn = SegBtn{
        draw_bg +: {
            color: #x072058
            color_hover: #x0A3A8C
            color_down: #x02030B
            color_focus: #x072058
            color_2: #x072058
            color_2_hover: #x0A3A8C
            color_2_down: #x02030B
            color_2_focus: #x072058
            border_color: #x072058
            border_radius: 22.
            border_size: 1.5
        }
        draw_text +: {
            color: #xffffff
            color_hover: #xffffff
            color_down: #xffffff
            color_focus: #xffffff
        }
    }

    // Booking desk page tabs (Parcel | Counter) — flat label + underline, not
    // bordered pills / radio chips (those are SegBtn for Domestic|International).
    let DeskTab = Button{
        width: Fill height: Fit
        margin: Inset{left: 0. right: 0. top: 0. bottom: 0.}
        padding: Inset{left: 8. right: 8. top: 12. bottom: 10.}
        align: Align{x: 0.5 y: 0.5}
        flow: Right{wrap: true}
        grab_key_focus: false
        reset_hover_on_click: true
        label_walk: Walk{width: Fit height: Fit}
        icon_walk: Walk{width: 0. height: 0.}
        draw_bg +: {
            color: #x00000000
            color_hover: #x00000000
            color_down: #x00000000
            color_focus: #x00000000
            color_2: #x00000000
            color_2_hover: #x00000000
            color_2_down: #x00000000
            color_2_focus: #x00000000
            border_color: #x00000000
            border_radius: 0.
            border_size: 0.
        }
        draw_text +: {
            color: #x3A4554
            color_hover: #x072058
            color_down: #x072058
            color_focus: #x3A4554
            ink_centered: true
            text_style +: {font_size: 14.}
        }
    }
    let DeskTabOn = DeskTab{
        draw_text +: {
            color: #x072058
            color_hover: #x072058
            color_down: #x02030B
            color_focus: #x072058
            text_style +: {font_size: 14.}
        }
    }
    // Selected filter chip (pair with ChipBtn via Overlay) — same soft pill
    // shape as ChipBtn, filled so it still reads as a chip not a SegBtn.
    let ChipBtnActive = Button{
        width: Fit{max: FitBound.Abs(160.)} height: 34
        margin: Inset{left: 0. right: 0. top: 0. bottom: 0.}
        padding: Inset{left: 12. right: 12. top: 6. bottom: 6.}
        align: Center
        label_walk: Walk{
            width: Fit
            height: Fit
            margin: Inset{left: 4. right: 4.}
        }
        icon_walk: Walk{width: 0. height: 0.}
        draw_bg +: {
            color: #x072058
            color_hover: #x0A3A8C
            color_down: #x02030B
            color_focus: #x072058
            color_2: #x072058
            color_2_hover: #x0A3A8C
            color_2_down: #x02030B
            color_2_focus: #x072058
            border_color: #x00000000
            border_radius: 16.
            border_size: 0.
        }
        draw_text +: {
            color: #xffffff
            color_hover: #xffffff
            color_down: #xffffff
            color_focus: #xffffff
            ink_centered: true
            max_lines: 1
            text_overflow: TextOverflow.Ellipsis
            text_style +: {font_size: 12.}
        }
    }
    // Inner row for ChipStrip — Fit-wide so the strip can scroll horizontally.
    let ChipStripRow = View{
        width: Fit height: Fill
        flow: Right spacing: 8
        align: Align{y: 0.5}
        padding: Inset{left: 0. right: 12. top: 4. bottom: 4.}
    }
    // Horizontally scrollable chip/tab strip. Children: ChipStripRow { ChipBtn… }.
    // ‹ / › at the ends of a sideways-scrolling tab strip.
    // Fills the strip's whole height (taller than the tabs, never smaller),
    // so it reads as the strip's own control, not a stray dot.
    let ChevronBtn = Button{
        width: 46 height: Fill
        margin: Inset{left: 0. right: 0. top: 1. bottom: 1.}
        padding: Inset{left: 0. right: 0. top: 0. bottom: 0.}
        align: Align{x: 0.5 y: 0.5}
        draw_bg +: {
            color: #xffffff
            color_hover: #xE0E8F0
            color_down: #xC8D2E0
            color_focus: #xffffff
            color_2: #xffffff
            color_2_hover: #xE0E8F0
            color_2_down: #xC8D2E0
            color_2_focus: #xffffff
            border_color: #xA8B4C4
            border_radius: 5.
            border_size: 1.
        }
        draw_text +: {
            color: #x072058
            color_hover: #x072058
            color_down: #x072058
            color_focus: #x072058
            text_style +: {font_size: 30.}
        }
    }

    let ChipStrip = ScrollXView{
        width: Fill height: 52
    }

    let RoleChip = Button{
        width: Fill height: 40
        margin: Inset{left: 0. right: 0. top: 0. bottom: 0.}
        padding: Inset{left: 14. right: 14. top: 8. bottom: 8.}
        align: Center
        label_walk: Walk{
            width: Fit
            height: Fit
            margin: Inset{left: 14. right: 14.}
        }
        icon_walk: Walk{width: 0. height: 0.}
        draw_bg +: {
            color: #xE0E8F0
            color_hover: #x072058
            color_down: #x02030B
            color_focus: #xE0E8F0
            color_2: #xE0E8F0
            color_2_hover: #x072058
            color_2_down: #x02030B
            color_2_focus: #xE0E8F0
            border_color: #xA8B4C4
            border_radius: 10.
            border_size: 1.
        }
        draw_text +: {
            color: #x0B1220
            color_hover: #xffffff
            color_down: #xffffff
            color_focus: #x0B1220
            ink_centered: true
            max_lines: 1
            text_overflow: TextOverflow.Ellipsis
            text_style +: {font_size: 13.}
        }
    }

    let RoleChipActive = Button{
        width: Fill height: 40
        margin: Inset{left: 0. right: 0. top: 0. bottom: 0.}
        padding: Inset{left: 14. right: 14. top: 8. bottom: 8.}
        align: Center
        label_walk: Walk{
            width: Fit
            height: Fit
            margin: Inset{left: 14. right: 14.}
        }
        icon_walk: Walk{width: 0. height: 0.}
        draw_bg +: {
            color: #x072058
            color_hover: #x0A3A8C
            color_down: #x02030B
            color_focus: #x072058
            color_2: #x072058
            color_2_hover: #x0A3A8C
            color_2_down: #x02030B
            color_2_focus: #x072058
            border_color: #x072058
            border_radius: 10.
            border_size: 0.
        }
        draw_text +: {
            color: #xffffff
            color_hover: #xffffff
            color_down: #xffffff
            color_focus: #xffffff
            ink_centered: true
            max_lines: 1
            text_overflow: TextOverflow.Ellipsis
            text_style +: {font_size: 13.}
        }
    }


    // Styled field — padding/blink/caret/placeholder for ALL app inputs.
    // White well on cool canvas so placeholders stay readable (never gray-on-gray).
    // Define draw_bg here only — do not mutate Animator color tracks at runtime
    // on Android (use-after-free).
    let FieldInput = TextInput{
        width: Fill
        height: Fit
        // Extra vertical margin so scroll gestures miss the field (fewer
        // accidental keyboard opens mid-scroll).
        margin: Inset{left: 0. right: 0. top: 6. bottom: 6.}
        padding: Inset{left: 14. right: 14. top: 15. bottom: 15.}
        blink_speed: 0.5
        // Soft keyboard: Done/Go fires TextInputAction::Returned → form submit.
        return_key_type: ReturnKeyType.Done
        // Once filled, the placeholder's short form stays as a small label
        // on the top edge so a filled form still says what each box is.
        float_caption: true
        draw_bg +: {
            color: #xffffff
            color_hover: #xEEF3F8
            color_focus: #xffffff
            color_down: #xE0E8F0
            color_empty: #xffffff
            border_color: #x4A5568
            border_color_hover: #x072058
            border_color_focus: #x072058
            border_radius: 12.
            border_size: 1.5
        }
        draw_cursor +: {
            color: #x02030B
        }
        // Flat opaque selection — color_2 must match (no light gradient wash).
        draw_selection +: {
            gradient_fill_horizontal: 0.0
            color: #x2E5BADFF
            color_hover: #x2E5BADFF
            color_focus: #x2E5BADFF
            color_down: #x02339EFF
            color_empty: #x2E5BADFF
            color_2: #x2E5BADFF
            color_2_hover: #x2E5BADFF
            color_2_focus: #x2E5BADFF
            color_2_down: #x02339EFF
            color_2_empty: #x2E5BADFF
        }
        draw_text +: {
            color: #x0B1220
            color_hover: #x0B1220
            color_focus: #x0B1220
            color_down: #x0B1220
            color_empty: #x3A4554
            color_empty_hover: #x0B1220
            color_empty_focus: #x0B1220
            text_style +: {font_size: 16.}
        }
        // Float caption uses draw_caption (not draw_text). Unstyled defaults are
        // light/theme ink → invisible on the white well; value then looks
        // "stuck at the bottom with no label".
        draw_caption +: {
            color: #x3A4554
            text_style +: {font_size: 11.}
        }
    }

    // Compact cart / totals field — same caret as FieldInput; tighter padding.
    let CartField = FieldInput{
        padding: Inset{left: 10. right: 10. top: 12. bottom: 12.}
        draw_bg +: {
            color: #xffffff
            color_hover: #xEEF3F8
            color_focus: #xffffff
            color_down: #xE0E8F0
            color_empty: #xffffff
            border_color: #x4A5568
            border_color_hover: #x02030B
            border_color_focus: #x02030B
            border_radius: 10.
            border_size: 2.
        }
        draw_text +: {
            color: #x0B1220
            color_empty: #x56575D
            color_empty_hover: #x3A4554
            color_empty_focus: #x3A4554
            color_hover: #x0B1220
            color_down: #x0B1220
            color_focus: #x0B1220
            text_style +: {font_size: 15.}
        }
    }

    // Full-width selectable copy (Markdown TextFlow) with real `visible`
    // support — stock Markdown/TextFlow ignore set_visible. Use `body:`.
    // Default: display text is selectable. Only tap targets (buttons,
    // tappable rows) keep Label / selectable: false.
    mod.widgets.WrapLabelBase = #(WrapLabel::register_widget(vm))
    mod.widgets.WrapLabel = set_type_default() do mod.widgets.WrapLabelBase{
        width: Fill
        height: Fit
        padding: 0.
        selectable: true
        paragraph_spacing: 0.
        pre_code_spacing: 0.
        font_size: 14.
        font_color: #x0B1220
        draw_text +: {
            color: #x0B1220
        }
        draw_block +: {
            // Opaque mid-blue (RRGGBBAA) — readable dark ink on top, clear on white page.
            selection_color: #x2E5BADFF
        }
    }
    let WrapLabel = mod.widgets.WrapLabel{}

    // Settings → Sidebar: reorder + show/hide chrome peers.
    // Settings → Sidebar row: icon+name (drag via nav_drag_ghost) +
    // pill toggle switch (grey off / brand on, white thumb). Settings/Lock
    // stay locked on, with an "Always on" caption.
    let SidebarNavRow = Button{
        width: Fill height: Fit
        align: Align{x: 0.0 y: 0.5}
        flow: Right{wrap: true}
        grab_key_focus: false
        reset_hover_on_click: true
        margin: Inset{top: 0. right: 0. bottom: 0. left: 0.}
        // Right pad clears the in-row pill toggle (and "Always on").
        padding: Inset{left: 40. right: 110. top: 12. bottom: 12.}
        label_walk: Walk{
            width: Fit
            height: Fit
            margin: Inset{left: 0. right: 8.}
        }
        icon_walk: Walk{width: 0. height: 0.}
        draw_bg +: {
            color: #xffffff
            color_hover: #x072058
            color_down: #x02030B
            color_focus: #xffffff
            color_disabled: #xEEF3F8
            color_2: #xffffff
            color_2_hover: #x072058
            color_2_down: #x02030B
            color_2_focus: #xffffff
            border_color: #xA8B4C4
            border_radius: 12.
            border_size: 1.
        }
        draw_text +: {
            color: #x0B1220
            color_hover: #xffffff
            color_down: #xffffff
            color_focus: #x0B1220
            color_disabled: #x8A9AAB
            text_style +: {font_size: 13.}
        }
    }
    // Transparent hit target over the pill toggle track + thumb.
    let SidebarToggleHit = Button{
        width: Fill height: Fill
        margin: Inset{top: 0. right: 0. bottom: 0. left: 0.}
        padding: Inset{left: 0. right: 0. top: 0. bottom: 0.}
        grab_key_focus: false
        reset_hover_on_click: true
        label_walk: Walk{width: 0. height: 0.}
        icon_walk: Walk{width: 0. height: 0.}
        draw_bg +: {
            color: #x00000000
            color_hover: #x00000000
            color_down: #x00000000
            color_focus: #x00000000
            color_disabled: #x00000000
            color_2: #x00000000
            color_2_hover: #x00000000
            color_2_down: #x00000000
            color_2_focus: #x00000000
            border_size: 0.
        }
        draw_text +: {
            color: #x00000000
            color_hover: #x00000000
            color_down: #x00000000
            color_focus: #x00000000
            color_disabled: #x00000000
        }
    }
    mod.widgets.SidebarNavListBase = #(SidebarNavList::register_widget(vm))
    mod.widgets.SidebarNavList = set_type_default() do mod.widgets.SidebarNavListBase{
        width: Fill height: Fit
        list := PortalList{
            width: Fill height: Fill
            spacing: 6
            drag_scrolling: false
            bounce_at_start: false
            bounce_at_end: false
            capture_overload: false
            scroll_bar: ScrollBar{bar_size: 0.}
            Item := View{
                width: Fill height: Fit
                flow: Overlay
                // Full-row card; icon left, pill toggle inside on the right.
                face := SidebarNavRow{text: "Books"}
                View{
                    width: Fit height: 48
                    align: Align{x: 0.0 y: 0.5}
                    padding: Inset{left: 10. right: 0. top: 0. bottom: 0.}
                    // No DSL src — paint_sidebar_settings_icons /
                    // load_nav_icon_once supply the real WebP.
                    icon := Image{
                        width: 22 height: 22
                        fit: ImageFit.Stretch
                    }
                }
                View{
                    width: Fill height: 48
                    align: Align{x: 1.0 y: 0.5}
                    padding: Inset{left: 0. right: 10. top: 0. bottom: 0.}
                    vis_toggle := View{
                        width: Fit height: Fit
                        flow: Right
                        spacing: 8
                        align: Align{y: 0.5}
                        always_label := Label{
                            width: Fit height: Fit
                            visible: false
                            text: "Always on"
                            draw_text.color: #x8A9AAB
                            draw_text.text_style.font_size: 12.
                        }
                        // Pill switch: grey track + left thumb (off) /
                        // brand track + right thumb (on).
                        toggle := View{
                            width: 46 height: 28
                            flow: Overlay
                            track_off := RoundedView{
                                width: Fill height: Fill
                                draw_bg.color: #xD0D5DD
                                draw_bg.border_radius: 14.
                            }
                            track_on := RoundedView{
                                width: Fill height: Fill
                                visible: false
                                draw_bg.color: #x072058
                                draw_bg.border_radius: 14.
                            }
                            thumb_off := View{
                                width: Fill height: Fill
                                align: Align{x: 0.0 y: 0.5}
                                padding: Inset{left: 3. right: 0. top: 0. bottom: 0.}
                                RoundedView{
                                    width: 22 height: 22
                                    draw_bg.color: #xffffff
                                    draw_bg.border_radius: 11.
                                }
                            }
                            thumb_on := View{
                                width: Fill height: Fill
                                visible: false
                                align: Align{x: 1.0 y: 0.5}
                                padding: Inset{left: 0. right: 3. top: 0. bottom: 0.}
                                RoundedView{
                                    width: 22 height: 22
                                    draw_bg.color: #xffffff
                                    draw_bg.border_radius: 11.
                                }
                            }
                            toggle_hit := SidebarToggleHit{text: ""}
                        }
                    }
                }
            }
        }
    }


    // Screen title + info tip. Hover or tap the ℹ to open a CalloutTooltip
    // (smart placement: below by default, flips above / clamps to edges).
    // One window-level CalloutTooltip hosts every tip — see app_callout.
    mod.widgets.InfoHelpBase = #(InfoHelp::register_widget(vm))
    mod.widgets.InfoHelp = set_type_default() do mod.widgets.InfoHelpBase{
        // Fit so title+ℹ pack as one cluster; do not stretch ℹ to the row end.
        width: Fit
        height: Fit
        flow: Right
        spacing: 6
        align: Align{x: 0.0 y: 0.5}
        tip: ""
        // Named children use := (same as ProductList.list) — bare `=` is a
        // script assign and leaves title/tip_btn missing at runtime.
        // Colored PNG (same asset as the matching nav tab) — not emoji text.
        icon := Image{
            width: 28 height: 28
            fit: ImageFit.Stretch
            visible: false
        }
        title := Label{
            width: Fit height: Fit
            text: ""
            draw_text.color: #x0B1220
            draw_text.text_style.font_size: 20.
        }
        tip_btn := Button{
            width: 32 height: 32
            text: "ℹ"
            margin: Inset{left: 0. right: 0. top: 0. bottom: 0.}
            padding: Inset{left: 0. right: 0. top: 0. bottom: 0.}
            align: Align{x: 0.5 y: 0.5}
            label_walk: Walk{width: Fit height: Fit}
            icon_walk: Walk{width: 0. height: 0.}
            draw_bg +: {
                color: #xE0E8F0
                color_hover: #x072058
                color_down: #x02030B
                color_focus: #xE0E8F0
                color_2: #xE0E8F0
                color_2_hover: #x072058
                color_2_down: #x02030B
                color_2_focus: #xE0E8F0
                border_color: #xA8B4C4
                border_radius: 16.
                border_size: 1.
            }
            draw_text +: {
                color: #x072058
                color_hover: #xffffff
                color_down: #xffffff
                color_focus: #x072058
                text_style +: {font_size: 15.}
            }
        }
    }
    let InfoHelp = mod.widgets.InfoHelp{}

    // Photo or name-initials avatar. When no photo: navy well + 1–2 letters
    // (see menu_images::name_initials). Size overridden per call site.
    // Khaynjo practise: a Fill label keeps glyphs at the top — wrap Fit
    // initials in a Fill centering view so letters sit in the middle.
    let ThumbAvatar = RoundedView{
        width: 64 height: 64
        margin: Inset{top: 4. bottom: 4. left: 4. right: 4.}
        flow: Overlay
        align: Align{x: 0.5 y: 0.5}
        draw_bg.color: #x072058
        draw_bg.border_radius: 12.
        draw_bg.border_size: 0.
        View{
            width: Fill height: Fill
            align: Align{x: 0.5 y: 0.5}
            thumb_initials := Label{
                width: Fit height: Fit
                text: ""
                draw_text.color: #xffffff
                draw_text.text_style.font_size: 20.
            }
        }
        thumb := Image{
            width: Fill height: Fill
            fit: ImageFit.CropToFill
            visible: false
        }
    }

    // Sell token: Fit height — list row estimate must stay >= natural size.
    let ProductCard = RoundedView{
        width: Fill height: Fit
        padding: Inset{left: 10. right: 10. top: 12. bottom: 12.}
        flow: Down spacing: 8
        align: Align{x: 0.5 y: 0.0}
        draw_bg.color: #xffffff
        draw_bg.border_radius: 16.
        draw_bg.border_size: 2.
        draw_bg.border_color: #x4A5568
        ThumbAvatar{
            width: 64 height: 64
        }
        name := Label{
            width: Fill
            text: "Item"
            align: Align{x: 0.5}
            flow: Right{wrap: false}
            max_lines: 1
            text_overflow: Ellipsis
            draw_text.color: #x0B1220
            draw_text.text_style.font_size: 13.
        }
        price := Label{
            width: Fill
            text: "0"
            align: Align{x: 0.5}
            flow: Right{wrap: false}
            max_lines: 1
            draw_text.color: #xC44E10
            draw_text.text_style.font_size: 15.
        }
        // Terracotta + white — never light-on-light; big enough to tap.
        add := Button{
            text: "ADD"
            width: Fill height: 48
            align: Center
            padding: Inset{left: 16. right: 16. top: 10. bottom: 10.}
            label_walk: Walk{
                width: Fit
                height: Fit
                margin: Inset{left: 16. right: 16.}
            }
            icon_walk: Walk{width: 0. height: 0.}
            draw_bg +: {
                color: #xC44E10
                color_hover: #x822801
                color_down: #x6f2000
                color_focus: #xC44E10
                color_disabled: #xA8B4C4
                color_2: #xC44E10
                color_2_hover: #x822801
                color_2_down: #x6f2000
                color_2_focus: #xC44E10
                color_2_disabled: #xA8B4C4
                border_color: #xC44E10
                border_color_hover: #x822801
                border_color_down: #x6f2000
                border_color_focus: #xC44E10
                border_radius: 12.
                border_size: 0.
            }
            draw_text +: {
                color: #xffffff
                color_hover: #xffffff
                color_down: #xffffff
                color_focus: #xffffff
                color_disabled: #x56575D
                text_style +: {font_size: 15.}
            }
        }
    }

    // Items tab: manage catalogue — remove instead of add-to-cart
    let MenuCard = RoundedView{
        cursor: MouseCursor.Hand
        width: Fill height: Fit
        padding: Inset{left: 12. right: 12. top: 12. bottom: 12.}
        flow: Down spacing: 4
        draw_bg.color: #ffffff
        draw_bg.border_radius: 16.
        draw_bg.border_size: 2.
        draw_bg.border_color: #x4A5568
        name := Label{
            width: Fill
            height: Fit
            padding: 0.
            align: Align{x: 0.5}
            text: "Item"
            flow: Right{wrap: false}
            max_lines: 1
            text_overflow: Ellipsis
            draw_text.color: #x0B1220
            draw_text.text_style.font_size: 18.
        }
        View{
            width: Fill height: Fit
            flow: Right spacing: 10
            align: Align{y: 0.0}
            View{
                width: Fill height: Fit
                flow: Right spacing: 10
                align: Align{y: 0.5}
                ThumbAvatar{
                    width: 56 height: 56
                    margin: Inset{top: 10. bottom: 2. left: 2. right: 4.}
                }
            View{
                width: Fill height: Fit
                flow: Down spacing: 2
                price := Label{
                    width: Fill
                    text: "0"
                    flow: Right{wrap: false}
                    max_lines: 1
                    draw_text.color: #xC44E10
                    draw_text.text_style.font_size: 14.
                }
                stock := Label{
                    width: Fill
                    text: " "
                    flow: Right{wrap: false}
                    max_lines: 1
                    text_overflow: Ellipsis
                    draw_text.color: #x3A4554
                    draw_text.text_style.font_size: 12.
                }
            }
            }
            View{
                width: Fit height: Fit
                flow: Down spacing: 8
                align: Align{x: 0.5}
                remove := Button{
                    text: "✕"
                    width: 48 height: 44
                    align: Align{x: 0.5 y: 0.5}
                    draw_bg +: {
                        color: #x072058
                        color_hover: #x0A3A8C
                        color_down: #x02030B
                        color_focus: #x072058
                        color_2: #x072058
                        color_2_hover: #x0A3A8C
                        color_2_down: #x02030B
                        color_2_focus: #x072058
                        border_color: #x072058
                        border_radius: 16.
                        border_size: 0.
                    }
                    draw_text +: {
                        color: #xffffff
                        color_hover: #ffffff
                        color_down: #xffffff
                        color_focus: #xffffff
                        text_style +: {font_size: 20.}
                    }
                }
                sold_out := Button{
                    text: "Sold Out"
                    width: Fit height: 44
                    align: Align{x: 0.5 y: 0.5}
                    padding: Inset{left: 16. right: 16. top: 8. bottom: 8.}
            draw_bg +: {
                color: #xC44E10
                color_hover: #xDF5E16
                color_down: #x8A320C
                color_focus: #xC44E10
                color_2: #xC44E10
                color_2_hover: #xDF5E16
                color_2_down: #x8A320C
                color_2_focus: #xC44E10
                border_color: #xC44E10
                border_radius: 16.
                border_size: 0.
            }
            draw_text +: {
                color: #xffffff
                color_hover: #xffffff
                color_down: #xffffff
                color_focus: #xffffff
                        text_style +: {font_size: 13.}
                    }
                }
            }
        }
    }

    let CartRow = RoundedView{
        width: Fill height: Fit
        padding: Inset{left: 12. right: 12. top: 12. bottom: 12.}
        flow: Down spacing: 8
        // Mid gray well on white ledger — never white-in-white.
        draw_bg.color: #xE0E8F0
        draw_bg.border_radius: 14.
        draw_bg.border_size: 2.
        draw_bg.border_color: #x4A5568
        label := WrapLabel{
            body: "Item"
            font_color: #x02030B
            font_size: 16.
        }
        // Stitch qty dial: Volume − [N] +  (screen 03 / layout spec qty_stepper)
        RoundedView{
            width: Fill height: Fit
            padding: Inset{left: 10. right: 10. top: 8. bottom: 8.}
            flow: Right spacing: 8
            align: Align{y: 0.5}
            draw_bg.color: #xD6E0EC
            draw_bg.border_radius: 12.
            View{
                width: Fill height: Fit
                flow: Down spacing: 2
                h_qty_cap := Label{
                    text: "QTY"
                    draw_text.color: #x3A4554
                    draw_text.text_style.font_size: 10.
                }
            }
            minus := Button{
                text: "−"
                width: 44 height: 44
                draw_bg +: {
                    color: #xffffff
                    color_hover: #x072058
                    color_down: #x02030B
                    color_focus: #xffffff
                    color_2: #xffffff
                    color_2_hover: #x072058
                    color_2_down: #x02030B
                    color_2_focus: #xffffff
                    border_color: #x4A5568
                    border_color_hover: #x072058
                    border_color_down: #x3A4554
                    border_color_focus: #x4A5568
                    border_radius: 12.
                    border_size: 2.
                }
                draw_text +: {
                    color: #x02030B
                    color_hover: #xffffff
                    color_down: #xffffff
                    color_focus: #x02030B
                    text_style +: {font_size: 22.}
                }
            }
            // As wide as "0.25 kg" needs: a fixed width stacked a weighed
            // quantity into one character per line.
            qty_label := Label{
                width: Fit{min: FitBound.Abs(40.)}
                text: "1"
                draw_text.color: #x02030B
                draw_text.text_style.font_size: 26.
            }
            plus := Button{
                text: "+"
                width: 44 height: 44
                draw_bg +: {
                    color: #x072058
                    color_hover: #x0A3A8C
                    color_down: #x02030B
                    color_focus: #x072058
                    color_2: #x072058
                    color_2_hover: #x0A3A8C
                    color_2_down: #x02030B
                    color_2_focus: #x072058
                    border_color: #x072058
                    border_color_hover: #x0A3A8C
                    border_color_down: #x02030B
                    border_color_focus: #x072058
                    border_radius: 12.
                    border_size: 0.
                }
                draw_text +: {
                    color: #xffffff
                    color_hover: #xffffff
                    color_down: #xffffff
                    color_focus: #xffffff
                    text_style +: {font_size: 22.}
                }
            }
        }
        View{
            width: Fill height: Fit
            flow: Right spacing: 8
            align: Align{y: 0.5}
            h_unit_at_cap := Label{
                text: "Unit @"
                draw_text.color: #x0B1220
                draw_text.text_style.font_size: 13.
            }
            unit_price := CartField{
                empty_text: "0"
                text: "0"
            }
        }
        View{
            width: Fill height: Fit
            flow: Right spacing: 8
            line_actual_label := WrapLabel{
                body: "Actual 0"
                font_color: #x0B1220
                font_size: 13.
            }
            View{
                width: Fill height: Fit
                flow: Down spacing: 4
                h_applied := Label{
                    text: "Applied"
                    draw_text.color: #xC44E10
                    draw_text.text_style.font_size: 11.
                }
                line_total := CartField{
                    empty_text: "0"
                    text: "0"
                }
            }
        }
        mod_row := View{
            width: Fill height: Fit
            flow: Right spacing: 6
            align: Align{y: 0.5}
            mod_prev := RoleChip{text: "‹" width: 36}
            mod_0 := RoleChip{text: ""}
            mod_1 := RoleChip{text: ""}
            mod_2 := RoleChip{text: ""}
            mod_next := RoleChip{text: "›" width: 36}
        }
        note := CartField{
            height: 48
            padding: Inset{left: 10. right: 10. top: 10. bottom: 10.}
            empty_text: "Note (optional)"
            text: ""
            draw_text +: {
                text_style +: {font_size: 14.}
            }
        }
    }

    // Action on a white/light card — ALWAYS navy face + white ink (never default white Button).
    let CardActionBtn = Button{
        width: Fill height: 48
        draw_bg +: {
            color: #x072058
            color_hover: #x0A3A8C
            color_down: #x02030B
            color_focus: #x072058
            color_disabled: #x6B7F9A
            color_2: #x072058
            color_2_hover: #x0A3A8C
            color_2_down: #x02030B
            color_2_focus: #x072058
            color_2_disabled: #x6B7F9A
            border_color: #x072058
            border_color_hover: #x0A3A8C
            border_color_down: #x02030B
            border_color_focus: #x072058
            border_radius: 12.
            border_size: 0.
        }
        draw_text +: {
            color: #xffffff
            color_hover: #xffffff
            color_down: #xffffff
            color_focus: #xffffff
            color_disabled: #xE0E8F0
            text_style +: {font_size: 15.}
        }
    }

    // One booking: AWB and where it goes, when / how paid / by whom under
    // it, the amount at the right — not one run-on line.
    let SaleRow = RoundedView{
        width: Fill height: Fit
        padding: Inset{left: 12. right: 12. top: 10. bottom: 10.}
        flow: Down spacing: 8
        draw_bg.color: #xffffff
        draw_bg.border_radius: 14.
        draw_bg.border_size: 2.
        draw_bg.border_color: #x4A5568
        View{
            width: Fill height: Fit
            flow: Right spacing: 12
            align: Align{y: 0.5}
            View{
                width: Fill height: Fit
                flow: Down spacing: 2
                sale_main := Label{
                    width: Fill
                    text: ""
                    flow: Right{wrap: true}
                    draw_text.color: #x0B1220
                    draw_text.text_style.font_size: 14.
                }
                label := Label{
                    width: Fill
                    text: ""
                    flow: Right{wrap: true}
                    draw_text.color: #x3A4554
                    draw_text.text_style.font_size: 12.
                }
            }
            sale_amount := Label{
                width: Fit
                text: ""
                draw_text.color: #x072058
                draw_text.text_style.font_size: 16.
            }
        }
        pick := CardActionBtn{text: "Select" height: 40}
    }

    // Khaynjo practise: WrapLabel + widget().set_text(plain_display_text).
    // Optional ThumbAvatar for party logo (initials when none).
    let CustomerRow = RoundedView{
        width: Fill height: Fit
        padding: Inset{left: 12. right: 12. top: 10. bottom: 10.}
        flow: Right spacing: 12
        align: Align{y: 0.5}
        draw_bg.color: #xffffff
        draw_bg.border_radius: 14.
        draw_bg.border_size: 2.
        draw_bg.border_color: #x4A5568
        ThumbAvatar{
            width: 48 height: 48
            margin: Inset{top: 0. bottom: 0. left: 0. right: 0.}
        }
        View{
            width: Fill height: Fit
            flow: Down spacing: 2
            label := WrapLabel{
                body: "customer"
                font_color: #x0B1220
                font_size: 14.
            }
            sublabel := WrapLabel{
                body: "id"
                font_color: #x3A4554
                font_size: 11.
            }
        }
        pick := Button{
            text: "Pick"
            width: 72 height: 44
            draw_bg +: {
                color: #x072058
                color_hover: #x0A3A8C
                color_down: #x02030B
                color_focus: #x072058
                color_2: #x072058
                color_2_hover: #x0A3A8C
                color_2_down: #x02030B
                color_2_focus: #x072058
                border_color: #x072058
                border_radius: 12.
                border_size: 0.
            }
            draw_text +: {
                color: #xffffff
                color_hover: #xffffff
                color_down: #xffffff
                color_focus: #xffffff
            }
        }
    }

    // Whole-row language pick (no separate Use button). Same inset law as
    // every other face: padding *and* label_walk.margin. Long labels wrap
    // onto the next line inside that inset (no ellipsis / crop).
    let LanguageRow = Button{
        width: Fill height: Fit
        align: Align{x: 0.0 y: 0.5}
        flow: Right{wrap: true}
        padding: Inset{left: 16. right: 16. top: 12. bottom: 12.}
        label_walk: Walk{
            width: Fit
            height: Fit
            margin: Inset{left: 16. right: 16.}
        }
        icon_walk: Walk{width: 0. height: 0.}
        enable_long_press: true
        draw_bg +: {
            color: #xffffff
            color_hover: #x072058
            color_down: #x02030B
            color_focus: #xffffff
            color_2: #xffffff
            color_2_hover: #x072058
            color_2_down: #x02030B
            color_2_focus: #xffffff
            border_color: #x4A5568
            border_color_hover: #x072058
            border_color_down: #x3A4554
            border_color_focus: #x4A5568
            border_radius: 14.
            border_size: 2.
        }
        draw_text +: {
            color: #x0B1220
            color_hover: #xffffff
            color_down: #xffffff
            color_focus: #x0B1220
            ink_centered: false
            text_style +: {font_size: 14.}
        }
    }

    let LocaleRow = RoundedView{
        width: Fill height: Fit
        padding: Inset{left: 16. right: 10. top: 10. bottom: 10.}
        flow: Right spacing: 10
        align: Align{y: 0.5}
        draw_bg.color: #xffffff
        draw_bg.border_radius: 14.
        draw_bg.border_size: 2.
        draw_bg.border_color: #x4A5568
        label := WrapLabel{
            width: Fill
            body: "locale"
            font_color: #x0B1220
            font_size: 14.
        }
        pick := Button{
            text: "Use"
            width: 80 height: 48
            draw_bg +: {
                color: #x072058
                color_hover: #x0A3A8C
                color_down: #x02030B
                color_focus: #x072058
                color_2: #x072058
                color_2_hover: #x0A3A8C
                color_2_down: #x02030B
                color_2_focus: #x072058
                border_color: #x072058
                border_radius: 12.
                border_size: 0.
            }
            draw_text +: {
                color: #xffffff
                color_hover: #xffffff
                color_down: #xffffff
                color_focus: #xffffff
            }
        }
    }

    let MoreRow = Button{
        width: Fill height: Fit
        align: Align{x: 0.0 y: 0.5}
        flow: Right{wrap: true}
        padding: Inset{left: 16. right: 16. top: 14. bottom: 14.}
        label_walk: Walk{
            width: Fit
            height: Fit
            margin: Inset{left: 16. right: 16.}
        }
        icon_walk: Walk{width: 0. height: 0.}
        // So a touch-drag that scrolls the page is not also treated as a click.
        enable_long_press: true
        draw_bg +: {
            color: #xffffff
            color_hover: #x072058
            color_down: #x02030B
            color_focus: #xffffff
            color_2: #xffffff
            color_2_hover: #x072058
            color_2_down: #x02030B
            color_2_focus: #xffffff
            border_color: #x4A5568
            border_color_hover: #x072058
            border_color_down: #x3A4554
            border_color_focus: #x4A5568
            border_radius: 16.
            border_size: 2.
        }
        draw_text +: {
            color: #x0B1220
            color_hover: #xffffff
            color_down: #xffffff
            color_focus: #x0B1220
            ink_centered: false
            text_style +: {font_size: 16.}
        }
    }

    // Lists size to content so the parent page ScrollYView scrolls form + rows together.
    // These PortalLists must NOT scroll themselves — otherwise wheel/drag over the
    // list eats the gesture and the page bottom (next cart lines, pay) is unreachable.
    mod.widgets.ProductListBase = #(ProductList::register_widget(vm))
    mod.widgets.ProductList = set_type_default() do mod.widgets.ProductListBase{
        width: Fill height: Fit
        list := PortalList{
            width: Fill height: Fill
            spacing: 8
            drag_scrolling: false
            bounce_at_start: false
            bounce_at_end: false
            capture_overload: false
            scroll_bar: ScrollBar{bar_size: 0.}
            Item := View{
                width: Fill height: Fit
                flow: Right spacing: 12
                padding: Inset{bottom: 8.}
                texture_caching: false
                c0 := ProductCard{}
                c1 := ProductCard{}
                c2 := ProductCard{}
            }
        }
    }

    mod.widgets.MenuListBase = #(MenuList::register_widget(vm))
    mod.widgets.MenuList = set_type_default() do mod.widgets.MenuListBase{
        width: Fill height: Fit
        list := PortalList{
            width: Fill height: Fill
            spacing: 10
            drag_scrolling: false
            bounce_at_start: false
            bounce_at_end: false
            capture_overload: false
            scroll_bar: ScrollBar{bar_size: 0.}
            Item := View{
                width: Fill height: Fit
                padding: Inset{bottom: 8.}
                texture_caching: false
                card := MenuCard{}
            }
        }
    }

    mod.widgets.CartListBase = #(CartList::register_widget(vm))
    mod.widgets.CartList = set_type_default() do mod.widgets.CartListBase{
        width: Fill height: Fit
        list := PortalList{
            width: Fill height: Fill
            spacing: 8
            drag_scrolling: false
            bounce_at_start: false
            bounce_at_end: false
            capture_overload: false
            scroll_bar: ScrollBar{bar_size: 0.}
            Item := View{
                width: Fill height: Fit
                padding: Inset{bottom: 8.}
                texture_caching: false
                CartRow{}
            }
        }
    }

    mod.widgets.SaleListBase = #(SaleList::register_widget(vm))
    mod.widgets.SaleList = set_type_default() do mod.widgets.SaleListBase{
        width: Fill height: Fit
        list := PortalList{
            width: Fill height: Fill
            spacing: 8
            drag_scrolling: false
            bounce_at_start: false
            bounce_at_end: false
            capture_overload: false
            scroll_bar: ScrollBar{bar_size: 0.}
            Item := View{
                width: Fill height: Fit
                padding: Inset{bottom: 8.}
                texture_caching: false
                SaleRow{}
            }
        }
    }

    // One "type or pick" row: value, a small distinguishing line, and Use.
    // Plain Labels — selectable text would steal the row tap.
    let SuggestRowView = RoundedView{
        cursor: MouseCursor.Hand
        width: Fill height: Fit
        padding: Inset{left: 14. right: 14. top: 12. bottom: 12.}
        flow: Right spacing: 10
        align: Align{y: 0.5}
        draw_bg.color: #xffffff
        draw_bg.border_radius: 10.
        draw_bg.border_size: 1.
        draw_bg.border_color: #x4A5568
        title := Label{
            width: Fill
            text: ""
            max_lines: 1
            text_overflow: Ellipsis
            draw_text.color: #x0B1220
            draw_text.text_style.font_size: 15.
        }
        subtitle := Label{
            width: Fit{max: FitBound.Abs(96.)}
            text: ""
            max_lines: 1
            text_overflow: Ellipsis
            draw_text.color: #x3A4554
            draw_text.text_style.font_size: 13.
        }
        action := Label{
            width: Fit
            text: "Use"
            draw_text.color: #xC44E10
            draw_text.text_style.font_size: 13.
        }
    }
    mod.widgets.SuggestListBase = #(SuggestList::register_widget(vm))
    mod.widgets.SuggestList = set_type_default() do mod.widgets.SuggestListBase{
        width: Fill height: Fit
        list := PortalList{
            width: Fill height: Fill
            spacing: 6
            scroll_bar: ScrollBar{}
            Item := View{
                width: Fill height: Fit
                texture_caching: false
                row := SuggestRowView{}
            }
        }
    }

    mod.widgets.CustomerListBase = #(CustomerList::register_widget(vm))
    mod.widgets.CustomerList = set_type_default() do mod.widgets.CustomerListBase{
        width: Fill height: Fit
        margin: Inset{top: 4. bottom: 4.}
        // Page-scroll like SaleList: full content height, no inner scroll.
        // A capped nested PortalList left a stuck scroll offset → huge blank
        // gap under "Saved parties" until you scrub the list.
        list := PortalList{
            width: Fill height: Fill
            spacing: 6
            drag_scrolling: false
            bounce_at_start: false
            bounce_at_end: false
            capture_overload: false
            scroll_bar: ScrollBar{bar_size: 0.}
            Item := View{
                width: Fill height: Fit
                padding: Inset{bottom: 8.}
                texture_caching: false
                CustomerRow{}
            }
        }
    }

    mod.widgets.LanguageListBase = #(LanguageList::register_widget(vm))
    mod.widgets.LanguageList = set_type_default() do mod.widgets.LanguageListBase{
        // Fixed viewport so nested touch scroll chains to the page at edges.
        width: Fill height: Fixed(220.)
        list := PortalList{
            width: Fill height: Fill
            spacing: 6
            bounce_at_start: false
            bounce_at_end: false
            scroll_bar: ScrollBar{bar_size: 0.}
            Item := View{
                width: Fill height: Fit
                padding: Inset{bottom: 10.}
                texture_caching: false
                pick := LanguageRow{text: "language"}
            }
        }
    }

    // Server pick rows use LocaleRow (~60px) + item pad — PortalList height
    // must be ≥ one full Open row or the pick button is clipped top/bottom.
    // Devices paired to this Server: name, when last seen, Forget.
    mod.widgets.PairedListBase = #(PairedList::register_widget(vm))
    mod.widgets.PairedList = set_type_default() do mod.widgets.PairedListBase{
        width: Fill height: Fit
        margin: Inset{top: 4. bottom: 4.}
        list := PortalList{
            width: Fill height: Fill
            spacing: 10
            scroll_bar: ScrollBar{}
            Item := View{
                width: Fill height: Fit
                padding: Inset{top: 2. bottom: 8.}
                texture_caching: false
                LocaleRow{}
            }
        }
    }

    mod.widgets.ServerListBase = #(ServerList::register_widget(vm))
    mod.widgets.ServerList = set_type_default() do mod.widgets.ServerListBase{
        width: Fill height: Fit
        margin: Inset{top: 4. bottom: 4.}
        list := PortalList{
            width: Fill height: Fill
            spacing: 10
            scroll_bar: ScrollBar{}
            Item := View{
                width: Fill height: Fit
                padding: Inset{top: 2. bottom: 8.}
                texture_caching: false
                LocaleRow{}
            }
        }
    }

    mod.widgets.CurrencyListBase = #(CurrencyList::register_widget(vm))
    mod.widgets.CurrencyList = set_type_default() do mod.widgets.CurrencyListBase{
        width: Fill height: Fit
        list := PortalList{
            width: Fill height: Fill
            spacing: 6
            scroll_bar: ScrollBar{}
            Item := View{
                width: Fill height: Fit
                padding: Inset{bottom: 10.}
                texture_caching: false
                LocaleRow{}
            }
        }
    }

    // A report or list on screen (day close, books, ops, units …):
    // headings, "label  value" rows with the value at the right, and plain
    // lines — in a white card like the Day book's, never one run-on
    // paragraph. Fed with the same text that prints, or with rows.
    mod.widgets.ReportTextBase = #(ReportText::register_widget(vm))
    mod.widgets.ReportText = set_type_default() do mod.widgets.ReportTextBase{
        width: Fill height: Fit
        card := RoundedView{
            width: Fill height: Fit
            padding: Inset{left: 14. right: 14. top: 4. bottom: 10.}
            draw_bg.color: #xffffff
            draw_bg.border_radius: 14.
            list := PortalList{
                width: Fill height: Fill
                spacing: 0
                drag_scrolling: false
                bounce_at_start: false
                bounce_at_end: false
                capture_overload: false
                scroll_bar: ScrollBar{bar_size: 0.}
                Head := View{
                    width: Fill height: Fit
                    padding: Inset{top: 12. bottom: 4.}
                    head := WrapLabel{
                        body: ""
                        margin: Inset{right: 8.}
                        font_color: #x072058
                        font_size: 13.
                    }
                }
                Kv := View{
                    width: Fill height: Fit
                    padding: Inset{top: 5. bottom: 5.}
                    flow: Right spacing: 8
                    align: Align{y: 0.5}
                    k := WrapLabel{
                        width: Fill
                        body: ""
                        font_color: #x3A4554
                        font_size: 13.
                    }
                    v := WrapLabel{
                        width: Fit
                        body: ""
                        font_color: #x0B1220
                        font_size: 14.
                    }
                    // Per-figure eye for money values (same rules as Books).
                    eye := EyeIconBtn{visible: false}
                }
                // Right-to-left languages: the label at the right, its
                // value at the left. Text set right-aligned overruns its box
                // by a few px, so the labels keep a margin at the right.
                KvRtl := View{
                    width: Fill height: Fit
                    padding: Inset{top: 5. bottom: 5.}
                    flow: Right spacing: 8
                    align: Align{y: 0.5}
                    eye := EyeIconBtn{visible: false}
                    v := WrapLabel{
                        width: Fit
                        body: ""
                        font_color: #x0B1220
                        font_size: 14.
                    }
                    k := WrapLabel{
                        width: Fill
                        body: ""
                        margin: Inset{right: 8.}
                        font_color: #x3A4554
                        font_size: 13.
                    }
                }
                Line := View{
                    width: Fill height: Fit
                    padding: Inset{top: 4. bottom: 4.}
                    line := WrapLabel{
                        body: ""
                        margin: Inset{right: 8.}
                        font_color: #x0B1220
                        font_size: 13.
                    }
                }
                Sub := View{
                    width: Fill height: Fit
                    padding: Inset{left: 12. top: 3. bottom: 3.}
                    line := WrapLabel{
                        body: ""
                        margin: Inset{right: 8.}
                        font_color: #x3A4554
                        font_size: 12.
                    }
                }
                Gap := View{
                    width: Fill height: 10
                }
            }
        }
    }

    startup() do #(App::script_component(vm)){
        ui: Root{
            main_window := Window{
                window.title: "Kumkaj"
                // First-run desktop default (not a phone frame). Later launches
                // restore last size/position from window_geom.json in App code.
                // Layout content still never goes below 320 css px (app_shell).
                window.inner_size: vec2(1024, 768)
                pass.clear_color: #xD6E0EC
                caption_bar +: {
                    draw_bg.color: #x072058
                    // Left cluster in the title bar: size · Kumkaj (not over content).
                    caption_label: View{
                        width: Fill height: Fill
                        flow: Right
                        align: Align{x: 0.0 y: 0.5}
                        padding: Inset{left: 10. right: 10.}
                        spacing: 8
                        dev_size_badge := View{
                            width: Fit height: Fit
                            align: Align{y: 0.5}
                            dev_size_label := Label{
                                width: Fit height: Fit
                                text: "—"
                                draw_text.color: #xfe8357
                                draw_text.text_style.font_size: 11.
                            }
                        }
                        label := Label{
                            text: "Kumkaj"
                            draw_text.color: #xffffff
                        }
                    }
                }
                body +: {
                    width: Fill height: Fill
                    // Overlay — app_callout must NOT be a Down sibling of
                    // app_hscroll or it steals half the window height (every
                    // screen looks split: content above, empty below).
                    flow: Overlay
                    // Min layout width 320 css px. Narrower viewports scroll horizontally
                    // instead of crushing faces below that floor.
                    app_hscroll := ScrollXView{
                        width: Fill height: Fill
                        app_shell := View{
                            width: Fill{min: 320.} height: Fill
                            // Wide (≥720): side rail. Narrow: bottom tabs. See apply_shell_layout.
                            flow: Right spacing: 0

                    // Tablet / desktop nav. More hub expanded into its destinations.
                    nav_rail := SolidView{
                        visible: false
                        width: 220 height: Fill
                        draw_bg.color: #x02030B
                        rail_scroll := ScrollYView{
                            width: Fill height: Fill
                            View{
                                width: Fill height: Fit
                                padding: Inset{left: 12. right: 12. top: 12. bottom: 12.}
                                flow: Down spacing: 6
                                View{
                                    width: Fill height: Fit
                                    flow: Right
                                    align: Align{x: 1.0 y: 0.5}
                                    rail_collapse_btn := RailToggleBtn{text: "‹"}
                                }
                                rail_books_slot := View{
                                    width: Fill height: Fit
                                    flow: Overlay
                                    rail_books := TabBtn{text: "Books"}
                                    rail_books_on := TabBtnActive{text: "Books" visible: false}
                                    View{
                                        width: Fit height: 40
                                        align: Align{x: 0.0 y: 0.5}
                                        padding: Inset{left: 10. right: 0. top: 0. bottom: 0.}
                                        rail_books_icon := Image{
                                            width: 22 height: 22
                                            fit: ImageFit.Stretch
                                            src: crate_resource("self:resources/nav/books.webp")
                                        }
                                    }
                                }
                                rail_sell_slot := View{
                                    width: Fill height: Fit
                                    flow: Overlay
                                    rail_sell := TabBtn{text: "Sell"}
                                    rail_sell_on := TabBtnActive{text: "Sell" visible: false}
                                    View{
                                        width: Fit height: 40
                                        align: Align{x: 0.0 y: 0.5}
                                        padding: Inset{left: 10. right: 0. top: 0. bottom: 0.}
                                        rail_sell_icon := Image{
                                            width: 22 height: 22
                                            fit: ImageFit.Stretch
                                            src: crate_resource("self:resources/nav/sell.webp")
                                        }
                                    }
                                }
                                rail_buy_slot := View{
                                    width: Fill height: Fit
                                    flow: Overlay
                                    rail_buy := TabBtn{text: "Buy"}
                                    rail_buy_on := TabBtnActive{text: "Buy" visible: false}
                                    View{
                                        width: Fit height: 40
                                        align: Align{x: 0.0 y: 0.5}
                                        padding: Inset{left: 10. right: 0. top: 0. bottom: 0.}
                                        rail_buy_icon := Image{
                                            width: 22 height: 22
                                            fit: ImageFit.Stretch
                                            src: crate_resource("self:resources/nav/buy.webp")
                                        }
                                    }
                                }
                                rail_rates_slot := View{
                                    width: Fill height: Fit
                                    flow: Overlay
                                    rail_rates := TabBtn{text: "Rates"}
                                    rail_rates_on := TabBtnActive{text: "Rates" visible: false}
                                    View{
                                        width: Fit height: 40
                                        align: Align{x: 0.0 y: 0.5}
                                        padding: Inset{left: 10. right: 0. top: 0. bottom: 0.}
                                        rail_rates_icon := Image{
                                            width: 22 height: 22
                                            fit: ImageFit.Stretch
                                            src: crate_resource("self:resources/nav/rates.webp")
                                        }
                                    }
                                }
                                rail_ops_slot := View{
                                    width: Fill height: Fit
                                    flow: Overlay
                                    rail_ops := TabBtn{text: "Ops"}
                                    rail_ops_on := TabBtnActive{text: "Ops" visible: false}
                                    View{
                                        width: Fit height: 40
                                        align: Align{x: 0.0 y: 0.5}
                                        padding: Inset{left: 10. right: 0. top: 0. bottom: 0.}
                                        rail_ops_icon := Image{
                                            width: 22 height: 22
                                            fit: ImageFit.Stretch
                                            src: crate_resource("self:resources/nav/ops.webp")
                                        }
                                    }
                                }
                                rail_cod_slot := View{
                                    width: Fill height: Fit
                                    flow: Overlay
                                    rail_cod := TabBtn{text: "COD"}
                                    rail_cod_on := TabBtnActive{text: "COD" visible: false}
                                    View{
                                        width: Fit height: 40
                                        align: Align{x: 0.0 y: 0.5}
                                        padding: Inset{left: 10. right: 0. top: 0. bottom: 0.}
                                        rail_cod_icon := Image{
                                            width: 22 height: 22
                                            fit: ImageFit.Stretch
                                            src: crate_resource("self:resources/nav/cod.webp")
                                        }
                                    }
                                }
                                rail_riders_slot := View{
                                    width: Fill height: Fit
                                    flow: Overlay
                                    rail_riders := TabBtn{text: "Riders"}
                                    rail_riders_on := TabBtnActive{text: "Riders" visible: false}
                                    View{
                                        width: Fit height: 40
                                        align: Align{x: 0.0 y: 0.5}
                                        padding: Inset{left: 10. right: 0. top: 0. bottom: 0.}
                                        rail_riders_icon := Image{
                                            width: 22 height: 22
                                            fit: ImageFit.Stretch
                                            src: crate_resource("self:resources/nav/riders.webp")
                                        }
                                    }
                                }
                                rail_sales_slot := View{
                                    width: Fill height: Fit
                                    flow: Overlay
                                    rail_sales := TabBtn{text: "Sales"}
                                    rail_sales_on := TabBtnActive{text: "Sales" visible: false}
                                    View{
                                        width: Fit height: 40
                                        align: Align{x: 0.0 y: 0.5}
                                        padding: Inset{left: 10. right: 0. top: 0. bottom: 0.}
                                        rail_sales_icon := Image{
                                            width: 22 height: 22
                                            fit: ImageFit.Stretch
                                            src: crate_resource("self:resources/nav/sales.webp")
                                        }
                                    }
                                }
                                rail_day_close_slot := View{
                                    width: Fill height: Fit
                                    flow: Overlay
                                    rail_day_close := TabBtn{text: "Day close"}
                                    rail_day_close_on := TabBtnActive{text: "Day close" visible: false}
                                    View{
                                        width: Fit height: 40
                                        align: Align{x: 0.0 y: 0.5}
                                        padding: Inset{left: 10. right: 0. top: 0. bottom: 0.}
                                        rail_day_close_icon := Image{
                                            width: 22 height: 22
                                            fit: ImageFit.Stretch
                                            src: crate_resource("self:resources/nav/day_close.webp")
                                        }
                                    }
                                }
                                rail_reports_slot := View{
                                    width: Fill height: Fit
                                    flow: Overlay
                                    rail_reports := TabBtn{text: "Reports"}
                                    rail_reports_on := TabBtnActive{text: "Reports" visible: false}
                                    View{
                                        width: Fit height: 40
                                        align: Align{x: 0.0 y: 0.5}
                                        padding: Inset{left: 10. right: 0. top: 0. bottom: 0.}
                                        rail_reports_icon := Image{
                                            width: 22 height: 22
                                            fit: ImageFit.Stretch
                                            src: crate_resource("self:resources/nav/reports.webp")
                                        }
                                    }
                                }
                                rail_inventory_slot := View{
                                    width: Fill height: Fit
                                    flow: Overlay
                                    rail_inventory := TabBtn{text: "Stock"}
                                    rail_inventory_on := TabBtnActive{text: "Stock" visible: false}
                                    View{
                                        width: Fit height: 40
                                        align: Align{x: 0.0 y: 0.5}
                                        padding: Inset{left: 10. right: 0. top: 0. bottom: 0.}
                                        rail_inventory_icon := Image{
                                            width: 22 height: 22
                                            fit: ImageFit.Stretch
                                            src: crate_resource("self:resources/nav/inventory.webp")
                                        }
                                    }
                                }
                                rail_branches_slot := View{
                                    width: Fill height: Fit
                                    flow: Overlay
                                    rail_branches := TabBtn{text: "Branches"}
                                    rail_branches_on := TabBtnActive{text: "Branches" visible: false}
                                    View{
                                        width: Fit height: 40
                                        align: Align{x: 0.0 y: 0.5}
                                        padding: Inset{left: 10. right: 0. top: 0. bottom: 0.}
                                        rail_branches_icon := Image{
                                            width: 22 height: 22
                                            fit: ImageFit.Stretch
                                            src: crate_resource("self:resources/nav/branches.webp")
                                        }
                                    }
                                }
                                rail_credit_slot := View{
                                    width: Fill height: Fit
                                    flow: Overlay
                                    rail_credit := TabBtn{text: "Parties"}
                                    rail_credit_on := TabBtnActive{text: "Parties" visible: false}
                                    View{
                                        width: Fit height: 40
                                        align: Align{x: 0.0 y: 0.5}
                                        padding: Inset{left: 10. right: 0. top: 0. bottom: 0.}
                                        rail_credit_icon := Image{
                                            width: 22 height: 22
                                            fit: ImageFit.Stretch
                                            src: crate_resource("self:resources/nav/credit.webp")
                                        }
                                    }
                                }
                                rail_team_slot := View{
                                    width: Fill height: Fit
                                    flow: Overlay
                                    rail_team := TabBtn{text: "Team"}
                                    rail_team_on := TabBtnActive{text: "Team" visible: false}
                                    View{
                                        width: Fit height: 40
                                        align: Align{x: 0.0 y: 0.5}
                                        padding: Inset{left: 10. right: 0. top: 0. bottom: 0.}
                                        rail_team_icon := Image{
                                            width: 22 height: 22
                                            fit: ImageFit.Stretch
                                            src: crate_resource("self:resources/nav/team.webp")
                                        }
                                    }
                                }
                                rail_settings_slot := View{
                                    width: Fill height: Fit
                                    flow: Overlay
                                    rail_settings := TabBtn{text: "Settings"}
                                    rail_settings_on := TabBtnActive{text: "Settings" visible: false}
                                    View{
                                        width: Fit height: 40
                                        align: Align{x: 0.0 y: 0.5}
                                        padding: Inset{left: 10. right: 0. top: 0. bottom: 0.}
                                        rail_settings_icon := Image{
                                            width: 22 height: 22
                                            fit: ImageFit.Stretch
                                            src: crate_resource("self:resources/nav/settings.webp")
                                        }
                                    }
                                }
                                rail_lock_slot := View{
                                    width: Fill height: Fit
                                    flow: Overlay
                                    rail_lock := TabBtn{text: "Lock"}
                                    rail_lock_on := TabBtnActive{text: "Lock" visible: false}
                                    View{
                                        width: Fit height: 40
                                        align: Align{x: 0.0 y: 0.5}
                                        padding: Inset{left: 10. right: 0. top: 0. bottom: 0.}
                                        rail_lock_icon := Image{
                                            width: 22 height: 22
                                            fit: ImageFit.Stretch
                                            src: crate_resource("self:resources/nav/lock.webp")
                                        }
                                    }
                                }
                            }
                        }
                    }
                    // Drag handle between rail and main (wider than the hairline for hit).
                    rail_divider := SolidView{
                        visible: false
                        width: 6 height: Fill
                        cursor: MouseCursor.ColResize
                        draw_bg.color: #x072058
                    }
                    main_column := View{
                        width: Fill height: Fill
                        flow: Down spacing: 0

                    header := SolidView{
                        width: Fill height: Fit
                        padding: Inset{left: 16. right: 16. top: 12. bottom: 12.}
                        flow: Down spacing: 8
                        draw_bg.color: #x02030B

                        View{
                            width: Fill height: Fit
                            flow: Right spacing: 10
                            align: Align{y: 0.5}
                            rail_expand_btn := RailToggleBtn{text: "›" visible: false}
                            // Elegant back — left of the shop mark; body "Back" buttons stay hidden.
                            header_back := Button{
                                width: 36 height: 36
                                visible: false
                                align: Center
                                padding: Inset{left: 0. right: 0. top: 0. bottom: 0.}
                                margin: Inset{left: 0. right: 2. top: 0. bottom: 0.}
                                text: "\u{f053}"
                                label_walk: Walk{width: Fit height: Fit}
                                icon_walk: Walk{width: 0. height: 0.}
                                draw_bg +: {
                                    color: #x00000000
                                    color_hover: #x072058
                                    color_down: #x02030B
                                    color_focus: #x00000000
                                    color_2: #x00000000
                                    color_2_hover: #x072058
                                    color_2_down: #x02030B
                                    color_2_focus: #x00000000
                                    border_color: #x00000000
                                    border_radius: 18.
                                    border_size: 0.
                                }
                                draw_text +: {
                                    color: #xffffff
                                    color_hover: #xffffff
                                    color_down: #xffffff
                                    color_focus: #xffffff
                                    text_style +: {font_size: 18.}
                                }
                            }
                            brand_mark := Image{
                                // No DSL src — async bundled mark used to overwrite the shop logo after sync.
                                // sync_venue_brand loads the shop mark (or default) inline.
                                width: 36 height: 36
                                fit: ImageFit.Stretch
                            }
                            View{
                                width: Fill height: Fit
                                flow: Down spacing: 2
                                // Label (not Markdown WrapLabel): white ink must
                                // stay visible on the dark header — Markdown
                                // sometimes reverts to the dark WrapLabel default
                                // and the name/org vanish against the header.
                                // Selectable WrapLabel — font_color stays white on navy.
                                brand_title := WrapLabel{
                                    body: "Moklanjo"
                                    font_color: #xffffff
                                    font_size: 18.
                                }
                                session_label := WrapLabel{
                                    body: "Sell"
                                    font_color: #x8392b7
                                    font_size: 11.
                                }
                            }
                            // Screen eye/lock — was the body-top control; lives
                            // here in place of the old "Shop" word. Terracotta
                            // ink so the glyph reads on the navy header.
                            amounts_eye := EyeIconBtn{
                                margin: Inset{left: 4. right: 0. top: 0. bottom: 0.}
                                draw_bg +: {
                                    color_hover: #x00000040
                                    color_down: #x00000055
                                    color_2_hover: #x00000040
                                    color_2_down: #x00000055
                                }
                                draw_text +: {
                                    color: #xfe8357
                                    color_hover: #xfe8357
                                    color_down: #xfe8357
                                    color_focus: #xfe8357
                                }
                            }
                        }
                    }

                    today_bar := SolidView{
                        width: Fill height: Fit
                        flow: Right
                        spacing: 8
                        align: Align{y: 0.5}
                        padding: Inset{left: 16. right: 12. top: 8. bottom: 8.}
                        draw_bg.color: #xE0E8F0
                        today_label := WrapLabel{
                            width: Fill
                            body: "Today: —"
                            font_color: #x0B1220
                            font_size: 12.
                        }
                    }

                    body_stack := View{
                        width: Fill height: Fill
                        flow: Overlay
                        padding: 0.

                        device_setup_screen := ScrollYView{
                            width: Fill height: Fill
                            visible: true

                            View{
                                width: Fill height: Fit
                                flow: Down spacing: 16
                                padding: Inset{left: 22. right: 22. top: 16. bottom: 120.}

                                RoundedView{
                                    width: Fill height: Fit
                                    padding: Inset{left: 12. right: 12. top: 8. bottom: 8.}
                                    flow: Right
                                    align: Align{y: 0.5}
                                    draw_bg.color: #x072058
                                    draw_bg.border_radius: 10.
                                    View{
                                        width: Fill height: Fit
                                        device_setup_banner := WrapLabel{
                                            body: "THIS DEVICE"
                                            font_color: #xffffff
                                            font_size: 10.
                                        }
                                    }
                                    h_local_badge := Label{
                                        text: "LOCAL"
                                        draw_text.color: #xF7B043
                                        draw_text.text_style.font_size: 10.
                                    }
                                }

                                RoundedView{
                                    width: Fill height: Fit
                                    padding: Inset{left: 16. right: 16. top: 16. bottom: 16.}
                                    flow: Down spacing: 14
                                    draw_bg.color: #xD6E0EC
                                    draw_bg.border_radius: 16.

                                    RoundedView{
                                        width: Fill height: Fit
                                        padding: Inset{left: 14. right: 14. top: 14. bottom: 14.}
                                        flow: Down spacing: 8
                                        draw_bg.color: #xffffff
                                        draw_bg.border_radius: 12.
                                        draw_bg.border_size: 1.
                                        draw_bg.border_color: #xE0E8F0
                                        View{
                                            width: Fill height: Fit
                                            flow: Right spacing: 12
                                            align: Align{y: 0.5}
                                            setup_brand_mark := Image{
                                                // No DSL src — async bundled mark used to overwrite the shop logo after sync.
                                // sync_venue_brand loads the shop mark (or default) inline.
                                                width: 48 height: 48
                                                fit: ImageFit.Stretch
                                                            }
                                            View{
                                                width: Fill height: Fit
                                                flow: Down spacing: 4
                                                setup_brand_title := WrapLabel{
                                                    body: "Your shop"
                                                    font_color: #x02030B
                                                    font_size: 18.
                                                    selectable: false
                                                }
                                                device_use_question := WrapLabel{
                                                    body: ""
                                                    font_color: #x3A4554
                                                    font_size: 12.
                                                    visible: false
                                                    selectable: false
                                                }
                                            }
                                        }
                                    }

                                    RoundedView{
                                        width: Fill height: Fit
                                        padding: Inset{left: 10. right: 10. top: 8. bottom: 8.}
                                        flow: Down spacing: 8
                                        draw_bg.color: #xffffff
                                        draw_bg.border_radius: 12.
                                        View{
                                            width: Fill height: Fit
                                            flow: Right spacing: 6
                                            align: Align{y: 0.5}
                                            // Fit (not Fill) so ℹ sits right after the title text,
                                            // not at the far end of the row.
                                            setup_lang_hdr := AccordionHdrBtn{width: Fit text: "LANGUAGE"}
                                            setup_lang_help := InfoHelp{
                                                width: Fit
                                                tip: "Follow this device’s language, or pick one yourself."
                                            }
                                        }
                                        setup_lang_body := View{
                                            width: Fill height: Fit
                                            flow: Down spacing: 8
                                            visible: false
                                            ActionStack{
                                                View{
                                                    width: Fill height: Fit
                                                    flow: Overlay
                                                    lang_setup_platform := StackSecondaryBtn{text: "Device language"}
                                                    lang_setup_platform_on := StackPrimaryBtn{text: "Device language" visible: false}
                                                }
                                                View{
                                                    width: Fill height: Fit
                                                    flow: Overlay
                                                    lang_setup_user := StackSecondaryBtn{text: "Choose language"}
                                                    lang_setup_user_on := StackPrimaryBtn{text: "Choose language" visible: false}
                                                }
                                            }
                                            setup_lang_selected := WrapLabel{
                                                body: ""
                                                font_color: #x0B1220
                                                font_size: 13.
                                                selectable: false
                                            }
                                            setup_lang_list := mod.widgets.LanguageList{}
                                        }
                                    }

                                    RoundedView{
                                        width: Fill height: Fit
                                        padding: Inset{left: 10. right: 10. top: 8. bottom: 8.}
                                        flow: Down spacing: 8
                                        draw_bg.color: #xffffff
                                        draw_bg.border_radius: 12.
                                        View{
                                            width: Fill height: Fit
                                            flow: Right spacing: 6
                                            align: Align{y: 0.5}
                                            setup_role_hdr := AccordionHdrBtn{width: Fit text: "THIS DEVICE"}
                                            setup_device_help := InfoHelp{
                                                width: Fit
                                                tip: "Server keeps the books here. Client pairs to a Server."
                                            }
                                        }
                                        setup_role_body := View{
                                            width: Fill height: Fit
                                            flow: Down spacing: 8
                                            visible: true
                                            ActionStack{
                                                device_role_server := StackSecondaryBtn{text: "Server"}
                                                device_role_server_on := StackPrimaryBtn{text: "Server" visible: false}
                                                device_role_client := StackSecondaryBtn{text: "Client"}
                                                device_role_client_on := StackPrimaryBtn{text: "Client" visible: false}
                                            }
                                            server_pick_section := View{
                                                width: Fill height: Fit
                                                flow: Down spacing: 12
                                                visible: false
                                                server_pick_help := InfoHelp{
                                                    width: Fit
                                                    tip: "Open an existing server, or create one for another business."
                                                }
                                                server_name_input := FieldInput{empty_text: "New server name (optional)"}
                                                server_create_new := NavBtn{text: "Create new server"}
                                                setup_server_list := mod.widgets.ServerList{}
                                                server_pick_cancel := BackBtn{text: "Back"}
                                            }
                                            device_setup_status := WrapLabel{
                                                body: " "
                                                font_color: #xC44E10
                                                font_size: 12.
                                                selectable: false
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        lock_screen := ScrollYView{
                            width: Fill height: Fill
                            visible: false

                            View{
                                width: Fill height: Fit
                                flow: Down spacing: 12
                                padding: Inset{left: 22. right: 22. top: 16. bottom: 120.}

                                // Offline strip (Stitch screen 01)
                                RoundedView{
                                    width: Fill height: Fit
                                    padding: Inset{left: 12. right: 12. top: 8. bottom: 8.}
                                    flow: Right
                                    align: Align{y: 0.5}
                                    draw_bg.color: #x072058
                                    draw_bg.border_radius: 10.
                                    View{
                                        width: Fill height: Fit
                                        h_offline_badge := WrapLabel{
                                            body: "OFFLINE"
                                            font_color: #xffffff
                                            font_size: 10.
                                        }
                                    }
                                    h_local_badge_2 := Label{
                                        text: "LOCAL"
                                        draw_text.color: #xF7B043
                                        draw_text.text_style.font_size: 10.
                                    }
                                }

                                // Brand + lock card
                                RoundedView{
                                    width: Fill height: Fit
                                    padding: Inset{left: 16. right: 16. top: 16. bottom: 16.}
                                    flow: Down spacing: 14
                                    draw_bg.color: #xD6E0EC
                                    draw_bg.border_radius: 16.

                                    RoundedView{
                                        width: Fill height: Fit
                                        padding: Inset{left: 14. right: 14. top: 14. bottom: 14.}
                                        flow: Down spacing: 8
                                        draw_bg.color: #xffffff
                                        draw_bg.border_radius: 12.
                                        draw_bg.border_size: 1.
                                        draw_bg.border_color: #xE0E8F0
                                        View{
                                            width: Fill height: Fit
                                            flow: Right spacing: 12
                                            align: Align{y: 0.5}
                                            lock_brand_mark := Image{
                                                // No DSL src — async bundled mark used to overwrite the shop logo after sync.
                                // sync_venue_brand loads the shop mark (or default) inline.
                                                width: 48 height: 48
                                                fit: ImageFit.Stretch
                                                            }
                                            View{
                                                width: Fill height: Fit
                                                flow: Down spacing: 4
                                                lock_brand_title := WrapLabel{
                                                    body: "Your shop"
                                                    font_color: #x02030B
                                                    font_size: 18.
                                                }
                                                h_counter_locked := WrapLabel{
                                                    body: "Counter 01 · Locked"
                                                    font_color: #x3A4554
                                                    font_size: 12.
                                                }
                                            }
                                            h_shop := Label{
                                                text: "Shop"
                                                draw_text.color: #xC44E10
                                                draw_text.text_style.font_size: 10.
                                            }
                                        }
                                    }

                                    // Tabs that scroll sideways: ‹ › at the ends, each only while there
                                    // is more that way (sync_tab_chevrons).
                                    lock_role_tabs := View{
                                        width: Fill height: Fit
                                        flow: Overlay
                                        tabs := ChipStrip{
                                            ChipStripRow{
                                                View{width: Fit height: 40 flow: Overlay
                                                    role_owner := ChipBtn{
                                                        text: "Owner"
                                                        padding: Inset{left: 18. right: 18.}
                                                    }
                                                    role_owner_on := ChipBtnActive{
                                                        text: "Owner"
                                                        visible: false
                                                        padding: Inset{left: 18. right: 18.}
                                                    }
                                                }
                                                View{width: Fit height: 40 flow: Overlay
                                                    role_staff := ChipBtn{
                                                        text: "Staff"
                                                        visible: false
                                                        padding: Inset{left: 18. right: 18.}
                                                    }
                                                    role_staff_on := ChipBtnActive{
                                                        text: "Staff"
                                                        visible: false
                                                        padding: Inset{left: 18. right: 18.}
                                                    }
                                                }
                                            }
                                        }
                                        View{
                                            width: Fill height: 52
                                            align: Align{x: 0.0 y: 0.5}
                                            chev_l := ChevronBtn{text: "‹" visible: false}
                                        }
                                        View{
                                            width: Fill height: 52
                                            align: Align{x: 1.0 y: 0.5}
                                            chev_r := ChevronBtn{text: "›" visible: false}
                                        }
                                    }

                                    lock_staff_row := View{
                                        width: Fill height: Fit
                                        flow: Down spacing: 8
                                        visible: false
                                        h_who_unlocking := WrapLabel{
                                            body: "Who is unlocking?"
                                            font_color: #x0B1220
                                            font_size: 11.
                                        }
                                        lock_staff_label := WrapLabel{
                                            width: Fit
                                            body: "No staff yet — unlock as Owner, then Team"
                                            font_color: #x0B1220
                                            font_size: 14.
                                        }
                                        ActionStack{
                                            lock_staff_prev := NavBtn{text: "Previous staff"}
                                            lock_staff_next := NavBtn{text: "Next staff"}
                                        }
                                    }

                                    RoundedView{
                                        width: Fill height: Fit
                                        padding: Inset{left: 12. right: 12. top: 16. bottom: 16.}
                                        flow: Down spacing: 10
                                        draw_bg.color: #xffffff
                                        draw_bg.border_radius: 12.
                                        draw_bg.border_size: 1.
                                        draw_bg.border_color: #xE0E8F0
                                        pin_help := InfoHelp{
                                            width: Fit
                                            tip: "4–12 digits. System keyboard only — Enter unlocks."
                                        }
                                        View{
                                            width: Fill height: Fit
                                            flow: Right spacing: 8
                                            align: Align{y: 0.5}
                                            lock_pin_input := FieldInput{
                                                empty_text: "PIN"
                                                is_password: true
                                                is_numeric_only: true
                                            }
                                            pin_reveal := EyeIconBtn{}
                                        }
                                        pin_ok := RowPrimaryBtn{text: "Unlock"}
                                        pin_status := WrapLabel{
                                            width: Fit
                                            body: " "
                                            font_color: #xC44E10
                                            font_size: 12.
                                        }
                                    }

                                    lock_login_help := InfoHelp{
                                        width: Fit
                                        tip: "Optional: unlock with login id and password instead of PIN."
                                    }
                                    lock_login_id := FieldInput{empty_text: "Login id"}
                                    lock_login_password := FieldInput{
                                        empty_text: "Password"
                                        is_password: true
                                    }
                                    lock_login_ok := RowPrimaryBtn{text: "Unlock with login"}
                                }
                            }
                        }

                        onboard_screen := ScrollYView{
                            width: Fill height: Fill
                            visible: false
                            View{
                                width: Fill height: Fit
                                flow: Down spacing: 12
                                padding: Inset{left: 22. right: 22. top: 16. bottom: 120.}
                                h_setup_app_moklanjo := WrapLabel{
                                    body: "Set up Moklanjo"
                                    font_color: #x0B1220
                                    font_size: 22.
                                }
                                onboard_status := WrapLabel{
                                    body: "Name this counter if you want, then Start fresh or Bring my book."
                                    font_color: #x3A4554
                                    font_size: 13.
                                }
                                onboard_venue_input := FieldInput{empty_text: "Counter / agency name (optional)"}
                                onboard_org_input := FieldInput{empty_text: "Organization (optional)"}
                                onboard_save_venue := RowPrimaryBtn{text: "Save names"}
                                View{
                                    width: Fill height: Fit
                                    flow: Right spacing: 12
                                    align: Align{y: 0.5}
                                    onboard_brand_mark := Image{
                                        // No DSL src — async bundled mark used to overwrite the shop logo after sync.
                                // sync_venue_brand loads the shop mark (or default) inline.
                                        width: 48 height: 48
                                        fit: ImageFit.Stretch
                                            }
                                    View{
                                        width: Fill height: Fit
                                        flow: Down spacing: 4
                                        View{
                                            width: Fill height: Fit
                                            flow: Right spacing: 8
                                            align: Align{y: 0.5}
                                            h_shop_logo_optional := WrapLabel{
                                                    width: Fit
                                                body: "Shop logo (optional)"
                                                font_color: #x0B1220
                                                font_size: 15.
                                            }
                                            onboard_icon_help := InfoHelp{
                                                width: Fit
                                                tip: "Optional shop mark on screens in the app."
                                            }
                                        }
                                        onboard_icon_status := WrapLabel{
                                            body: " "
                                            font_color: #x3A4554
                                            font_size: 12.
                                        }
                                    }
                                }
                                onboard_icon_pick := NavBtn{text: "Choose logo"}
                                h_product_locked_moklanjo := WrapLabel{
                                    body: "Moklanjo"
                                    font_color: #x0B1220
                                    font_size: 15.
                                    selectable: false
                                }
                                View{
                                    width: Fill height: Fit
                                    flow: Right spacing: 8
                                    align: Align{y: 0.5}
                                    h_onboard_layout := WrapLabel{
                                                    width: Fit
                                        body: "What you get"
                                        font_color: #x0B1220
                                        font_size: 15.
                                        selectable: false
                                    }
                                    onboard_tmpl_help := InfoHelp{
                                        width: Fit
                                        tip: " "
                                    }
                                }
                                h_how_start := WrapLabel{
                                    body: "How do you want to start?"
                                    font_color: #x0B1220
                                    font_size: 15.
                                    selectable: false
                                }
                                View{
                                    width: Fill height: 52
                                    flow: Overlay
                                    onboard_fresh := RowSecondaryBtn{text: "Start fresh"}
                                    onboard_fresh_on := RowPrimaryBtn{text: "Start fresh" visible: false}
                                }
                                View{
                                    width: Fill height: 52
                                    flow: Overlay
                                    onboard_book := RowSecondaryBtn{text: "Bring my book"}
                                    onboard_book_on := RowPrimaryBtn{text: "Bring my book" visible: false}
                                }
                                h_fresh_book_moklanjo := WrapLabel{
                                    body: ""
                                    font_color: #x3A4554
                                    font_size: 12.
                                    visible: false
                                }
                                onboard_fresh_panel := View{
                                    width: Fill height: Fit
                                    flow: Down spacing: 8
                                    visible: false
                                    h_fresh_selected := WrapLabel{
                                        body: "Fresh selected — catalogue stays empty. Tap Done when ready."
                                        font_color: #x3A4554
                                        font_size: 12.
                                    }
                                }
                                onboard_book_panel := View{
                                    width: Fill height: Fit
                                    flow: Down spacing: 8
                                    visible: false
                                    h_book_add_product := WrapLabel{body: "Book — add product / rate" font_color: #x0B1220 font_size: 15.}
                                    onboard_item_name := FieldInput{empty_text: "Item name"}
                                    onboard_item_price := FieldInput{empty_text: "Price"}
                                    onboard_add_item := NavBtn{text: "Add item"}
                                    h_book_opening_credit := WrapLabel{body: "Book — opening credit" font_color: #x0B1220 font_size: 15.}
                                    onboard_cust_name := FieldInput{empty_text: "Customer name"}
                                    onboard_cust_bal := FieldInput{empty_text: "Amount owed"}
                                    onboard_add_credit := NavBtn{text: "Add opening balance"}
                                }
                                onboard_done := PinOkKey{text: "Done — open books"}
                                h_later_country_pack := WrapLabel{
                                    body: ""
                                    font_color: #x3A4554
                                    font_size: 12.
                                    visible: false
                                }
                            }
                        }

                        client_pair_screen := ScrollYView{
                            width: Fill height: Fill
                            visible: false
                            View{
                                width: Fill height: Fit
                                flow: Down spacing: 12
                                padding: Inset{left: 22. right: 22. top: 16. bottom: 120.}
                                View{
                                    width: Fill height: Fit
                                    flow: Right spacing: 8
                                    align: Align{y: 0.5}
                                    h_connect_to_server := WrapLabel{
                                        width: Fit
                                        body: "Connect to the Server"
                                        font_color: #x0B1220
                                        font_size: 22.
                                    }
                                    client_pair_help := InfoHelp{
                                        width: Fit
                                        tip: "Find the shop's server on Wi‑Fi, or type its address and pairing code."
                                    }
                                }
                                // Back is header_back (left of brand) — not a body button.
                                client_pair_back := BackBtn{text: "Back" visible: false}
                                client_pair_status := WrapLabel{
                                    body: " "
                                    font_color: #x3A4554
                                    font_size: 13.
                                    selectable: false
                                }
                                // No typing IPs: the office's server answers on this Wi‑Fi.
                                client_find_btn := StackPrimaryBtn{text: "Find the shop's server"}
                                client_found_label := WrapLabel{
                                    body: " "
                                    font_color: #x3A4554
                                    font_size: 13.
                                }
                                client_found_list := mod.widgets.ReportText{}
                                client_host_input := FieldInput{empty_text: "Server IP (e.g. 192.168.1.10)"}
                                client_code_input := FieldInput{empty_text: "Pairing code"}
                                // How the office's server lists this device (Forget picks it by this).
                                // Filled with this device's own name: keep a small label on it.
                                client_name_input := FieldInput{
                                    empty_text: "This device's name"
                                    float_caption: true
                                    draw_caption +: {
                                        color: #x3A4554
                                        text_style: theme.font_regular{font_size: 10.5}
                                    }
                                }
                                client_pair_btn := PinOkKey{text: "Pair now"}
                                client_sync_btn := NavBtn{text: "Refresh from shop"}
                                client_queue_label := mod.widgets.ReportText{}
                                client_clear_refused := NavBtn{text: "Checked — clear this list" visible: false}
                                client_unpair := NavBtn{text: "Forget this shop"}
                                client_skip_lock := NavBtn{text: "Enter PIN" visible: false}
                            }
                        }

                        // Books home — today's position (commons: not Sell-first).
                        books_screen := ScrollYView{
                            width: Fill height: Fill
                            visible: false
                            View{
                                width: Fill height: Fit
                                flow: Down spacing: 10
                                padding: Inset{left: 16. right: 16. top: 8. bottom: 120.}
                                books_help := InfoHelp{
                                    width: Fit
                                    tip: "Today’s position. Booking = parcels in. Pay out = carrier bills & expenses. Parties = consignors & carriers."
                                }
                                books_backup_warning := WrapLabel{body: " " font_color: #xC44E10 font_size: 14. visible: false}
                                // Hub jumps only — every row navigates, so every
                                // row is HubDestBtn + mark (never a filled primary).
                                ActionStack{
                                    books_to_sell_slot := View{
                                        width: Fill height: Fit
                                        flow: Overlay
                                        books_to_sell := HubDestBtn{text: "Book a parcel"}
                                        View{
                                            width: Fill height: Fill
                                            flow: Right
                                            align: Align{x: 0.0 y: 0.5}
                                            padding: Inset{left: 14. right: 0. top: 0. bottom: 0.}
                                            books_to_sell_icon := Image{
                                                width: 22 height: 22
                                                fit: ImageFit.Stretch
                                                src: crate_resource("self:resources/nav/sell.webp")
                                            }
                                        }
                                    }
                                    books_to_buy_slot := View{
                                        width: Fill height: Fit
                                        flow: Overlay
                                        books_to_buy := HubDestBtn{text: "Expense / carrier bill"}
                                        View{
                                            width: Fill height: Fill
                                            flow: Right
                                            align: Align{x: 0.0 y: 0.5}
                                            padding: Inset{left: 14. right: 0. top: 0. bottom: 0.}
                                            books_to_buy_icon := Image{
                                                width: 22 height: 22
                                                fit: ImageFit.Stretch
                                                src: crate_resource("self:resources/nav/buy.webp")
                                            }
                                        }
                                    }
                                    books_day_close_slot := View{
                                        width: Fill height: Fit
                                        flow: Overlay
                                        books_day_close := HubDestBtn{text: "Day close"}
                                        View{
                                            width: Fill height: Fill
                                            flow: Right
                                            align: Align{x: 0.0 y: 0.5}
                                            padding: Inset{left: 14. right: 0. top: 0. bottom: 0.}
                                            books_day_close_icon := Image{
                                                width: 22 height: 22
                                                fit: ImageFit.Stretch
                                                src: crate_resource("self:resources/nav/day_close.webp")
                                            }
                                        }
                                    }
                                    books_reports_slot := View{
                                        width: Fill height: Fit
                                        flow: Overlay
                                        books_reports := HubDestBtn{text: "Bank book & P&L"}
                                        View{
                                            width: Fill height: Fill
                                            flow: Right
                                            align: Align{x: 0.0 y: 0.5}
                                            padding: Inset{left: 14. right: 0. top: 0. bottom: 0.}
                                            books_reports_icon := Image{
                                                width: 22 height: 22
                                                fit: ImageFit.Stretch
                                                src: crate_resource("self:resources/nav/reports.webp")
                                            }
                                        }
                                    }
                                    books_credit_slot := View{
                                        width: Fill height: Fit
                                        flow: Overlay
                                        books_credit := HubDestBtn{text: "Parties (debtors & creditors)"}
                                        View{
                                            width: Fill height: Fill
                                            flow: Right
                                            align: Align{x: 0.0 y: 0.5}
                                            padding: Inset{left: 14. right: 0. top: 0. bottom: 0.}
                                            books_credit_icon := Image{
                                                width: 22 height: 22
                                                fit: ImageFit.Stretch
                                                src: crate_resource("self:resources/nav/credit.webp")
                                            }
                                        }
                                    }
                                }
                                // Accordion — only TODAY open by default.
                                RoundedView{
                                    width: Fill height: Fit
                                    padding: Inset{left: 14. right: 14. top: 10. bottom: 10.}
                                    flow: Down spacing: 7
                                    draw_bg.color: #xffffff
                                    draw_bg.border_radius: 14.
                                    books_sec_today_hdr := AccordionHdrBtn{text: "TODAY"}
                                    books_sec_today_body := View{
                                        width: Fill height: Fit
                                        flow: Down spacing: 7
                                        visible: false
                                        h_caption_today := Label{text: "TODAY" visible: false draw_text.color: #x66788F draw_text.text_style.font_size: 10.}
                                        View{width: Fill height: Fit flow: Right spacing: 8 align: Align{y: 0.5}
                                            h_money_in_sales := Label{width: Fill height: Fit text: "Money in (sales)" draw_text.color: #x3A4554 draw_text.text_style.font_size: 12.}
                                            View{width: Fit height: Fit flow: Right spacing: 4 align: Align{y: 0.5}
                                            books_in_value := Label{width: Fit height: Fit text: "—" draw_text.color: #x0B1220 draw_text.text_style.font_size: 13.}
                                            books_in_eye := EyeIconBtn{}
                                        }
                                        }
                                        View{width: Fill height: Fit flow: Right spacing: 8 align: Align{y: 0.5}
                                            h_purchases_cogs := Label{width: Fill height: Fit text: "Purchases (COGS)" draw_text.color: #x3A4554 draw_text.text_style.font_size: 12.}
                                            View{width: Fit height: Fit flow: Right spacing: 4 align: Align{y: 0.5}
                                            books_purchases_value := Label{width: Fit height: Fit text: "—" draw_text.color: #x0B1220 draw_text.text_style.font_size: 13.}
                                            books_purchases_eye := EyeIconBtn{}
                                        }
                                        }
                                        View{width: Fill height: Fit flow: Right spacing: 8 align: Align{y: 0.5}
                                            h_expenses_word := Label{width: Fill height: Fit text: "Expenses" draw_text.color: #x3A4554 draw_text.text_style.font_size: 12.}
                                            View{width: Fit height: Fit flow: Right spacing: 4 align: Align{y: 0.5}
                                            books_expenses_value := Label{width: Fit height: Fit text: "—" draw_text.color: #x0B1220 draw_text.text_style.font_size: 13.}
                                            books_expenses_eye := EyeIconBtn{}
                                        }
                                        }
                                        View{width: Fill height: Fit flow: Right spacing: 8 align: Align{y: 0.5}
                                            h_gross_word := Label{width: Fill height: Fit text: "Gross" draw_text.color: #x3A4554 draw_text.text_style.font_size: 12.}
                                            View{width: Fit height: Fit flow: Right spacing: 4 align: Align{y: 0.5}
                                            books_gross_value := Label{width: Fit height: Fit text: "—" draw_text.color: #x0B1220 draw_text.text_style.font_size: 13.}
                                            books_gross_eye := EyeIconBtn{}
                                        }
                                        }
                                        View{width: Fill height: Fit flow: Right spacing: 8 align: Align{y: 0.5}
                                            h_net_word := Label{width: Fill height: Fit text: "Net" draw_text.color: #x072058 draw_text.text_style.font_size: 13.}
                                            View{width: Fit height: Fit flow: Right spacing: 4 align: Align{y: 0.5}
                                            books_net_value := Label{width: Fit height: Fit text: "—" draw_text.color: #x072058 draw_text.text_style.font_size: 15.}
                                            books_net_eye := EyeIconBtn{}
                                        }
                                        }
                                    }
                                }
                                RoundedView{
                                    width: Fill height: Fit
                                    padding: Inset{left: 14. right: 14. top: 10. bottom: 10.}
                                    flow: Down spacing: 7
                                    draw_bg.color: #xffffff
                                    draw_bg.border_radius: 14.
                                    books_sec_bookings_hdr := AccordionHdrBtn{text: "BOOKINGS"}
                                    books_sec_bookings_body := View{
                                        width: Fill height: Fit
                                        flow: Down spacing: 7
                                        visible: false
                                        h_caption_bookings := Label{text: "BOOKINGS" visible: false draw_text.color: #x66788F draw_text.text_style.font_size: 10.}
                                        View{width: Fill height: Fit flow: Right spacing: 8 align: Align{y: 0.5}
                                            books_declared_name := Label{width: Fill height: Fit text: "Account booking" draw_text.color: #x3A4554 draw_text.text_style.font_size: 12.}
                                            View{width: Fit height: Fit flow: Right spacing: 4 align: Align{y: 0.5}
                                            books_declared_value := Label{width: Fit height: Fit text: "—" draw_text.color: #x0B1220 draw_text.text_style.font_size: 13.}
                                            books_declared_eye := EyeIconBtn{}
                                        }
                                        }
                                        View{width: Fill height: Fit flow: Right spacing: 8 align: Align{y: 0.5}
                                            books_owner_name := Label{width: Fill height: Fit text: "Cash booking" draw_text.color: #x3A4554 draw_text.text_style.font_size: 12.}
                                            View{width: Fit height: Fit flow: Right spacing: 4 align: Align{y: 0.5}
                                            books_owner_value := Label{width: Fit height: Fit text: "—" draw_text.color: #x0B1220 draw_text.text_style.font_size: 13.}
                                            books_owner_eye := EyeIconBtn{}
                                        }
                                        }
                                    }
                                }
                                RoundedView{
                                    width: Fill height: Fit
                                    padding: Inset{left: 14. right: 14. top: 10. bottom: 10.}
                                    flow: Down spacing: 7
                                    draw_bg.color: #xffffff
                                    draw_bg.border_radius: 14.
                                    books_sec_payments_hdr := AccordionHdrBtn{text: "PAYMENTS TODAY"}
                                    books_sec_payments_body := View{
                                        width: Fill height: Fit
                                        flow: Down spacing: 7
                                        visible: false
                                        h_caption_payments_today := Label{text: "PAYMENTS TODAY" visible: false draw_text.color: #x66788F draw_text.text_style.font_size: 10.}
                                        View{width: Fill height: Fit flow: Right spacing: 8 align: Align{y: 0.5}
                                            h_cash := Label{width: Fill height: Fit text: "Cash" draw_text.color: #x3A4554 draw_text.text_style.font_size: 12.}
                                            View{width: Fit height: Fit flow: Right spacing: 4 align: Align{y: 0.5}
                                            books_cash_value := Label{width: Fit height: Fit text: "—" draw_text.color: #x0B1220 draw_text.text_style.font_size: 13.}
                                            books_cash_eye := EyeIconBtn{}
                                        }
                                        }
                                        View{width: Fill height: Fit flow: Right spacing: 8 align: Align{y: 0.5}
                                            books_digital_name := Label{width: Fill height: Fit text: "UPI" draw_text.color: #x3A4554 draw_text.text_style.font_size: 12.}
                                            View{width: Fit height: Fit flow: Right spacing: 4 align: Align{y: 0.5}
                                            books_digital_value := Label{width: Fit height: Fit text: "—" draw_text.color: #x0B1220 draw_text.text_style.font_size: 13.}
                                            books_digital_eye := EyeIconBtn{}
                                        }
                                        }
                                        View{width: Fill height: Fit flow: Right spacing: 8 align: Align{y: 0.5}
                                            h_card := Label{width: Fill height: Fit text: "Card" draw_text.color: #x3A4554 draw_text.text_style.font_size: 12.}
                                            View{width: Fit height: Fit flow: Right spacing: 4 align: Align{y: 0.5}
                                            books_card_value := Label{width: Fit height: Fit text: "—" draw_text.color: #x0B1220 draw_text.text_style.font_size: 13.}
                                            books_card_eye := EyeIconBtn{}
                                        }
                                        }
                                    }
                                }
                                RoundedView{
                                    width: Fill height: Fit
                                    padding: Inset{left: 14. right: 14. top: 10. bottom: 10.}
                                    flow: Down spacing: 7
                                    draw_bg.color: #xffffff
                                    draw_bg.border_radius: 14.
                                    books_sec_outstanding_hdr := AccordionHdrBtn{text: "OUTSTANDING"}
                                    books_sec_outstanding_body := View{
                                        width: Fill height: Fit
                                        flow: Down spacing: 7
                                        visible: false
                                        h_caption_outstanding := Label{text: "OUTSTANDING" visible: false draw_text.color: #x66788F draw_text.text_style.font_size: 10.}
                                        View{width: Fill height: Fit flow: Right spacing: 8 align: Align{y: 0.5}
                                            h_they_owe_us := Label{width: Fill height: Fit text: "They owe us" draw_text.color: #x3A4554 draw_text.text_style.font_size: 12.}
                                            View{width: Fit height: Fit flow: Right spacing: 4 align: Align{y: 0.5}
                                            books_they_owe_value := Label{width: Fit height: Fit text: "—" draw_text.color: #x137333 draw_text.text_style.font_size: 13.}
                                            books_they_owe_eye := EyeIconBtn{}
                                        }
                                        }
                                        View{width: Fill height: Fit flow: Right spacing: 8 align: Align{y: 0.5}
                                            h_we_owe_them := Label{width: Fill height: Fit text: "We owe them" draw_text.color: #x3A4554 draw_text.text_style.font_size: 12.}
                                            View{width: Fit height: Fit flow: Right spacing: 4 align: Align{y: 0.5}
                                            books_we_owe_value := Label{width: Fit height: Fit text: "—" draw_text.color: #xB3261E draw_text.text_style.font_size: 13.}
                                            books_we_owe_eye := EyeIconBtn{}
                                        }
                                        }
                                    }
                                }
                                RoundedView{
                                    width: Fill height: Fit
                                    padding: Inset{left: 14. right: 14. top: 10. bottom: 10.}
                                    flow: Down spacing: 7
                                    draw_bg.color: #xffffff
                                    draw_bg.border_radius: 14.
                                    books_sec_bank_hdr := AccordionHdrBtn{text: "BANK"}
                                    books_sec_bank_body := View{
                                        width: Fill height: Fit
                                        flow: Down spacing: 7
                                        visible: false
                                        h_caption_bank := Label{text: "BANK" visible: false draw_text.color: #x66788F draw_text.text_style.font_size: 10.}
                                        View{width: Fill height: Fit flow: Right spacing: 8 align: Align{y: 0.5}
                                            h_bank_balance := Label{width: Fill height: Fit text: "Bank balance" draw_text.color: #x3A4554 draw_text.text_style.font_size: 12.}
                                            View{width: Fit height: Fit flow: Right spacing: 4 align: Align{y: 0.5}
                                            books_bank_balance_value := Label{width: Fit height: Fit text: "—" draw_text.color: #x0B1220 draw_text.text_style.font_size: 13.}
                                            books_bank_balance_eye := EyeIconBtn{}
                                        }
                                        }
                                        View{width: Fill height: Fit flow: Right spacing: 8 align: Align{y: 0.5}
                                            h_cash_in_hand := Label{width: Fill height: Fit text: "Cash in hand (till)" draw_text.color: #x3A4554 draw_text.text_style.font_size: 12.}
                                            View{width: Fit height: Fit flow: Right spacing: 4 align: Align{y: 0.5}
                                            books_till_value := Label{width: Fit height: Fit text: "—" draw_text.color: #x0B1220 draw_text.text_style.font_size: 13.}
                                            books_till_eye := EyeIconBtn{}
                                        }
                                        }
                                        View{width: Fill height: Fit flow: Right spacing: 8 align: Align{y: 0.5}
                                            h_bank_net_today := Label{width: Fill height: Fit text: "Bank net today" draw_text.color: #x3A4554 draw_text.text_style.font_size: 12.}
                                            View{width: Fit height: Fit flow: Right spacing: 4 align: Align{y: 0.5}
                                            books_bank_net_value := Label{width: Fit height: Fit text: "—" draw_text.color: #x0B1220 draw_text.text_style.font_size: 13.}
                                            books_bank_net_eye := EyeIconBtn{}
                                        }
                                        }
                                    }
                                }
                            }
                        }

                        // Booking: Parcel (rates/weigh) + Counter (cart/pay) always
                        // Parcel (rates) + Counter (cart/pay) side by side —
                        // Fill{min} panes; horizontal scroll when both don't fit.
                        pos_screen := View{
                            width: Fill height: Fill
                            visible: false
                            flow: Down spacing: 0

                            booking_desk_top := View{
                                width: Fill height: Fit
                                flow: Down spacing: 10
                                padding: Inset{left: 16. right: 16. top: 8. bottom: 8.}
                                // Operational strip (owner / till status)
                                RoundedView{
                                    width: Fill height: Fit
                                    padding: Inset{left: 10. right: 10. top: 8. bottom: 8.}
                                    flow: Right spacing: 8
                                    align: Align{y: 0.5}
                                    draw_bg.color: #xEEF3F8
                                    draw_bg.border_radius: 12.
                                    View{
                                        width: Fill height: Fit
                                        flow: Down spacing: 2
                                        session_chip := WrapLabel{
                                            body: "Counter"
                                            font_color: #x0B1220
                                            font_size: 12.
                                        }
                                        table_chip := WrapLabel{
                                            body: "Ready to sell"
                                            font_color: #x0B1220
                                            font_size: 11.
                                        }
                                    }
                                    h_local_badge_3 := Label{
                                        width: Fit
                                        text: "LOCAL"
                                        draw_text.color: #x9A6206
                                        draw_text.text_style.font_size: 10.
                                    }
                                }
                                booking_help := InfoHelp{
                                    width: Fit
                                    tip: "Consignor + consignee (blank OK) → destination → weigh → pick rate → Account/Cash → pay. AWB prints on the receipt. Next booking clears the desk. Rounded starts equal to Applied — edit only to round the bill."
                                }
                            }

                            // Fit row + Fixed panes under sticky booking_desk_top.
                            // Side by side; ScrollX pans when mins do not fit.
                            booking_desk_hscroll := ScrollXView{
                                width: Fill height: Fill
                                padding: Inset{left: 16. right: 16. top: 8. bottom: 16.}
                                scroll_bars: mod.widgets.ScrollBars {
                                    show_scroll_x: true show_scroll_y: false
                                    scroll_bar_x.drag_scrolling: true
                                    scroll_bar_x.mouse_capture_overload: true
                                    scroll_bar_x.bar_size: 8.
                                }
                                booking_desk_row := View{
                                    width: Fit height: Fill
                                    flow: Right spacing: 16
                                    pos_catalog_scroll := ScrollYView{
                                        width: 600 height: Fill
                                    pos_catalog := View{
                                        width: Fill height: Fit
                                        flow: Down spacing: 10
                                        padding: Inset{bottom: 48.}
                                // Address blocks always on the form (structure
                                // required); values may stay empty for walk-ins.
                                // Consignor = customer / sender — same party.
                                View{
                                    width: Fill height: Fit
                                    flow: Right spacing: 8
                                    align: Align{x: 0.0 y: 0.5}
                                    h_booking_consignor := WrapLabel{
                                        width: Fit
                                        body: "Consignor (customer / sender)"
                                        font_color: #x0B1220
                                        font_size: 14.
                                    }
                                    booking_consignor_help := InfoHelp{
                                        width: Fit
                                        tip: "Pick a party to fill, or leave blank for walk-in. Needed for home pickup and RTO."
                                    }
                                }
                                booking_origin_name_input := FieldInput{empty_text: "Sender name"}
                                booking_origin_suggest := mod.widgets.SuggestList{field: "booking_origin"}
                                booking_origin_phone_input := FieldInput{empty_text: "Sender phone"}
                                booking_origin_address_input := FieldInput{empty_text: "Pickup / origin address"}
                                booking_consignor_hint := WrapLabel{
                                    selectable: false
                                    body: "Pick a party to fill, or leave blank for walk-in. Needed for home pickup and RTO."
                                    font_color: #x3A4554
                                    font_size: 11.
                                    visible: false
                                }
                                View{
                                    width: Fill height: Fit
                                    flow: Right spacing: 8
                                    align: Align{x: 0.0 y: 0.5}
                                    h_booking_consignee := WrapLabel{
                                        width: Fit
                                        body: "Consignee (receiver)"
                                        font_color: #x0B1220
                                        font_size: 14.
                                    }
                                    booking_consignee_help := InfoHelp{
                                        width: Fit
                                        tip: "Receiver of the parcel. Leave blank for walk-in counter drop-offs."
                                    }
                                }
                                booking_consignee_input := FieldInput{empty_text: "Receiver name"}
                                booking_consignee_phone_input := FieldInput{empty_text: "Receiver phone"}
                                booking_consignee_address_input := FieldInput{empty_text: "Delivery address"}
                                booking_destination_input := FieldInput{empty_text: "Destination city / zone"}
                                booking_destination_suggest := mod.widgets.SuggestList{field: "booking_destination"}
                                booking_zone_input := FieldInput{empty_text: "Zone (name from Rates)"}
                                booking_zone_suggest := mod.widgets.SuggestList{field: "booking_zone"}
                                booking_pincode_label := WrapLabel{
                                    width: Fit
                                    body: ""
                                    font_color: #x006e1c
                                    font_size: 12.
                                }
                                // Domestic ÷ shop divisor (usually 5000); International ÷6000
                                // air — same desk-chip pattern as kg/lb. Settings keep the
                                // shop default; this only changes maths for the open booking.
                                booking_scope_help := InfoHelp{
                                    width: Fit
                                    tip: "Volumetric ÷5000 (domestic) — tap International for ÷6000 air"
                                }
                                View{
                                    width: Fill height: Fit
                                    flow: Right spacing: 6
                                    align: Align{x: 0.0 y: 0.5}
                                    View{width: Fill height: Fit flow: Overlay
                                        booking_scope_domestic := SegBtn{text: "Domestic"}
                                        booking_scope_domestic_on := SegBtnOn{text: "Domestic" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        booking_scope_intl := SegBtn{text: "International"}
                                        booking_scope_intl_on := SegBtnOn{text: "International" visible: false}
                                    }
                                }
                                booking_scope_hint := WrapLabel{
                                    selectable: false
                                    body: "Volumetric ÷5000 (domestic) — tap International for ÷6000 air"
                                    font_color: #x3A4554
                                    font_size: 11.
                                    visible: false
                                }
                                booking_actual_kg_input := FieldInput{empty_text: "Actual kg"}
                                // Unit choices live on the desk (tap to switch) —
                                // nobody should have to hunt Settings to weigh
                                // a parcel in pounds. Custom units, divisor
                                // and slab stay in Settings → Booking measures.
                                View{
                                    width: Fill height: Fit
                                    flow: Right spacing: 6
                                    align: Align{y: 0.5}
                                    h_weight_in := Label{text: "Weight in" draw_text.color: #x3A4554 draw_text.text_style.font_size: 11.}
                                    View{width: Fit height: 40 flow: Overlay
                                        wunit_kg := ChipBtn{text: "kg"}
                                        wunit_kg_on := ChipBtnActive{text: "kg" visible: false}
                                    }
                                    View{width: Fit height: 40 flow: Overlay
                                        wunit_lb := ChipBtn{text: "lb"}
                                        wunit_lb_on := ChipBtnActive{text: "lb" visible: false}
                                    }
                                    View{width: Fit height: 40 flow: Overlay
                                        wunit_g := ChipBtn{text: "g"}
                                        wunit_g_on := ChipBtnActive{text: "g" visible: false}
                                    }
                                    View{width: Fit height: 40 flow: Overlay
                                        wunit_oz := ChipBtn{text: "oz"}
                                        wunit_oz_on := ChipBtnActive{text: "oz" visible: false}
                                    }
                                }
                                // L×B×H on their own row — equal Fill, wide enough that
                                // "L cm" never crushes into a vertical letter stack.
                                View{
                                    width: Fill height: Fit
                                    flow: Right
                                    spacing: 8
                                    booking_length_cm_input := FieldInput{
                                        width: Fill{min: 88.}
                                        empty_text: "L cm"
                                    }
                                    booking_breadth_cm_input := FieldInput{
                                        width: Fill{min: 88.}
                                        empty_text: "B cm"
                                    }
                                    booking_height_cm_input := FieldInput{
                                        width: Fill{min: 88.}
                                        empty_text: "H cm"
                                    }
                                }
                                View{
                                    width: Fill height: Fit
                                    flow: Right spacing: 6
                                    align: Align{y: 0.5}
                                    h_size_in := Label{text: "Size in" draw_text.color: #x3A4554 draw_text.text_style.font_size: 11.}
                                    View{width: Fit height: 40 flow: Overlay
                                        dunit_cm := ChipBtn{text: "cm"}
                                        dunit_cm_on := ChipBtnActive{text: "cm" visible: false}
                                    }
                                    View{width: Fit height: 40 flow: Overlay
                                        dunit_in := ChipBtn{text: "in"}
                                        dunit_in_on := ChipBtnActive{text: "in" visible: false}
                                    }
                                    View{width: Fit height: 40 flow: Overlay
                                        dunit_ft := ChipBtn{text: "ft"}
                                        dunit_ft_on := ChipBtnActive{text: "ft" visible: false}
                                    }
                                    View{width: Fit height: 40 flow: Overlay
                                        dunit_m := ChipBtn{text: "m"}
                                        dunit_m_on := ChipBtnActive{text: "m" visible: false}
                                    }
                                }
                                    booking_pieces_input := FieldInput{
                                        empty_text: "Pieces (blank = 1) — expand for cargo / multi-piece"
                                    }
                                    FormFieldRow{
                                        booking_cod_input := FieldInput{empty_text: "COD amount"}
                                        booking_carrier_cost_input := FieldInput{empty_text: "Carrier cost"}
                                    }
                                    booking_compare_label := WrapLabel{
                                        body: ""
                                        font_color: #x0B1220
                                        font_size: 12.
                                    }
                                    booking_carrier_pick_input := FieldInput{empty_text: "Carrier for cost (from compare)"}
                                    booking_carrier_suggest := mod.widgets.SuggestList{field: "booking_carrier"}
                                    booking_apply_carrier_cost := RowSecondaryBtn{text: "Use carrier cost"}
                                    FormFieldRow{
                                        booking_declared_input := FieldInput{empty_text: "Declared value"}
                                        booking_contents_input := FieldInput{empty_text: "Contents"}
                                    }
                                    booking_eway_input := FieldInput{empty_text: "E-way bill no."}
                                    h_booking_insurance := Label{
                                        text: "Insurance"
                                        draw_text.color: #x3A4554
                                        draw_text.text_style.font_size: 11.
                                    }
                                    View{
                                        width: Fill height: Fit
                                        flow: Right spacing: 6
                                        View{width: Fill height: Fit flow: Overlay
                                            booking_insurance_no := SegBtn{text: "No"}
                                            booking_insurance_no_on := SegBtnOn{text: "No" visible: false}
                                        }
                                        View{width: Fill height: Fit flow: Overlay
                                            booking_insurance_yes := SegBtn{text: "Yes"}
                                            booking_insurance_yes_on := SegBtnOn{text: "Yes" visible: false}
                                        }
                                    }
                                    FormFieldRow{
                                        booking_fuel_input := FieldInput{empty_text: "Fuel"}
                                        booking_docket_input := FieldInput{empty_text: "Docket charge"}
                                    }
                                booking_notify_input := FieldInput{empty_text: "Notify phone (WhatsApp/SMS hook)"}
                                booking_bulk_input := FieldInput{
                                    empty_text: "Bulk CSV — destination,consignee,kg,cod_paise,note"
                                }
                                booking_bulk_load := RowSecondaryBtn{text: "Load bulk row → desk"}
                                chargeable_kg_label := WrapLabel{
                                    body: "Chargeable kg: —"
                                    font_color: #x0B1220
                                    font_size: 13.
                                }
                                last_awb_label := WrapLabel{
                                    body: ""
                                    font_color: #x006e1c
                                    font_size: 14.
                                }
                                next_booking_btn := RowPrimaryBtn{text: "Next booking" visible: false}
                                empty_catalog := WrapLabel{
                                    body: "No rates yet. Open Rates & services, add one (name + price, unit kg), then Booking."
                                    font_color: #x0B1220
                                    font_size: 14.
                                }
                                product_list := mod.widgets.ProductList{}
                                    } // pos_catalog
                                    } // pos_catalog_scroll

                                    pos_cart_scroll := ScrollYView{
                                        width: 520 height: Fill
                                    pos_cart_col := View{
                                        width: Fill height: Fit
                                        flow: Down spacing: 10
                                        padding: Inset{bottom: 48.}
                                View{
                                    width: Fill height: Fit
                                    flow: Right spacing: 12
                                    align: Align{y: 0.5}
                                    padding: Inset{top: 4. bottom: 4.}
                                    h_active_ledger := WrapLabel{
                                        width: Fit
                                        body: "Active ledger"
                                        font_color: #x3A4554
                                        font_size: 13.
                                    }
                                    View{width: Fill height: 1}
                                    receipt_label := WrapLabel{
                                        width: Fit
                                        body: "Ready"
                                        font_color: #x3A4554
                                        font_size: 13.
                                    }
                                    clear_cart := Button{
                                        text: "Clear"
                                        width: Fit height: 32
                                        padding: Inset{left: 14. right: 14. top: 4. bottom: 4.}
                                        draw_bg +: {
                                            color: #xE0E8F0
                                            color_hover: #x072058
                                            color_down: #x02030B
                                            color_focus: #xE0E8F0
                                            border_color: #xA8B4C4
                                            border_radius: 8.
                                        }
                                        draw_text +: {
                                            color: #x93000a
                                            color_hover: #xffffff
                                            color_down: #xffffff
                                            color_focus: #x93000a
                                            text_style +: {font_size: 13.}
                                        }
                                    }
                                }

                                RoundedView{
                                    width: Fill height: Fit
                                    padding: Inset{left: 16. right: 16. top: 14. bottom: 14.}
                                    flow: Down spacing: 10
                                    draw_bg.color: #xffffff
                                    draw_bg.border_radius: 16.
                                    draw_bg.border_size: 2.
                                    draw_bg.border_color: #x4A5568
                                    cart_list := mod.widgets.CartList{}
                                    cart_empty_hint := WrapLabel{
                                        body: "Desk empty — set destination & weight, tap ADD on a rate, then pay."
                                        font_color: #x3A4554
                                        font_size: 12.
                                    }

                                    // Triple totals strip (Actual | Applied | Rounded)
                                    RoundedView{
                                        width: Fill height: Fit
                                        padding: Inset{left: 8. right: 8. top: 8. bottom: 8.}
                                        flow: Right spacing: 4
                                        draw_bg.color: #xE0E8F0
                                        draw_bg.border_radius: 12.
                                        View{
                                            width: Fill height: Fit
                                            flow: Down spacing: 2
                                            align: Align{x: 0.5}
                                            h_actual_total_cap := WrapLabel{
                                                body: "ACTUAL TOTAL"
                                                font_color: #x3A4554
                                                font_size: 10.
                                            }
                                            cart_natural_label := WrapLabel{
                                                body: "0"
                                                font_color: #x0B1220
                                                font_size: 14.
                                            }
                                        }
                                        View{
                                            width: Fill height: Fit
                                            flow: Down spacing: 2
                                            align: Align{x: 0.5}
                                            h_applied_total_cap := WrapLabel{
                                                body: "APPLIED TOTAL"
                                                font_color: #x3A4554
                                                font_size: 10.
                                            }
                                            cart_applied_label := WrapLabel{
                                                body: "0"
                                                font_color: #x0B1220
                                                font_size: 14.
                                            }
                                        }
                                        View{
                                            width: Fill height: Fit
                                            flow: Down spacing: 2
                                            align: Align{x: 0.5}
                                            cart_charge_caption := WrapLabel{
                                                body: "ROUNDED"
                                                font_color: #xC44E10
                                                font_size: 10.
                                            }
                                            cart_total_input := CartField{
                                                empty_text: "0"
                                                text: "0"
                                                draw_bg +: {
                                                    border_color: #xC44E10
                                                    border_color_hover: #x822801
                                                    border_color_focus: #x02030B
                                                    border_radius: 8.
                                                    border_size: 1.
                                                }
                                                draw_text +: {
                                                    color: #x822801
                                                    color_hover: #x822801
                                                    color_down: #x822801
                                                    color_focus: #x822801
                                                    color_empty: #x3A4554
                                                    color_empty_hover: #x0B1220
                                                    color_empty_focus: #x0B1220
                                                    text_style +: {font_size: 16.}
                                                }
                                            }
                                        }
                                    }

                                    // Exclusive-tax shops quote net prices, so the strip above is
                                    // not what the customer pays — this line is.
                                    tax_estimate_label := WrapLabel{
                                        body: " "
                                        font_color: #xC44E10
                                        font_size: 13.
                                    }

                                    // Amounts only — full bill in one field, or mix that adds up.
                                    h_mix_type_hint_moklanjo := WrapLabel{
                                        body: "Type how much in each (cash / UPI / card / credit). Must add up to the bill. Credit needs a customer above."
                                        font_color: #x3A4554
                                        font_size: 11.
                                        visible: false
                                    }
                                    View{
                                        width: Fill height: Fit
                                        flow: Right spacing: 8
                                        align: Align{x: 0.0 y: 0.5}
                                        money_lane_label := WrapLabel{
                                            width: Fit
                                            body: "Bill: Tax invoice"
                                            font_color: #x3A4554
                                            font_size: 12.
                                        }
                                        payment_help := InfoHelp{
                                            width: Fit
                                            tip: "Type how much in each (cash / UPI / card / credit). Must add up to the bill. Credit needs a customer above."
                                        }
                                    }
                                    View{
                                        width: Fill height: Fit
                                        flow: Right spacing: 6
                                        View{width: Fill height: Fit flow: Overlay
                                            lane_declared := SegBtn{text: "Tax invoice"}
                                            lane_declared_on := SegBtnOn{text: "Tax invoice" visible: false}
                                        }
                                        View{width: Fill height: Fit flow: Overlay
                                            lane_owner := SegBtn{text: "Cash memo"}
                                            lane_owner_on := SegBtnOn{text: "Cash memo" visible: false}
                                        }
                                    }
                                    mix_cash_input := FieldInput{empty_text: "Cash amount"}
                                    mix_upi_input := FieldInput{empty_text: "UPI amount"}
                                    mix_card_input := FieldInput{empty_text: "Card amount"}
                                    mix_credit_input := FieldInput{empty_text: "Credit / khata amount"}
                                    ActionStack{
                                        pay_mix := PayBtn{text: "Take payment"}
                                    }

                                }

                                cart_round_hint := WrapLabel{
                                    body: "Rounded starts equal to Applied. Edit only to round the bill."
                                    font_color: #x0B1220
                                    font_size: 12.
                                    visible: false
                                }
                                credit_chip := WrapLabel{
                                    body: "No customer yet — type a name, then tap Credit"
                                    font_color: #x0B1220
                                    font_size: 13.
                                }
                                customer_name_input := FieldInput{
                                    empty_text: "Customer name"
                                }
                                customer_name_suggest := mod.widgets.SuggestList{field: "customer_name"}
                                customer_use := NavBtn{text: "Add / use this name"}
                                customer_clear := NavBtn{text: "Clear customer"}
                                customers_empty_hint := WrapLabel{
                                    width: Fit
                                    body: "Saved customers show up here for quick pick."
                                    font_color: #x0B1220
                                    font_size: 12.
                                }
                                customer_list_sell := mod.widgets.CustomerList{}
                                    } // pos_cart_col
                                    } // pos_cart_scroll
                                } // booking_desk_row
                            } // booking_desk_hscroll

                        } // pos_screen

                        menu_screen := ScrollYView{
                            width: Fill height: Fill
                            visible: false
                            View{
                                width: Fill height: Fit
                                flow: Down spacing: 10
                                padding: Inset{left: 16. right: 16. top: 8. bottom: 120.}
                                items_help_label := InfoHelp{
                                    width: Fit
                                    tip: "Each row is a rate (e.g. Surface Zone A). Unit is usually kg. Tap to edit."
                                }
                                FormCard{
                                menu_mode_label := WrapLabel{body: "Adding a new item" font_color: #x02030B font_size: 14.}
                                menu_name_input := FieldInput{
                                    empty_text: "Item name — type it in your language"
                                }
                                menu_aliases_input := FieldInput{
                                    empty_text: "Aliases — other names, comma separated"
                                }
                                FormFieldRow{
                                    menu_price_input := FieldInput{empty_text: "0"}
                                    menu_unit_input := FieldInput{empty_text: "Unit (kg or blank)"}
                                }
                                menu_unit_suggest := mod.widgets.SuggestList{field: "menu_unit"}
                                FormFieldRow{
                                    menu_first_kg_input := FieldInput{empty_text: "First kg (slab)"}
                                    menu_first_amt_input := FieldInput{empty_text: "First amount"}
                                }
                                FormFieldRow{
                                    menu_extra_kg_input := FieldInput{empty_text: "Extra / kg"}
                                    menu_zone_input := FieldInput{empty_text: "Zone name"}
                                }
                                menu_zone_suggest := mod.widgets.SuggestList{field: "menu_zone"}
                                FormFieldRow{
                                    menu_carrier_input := FieldInput{empty_text: "Carrier (blank = our sell rate)"}
                                    menu_party_input := FieldInput{empty_text: "Party (blank = all customers)"}
                                }
                                menu_carrier_suggest := mod.widgets.SuggestList{field: "menu_carrier"}
                                menu_party_suggest := mod.widgets.SuggestList{field: "menu_party"}
                                menu_tax_input := FieldInput{
                                    empty_text: "Tax rate name or % — Standard, Reduced, 0, 5, 12, 18, 20"
                                }
                                menu_tax_suggest := mod.widgets.SuggestList{field: "menu_tax"}
                                menu_category_input := FieldInput{empty_text: "Category (optional, created if new)"}
                                menu_category_suggest := mod.widgets.SuggestList{field: "menu_category"}
                                menu_modifiers_input := FieldInput{empty_text: "Options (optional) — Large=20, Gift wrap=10"}
                                View{
                                    width: Fill height: Fit
                                    flow: Right spacing: 8
                                    align: Align{y: 0.5}
                                    h_item_photo := WrapLabel{
                                                    width: Fit
                                        body: "Photo"
                                        font_color: #x0B1220
                                        font_size: 15.
                                    }
                                    item_photo_help := InfoHelp{
                                        width: Fit
                                        tip: "Optional. Without a photo we show initials from the item name."
                                    }
                                }
                                image_status := WrapLabel{
                                    body: " "
                                    font_color: #x3A4554
                                    font_size: 12.
                                }
                                draft_preview := ThumbAvatar{
                                    width: 96 height: 96
                                    margin: Inset{top: 0. bottom: 0. left: 0. right: 0.}
                                }
                                ActionStack{
                                    image_clear := RowSecondaryBtn{text: "Clear photo"}
                                    image_pick := RowSecondaryBtn{text: "Pick photo"}
                                }
                                // Add / Save after every field — match Khaynjo.
                                ActionStack{
                                    menu_new := StackSecondaryBtn{
                                        text: "New item"
                                        visible: false
                                    }
                                    menu_add := StackPrimaryBtn{text: "Add rate"}
                                }
                                }
                                menu_empty := WrapLabel{
                                    width: Fit
                                    body: "No items yet — add one above."
                                    font_color: #x0B1220
                                    font_size: 14.
                                }
                                product_list_menu := mod.widgets.MenuList{}
                            }
                        }

                        // Money out — purchase + expense (peer of Sell).
                        buy_screen := ScrollYView{
                            width: Fill height: Fill
                            visible: false
                            View{
                                width: Fill height: Fit
                                flow: Down spacing: 10
                                padding: Inset{left: 16. right: 16. top: 8. bottom: 120.}
                                buy_help := InfoHelp{
                                    width: Fit
                                    tip: "Money out: goods received (purchase) or running costs (expense). Same books as Booking."
                                }
                                h_goods_received := WrapLabel{body: "Goods received" font_color: #x0B1220 font_size: 15.}
                                buy_item_input := FieldInput{empty_text: "Item name"}
                                buy_item_suggest := mod.widgets.SuggestList{field: "buy_item"}
                                buy_qty_input := FieldInput{empty_text: "Qty (e.g. 1.5 for kg)"}
                                buy_cost_input := FieldInput{empty_text: "Unit cost"}
                                buy_supplier_input := FieldInput{empty_text: "Supplier name (or party)"}
                                buy_supplier_suggest := mod.widgets.SuggestList{field: "buy_supplier"}
                                h_buy_party_kind := Label{
                                    text: "New party kind"
                                    draw_text.color: #x3A4554
                                    draw_text.text_style.font_size: 11.
                                }
                                View{
                                    width: Fill height: Fit
                                    flow: Right spacing: 6
                                    View{width: Fill height: Fit flow: Overlay
                                        buy_kind_supplier := SegBtn{text: "Supplier"}
                                        buy_kind_supplier_on := SegBtnOn{text: "Supplier" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        buy_kind_carrier := SegBtn{text: "Carrier"}
                                        buy_kind_carrier_on := SegBtnOn{text: "Carrier" visible: false}
                                    }
                                }
                                buy_note_input := FieldInput{empty_text: "Note (optional)"}
                                View{
                                    width: Fill height: Fit
                                    flow: Right spacing: 6
                                    View{width: Fill height: Fit flow: Overlay
                                        buy_paid := SegBtn{text: "Paid now"}
                                        buy_paid_on := SegBtnOn{text: "Paid now" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        buy_on_account := SegBtn{text: "On account (AP)"}
                                        buy_on_account_on := SegBtnOn{text: "On account (AP)" visible: false}
                                    }
                                }
                                buy_purchase := RowPrimaryBtn{text: "Record purchase"}
                                h_expense_heading := WrapLabel{body: "Expense" font_color: #x0B1220 font_size: 15.}
                                buy_expense_amount := FieldInput{empty_text: "Amount"}
                                buy_expense_note := FieldInput{empty_text: "Note (fuel, rent, …)"}
                                h_paid_by := Label{text: "Paid by" draw_text.color: #x3A4554 draw_text.text_style.font_size: 11.}
                                View{
                                    width: Fill height: Fit
                                    flow: Right spacing: 6
                                    View{width: Fill height: Fit flow: Overlay
                                        buy_exp_t1_cash := SegBtn{text: "Cash"}
                                        buy_exp_t1_cash_on := SegBtnOn{text: "Cash" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        buy_exp_t1_bank := SegBtn{text: "Bank"}
                                        buy_exp_t1_bank_on := SegBtnOn{text: "Bank" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        buy_exp_t1_upi := SegBtn{text: "UPI"}
                                        buy_exp_t1_upi_on := SegBtnOn{text: "UPI" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        buy_exp_t1_card := SegBtn{text: "Card"}
                                        buy_exp_t1_card_on := SegBtnOn{text: "Card" visible: false}
                                    }
                                }
                                buy_exp_part2 := FieldInput{empty_text: "Part by a second way (optional)"}
                                h_second_way := Label{text: "Second way" draw_text.color: #x3A4554 draw_text.text_style.font_size: 11.}
                                View{
                                    width: Fill height: Fit
                                    flow: Right spacing: 6
                                    View{width: Fill height: Fit flow: Overlay
                                        buy_exp_t2_cash := SegBtn{text: "Cash"}
                                        buy_exp_t2_cash_on := SegBtnOn{text: "Cash" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        buy_exp_t2_bank := SegBtn{text: "Bank"}
                                        buy_exp_t2_bank_on := SegBtnOn{text: "Bank" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        buy_exp_t2_upi := SegBtn{text: "UPI"}
                                        buy_exp_t2_upi_on := SegBtnOn{text: "UPI" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        buy_exp_t2_card := SegBtn{text: "Card"}
                                        buy_exp_t2_card_on := SegBtnOn{text: "Card" visible: false}
                                    }
                                }
                                buy_expense_add := RowPrimaryBtn{text: "Add expense"}
                                buy_to_inventory := RowSecondaryBtn{text: "Inventory & stock adjust"}
                            }
                        }

                        more_screen := ScrollYView{
                            width: Fill height: Fill
                            visible: false
                            View{
                                width: Fill height: Fit
                                flow: Down spacing: 10
                                padding: Inset{left: 16. right: 16. top: 8. bottom: 120.}
                                more_help := InfoHelp{
                                    width: Fit
                                    tip: "Catalogue, sales, credit, team, backup — plus stock and extra tills when needed."
                                }
                                more_items := MoreRow{text: "Rates & services"}
                                more_ops := MoreRow{text: "Consignments & dispatch"}
                                more_cod := MoreRow{text: "COD register"}
                                more_riders := MoreRow{text: "Riders (own fleet)"}
                                more_sales := MoreRow{text: "Sales & refunds"}
                                more_day_close := MoreRow{text: "Day close"}
                                more_reports := MoreRow{text: "Reports & bank"}
                                more_inventory := MoreRow{text: "Inventory & stock"}
                                more_branches := MoreRow{text: "Branches & org"}
                                more_credit := MoreRow{text: "Parties & credit"}
                                more_team := MoreRow{text: "Team & PINs"}
                                more_settings := MoreRow{text: "Backup & settings"}
                                more_lock := MoreRow{text: "Lock app"}
                                more_mode_hint := WrapLabel{
                                    width: Fit
                                    body: " "
                                    font_color: #x3A4554
                                    font_size: 12.
                                }
                            }
                        }

                        sales_screen := ScrollYView{
                            width: Fill height: Fill
                            visible: false
                            View{
                                width: Fill height: Fit
                                flow: Down spacing: 10
                                padding: Inset{left: 16. right: 16. top: 8. bottom: 120.}
                                sales_help := InfoHelp{
                                    width: Fit
                                    tip: "Select a sale to amend, void, or refund. Amendments keep full history. Rounded Total starts equal to Applied — edit only to round. Void and refund may need an Owner or Manager PIN."
                                }
                                sale_selected_label := WrapLabel{
                                    body: "No sale selected"
                                    font_color: #x0B1220
                                    font_size: 13.
                                }
                                receipt_share_btn := NavBtn{text: "Copy receipt"}
                                receipt_print_btn := NavBtn{text: "Print / open"}
                                day_close_from_sales := NavBtn{text: "Day close summary"}
                                amend_cart_list := mod.widgets.CartList{}
                                amend_natural_label := WrapLabel{
                                    body: "Actual Total 0"
                                    font_color: #x0B1220
                                    font_size: 13.
                                }
                                amend_applied_label := WrapLabel{
                                    body: "Applied Total 0"
                                    font_color: #x0B1220
                                    font_size: 13.
                                }
                                    View{
                                        width: Fill height: Fit
                                        flow: Right spacing: 8
                                        align: Align{y: 0.5}
                                        h_rounded_total := WrapLabel{
                                            body: "Rounded Total"
                                            font_color: #xC44E10
                                            font_size: 14.
                                        }
                                        amend_total_input := CartField{
                                        padding: Inset{left: 14. right: 14. top: 14. bottom: 14.}
                                        empty_text: "0"
                                        text: "0"
                                        draw_bg +: {
                                            border_color: #xC44E10
                                            border_color_hover: #x822801
                                            border_color_focus: #x02030B
                                            border_radius: 12.
                                            border_size: 2.
                                        }
                                        draw_text +: {
                                            color: #x822801
                                            color_hover: #x822801
                                            color_down: #x822801
                                            color_focus: #x822801
                                            color_empty: #x3A4554
                                            color_empty_hover: #x0B1220
                                            color_empty_focus: #x0B1220
                                            text_style +: {font_size: 16.}
                                        }
                                    }
                                }
                                amend_round_hint := WrapLabel{
                                    body: "Rounded Total starts equal to Applied Total. Edit only to round the bill."
                                    font_color: #x0B1220
                                    font_size: 12.
                                    visible: false
                                }
                                View{
                                    width: Fill height: Fit
                                    flow: Right spacing: 6
                                    View{width: Fill height: Fit flow: Overlay
                                        amend_pay_cash := SegBtn{text: "Cash"}
                                        amend_pay_cash_on := SegBtnOn{text: "Cash" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        amend_pay_upi := SegBtn{text: "UPI"}
                                        amend_pay_upi_on := SegBtnOn{text: "UPI" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        amend_pay_card := SegBtn{text: "Card"}
                                        amend_pay_card_on := SegBtnOn{text: "Card" visible: false}
                                    }
                                }
                                reason_input := FieldInput{
                                    empty_text: "Reason for amend / void / refund"
                                }
                                approval_pin_input := FieldInput{
                                    empty_text: "Manager PIN to approve (only if you cannot void)"
                                }
                                approval_hint := WrapLabel{
                                    body: " "
                                    font_color: #x3A4554
                                    font_size: 12.
                                    visible: false
                                }
                                ActionStack{
                                    amend_btn := RowPrimaryBtn{text: "Amend"}
                                    void_btn := DangerBtn{text: "Void"}
                                    refund_btn := DangerBtn{text: "Refund"}
                                }
                                sale_history_label := mod.widgets.ReportText{}
                                h_notes_attach_sale := WrapLabel{body: "Notes & attachments (selected sale)" font_color: #x0B1220 font_size: 15.}
                                sale_attach_note := FieldInput{empty_text: "Extra note on this AWB / sale"}
                                ActionStack{
                                    sale_attach_add_note := RowPrimaryBtn{text: "Add note"}
                                    sale_attach_photo := RowSecondaryBtn{text: "Attach photo"}
                                    sale_attach_pdf := RowSecondaryBtn{text: "Attach PDF"}
                                }
                                sale_attach_hint := WrapLabel{
                                    width: Fit
                                    body: "(select a sale)"
                                    font_color: #x3A4554
                                    font_size: 12.
                                }
                                sale_attach_list := mod.widgets.ReportText{}
                                sale_list := mod.widgets.SaleList{}
                            }
                        }

                        credit_screen := ScrollYView{
                            width: Fill height: Fill
                            visible: false
                            View{
                                width: Fill height: Fit
                                flow: Down spacing: 10
                                padding: Inset{left: 16. right: 16. top: 8. bottom: 120.}
                                credit_help := InfoHelp{
                                    width: Fit
                                    tip: "Party id is unique (names may repeat). Roles: Consignor (AR), Carrier (AP), or both on that id. Ledgers stay separate; reports show net."
                                }
                                FormCard{
                                FormFieldRow{
                                    credit_add_name := FieldInput{empty_text: "Party name"}
                                    credit_add_phone := FieldInput{empty_text: "Phone"}
                                }
                                FormFieldRow{
                                    party_country_input := FieldInput{empty_text: "Country"}
                                    party_state_input := FieldInput{empty_text: "State / region"}
                                }
                                party_country_suggest := mod.widgets.SuggestList{field: "party_country"}
                                party_state_suggest := mod.widgets.SuggestList{field: "party_state"}
                                party_pincode_input := FieldInput{empty_text: "PIN / ZIP / postcode"}
                                party_pincode_suggest := mod.widgets.SuggestList{field: "party_pincode"}
                                credit_add_address := FieldInput{empty_text: "Street / area address"}
                                // Optional logo — same pick/clear pattern as Rates photos.
                                party_image_hint := WrapLabel{
                                    body: "Logo: none (initials)"
                                    font_color: #x0B1220
                                    font_size: 12.
                                }
                                party_draft_preview := ThumbAvatar{
                                    width: 72 height: 72
                                    margin: Inset{top: 0. bottom: 0. left: 0. right: 0.}
                                }
                                ActionStack{
                                    party_image_clear := RowSecondaryBtn{text: "Clear logo"}
                                    party_image_pick := RowSecondaryBtn{text: "Pick logo"}
                                }
                                h_account_kind := WrapLabel{
                                    body: "Account kind (tap one)"
                                    font_color: #x3A4554
                                    font_size: 12.
                                }
                                // Khaynjo practise: equal-width SegBtn row — all four
                                // roles always visible; no ChipStrip / Overlay chevrons.
                                View{
                                    width: Fill height: Fit
                                    flow: Right spacing: 6
                                    View{width: Fill height: Fit flow: Overlay
                                        credit_kind_consignor := SegBtn{text: "Consignor"}
                                        credit_kind_consignor_on := SegBtnOn{text: "Consignor" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        credit_kind_carrier := SegBtn{text: "Carrier"}
                                        credit_kind_carrier_on := SegBtnOn{text: "Carrier" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        credit_kind_both := SegBtn{text: "Both"}
                                        credit_kind_both_on := SegBtnOn{text: "Both" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        credit_kind_supplier := SegBtn{text: "Supplier"}
                                        credit_kind_supplier_on := SegBtnOn{text: "Supplier" visible: false}
                                    }
                                }
                                // SegBtns write here (Khaynjo hides this; Moklanjo keeps it
                                // visible so a rare custom role word can still be typed).
                                credit_add_kind := FieldInput{empty_text: "Custom role (optional) — or tap a chip above"}
                                credit_add_btn := StackPrimaryBtn{text: "Add party"}
                                ActionStack{
                                    party_form_clear := RowSecondaryBtn{text: "New party (clear form)"}
                                    party_delete := DangerBtn{text: "Delete selected party"}
                                }
                                }
                                parties_list_heading := WrapLabel{
                                    body: "Saved parties"
                                    font_color: #x0B1220
                                    font_size: 15.
                                }
                                customer_list_credit := mod.widgets.CustomerList{}
                                // Detail tools only after Pick (Khaynjo party_detail pattern).
                                party_detail := View{
                                    width: Fill height: Fit
                                    flow: Down spacing: 10
                                    visible: false
                                FormCard{
                                View{
                                    width: Fill height: Fit
                                    flow: Right spacing: 8
                                    align: Align{x: 0.0 y: 0.5}
                                    h_party_custom_heading := WrapLabel{ width: Fit
                                                    body: "Custom fields" font_color: #x0B1220 font_size: 15.}
                                    party_custom_help := InfoHelp{
                                        width: Fit
                                        tip: "Add any label and value on this party — WhatsApp, GSTIN, route code, …"
                                    }
                                }
                                FormFieldRow{
                                    party_custom_key := FieldInput{empty_text: "Field name (e.g. WhatsApp)"}
                                    party_custom_value := FieldInput{empty_text: "Value"}
                                }
                                ActionStack{
                                    party_custom_add := RowSecondaryBtn{text: "Add field"}
                                    party_custom_remove := RowSecondaryBtn{text: "Remove field"}
                                }
                                party_custom_list := mod.widgets.ReportText{}
                                }
                                FormCard{
                                h_credit_limit_heading := WrapLabel{body: "Credit limit" font_color: #x0B1220 font_size: 15.}
                                FormFieldRow{
                                    credit_limit_input := FieldInput{empty_text: "Credit limit (0 = none)"}
                                    credit_limit_party := FieldInput{empty_text: "Party name (or use selected)"}
                                }
                                credit_party_suggest := mod.widgets.SuggestList{field: "credit_party"}
                                ActionStack{
                                    credit_limit_set := RowSecondaryBtn{text: "Apply limit to party"}
                                    credit_period_bill := RowSecondaryBtn{text: "Period bill"}
                                }
                                }
                                FormCard{
                                h_pay_selected_heading := WrapLabel{body: "Pay selected (settle debt / take advance)" font_color: #x0B1220 font_size: 15.}
                                settle_amount_input := FieldInput{empty_text: "Amount received"}
                                FormFieldRow{
                                    settle_tds_input := FieldInput{empty_text: "TDS cut (optional)"}
                                    settle_discount_input := FieldInput{empty_text: "Discount (optional)"}
                                }
                                settle_cheque_input := FieldInput{empty_text: "Cheque number (optional)"}
                                ActionStack{
                                    settle_cash := NavBtn{text: "Pay cash"}
                                    settle_upi := NavBtn{text: "Pay UPI"}
                                    settle_card := NavBtn{text: "Pay card"}
                                }
                                settle_open_bills := mod.widgets.ReportText{}
                                settle_bill_input := FieldInput{empty_text: "Against bill # above (optional)"}
                                credit_bill_chase := RowSecondaryBtn{text: "Bill reminder chase list"}
                                h_write_off_heading := WrapLabel{body: "Write off (clear without payment)" font_color: #x0B1220 font_size: 15.}
                                writeoff_amount_input := FieldInput{empty_text: "Amount (blank = full balance)"}
                                writeoff_reason_input := FieldInput{empty_text: "Reason (required)"}
                                credit_writeoff := NavBtn{text: "Write off / nil balance"}
                                h_pay_ap_heading := WrapLabel{body: "Pay what we owe them (AP)" font_color: #x0B1220 font_size: 15.}
                                payable_amount_input := FieldInput{empty_text: "Amount we pay"}
                                ActionStack{
                                    payable_cash := NavBtn{text: "Pay AP cash"}
                                    payable_upi := NavBtn{text: "Pay AP UPI"}
                                    payable_card := NavBtn{text: "Pay AP card"}
                                }
                                payable_open_bills := mod.widgets.ReportText{}
                                payable_bill_input := FieldInput{empty_text: "Against bill # above (optional)"}
                                payable_writeoff := NavBtn{text: "Write off AP"}
                                }
                                FormCard{
                                h_notes_attach_party := WrapLabel{body: "Notes & attachments" font_color: #x0B1220 font_size: 15.}
                                party_attach_note := FieldInput{empty_text: "Note / GSTIN / agreement reminder"}
                                ActionStack{
                                    party_attach_add_note := RowSecondaryBtn{text: "Add note"}
                                    party_attach_photo := RowSecondaryBtn{text: "Attach photo"}
                                    party_attach_pdf := RowSecondaryBtn{text: "Attach PDF"}
                                }
                                party_attach_hint := WrapLabel{
                                    width: Fit
                                    body: "(select a party)"
                                    font_color: #x3A4554
                                    font_size: 12.
                                }
                                party_attach_list := mod.widgets.ReportText{}
                                }
                                } // party_detail
                            }
                        }

                        team_screen := ScrollYView{
                            width: Fill height: Fill
                            visible: false
                            View{
                                width: Fill height: Fit
                                flow: Down spacing: 10
                                padding: Inset{left: 16. right: 16. top: 8. bottom: 120.}
                                team_help := InfoHelp{
                                    width: Fit
                                    tip: "Owner creates each person with a name, role title, permission preset, and starting PIN. Owner cannot change anyone's PIN later — only that person can update their own PIN."
                                }
                                staff_label := mod.widgets.ReportText{}
                                FormCard{
                                staff_name_input := FieldInput{empty_text: "Staff name / id"}
                                staff_title_input := FieldInput{empty_text: "Role title (e.g. Barista)"}
                                staff_pin_input := FieldInput{empty_text: "Starting PIN (4–12 digits)"}
                                h_permission_preset := WrapLabel{
                                    body: "Permission preset"
                                    font_color: #x0B1220
                                    font_size: 13.
                                }
                                h_preset_help_moklanjo := WrapLabel{
                                    width: Fit
                                    body: ""
                                    font_color: #x3A4554
                                    font_size: 12.
                                    visible: false
                                }
                                View{
                                    width: Fill height: Fit
                                    flow: Right spacing: 6
                                    View{width: Fill height: Fit flow: Overlay
                                        staff_role_cashier := SegBtn{text: "Cashier"}
                                        staff_role_cashier_on := SegBtnOn{text: "Cashier" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        staff_role_manager := SegBtn{text: "Manager"}
                                        staff_role_manager_on := SegBtnOn{text: "Manager" visible: false}
                                    }
                                }
                                staff_add_cashier := StackPrimaryBtn{text: "Add staff"}
                                staff_deactivate := DangerBtn{text: "Remove selected"}
                                }
                                FormCard{
                                h_optional_login_help := InfoHelp{
                                    width: Fit
                                    tip: "Optional login (id + password). Owner sets once; only the user can change password later."
                                }
                                staff_login_input := FieldInput{empty_text: "Login id (optional)"}
                                staff_password_input := FieldInput{empty_text: "Starting password (optional)"}
                                staff_set_login := RowSecondaryBtn{text: "Apply login to selected"}
                                }
                                FormCard{
                                h_feature_access_help := WrapLabel{
                                    body: "Feature access for selected staff"
                                    font_color: #x0B1220
                                    font_size: 13.
                                }
                                // Tabs that scroll sideways: ‹ › at the ends, each only while there
                                // is more that way (sync_tab_chevrons).
                                feat_tabs := View{
                                    width: Fill height: Fit
                                    flow: Overlay
                                    tabs := ChipStrip{
                                        ChipStripRow{
                                            View{width: Fit height: 40 flow: Overlay
                                                feat_sell := ChipBtn{text: "Sell"}
                                                feat_sell_on := ChipBtnActive{text: "Sell" visible: false}
                                            }
                                            View{width: Fit height: 40 flow: Overlay
                                                feat_items := ChipBtn{text: "Items"}
                                                feat_items_on := ChipBtnActive{text: "Items" visible: false}
                                            }
                                            View{width: Fit height: 40 flow: Overlay
                                                feat_sales := ChipBtn{text: "Sales"}
                                                feat_sales_on := ChipBtnActive{text: "Sales" visible: false}
                                            }
                                            View{width: Fit height: 40 flow: Overlay
                                                feat_credit := ChipBtn{text: "Credit"}
                                                feat_credit_on := ChipBtnActive{text: "Credit" visible: false}
                                            }
                                            View{width: Fit height: 40 flow: Overlay
                                                feat_team := ChipBtn{text: "Team"}
                                                feat_team_on := ChipBtnActive{text: "Team" visible: false}
                                            }
                                            View{width: Fit height: 40 flow: Overlay
                                                feat_backup := ChipBtn{text: "Backup"}
                                                feat_backup_on := ChipBtnActive{text: "Backup" visible: false}
                                            }
                                            View{width: Fit height: 40 flow: Overlay
                                                feat_inventory := ChipBtn{text: "Inventory"}
                                                feat_inventory_on := ChipBtnActive{text: "Inventory" visible: false}
                                            }
                                            View{width: Fit height: 40 flow: Overlay
                                                feat_branches := ChipBtn{text: "Branches"}
                                                feat_branches_on := ChipBtnActive{text: "Branches" visible: false}
                                            }
                                        }
                                    }
                                    View{
                                        width: Fill height: 52
                                        align: Align{x: 0.0 y: 0.5}
                                        chev_l := ChevronBtn{text: "‹" visible: false}
                                    }
                                    View{
                                        width: Fill height: 52
                                        align: Align{x: 1.0 y: 0.5}
                                        chev_r := ChevronBtn{text: "›" visible: false}
                                    }
                                }
                                // Pick who to edit by name; their rights below are a
                                // draft until Save.
                                team_pick_input := FieldInput{empty_text: "Tap a staff name in the list to edit"}
                                team_pick_suggest := mod.widgets.SuggestList{field: "team_pick"}
                                // Pick selects immediately; keep for typed + Enter only (hidden).
                                team_pick_btn := NavBtn{text: "Select this staff" visible: false}
                                team_selected_label := WrapLabel{
                                    width: Fit
                                    body: " "
                                    font_color: #x0B1220
                                    font_size: 13.
                                }
                                h_rights_help := InfoHelp{
                                    width: Fit
                                    tip: "Rights — tap to change: Allowed → With approval (a manager's PIN at that moment) → Not allowed."
                                }
                                right_void_btn := NavBtn{text: "Void or refund a booking"}
                                right_amend_btn := NavBtn{text: "Amend a recorded booking"}
                                feat_save := StackPrimaryBtn{text: "Save features"}
                                }
                                FormCard{
                                h_change_my_pin_heading := WrapLabel{
                                    body: "Change my PIN (only yours)"
                                    font_color: #x0B1220
                                    font_size: 15.
                                }
                                my_pin_input := FieldInput{empty_text: "New PIN (4–12 digits)"}
                                change_my_pin := StackPrimaryBtn{text: "Update my PIN"}
                                my_password_input := FieldInput{empty_text: "New password (optional)"}
                                change_my_password := RowSecondaryBtn{text: "Update my password"}
                                }
                                FormCard{
                                View{
                                    width: Fill height: Fit
                                    flow: Right spacing: 6
                                    align: Align{y: 0.5}
                                    h_duress_heading := WrapLabel{
                                        width: Fit
                                        body: "Emergency unlock PIN (Owner only)"
                                        font_color: #x0B1220
                                        font_size: 15.
                                    }
                                    h_duress_help := InfoHelp{
                                        width: Fit
                                        tip: "Optional. If this PIN is typed at unlock instead of yours, the desk opens with no booking history — rates, parties, zones and staff stay. Use only if you are forced to unlock. Leave blank and save to turn off. Never the same as anyone's PIN."
                                    }
                                }
                                duress_status := WrapLabel{
                                    width: Fit
                                    body: " "
                                    font_color: #x475569
                                    font_size: 12.
                                }
                                duress_pin_input := FieldInput{empty_text: "Emergency PIN (4–12 digits)"}
                                duress_pin_confirm := FieldInput{empty_text: "Type again to confirm"}
                                save_duress_pin := StackPrimaryBtn{text: "Save emergency PIN"}
                                clear_duress_pin := RowSecondaryBtn{text: "Turn off emergency PIN"}
                                }
                            }
                        }

                        settings_screen := ScrollYView{
                            width: Fill height: Fill
                            visible: false
                            View{
                                width: Fill height: Fit
                                flow: Down spacing: 12
                                padding: Inset{left: 16. right: 16. top: 8. bottom: 120.}
                                settings_help := InfoHelp{
                                    width: Fit
                                    tip: "Follow this device’s language, or pick one yourself."
                                }
                                sidebar_settings_open := NavBtn{text: "Sidebar settings"}
                                locale_label := mod.widgets.ReportText{}
                                h_country_tax := WrapLabel{body: "Country & tax" font_color: #x0B1220 font_size: 20.}
                                h_country_pack_help := WrapLabel{
                                    width: Fill
                                    body: ""
                                    font_color: #x0B1220
                                    font_size: 12.
                                    visible: false
                                }
                                FormCard{
                                fiscal_label := mod.widgets.ReportText{}
                                country_code_input := FieldInput{
                                    empty_text: "Country code e.g. IN, GB, US, AE, BR"
                                }
                                country_suggest := mod.widgets.SuggestList{field: "country"}
                                ActionStack{
                                    country_prev := RowSecondaryBtn{text: "Previous country"}
                                    country_next := RowSecondaryBtn{text: "Next country"}
                                }
                                country_apply := StackPrimaryBtn{text: "Apply country pack"}
                                country_hint := WrapLabel{
                                    body: " "
                                    font_color: #x3A4554
                                    font_size: 12.
                                }
                                country_pack_list := mod.widgets.ReportText{}
                                }
                                FormCard{
                                View{
                                    width: Fill height: Fit
                                    flow: Right spacing: 6
                                    View{width: Fill height: Fit flow: Overlay
                                        tax_inclusive_btn := SegBtn{text: "Tax inside price"}
                                        tax_inclusive_btn_on := SegBtnOn{text: "Tax inside price" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        tax_exclusive_btn := SegBtn{text: "Tax added at till"}
                                        tax_exclusive_btn_on := SegBtnOn{text: "Tax added at till" visible: false}
                                    }
                                }
                                business_tax_id_input := FieldInput{empty_text: "Business tax id"}
                                service_bps_input := FieldInput{
                                    empty_text: "Service % optional 0/10/12"
                                }
                                fiscal_save := StackPrimaryBtn{text: "Save business id"}
                                }
                                h_language := WrapLabel{body: "Language" font_color: #x0B1220 font_size: 15.}
                                h_language_setup_hint := WrapLabel{
                                    width: Fill
                                    body: "Follow this device’s language, or pick one yourself."
                                    font_color: #x0B1220
                                    font_size: 12.
                                }
                                ActionStack{
                                    View{
                                        width: Fill height: Fit
                                        flow: Overlay
                                        settings_lang_platform := StackSecondaryBtn{text: "Use platform native language"}
                                        settings_lang_platform_on := StackPrimaryBtn{text: "Use platform native language" visible: false}
                                    }
                                    View{
                                        width: Fill height: Fit
                                        flow: Overlay
                                        settings_lang_user := StackSecondaryBtn{text: "Use user selected language"}
                                        settings_lang_user_on := StackPrimaryBtn{text: "Use user selected language" visible: false}
                                    }
                                }
                                settings_lang_selected := WrapLabel{
                                    body: ""
                                    font_color: #x0B1220
                                    font_size: 13.
                                }
                                language_list := mod.widgets.LanguageList{}
                                h_currency := WrapLabel{body: "Currency" font_color: #x0B1220 font_size: 15.}
                                h_currency_use_help := WrapLabel{
                                    width: Fill
                                    body: "Scroll and tap Use for any supported currency."
                                    font_color: #x0B1220
                                    font_size: 12.
                                }
                                currency_list := mod.widgets.CurrencyList{}
                                View{
                                    width: Fill height: Fit
                                    flow: Right spacing: 6
                                    align: Align{y: 0.5}
                                    h_extra_tills_heading := WrapLabel{
                                        width: Fit
                                        body: "Extra tills"
                                        font_color: #x0B1220
                                        font_size: 20.
                                    }
                                    server_pair_help := InfoHelp{
                                        width: Fit
                                        tip: "On the other device: Client Mode → Find server → enter this pairing code."
                                    }
                                }
                                h_other_devices_help_heading := WrapLabel{
                                    body: "Set up other devices"
                                    font_color: #x0B1220
                                    font_size: 15.
                                    visible: false
                                }
                                h_other_devices_help := WrapLabel{
                                    width: Fill
                                    body: " "
                                    font_color: #x3A4554
                                    font_size: 12.
                                    visible: false
                                }
                                // Big digits stay visible even when firewall/listen notes fill the status line.
                                pairing_code_value := WrapLabel{
                                    width: Fit
                                    body: " "
                                    font_color: #x0B1220
                                    font_size: 28.
                                    selectable: true
                                    visible: false
                                }
                                server_pair_label := WrapLabel{
                                    width: Fill
                                    body: " "
                                    font_color: #x3A4554
                                    font_size: 13.
                                    selectable: false
                                }
                                ActionStack{
                                    allow_extra_tills := RowPrimaryBtn{text: "Allow extra tills"}
                                    stop_extra_tills := RowSecondaryBtn{text: "Stop extra tills"}
                                    allow_firewall := RowPrimaryBtn{text: "Allow other devices to connect" visible: false}
                                }
                                autostart_btn := RowSecondaryBtn{text: "Start with this computer" visible: false}
                                new_pair_code := RowSecondaryBtn{text: "New pairing code" visible: false}
                                paired_list := mod.widgets.PairedList{}
                                // A paired device: its link, what waits, what was not accepted.
                                client_connection_btn := RowSecondaryBtn{text: "Connection to the shop" visible: false}
                                settings_open_branches := NavBtn{text: "Branches & org"}
                                h_start_template := WrapLabel{body: "Product" font_color: #x0B1220 font_size: 20.}
                                settings_tmpl_label := WrapLabel{
                                    body: "Moklanjo"
                                    font_color: #x3A4554
                                    font_size: 13.
                                }
                                h_product_only_moklanjo := WrapLabel{
                                    width: Fill
                                    body: ""
                                    font_color: #x3A4554
                                    font_size: 12.
                                    visible: false
                                }
                                FormCard{
                                h_your_brand := WrapLabel{body: "Your brand (in-app)" font_color: #x0B1220 font_size: 20.}
                                h_brand_shown_help := WrapLabel{
                                    width: Fill
                                    body: ""
                                    font_color: #x0B1220
                                    font_size: 12.
                                    visible: false
                                }
                                venue_title_input := FieldInput{empty_text: "Name of this place"}
                                View{
                                    width: Fill height: Fit
                                    flow: Right spacing: 8
                                    align: Align{y: 0.5}
                                    h_venue_icon := WrapLabel{
                                                    width: Fit
                                        body: "App icon"
                                        font_color: #x0B1220
                                        font_size: 15.
                                    }
                                    venue_icon_help := InfoHelp{
                                        width: Fit
                                        tip: "Shown on screens inside the app. The installed app name and icon on this device stay as the platform listing."
                                    }
                                }
                                venue_icon_status := WrapLabel{
                                    body: " "
                                    font_color: #x3A4554
                                    font_size: 12.
                                }
                                ActionStack{
                                    venue_icon_pick := RowSecondaryBtn{text: "Choose icon"}
                                    venue_icon_clear := RowSecondaryBtn{text: "Clear icon"}
                                }
                                venue_brand_save := StackPrimaryBtn{text: "Save brand"}
                                }
                                FormCard{
                                View{
                                    width: Fill height: Fit
                                    flow: Right spacing: 8
                                    align: Align{y: 0.5}
                                    h_print_faces := WrapLabel{ width: Fit
                                                    body: "Print faces (receipt / period bill)" font_color: #x0B1220 font_size: 16.}
                                    print_faces_help := InfoHelp{
                                        width: Fit
                                        tip: "Optional stamp and payment QR next to your logo on booking receipts and period bills."
                                    }
                                }
                                ActionStack{
                                    print_stamp_pick := RowSecondaryBtn{text: "Choose receipt stamp"}
                                    print_stamp_clear := RowSecondaryBtn{text: "Clear stamp"}
                                }
                                ActionStack{
                                    print_qr_pick := RowSecondaryBtn{text: "Choose payment QR"}
                                    print_qr_clear := RowSecondaryBtn{text: "Clear payment QR"}
                                }
                                }
                                FormCard{
                                h_awb_pools := WrapLabel{body: "Carrier AWB number pools" font_color: #x0B1220 font_size: 16.}
                                awb_pools_label := mod.widgets.ReportText{}
                                awb_pool_carrier_input := FieldInput{empty_text: "Carrier party name"}
                                awb_pool_carrier_suggest := mod.widgets.SuggestList{field: "awb_pool_carrier"}
                                FormFieldRow{
                                    awb_pool_start_input := FieldInput{empty_text: "Start no."}
                                    awb_pool_end_input := FieldInput{empty_text: "End no."}
                                }
                                FormFieldRow{
                                    awb_pool_prefix_input := FieldInput{empty_text: "Prefix (optional)"}
                                    awb_pool_label_input := FieldInput{empty_text: "Label (optional)"}
                                }
                                awb_pool_add := StackPrimaryBtn{text: "Add AWB pool"}
                                }
                                FormCard{
                                h_units_heading := WrapLabel{body: "Units" font_color: #x0B1220 font_size: 20.}
                                h_units_help := WrapLabel{
                                    width: Fill
                                    body: ""
                                    font_color: #x3A4554
                                    font_size: 12.
                                    visible: false
                                }
                                units_label := mod.widgets.ReportText{}
                                unit_code_input := FieldInput{empty_text: "Code (bag)"}
                                unit_name_input := FieldInput{empty_text: "Name (Bag)"}
                                unit_add := StackPrimaryBtn{text: "Add unit"}
                                }
                                FormCard{
                                conv_from_input := FieldInput{empty_text: "From code (bag)"}
                                conv_from_suggest := mod.widgets.SuggestList{field: "conv_from"}
                                conv_to_input := FieldInput{empty_text: "To code (kg)"}
                                conv_to_suggest := mod.widgets.SuggestList{field: "conv_to"}
                                conv_factor_input := FieldInput{empty_text: "Factor (25 = 1 bag → 25 kg)"}
                                conv_add := StackPrimaryBtn{text: "Add conversion"}
                                }
                                FormCard{
                                h_booking_measures := WrapLabel{body: "Booking measures" font_color: #x0B1220 font_size: 20.}
                                h_measures_help_moklanjo := WrapLabel{
                                    width: Fill
                                    body: ""
                                    font_color: #x3A4554
                                    font_size: 12.
                                    visible: false
                                }
                                measure_label := mod.widgets.ReportText{}
                                measure_dim_input := FieldInput{empty_text: "Dimension unit code (cm)"}
                                measure_dim_suggest := mod.widgets.SuggestList{field: "measure_dim"}
                                measure_weight_input := FieldInput{empty_text: "Weight unit code (kg)"}
                                measure_weight_suggest := mod.widgets.SuggestList{field: "measure_weight"}
                                measure_divisor_input := FieldInput{empty_text: "Volumetric divisor (5000 domestic)"}
                                measure_slab_input := FieldInput{empty_text: "Chargeable slab in kg (0.5)"}
                                measure_save := StackPrimaryBtn{text: "Save booking measures"}
                                }
                                // Printers belong to this device (any OS): a system
                                // printer by name, or a network receipt printer's address.
                                FormCard{
                                h_printers := WrapLabel{body: "Printers on this device" font_color: #x0B1220 font_size: 20.}
                                h_printers_help := WrapLabel{
                                    width: Fill
                                    body: " "
                                    font_color: #x3A4554
                                    font_size: 12.
                                    visible: false
                                }
                                printers_found_label := WrapLabel{
                                    width: Fill
                                    body: " "
                                    font_color: #x3A4554
                                    font_size: 12.
                                }
                                printers_found_list := mod.widgets.ReportText{}
                                find_printers_btn := RowSecondaryBtn{text: "Find printers"}
                                h_printers_pool := WrapLabel{body: "Printers" font_color: #x0B1220 font_size: 16.}
                                printers_pool_label := mod.widgets.ReportText{}
                                printer_label_input := FieldInput{empty_text: "Name (e.g. Desk)"}
                                printer_target_input := FieldInput{empty_text: "Address or system name"}
                                printer_target_suggest := mod.widgets.SuggestList{field: "printer_target"}
                                ActionStack{
                                    printer_add := RowSecondaryBtn{text: "Add printer"}
                                    printer_remove := RowSecondaryBtn{text: "Remove last"}
                                }
                                h_printers_routes := WrapLabel{body: "Where things print" font_color: #x0B1220 font_size: 16.}
                                printer_routes_label := mod.widgets.ReportText{}
                                h_route_pick := Label{
                                    text: "Print route"
                                    draw_text.color: #x3A4554
                                    draw_text.text_style.font_size: 11.
                                }
                                View{
                                    width: Fill height: Fit
                                    flow: Right spacing: 6
                                    View{width: Fill height: Fit flow: Overlay
                                        route_pick_receipt := SegBtn{text: "Receipt"}
                                        route_pick_receipt_on := SegBtnOn{text: "Receipt" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        route_pick_label := SegBtn{text: "Label"}
                                        route_pick_label_on := SegBtnOn{text: "Label" visible: false}
                                    }
                                }
                                route_add_printer_input := FieldInput{empty_text: "Add printer to this route"}
                                route_add_printer_suggest := mod.widgets.SuggestList{field: "route_add_printer"}
                                ActionStack{
                                    route_add := RowSecondaryBtn{text: "Add to route"}
                                    route_up := RowSecondaryBtn{text: "Up"}
                                    route_down := RowSecondaryBtn{text: "Down"}
                                    route_remove := RowSecondaryBtn{text: "Remove last"}
                                    route_clear := RowSecondaryBtn{text: "Clear printers for this place"}
                                }
                                ActionStack{
                                    printers_save := StackPrimaryBtn{text: "Save printers"}
                                    print_test_btn := RowSecondaryBtn{text: "Print a test page"}
                                }
                                }
                                h_backup := WrapLabel{body: "Backup" font_color: #x0B1220 font_size: 20.}
                                backup_auto_label := WrapLabel{
                                    width: Fill
                                    body: ""
                                    font_color: #x3A4554
                                    font_size: 12.
                                }
                                backup_copy_btn := NavBtn{text: "Also copy backups to a folder…"}
                                db_path_label := WrapLabel{body: "" font_color: #x0B1220}
                                audit_label := mod.widgets.ReportText{}
                                backup_btn := NavBtn{text: "Create backup"}
                                backup_path_label := WrapLabel{body: "" font_color: #x0B1220}
                                h_restore_help := WrapLabel{
                                    width: Fill
                                    body: "Restore: put restore.sqlite3 in the backups folder, then tap Restore."
                                    font_color: #x0B1220
                                }
                                restore_btn := NavBtn{text: "Restore backup"}
                                View{
                                    width: Fill height: Fit
                                    flow: Right spacing: 8
                                    align: Align{y: 0.5}
                                    h_diag := WrapLabel{
                                                    width: Fit
                                                    body: "Diagnostics" font_color: #x0B1220 font_size: 20.}
                                    diag_privacy_help := InfoHelp{
                                        width: Fit
                                        tip: "Logs stay on this device. Secrets (PIN, passwords) are never stored. Sharing a package off the device needs your explicit approval each time — never sold or given to third parties."
                                    }
                                }
                                diag_status_label := WrapLabel{
                                    width: Fill
                                    body: ""
                                    font_color: #x0B1220
                                    font_size: 12.
                                }
                                h_diag_level := WrapLabel{body: "Detail level" font_color: #x0B1220 font_size: 15.}
                                View{
                                    width: Fill height: Fit
                                    flow: Right spacing: 6
                                    View{width: Fill height: Fit flow: Overlay
                                        diag_level_error := SegBtn{text: "Errors"}
                                        diag_level_error_on := SegBtnOn{text: "Errors" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        diag_level_warn := SegBtn{text: "Warnings"}
                                        diag_level_warn_on := SegBtnOn{text: "Warnings" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        diag_level_info := SegBtn{text: "Info"}
                                        diag_level_info_on := SegBtnOn{text: "Info" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        diag_level_debug := SegBtn{text: "Debug"}
                                        diag_level_debug_on := SegBtnOn{text: "Debug" visible: false}
                                    }
                                }
                                ActionStack{
                                    diag_support_start := RowSecondaryBtn{text: "Start 30-min support session"}
                                    diag_support_stop := RowSecondaryBtn{text: "End support session"}
                                }
                                diag_export_local := RowSecondaryBtn{text: "Export local copy"}
                                diag_send_help := InfoHelp{
                                    width: Fit
                                    tip: "Approval is one package, then expires. Temporary — used only to fix or improve the product. Not shared with third parties. Works on Linux, Windows, macOS, and Android."
                                }
                                diag_approve_send := RowPrimaryBtn{text: "Approve share off this device"}
                                diag_export_send := RowSecondaryBtn{text: "Create share package (needs approval)"}
                                diag_clear := NavBtn{text: "Clear diagnostic logs"}
                            }
                        }

                        sidebar_settings_screen := ScrollYView{
                            width: Fill height: Fill
                            visible: false
                            View{
                                width: Fill height: Fit
                                flow: Down spacing: 12
                                padding: Inset{left: 16. right: 16. top: 8. bottom: 120.}
                                // Back is header_back — body Back stays hidden.
                                sidebar_settings_back := QuietFitBtn{text: "Back" visible: false}
                                sidebar_settings_help := InfoHelp{
                                    width: Fit
                                    tip: "Same icons as the side rail. Drag a row to reorder. Flip the switch to show or hide. Settings and Lock always stay on the bar."
                                }
                                View{
                                    width: Fill height: Fit
                                    flow: Right
                                    spacing: 8
                                    sidebar_enable_all := NavBtn{text: "Enable All"}
                                    sidebar_disable_all := NavBtn{text: "Disable All"}
                                }
                                sidebar_nav_list := mod.widgets.SidebarNavList{}
                            }
                        }

                        day_close_screen := ScrollYView{
                            width: Fill height: Fill
                            visible: false
                            View{
                                width: Fill height: Fit
                                flow: Down spacing: 10
                                padding: Inset{left: 16. right: 16. top: 8. bottom: 120.}
                                day_close_help := InfoHelp{
                                    tip: "Recording a day close saves this summary (and drawer variance if counted) to the audit log. It does not stop selling."
                                }
                                day_close_label := mod.widgets.ReportText{}
                                h_drawer_count := WrapLabel{ width: Fit
                                                    body: "Drawer count (optional)" font_color: #x0B1220 font_size: 15.}
                                day_close_counted_input := FieldInput{empty_text: "Counted cash in drawer"}
                                day_close_copy := NavBtn{text: "Copy day close"}
                                day_close_commit := RowPrimaryBtn{text: "Record day close"}
                            }
                        }

                        reports_screen := ScrollYView{
                            width: Fill height: Fill
                            visible: false
                            View{
                                width: Fill height: Fit
                                flow: Down spacing: 10
                                padding: Inset{left: 16. right: 16. top: 8. bottom: 120.}
                                reports_help := InfoHelp{
                                    width: Fit
                                    tip: "Day books, party aging, bank posts, and period P&L — same books for every entity."
                                }
                                reports_books_btns := ActionStack{
                                    reports_today := RowSecondaryBtn{text: "Today books"}
                                    reports_today_on := RowPrimaryBtn{text: "Today books" visible: false}
                                    reports_month := RowSecondaryBtn{text: "This month"}
                                    reports_month_on := RowPrimaryBtn{text: "This month" visible: false}
                                    reports_aging := RowSecondaryBtn{text: "Outstanding / Dr-Cr"}
                                    reports_aging_on := RowPrimaryBtn{text: "Outstanding / Dr-Cr" visible: false}
                                    reports_balance_sheet := RowSecondaryBtn{text: "Balance sheet"}
                                    reports_balance_sheet_on := RowPrimaryBtn{text: "Balance sheet" visible: false}
                                }
                                reports_label := mod.widgets.ReportText{}
                                // Moklanjo ops desk (Ops / COD / Riders peers). Hidden on bank Reports.
                                ops_desk := View{
                                    width: Fill height: Fit
                                    flow: Down spacing: 8
                                    visible: false
                                    ops_desk_help := InfoHelp{
                                        width: Fit
                                        tip: "Scan AWB with a USB barcode scanner (or type it) → Find. Then advance, print label, or collect COD. Labels print Code 128."
                                    }
                                    ops_awb_input := FieldInput{empty_text: "AWB / scan barcode"}
                                    ops_bulk_awb_input := FieldInput{
                                        empty_text: "Bulk AWBs — paste many (newline or comma) for one status advance"
                                    }
                                    ActionStack{
                                        ops_find_awb := RowPrimaryBtn{text: "Find / Scan AWB"}
                                        ops_label := RowSecondaryBtn{text: "Print AWB label"}
                                    }
                                    ops_awb_card := mod.widgets.ReportText{}
                                    ops_note_input := FieldInput{empty_text: "Note / POD (optional)"}
                                    ops_pod_pick := RowSecondaryBtn{text: "Pick POD photo"}
                                    ops_pod_status := WrapLabel{
                                        body: "No POD photo selected"
                                        font_color: #x3A4554
                                        font_size: 12.
                                    }
                                    FormFieldRow{
                                        ops_rider_input := FieldInput{empty_text: "Rider name"}
                                        ops_carrier_input := FieldInput{empty_text: "Carrier party"}
                                    }
                                    ops_rider_suggest := mod.widgets.SuggestList{field: "ops_rider"}
                                    ops_carrier_suggest := mod.widgets.SuggestList{field: "ops_carrier"}
                                    ops_party_input := FieldInput{empty_text: "Consignor / party name"}
                                    ops_party_suggest := mod.widgets.SuggestList{field: "ops_party"}
                                    ActionStack{
                                        ops_advance_ofd := RowPrimaryBtn{text: "Out for delivery"}
                                        ops_advance_delivered := RowSecondaryBtn{text: "Mark delivered"}
                                    }
                                    ActionStack{
                                        ops_ndr := RowPrimaryBtn{text: "Failed delivery"}
                                        ops_reattempt := RowSecondaryBtn{text: "Reattempt"}
                                    }
                                    View{
                                        width: Fill height: Fit
                                        flow: Right{wrap: true} spacing: 6
                                        ops_ndr_door := QuietFitBtn{text: "Door locked"}
                                        ops_ndr_refused := QuietFitBtn{text: "Refused"}
                                        ops_ndr_address := QuietFitBtn{text: "Address wrong"}
                                        ops_ndr_unavailable := QuietFitBtn{text: "Unavailable"}
                                        ops_ndr_other := QuietFitBtn{text: "Other"}
                                    }
                                    ActionStack{
                                        ops_dispatch_fleet := RowPrimaryBtn{text: "Dispatch booked → rider"}
                                        ops_handover_carrier := RowSecondaryBtn{text: "Hand booked → carrier"}
                                    }
                                    ActionStack{
                                        ops_print_run_sheet := RowPrimaryBtn{text: "Print run sheet"}
                                        ops_print_drs := RowSecondaryBtn{text: "Print DRS"}
                                    }
                                    ActionStack{
                                        ops_cod_collect := RowPrimaryBtn{text: "Collect COD (AWB)"}
                                        ops_cod_remit := RowSecondaryBtn{text: "Remit COD (party)"}
                                    }
                                    ActionStack{
                                        ops_add_rider := RowPrimaryBtn{text: "Add rider"}
                                        ops_period_bill := RowSecondaryBtn{text: "Period bill (party)"}
                                    }
                                    FormFieldRow{
                                        ops_rider_phone := FieldInput{empty_text: "Rider phone"}
                                        ops_rider_vehicle := FieldInput{empty_text: "Vehicle"}
                                    }
                                    FormFieldRow{
                                        ops_rider_cod_input := FieldInput{empty_text: "COD handed in"}
                                    }
                                    ActionStack{
                                        ops_settle_rider_cod := RowPrimaryBtn{text: "Settle rider COD"}
                                    }
                                }
                                reports_expense_heading := WrapLabel{body: "Log expense" font_color: #x0B1220 font_size: 15.}
                                expense_amount_input := FieldInput{empty_text: "Amount"}
                                expense_note_input := FieldInput{empty_text: "Note (fuel, office, …)"}
                                h_paid_by_2 := Label{text: "Paid by" draw_text.color: #x3A4554 draw_text.text_style.font_size: 11.}
                                exp_t1_row := View{
                                    width: Fill height: Fit
                                    flow: Right spacing: 6
                                    View{width: Fill height: Fit flow: Overlay
                                        exp_t1_cash := SegBtn{text: "Cash"}
                                        exp_t1_cash_on := SegBtnOn{text: "Cash" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        exp_t1_bank := SegBtn{text: "Bank"}
                                        exp_t1_bank_on := SegBtnOn{text: "Bank" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        exp_t1_upi := SegBtn{text: "UPI"}
                                        exp_t1_upi_on := SegBtnOn{text: "UPI" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        exp_t1_card := SegBtn{text: "Card"}
                                        exp_t1_card_on := SegBtnOn{text: "Card" visible: false}
                                    }
                                }
                                exp_part2 := FieldInput{empty_text: "Part by a second way (optional)"}
                                h_second_way_2 := Label{text: "Second way" draw_text.color: #x3A4554 draw_text.text_style.font_size: 11.}
                                exp_t2_row := View{
                                    width: Fill height: Fit
                                    flow: Right spacing: 6
                                    View{width: Fill height: Fit flow: Overlay
                                        exp_t2_cash := SegBtn{text: "Cash"}
                                        exp_t2_cash_on := SegBtnOn{text: "Cash" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        exp_t2_bank := SegBtn{text: "Bank"}
                                        exp_t2_bank_on := SegBtnOn{text: "Bank" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        exp_t2_upi := SegBtn{text: "UPI"}
                                        exp_t2_upi_on := SegBtnOn{text: "UPI" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        exp_t2_card := SegBtn{text: "Card"}
                                        exp_t2_card_on := SegBtnOn{text: "Card" visible: false}
                                    }
                                }
                                expense_add := RowPrimaryBtn{text: "Add expense"}
                                h_attachments_last_expense := WrapLabel{body: "Attachments (last expense)" font_color: #x0B1220 font_size: 15.}
                                expense_attach_note := FieldInput{empty_text: "Extra note on last expense"}
                                ActionStack{
                                    expense_attach_add_note := RowPrimaryBtn{text: "Add note"}
                                    expense_attach_photo := RowSecondaryBtn{text: "Attach bill photo"}
                                    expense_attach_pdf := RowSecondaryBtn{text: "Attach PDF"}
                                }
                                expense_attach_hint := WrapLabel{
                                    body: "(log an expense first)"
                                    font_color: #x3A4554
                                    font_size: 12.
                                }
                                expense_attach_list := mod.widgets.ReportText{}
                                reports_bank_heading := WrapLabel{body: "Bank / cashbook" font_color: #x0B1220 font_size: 15.}
                                bank_amount_input := FieldInput{empty_text: "Amount"}
                                bank_note_input := FieldInput{empty_text: "Note (deposit, withdrawal, …)"}
                                ActionStack{
                                    bank_till_to_bank := RowPrimaryBtn{text: "Till → bank"}
                                    bank_bank_to_till := RowSecondaryBtn{text: "Bank → till"}
                                    bank_in := RowSecondaryBtn{text: "Bank in"}
                                    bank_out := RowSecondaryBtn{text: "Bank out"}
                                }
                                reports_copy := NavBtn{text: "Copy report"}
                                h_bank_reconciliation := WrapLabel{body: "Bank reconciliation (tick off against your bank statement)" font_color: #x0B1220 font_size: 15.}
                                bank_recon_label := mod.widgets.ReportText{}
                                bank_clear_input := FieldInput{empty_text: "Uncleared # above"}
                                ActionStack{
                                    bank_mark_cleared := RowPrimaryBtn{text: "Mark cleared"}
                                    bank_mark_uncleared := RowSecondaryBtn{text: "Unmark"}
                                }
                                h_spreadsheet_csv := WrapLabel{body: "Spreadsheet (CSV)" font_color: #x0B1220 font_size: 15.}
                                ActionStack{
                                    csv_copy_day := RowPrimaryBtn{text: "Copy day book CSV"}
                                    csv_copy_outstanding := RowSecondaryBtn{text: "Copy outstanding CSV"}
                                }
                                ActionStack{
                                    reports_bill_chase := RowPrimaryBtn{text: "Bill reminder chase list"}
                                    reports_month_mix := RowSecondaryBtn{text: "Month mix report"}
                                }
                                csv_copy_gst_month := NavBtn{text: "Copy month GST CSV"}
                                ActionStack{
                                    csv_copy_parties := RowPrimaryBtn{text: "Copy parties CSV"}
                                    csv_copy_items := RowSecondaryBtn{text: "Copy items CSV"}
                                }
                                ActionStack{
                                    csv_copy_ledger := RowPrimaryBtn{text: "Copy party ledger CSV"}
                                    csv_import_parties := RowSecondaryBtn{text: "Import parties CSV…"}
                                }
                                csv_import_items := NavBtn{text: "Import items / rates CSV…"}
                                ActionStack{
                                    csv_import_pincodes := RowPrimaryBtn{text: "Import pincodes CSV…"}
                                    csv_import_carrier_bill := RowSecondaryBtn{text: "Import carrier bill CSV…"}
                                }
                                csv_copy_carrier_bill := NavBtn{text: "Copy last carrier bill report"}
                            }
                        }

                        inventory_screen := ScrollYView{
                            width: Fill height: Fill
                            visible: false
                            View{
                                width: Fill height: Fit
                                flow: Down spacing: 10
                                padding: Inset{left: 16. right: 16. top: 8. bottom: 120.}
                                inventory_help := InfoHelp{
                                    width: Fit
                                    tip: "Type the item name (part of it is enough), a quantity, then adjust. Items that are not counted yet need Start tracking once."
                                }
                                inventory_label := mod.widgets.ReportText{}
                                inv_item_input := FieldInput{empty_text: "Item name"}
                                inv_item_suggest := mod.widgets.SuggestList{field: "inv_item"}
                                inv_qty_input := FieldInput{empty_text: "Quantity"}
                                inv_note_input := FieldInput{empty_text: "Reason / note (wastage, count, …)"}
                                ActionStack{
                                    inv_adjust_minus := NavBtn{text: "− Stock"}
                                    inv_adjust_plus := NavBtn{text: "+ Stock"}
                                    inv_track := NavBtn{text: "Start tracking"}
                                }
                                h_goods_received_2 := WrapLabel{ width: Fit
                                                    body: "Goods received" font_color: #x0B1220 font_size: 15.}
                                inv_cost_input := FieldInput{empty_text: "Unit cost"}
                                inv_supplier_input := FieldInput{empty_text: "Supplier"}
                                inv_supplier_suggest := mod.widgets.SuggestList{field: "inv_supplier"}
                                inv_purchase := NavBtn{text: "Record purchase"}
                            }
                        }

                        branches_screen := ScrollYView{
                            width: Fill height: Fill
                            visible: false
                            View{
                                width: Fill height: Fit
                                flow: Down spacing: 10
                                padding: Inset{left: 16. right: 16. top: 8. bottom: 120.}
                                branches_help := InfoHelp{
                                    width: Fit
                                    tip: "Export writes a day snapshot into the backups folder and copies it to the clipboard. To import another outlet’s day, save its file as import_branch_snapshot.json in that folder, then tap Import."
                                }
                                org_name_input := FieldInput{empty_text: "Business name"}
                                org_save := NavBtn{text: "Save business name"}
                                branches_label := mod.widgets.ReportText{}
                                // A branch is its own business (vision §5): open another
                                // local server in Server Mode, never switch inside these
                                // books. Hidden; export/import below compares branches.
                                View{
                                    width: Fill height: Fit
                                    flow: Down spacing: 10
                                    visible: false
                                    branch_name_input := FieldInput{empty_text: "New branch name"}
                                    branch_code_input := FieldInput{empty_text: "Short code (MG02)"}
                                    branch_add := NavBtn{text: "Add branch"}
                                    branch_switch_input := FieldInput{empty_text: "Branch name to sell from"}
                                    branch_switch := NavBtn{text: "Switch this device"}
                                }
                                h_branch_export_title := WrapLabel{
                                    body: "Outlet day transfer"
                                    font_color: #x0B1220
                                    font_size: 15.
                                }
                                ActionStack{
                                    branch_export := RowPrimaryBtn{text: "Export day"}
                                    branch_import := RowPrimaryBtn{text: "Import day"}
                                    branch_consolidated := RowSecondaryBtn{text: "Roll-up"}
                                }
                            }
                        }

                    }

                    // Standard snackbar: Overlay faces by kind (info/ok/warn/error).
                    app_snackbar := View{
                        visible: false
                        width: Fill height: Fit
                        flow: Overlay
                        snack_face_info := RoundedView{
                            visible: false
                            width: Fill height: Fit
                            padding: Inset{left: 12. right: 8. top: 10. bottom: 10.}
                            margin: Inset{left: 8. right: 8. bottom: 4.}
                            flow: Right spacing: 8
                            align: Align{y: 0.5}
                            draw_bg.color: #x072058
                            draw_bg.border_radius: 12.
                            snack_label_info := WrapLabel{width: Fill body: "" font_color: #xffffff font_size: 13.}
                            snack_dismiss_info := Button{
                                text: "✕" width: 36 height: 36
                                draw_bg +: { color: #x0A3A8C color_hover: #x0D4A9E color_down: #x02030B color_focus: #x0A3A8C color_2: #x0A3A8C color_2_hover: #x0D4A9E color_2_down: #x02030B color_2_focus: #x0A3A8C border_radius: 10. border_size: 0. }
                                draw_text +: { color: #xffffff color_hover: #xffffff color_down: #xffffff color_focus: #xffffff text_style +: {font_size: 14.} }
                            }
                        }
                        snack_face_ok := RoundedView{
                            visible: false
                            width: Fill height: Fit
                            padding: Inset{left: 12. right: 8. top: 10. bottom: 10.}
                            margin: Inset{left: 8. right: 8. bottom: 4.}
                            flow: Right spacing: 8
                            align: Align{y: 0.5}
                            draw_bg.color: #x2F5D3A
                            draw_bg.border_radius: 12.
                            snack_label_ok := WrapLabel{width: Fill body: "" font_color: #xffffff font_size: 13.}
                            snack_dismiss_ok := Button{
                                text: "✕" width: 36 height: 36
                                draw_bg +: { color: #x1E3D28 color_hover: #x1E3D28 color_down: #x152A1C color_focus: #x1E3D28 color_2: #x1E3D28 color_2_hover: #x1E3D28 color_2_down: #x152A1C color_2_focus: #x1E3D28 border_radius: 10. border_size: 0. }
                                draw_text +: { color: #ffffff color_hover: #xffffff color_down: #ffffff color_focus: #xffffff text_style +: {font_size: 14.} }
                            }
                        }
                        snack_face_warn := RoundedView{
                            visible: false
                            width: Fill height: Fit
                            padding: Inset{left: 12. right: 8. top: 10. bottom: 10.}
                            margin: Inset{left: 8. right: 8. bottom: 4.}
                            flow: Right spacing: 8
                            align: Align{y: 0.5}
                            draw_bg.color: #xC44E10
                            draw_bg.border_radius: 12.
                            snack_label_warn := WrapLabel{width: Fill body: "" font_color: #xffffff font_size: 13.}
                            snack_dismiss_warn := Button{
                                text: "✕" width: 36 height: 36
                                draw_bg +: { color: #x072058 color_hover: #x0A3A8C color_down: #x02030B color_focus: #x072058 color_2: #x072058 color_2_hover: #x0A3A8C color_2_down: #x02030B color_2_focus: #x072058 border_radius: 10. border_size: 0. }
                                draw_text +: { color: #xffffff color_hover: #xffffff color_down: #xffffff color_focus: #xffffff text_style +: {font_size: 14.} }
                            }
                        }
                        snack_face_err := RoundedView{
                            visible: false
                            width: Fill height: Fit
                            padding: Inset{left: 12. right: 8. top: 10. bottom: 10.}
                            margin: Inset{left: 8. right: 8. bottom: 4.}
                            flow: Right spacing: 8
                            align: Align{y: 0.5}
                            draw_bg.color: #x9A0E10
                            draw_bg.border_radius: 12.
                            snack_label_err := WrapLabel{width: Fill body: "" font_color: #xffffff font_size: 13.}
                            snack_dismiss_err := Button{
                                text: "✕" width: 36 height: 36
                                draw_bg +: { color: #xB43A12 color_hover: #xB43A12 color_down: #x69090C color_focus: #xB43A12 color_2: #xB43A12 color_2_hover: #xB43A12 color_2_down: #x69090C color_2_focus: #xB43A12 border_radius: 10. border_size: 0. }
                                draw_text +: { color: #xffffff color_hover: #xffffff color_down: #xffffff color_focus: #xffffff text_style +: {font_size: 14.} }
                            }
                        }
                    }

                    // Post-charge undo snackbar (above tabs; auto-hides after 10s).
                    undo_snackbar := RoundedView{
                        visible: false
                        width: Fill height: Fit
                        padding: Inset{left: 12. right: 8. top: 10. bottom: 10.}
                        margin: Inset{left: 8. right: 8. bottom: 4.}
                        flow: Right spacing: 8
                        align: Align{y: 0.5}
                        draw_bg.color: #x072058
                        draw_bg.border_radius: 12.
                        undo_snack_label := WrapLabel{
                            width: Fill
                            body: "Sale recorded"
                            font_color: #xffffff
                            font_size: 13.
                        }
                        undo_snack_btn := Button{
                            text: "Undo"
                            width: Fit height: 36
                            padding: Inset{left: 14. right: 14.}
                            draw_bg +: {
                                color: #xF7B043
                                color_hover: #xffffff
                                color_down: #xE0E8F0
                                color_focus: #xF7B043
                                color_2: #xF7B043
                                color_2_hover: #xffffff
                                color_2_down: #xE0E8F0
                                color_2_focus: #xF7B043
                                border_radius: 10.
                                border_size: 0.
                            }
                            draw_text +: {
                                color: #x02030B
                                color_hover: #x02030B
                                color_down: #x02030B
                                color_focus: #x02030B
                                text_style +: {font_size: 13.}
                            }
                        }
                        undo_snack_dismiss := Button{
                            text: "✕"
                            width: 36 height: 36
                            draw_bg +: {
                                color: #x0A3A8C
                                color_hover: #x0D4A9E
                                color_down: #x02030B
                                color_focus: #x0A3A8C
                                color_2: #x0A3A8C
                                color_2_hover: #x0D4A9E
                                color_2_down: #x02030B
                                color_2_focus: #x0A3A8C
                                border_radius: 10.
                                border_size: 0.
                            }
                            draw_text +: {
                                color: #xffffff
                                color_hover: #xffffff
                                color_down: #xffffff
                                color_focus: #xffffff
                                text_style +: {font_size: 14.}
                            }
                        }
                    }

                    // Phone chrome: Books · Booking · Pay out + More destinations (scroll).
                    // Selected peer = *Active face visible (paint_nav_tabs), not focus.
                    bottom_nav := SolidView{
                        width: Fill height: Fit
                        padding: Inset{left: 4. right: 4. top: 2. bottom: 6.}
                        draw_bg.color: #x02030B
                        bottom_nav_tabs := View{
                            width: Fill height: Fit
                            flow: Overlay
                            bottom_nav_scroll := ScrollXView{
                                width: Fill height: Fit
                                // Whitespace lanes around the chips (top,
                                // bottom, ends, gaps): a press there belongs
                                // to the scroll view, a press on a chip is a
                                // tap or a drag — see pointer_nav_drag.
                                View{
                                    width: Fit height: Fit
                                    padding: Inset{left: 12. right: 12. top: 12. bottom: 12.}
                                    flow: Right spacing: 6
                                    nav_books_slot := View{
                                        width: Fit{min: FitBound.Abs(72.)} height: Fit
                                        flow: Overlay
                                        nav_books := NavChip{text: "Books"}
                                        nav_books_on := NavChipActive{text: "Books" visible: false}
                                        View{
                                            width: Fit height: 56
                                            align: Align{x: 0.0 y: 0.5}
                                            padding: Inset{left: 10. right: 0. top: 0. bottom: 0.}
                                            nav_books_icon := Image{
                                                width: 22 height: 22
                                                fit: ImageFit.Stretch
                                                src: crate_resource("self:resources/nav/books.webp")
                                            }
                                        }
                                    }
                                    nav_sell_slot := View{
                                        width: Fit{min: FitBound.Abs(72.)} height: Fit
                                        flow: Overlay
                                        nav_sell := NavChip{text: "Booking"}
                                        nav_sell_on := NavChipActive{text: "Booking" visible: false}
                                        View{
                                            width: Fit height: 56
                                            align: Align{x: 0.0 y: 0.5}
                                            padding: Inset{left: 10. right: 0. top: 0. bottom: 0.}
                                            nav_sell_icon := Image{
                                                width: 22 height: 22
                                                fit: ImageFit.Stretch
                                                src: crate_resource("self:resources/nav/sell.webp")
                                            }
                                        }
                                    }
                                    nav_buy_slot := View{
                                        width: Fit{min: FitBound.Abs(72.)} height: Fit
                                        flow: Overlay
                                        nav_buy := NavChip{text: "Pay out"}
                                        nav_buy_on := NavChipActive{text: "Pay out" visible: false}
                                        View{
                                            width: Fit height: 56
                                            align: Align{x: 0.0 y: 0.5}
                                            padding: Inset{left: 10. right: 0. top: 0. bottom: 0.}
                                            nav_buy_icon := Image{
                                                width: 22 height: 22
                                                fit: ImageFit.Stretch
                                                src: crate_resource("self:resources/nav/buy.webp")
                                            }
                                        }
                                    }
                                    nav_rates_slot := View{
                                        width: Fit{min: FitBound.Abs(72.)} height: Fit
                                        flow: Overlay
                                        nav_rates := NavChip{text: "Rates"}
                                        nav_rates_on := NavChipActive{text: "Rates" visible: false}
                                        View{
                                            width: Fit height: 56
                                            align: Align{x: 0.0 y: 0.5}
                                            padding: Inset{left: 10. right: 0. top: 0. bottom: 0.}
                                            nav_rates_icon := Image{
                                                width: 22 height: 22
                                                fit: ImageFit.Stretch
                                                src: crate_resource("self:resources/nav/rates.webp")
                                            }
                                        }
                                    }
                                    nav_ops_slot := View{
                                        width: Fit{min: FitBound.Abs(72.)} height: Fit
                                        flow: Overlay
                                        nav_ops := NavChip{text: "Ops"}
                                        nav_ops_on := NavChipActive{text: "Ops" visible: false}
                                        View{
                                            width: Fit height: 56
                                            align: Align{x: 0.0 y: 0.5}
                                            padding: Inset{left: 10. right: 0. top: 0. bottom: 0.}
                                            nav_ops_icon := Image{
                                                width: 22 height: 22
                                                fit: ImageFit.Stretch
                                                src: crate_resource("self:resources/nav/ops.webp")
                                            }
                                        }
                                    }
                                    nav_cod_slot := View{
                                        width: Fit{min: FitBound.Abs(72.)} height: Fit
                                        flow: Overlay
                                        nav_cod := NavChip{text: "COD"}
                                        nav_cod_on := NavChipActive{text: "COD" visible: false}
                                        View{
                                            width: Fit height: 56
                                            align: Align{x: 0.0 y: 0.5}
                                            padding: Inset{left: 10. right: 0. top: 0. bottom: 0.}
                                            nav_cod_icon := Image{
                                                width: 22 height: 22
                                                fit: ImageFit.Stretch
                                                src: crate_resource("self:resources/nav/cod.webp")
                                            }
                                        }
                                    }
                                    nav_riders_slot := View{
                                        width: Fit{min: FitBound.Abs(72.)} height: Fit
                                        flow: Overlay
                                        nav_riders := NavChip{text: "Riders"}
                                        nav_riders_on := NavChipActive{text: "Riders" visible: false}
                                        View{
                                            width: Fit height: 56
                                            align: Align{x: 0.0 y: 0.5}
                                            padding: Inset{left: 10. right: 0. top: 0. bottom: 0.}
                                            nav_riders_icon := Image{
                                                width: 22 height: 22
                                                fit: ImageFit.Stretch
                                                src: crate_resource("self:resources/nav/riders.webp")
                                            }
                                        }
                                    }
                                    nav_sales_slot := View{
                                        width: Fit{min: FitBound.Abs(72.)} height: Fit
                                        flow: Overlay
                                        nav_sales := NavChip{text: "Sales"}
                                        nav_sales_on := NavChipActive{text: "Sales" visible: false}
                                        View{
                                            width: Fit height: 56
                                            align: Align{x: 0.0 y: 0.5}
                                            padding: Inset{left: 10. right: 0. top: 0. bottom: 0.}
                                            nav_sales_icon := Image{
                                                width: 22 height: 22
                                                fit: ImageFit.Stretch
                                                src: crate_resource("self:resources/nav/sales.webp")
                                            }
                                        }
                                    }
                                    nav_day_close_slot := View{
                                        width: Fit{min: FitBound.Abs(72.)} height: Fit
                                        flow: Overlay
                                        nav_day_close := NavChip{text: "Close"}
                                        nav_day_close_on := NavChipActive{text: "Close" visible: false}
                                        View{
                                            width: Fit height: 56
                                            align: Align{x: 0.0 y: 0.5}
                                            padding: Inset{left: 10. right: 0. top: 0. bottom: 0.}
                                            nav_day_close_icon := Image{
                                                width: 22 height: 22
                                                fit: ImageFit.Stretch
                                                src: crate_resource("self:resources/nav/day_close.webp")
                                            }
                                        }
                                    }
                                    nav_reports_slot := View{
                                        width: Fit{min: FitBound.Abs(72.)} height: Fit
                                        flow: Overlay
                                        nav_reports := NavChip{text: "Reports"}
                                        nav_reports_on := NavChipActive{text: "Reports" visible: false}
                                        View{
                                            width: Fit height: 56
                                            align: Align{x: 0.0 y: 0.5}
                                            padding: Inset{left: 10. right: 0. top: 0. bottom: 0.}
                                            nav_reports_icon := Image{
                                                width: 22 height: 22
                                                fit: ImageFit.Stretch
                                                src: crate_resource("self:resources/nav/reports.webp")
                                            }
                                        }
                                    }
                                    nav_inventory_slot := View{
                                        width: Fit{min: FitBound.Abs(72.)} height: Fit
                                        flow: Overlay
                                        nav_inventory := NavChip{text: "Stock"}
                                        nav_inventory_on := NavChipActive{text: "Stock" visible: false}
                                        View{
                                            width: Fit height: 56
                                            align: Align{x: 0.0 y: 0.5}
                                            padding: Inset{left: 10. right: 0. top: 0. bottom: 0.}
                                            nav_inventory_icon := Image{
                                                width: 22 height: 22
                                                fit: ImageFit.Stretch
                                                src: crate_resource("self:resources/nav/inventory.webp")
                                            }
                                        }
                                    }
                                    nav_branches_slot := View{
                                        width: Fit{min: FitBound.Abs(72.)} height: Fit
                                        flow: Overlay
                                        nav_branches := NavChip{text: "Branches"}
                                        nav_branches_on := NavChipActive{text: "Branches" visible: false}
                                        View{
                                            width: Fit height: 56
                                            align: Align{x: 0.0 y: 0.5}
                                            padding: Inset{left: 10. right: 0. top: 0. bottom: 0.}
                                            nav_branches_icon := Image{
                                                width: 22 height: 22
                                                fit: ImageFit.Stretch
                                                src: crate_resource("self:resources/nav/branches.webp")
                                            }
                                        }
                                    }
                                    nav_credit_slot := View{
                                        width: Fit{min: FitBound.Abs(72.)} height: Fit
                                        flow: Overlay
                                        nav_credit := NavChip{text: "Parties"}
                                        nav_credit_on := NavChipActive{text: "Parties" visible: false}
                                        View{
                                            width: Fit height: 56
                                            align: Align{x: 0.0 y: 0.5}
                                            padding: Inset{left: 10. right: 0. top: 0. bottom: 0.}
                                            nav_credit_icon := Image{
                                                width: 22 height: 22
                                                fit: ImageFit.Stretch
                                                src: crate_resource("self:resources/nav/credit.webp")
                                            }
                                        }
                                    }
                                    nav_team_slot := View{
                                        width: Fit{min: FitBound.Abs(72.)} height: Fit
                                        flow: Overlay
                                        nav_team := NavChip{text: "Team"}
                                        nav_team_on := NavChipActive{text: "Team" visible: false}
                                        View{
                                            width: Fit height: 56
                                            align: Align{x: 0.0 y: 0.5}
                                            padding: Inset{left: 10. right: 0. top: 0. bottom: 0.}
                                            nav_team_icon := Image{
                                                width: 22 height: 22
                                                fit: ImageFit.Stretch
                                                src: crate_resource("self:resources/nav/team.webp")
                                            }
                                        }
                                    }
                                    nav_settings_slot := View{
                                        width: Fit{min: FitBound.Abs(72.)} height: Fit
                                        flow: Overlay
                                        nav_settings := NavChip{text: "Settings"}
                                        nav_settings_on := NavChipActive{text: "Settings" visible: false}
                                        View{
                                            width: Fit height: 56
                                            align: Align{x: 0.0 y: 0.5}
                                            padding: Inset{left: 10. right: 0. top: 0. bottom: 0.}
                                            nav_settings_icon := Image{
                                                width: 22 height: 22
                                                fit: ImageFit.Stretch
                                                src: crate_resource("self:resources/nav/settings.webp")
                                            }
                                        }
                                    }
                                    nav_lock_slot := View{
                                        width: Fit{min: FitBound.Abs(72.)} height: Fit
                                        flow: Overlay
                                        nav_lock := NavChip{text: "Lock"}
                                        nav_lock_on := NavChipActive{text: "Lock" visible: false}
                                        View{
                                            width: Fit height: 56
                                            align: Align{x: 0.0 y: 0.5}
                                            padding: Inset{left: 10. right: 0. top: 0. bottom: 0.}
                                            nav_lock_icon := Image{
                                                width: 22 height: 22
                                                fit: ImageFit.Stretch
                                                src: crate_resource("self:resources/nav/lock.webp")
                                            }
                                        }
                                    }
                                }
                            }
                            // Fill the strip (tabs + padding) so ‹ › sit on the
                            // same vertical centre as the chips — height:56
                            // pinned them to the top of the taller Overlay.
                            View{
                                width: Fill height: Fill
                                align: Align{x: 0.0 y: 0.5}
                                chev_l := ChevronBtn{text: "‹" height: 46 visible: false}
                            }
                            View{
                                width: Fill height: Fill
                                align: Align{x: 1.0 y: 0.5}
                                chev_r := ChevronBtn{text: "›" height: 46 visible: false}
                            }
                        }
                    }
                    } // main_column
                    } // app_shell
                    } // app_hscroll

                    // Floating chip that follows the pointer while a nav item
                    // is dragged to another slot (see pointer_nav_drag). It
                    // lives in a Modal because a Modal paints on the overlay
                    // draw list — above every scroll view. A plain sibling
                    // was covered by the page scroll view's own pass (the
                    // ghost showed over the rail but not over the page, and
                    // not in bottom-bar mode at all). The modal's dim
                    // backdrop is overridden transparent and its content
                    // fills the window, so the ghost's abs_pos is in window
                    // coordinates and nothing else is visible.
                    nav_drag_modal := Modal{
                        can_dismiss: false
                        bg_view +: {
                            draw_bg +: {color: #x00000000}
                        }
                        content +: {
                            width: Fill height: Fill
                            flow: Overlay
                            nav_drag_ghost := RoundedView{
                                visible: false
                                width: Fit height: Fit
                                flow: Right
                                spacing: 8
                                align: Align{x: 0.0 y: 0.5}
                                padding: Inset{left: 12. right: 16. top: 10. bottom: 10.}
                                draw_bg.color: #x072058E6
                                draw_bg.border_radius: 12.
                                draw_bg.border_color: #x8BB8E8
                                draw_bg.border_size: 1.
                                nav_drag_ghost_icon := Image{
                                    width: 22 height: 22
                                    fit: ImageFit.Stretch
                                }
                                nav_drag_ghost_label := Label{
                                    width: Fit height: Fit
                                    text: ""
                                    draw_text.color: #xffffff
                                    draw_text.text_style.font_size: 13.
                                }
                            }
                        }
                    }

                    // Host for every InfoHelp ℹ tip. Listens for TooltipAction
                    // (hover / tap); places callout with edge-aware flip.
                    app_callout := CalloutTooltip{}
                }
            }
        }
    }
}
