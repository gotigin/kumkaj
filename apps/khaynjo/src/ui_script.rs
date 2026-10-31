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
    // Palette from apps/khaynjo/assets/khaynjo-app-icon.png (deep red,
    // maroon, red-orange, gold, warm cream — sampled with ImageMagick).
    // Light face → dark ink (#260B08 / #6B4A3F / #8A6A5C) in EVERY state.
    // Deep-red face (#9A0E10) → white ink in EVERY state.
    // Makepad mixes color→color_hover→color_down; mismatched lightness = wash.
    // Gold (#F69D2C) is the selection/accent band — dark ink stays readable on it.
    mod.theme.color_bg_app = #xF9F4F0
    mod.theme.color_app_caption_bar = #x9A0E10
    mod.theme.color_text = #x260B08
    mod.theme.color_text_hover = #x260B08
    mod.theme.color_text_focus = #x260B08
    mod.theme.color_text_down = #x260B08
    mod.theme.color_text_hl = #x260B08
    mod.theme.color_text_disabled = #x8A6A5C
    mod.theme.color_text_val = #x6B4A3F
    mod.theme.color_text_meta = #x8A6A5C
    mod.theme.color_label_inner = #x260B08
    mod.theme.color_label_inner_hover = #x260B08
    mod.theme.color_label_inner_down = #x260B08
    mod.theme.color_label_inner_focus = #x260B08
    mod.theme.color_label_inner_active = #x260B08
    mod.theme.color_text_placeholder = #x6B4A3F
    mod.theme.color_text_placeholder_hover = #x260B08
    // Caret must be dark on cool/white wells (stock theme uses white = invisible).
    mod.theme.color_text_cursor = #x69090C
    mod.theme.color_cursor = #x69090C
    mod.theme.color_cursor_focus = #x69090C
    mod.theme.color_cursor_empty = #x69090C
    mod.theme.color_cursor_border = #x69090C

    // Stock TextInput caret + placeholder colors are baked from theme at
    // widget-def time (often white on gray wells). Re-bind after our theme so
    // EVERY TextInput shows a visible caret and readable placeholders.
    // Do NOT use set_type_default() here (Android Animator UAF).
    mod.widgets.TextInputFlat = mod.widgets.TextInputFlat{
        blink_speed: 0.5
        draw_cursor +: {
            color: #x69090C
        }
        draw_text +: {
            color: #x260B08
            color_hover: #x260B08
            color_focus: #x260B08
            color_down: #x260B08
            color_empty: #x8A6A5C
            color_empty_hover: #x6B4A3F
            color_empty_focus: #x6B4A3F
        }
    }
    mod.widgets.TextInput = mod.widgets.TextInput{
        blink_speed: 0.5
        draw_cursor +: {
            color: #x69090C
        }
        draw_text +: {
            color: #x260B08
            color_hover: #x260B08
            color_focus: #x260B08
            color_down: #x260B08
            color_empty: #x8A6A5C
            color_empty_hover: #x6B4A3F
            color_empty_focus: #x6B4A3F
        }
    }
    mod.theme.color_icon = #x260B08
    mod.theme.color_icon_active = #x260B08
    mod.theme.color_icon_inactive = #x8A6A5C
    mod.theme.color_icon_disabled = #xD8BBA6
    // Cool inset wells — mid gray, never near-white cream on white cards.
    // Inset = white wells on cool canvas (readable placeholders). Never match color_bg_app.
    mod.theme.color_inset = #xffffff
    mod.theme.color_inset_hover = #xF6E3D3
    mod.theme.color_inset_down = #xEFD2BC
    mod.theme.color_inset_focus = #xffffff
    mod.theme.color_inset_empty = #xffffff
    // Solid light faces — color_2 must match color (never transparent wash).
    mod.theme.color_outset = #xffffff
    mod.theme.color_outset_hover = #xF6E3D3
    mod.theme.color_outset_down = #xEFD2BC
    mod.theme.color_outset_focus = #xffffff
    mod.theme.color_outset_1 = #xffffff
    mod.theme.color_outset_1_hover = #xF6E3D3
    mod.theme.color_outset_1_down = #xEFD2BC
    mod.theme.color_outset_1_focus = #xffffff
    // Same stop as color_outset — transparent color_2 washes light-on-light.
    mod.theme.color_outset_2 = #xffffff
    mod.theme.color_outset_2_hover = #xF6E3D3
    mod.theme.color_outset_2_down = #xEFD2BC
    mod.theme.color_outset_2_focus = #xffffff
    // Selection band: opaque mid-blue on light page (plan: visible, not pale wash).
    // Dark ink (#x260B08) stays readable on this band.
    mod.theme.color_selection = #xF69D2CFF
    mod.theme.color_selection_hover = #xF69D2CFF
    mod.theme.color_selection_down = #xE8890CFF
    mod.theme.color_selection_focus = #xF69D2CFF
    mod.theme.color_selection_empty = #xF69D2CFF
    mod.theme.color_selection_disabled = #xEFD2BC

    // Default Label ink. Keep stock flow — forcing Down drops glyphs on Android GLES.
    mod.widgets.Label = set_type_default() do mod.widgets.Label{
        height: Fit
        draw_text +: {
            color: #x260B08
        }
    }

    // Default Button: light face + dark ink (identical in every pointer state).
    // Set color_2* = color* so the bevel gradient never washes to a second light.
    // Label inset: padding does the face inset; label_walk.margin stays small
    // (padding+16+16 crushed short labels). Never omit margin entirely.
    mod.widgets.Button = set_type_default() do mod.widgets.Button{
        align: Center
        padding: Inset{left: 16. right: 16. top: 10. bottom: 10.}
        label_walk: Walk{
            width: Fit
            height: Fit
            margin: Inset{left: 4. right: 4.}
        }
        icon_walk: Walk{width: 0. height: 0.}
        draw_bg +: {
            color: #xffffff
            color_hover: #x9A0E10
            color_down: #x69090C
            color_focus: #xffffff
            color_disabled: #xEFD2BC
            color_2: #xffffff
            color_2_hover: #x9A0E10
            color_2_down: #x69090C
            color_2_focus: #xffffff
            color_2_disabled: #xEFD2BC
            border_color: #xD8BBA6
            border_color_hover: #x9A0E10
            border_color_down: #x69090C
            border_color_focus: #xD8BBA6
            border_radius: 12.
            border_size: 1.
        }
        draw_text +: {
            color: #x260B08
            color_hover: #xffffff
            color_down: #xffffff
            color_focus: #x260B08
            color_disabled: #x8A6A5C
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
                // Pages feel jumpy with iOS rubber-band; keep fling, drop bounce.
                bounce_at_start: false
                bounce_at_end: false
                bar_size: 10.
                min_handle_size: 40.
                draw_bg +: {
                    size: 6.
                    border_radius: 3.
                    color: #xC9AFA2
                    color_hover: #x9A0E10
                    color_drag: #x69090C
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
        pass.clear_color: #xF9F4F0
        caption_bar +: {
            draw_bg.color: #x9A0E10
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
                        color_down: #xEBD5C2
                        bg_color: #00000000
                        bg_color_hover: #xB43A12
                        bg_color_down: #x69090C
                    }
                }
                max +: {
                    draw_bg +: {
                        color: #xffffff
                        color_hover: #xffffff
                        color_down: #xEBD5C2
                        bg_color: #00000000
                        bg_color_hover: #xB43A12
                        bg_color_down: #x69090C
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
                        color_down: #xEBD5C2
                    }
                }
            }
        }
    }

    // Face language (intent first — same job → same look):
    //   Navigate hub jump → HubDestBtn + mark
    //   Select in place → DeskTab / SegBtn / ChipBtn
    //   Submit / do → one StackPrimary / RowPrimary (or PayBtn on ticket)
    //   Quiet alt → StackSecondary / RowSecondary / NavBtn (outlined);
    //                QuietFitBtn for inline Fit actions (Show PIN, line Cancel)
    //   Destroy → DangerBtn
    // Never put Row*/Stack* (Fill + wrap) at width:Fit — wrap collapses to
    // one letter per line. Use QuietFitBtn instead.
    // Ordinary full-width action — outlined, not shouty.
    let NavBtn = Button{
        width: Fill height: Fit
        margin: Inset{left: 0. right: 0. top: 0. bottom: 0.}
        padding: Inset{left: 14. right: 14. top: 10. bottom: 10.}
        align: Align{x: 0.5 y: 0.5}
        flow: Right{wrap: true}
        label_walk: Walk{
            width: Fit
            height: Fit
            margin: Inset{left: 4. right: 4.}
        }
        icon_walk: Walk{width: 0. height: 0.}
        draw_bg +: {
            color: #xffffff
            color_hover: #xFBEDE4
            color_down: #xEFD2BC
            color_focus: #xffffff
            color_disabled: #xF1ECE8
            color_2: #xffffff
            color_2_hover: #xFBEDE4
            color_2_down: #xEFD2BC
            color_2_focus: #xffffff
            color_2_disabled: #xF1ECE8
            border_color: #xD8BBA6
            border_color_hover: #xB43A12
            border_color_down: #x9A0E10
            border_color_focus: #xD8BBA6
            border_radius: 10.
            border_size: 1.
        }
        draw_text +: {
            color: #x69090C
            color_hover: #x69090C
            color_down: #x69090C
            color_focus: #x69090C
            color_disabled: #xA9806E
            ink_centered: true
            text_style +: {font_size: 14.}
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
            margin: Inset{left: 4. right: 4.}
        }
        icon_walk: Walk{width: 0. height: 0.}
        draw_bg +: {
            color: #x9A0E10
            color_hover: #xB43A12
            color_down: #x69090C
            color_focus: #x9A0E10
            color_2: #x9A0E10
            color_2_hover: #xB43A12
            color_2_down: #x69090C
            color_2_focus: #x9A0E10
            border_color: #x9A0E10
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
        // Left pad clears the colored PNG overlaid at x≈10 (22px mark + gap).
        align: Align{x: 0.0 y: 0.5}
        flow: Right{wrap: true}
        grab_key_focus: false
        reset_hover_on_click: true
        margin: Inset{top: 0. right: 0. bottom: 0. left: 0.}
        padding: Inset{left: 40. right: 12. top: 12. bottom: 12.}
        // Fit label inside wrap:true — Fill freezes the UI (Settings hang).
        label_walk: Walk{
            width: Fit
            height: Fit
            margin: Inset{left: 0. right: 8.}
        }
        icon_walk: Walk{width: 0. height: 0.}
        draw_bg +: {
            color: #x00000000
            color_hover: #x9A0E10
            color_down: #x69090C
            color_focus: #x00000000
            color_disabled: #x00000000
            color_2: #x00000000
            color_2_hover: #x9A0E10
            color_2_down: #x69090C
            color_2_focus: #x00000000
            border_color: #x00000000
            border_radius: 12.
            border_size: 0.
        }
        draw_text +: {
            color: #xF2B183
            color_hover: #xffffff
            color_down: #xffffff
            color_focus: #xF2B183
            color_disabled: #x8A6A5C
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
            color_hover: #x9A0E10
            color_down: #x69090C
            color_focus: #x00000000
            color_disabled: #x00000000
            color_2: #x00000000
            color_2_hover: #x9A0E10
            color_2_down: #x69090C
            color_2_focus: #x00000000
            border_color: #x00000000
            border_radius: 8.
            border_size: 0.
        }
        draw_text +: {
            color: #xF2B183
            color_hover: #xffffff
            color_down: #xffffff
            color_focus: #xF2B183
            color_disabled: #x8A6A5C
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
            color_hover: #x9A0E10
            color_down: #x69090C
            color_focus: #x00000000
            color_disabled: #x00000000
            color_2: #x00000000
            color_2_hover: #x9A0E10
            color_2_down: #x69090C
            color_2_focus: #x00000000
            border_color: #x00000000
            border_radius: 12.
            border_size: 0.
        }
        draw_text +: {
            color: #xF2B183
            color_hover: #xffffff
            color_down: #xffffff
            color_focus: #xF2B183
            color_disabled: #x8A6A5C
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
            color: #xB43A12
            color_hover: #xB43A12
            color_down: #x9A0E10
            color_focus: #xB43A12
            color_disabled: #xEFD2BC
            color_2: #xB43A12
            color_2_hover: #xB43A12
            color_2_down: #x9A0E10
            color_2_focus: #xB43A12
            border_color: #x00000000
            border_radius: 12.
            border_size: 0.
        }
        draw_text +: {
            color: #xffffff
            color_hover: #xffffff
            color_down: #xffffff
            color_focus: #xffffff
            color_disabled: #x8A6A5C
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
        // Fit label inside wrap:true — Fill freezes the UI (Settings hang).
        label_walk: Walk{
            width: Fit
            height: Fit
            margin: Inset{left: 0. right: 8.}
        }
        icon_walk: Walk{width: 0. height: 0.}
        draw_bg +: {
            color: #x9A0E10
            color_hover: #x9A0E10
            color_down: #x9A0E10
            color_focus: #x9A0E10
            color_disabled: #x9A0E10
            color_2: #x9A0E10
            color_2_hover: #x9A0E10
            color_2_down: #x9A0E10
            color_2_focus: #x9A0E10
            color_2_disabled: #x9A0E10
            border_color: #x9A0E10
            border_radius: 12.
            border_size: 0.
        }
        draw_text +: {
            color: #xffffff
            color_hover: #xffffff
            color_down: #xffffff
            color_focus: #xffffff
            color_disabled: #x8A6A5C
            text_style +: {font_size: 13.}
        }
    }

    // Ticket take-payment only — taller filled CTA with warm edge so it is
    // not Amend/Void twin and not a form StackPrimary.
    let PayBtn = Button{
        width: Fill height: Fit
        margin: Inset{left: 0. right: 0. top: 0. bottom: 0.}
        padding: Inset{left: 16. right: 16. top: 16. bottom: 16.}
        align: Align{x: 0.5 y: 0.5}
        flow: Right{wrap: true}
        label_walk: Walk{
            width: Fit
            height: Fit
            margin: Inset{left: 4. right: 4.}
        }
        icon_walk: Walk{width: 0. height: 0.}
        draw_bg +: {
            color: #xB43A12
            color_hover: #x9A0E10
            color_down: #x69090C
            color_focus: #xB43A12
            color_disabled: #xA9806E
            color_2: #xB43A12
            color_2_hover: #x9A0E10
            color_2_down: #x69090C
            color_2_focus: #xB43A12
            color_2_disabled: #xA9806E
            border_color: #xF2B183
            border_color_hover: #xF2B183
            border_color_down: #xEFD2BC
            border_color_focus: #xF2B183
            border_radius: 12.
            border_size: 2.
        }
        draw_text +: {
            color: #xffffff
            color_hover: #xffffff
            color_down: #xffffff
            color_focus: #xffffff
            color_disabled: #xEFD2BC
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
            color: #x9A0E10
            color_hover: #xB43A12
            color_down: #x69090C
            color_focus: #x9A0E10
            color_2: #x9A0E10
            color_2_hover: #xB43A12
            color_2_down: #x69090C
            color_2_focus: #x9A0E10
            border_color: #x9A0E10
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
            color: #x9A0E10
            color_hover: #xB43A12
            color_down: #x69090C
            color_focus: #x9A0E10
            color_2: #x9A0E10
            color_2_hover: #xB43A12
            color_2_down: #x69090C
            color_2_focus: #x9A0E10
            border_color: #x9A0E10
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
        padding: Inset{left: 14. right: 14. top: 10. bottom: 10.}
        align: Align{x: 0.5 y: 0.5}
        flow: Right{wrap: true}
        label_walk: Walk{
            width: Fit
            height: Fit
            margin: Inset{left: 4. right: 4.}
        }
        icon_walk: Walk{width: 0. height: 0.}
        draw_bg +: {
            color: #xffffff
            color_hover: #xFBEDE4
            color_down: #xEFD2BC
            color_focus: #xffffff
            color_disabled: #xF1ECE8
            color_2: #xffffff
            color_2_hover: #xFBEDE4
            color_2_down: #xEFD2BC
            color_2_focus: #xffffff
            color_2_disabled: #xF1ECE8
            border_color: #xD8BBA6
            border_color_hover: #xB43A12
            border_color_down: #x9A0E10
            border_color_focus: #xD8BBA6
            border_radius: 8.
            border_size: 1.
        }
        draw_text +: {
            color: #x69090C
            color_hover: #x69090C
            color_down: #x69090C
            color_focus: #x69090C
            color_disabled: #xA9806E
            ink_centered: true
            text_style +: {font_size: 14.}
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
            color_hover: #xFBEDE4
            color_down: #xEFD2BC
            color_focus: #x00000000
            color_2: #x00000000
            color_2_hover: #xFBEDE4
            color_2_down: #xEFD2BC
            color_2_focus: #x00000000
            border_color: #x00000000
            border_radius: 6.
            border_size: 0.
        }
        draw_text +: {
            color: #x69090C
            color_hover: #x69090C
            color_down: #x69090C
            color_focus: #x69090C
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
            color_hover: #xFBEDE4
            color_down: #xEFD2BC
            color_focus: #x00000000
            color_2: #x00000000
            color_2_hover: #xFBEDE4
            color_2_down: #xEFD2BC
            color_2_focus: #x00000000
            border_color: #x00000000
            border_radius: 8.
            border_size: 0.
        }
        draw_text +: {
            color: #x69090C
            color_hover: #x3D0507
            color_down: #x3D0507
            color_focus: #x69090C
            text_style +: {font_size: 14.}
        }
        text: "TODAY"
    }
    // Screen title + ℹ tip (CalloutTooltip via app_callout). Docs stay off the
    // work surface — tap/hover the icon when the tip is useful.
    mod.widgets.InfoHelpBase = #(InfoHelp::register_widget(vm))
    mod.widgets.InfoHelp = set_type_default() do mod.widgets.InfoHelpBase{
        // Fit so title+ℹ pack as one cluster; do not stretch ℹ to the row end.
        width: Fit
        height: Fit
        flow: Right
        spacing: 6
        align: Align{x: 0.0 y: 0.5}
        tip: ""
        icon := Image{
            width: 28 height: 28
            fit: ImageFit.Stretch
            visible: false
        }
        title := Label{
            width: Fit height: Fit
            text: ""
            draw_text.color: #x260B08
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
                color: #xFBEDE4
                color_hover: #x9A0E10
                color_down: #x69090C
                color_focus: #xFBEDE4
                color_2: #xFBEDE4
                color_2_hover: #x9A0E10
                color_2_down: #x69090C
                color_2_focus: #xFBEDE4
                border_color: #xEFD2BC
                border_radius: 16.
                border_size: 1.
            }
            draw_text +: {
                color: #x69090C
                color_hover: #xffffff
                color_down: #xffffff
                color_focus: #x69090C
                text_style +: {font_size: 15.}
            }
        }
    }
    let InfoHelp = mod.widgets.InfoHelp{}
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
            color: #xffffff
            color_hover: #xFBEDE4
            color_down: #xEFD2BC
            color_focus: #xffffff
            color_disabled: #xF1ECE8
            color_2: #xffffff
            color_2_hover: #xFBEDE4
            color_2_down: #xEFD2BC
            color_2_focus: #xffffff
            color_2_disabled: #xF1ECE8
            border_color: #xD8BBA6
            border_color_hover: #xB43A12
            border_color_down: #x9A0E10
            border_color_focus: #xD8BBA6
            border_radius: 8.
            border_size: 1.
        }
        draw_text +: {
            color: #x69090C
            color_hover: #x69090C
            color_down: #x69090C
            color_focus: #x69090C
            color_disabled: #xA9806E
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
            margin: Inset{left: 4. right: 4.}
        }
        icon_walk: Walk{width: 0. height: 0.}
        draw_bg +: {
            color: #xffffff
            color_hover: #xFBEDE4
            color_down: #xEFD2BC
            color_focus: #xffffff
            color_2: #xffffff
            color_2_hover: #xFBEDE4
            color_2_down: #xEFD2BC
            color_2_focus: #xffffff
            border_color: #x9A0E10
            border_color_hover: #x69090C
            border_color_down: #x4A0608
            border_color_focus: #x9A0E10
            border_radius: 8.
            border_size: 1.5
        }
        draw_text +: {
            color: #x9A0E10
            color_hover: #x69090C
            color_down: #x4A0608
            color_focus: #x9A0E10
            ink_centered: true
            text_style +: {font_size: 15.}
        }
    }
    // Day-book hub destination — left-aligned with room for a colored nav mark.
    let HubDestBtn = Button{
        width: Fill height: Fit
        margin: Inset{left: 0. right: 0. top: 0. bottom: 0.}
        padding: Inset{left: 44. right: 16. top: 12. bottom: 12.}
        align: Align{x: 0.0 y: 0.5}
        flow: Right{wrap: true}
        // Fit label inside wrap:true — Fill freezes the UI.
        label_walk: Walk{
            width: Fit
            height: Fit
            margin: Inset{left: 0. right: 8.}
        }
        icon_walk: Walk{width: 0. height: 0.}
        draw_bg +: {
            color: #xffffff
            color_hover: #xFBEDE4
            color_down: #xEFD2BC
            color_focus: #xffffff
            color_2: #xffffff
            color_2_hover: #xFBEDE4
            color_2_down: #xEFD2BC
            color_2_focus: #xffffff
            border_color: #xD8BBA6
            border_radius: 10.
            border_size: 1.
        }
        draw_text +: {
            color: #x260B08
            color_hover: #x260B08
            color_down: #x260B08
            color_focus: #x260B08
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
    // A section of a form: white card, roomy padding, one job per card.
    let FormCard = RoundedView{
        width: Fill height: Fit
        flow: Down spacing: 14
        padding: Inset{left: 18. right: 18. top: 16. bottom: 18.}
        draw_bg.color: #xffffff
        draw_bg.border_radius: 16.
        draw_bg.border_size: 1.
        draw_bg.border_color: #xE3D3C8
    }
    let FormFieldRow = View{
        width: Fill height: Fit
        flow: Right spacing: 8
        align: Align{y: 0.5}
    }
    // ActionStack faces — full-width stacked CTAs (language source, Server/Client,
    // Add printer, etc.). Never ChipBtn for multi-word labels.
    // Centered + wrap (no ellipsis). Hang avoided by Makepad patch
    // `button_label_center`: wrap faces keep a Fit label walk (Fill label inside
    // wrap:true freezes the UI). Also never put width:Fill children in a
    // parent View with flow Right{wrap:true} — use ActionStack / flow Down.
    // Face padding insets the pill; label_walk.margin stays small (not another
    // 16+16) so long wrap lines like "Send → Kitchen5 · …" keep room to breathe.
    let StackPrimaryBtn = Button{
        width: Fill height: Fit
        margin: Inset{left: 0. right: 0. top: 0. bottom: 0.}
        padding: Inset{left: 16. right: 16. top: 14. bottom: 14.}
        align: Align{x: 0.5 y: 0.5}
        flow: Right{wrap: true}
        label_walk: Walk{
            width: Fit
            height: Fit
            margin: Inset{left: 4. right: 4.}
        }
        icon_walk: Walk{width: 0. height: 0.}
        draw_bg +: {
            color: #x9A0E10
            color_hover: #xB43A12
            color_down: #x69090C
            color_focus: #x9A0E10
            color_2: #x9A0E10
            color_2_hover: #xB43A12
            color_2_down: #x69090C
            color_2_focus: #x9A0E10
            border_color: #x9A0E10
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
        padding: Inset{left: 16. right: 16. top: 14. bottom: 14.}
        align: Align{x: 0.5 y: 0.5}
        flow: Right{wrap: true}
        label_walk: Walk{
            width: Fit
            height: Fit
            margin: Inset{left: 4. right: 4.}
        }
        icon_walk: Walk{width: 0. height: 0.}
        draw_bg +: {
            color: #xEFD2BC
            color_hover: #x9A0E10
            color_down: #x69090C
            color_focus: #xEFD2BC
            color_2: #xEFD2BC
            color_2_hover: #x9A0E10
            color_2_down: #x69090C
            color_2_focus: #xEFD2BC
            border_color: #xD8BBA6
            border_radius: 8.
            border_size: 1.
        }
        draw_text +: {
            color: #x260B08
            color_hover: #xffffff
            color_down: #xffffff
            color_focus: #x260B08
            ink_centered: true
            text_style +: {font_size: 15.}
        }
    }

    // Quiet stacked action (Print bill, Move table, Next order): white face,
    // thin border, smaller type — present, but not competing with the one
    // primary action on the ticket. Label margin stays small — padding already
    // insets the face (double 12+12 crushed long labels on the ticket).
    let StackLightBtn = Button{
        width: Fill height: Fit
        margin: Inset{left: 0. right: 0. top: 0. bottom: 0.}
        padding: Inset{left: 14. right: 14. top: 10. bottom: 10.}
        align: Align{x: 0.5 y: 0.5}
        flow: Right{wrap: true}
        label_walk: Walk{
            width: Fit
            height: Fit
            margin: Inset{left: 2. right: 2.}
        }
        icon_walk: Walk{width: 0. height: 0.}
        draw_bg +: {
            color: #xffffff
            color_hover: #xFBEDE4
            color_down: #xEFD2BC
            color_focus: #xffffff
            color_2: #xffffff
            color_2_hover: #xFBEDE4
            color_2_down: #xEFD2BC
            color_2_focus: #xffffff
            border_color: #xD8BBA6
            border_radius: 10.
            border_size: 1.
        }
        draw_text +: {
            color: #x69090C
            color_hover: #x69090C
            color_down: #x69090C
            color_focus: #x69090C
            ink_centered: true
            text_style +: {font_size: 13.}
        }
    }

    // RADIO SEGMENT — equal Fill share (order type, pay rail…). Stadium
    // outline; selected = filled brand red. Not a page tab (DeskTab) and
    // not a soft filter chip (ChipBtn).
    let SegBtn = Button{
        width: Fill height: Fit
        margin: Inset{left: 0. right: 0. top: 0. bottom: 0.}
        padding: Inset{left: 8. right: 8. top: 10. bottom: 10.}
        align: Align{x: 0.5 y: 0.5}
        flow: Right{wrap: true}
        label_walk: Walk{width: Fit height: Fit}
        icon_walk: Walk{width: 0. height: 0.}
        draw_bg +: {
            color: #xFFFFFF
            color_hover: #xFFF6EE
            color_down: #xFBE3D3
            color_focus: #xFFFFFF
            color_2: #xFFFFFF
            color_2_hover: #xFFF6EE
            color_2_down: #xFBE3D3
            color_2_focus: #xFFFFFF
            border_color: #x9A0E10
            border_radius: 22.
            border_size: 1.5
        }
        draw_text +: {
            color: #x9A0E10
            color_hover: #x9A0E10
            color_down: #x9A0E10
            color_focus: #x9A0E10
            ink_centered: true
            text_style +: {font_size: 13.}
        }
    }
    let SegBtnOn = SegBtn{
        draw_bg +: {
            color: #x9A0E10
            color_hover: #xB43A12
            color_down: #x69090C
            color_focus: #x9A0E10
            color_2: #x9A0E10
            color_2_hover: #xB43A12
            color_2_down: #x69090C
            color_2_focus: #x9A0E10
            border_color: #x9A0E10
            border_radius: 22.
            border_size: 1.5
        }
        draw_text +: {
            color: #xFFFFFF
            color_hover: #xFFFFFF
            color_down: #xFFFFFF
            color_focus: #xFFFFFF
        }
    }

    // PAGE TAB — flat label + underline (Booking Items|Payment). Not a chip.
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
            color: #x8A6A5C
            color_hover: #x9A0E10
            color_down: #x9A0E10
            color_focus: #x8A6A5C
            ink_centered: true
            text_style +: {font_size: 14.}
        }
    }
    let DeskTabOn = DeskTab{
        draw_text +: {
            color: #x9A0E10
            color_hover: #x9A0E10
            color_down: #x69090C
            color_focus: #x9A0E10
            text_style +: {font_size: 14.}
        }
    }

    // The ticket's table: dashed-feel outline in the brand red, left-aligned
    // like a field, so it reads as "this order's table — tap to change".
    let TablePickBtn = Button{
        width: Fill height: 46
        margin: Inset{left: 0. right: 0. top: 0. bottom: 0.}
        padding: Inset{left: 14. right: 14. top: 8. bottom: 8.}
        align: Align{x: 0.0 y: 0.5}
        label_walk: Walk{width: Fill height: Fit}
        icon_walk: Walk{width: 0. height: 0.}
        draw_bg +: {
            color: #xFFF6EE
            color_hover: #xFBE3D3
            color_down: #xEFD2BC
            color_focus: #xFFF6EE
            color_2: #xFFF6EE
            color_2_hover: #xFBE3D3
            color_2_down: #xEFD2BC
            color_2_focus: #xFFF6EE
            border_color: #x9A0E10
            border_radius: 12.
            border_size: 1.5
        }
        draw_text +: {
            color: #x69090C
            color_hover: #x69090C
            color_down: #x69090C
            color_focus: #x69090C
            text_style +: {font_size: 15.}
        }
    }

    // FILTER CHIP — Fit width in a ChipStrip. Soft tag, not DeskTab / SegBtn.
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
            color: #xFFF6EE
            color_hover: #xFBE3D3
            color_down: #xEFD2BC
            color_focus: #xFFF6EE
            color_2: #xFFF6EE
            color_2_hover: #xFBE3D3
            color_2_down: #xEFD2BC
            color_2_focus: #xFFF6EE
            border_color: #x00000000
            border_radius: 16.
            border_size: 0.
        }
        draw_text +: {
            color: #x6B4A3F
            color_hover: #x260B08
            color_down: #x260B08
            color_focus: #x6B4A3F
            ink_centered: true
            max_lines: 1
            text_overflow: TextOverflow.Ellipsis
            text_style +: {font_size: 12.}
        }
    }
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
            color: #x9A0E10
            color_hover: #xB43A12
            color_down: #x69090C
            color_focus: #x9A0E10
            color_2: #x9A0E10
            color_2_hover: #xB43A12
            color_2_down: #x69090C
            color_2_focus: #x9A0E10
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
    // The chevron is drawn, not a "‹ ›" glyph: a glyph sits on the text
    // baseline and read low in the strip (2026-10-25); drawn, it is centred
    // exactly. `draw_bg.point`: 1 = › (default), -1 = ‹.
    let ChevronBtn = Button{
        width: 46 height: Fill
        margin: Inset{left: 0. right: 0. top: 1. bottom: 1.}
        padding: Inset{left: 0. right: 0. top: 0. bottom: 0.}
        text: ""
        label_walk: Walk{width: 0. height: 0.}
        icon_walk: Walk{width: 0. height: 0.}
        draw_bg +: {
            point: uniform(1.0)
            mark_color: uniform(#x69090C)
            color: #xffffff
            color_hover: #xFBEDE4
            color_down: #xEFD2BC
            border_color: #xD8BBA6
            border_radius: 5.
            border_size: 1.
            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                sdf.box(
                    self.border_size,
                    self.border_size,
                    self.rect_size.x - self.border_size * 2.,
                    self.rect_size.y - self.border_size * 2.,
                    self.border_radius
                )
                let fill = self.color
                    .mix(self.color_hover, self.hover)
                    .mix(self.color_down, self.down)
                sdf.fill_keep(fill)
                sdf.stroke(self.border_color, self.border_size)
                let c = self.rect_size * vec2(0.5, 0.5)
                let w = 4.5 * self.point
                let h = 8.5
                sdf.move_to(c.x - w, c.y - h)
                sdf.line_to(c.x + w, c.y)
                sdf.line_to(c.x - w, c.y + h)
                sdf.stroke(self.mark_color, 2.5)
                return sdf.result
            }
        }
    }

    let ChipStrip = ScrollXView{
        width: Fill height: 52
    }

    // Fill chips in a row: padding insets the face; keep label_walk.margin
    // tiny — padding+14+14 crushed "Hold"-length labels into "Ho…".
    let RoleChip = Button{
        width: Fill height: 40
        margin: Inset{left: 0. right: 0. top: 0. bottom: 0.}
        padding: Inset{left: 10. right: 10. top: 8. bottom: 8.}
        align: Center
        label_walk: Walk{
            width: Fit
            height: Fit
            margin: Inset{left: 2. right: 2.}
        }
        icon_walk: Walk{width: 0. height: 0.}
        draw_bg +: {
            color: #xEFD2BC
            color_hover: #x9A0E10
            color_down: #x69090C
            color_focus: #xEFD2BC
            color_2: #xEFD2BC
            color_2_hover: #x9A0E10
            color_2_down: #x69090C
            color_2_focus: #xEFD2BC
            border_color: #xD8BBA6
            border_radius: 10.
            border_size: 1.
        }
        draw_text +: {
            color: #x260B08
            color_hover: #xffffff
            color_down: #xffffff
            color_focus: #x260B08
            ink_centered: true
            max_lines: 1
            text_overflow: TextOverflow.Ellipsis
            text_style +: {font_size: 13.}
        }
    }

    let RoleChipActive = Button{
        width: Fill height: 40
        margin: Inset{left: 0. right: 0. top: 0. bottom: 0.}
        padding: Inset{left: 10. right: 10. top: 8. bottom: 8.}
        align: Center
        label_walk: Walk{
            width: Fit
            height: Fit
            margin: Inset{left: 2. right: 2.}
        }
        icon_walk: Walk{width: 0. height: 0.}
        draw_bg +: {
            color: #x9A0E10
            color_hover: #xB43A12
            color_down: #x69090C
            color_focus: #x9A0E10
            color_2: #x9A0E10
            color_2_hover: #xB43A12
            color_2_down: #x69090C
            color_2_focus: #x9A0E10
            border_color: #x9A0E10
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
    // Every form field: once it holds text, its placeholder's short form
    // stays as a small label in the top edge ("Qty", "Unit cost"), so a
    // filled form still says what each box is — nobody has to remember.
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
        float_caption: true
        draw_caption +: {
            color: #x9A6A5C
            text_style: theme.font_regular{font_size: 10.5}
        }
        draw_bg +: {
            color: #xffffff
            color_hover: #xF6E3D3
            color_focus: #xffffff
            color_down: #xEFD2BC
            color_empty: #xffffff
            border_color: #x5A4238
            border_color_hover: #x9A0E10
            border_color_focus: #x9A0E10
            border_radius: 12.
            border_size: 1.5
        }
        draw_cursor +: {
            color: #x69090C
        }
        // Flat opaque selection — color_2 must match (no light gradient wash).
        draw_selection +: {
            gradient_fill_horizontal: 0.0
            color: #xF69D2CFF
            color_hover: #xF69D2CFF
            color_focus: #xF69D2CFF
            color_down: #xE8890CFF
            color_empty: #xF69D2CFF
            color_2: #xF69D2CFF
            color_2_hover: #xF69D2CFF
            color_2_focus: #xF69D2CFF
            color_2_down: #xE8890CFF
            color_2_empty: #xF69D2CFF
        }
        draw_text +: {
            color: #x260B08
            color_hover: #x260B08
            color_focus: #x260B08
            color_down: #x260B08
            color_empty: #x6B4A3F
            color_empty_hover: #x260B08
            color_empty_focus: #x260B08
            text_style +: {font_size: 16.}
        }
    }

    // Compact cart / totals field — same caret as FieldInput; tighter padding.
    // Ticket fields sit beside their own labels: no floating caption.
    let CartField = FieldInput{
        float_caption: false
        padding: Inset{left: 10. right: 10. top: 12. bottom: 12.}
        draw_bg +: {
            color: #xffffff
            color_hover: #xF6E3D3
            color_focus: #xffffff
            color_down: #xEFD2BC
            color_empty: #xffffff
            border_color: #x5A4238
            border_color_hover: #x69090C
            border_color_focus: #x69090C
            border_radius: 10.
            border_size: 2.
        }
        draw_text +: {
            color: #x260B08
            color_empty: #x8A6A5C
            color_empty_hover: #x6B4A3F
            color_empty_focus: #x6B4A3F
            color_hover: #x260B08
            color_down: #x260B08
            color_focus: #x260B08
            text_style +: {font_size: 15.}
        }
    }

    // Full-width selectable copy (Markdown TextFlow) with real `visible`
    // support — stock Markdown/TextFlow ignore set_visible. Use `body:`.
    // Default: display text is selectable. Only tap targets (cart lines,
    // floor tiles, buttons) keep Label / selectable: false.
    mod.widgets.WrapLabelBase = #(WrapLabel::register_widget(vm))
    mod.widgets.WrapLabel = set_type_default() do mod.widgets.WrapLabelBase{
        width: Fill
        height: Fit
        padding: 0.
        selectable: true
        paragraph_spacing: 0.
        pre_code_spacing: 0.
        font_size: 14.
        font_color: #x260B08
        draw_text +: {
            color: #x260B08
        }
        draw_block +: {
            // Opaque mid-blue (RRGGBBAA) — readable dark ink on top, clear on white page.
            selection_color: #xF69D2CFF
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
        // Fit label inside wrap:true — Fill freezes the UI.
        label_walk: Walk{
            width: Fit
            height: Fit
            margin: Inset{left: 0. right: 8.}
        }
        icon_walk: Walk{width: 0. height: 0.}
        draw_bg +: {
            color: #xffffff
            color_hover: #x9A0E10
            color_down: #x69090C
            color_focus: #xffffff
            color_disabled: #xF5EBE4
            color_2: #xffffff
            color_2_hover: #x9A0E10
            color_2_down: #x69090C
            color_2_focus: #xffffff
            border_color: #xD8BBA6
            border_radius: 12.
            border_size: 1.
        }
        draw_text +: {
            color: #x260B08
            color_hover: #xffffff
            color_down: #xffffff
            color_focus: #x260B08
            color_disabled: #xA08070
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
                            draw_text.color: #xA08070
                            draw_text.text_style.font_size: 12.
                        }
                        // Pill switch: grey track + left thumb (off) /
                        // brand track + right thumb (on).
                        toggle := View{
                            width: 46 height: 28
                            flow: Overlay
                            track_off := RoundedView{
                                width: Fill height: Fill
                                draw_bg.color: #xE0D4C8
                                draw_bg.border_radius: 14.
                            }
                            track_on := RoundedView{
                                width: Fill height: Fill
                                visible: false
                                draw_bg.color: #x9A0E10
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


    // Photo or name-initials avatar. When no photo: navy well + 1–2 letters
    // (see menu_images::name_initials). Size overridden per call site.
    let ThumbAvatar = RoundedView{
        width: 64 height: 64
        margin: Inset{top: 4. bottom: 4. left: 4. right: 4.}
        flow: Overlay
        align: Align{x: 0.5 y: 0.5}
        draw_bg.color: #x9A0E10
        draw_bg.border_radius: 12.
        draw_bg.border_size: 0.
        // Centred both ways: a Fill label keeps its text at the top, so the
        // initials sit in a centring view at their own size.
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

    // − / + on a dish card: white on the deep-red pill in every state
    // (white ink only on deep red; ink never flips while the face does).
    let CardStepBtn = Button{
        width: 34 height: 34
        padding: Inset{left: 0. right: 0. top: 0. bottom: 0.}
        label_walk: Walk{margin: Inset{left: 0. right: 0. top: 0. bottom: 0.}}
        draw_bg +: {
            color: #x9A0E10
            color_hover: #xB43A12
            color_down: #x69090C
            color_focus: #x9A0E10
            color_2: #x9A0E10
            color_2_hover: #xB43A12
            color_2_down: #x69090C
            color_2_focus: #x9A0E10
            border_color: #x9A0E10
            border_color_hover: #xB43A12
            border_color_down: #x69090C
            border_color_focus: #x9A0E10
            border_radius: 17.
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

    // One dish in the menu window. The photo is the whole card (cropped to
    // fill, rounded with the card); a dark wash at the bottom carries the
    // name and, always on the same baseline, the price. No photo: a red
    // card with the dish's initials. A tap anywhere on the card adds one;
    // once the dish is on the order the − n + pill shows at the top and the
    // card wears a gold edge. A sold-out dish is dimmed. Layers are stacked and shown by state,
    // because a DSL colour cannot be restyled at runtime.
    let ProductCard = View{
        width: Fill height: Fit
        tile := View{
            cursor: MouseCursor.Hand
            width: Fill height: 184
            flow: Overlay
            // No photo yet (or none): brand red, the initials on a frosted
            // plate up top (hidden while the − n + sits there).
            RoundedView{
                width: Fill height: Fill
                padding: Inset{top: 16.}
                align: Align{x: 0.5 y: 0.}
                draw_bg.color: #xB43A12
                draw_bg.color_2: #x69090C
                draw_bg.border_radius: 16.
                initials_plate := RoundedView{
                    width: 60 height: 60
                    align: Align{x: 0.5 y: 0.5}
                    draw_bg.color: #xFFFFFF24
                    draw_bg.border_radius: 30.
                    draw_bg.border_size: 1.5
                    draw_bg.border_color: #xFFFFFF66
                    thumb_initials := Label{
                        width: Fit height: Fit
                        text: ""
                        draw_text.color: #xFFFFFF
                        draw_text.text_style.font_size: 20.
                    }
                }
            }
            thumb := Image{
                width: Fill height: Fill
                fit: ImageFit.CropToFill
                visible: false
                draw_bg +: {
                    radius: uniform(16.0)
                    pixel: fn() {
                        let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                        sdf.box(0., 0., self.rect_size.x, self.rect_size.y, self.radius)
                        sdf.fill(self.get_color())
                        return sdf.result
                    }
                }
            }
            // The wash the name and price sit on: clear over the top third,
            // easing into near-black at the foot (eased, so no line shows
            // where it starts).
            View{
                width: Fill height: Fill
                show_bg: true
                draw_bg +: {
                    radius: uniform(16.0)
                    pixel: fn() {
                        let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                        sdf.box(0., 0., self.rect_size.x, self.rect_size.y, self.radius)
                        let t = smoothstep(0.3, 1.0, self.pos.y)
                        sdf.fill(vec4(0.10, 0.02, 0.02, 0.93 * t))
                        return sdf.result
                    }
                }
            }
            // Sold out: the whole card dimmed.
            dim := RoundedView{
                visible: false
                width: Fill height: Fill
                draw_bg.color: #x1A0505A6
                draw_bg.border_radius: 16.
            }
            View{
                width: Fill height: Fill
                flow: Down
                padding: Inset{left: 10. right: 10. top: 8. bottom: 10.}
                View{
                    width: Fill height: Fit
                    align: Align{x: 0.5}
                    stepper := RoundedView{
                        visible: false
                        width: Fit height: Fit
                        padding: Inset{left: 2. right: 2. top: 2. bottom: 2.}
                        flow: Right
                        align: Align{y: 0.5}
                        draw_bg.color: #x9A0E10
                        draw_bg.border_radius: 19.
                        draw_bg.border_size: 1.5
                        draw_bg.border_color: #xF69D2C
                        minus := CardStepBtn{text: "−"}
                        View{
                            width: Fit{min: FitBound.Abs(26.)} height: 34
                            align: Align{x: 0.5 y: 0.5}
                            step_qty := Label{
                                width: Fit height: Fit
                                text: "1"
                                draw_text.color: #xFFFFFF
                                draw_text.text_style.font_size: 15.
                            }
                        }
                        plus := CardStepBtn{text: "+"}
                    }
                }
                View{width: Fill height: Fill}
                name := Label{
                    width: Fill
                    text: "Item"
                    flow: Right{wrap: true}
                    max_lines: 2
                    text_overflow: Ellipsis
                    draw_text.color: #xFFFFFF
                    draw_text.text_style.font_size: 14.
                }
                price := Label{
                    width: Fill
                    margin: Inset{top: 3.}
                    text: "0"
                    flow: Right{wrap: false}
                    max_lines: 1
                    draw_text.color: #xF7B043
                    draw_text.text_style.font_size: 15.
                }
                // In the price's place when sold out; gold is for prices.
                price_off := Label{
                    visible: false
                    width: Fill
                    margin: Inset{top: 3.}
                    text: ""
                    flow: Right{wrap: false}
                    max_lines: 1
                    draw_text.color: #xF3E3DA
                    draw_text.text_style.font_size: 15.
                }
            }
            // The card's edge: a fine line, or gold once it is on the order.
            edge := RoundedView{
                width: Fill height: Fill
                draw_bg.color: #x00000000
                draw_bg.border_radius: 16.
                draw_bg.border_size: 1.
                draw_bg.border_color: #xD8C7BD
            }
            edge_on := RoundedView{
                visible: false
                width: Fill height: Fill
                draw_bg.color: #x00000000
                draw_bg.border_radius: 16.
                draw_bg.border_size: 3.
                draw_bg.border_color: #xF69D2C
            }
        }
    }

    // Floor tab: one card per table — free (cream) or occupied (deep red)
    // One table on the floor grid. The visit state is the colour (one of
    // the stacked backgrounds is shown — Makepad's reliable way to restyle by
    // state, like the idle/active button faces); the whole tile is the tap
    // target; a small action appears only where it helps (Bill / Ready).
    let FloorTile = View{
        cursor: MouseCursor.Hand
        width: Fill height: 132
        flow: Overlay
        bg_free := RoundedView{
            width: Fill height: Fill
            visible: true
            draw_bg.color: #xFFFFFF
            draw_bg.border_radius: 14.
            draw_bg.border_size: 2.
            draw_bg.border_color: #xD8C7BD
        }
        bg_seated := RoundedView{
            width: Fill height: Fill
            visible: false
            draw_bg.color: #xFFF4CC
            draw_bg.border_radius: 14.
            draw_bg.border_size: 2.
            draw_bg.border_color: #xE8C200
        }
        bg_kitchen := RoundedView{
            width: Fill height: Fill
            visible: false
            draw_bg.color: #xFFE9A8
            draw_bg.border_radius: 14.
            draw_bg.border_size: 2.
            draw_bg.border_color: #xD9A400
        }
        bg_ready := RoundedView{
            width: Fill height: Fill
            visible: false
            draw_bg.color: #xFFC48A
            draw_bg.border_radius: 14.
            draw_bg.border_size: 2.
            draw_bg.border_color: #xE06A00
        }
        bg_eating := RoundedView{
            width: Fill height: Fill
            visible: false
            draw_bg.color: #xCFE3FF
            draw_bg.border_radius: 14.
            draw_bg.border_size: 2.
            draw_bg.border_color: #x3B7DD8
        }
        bg_bill := RoundedView{
            width: Fill height: Fill
            visible: false
            draw_bg.color: #xBFEBC8
            draw_bg.border_radius: 14.
            draw_bg.border_size: 2.
            draw_bg.border_color: #x2E9E4F
        }
        bg_paid := RoundedView{
            width: Fill height: Fill
            visible: false
            draw_bg.color: #xE3DCD7
            draw_bg.border_radius: 14.
            draw_bg.border_size: 2.
            draw_bg.border_color: #x8C7B72
        }
        View{
            width: Fill height: Fill
            padding: Inset{left: 12. right: 8. top: 8. bottom: 8.}
            flow: Down spacing: 2
            // Name and the one useful action share the top row, so the
            // action never falls out of the tile.
            View{
                width: Fill height: Fit
                flow: Right spacing: 6
                align: Align{y: 0.5}
                name := Label{
                    width: Fill
                    text: "T1"
                    flow: Right{wrap: false}
                    max_lines: 1
                    draw_text.color: #x1F1A17
                    draw_text.text_style.font_size: 18.
                }
                act := Button{
                    text: "Bill"
                    width: Fit height: 26
                    padding: Inset{left: 12. right: 12. top: 4. bottom: 4.}
                    label_walk: Walk{width: Fit height: Fit margin: Inset{left: 2. right: 2.}}
                    draw_bg +: {
                        color: #xFFFFFF
                        color_hover: #xF4E8E0
                        color_down: #xE8D6CA
                        color_focus: #xFFFFFF
                        color_2: #xFFFFFF
                        color_2_hover: #xF4E8E0
                        color_2_down: #xE8D6CA
                        color_2_focus: #xFFFFFF
                        border_radius: 9.
                        border_size: 1.5
                        border_color: #x5A4238
                        border_color_hover: #x5A4238
                        border_color_down: #x5A4238
                        border_color_focus: #x5A4238
                    }
                    draw_text +: {
                        color: #x1F1A17
                        color_hover: #x1F1A17
                        color_down: #x1F1A17
                        color_focus: #x1F1A17
                        text_style +: {font_size: 12.}
                    }
                }
            }
            // Money first (what the counter scans for), then the state and
            // how long the guests have been here.
            meta := Label{
                width: Fill
                text: ""
                flow: Right{wrap: false}
                max_lines: 1
                draw_text.color: #x1F1A17
                draw_text.text_style.font_size: 15.
            }
            state := Label{
                width: Fill
                text: "Free"
                flow: Right{wrap: false}
                max_lines: 1
                draw_text.color: #x3A2C26
                draw_text.text_style.font_size: 12.
            }
        }
    }

    // Kitchen tab: one card per open KOT — items + state, never prices.
    // Age colour via stacked backgrounds (same idea as FloorTile).
    let KotCard = View{
        width: Fill height: Fit
        flow: Overlay
        bg_ok := RoundedView{
            width: Fill height: Fill
            visible: true
            draw_bg.color: #ffffff
            draw_bg.border_radius: 16.
            draw_bg.border_size: 2.
            draw_bg.border_color: #x5A4238
        }
        bg_amber := RoundedView{
            width: Fill height: Fill
            visible: false
            draw_bg.color: #xFFF4CC
            draw_bg.border_radius: 16.
            draw_bg.border_size: 2.
            draw_bg.border_color: #xD9A400
        }
        bg_red := RoundedView{
            width: Fill height: Fill
            visible: false
            draw_bg.color: #xFFD6D0
            draw_bg.border_radius: 16.
            draw_bg.border_size: 2.
            draw_bg.border_color: #xB43A12
        }
        View{
            width: Fill height: Fit
            padding: Inset{left: 12. right: 12. top: 12. bottom: 12.}
            flow: Down spacing: 6
            title := WrapLabel{
                width: Fill
                body: "Kitchen"
                font_color: #x69090C
                font_size: 14.
            }
            summary := WrapLabel{
                width: Fill
                body: ""
                font_color: #x260B08
                font_size: 15.
            }
            View{
                width: Fill height: Fit
                flow: Right spacing: 10
                align: Align{y: 0.5}
                state := Label{
                    width: Fill
                    text: "New"
                    flow: Right{wrap: false}
                    max_lines: 1
                    draw_text.color: #x9A6206
                    draw_text.text_style.font_size: 13.
                }
                advance := Button{
                    text: "Start"
                    width: Fit height: 44
                    padding: Inset{left: 18. right: 18. top: 10. bottom: 10.}
                    draw_bg +: {
                        color: #xB43A12
                        color_hover: #x822801
                        color_down: #x6f2000
                        color_focus: #xB43A12
                        color_2: #xB43A12
                        color_2_hover: #x822801
                        color_2_down: #x6f2000
                        color_2_focus: #xB43A12
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
            }
        }
    }

    // Items tab: manage catalogue. Photo fills the face; a dark foot band
    // under name/price keeps type readable. Actions float as frosted pills.
    let MenuCard = View{
        cursor: MouseCursor.Hand
        width: Fill height: 140
        flow: Overlay
        RoundedView{
            width: Fill height: Fill
            align: Align{x: 0.5 y: 0.5}
            draw_bg.color: #xB43A12
            draw_bg.color_2: #x69090C
            draw_bg.border_radius: 16.
            thumb_initials := Label{
                width: Fit height: Fit
                text: ""
                draw_text.color: #xFFFFFF
                draw_text.text_style.font_size: 24.
            }
        }
        thumb := Image{
            width: Fill height: Fill
            fit: ImageFit.CropToFill
            visible: false
            draw_bg +: {
                radius: uniform(16.0)
                pixel: fn() {
                    let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                    sdf.box(0., 0., self.rect_size.x, self.rect_size.y, self.radius)
                    sdf.fill(self.get_color())
                    return sdf.result
                }
            }
        }
        // Clear photo above; opaque-enough foot under the text — no side wash.
        View{
            width: Fill height: Fill
            show_bg: true
            draw_bg +: {
                radius: uniform(16.0)
                pixel: fn() {
                    let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                    sdf.box(0., 0., self.rect_size.x, self.rect_size.y, self.radius)
                    let t = smoothstep(0.50, 0.90, self.pos.y)
                    sdf.fill(vec4(0.04, 0.02, 0.01, 0.88 * t))
                    return sdf.result
                }
            }
        }
        View{
            width: Fill height: Fill
            padding: Inset{left: 14. right: 12. top: 10. bottom: 14.}
            flow: Right spacing: 10
            align: Align{y: 1.0}
            View{
                width: Fill height: Fit
                flow: Down spacing: 3
                name := Label{
                    width: Fill
                    text: "Item"
                    flow: Right{wrap: true}
                    max_lines: 2
                    text_overflow: Ellipsis
                    draw_text.color: #xFFFFFF
                    draw_text.text_style.font_size: 16.
                }
                View{
                    width: Fill height: Fit
                    flow: Right spacing: 8
                    align: Align{y: 0.5}
                    price := Label{
                        width: Fit
                        text: "0"
                        flow: Right{wrap: false}
                        max_lines: 1
                        draw_text.color: #xFFD28A
                        draw_text.text_style.font_size: 14.
                    }
                    stock := Label{
                        width: Fill
                        text: " "
                        flow: Right{wrap: false}
                        max_lines: 1
                        text_overflow: Ellipsis
                        draw_text.color: #xF6E8DF
                        draw_text.text_style.font_size: 12.
                    }
                }
            }
            View{
                width: Fit height: Fill
                flow: Down spacing: 8
                align: Align{x: 1.0 y: 0.0}
                remove := Button{
                    text: "✕"
                    width: 34 height: 34
                    align: Align{x: 0.5 y: 0.5}
                    draw_bg +: {
                        // Frosted dark — readable on any photo without a side wash.
                        color: #x1A0A08CC
                        color_hover: #x9A0E10
                        color_down: #x69090C
                        color_focus: #x1A0A08CC
                        color_2: #x1A0A08CC
                        color_2_hover: #x9A0E10
                        color_2_down: #x69090C
                        color_2_focus: #x1A0A08CC
                        border_color: #xFFFFFF88
                        border_radius: 17.
                        border_size: 1.
                    }
                    draw_text +: {
                        color: #xffffff
                        color_hover: #xffffff
                        color_down: #xffffff
                        color_focus: #xffffff
                        text_style +: {font_size: 14.}
                    }
                }
                View{width: Fit height: Fill}
                sold_out := Button{
                    text: "Sold Out"
                    width: Fit height: 28
                    align: Align{x: 0.5 y: 0.5}
                    padding: Inset{left: 12. right: 12. top: 5. bottom: 5.}
                    draw_bg +: {
                        color: #x1A0A08CC
                        color_hover: #xB43A12
                        color_down: #x8A320C
                        color_focus: #x1A0A08CC
                        color_2: #x1A0A08CC
                        color_2_hover: #xB43A12
                        color_2_down: #x8A320C
                        color_2_focus: #x1A0A08CC
                        border_color: #xFFFFFF88
                        border_radius: 14.
                        border_size: 1.
                    }
                    draw_text +: {
                        color: #xFFD28A
                        color_hover: #xffffff
                        color_down: #xffffff
                        color_focus: #xFFD28A
                        text_style +: {font_size: 11.}
                    }
                }
                restore := Button{
                    text: "Restore"
                    width: Fit height: 28
                    align: Align{x: 0.5 y: 0.5}
                    padding: Inset{left: 12. right: 12. top: 5. bottom: 5.}
                    draw_bg +: {
                        color: #x1A0A08CC
                        color_hover: #xB43A12
                        color_down: #x69090C
                        color_focus: #x1A0A08CC
                        color_2: #x1A0A08CC
                        color_2_hover: #xB43A12
                        color_2_down: #x69090C
                        color_2_focus: #x1A0A08CC
                        border_color: #xFFFFFF88
                        border_radius: 14.
                        border_size: 1.
                    }
                    draw_text +: {
                        color: #xffffff
                        color_hover: #xffffff
                        color_down: #xffffff
                        color_focus: #xffffff
                        text_style +: {font_size: 11.}
                    }
                }
            }
        }
        RoundedView{
            width: Fill height: Fill
            draw_bg.color: #x00000000
            draw_bg.border_radius: 16.
            draw_bg.border_size: 1.
            draw_bg.border_color: #xC9B5A8
        }
    }

    // One dish on the ticket: a single compact line — [− qty +] · name ·
    // "2 × ₹250 · Sent" · amount. Tapping the line opens its details (price,
    // applied total, add-ons, note, and Cancel / NC / Move for sent lines),
    // one line at a time, so a ten-dish table still fits on screen.
    // Square ± only — never put a word label in this face (it clips).
    let TicketStepBtn = Button{
        width: 34 height: 34
        padding: Inset{left: 0. right: 0. top: 0. bottom: 0.}
        align: Align{x: 0.5 y: 0.5}
        label_walk: Walk{width: Fit height: Fit}
        icon_walk: Walk{width: 0. height: 0.}
        draw_bg +: {
            color: #xffffff
            color_hover: #x9A0E10
            color_down: #x69090C
            color_focus: #xffffff
            color_2: #xffffff
            color_2_hover: #x9A0E10
            color_2_down: #x69090C
            color_2_focus: #xffffff
            border_color: #xC9B0A2
            border_color_hover: #x9A0E10
            border_color_down: #x69090C
            border_color_focus: #xC9B0A2
            border_radius: 9.
            border_size: 1.
        }
        draw_text +: {
            color: #x69090C
            color_hover: #xffffff
            color_down: #xffffff
            color_focus: #x69090C
            ink_centered: true
            text_style +: {font_size: 17.}
        }
    }
    // Word action on the ticket (Hold / Held): Fit width — never reuse
    // TicketStepBtn's 34×34 square.
    let TicketHoldBtn = Button{
        width: Fit height: 34
        margin: Inset{left: 4. right: 0. top: 0. bottom: 0.}
        padding: Inset{left: 12. right: 12. top: 6. bottom: 6.}
        align: Align{x: 0.5 y: 0.5}
        label_walk: Walk{
            width: Fit
            height: Fit
            margin: Inset{left: 2. right: 2.}
        }
        icon_walk: Walk{width: 0. height: 0.}
        draw_bg +: {
            color: #xffffff
            color_hover: #x9A0E10
            color_down: #x69090C
            color_focus: #xffffff
            color_2: #xffffff
            color_2_hover: #x9A0E10
            color_2_down: #x69090C
            color_2_focus: #xffffff
            border_color: #xC9B0A2
            border_color_hover: #x9A0E10
            border_color_down: #x69090C
            border_color_focus: #xC9B0A2
            border_radius: 9.
            border_size: 1.
        }
        draw_text +: {
            color: #x69090C
            color_hover: #xffffff
            color_down: #xffffff
            color_focus: #x69090C
            ink_centered: true
            max_lines: 1
            text_style +: {font_size: 12.}
        }
    }
    let CartRow = RoundedView{
        width: Fill height: Fit
        padding: Inset{left: 12. right: 10. top: 8. bottom: 8.}
        flow: Down spacing: 8
        draw_bg.color: #xFBF7F4
        draw_bg.border_radius: 10.
        draw_bg.border_size: 1.
        draw_bg.border_color: #xE3D3C8
        // Plain (not selectable) text: a selectable WrapLabel takes the
        // press for itself and the row never sees the tap.
        summary := View{
            cursor: MouseCursor.Hand
            width: Fill height: Fit
            flow: Right spacing: 10
            align: Align{y: 0.5}
            // The dish's photo (or initials), so a busy ticket reads at a glance.
            ThumbAvatar{
                width: 44 height: 44
                margin: Inset{top: 0. bottom: 0. left: 0. right: 0.}
            }
            View{
                width: Fill height: Fit
                flow: Down spacing: 4
                View{
                    width: Fill height: Fit
                    flow: Right spacing: 8
                    align: Align{y: 0.5}
                    label := WrapLabel{
                        selectable: false
                        body: "Item"
                        font_color: #x260B08
                        font_size: 14.
                    }
                    line_amount := Label{
                        width: Fit
                        text: "0"
                        draw_text.color: #x69090C
                        draw_text.text_style.font_size: 14.
                    }
                }
                line_meta := WrapLabel{
                    selectable: false
                    body: " "
                    font_color: #x6B4A3F
                    font_size: 11.
                }
                // Own row so ± and Hold are never crushed by meta text.
                // Only on lines not yet sent: a sent quantity changes only
                // by a reasoned Cancel. Hold skips this line on the next
                // Send KOT (tap again to include).
                stepper := View{
                    width: Fill height: Fit
                    flow: Right spacing: 4
                    align: Align{y: 0.5}
                    minus := TicketStepBtn{text: "−"}
                    // Centred in a fixed box so 1 and 12 sit in the same place.
                    View{
                        width: 36 height: 34
                        align: Align{x: 0.5 y: 0.5}
                        qty_label := Label{
                            width: Fit
                            text: "1"
                            draw_text.color: #x69090C
                            draw_text.text_style.font_size: 15.
                        }
                    }
                    plus := TicketStepBtn{text: "+"}
                    View{width: Fill height: 1}
                    hold_send_btn := TicketHoldBtn{text: "Hold"}
                }
            }
        }
        details := View{
            width: Fill height: Fit
            visible: false
            flow: Down spacing: 8
            padding: Inset{top: 6. left: 2.}
            FormFieldRow{
                h_unit_at_cap := Label{
                    width: 64
                    text: "Unit @"
                    draw_text.color: #x260B08
                    draw_text.text_style.font_size: 13.
                }
                unit_price := CartField{
                    empty_text: "0"
                    text: "0"
                }
            }
            FormFieldRow{
                h_applied := Label{
                    width: 64
                    text: "Applied"
                    draw_text.color: #xB43A12
                    draw_text.text_style.font_size: 13.
                }
                line_total := CartField{
                    empty_text: "0"
                    text: "0"
                }
            }
            mod_row := View{
                width: Fill height: Fit
                flow: Right spacing: 6
                align: Align{y: 0.5}
                mod_prev := RoleChip{text: "‹" width: 36}
                View{width: Fill height: Fit flow: Overlay
                    mod_0 := RoleChip{text: ""}
                    mod_0_on := RoleChipActive{text: "" visible: false}
                }
                View{width: Fill height: Fit flow: Overlay
                    mod_1 := RoleChip{text: ""}
                    mod_1_on := RoleChipActive{text: "" visible: false}
                }
                View{width: Fill height: Fit flow: Overlay
                    mod_2 := RoleChip{text: ""}
                    mod_2_on := RoleChipActive{text: "" visible: false}
                }
                mod_next := RoleChip{text: "›" width: 36}
            }
            note := CartField{
                height: 44
                padding: Inset{left: 10. right: 10. top: 8. bottom: 8.}
                empty_text: "Note (optional)"
                text: ""
                draw_text +: {
                    text_style +: {font_size: 14.}
                }
            }
            // A line the kitchen already has changes only by Cancel
            // (reasoned), NC or Move. QuietFitBtn — never Row* at width:Fit
            // (Fill+wrap collapses to vertical letters).
            sent_row := View{
                width: Fill height: Fit
                flow: Right{wrap: true}
                spacing: 8
                visible: false
                line_cancel := QuietFitBtn{text: "Cancel"}
                line_nc := QuietFitBtn{text: "NC"}
                line_move := QuietFitBtn{text: "Move"}
            }
        }
    }

    // Action on a white/light card — ALWAYS navy face + white ink (never default white Button).
    let CardActionBtn = Button{
        width: Fill height: 48
        draw_bg +: {
            color: #x9A0E10
            color_hover: #xB43A12
            color_down: #x69090C
            color_focus: #x9A0E10
            color_disabled: #xA9806E
            color_2: #x9A0E10
            color_2_hover: #xB43A12
            color_2_down: #x69090C
            color_2_focus: #x9A0E10
            color_2_disabled: #xA9806E
            border_color: #x9A0E10
            border_color_hover: #xB43A12
            border_color_down: #x69090C
            border_color_focus: #x9A0E10
            border_radius: 12.
            border_size: 0.
        }
        draw_text +: {
            color: #xffffff
            color_hover: #xffffff
            color_down: #xffffff
            color_focus: #xffffff
            color_disabled: #xEFD2BC
            text_style +: {font_size: 15.}
        }
    }

    // One bill: amount and how it was paid, then when and who. The whole
    // row picks it (no separate Select button).
    let SaleRow = RoundedView{
        cursor: MouseCursor.Hand
        width: Fill height: Fit
        padding: Inset{left: 16. right: 16. top: 10. bottom: 10.}
        flow: Right spacing: 12
        align: Align{y: 0.5}
        draw_bg.color: #xffffff
        draw_bg.border_radius: 12.
        draw_bg.border_size: 1.
        draw_bg.border_color: #xE3D3C8
        View{
            width: Fill height: Fit
            flow: Down spacing: 2
            sale_main := Label{
                width: Fill
                text: ""
                draw_text.color: #x260B08
                draw_text.text_style.font_size: 15.
            }
            label := Label{
                width: Fill
                text: ""
                draw_text.color: #x8A6A5C
                draw_text.text_style.font_size: 12.
            }
        }
        sale_amount := Label{
            width: Fit
            text: ""
            draw_text.color: #x69090C
            draw_text.text_style.font_size: 16.
        }
    }

    let CustomerRow = RoundedView{
        width: Fill height: Fit
        padding: Inset{left: 16. right: 12. top: 12. bottom: 12.}
        flow: Right spacing: 12
        align: Align{y: 0.5}
        draw_bg.color: #xffffff
        draw_bg.border_radius: 14.
        draw_bg.border_size: 2.
        draw_bg.border_color: #x5A4238
        View{
            width: Fill height: Fit
            flow: Down spacing: 2
            label := WrapLabel{
                body: "customer"
                font_color: #x260B08
                font_size: 14.
            }
            sublabel := WrapLabel{
                body: "id"
                font_color: #x6B4A3F
                font_size: 11.
            }
        }
        pick := Button{
            text: "Pick"
            width: 72 height: 44
            draw_bg +: {
                color: #x9A0E10
                color_hover: #xB43A12
                color_down: #x69090C
                color_focus: #x9A0E10
                color_2: #x9A0E10
                color_2_hover: #xB43A12
                color_2_down: #x69090C
                color_2_focus: #x9A0E10
                border_color: #x9A0E10
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

    // Whole-row language pick (no separate Use button). Padding insets the
    // face; small label_walk.margin — long labels wrap, never crop.
    let LanguageRow = Button{
        width: Fill height: Fit
        align: Align{x: 0.5 y: 0.5}
        flow: Right{wrap: true}
        padding: Inset{left: 16. right: 16. top: 12. bottom: 12.}
        label_walk: Walk{
            width: Fit
            height: Fit
            margin: Inset{left: 4. right: 4.}
        }
        icon_walk: Walk{width: 0. height: 0.}
        enable_long_press: true
        draw_bg +: {
            color: #xffffff
            color_hover: #x9A0E10
            color_down: #x69090C
            color_focus: #xffffff
            color_2: #xffffff
            color_2_hover: #x9A0E10
            color_2_down: #x69090C
            color_2_focus: #xffffff
            border_color: #x5A4238
            border_color_hover: #x9A0E10
            border_color_down: #x6B4A3F
            border_color_focus: #x5A4238
            border_radius: 14.
            border_size: 2.
        }
        draw_text +: {
            color: #x260B08
            color_hover: #xffffff
            color_down: #xffffff
            color_focus: #x260B08
            ink_centered: false
            text_style +: {font_size: 14.}
        }
    }

    let LocaleRow = RoundedView{
        cursor: MouseCursor.Hand
        width: Fill height: Fit
        padding: Inset{left: 16. right: 10. top: 10. bottom: 10.}
        flow: Right spacing: 10
        align: Align{y: 0.5}
        draw_bg.color: #xffffff
        draw_bg.color_hover: #xF0F4FF
        draw_bg.color_down: #xE0E8FF
        draw_bg.border_radius: 14.
        draw_bg.border_size: 2.
        draw_bg.border_color: #x5A4238
        label := WrapLabel{
            width: Fill
            body: "locale"
            font_color: #x260B08
            font_size: 14.
        }
        pick := Button{
            text: "Use"
            width: 80 height: 48
            draw_bg +: {
                color: #x9A0E10
                color_hover: #xB43A12
                color_down: #x69090C
                color_focus: #x9A0E10
                color_2: #x9A0E10
                color_2_hover: #xB43A12
                color_2_down: #x69090C
                color_2_focus: #x9A0E10
                border_color: #x9A0E10
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

    // Avatar row for categories and catalogue dishes: photo when one
    // exists, otherwise the name-initials placeholder (ThumbAvatar).
    let CategoryRow = RoundedView{
        cursor: MouseCursor.Hand
        width: Fill height: Fit
        padding: Inset{left: 10. right: 10. top: 8. bottom: 8.}
        flow: Right spacing: 10
        align: Align{y: 0.5}
        draw_bg.color: #xffffff
        draw_bg.color_hover: #xF0F4FF
        draw_bg.color_down: #xE0E8FF
        draw_bg.border_radius: 14.
        draw_bg.border_size: 2.
        draw_bg.border_color: #x5A4238
        ThumbAvatar{
            width: 48 height: 48
        }
        label := WrapLabel{
            width: Fill
            body: "item"
            font_color: #x260B08
            font_size: 14.
        }
        photo := Button{
            text: "Photo"
            width: Fit height: 40
            padding: Inset{left: 12. right: 12. top: 6. bottom: 6.}
            draw_bg +: {
                color: #xffffff
                color_hover: #x9A0E10
                color_down: #x69090C
                color_focus: #xffffff
                color_2: #xffffff
                color_2_hover: #x9A0E10
                color_2_down: #x69090C
                color_2_focus: #xffffff
                border_radius: 10.
                border_size: 2.
                border_color: #x9A0E10
                border_color_hover: #x9A0E10
                border_color_down: #x69090C
                border_color_focus: #x9A0E10
            }
            draw_text +: {
                color: #x93000a
                color_hover: #xffffff
                color_down: #xffffff
                color_focus: #x93000a
                text_style +: {font_size: 13.}
            }
        }
        pick := Button{
            text: "Use"
            width: 80 height: 48
            draw_bg +: {
                color: #x9A0E10
                color_hover: #xB43A12
                color_down: #x69090C
                color_focus: #x9A0E10
                color_2: #x9A0E10
                color_2_hover: #xB43A12
                color_2_down: #x69090C
                color_2_focus: #x9A0E10
                border_color: #x9A0E10
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


    mod.widgets.ProductListBase = #(ProductList::register_widget(vm))
    // The menu window's dish grid: fills the window under the search and
    // category tabs and scrolls inside it; only rows on screen draw.
    mod.widgets.ProductList = set_type_default() do mod.widgets.ProductListBase{
        width: Fill height: Fit
        list := PortalList{
            width: Fill height: Fill
            spacing: 8
            drag_scrolling: true
            bounce_at_start: false
            bounce_at_end: false
            capture_overload: true
            scroll_bar: ScrollBar{
                bar_size: 10.
                draw_bg +: {
                    size: 6.
                    border_radius: 3.
                    color: #xC9AFA2
                    color_hover: #x9A0E10
                    color_drag: #x69090C
                }
            }
            Item := View{
                width: Fill height: Fit
                flow: Right spacing: 10
                // The gap between rows of cards (PortalList spacing adds
                // none here); a touch wider than across, so rows read apart.
                padding: Inset{bottom: 14.}
                texture_caching: false
                c0 := ProductCard{}
                c1 := ProductCard{}
                c2 := ProductCard{}
                c3 := ProductCard{}
                c4 := ProductCard{}
                c5 := ProductCard{}
            }
        }
    }

    mod.widgets.FloorListBase = #(FloorList::register_widget(vm))
    mod.widgets.FloorList = set_type_default() do mod.widgets.FloorListBase{
        width: Fill height: Fit
        list := PortalList{
            width: Fill height: Fill
            spacing: 10
            drag_scrolling: false
            bounce_at_start: false
            bounce_at_end: false
            capture_overload: false
            scroll_bar: ScrollBar{bar_size: 0.}
            // A row of up to 8 tiles; the draw shows as many as the width
            // allows (`floor_columns`).
            Item := View{
                width: Fill height: Fit
                padding: Inset{bottom: 10.}
                flow: Right spacing: 10
                texture_caching: false
                t0 := FloorTile{}
                t1 := FloorTile{}
                t2 := FloorTile{}
                t3 := FloorTile{}
                t4 := FloorTile{}
                t5 := FloorTile{}
                t6 := FloorTile{}
                t7 := FloorTile{}
            }
            // Area heading (Dine In, Parcel, AC …) above that area's tables.
            Header := View{
                width: Fill height: Fit
                padding: Inset{top: 6. bottom: 2.}
                area_label := WrapLabel{
                    body: ""
                    font_color: #x69090C
                    font_size: 16.
                }
            }
        }
    }

    // Drawn floor: each table at layout_x/y with layout_w/h (Overlay + margin).
    mod.widgets.FloorLayoutListBase = #(FloorLayoutList::register_widget(vm))
    mod.widgets.FloorLayoutList = set_type_default() do mod.widgets.FloorLayoutListBase{
        width: Fill height: Fit
        canvas := View{
            width: Fill height: Fit
            flow: Overlay
            t0 := FloorTile{}
            t1 := FloorTile{}
            t2 := FloorTile{}
            t3 := FloorTile{}
            t4 := FloorTile{}
            t5 := FloorTile{}
            t6 := FloorTile{}
            t7 := FloorTile{}
            t8 := FloorTile{}
            t9 := FloorTile{}
            t10 := FloorTile{}
            t11 := FloorTile{}
            t12 := FloorTile{}
            t13 := FloorTile{}
            t14 := FloorTile{}
            t15 := FloorTile{}
            t16 := FloorTile{}
            t17 := FloorTile{}
            t18 := FloorTile{}
            t19 := FloorTile{}
            t20 := FloorTile{}
            t21 := FloorTile{}
            t22 := FloorTile{}
            t23 := FloorTile{}
        }
    }

    mod.widgets.KitchenListBase = #(KitchenList::register_widget(vm))
    mod.widgets.KitchenList = set_type_default() do mod.widgets.KitchenListBase{
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
                kotcard := KotCard{}
            }
        }
    }

    mod.widgets.MenuListBase = #(MenuList::register_widget(vm))
    // Part of the page (the page scrolls); only rows on screen are drawn —
    // see ProductList for the top_gap / list_box arrangement.
    mod.widgets.MenuList = set_type_default() do mod.widgets.MenuListBase{
        width: Fill height: Fit
        flow: Down
        margin: Inset{top: 8.}
        top_gap := View{width: Fill height: 0}
        list_box := View{
        width: Fill height: 0
        list := PortalList{
            width: Fill height: Fill
            spacing: 12
            drag_scrolling: false
            bounce_at_start: false
            bounce_at_end: false
            capture_overload: false
            scroll_bar: ScrollBar{bar_size: 0.}
            Item := View{
                width: Fill height: Fit
                texture_caching: false
                card := MenuCard{}
            }
        }
        }
    }

    // Items taken off the menu (archived). Same card as MenuList, but the only
    // action is Restore: Remove is an archive, never a delete, so an
    // app-provided dish (or the owner's own item) can always come back exactly
    // as it was — price, photo, category, station.
    mod.widgets.RemovedListBase = #(RemovedList::register_widget(vm))
    mod.widgets.RemovedList = set_type_default() do mod.widgets.RemovedListBase{
        width: Fill height: Fit
        list := PortalList{
            width: Fill height: Fill
            spacing: 12
            drag_scrolling: false
            bounce_at_start: false
            bounce_at_end: false
            capture_overload: false
            scroll_bar: ScrollBar{bar_size: 0.}
            Item := View{
                width: Fill height: Fit
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
                flow: Down spacing: 4
                padding: Inset{bottom: 8.}
                texture_caching: false
                // Group header over the first line of each KOT ("KOT 3 ·
                // 20:40") and over the lines not sent yet.
                kot_head := Label{
                    visible: false
                    width: Fill
                    text: ""
                    padding: Inset{left: 4. top: 4.}
                    draw_text.color: #x8A5A48
                    draw_text.text_style.font_size: 12.
                }
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
                padding: Inset{bottom: 0.}
                texture_caching: false
                row := SaleRow{}
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

    // A booking: who, how many, when, table — Seated / Cancel.
    let ResvRow = RoundedView{
        width: Fill height: Fit
        padding: Inset{left: 16. right: 10. top: 10. bottom: 10.}
        flow: Right spacing: 8
        align: Align{y: 0.5}
        draw_bg.color: #xffffff
        draw_bg.border_radius: 14.
        draw_bg.border_size: 1.5
        draw_bg.border_color: #xD8C7BD
        label := WrapLabel{
            width: Fill
            body: " "
            font_color: #x260B08
            font_size: 14.
        }
        seat := RowSecondaryBtn{text: "Seated" width: 150}
        cancel := RowSecondaryBtn{text: "Cancel" width: 150}
    }

    mod.widgets.ResvListBase = #(ResvList::register_widget(vm))
    mod.widgets.ResvList = set_type_default() do mod.widgets.ResvListBase{
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
                texture_caching: false
                ResvRow{}
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

    // Dish / category pick row: photo fills the face; name + category sit in a
    // solid foot band so type never fights the food photo. Whole row is the
    // tap target (nested Button clicks are unreliable inside PortalList).
    let DishRow = View{
        cursor: MouseCursor.Hand
        width: Fill height: 104
        flow: Overlay
        RoundedView{
            width: Fill height: Fill
            align: Align{x: 0.5 y: 0.5}
            draw_bg.color: #xB43A12
            draw_bg.color_2: #x69090C
            draw_bg.border_radius: 14.
            thumb_initials := Label{
                width: Fit height: Fit
                text: ""
                draw_text.color: #xFFFFFF
                draw_text.text_style.font_size: 22.
            }
        }
        thumb := Image{
            width: Fill height: Fill
            fit: ImageFit.CropToFill
            visible: false
            draw_bg +: {
                radius: uniform(14.0)
                pixel: fn() {
                    let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                    sdf.box(0., 0., self.rect_size.x, self.rect_size.y, self.radius)
                    sdf.fill(self.get_color())
                    return sdf.result
                }
            }
        }
        // Photo stays open above; dark foot only under the text band.
        View{
            width: Fill height: Fill
            show_bg: true
            draw_bg +: {
                radius: uniform(14.0)
                pixel: fn() {
                    let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                    sdf.box(0., 0., self.rect_size.x, self.rect_size.y, self.radius)
                    let t = smoothstep(0.52, 0.92, self.pos.y)
                    sdf.fill(vec4(0.04, 0.02, 0.01, 0.88 * t))
                    return sdf.result
                }
            }
        }
        View{
            width: Fill height: Fill
            padding: Inset{left: 14. right: 12. top: 10. bottom: 12.}
            flow: Right spacing: 12
            align: Align{y: 1.0}
            View{
                width: Fill height: Fit
                flow: Down spacing: 3
                title := Label{
                    width: Fill
                    text: ""
                    flow: Right{wrap: false}
                    max_lines: 1
                    text_overflow: Ellipsis
                    draw_text.color: #xFFFFFF
                    draw_text.text_style.font_size: 16.
                }
                subtitle := Label{
                    width: Fill
                    text: ""
                    flow: Right{wrap: false}
                    max_lines: 1
                    text_overflow: Ellipsis
                    draw_text.color: #xFFD28A
                    draw_text.text_style.font_size: 12.
                }
            }
            // Frosted pill — readable on any photo; not a Button (PortalList).
            RoundedView{
                width: Fit height: Fit
                padding: Inset{left: 14. right: 14. top: 8. bottom: 8.}
                draw_bg.color: #x1A0A08D9
                draw_bg.border_radius: 14.
                draw_bg.border_size: 1.
                draw_bg.border_color: #xFFFFFF99
                action := Label{
                    width: Fit
                    text: "Use"
                    draw_text.color: #xFFD28A
                    draw_text.text_style.font_size: 13.
                }
            }
        }
        RoundedView{
            width: Fill height: Fill
            draw_bg.color: #x00000000
            draw_bg.border_radius: 14.
            draw_bg.border_size: 1.
            draw_bg.border_color: #xC9B5A8
        }
    }

    // The item form's picker: its own scrolling viewport (height set in
    // draw_walk), so only the rows on screen draw — the shelf has ~1,400
    // dishes. A page-height list draws every row, which is how the old shelf
    // decoded a whole folder of photos in one frame and froze the app.
    // The item form's category picker: same rows and viewport as the
    // item picker.
    // One "type or pick" row: the value, a small line that tells similar
    // rows apart, and "Use". Plain Labels — a selectable text would take
    // the tap from the row.
    let SuggestRowView = RoundedView{
        cursor: MouseCursor.Hand
        width: Fill height: Fit
        padding: Inset{left: 14. right: 14. top: 12. bottom: 12.}
        flow: Right spacing: 10
        align: Align{y: 0.5}
        draw_bg.color: #xffffff
        draw_bg.border_radius: 10.
        draw_bg.border_size: 1.
        draw_bg.border_color: #xE3D3C8
        title := Label{
            width: Fill
            text: ""
            max_lines: 1
            text_overflow: Ellipsis
            draw_text.color: #x260B08
            draw_text.text_style.font_size: 15.
        }
        subtitle := Label{
            width: Fit{max: FitBound.Abs(96.)}
            text: ""
            max_lines: 1
            text_overflow: Ellipsis
            draw_text.color: #x6B4A3F
            draw_text.text_style.font_size: 13.
        }
        action := Label{
            width: Fit
            text: "Use"
            draw_text.color: #x9A0E10
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

    // A report on screen (day close, books, …): headings, "label  value"
    // rows with the value at the right, and plain lines — in a white card,
    // never one run-on paragraph. Fed with the same text that prints.
    mod.widgets.ReportTextBase = #(ReportText::register_widget(vm))
    mod.widgets.ReportText = set_type_default() do mod.widgets.ReportTextBase{
        width: Fill height: Fit
        card := RoundedView{
            width: Fill height: Fit
            padding: Inset{left: 16. right: 16. top: 6. bottom: 12.}
            draw_bg.color: #xffffff
            draw_bg.border_radius: 16.
            draw_bg.border_size: 1.
            draw_bg.border_color: #xE3D3C8
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
                    padding: Inset{top: 14. bottom: 4.}
                    head := WrapLabel{
                        body: ""
                        margin: Inset{right: 8.}
                        font_color: #x9A0E10
                        font_size: 14.
                    }
                }
                Kv := View{
                    width: Fill height: Fit
                    row := View{
                        width: Fill height: Fit
                        padding: Inset{top: 5. bottom: 5.}
                        flow: Right spacing: 8
                        align: Align{y: 0.5}
                        k := WrapLabel{
                            width: Fill
                            body: ""
                            font_color: #x6B4A3F
                            font_size: 14.
                        }
                        v := WrapLabel{
                            width: Fit
                            body: ""
                            font_color: #x260B08
                            font_size: 15.
                        }
                        // Per-figure eye for money values (same rules as Books).
                        eye := EyeIconBtn{visible: false}
                    }
                }
                // Right-to-left languages: the label at the right, its
                // value at the left. Text set right-aligned overruns its box
                // by a few px, so the labels keep a margin at the right.
                KvRtl := View{
                    width: Fill height: Fit
                    row := View{
                        width: Fill height: Fit
                        padding: Inset{top: 5. bottom: 5.}
                        flow: Right spacing: 8
                        align: Align{y: 0.5}
                        eye := EyeIconBtn{visible: false}
                        v := WrapLabel{
                            width: Fit
                            body: ""
                            font_color: #x260B08
                            font_size: 15.
                        }
                        k := WrapLabel{
                            width: Fill
                            body: ""
                            margin: Inset{right: 8.}
                            font_color: #x6B4A3F
                            font_size: 14.
                        }
                    }
                }
                Line := View{
                    width: Fill height: Fit
                    padding: Inset{top: 4. bottom: 4.}
                    line := WrapLabel{
                        body: ""
                        margin: Inset{right: 8.}
                        font_color: #x260B08
                        font_size: 14.
                    }
                }
                Sub := View{
                    width: Fill height: Fit
                    padding: Inset{left: 12. top: 3. bottom: 3.}
                    line := WrapLabel{
                        body: ""
                        margin: Inset{right: 8.}
                        font_color: #x3A2C26
                        font_size: 13.
                    }
                }
                Gap := View{
                    width: Fill height: 10
                }
            }
        }
    }

    // One stock item: name and kind, what is on hand, and a Low mark.
    let StockRowView = RoundedView{
        cursor: MouseCursor.Hand
        width: Fill height: Fit
        padding: Inset{left: 14. right: 14. top: 10. bottom: 10.}
        flow: Right spacing: 12
        align: Align{y: 0.5}
        draw_bg.color: #xffffff
        draw_bg.border_radius: 12.
        draw_bg.border_size: 1.
        draw_bg.border_color: #xE3D3C8
        View{
            width: Fill height: Fit
            flow: Down spacing: 2
            stock_name := Label{
                width: Fill
                text: ""
                draw_text.color: #x260B08
                draw_text.text_style.font_size: 15.
            }
            stock_kind := Label{
                width: Fill
                text: ""
                draw_text.color: #x8A6A5C
                draw_text.text_style.font_size: 12.
            }
        }
        stock_low := RoundedView{
            visible: false
            width: Fit height: Fit
            padding: Inset{left: 8. right: 8. top: 3. bottom: 3.}
            draw_bg.color: #xFBE3D3
            draw_bg.border_radius: 8.
            stock_low_text := Label{
                text: "Low"
                draw_text.color: #x9A0E10
                draw_text.text_style.font_size: 12.
            }
        }
        stock_qty := Label{
            width: Fit
            text: ""
            draw_text.color: #x260B08
            draw_text.text_style.font_size: 16.
        }
    }
    mod.widgets.StockListBase = #(StockList::register_widget(vm))
    mod.widgets.StockList = set_type_default() do mod.widgets.StockListBase{
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
                texture_caching: false
                row := StockRowView{}
            }
        }
    }

    // Expense paid in several ways: one row per way (method chips, amount,
    // remove). A blank amount takes what is left.
    mod.widgets.PartsListBase = #(PartsList::register_widget(vm))
    mod.widgets.PartsList = set_type_default() do mod.widgets.PartsListBase{
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
                flow: Right spacing: 8
                align: Align{y: 0.5}
                texture_caching: false
                View{
                    width: Fill height: Fit
                    flow: Right spacing: 4
                    View{width: Fill height: Fit flow: Overlay
                        part_cash := SegBtn{text: "Cash"}
                        part_cash_on := SegBtnOn{text: "Cash" visible: false}
                    }
                    View{width: Fill height: Fit flow: Overlay
                        part_upi := SegBtn{text: "UPI"}
                        part_upi_on := SegBtnOn{text: "UPI" visible: false}
                    }
                    View{width: Fill height: Fit flow: Overlay
                        part_card := SegBtn{text: "Card"}
                        part_card_on := SegBtnOn{text: "Card" visible: false}
                    }
                    View{width: Fill height: Fit flow: Overlay
                        part_bank := SegBtn{text: "Bank"}
                        part_bank_on := SegBtnOn{text: "Bank" visible: false}
                    }
                }
                View{
                    width: 130 height: Fit
                    part_amount := FieldInput{
                        empty_text: "Rest"
                        float_caption: false
                    }
                }
                part_remove := Button{
                    text: "✕"
                    width: 40 height: 40
                    draw_bg +: {
                        color: #xffffff
                        color_hover: #xFBE3D3
                        color_down: #xEFD2BC
                        color_focus: #xffffff
                        border_color: #xD8BBA6
                        border_radius: 10.
                        border_size: 1.
                    }
                    draw_text +: {
                        color: #x69090C
                        color_hover: #x69090C
                        color_down: #x69090C
                        color_focus: #x69090C
                        text_style +: {font_size: 15.}
                    }
                }
            }
        }
    }

    mod.widgets.CategoryPickListBase = #(CategoryPickList::register_widget(vm))
    mod.widgets.CategoryPickList = set_type_default() do mod.widgets.CategoryPickListBase{
        width: Fill height: Fit
        margin: Inset{top: 8. bottom: 4.}
        list := PortalList{
            width: Fill height: Fill
            spacing: 12
            scroll_bar: ScrollBar{}
            Item := View{
                width: Fill height: Fit
                texture_caching: false
                dishrow := DishRow{}
            }
        }
    }

    mod.widgets.DishPickListBase = #(DishPickList::register_widget(vm))
    mod.widgets.DishPickList = set_type_default() do mod.widgets.DishPickListBase{
        width: Fill height: Fit
        margin: Inset{top: 8. bottom: 4.}
        list := PortalList{
            width: Fill height: Fill
            spacing: 12
            scroll_bar: ScrollBar{}
            Item := View{
                width: Fill height: Fit
                texture_caching: false
                dishrow := DishRow{}
            }
        }
    }

    mod.widgets.CategoryListBase = #(CategoryList::register_widget(vm))
    mod.widgets.CategoryList = set_type_default() do mod.widgets.CategoryListBase{
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
                padding: Inset{bottom: 10.}
                texture_caching: false
                CategoryRow{}
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
                pass.clear_color: #xF9F4F0
                caption_bar +: {
                    draw_bg.color: #x9A0E10
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
                    // app_hscroll or it steals half the window height.
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
                        draw_bg.color: #x69090C
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
                                    rail_sell := TabBtn{text: "Order"}
                                    rail_sell_on := TabBtnActive{text: "Order" visible: false}
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
                                    rail_rates := TabBtn{text: "Menu"}
                                    rail_rates_on := TabBtnActive{text: "Menu" visible: false}
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
                                    rail_ops := TabBtn{text: "Floor"}
                                    rail_ops_on := TabBtnActive{text: "Floor" visible: false}
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
                                    rail_cod := TabBtn{text: "Kitchen"}
                                    rail_cod_on := TabBtnActive{text: "Kitchen" visible: false}
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
                                // Team is Settings → Team (not a chrome peer).
                                rail_team_slot := View{
                                    visible: false
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
                        draw_bg.color: #x9A0E10
                    }
                    main_column := View{
                        width: Fill height: Fill
                        flow: Down spacing: 0

                    header := SolidView{
                        width: Fill height: Fit
                        padding: Inset{left: 16. right: 16. top: 12. bottom: 12.}
                        flow: Down spacing: 8
                        draw_bg.color: #x69090C

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
                                    color_hover: #x9A0E10
                                    color_down: #x3D0507
                                    color_focus: #x00000000
                                    color_2: #x00000000
                                    color_2_hover: #x9A0E10
                                    color_2_down: #x3D0507
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
                                // Selectable WrapLabel — font_color stays white on maroon.
                                brand_title := WrapLabel{
                                    body: "Khaynjo"
                                    font_color: #xffffff
                                    font_size: 18.
                                }
                                session_label := WrapLabel{
                                    body: "Sell"
                                    font_color: #xC4A18C
                                    font_size: 11.
                                }
                            }
                            // Screen eye/lock — was the body-top control; lives
                            // here in place of the old "Shop" word. Terracotta
                            // ink so the glyph reads on the maroon header.
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
                        draw_bg.color: #xEFD2BC
                        today_label := WrapLabel{
                            width: Fill
                            body: "Today: —"
                            font_color: #x260B08
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
                                    draw_bg.color: #x9A0E10
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
                                    draw_bg.color: #xF9F4F0
                                    draw_bg.border_radius: 16.

                                    RoundedView{
                                        width: Fill height: Fit
                                        padding: Inset{left: 14. right: 14. top: 14. bottom: 14.}
                                        flow: Down spacing: 8
                                        draw_bg.color: #xffffff
                                        draw_bg.border_radius: 12.
                                        draw_bg.border_size: 1.
                                        draw_bg.border_color: #xEFD2BC
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
                                                    font_color: #x69090C
                                                    font_size: 18.
                                                    selectable: false
                                                }
                                                device_use_question := WrapLabel{
                                                    body: ""
                                                    font_color: #x6B4A3F
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
                                                font_color: #x260B08
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
                                                font_color: #xB43A12
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
                                    draw_bg.color: #x9A0E10
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
                                    draw_bg.color: #xF9F4F0
                                    draw_bg.border_radius: 16.

                                    RoundedView{
                                        width: Fill height: Fit
                                        padding: Inset{left: 14. right: 14. top: 14. bottom: 14.}
                                        flow: Down spacing: 8
                                        draw_bg.color: #xffffff
                                        draw_bg.border_radius: 12.
                                        draw_bg.border_size: 1.
                                        draw_bg.border_color: #xEFD2BC
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
                                                    font_color: #x69090C
                                                    font_size: 18.
                                                }
                                                h_counter_locked := WrapLabel{
                                                    body: "Counter 01 · Locked"
                                                    font_color: #x6B4A3F
                                                    font_size: 12.
                                                }
                                            }
                                            h_shop := Label{
                                                text: "Shop"
                                                draw_text.color: #xB43A12
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
                                            chev_l := ChevronBtn{draw_bg.point: -1.0 visible: false}
                                        }
                                        View{
                                            width: Fill height: 52
                                            align: Align{x: 1.0 y: 0.5}
                                            chev_r := ChevronBtn{visible: false}
                                        }
                                    }

                                    lock_staff_row := View{
                                        width: Fill height: Fit
                                        flow: Down spacing: 8
                                        visible: false
                                        h_who_unlocking := WrapLabel{
                                            body: "Who is unlocking?"
                                            font_color: #x260B08
                                            font_size: 11.
                                        }
                                        lock_staff_label := WrapLabel{
                                            width: Fit
                                            body: "No staff yet — unlock as Owner, then Team"
                                            font_color: #x260B08
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
                                        draw_bg.border_color: #xEFD2BC
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
                                        pin_ok := StackPrimaryBtn{text: "Unlock"}
                                        pin_status := WrapLabel{
                                            width: Fit
                                            body: " "
                                            font_color: #xB43A12
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
                                    lock_login_ok := StackPrimaryBtn{text: "Unlock with login"}
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
                                h_setup_app_khaynjo := WrapLabel{
                                    body: "Set up Khaynjo"
                                    font_color: #x260B08
                                    font_size: 22.
                                }
                                onboard_status := WrapLabel{
                                    body: "Name this counter if you want, then Start fresh or Bring my book."
                                    font_color: #x6B4A3F
                                    font_size: 13.
                                }
                                onboard_venue_input := FieldInput{empty_text: "Restaurant name (optional)"}
                                onboard_org_input := FieldInput{empty_text: "Organization (optional)"}
                                onboard_save_venue := StackPrimaryBtn{text: "Save names"}
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
                                                font_color: #x260B08
                                                font_size: 15.
                                            }
                                            onboard_icon_help := InfoHelp{
                                                width: Fit
                                                tip: "Optional shop mark on screens in the app."
                                            }
                                        }
                                        onboard_icon_status := WrapLabel{
                                            body: " "
                                            font_color: #x6B4A3F
                                            font_size: 12.
                                        }
                                    }
                                }
                                onboard_icon_pick := NavBtn{text: "Choose logo"}
                                h_product_locked_khaynjo := WrapLabel{
                                    body: "Khaynjo"
                                    font_color: #x260B08
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
                                        font_color: #x260B08
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
                                    font_color: #x260B08
                                    font_size: 15.
                                    selectable: false
                                }
                                View{
                                    width: Fill height: 52
                                    flow: Overlay
                                    onboard_fresh := RowSecondaryBtn{text: "Start fresh"}
                                    onboard_fresh_on := RowPrimaryBtn{text: "Start fresh" visible: false}
                                }
                                // Every dish and category we ship, with photos —
                                // imported when setup is done; prices set after.
                                View{
                                    width: Fill height: 52
                                    flow: Overlay
                                    onboard_template := RowSecondaryBtn{text: "Start with our menu template"}
                                    onboard_template_on := RowPrimaryBtn{text: "Start with our menu template" visible: false}
                                }
                                View{
                                    width: Fill height: 52
                                    flow: Overlay
                                    onboard_book := RowSecondaryBtn{text: "Bring my book"}
                                    onboard_book_on := RowPrimaryBtn{text: "Bring my book" visible: false}
                                }
                                h_fresh_book_khaynjo := WrapLabel{
                                    body: ""
                                    font_color: #x6B4A3F
                                    font_size: 12.
                                    visible: false
                                }
                                onboard_fresh_panel := View{
                                    width: Fill height: Fit
                                    flow: Down spacing: 8
                                    visible: false
                                    h_fresh_selected := WrapLabel{
                                        body: "Fresh selected — catalogue stays empty. Tap Done when ready."
                                        font_color: #x6B4A3F
                                        font_size: 12.
                                    }
                                }
                                onboard_book_panel := View{
                                    width: Fill height: Fit
                                    flow: Down spacing: 8
                                    visible: false
                                    h_book_add_dish := WrapLabel{body: "Book — add dish / item" font_color: #x260B08 font_size: 15.}
                                    onboard_item_name := FieldInput{empty_text: "Item name"}
                                    onboard_item_price := FieldInput{empty_text: "Price"}
                                    onboard_add_item := NavBtn{text: "Add item"}
                                    h_book_opening_credit := WrapLabel{body: "Book — opening credit" font_color: #x260B08 font_size: 15.}
                                    onboard_cust_name := FieldInput{empty_text: "Customer name"}
                                    onboard_cust_bal := FieldInput{empty_text: "Amount owed"}
                                    onboard_add_credit := NavBtn{text: "Add opening balance"}
                                }
                                onboard_done := PinOkKey{text: "Done — open books"}
                                h_later_country_pack := WrapLabel{
                                    body: ""
                                    font_color: #x6B4A3F
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
                                // Back is header_back (left of brand) — not a body button.
                                client_pair_back := BackBtn{text: "Back" visible: false}
                                View{
                                    width: Fill height: Fit
                                    flow: Right spacing: 8
                                    align: Align{y: 0.5}
                                    h_connect_to_server := WrapLabel{
                                        width: Fit
                                        body: "Connect to the Server"
                                        font_color: #x260B08
                                        font_size: 22.
                                    }
                                    client_pair_help := InfoHelp{
                                        width: Fit
                                        tip: "Find the shop's server on Wi‑Fi, or type its address and pairing code."
                                    }
                                }
                                client_pair_status := WrapLabel{
                                    body: " "
                                    font_color: #x6B4A3F
                                    font_size: 13.
                                    selectable: false
                                }
                                // No typing IPs: the shop's server answers on this Wi‑Fi.
                                client_find_btn := StackPrimaryBtn{text: "Find the shop's server"}
                                client_found_label := WrapLabel{
                                    body: " "
                                    font_color: #x6B4A3F
                                    font_size: 13.
                                }
                                client_found_list := mod.widgets.ReportText{}
                                client_host_input := FieldInput{empty_text: "Server IP (e.g. 192.168.1.10)"}
                                client_code_input := FieldInput{empty_text: "Pairing code"}
                                // How the shop's server lists this device (Forget picks it by this).
                                client_name_input := FieldInput{empty_text: "This device's name"}
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
                                    tip: "Today’s position. Order = orders in. Buy = purchases & expenses. Parties = credit customers & suppliers."
                                }
                                books_low_stock := WrapLabel{body: " " font_color: #x9A0E10 font_size: 14. visible: false}
                                books_backup_warning := WrapLabel{body: " " font_color: #x9A0E10 font_size: 14. visible: false}
                                // Hub jumps only — every row navigates, so every
                                // row is HubDestBtn + mark (never a filled primary).
                                ActionStack{
                                    books_to_sell_slot := View{
                                        width: Fill height: Fit
                                        flow: Overlay
                                        books_to_sell := HubDestBtn{text: "Sell (money in)"}
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
                                        books_to_buy := HubDestBtn{text: "Buy (money out)"}
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
                                        books_reports := HubDestBtn{text: "Reports & bank"}
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
                                        books_credit := HubDestBtn{text: "Parties / credit"}
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
// Accordion sections — only TODAY open by default.
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
                                        h_caption_today := Label{text: "TODAY" visible: false draw_text.color: #x93705F draw_text.text_style.font_size: 10.}
                                        View{width: Fill height: Fit flow: Right spacing: 8 align: Align{y: 0.5}
                                            h_money_in_sales := Label{width: Fill height: Fit text: "Money in (sales)" draw_text.color: #x6B4A3F draw_text.text_style.font_size: 12.}
                                            View{width: Fit height: Fit flow: Right spacing: 4 align: Align{y: 0.5}
                                            books_in_value := Label{width: Fit height: Fit text: "—" draw_text.color: #x260B08 draw_text.text_style.font_size: 13.}
                                            books_in_eye := EyeIconBtn{}
                                        }
                                        }
                                        View{width: Fill height: Fit flow: Right spacing: 8 align: Align{y: 0.5}
                                            h_purchases_cogs := Label{width: Fill height: Fit text: "Purchases (COGS)" draw_text.color: #x6B4A3F draw_text.text_style.font_size: 12.}
                                            View{width: Fit height: Fit flow: Right spacing: 4 align: Align{y: 0.5}
                                            books_purchases_value := Label{width: Fit height: Fit text: "—" draw_text.color: #x260B08 draw_text.text_style.font_size: 13.}
                                            books_purchases_eye := EyeIconBtn{}
                                        }
                                        }
                                        View{width: Fill height: Fit flow: Right spacing: 8 align: Align{y: 0.5}
                                            h_expenses_word := Label{width: Fill height: Fit text: "Expenses" draw_text.color: #x6B4A3F draw_text.text_style.font_size: 12.}
                                            View{width: Fit height: Fit flow: Right spacing: 4 align: Align{y: 0.5}
                                            books_expenses_value := Label{width: Fit height: Fit text: "—" draw_text.color: #x260B08 draw_text.text_style.font_size: 13.}
                                            books_expenses_eye := EyeIconBtn{}
                                        }
                                        }
                                        View{width: Fill height: Fit flow: Right spacing: 8 align: Align{y: 0.5}
                                            h_gross_word := Label{width: Fill height: Fit text: "Gross" draw_text.color: #x6B4A3F draw_text.text_style.font_size: 12.}
                                            View{width: Fit height: Fit flow: Right spacing: 4 align: Align{y: 0.5}
                                            books_gross_value := Label{width: Fit height: Fit text: "—" draw_text.color: #x260B08 draw_text.text_style.font_size: 13.}
                                            books_gross_eye := EyeIconBtn{}
                                        }
                                        }
                                        View{width: Fill height: Fit flow: Right spacing: 8 align: Align{y: 0.5}
                                            h_net_word := Label{width: Fill height: Fit text: "Net" draw_text.color: #x9A0E10 draw_text.text_style.font_size: 13.}
                                            View{width: Fit height: Fit flow: Right spacing: 4 align: Align{y: 0.5}
                                            books_net_value := Label{width: Fit height: Fit text: "—" draw_text.color: #x9A0E10 draw_text.text_style.font_size: 15.}
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
                                    books_sec_bills_hdr := AccordionHdrBtn{text: "BILLS"}
                                    books_sec_bills_body := View{
                                        width: Fill height: Fit
                                        flow: Down spacing: 7
                                        visible: false
                                        h_caption_bills := Label{text: "BILLS" visible: false draw_text.color: #x93705F draw_text.text_style.font_size: 10.}
                                        View{width: Fill height: Fit flow: Right spacing: 8 align: Align{y: 0.5}
                                            books_declared_name := Label{width: Fill height: Fit text: "Tax invoice" draw_text.color: #x6B4A3F draw_text.text_style.font_size: 12.}
                                            View{width: Fit height: Fit flow: Right spacing: 4 align: Align{y: 0.5}
                                            books_declared_value := Label{width: Fit height: Fit text: "—" draw_text.color: #x260B08 draw_text.text_style.font_size: 13.}
                                            books_declared_eye := EyeIconBtn{}
                                        }
                                        }
                                        View{width: Fill height: Fit flow: Right spacing: 8 align: Align{y: 0.5}
                                            books_owner_name := Label{width: Fill height: Fit text: "Cash memo" draw_text.color: #x6B4A3F draw_text.text_style.font_size: 12.}
                                            View{width: Fit height: Fit flow: Right spacing: 4 align: Align{y: 0.5}
                                            books_owner_value := Label{width: Fit height: Fit text: "—" draw_text.color: #x260B08 draw_text.text_style.font_size: 13.}
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
                                        h_caption_payments_today := Label{text: "PAYMENTS TODAY" visible: false draw_text.color: #x93705F draw_text.text_style.font_size: 10.}
                                        View{width: Fill height: Fit flow: Right spacing: 8 align: Align{y: 0.5}
                                            h_cash := Label{width: Fill height: Fit text: "Cash" draw_text.color: #x6B4A3F draw_text.text_style.font_size: 12.}
                                            View{width: Fit height: Fit flow: Right spacing: 4 align: Align{y: 0.5}
                                            books_cash_value := Label{width: Fit height: Fit text: "—" draw_text.color: #x260B08 draw_text.text_style.font_size: 13.}
                                            books_cash_eye := EyeIconBtn{}
                                        }
                                        }
                                        View{width: Fill height: Fit flow: Right spacing: 8 align: Align{y: 0.5}
                                            books_digital_name := Label{width: Fill height: Fit text: "UPI" draw_text.color: #x6B4A3F draw_text.text_style.font_size: 12.}
                                            View{width: Fit height: Fit flow: Right spacing: 4 align: Align{y: 0.5}
                                            books_digital_value := Label{width: Fit height: Fit text: "—" draw_text.color: #x260B08 draw_text.text_style.font_size: 13.}
                                            books_digital_eye := EyeIconBtn{}
                                        }
                                        }
                                        View{width: Fill height: Fit flow: Right spacing: 8 align: Align{y: 0.5}
                                            h_card := Label{width: Fill height: Fit text: "Card" draw_text.color: #x6B4A3F draw_text.text_style.font_size: 12.}
                                            View{width: Fit height: Fit flow: Right spacing: 4 align: Align{y: 0.5}
                                            books_card_value := Label{width: Fit height: Fit text: "—" draw_text.color: #x260B08 draw_text.text_style.font_size: 13.}
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
                                        h_caption_outstanding := Label{text: "OUTSTANDING" visible: false draw_text.color: #x93705F draw_text.text_style.font_size: 10.}
                                        View{width: Fill height: Fit flow: Right spacing: 8 align: Align{y: 0.5}
                                            h_they_owe_us := Label{width: Fill height: Fit text: "They owe us" draw_text.color: #x6B4A3F draw_text.text_style.font_size: 12.}
                                            View{width: Fit height: Fit flow: Right spacing: 4 align: Align{y: 0.5}
                                            books_they_owe_value := Label{width: Fit height: Fit text: "—" draw_text.color: #x137333 draw_text.text_style.font_size: 13.}
                                            books_they_owe_eye := EyeIconBtn{}
                                        }
                                        }
                                        View{width: Fill height: Fit flow: Right spacing: 8 align: Align{y: 0.5}
                                            h_we_owe_them := Label{width: Fill height: Fit text: "We owe them" draw_text.color: #x6B4A3F draw_text.text_style.font_size: 12.}
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
                                        h_caption_bank := Label{text: "BANK" visible: false draw_text.color: #x93705F draw_text.text_style.font_size: 10.}
                                        View{width: Fill height: Fit flow: Right spacing: 8 align: Align{y: 0.5}
                                            h_bank_balance := Label{width: Fill height: Fit text: "Bank balance" draw_text.color: #x6B4A3F draw_text.text_style.font_size: 12.}
                                            View{width: Fit height: Fit flow: Right spacing: 4 align: Align{y: 0.5}
                                            books_bank_balance_value := Label{width: Fit height: Fit text: "—" draw_text.color: #x260B08 draw_text.text_style.font_size: 13.}
                                            books_bank_balance_eye := EyeIconBtn{}
                                        }
                                        }
                                        View{width: Fill height: Fit flow: Right spacing: 8 align: Align{y: 0.5}
                                            h_cash_in_hand := Label{width: Fill height: Fit text: "Cash in hand (till)" draw_text.color: #x6B4A3F draw_text.text_style.font_size: 12.}
                                            View{width: Fit height: Fit flow: Right spacing: 4 align: Align{y: 0.5}
                                            books_till_value := Label{width: Fit height: Fit text: "—" draw_text.color: #x260B08 draw_text.text_style.font_size: 13.}
                                            books_till_eye := EyeIconBtn{}
                                        }
                                        }
                                        View{width: Fill height: Fit flow: Right spacing: 8 align: Align{y: 0.5}
                                            h_bank_net_today := Label{width: Fill height: Fit text: "Bank net today" draw_text.color: #x6B4A3F draw_text.text_style.font_size: 12.}
                                            View{width: Fit height: Fit flow: Right spacing: 4 align: Align{y: 0.5}
                                            books_bank_net_value := Label{width: Fit height: Fit text: "—" draw_text.color: #x260B08 draw_text.text_style.font_size: 13.}
                                            books_bank_net_eye := EyeIconBtn{}
                                        }
                                        }
                                    }
                                }
                            }
                        }

                        // Seller: tap items → pay.
                        // Sticky = ONLY Dine-in|Takeaway|Delivery|Counter.
                        // Title, total, table pick, Add dishes live in the
                        // scrolling Added-items column so the ticket gets height.
                        pos_screen := View{
                            width: Fill height: Fill
                            visible: false
                            flow: Down spacing: 0
                            // Sticky: the four order-type buttons — nothing else.
                            pos_sticky := SolidView{
                                width: Fill height: Fit
                                padding: Inset{left: 16. right: 16. top: 8. bottom: 8.}
                                flow: Down spacing: 0
                                draw_bg.color: #xEFD2BC
                                otype_strip := View{
                                    width: Fill height: Fit
                                    flow: Right spacing: 8
                                    View{width: Fill height: Fit flow: Overlay
                                        otype_dinein := SegBtn{text: "Dine-in"}
                                        otype_dinein_on := SegBtnOn{text: "Dine-in" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        otype_takeaway := SegBtn{text: "Takeaway"}
                                        otype_takeaway_on := SegBtnOn{text: "Takeaway" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        otype_delivery := SegBtn{text: "Delivery"}
                                        otype_delivery_on := SegBtnOn{text: "Delivery" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        otype_counter := SegBtn{text: "Counter"}
                                        otype_counter_on := SegBtnOn{text: "Counter" visible: false}
                                    }
                                }
                            }
                            // Ticket: Added items | Counter under the four sticky chips.
                            pos_ticket := ScrollXView{
                                width: Fill height: Fill
                                padding: Inset{left: 16. right: 16. top: 8. bottom: 16.}
                                scroll_bars: mod.widgets.ScrollBars {
                                    show_scroll_x: true show_scroll_y: false
                                    scroll_bar_x.drag_scrolling: true
                                    scroll_bar_x.mouse_capture_overload: true
                                    scroll_bar_x.bar_size: 8.
                                }
                                pos_ticket_row := View{
                                    width: Fit height: Fill
                                    flow: Right spacing: 16
                                pos_scroll := ScrollYView{
                                    width: 600 height: Fill
                                    pos_page := View{
                                        width: Fill height: Fit
                                        flow: Down spacing: 10
                                        padding: Inset{top: 4. bottom: 16.}

                                    // Scrolls with the ticket — not sticky.
                                    sell_help := InfoHelp{
                                        width: Fit
                                        tip: "Tap an item to add it."
                                    }
                                    sticky_total := Label{
                                        width: Fill height: Fit
                                        text: "Total · ₹0.00"
                                        draw_text.color: #x260B08
                                        draw_text.text_style.font_size: 15.
                                    }
                                    table_pick_wrap := View{
                                        width: Fill height: Fit
                                        table_pick_btn := TablePickBtn{text: "Choose a table"}
                                    }
                                    catalog_header := View{
                                        width: Fill height: Fit
                                        align: Align{x: 0.5 y: 0.5}
                                        flow: Down spacing: 10
                                        menu_open_btn := TablePickBtn{
                                            text: "+ Add dishes"
                                            align: Align{x: 0.5 y: 0.5}
                                            label_walk: Walk{width: Fit height: Fit}
                                        }
                                    }
                                    pos_catalog := View{
                                        width: Fill height: Fit
                                        flow: Down spacing: 10
                                        order_help := WrapLabel{
                                            visible: false
                                            body: "Tap an item to add it."
                                            font_color: #x6B4A3F
                                            font_size: 12.
                                        }
                                        empty_catalog := WrapLabel{
                                            body: "No items yet. Add your dishes and items to start taking orders."
                                            font_color: #x260B08
                                            font_size: 14.
                                        }
                                        catalog_cta := ActionStack{
                                            add_items_btn := StackSecondaryBtn{text: "Add dishes & items"}
                                        }
                                    }

                                    // Operational strip (Stitch screen 02)
                                    pos_strip := RoundedView{
                                        // The header names who is on, the ticket names the
                                        // table: on the order desk this strip only cost height.
                                        visible: false
                                        width: Fill height: Fit
                                        padding: Inset{left: 10. right: 10. top: 8. bottom: 8.}
                                        flow: Right spacing: 8
                                        align: Align{y: 0.5}
                                        draw_bg.color: #xF6E3D3
                                        draw_bg.border_radius: 12.
                                        View{
                                            width: Fill height: Fit
                                            flow: Down spacing: 2
                                            session_chip := WrapLabel{
                                                body: "Counter"
                                                font_color: #x260B08
                                                font_size: 12.
                                            }
                                            table_chip := WrapLabel{
                                                body: "Ready to order"
                                                font_color: #x260B08
                                                font_size: 11.
                                            }
                                        }
                                        h_local_badge_3 := Label{
                                            text: "LOCAL"
                                            draw_text.color: #x9A6206
                                            draw_text.text_style.font_size: 10.
                                        }
                                    }

                                    items_body := View{
                                        width: Fill height: Fit
                                        flow: Down spacing: 10
                                        padding: Inset{bottom: 8.}
                                        View{
                                            width: Fill height: Fit
                                            flow: Right spacing: 12
                                            align: Align{y: 0.5}
                                            // Ticket state. Cart/pay/KOT errors land on
                                            // this line, so it must wrap (width: Fill).
                                            receipt_label := WrapLabel{
                                                width: Fill
                                                body: "Ready"
                                                font_color: #x6B4A3F
                                                font_size: 13.
                                            }
                                        }
                                        // Table actions (Print bill · Move / merge). Shown
                                        // while a table is open on the ticket.
                                        table_actions := ActionStack{
                                            print_bill_btn := StackLightBtn{text: "Print bill"}
                                            table_move_btn := StackLightBtn{text: "Move / merge table"}
                                            table_split_btn := StackLightBtn{text: "Split bill"}
                                            split_evenly_btn := StackLightBtn{text: "Split evenly"}
                                            fire_course_btn := StackLightBtn{text: "Fire course"}
                                            hold_item_btn := StackLightBtn{text: "Hold item"}
                                            waiter_call_btn := StackLightBtn{text: "Call waiter"}
                                        }
                                        // Split: tap the dishes one guest pays for, then make
                                        // their separate bill (a companion table, e.g. D3/2).
                                        split_bar := RoundedView{
                                            width: Fill height: Fit
                                            visible: false
                                            padding: Inset{left: 14. right: 14. top: 12. bottom: 12.}
                                            flow: Down spacing: 8
                                            draw_bg.color: #xFFF6EE
                                            draw_bg.border_radius: 14.
                                            draw_bg.border_size: 2.
                                            draw_bg.border_color: #xB43A12
                                            View{
                                                width: Fill height: Fit
                                                flow: Right spacing: 8
                                                align: Align{x: 0.0 y: 0.5}
                                                h_split_bill := WrapLabel{
                                                    width: Fit
                                                    body: "Split bill"
                                                    font_color: #x260B08
                                                    font_size: 14.
                                                }
                                                split_bill_help := InfoHelp{
                                                    width: Fit
                                                    tip: "Tap the dishes one guest pays for, then make their separate bill."
                                                }
                                            }
                                            split_help := WrapLabel{
                                                body: " "
                                                font_color: #x260B08
                                                font_size: 13.
                                                visible: false
                                            }
                                            ActionStack{
                                                split_go := StackPrimaryBtn{text: "Make separate bill"}
                                                split_cancel := StackSecondaryBtn{text: "Cancel"}
                                            }
                                        }
                                        table_move_panel := RoundedView{
                                            width: Fill height: Fit
                                            visible: false
                                            padding: Inset{left: 14. right: 14. top: 12. bottom: 12.}
                                            flow: Down spacing: 8
                                            draw_bg.color: #xFFF6EE
                                            draw_bg.border_radius: 14.
                                            draw_bg.border_size: 2.
                                            draw_bg.border_color: #xB43A12
                                            View{
                                                width: Fill height: Fit
                                                flow: Right spacing: 8
                                                align: Align{x: 0.0 y: 0.5}
                                                h_move_table := WrapLabel{
                                                    width: Fit
                                                    body: "Move / merge"
                                                    font_color: #x260B08
                                                    font_size: 14.
                                                }
                                                move_table_help := InfoHelp{
                                                    width: Fit
                                                    tip: "Move this table's order to a free table, or merge it into a running one."
                                                }
                                            }
                                            move_help := WrapLabel{
                                                body: "Move this table's order to a free table, or merge it into a running one."
                                                font_color: #x260B08
                                                font_size: 13.
                                                visible: false
                                            }
                                            move_target_input := FieldInput{empty_text: "Table name (e.g. D5, P2)"}
                                            move_target_suggest := mod.widgets.SuggestList{field: "move_table"}
                                            move_pin_input := FieldInput{empty_text: "Manager PIN (only if asked)"}
                                            ActionStack{
                                                move_table_go := StackPrimaryBtn{text: "Move here"}
                                                merge_table_go := StackSecondaryBtn{text: "Merge into it"}
                                            }
                                        }
                                        // One sent line's Cancel / NC / Move: reason, cooked or
                                        // not, how many, where to, and a manager PIN when the
                                        // person's right says "with approval".
                                        line_action_panel := RoundedView{
                                            width: Fill height: Fit
                                            visible: false
                                            padding: Inset{left: 14. right: 14. top: 12. bottom: 12.}
                                            flow: Down spacing: 8
                                            draw_bg.color: #xFFF6EE
                                            draw_bg.border_radius: 14.
                                            draw_bg.border_size: 2.
                                            draw_bg.border_color: #xB43A12
                                            line_action_title := WrapLabel{
                                                body: "Cancel item"
                                                font_color: #x69090C
                                                font_size: 15.
                                            }
                                            // Inputs hide through their wrapper (TextInput has no
                                            // `visible`).
                                            line_reason_wrap := View{
                                                width: Fill height: Fit
                                                line_reason_input := FieldInput{empty_text: "Reason (e.g. guest changed mind, out of stock)"}
                                            }
                                            line_qty_wrap := View{
                                                width: Fill height: Fit
                                                line_qty_input := FieldInput{empty_text: "How many to cancel (blank = all)"}
                                            }
                                            line_cooked_wrap := View{
                                                width: Fill height: Fit
                                                flow: Right spacing: 6
                                                View{width: Fill height: Fit flow: Overlay
                                                    line_cooked_raw := SegBtn{text: "Not cooked yet"}
                                                    line_cooked_raw_on := SegBtnOn{text: "Not cooked yet" visible: false}
                                                }
                                                View{width: Fill height: Fit flow: Overlay
                                                    line_cooked_done := SegBtn{text: "Already cooked"}
                                                    line_cooked_done_on := SegBtnOn{text: "Already cooked" visible: false}
                                                }
                                            }
                                            line_target_wrap := View{
                                                width: Fill height: Fit
                                                line_target_input := FieldInput{empty_text: "Move to table (e.g. D5)"}
                                                line_target_suggest := mod.widgets.SuggestList{field: "line_table"}
                                            }
                                            line_pin_input := FieldInput{empty_text: "Manager PIN (only if asked)"}
                                            ActionStack{
                                                line_action_back := StackSecondaryBtn{text: "Back"}
                                                line_action_go := StackPrimaryBtn{text: "Confirm"}
                                            }
                                        }

                                        // Only while the ticket has lines — an empty bordered
                                        // card is noise; + Add dishes already says what to do.
                                        cart_ticket_card := RoundedView{
                                            width: Fill height: Fit
                                            visible: false
                                            padding: Inset{left: 16. right: 16. top: 14. bottom: 14.}
                                            flow: Down spacing: 10
                                            draw_bg.color: #xffffff
                                            draw_bg.border_radius: 16.
                                            draw_bg.border_size: 2.
                                            draw_bg.border_color: #x5A4238
                                            cart_list := mod.widgets.CartList{}
                                            // Kept for locale/align; card is hidden when empty.
                                            cart_empty_hint := WrapLabel{
                                                width: Fit
                                                visible: false
                                                body: "Order empty — tap items to add."
                                                font_color: #x6B4A3F
                                                font_size: 12.
                                            }

                                            // Right under the lines just punched: pick a
                                            // station for dishes that have none, then send.
                                            kot_dest_help := InfoHelp{
                                                width: Fit
                                                tip: " "
                                                visible: false
                                            }
                                            kot_station_wrap := View{
                                                width: Fill height: Fit
                                                visible: false
                                                flow: Down spacing: 6
                                                kot_station_input := FieldInput{
                                                    empty_text: "Kitchen station for dishes without one (e.g. Tandoor)"
                                                }
                                                kot_station_suggest := mod.widgets.SuggestList{field: "kot_station"}
                                            }
                                            send_stack := ActionStack{
                                                kot_set_station := StackLightBtn{text: "Use this station" visible: false}
                                                send_kot_btn := StackPrimaryBtn{text: "Send KOT to kitchen"}
                                                next_order_btn := StackLightBtn{text: "Next order (clear desk)" visible: false}
                                            }
                                        }
                                    }
                                    } // pos_page
                                } // pos_scroll
                                counter_dock := ScrollYView{
                                    width: 520 height: Fill
                                    View{
                                        width: Fill height: Fit
                                        flow: Down spacing: 8
                                        padding: Inset{bottom: 12.}
                                    ticket_body := RoundedView{
                                    width: Fill height: Fit
                                    flow: Down spacing: 10
                                    padding: Inset{left: 16. right: 16. top: 14. bottom: 14.}
                                    draw_bg.color: #xffffff
                                    draw_bg.border_radius: 16.
                                    draw_bg.border_size: 1.5
                                    draw_bg.border_color: #xD8C7BD

                                    // The bill: one big total (tap it to round), with the
                                    // menu-vs-applied detail only when lines were re-priced.
                                    RoundedView{
                                        width: Fill height: Fit
                                        padding: Inset{left: 14. right: 10. top: 8. bottom: 8.}
                                        flow: Right spacing: 10
                                        align: Align{y: 0.5}
                                        draw_bg.color: #xEFD2BC
                                        draw_bg.border_radius: 12.
                                        View{
                                            width: Fill height: Fit
                                            flow: Down spacing: 0
                                            total_word := Label{
                                                text: "Total"
                                                draw_text.color: #x260B08
                                                draw_text.text_style.font_size: 18.
                                            }
                                            cart_charge_caption := WrapLabel{
                                                selectable: false
                                                body: "ROUNDED"
                                                font_color: #xB43A12
                                                font_size: 10.
                                            }
                                        }
                                        cart_total_input := CartField{
                                            width: 140
                                            empty_text: "0"
                                            text: "0"
                                            draw_bg +: {
                                                border_color: #xB43A12
                                                border_color_hover: #x822801
                                                border_color_focus: #x69090C
                                                border_radius: 8.
                                                border_size: 1.
                                            }
                                            draw_text +: {
                                                color: #x822801
                                                color_hover: #x822801
                                                color_down: #x822801
                                                color_focus: #x822801
                                                color_empty: #x6B4A3F
                                                color_empty_hover: #x260B08
                                                color_empty_focus: #x260B08
                                                text_style +: {font_size: 16.}
                                            }
                                        }
                                    }
                                    total_detail := View{
                                        width: Fill height: Fit
                                        visible: false
                                        RoundedView{
                                            width: Fill height: Fit
                                            padding: Inset{left: 8. right: 8. top: 8. bottom: 8.}
                                            flow: Right spacing: 4
                                            draw_bg.color: #xEFD2BC
                                            draw_bg.border_radius: 12.
                                            View{
                                                width: Fill height: Fit
                                                flow: Down spacing: 2
                                                align: Align{x: 0.5}
                                                h_actual_total_cap := WrapLabel{
                                                    body: "ACTUAL TOTAL"
                                                    font_color: #x6B4A3F
                                                    font_size: 10.
                                                }
                                                cart_natural_label := WrapLabel{
                                                    body: "0"
                                                    font_color: #x260B08
                                                    font_size: 14.
                                                }
                                            }
                                            View{
                                                width: Fill height: Fit
                                                flow: Down spacing: 2
                                                align: Align{x: 0.5}
                                                h_applied_total_cap := WrapLabel{
                                                    body: "APPLIED TOTAL"
                                                    font_color: #x6B4A3F
                                                    font_size: 10.
                                                }
                                                cart_applied_label := WrapLabel{
                                                    body: "0"
                                                    font_color: #x260B08
                                                    font_size: 14.
                                                }
                                            }
                                        }
                                    }
                                    // Exclusive-tax shops quote net prices, so the strip above is
                                    // not what the customer pays — this line is.
                                    tax_estimate_label := WrapLabel{
                                        body: " "
                                        font_color: #xB43A12
                                        font_size: 13.
                                    }

                                    // Bill discount: 10% or an amount, always with a reason;
                                    // a manager's PIN when the person's right asks for it.
                                    discount_label := WrapLabel{
                                        body: " "
                                        font_color: #x69090C
                                        font_size: 13.
                                    }
                                    ActionStack{
                                        discount_toggle := StackLightBtn{text: "Discount"}
                                    }
                                    discount_panel := View{
                                        width: Fill height: Fit
                                        flow: Down spacing: 8
                                        visible: false
                                        discount_input := FieldInput{empty_text: "10% or 50"}
                                        discount_reason_input := FieldInput{empty_text: "Reason (e.g. regular guest, late food)"}
                                        discount_pin_input := FieldInput{empty_text: "Manager PIN (only if asked)"}
                                        ActionStack{
                                            discount_remove := StackSecondaryBtn{text: "Remove discount"}
                                            discount_apply := StackPrimaryBtn{text: "Apply discount"}
                                        }
                                    }
                                    // Bill type: one compact segmented row.
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
                                    money_lane_label := WrapLabel{
                                        body: " "
                                        visible: false
                                    }
                                    // How they pay: one tap. A single method takes the
                                    // whole bill; Split opens the amounts.
                                    View{
                                        width: Fill height: Fit
                                        flow: Right spacing: 4
                                        View{width: Fill height: Fit flow: Overlay
                                            pm_cash := SegBtn{text: "Cash"}
                                            pm_cash_on := SegBtnOn{text: "Cash" visible: false}
                                        }
                                        View{width: Fill height: Fit flow: Overlay
                                            pm_upi := SegBtn{text: "UPI"}
                                            pm_upi_on := SegBtnOn{text: "UPI" visible: false}
                                        }
                                        View{width: Fill height: Fit flow: Overlay
                                            pm_card := SegBtn{text: "Card"}
                                            pm_card_on := SegBtnOn{text: "Card" visible: false}
                                        }
                                        View{width: Fill height: Fit flow: Overlay
                                            pm_credit := SegBtn{text: "Credit"}
                                            pm_credit_on := SegBtnOn{text: "Credit" visible: false}
                                        }
                                        View{width: Fill height: Fit flow: Overlay
                                            pm_split := SegBtn{text: "Split"}
                                            pm_split_on := SegBtnOn{text: "Split" visible: false}
                                        }
                                    }
                                    split_panel := View{
                                        width: Fill height: Fit
                                        flow: Down spacing: 8
                                        visible: false
                                        View{
                                            width: Fill height: Fit
                                            flow: Right spacing: 8
                                            align: Align{x: 0.0 y: 0.5}
                                            h_split_payment := WrapLabel{
                                                    width: Fit
                                                body: "Split"
                                                font_color: #x260B08
                                                font_size: 14.
                                            }
                                            payment_help := InfoHelp{
                                                width: Fit
                                                tip: "Split across cash / UPI / card / credit if needed. Credit needs a customer above."
                                            }
                                        }
                                        h_mix_type_hint_khaynjo := WrapLabel{
                                            body: "Split across cash / UPI / card / credit if needed. Credit needs a customer above."
                                            font_color: #x6B4A3F
                                            font_size: 11.
                                            visible: false
                                        }
                                        mix_cash_input := FieldInput{empty_text: "Cash amount"}
                                        mix_upi_input := FieldInput{empty_text: "UPI amount"}
                                        mix_card_input := FieldInput{empty_text: "Card amount"}
                                        mix_credit_input := FieldInput{empty_text: "Credit / khata amount"}
                                    }
                                    // Settle & Save: cash handed over → change to return;
                                    // tip is the staff's (not a sale).
                                    cash_panel := View{
                                        width: Fill height: Fit
                                        flow: Down spacing: 8
                                        tendered_input := FieldInput{empty_text: "Customer paid"}
                                        tip_input := FieldInput{empty_text: "Tip"}
                                        change_label := WrapLabel{
                                            body: " "
                                            font_color: #x260B08
                                            font_size: 14.
                                        }
                                    }
                                    ActionStack{
                                        show_upi_qr := StackLightBtn{text: "Show UPI QR"}
                                        pay_mix := PayBtn{text: "Take payment"}
                                    }
                                }
                                // Rounding note and the customer for credit / delivery:
                                // only once there is something on the ticket.
                                ticket_extras := View{
                                    width: Fill height: Fit
                                    flow: Down spacing: 10
                                    cart_round_hint := WrapLabel{
                                        body: "Rounded starts equal to Applied. Edit only to round the bill."
                                        font_color: #x260B08
                                        font_size: 12.
                                        visible: false
                                    }
                                    credit_chip := WrapLabel{
                                        body: "No customer yet — type a name, then tap Credit"
                                        font_color: #x260B08
                                        font_size: 13.
                                    }
                                    customer_name_input := FieldInput{
                                        empty_text: "Customer name"
                                    }
                                    customer_name_suggest := mod.widgets.SuggestList{field: "customer_name"}
                                    customer_use := StackLightBtn{text: "Add / use this name"}
                                    customer_clear := StackLightBtn{text: "Clear customer"}
                                    customers_empty_hint := WrapLabel{
                                        body: "Saved customers show up here for quick pick."
                                        font_color: #x260B08
                                        font_size: 12.
                                    }
                                    customer_list_sell := mod.widgets.CustomerList{}
                                    // Loyalty / wallet / voucher — applied as a bill discount on Take payment.
                                    till_loyalty_panel := View{
                                        width: Fill height: Fit
                                        visible: false
                                        flow: Down spacing: 6
                                        till_loyalty_chip := WrapLabel{
                                            body: ""
                                            font_color: #x69090C
                                            font_size: 12.
                                        }
                                        FormFieldRow{
                                            till_loyalty_input := FieldInput{empty_text: "Points to redeem"}
                                            till_wallet_input := FieldInput{empty_text: "Wallet ₹ to spend"}
                                        }
                                        FormFieldRow{
                                            till_voucher_input := FieldInput{empty_text: "Voucher code"}
                                            till_voucher_amt_input := FieldInput{empty_text: "Voucher ₹ (blank = all)"}
                                        }
                                        FormFieldRow{
                                            till_wallet_topup_input := FieldInput{empty_text: "Top-up wallet ₹"}
                                            till_voucher_issue_amt_input := FieldInput{empty_text: "Issue voucher ₹"}
                                        }
                                        till_voucher_issue_code := FieldInput{empty_text: "New voucher code"}
                                        ActionStack{
                                            till_apply_redeem := StackPrimaryBtn{text: "Apply on bill"}
                                            till_clear_redeem := StackLightBtn{text: "Clear redeem"}
                                            till_wallet_topup := StackLightBtn{text: "Top up wallet"}
                                            till_issue_voucher := StackLightBtn{text: "Issue voucher"}
                                        }
                                    }
                                    // Delivery orders: who takes it out (day report totals by them).
                                    delivery_by_input := FieldInput{empty_text: "Delivery by (rider or staff name)"}
                                    delivery_by_suggest := mod.widgets.SuggestList{field: "delivery_by"}
                                }
                                    } // counter dock inner
                                } // counter_dock
                                } // pos_ticket_row
                            } // pos_ticket
                        } // pos_screen

                        floor_screen := ScrollYView{
                            width: Fill height: Fill
                            visible: false
                            View{
                                width: Fill height: Fit
                                flow: Down spacing: 10
                                padding: Inset{left: 16. right: 16. top: 8. bottom: 120.}
                                // Came here from the order desk to seat an order: say
                                // so, and offer the way back without choosing.
                                floor_pick_bar := RoundedView{
                                    width: Fill height: Fit
                                    visible: false
                                    padding: Inset{left: 16. right: 12. top: 10. bottom: 10.}
                                    flow: Right spacing: 12
                                    align: Align{y: 0.5}
                                    draw_bg.color: #x9A0E10
                                    draw_bg.border_radius: 14.
                                    floor_pick_label := WrapLabel{
                                        selectable: false
                                        body: "Pick a table for this order"
                                        font_color: #xFFFFFF
                                        font_size: 15.
                                    }
                                    floor_pick_back := Button{
                                        text: "Back to order"
                                        width: Fit height: 40
                                        padding: Inset{left: 16. right: 16. top: 6. bottom: 6.}
                                        draw_bg +: {
                                            color: #xFFFFFF
                                            color_hover: #xFBE3D3
                                            color_down: #xEFD2BC
                                            color_focus: #xFFFFFF
                                            border_radius: 10.
                                            border_size: 0.
                                        }
                                        draw_text +: {
                                            color: #x69090C
                                            color_hover: #x69090C
                                            color_down: #x69090C
                                            color_focus: #x69090C
                                            text_style +: {font_size: 14.}
                                        }
                                    }
                                }
                                floor_help := InfoHelp{
                                    width: Fit
                                    tip: "Tap a table to take or change the order. Colours show free → seated → kitchen → ready → eating → bill → paid."
                                }
                                // At a glance: busy tables, bills to collect, money on
                                // the floor.
                                floor_summary := WrapLabel{
                                    body: " "
                                    font_color: #x3A2C26
                                    font_size: 14.
                                }
                                // Colour key — real swatches, the same colours as the tiles.
                                View{
                                    width: Fill height: Fit
                                    flow: Right{wrap: true}
                                    View{
                                        width: Fit height: Fit
                                        flow: Right spacing: 6
                                        align: Align{y: 0.5}
                                        margin: Inset{right: 14. bottom: 4.}
                                        RoundedView{
                                            width: 16 height: 16
                                            draw_bg.color: #xFFFFFF
                                            draw_bg.border_radius: 4.
                                            draw_bg.border_size: 1.5
                                            draw_bg.border_color: #xD8C7BD
                                        }
                                        lg_free := Label{
                                            width: Fit
                                            text: "free"
                                            draw_text.color: #x3A2C26
                                            draw_text.text_style.font_size: 12.
                                        }
                                    }
                                    View{
                                        width: Fit height: Fit
                                        flow: Right spacing: 6
                                        align: Align{y: 0.5}
                                        margin: Inset{right: 14. bottom: 4.}
                                        RoundedView{
                                            width: 16 height: 16
                                            draw_bg.color: #xFFF4CC
                                            draw_bg.border_radius: 4.
                                            draw_bg.border_size: 1.5
                                            draw_bg.border_color: #xE8C200
                                        }
                                        lg_seated := Label{
                                            width: Fit
                                            text: "occupied"
                                            draw_text.color: #x3A2C26
                                            draw_text.text_style.font_size: 12.
                                        }
                                    }
                                    View{
                                        width: Fit height: Fit
                                        flow: Right spacing: 6
                                        align: Align{y: 0.5}
                                        margin: Inset{right: 14. bottom: 4.}
                                        RoundedView{
                                            width: 16 height: 16
                                            draw_bg.color: #xFFE9A8
                                            draw_bg.border_radius: 4.
                                            draw_bg.border_size: 1.5
                                            draw_bg.border_color: #xD9A400
                                        }
                                        lg_kitchen := Label{
                                            width: Fit
                                            text: "kitchen"
                                            draw_text.color: #x3A2C26
                                            draw_text.text_style.font_size: 12.
                                        }
                                    }
                                    View{
                                        width: Fit height: Fit
                                        flow: Right spacing: 6
                                        align: Align{y: 0.5}
                                        margin: Inset{right: 14. bottom: 4.}
                                        RoundedView{
                                            width: 16 height: 16
                                            draw_bg.color: #xFFC48A
                                            draw_bg.border_radius: 4.
                                            draw_bg.border_size: 1.5
                                            draw_bg.border_color: #xE06A00
                                        }
                                        lg_ready := Label{
                                            width: Fit
                                            text: "ready"
                                            draw_text.color: #x3A2C26
                                            draw_text.text_style.font_size: 12.
                                        }
                                    }
                                    View{
                                        width: Fit height: Fit
                                        flow: Right spacing: 6
                                        align: Align{y: 0.5}
                                        margin: Inset{right: 14. bottom: 4.}
                                        RoundedView{
                                            width: 16 height: 16
                                            draw_bg.color: #xCFE3FF
                                            draw_bg.border_radius: 4.
                                            draw_bg.border_size: 1.5
                                            draw_bg.border_color: #x3B7DD8
                                        }
                                        lg_eating := Label{
                                            width: Fit
                                            text: "eating"
                                            draw_text.color: #x3A2C26
                                            draw_text.text_style.font_size: 12.
                                        }
                                    }
                                    View{
                                        width: Fit height: Fit
                                        flow: Right spacing: 6
                                        align: Align{y: 0.5}
                                        margin: Inset{right: 14. bottom: 4.}
                                        RoundedView{
                                            width: 16 height: 16
                                            draw_bg.color: #xBFEBC8
                                            draw_bg.border_radius: 4.
                                            draw_bg.border_size: 1.5
                                            draw_bg.border_color: #x2E9E4F
                                        }
                                        lg_bill := Label{
                                            width: Fit
                                            text: "bill"
                                            draw_text.color: #x3A2C26
                                            draw_text.text_style.font_size: 12.
                                        }
                                    }
                                    View{
                                        width: Fit height: Fit
                                        flow: Right spacing: 6
                                        align: Align{y: 0.5}
                                        margin: Inset{right: 14. bottom: 4.}
                                        RoundedView{
                                            width: 16 height: 16
                                            draw_bg.color: #xE3DCD7
                                            draw_bg.border_radius: 4.
                                            draw_bg.border_size: 1.5
                                            draw_bg.border_color: #x8C7B72
                                        }
                                        lg_paid := Label{
                                            width: Fit
                                            text: "paid"
                                            draw_text.color: #x3A2C26
                                            draw_text.text_style.font_size: 12.
                                        }
                                    }
                                }
                                floor_list := mod.widgets.FloorList{}
                                // Drawn-layout editor removed from Floor chrome —
                                // widgets kept off-screen so old ids still resolve.
                                floor_layout_list := mod.widgets.FloorLayoutList{visible: false}
                                floor_layout_editor := View{
                                    width: Fill height: Fit
                                    flow: Down spacing: 8
                                    visible: false
                                    floor_layout_edit_heading := WrapLabel{
                                        body: "Layout"
                                        font_color: #x260B08
                                        font_size: 15.
                                    }
                                    FormFieldRow{
                                        layout_x_input := FieldInput{empty_text: "X"}
                                        layout_y_input := FieldInput{empty_text: "Y"}
                                        layout_w_input := FieldInput{empty_text: "W"}
                                        layout_h_input := FieldInput{empty_text: "H"}
                                    }
                                    View{
                                        width: Fill height: Fit
                                        flow: Down spacing: 8
                                        layout_nudge_left := NavBtn{text: "← 20"}
                                        layout_nudge_right := NavBtn{text: "→ 20"}
                                        layout_nudge_up := NavBtn{text: "↑ 20"}
                                        layout_nudge_down := NavBtn{text: "↓ 20"}
                                    }
                                    ActionStack{
                                        layout_save := StackPrimaryBtn{text: "Save layout"}
                                        layout_open := StackLightBtn{text: "Open table"}
                                    }
                                }
                                // Secondary Floor tools — collapsed by default so
                                // tables stay the hero (not a wall of ops text).
                                RoundedView{
                                    width: Fill height: Fit
                                    padding: Inset{left: 14. right: 14. top: 10. bottom: 10.}
                                    flow: Down spacing: 7
                                    draw_bg.color: #xffffff
                                    draw_bg.border_radius: 14.
                                    floor_ops_hdr := AccordionHdrBtn{text: "LOCAL OPS"}
                                    floor_ops_body := View{
                                        width: Fill height: Fit
                                        flow: Down spacing: 8
                                        visible: false
                                        floor_ops_label := mod.widgets.ReportText{}
                                        floor_waiter_filter := FieldInput{empty_text: "Filter floor by waiter name"}
                                        floor_waiter_suggest := mod.widgets.SuggestList{field: "floor_waiter"}
                                    }
                                }
                                RoundedView{
                                    width: Fill height: Fit
                                    padding: Inset{left: 14. right: 14. top: 10. bottom: 10.}
                                    flow: Down spacing: 7
                                    draw_bg.color: #xffffff
                                    draw_bg.border_radius: 14.
                                    waitlist_hdr := AccordionHdrBtn{text: "WAITLIST"}
                                    waitlist_body := View{
                                        width: Fill height: Fit
                                        flow: Down spacing: 8
                                        visible: false
                                        waitlist_heading := WrapLabel{body: "Waitlist" font_color: #x260B08 font_size: 15. visible: false}
                                        FormCard{
                                            FormFieldRow{
                                                wait_name_input := FieldInput{empty_text: "Guest name"}
                                                wait_party_input := FieldInput{empty_text: "Party size"}
                                                wait_mins_input := FieldInput{empty_text: "Quoted wait (min)"}
                                            }
                                            waitlist_add := StackPrimaryBtn{text: "Add to waitlist"}
                                        }
                                    }
                                }
                                RoundedView{
                                    width: Fill height: Fit
                                    padding: Inset{left: 14. right: 14. top: 10. bottom: 10.}
                                    flow: Down spacing: 7
                                    draw_bg.color: #xffffff
                                    draw_bg.border_radius: 14.
                                    resv_hdr := AccordionHdrBtn{text: "RESERVATIONS"}
                                    resv_body := View{
                                        width: Fill height: Fit
                                        flow: Down spacing: 8
                                        visible: false
                                        resv_heading := WrapLabel{body: "Reservations" font_color: #x260B08 font_size: 15. visible: false}
                                        resv_list := mod.widgets.ResvList{}
                                        resv_empty := WrapLabel{body: " " font_color: #x6B4A3F font_size: 13.}
                                        FormCard{
                                            FormFieldRow{
                                                resv_name_input := FieldInput{empty_text: "Guest name"}
                                                resv_phone_input := FieldInput{empty_text: "Phone (optional)"}
                                            }
                                            FormFieldRow{
                                                View{
                                                    width: 140 height: Fit
                                                    resv_guests_input := FieldInput{empty_text: "Guests"}
                                                }
                                                resv_when_input := FieldInput{empty_text: "When — 20:30, tomorrow 13:00"}
                                                View{
                                                    width: 230 height: Fit
                                                    flow: Down spacing: 4
                                                    resv_table_input := FieldInput{empty_text: "Table (optional)"}
                                                    resv_table_suggest := mod.widgets.SuggestList{field: "resv_table"}
                                                }
                                            }
                                            resv_add := StackPrimaryBtn{text: "Book"}
                                        }
                                    }
                                }
                                RoundedView{
                                    width: Fill height: Fit
                                    padding: Inset{left: 14. right: 14. top: 10. bottom: 10.}
                                    flow: Down spacing: 7
                                    draw_bg.color: #xffffff
                                    draw_bg.border_radius: 14.
                                    View{
                                        width: Fill height: Fit
                                        flow: Right spacing: 6
                                        align: Align{y: 0.5}
                                        floor_add_hdr := AccordionHdrBtn{width: Fit text: "ADD TABLES"}
                                        floor_areas_help := InfoHelp{
                                            width: Fit
                                            tip: "Areas are rooms or zones (Dine In, Parcel, AC …). Add an area first, then put tables in it — each area is a separate group on the floor."
                                        }
                                    }
                                    floor_add_body := View{
                                        width: Fill height: Fit
                                        flow: Down spacing: 8
                                        visible: false
                                        floor_add_heading := WrapLabel{
                                            width: Fit
                                            body: "Add a table"
                                            font_color: #x260B08
                                            font_size: 15.
                                            visible: false
                                        }
                                        area_new_input := FieldInput{empty_text: "New area / room (e.g. AC, Garden)"}
                                        area_new_add := NavBtn{text: "Add area"}
                                        table_name_input := FieldInput{empty_text: "Table name (e.g. Patio 1, Counter 2)"}
                                        // Areas group the floor (Dine In, Parcel, AC …); a
                                        // parcel waiting for pickup is a table too.
                                        table_area_input := FieldInput{empty_text: "Area (e.g. Dine In, Parcel, AC)"}
                                        table_area_suggest := mod.widgets.SuggestList{field: "table_area"}
                                        table_add := StackPrimaryBtn{text: "Add table"}
                                        table_range_input := FieldInput{empty_text: "Many at once: D1-D20"}
                                        table_range_add := NavBtn{text: "Add these tables"}
                                        // Fresh shops start with T1..T24; a floor of
                                        // its own (D1-D20, P1-P20) takes them off.
                                        table_remove_input := FieldInput{empty_text: "Remove: T1-T24, or one table's name"}
                                        table_remove_btn := DangerBtn{text: "Remove these tables"}
                                    }
                                }
                            }
                        }

                        kitchen_screen := ScrollYView{
                            width: Fill height: Fill
                            visible: false
                            View{
                                width: Fill height: Fit
                                flow: Down spacing: 10
                                padding: Inset{left: 16. right: 16. top: 8. bottom: 120.}
                                kitchen_help := InfoHelp{
                                    width: Fit
                                    tip: "Kitchen tickets (KOT), newest last. Start → Ready → Done moves a ticket along. Tickets never show prices."
                                }
                                // All-day counts: how much of each dish is still to cook.
                                kitchen_counts := WrapLabel{body: " " font_color: #x69090C font_size: 15. visible: false}
                                kitchen_list := mod.widgets.KitchenList{}
                                kitchen_empty := WrapLabel{
                                    body: "No open tickets. Send a KOT from the Ordering screen."
                                    font_color: #x6B4A3F
                                    font_size: 13.
                                }
                                recently_done_heading := WrapLabel{body: "Recently done" font_color: #x260B08 font_size: 15. visible: false}
                                recently_done_label := mod.widgets.ReportText{}
                                recall_kot_input := FieldInput{empty_text: "KOT # to recall (or tap Recall below)"}
                                recall_kot_btn := NavBtn{text: "Recall"}
                                RoundedView{
                                    width: Fill height: Fit
                                    padding: Inset{left: 14. right: 14. top: 10. bottom: 10.}
                                    flow: Down spacing: 7
                                    draw_bg.color: #xffffff
                                    draw_bg.border_radius: 14.
                                    View{
                                        width: Fill height: Fit
                                        flow: Right spacing: 6
                                        align: Align{y: 0.5}
                                        stations_hdr := AccordionHdrBtn{width: Fit text: "KITCHEN STATIONS"}
                                        stations_help := InfoHelp{
                                            width: Fit
                                            tip: "Each station prints on its own printers (Settings → Kitchen printing). Add Tandoor, Fryer, Cold…"
                                        }
                                    }
                                    stations_body := View{
                                        width: Fill height: Fit
                                        flow: Down spacing: 8
                                        visible: false
                                        stations_heading := WrapLabel{
                                            width: Fit
                                            body: "Kitchen stations"
                                            font_color: #x260B08
                                            font_size: 15.
                                            visible: false
                                        }
                                        stations_label := mod.widgets.ReportText{}
                                        station_add_heading := WrapLabel{body: "Add a kitchen station" font_color: #x260B08 font_size: 15.}
                                        station_name_input := FieldInput{empty_text: "Station name (e.g. Tandoor, Fryer, Cold)"}
                                        station_add := StackPrimaryBtn{text: "Add station"}
                                        station_remove_heading := WrapLabel{body: "Remove a kitchen station" font_color: #x260B08 font_size: 15.}
                                        station_remove_input := FieldInput{empty_text: "Tap a station in the list to remove it"}
                                        station_remove_suggest := mod.widgets.SuggestList{field: "station_remove"}
                                        // Pick removes immediately; button kept for typed + tap.
                                        station_remove := StackSecondaryBtn{text: "Remove station"}
                                    }
                                }
                            }
                        }

                        menu_screen := ScrollYView{
                            width: Fill height: Fill
                            visible: false
                            View{
                                width: Fill height: Fit
                                flow: Down spacing: 14
                                padding: Inset{left: 22. right: 22. top: 16. bottom: 120.}
                                items_help_label := InfoHelp{
                                    width: Fit
                                    tip: "Tap an item to edit it. New item clears the form. Search our dishes or type your own name and photo, set a price (0 is fine), then Add. Sales already recorded keep the name they were charged with."
                                }
                                // Form errors and confirmations have to be visible
                                // where the form is: this screen had no status line,
                                // so "Name required" / "Set a price …" failed
                                // silently (2026-10-22).
                                menu_status := WrapLabel{
                                    body: " "
                                    font_color: #x69090C
                                    font_size: 13.
                                }
                                // The whole provided menu in one go: every category and
                                // dish we ship, with photos; prices are the owner's.
                                template_card := FormCard{
                                    template_heading := Label{
                                        text: "Our menu template"
                                        draw_text.color: #x260B08
                                        draw_text.text_style.font_size: 17.
                                    }
                                    template_info := Label{
                                        width: Fill
                                        text: " "
                                        flow: Right{wrap: true}
                                        draw_text.color: #x6B4A3F
                                        draw_text.text_style.font_size: 13.
                                    }
                                    template_import := NavBtn{text: "Import complete template"}
                                    template_confirm := View{
                                        width: Fill height: Fit
                                        visible: false
                                        flow: Right spacing: 8
                                        template_yes := StackPrimaryBtn{text: "Yes, import"}
                                        template_no := StackLightBtn{text: "Cancel"}
                                    }
                                }
                                menu_mode_label := WrapLabel{body: "Adding a new item" font_color: #x69090C font_size: 14.}
                                menu_name_input := FieldInput{
                                    empty_text: "Item name — search our dishes or type your own"
                                }
                                // Picker under the name: opens when the field is
                                // focused (empty = browse every dish), typing searches
                                // our dishes and your menu, and the typed name itself
                                // is always the first row, so your own name is one
                                // tap. Draws nothing while closed.
                                dish_pick_list := mod.widgets.DishPickList{}
                                ActionStack{
                                }
                                // What this item is: cooked here, or bought and sold as
                                // is (counted in Inventory, offered on Buy).
                                View{
                                    width: Fill height: Fit
                                    flow: Right spacing: 6
                                    View{width: Fill height: Fit flow: Overlay
                                        menu_kind_dish := SegBtn{text: "Dish we make"}
                                        menu_kind_dish_on := SegBtnOn{text: "Dish we make" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        menu_kind_resale := SegBtn{text: "Bought & resold"}
                                        menu_kind_resale_on := SegBtnOn{text: "Bought & resold" visible: false}
                                    }
                                }
                                menu_aliases_input := FieldInput{
                                    empty_text: "Aliases — other names, comma separated"
                                }
                                FormFieldRow{
                                    menu_price_input := FieldInput{empty_text: "Price"}
                                    menu_unit_input := FieldInput{empty_text: "Unit (plate / pc — blank = each)"}
                                }
                                menu_unit_suggest := mod.widgets.SuggestList{field: "menu_unit"}
                                menu_tax_input := FieldInput{
                                    empty_text: "Tax rate name or % — Standard, Reduced, 0, 5, 12, 18, 20"
                                }
                                menu_tax_suggest := mod.widgets.SuggestList{field: "menu_tax"}
                                menu_category_input := FieldInput{empty_text: "Category — search yours or type a new one"}
                                // Category picker: opens on focus, searches as you
                                // type; the typed name is always a row too.
                                category_pick_list := mod.widgets.CategoryPickList{}
                                ActionStack{
                                }
                                menu_station_input := FieldInput{empty_text: "Kitchen station (optional, e.g. Tandoor — created if new)"}
                                menu_station_suggest := mod.widgets.SuggestList{field: "menu_station"}
                                // Variations (* pick one) and add-ons (+ multi), or a flat add-on list.
                                menu_modifiers_input := FieldInput{empty_text: "Options — Size*: Small=199, Large=399 | Toppings+3: Cheese=40"}
                                // Quick entry: Half / Full become items of their own
                                // (own UUID, price, stock, station), grouped by name.
                                menu_variations_input := FieldInput{empty_text: "Sizes as separate items (optional) — Half=120, Full=200"}
                                menu_group_input := FieldInput{empty_text: "Dish group (optional) — e.g. Paneer Tikka"}
                                menu_group_suggest := mod.widgets.SuggestList{field: "menu_group"}
                                // A dearer area (AC) or a parcel price; other areas pay
                                // the item price.
                                menu_area_prices_input := FieldInput{empty_text: "Area prices (optional) — AC=220, Parcel=210"}
                                menu_schedule_input := FieldInput{empty_text: "Hours / happy hour — Mon-Fri 07:00-11:00 or HappyHour 17:00-19:00=150"}
                                // What one portion takes off the shelf.
                                menu_recipe_input := FieldInput{empty_text: "Recipe (optional) — one portion uses: Rice=0.2, Paneer=0.15"}
                                View{
                                    width: Fill height: Fit
                                    flow: Right spacing: 8
                                    align: Align{y: 0.5}
                                    h_item_photo := WrapLabel{
                                                    width: Fit
                                        body: "Photo"
                                        font_color: #x260B08
                                        font_size: 15.
                                    }
                                    item_photo_help := InfoHelp{
                                        width: Fit
                                        tip: "Optional. Without a photo we show initials from the item name."
                                    }
                                }
                                image_status := WrapLabel{
                                    body: " "
                                    font_color: #x6B4A3F
                                    font_size: 12.
                                }
                                draft_preview := ThumbAvatar{
                                    width: 96 height: 96
                                    margin: Inset{top: 0. bottom: 0. left: 0. right: 0.}
                                }
                                ActionStack{
                                    image_clear := RowSecondaryBtn{text: "Clear photo"}
                                    image_pick := RowPrimaryBtn{text: "Pick photo"}
                                }
                                // Add / Save closes the form, after every field
                                // and the photo — the last thing the owner does.
                                ActionStack{
                                    menu_new := StackSecondaryBtn{
                                        text: "New item"
                                        visible: false
                                    }
                                    menu_add := StackPrimaryBtn{text: "Add item"}
                                }
                                menu_empty := WrapLabel{
                                    body: "No items yet — add one above."
                                    font_color: #x260B08
                                    font_size: 14.
                                }
                                // A full menu is hundreds of dishes: find one by name.
                                menu_list_search := FieldInput{empty_text: "Find in your menu"}
                                // Shown while dishes have no price yet (a fresh
                                // template import): how many, and a filter to
                                // price them one after another.
                                menu_unpriced_note := WrapLabel{
                                    body: " "
                                    font_color: #x9A0E10
                                    font_size: 13.
                                    visible: false
                                }
                                menu_unpriced_btn := RowSecondaryBtn{text: "Show only dishes without a price" visible: false}
                                product_list_menu := mod.widgets.MenuList{}
                                // Remove is an archive, not a delete — these are
                                // the items taken off the menu, restorable
                                // exactly as they were. Hidden while none are.
                                h_removed_items := WrapLabel{body: "Removed items" font_color: #x260B08 font_size: 15.}
                                removed_help := WrapLabel{
                                    body: "Restore puts the item back on the menu exactly as it was — price, photo and category. Our shelf of dishes is never removed, so a dish can always be added again."
                                    font_color: #x6B4A3F
                                    font_size: 12.
                                }
                                removed_list := mod.widgets.RemovedList{}
                                h_your_categories := WrapLabel{body: "Your categories" font_color: #x260B08 font_size: 15.}
                                categories_help := WrapLabel{
                                    width: Fit
                                    body: "Tap Use to file the item above under that category. Photo gives the category a picture; without one it shows its initials."
                                    font_color: #x6B4A3F
                                    font_size: 12.
                                }
                                category_list := mod.widgets.CategoryList{}
                            }
                        }

                        // Money out — purchase + expense (peer of Sell).
                        // Money out, in the two shapes it takes: goods the kitchen
                        // received (raw material and resold goods — never its own
                        // dishes) and running costs. One card each.
                        buy_screen := ScrollYView{
                            width: Fill height: Fill
                            visible: false
                            View{
                                width: Fill height: Fit
                                flow: Down spacing: 14
                                padding: Inset{left: 22. right: 22. top: 16. bottom: 120.}
                                buy_help := InfoHelp{
                                    width: Fit
                                    tip: "Expense vouchers and supplier bills (money out). On-account stays on the party ledger."
                                }
                                buy_low_stock := WrapLabel{body: " " font_color: #x9A0E10 font_size: 14. visible: false}
                                FormCard{
                                    h_goods_received := WrapLabel{body: "Goods received" font_color: #x260B08 font_size: 17.}
                                    buy_item_input := FieldInput{empty_text: "Item name"}
                                    buy_item_suggest := mod.widgets.SuggestList{field: "buy_item"}
                                    FormFieldRow{
                                        buy_qty_input := FieldInput{empty_text: "Qty (e.g. 1.5 for kg)"}
                                        buy_unit_input := FieldInput{empty_text: "Unit (kg, litre, pcs)"}
                                    }
                                    buy_unit_suggest := mod.widgets.SuggestList{field: "buy_unit"}
                                    buy_cost_input := FieldInput{empty_text: "Unit cost"}
                                    buy_supplier_input := FieldInput{empty_text: "Supplier name (or party)"}
                                    buy_supplier_suggest := mod.widgets.SuggestList{field: "buy_supplier"}
                                    // The chips (or the default, Supplier) set it — never typed.
                                    View{
                                        width: Fill height: Fit
                                        visible: false
                                        buy_party_kind := FieldInput{empty_text: "New party kind (Supplier)"}
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
                                    ActionStack{
                                        buy_purchase := StackPrimaryBtn{text: "Record purchase"}
                                    }
                                }
                                FormCard{
                                    h_buy_expense := WrapLabel{body: "Expense" font_color: #x260B08 font_size: 17.}
                                    FormFieldRow{
                                        buy_expense_amount := FieldInput{empty_text: "Amount"}
                                        buy_expense_note := FieldInput{empty_text: "Note (fuel, rent, …)"}
                                    }
                                    // Paid in as many ways as it was: one row per way; a
                                    // blank amount takes what is left.
                                    buy_parts := mod.widgets.PartsList{}
                                    View{
                                        width: Fill height: Fit
                                        flow: Right spacing: 10
                                        align: Align{y: 0.5}
                                        buy_parts_status := Label{
                                            width: Fill
                                            text: ""
                                            draw_text.color: #x6B4A3F
                                            draw_text.text_style.font_size: 13.
                                        }
                                        buy_parts_add := Button{
                                            text: "+ Add another way"
                                            width: Fit height: 40
                                            padding: Inset{left: 14. right: 14. top: 6. bottom: 6.}
                                            draw_bg +: {
                                                color: #xffffff
                                                color_hover: #xFBE3D3
                                                color_down: #xEFD2BC
                                                color_focus: #xffffff
                                                border_color: #xD8BBA6
                                                border_radius: 10.
                                                border_size: 1.
                                            }
                                            draw_text +: {
                                                color: #x69090C
                                                color_hover: #x69090C
                                                color_down: #x69090C
                                                color_focus: #x69090C
                                                text_style +: {font_size: 13.}
                                            }
                                        }
                                    }
                                    ActionStack{
                                        buy_expense_add := StackPrimaryBtn{text: "Save expense"}
                                    }
                                    // The bill for the expense just saved (its note is the
                                    // Note above — no second note field).
                                    View{
                                        width: Fill height: Fit
                                        flow: Right spacing: 8
                                        expense_attach_photo := StackLightBtn{text: "Attach bill photo"}
                                        expense_attach_pdf := StackLightBtn{text: "Attach PDF"}
                                    }
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
                                // The day's sales first; the picked sale's details
                                // (copy, print, amend, void, refund, notes) under them.
                                sale_list := mod.widgets.SaleList{}
                                sale_detail := FormCard{
                                    visible: false
                                    sale_selected_label := WrapLabel{
                                        body: "No sale selected"
                                        font_color: #x260B08
                                        font_size: 13.
                                    }
                                    View{
                                        width: Fill height: Fit
                                        flow: Right spacing: 8
                                        receipt_share_btn := NavBtn{text: "Copy receipt"}
                                        receipt_whatsapp_btn := NavBtn{text: "Share WhatsApp"}
                                        receipt_print_btn := NavBtn{text: "Print / open"}
                                    }
                                    amend_cart_list := mod.widgets.CartList{}
                                    amend_natural_label := WrapLabel{
                                        body: "Actual Total 0"
                                        font_color: #x260B08
                                        font_size: 13.
                                    }
                                    amend_applied_label := WrapLabel{
                                        body: "Applied Total 0"
                                        font_color: #x260B08
                                        font_size: 13.
                                    }
                                        View{
                                            width: Fill height: Fit
                                            flow: Right spacing: 8
                                            align: Align{y: 0.5}
                                            h_rounded_total := WrapLabel{
                                                body: "Rounded Total"
                                                font_color: #xB43A12
                                                font_size: 14.
                                            }
                                            amend_total_input := CartField{
                                            padding: Inset{left: 14. right: 14. top: 14. bottom: 14.}
                                            empty_text: "0"
                                            text: "0"
                                            draw_bg +: {
                                                border_color: #xB43A12
                                                border_color_hover: #x822801
                                                border_color_focus: #x69090C
                                                border_radius: 12.
                                                border_size: 2.
                                            }
                                            draw_text +: {
                                                color: #x822801
                                                color_hover: #x822801
                                                color_down: #x822801
                                                color_focus: #x822801
                                                color_empty: #x6B4A3F
                                                color_empty_hover: #x260B08
                                                color_empty_focus: #x260B08
                                                text_style +: {font_size: 16.}
                                            }
                                        }
                                    }
                                    amend_round_hint := WrapLabel{
                                        body: "Rounded Total starts equal to Applied Total. Edit only to round the bill."
                                        font_color: #x260B08
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
                                        font_color: #x6B4A3F
                                        font_size: 12.
                                        visible: false
                                    }
                                    ActionStack{
                                        amend_btn := StackPrimaryBtn{text: "Amend"}
                                        void_btn := DangerBtn{text: "Void"}
                                        refund_btn := DangerBtn{text: "Refund"}
                                    }
                                    sale_history_label := mod.widgets.ReportText{}
                                    h_notes_attach_sale := WrapLabel{body: "Notes & attachments (selected sale)" font_color: #x260B08 font_size: 15.}
                                    sale_attach_note := FieldInput{empty_text: "Extra note on this bill / sale"}
                                    ActionStack{
                                        sale_attach_add_note := RowSecondaryBtn{text: "Add note"}
                                        sale_attach_photo := RowSecondaryBtn{text: "Attach photo"}
                                        sale_attach_pdf := RowSecondaryBtn{text: "Attach PDF"}
                                    }
                                    sale_attach_hint := WrapLabel{
                                        width: Fit
                                        body: "(select a sale)"
                                        font_color: #x6B4A3F
                                        font_size: 12.
                                    }
                                    sale_attach_list := mod.widgets.ReportText{}
                                }
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
                                    tip: "Each party has a unique id (names may repeat). A Customer runs a khata tab (owes us); a Supplier sells us stock (we owe them); Both does the two. Khata and payables stay separate; net who-owes-whom is in Reports."
                                }
                                credit_summary_label := WrapLabel{
                                    body: ""
                                    font_color: #x260B08
                                    font_size: 13.
                                }
                                credit_recent_list := mod.widgets.ReportText{}
                                // New party, or the picked one being edited.
                                FormCard{
                                    party_selected_label := WrapLabel{
                                        body: "New party — fill the form and tap Add party, or Pick a party from the list to edit it."
                                        font_color: #x9A0E10
                                        font_size: 13.
                                    }
                                    FormFieldRow{
                                        credit_add_name := FieldInput{empty_text: "Party name"}
                                        credit_add_phone := FieldInput{empty_text: "Phone"}
                                    }
                                    credit_add_birthday := FieldInput{empty_text: "Birthday (MM-DD or YYYY-MM-DD)"}
                                    h_account_kind := WrapLabel{
                                        body: "Account kind (tap one)"
                                        font_color: #x6B4A3F
                                        font_size: 12.
                                    }
                                    View{
                                        width: Fill height: Fit
                                        flow: Right spacing: 6
                                        View{width: Fill height: Fit flow: Overlay
                                            credit_kind_customer := SegBtn{text: "Customer"}
                                            credit_kind_customer_on := SegBtnOn{text: "Customer" visible: false}
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
                                    // Kept for the code that reads it; the chips (or the default,
                                    // Supplier) set it — never typed from memory.
                                    View{
                                        width: Fill height: Fit
                                        visible: false
                                        credit_add_kind := FieldInput{empty_text: "Roles (Customer, Supplier, Both…)"}
                                    }
                                    credit_add_btn := StackPrimaryBtn{text: "Add party"}
                                    View{
                                        width: Fill height: Fit
                                        flow: Right spacing: 8
                                        party_form_clear := RowSecondaryBtn{text: "New party (clear form)"}
                                        party_delete := DangerBtn{text: "Delete selected party"}
                                    }
                                }
                                // Local CRM: pick a guest segment, preview, queue SMS / WhatsApp
                                // intents (Net optional to deliver — never auto-blasts).
                                FormCard{
                                    crm_heading := WrapLabel{body: "Guests & messages" font_color: #x260B08 font_size: 15.}
                                    crm_help := WrapLabel{
                                        width: Fill
                                        body: ""
                                        font_color: #x6B4A3F
                                        font_size: 12.
                                        visible: false
                                    }
                                    // SegBtn is width:Fill — cannot sit in wrap:true (freezes UI).
                                    ActionStack{
                                        crm_seg_bday_week := SegBtn{text: "Birthday week"}
                                        crm_seg_bday_month := SegBtn{text: "Birthday month"}
                                        crm_seg_phone := SegBtn{text: "With phone"}
                                        crm_seg_loyalty := SegBtn{text: "Loyalty"}
                                        crm_seg_inactive := SegBtn{text: "Inactive"}
                                        crm_seg_top := SegBtn{text: "Top spenders"}
                                    }
                                    crm_preview := mod.widgets.ReportText{}
                                    crm_message_input := FieldInput{empty_text: "Happy birthday {name}! You have {points} points."}
                                    ActionStack{
                                        crm_queue_sms := StackPrimaryBtn{text: "Queue SMS"}
                                        crm_queue_wa := StackLightBtn{text: "Queue WhatsApp"}
                                    }
                                }
                                customer_list_credit := mod.widgets.CustomerList{}
                                // Everything about the picked party: limit, money in and
                                // out, write-offs, notes. Hidden until one is picked.
                                party_detail := View{
                                    width: Fill height: Fit
                                    flow: Down spacing: 10
                                    visible: false
                                    FormCard{
                                    h_credit_limit_heading := WrapLabel{body: "Credit limit" font_color: #x260B08 font_size: 15.}
                                    FormFieldRow{
                                        credit_limit_input := FieldInput{empty_text: "Credit limit (0 = none)"}
                                        View{
                                            width: Fill height: Fit
                                            visible: false
                                            credit_limit_party := FieldInput{empty_text: "Party name (or use selected)"}
                                        }
                                    }
                                    credit_party_suggest := mod.widgets.SuggestList{field: "credit_party"}
                                    ActionStack{
                                        credit_limit_set := RowSecondaryBtn{text: "Apply limit to party"}
                                    }
                                    }
                                    FormCard{
                                    h_pay_selected_heading := WrapLabel{body: "Pay selected (settle debt / take advance)" font_color: #x260B08 font_size: 15.}
                                    settle_amount_input := FieldInput{empty_text: "Amount received"}
                                    View{
                                        width: Fill height: Fit
                                        flow: Right spacing: 8
                                        settle_cash := NavBtn{text: "Pay cash"}
                                        settle_upi := NavBtn{text: "Pay UPI"}
                                        settle_card := NavBtn{text: "Pay card"}
                                    }
                                    settle_open_bills := mod.widgets.ReportText{}
                                    settle_bill_input := FieldInput{empty_text: "Against bill # above (optional)"}
                                    h_write_off_heading := WrapLabel{body: "Write off (clear without payment)" font_color: #x260B08 font_size: 15.}
                                    writeoff_amount_input := FieldInput{empty_text: "Amount (blank = full balance)"}
                                    writeoff_reason_input := FieldInput{empty_text: "Reason (required)"}
                                    credit_writeoff := NavBtn{text: "Write off / nil balance"}
                                    h_pay_ap_heading := WrapLabel{body: "Pay what we owe them (AP)" font_color: #x260B08 font_size: 15.}
                                    payable_amount_input := FieldInput{empty_text: "Amount we pay"}
                                    View{
                                        width: Fill height: Fit
                                        flow: Right spacing: 8
                                        payable_cash := NavBtn{text: "Pay AP cash"}
                                        payable_upi := NavBtn{text: "Pay AP UPI"}
                                        payable_card := NavBtn{text: "Pay AP card"}
                                    }
                                    payable_open_bills := mod.widgets.ReportText{}
                                    payable_bill_input := FieldInput{empty_text: "Against bill # above (optional)"}
                                    payable_writeoff := NavBtn{text: "Write off AP"}
                                    }
                                    FormCard{
                                    h_notes_attach_party := WrapLabel{body: "Notes & attachments" font_color: #x260B08 font_size: 15.}
                                    party_attach_note := FieldInput{empty_text: "Note / GSTIN / agreement reminder"}
                                    ActionStack{
                                        party_attach_add_note := RowSecondaryBtn{text: "Add note"}
                                        party_attach_photo := RowSecondaryBtn{text: "Attach photo"}
                                        party_attach_pdf := RowSecondaryBtn{text: "Attach PDF"}
                                    }
                                    party_attach_hint := WrapLabel{
                                        width: Fit
                                        body: "(select a party)"
                                        font_color: #x6B4A3F
                                        font_size: 12.
                                    }
                                    party_attach_list := mod.widgets.ReportText{}
                                    }
                                }
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
                                staff_title_input := FieldInput{empty_text: "Role title (e.g. Captain, Head cook)"}
                                staff_pin_input := FieldInput{empty_text: "Starting PIN (4–12 digits)"}
                                h_permission_preset := WrapLabel{
                                    body: "Permission preset"
                                    font_color: #x260B08
                                    font_size: 13.
                                }
                                h_preset_help_khaynjo := WrapLabel{
                                    width: Fill
                                    body: ""
                                    font_color: #x6B4A3F
                                    font_size: 12.
                                    visible: false
                                }
                                // A restaurant's people: who takes orders, who takes
                                // money, who cooks, who manages.
                                View{
                                    width: Fill height: Fit
                                    flow: Right spacing: 6
                                    View{width: Fill height: Fit flow: Overlay
                                        staff_role_waiter := SegBtn{text: "Waiter"}
                                        staff_role_waiter_on := SegBtnOn{text: "Waiter" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        staff_role_cashier := SegBtn{text: "Cashier"}
                                        staff_role_cashier_on := SegBtnOn{text: "Cashier" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        staff_role_kitchen := SegBtn{text: "Kitchen"}
                                        staff_role_kitchen_on := SegBtnOn{text: "Kitchen" visible: false}
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
                                h_optional_login_help := WrapLabel{
                                    body: ""
                                    font_color: #x6B4A3F
                                    font_size: 12.
                                    visible: false
                                }
                                staff_login_input := FieldInput{empty_text: "Login id (optional)"}
                                staff_password_input := FieldInput{empty_text: "Starting password (optional)"}
                                staff_set_login := RowSecondaryBtn{text: "Apply login to selected"}
                                }
                                FormCard{
                                h_feature_access_help := WrapLabel{
                                    body: "Feature access for selected staff"
                                    font_color: #x260B08
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
                                                feat_floor := ChipBtn{text: "Floor"}
                                                feat_floor_on := ChipBtnActive{text: "Floor" visible: false}
                                            }
                                            View{width: Fit height: 40 flow: Overlay
                                                feat_kitchen := ChipBtn{text: "Kitchen"}
                                                feat_kitchen_on := ChipBtnActive{text: "Kitchen" visible: false}
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
                                        chev_l := ChevronBtn{draw_bg.point: -1.0 visible: false}
                                    }
                                    View{
                                        width: Fill height: 52
                                        align: Align{x: 1.0 y: 0.5}
                                        chev_r := ChevronBtn{visible: false}
                                    }
                                }
                                // Pick who to edit (by name — names may repeat, so the
                                // first match is shown with their role to confirm).
                                team_pick_input := FieldInput{empty_text: "Staff name to edit"}
                                team_pick_suggest := mod.widgets.SuggestList{field: "team_pick"}
                                team_selected_label := WrapLabel{
                                    body: " "
                                    font_color: #x69090C
                                    font_size: 13.
                                }
                                h_rights_help := WrapLabel{
                                    body: ""
                                    font_color: #x260B08
                                    font_size: 13.
                                    visible: false
                                }
                                right_settle_btn := NavBtn{text: "Take payment"}
                                right_cancel_btn := NavBtn{text: "Cancel a sent item"}
                                right_nc_btn := NavBtn{text: "Complimentary (NC)"}
                                right_move_btn := NavBtn{text: "Move / merge tables"}
                                right_reprint_btn := NavBtn{text: "Print bill again"}
                                right_discount_btn := NavBtn{text: "Give a discount"}
                                feat_save := StackPrimaryBtn{text: "Save features"}
                                }
                                FormCard{
                                h_change_my_pin_heading := WrapLabel{
                                    body: "Change my PIN (only yours)"
                                    font_color: #x260B08
                                    font_size: 15.
                                }
                                my_pin_input := FieldInput{empty_text: "New PIN (4–12 digits)"}
                                change_my_pin := StackPrimaryBtn{text: "Update my PIN"}
                                my_password_input := FieldInput{empty_text: "New password (optional)"}
                                change_my_password := RowSecondaryBtn{text: "Update my password"}
                                }
                                // Owner-only. Emergency unlock PIN: looks like a
                                // normal login but clears trading history first.
                                FormCard{
                                h_duress_heading := WrapLabel{
                                    body: "Emergency unlock PIN (Owner only)"
                                    font_color: #x260B08
                                    font_size: 15.
                                }
                                h_duress_help := WrapLabel{
                                    body: ""
                                    font_color: #x69090C
                                    font_size: 12.
                                    visible: false
                                }
                                duress_status := WrapLabel{
                                    width: Fit
                                    body: " "
                                    font_color: #x69090C
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
                                settings_hub := View{
                                    width: Fill height: Fit
                                    flow: Down spacing: 8
                                    settings_hub_help := InfoHelp{
                                        width: Fit
                                        tip: "Open one area at a time — language, printers, bill format, backup, and more."
                                    }
                                    settings_hub_language := StackSecondaryBtn{text: "Language & currency"}
                                    settings_hub_country := StackSecondaryBtn{text: "Country & tax"}
                                    settings_hub_tills := StackSecondaryBtn{text: "Extra tills & product"}
                                    settings_hub_brand := StackSecondaryBtn{text: "Your brand"}
                                    settings_hub_units := StackSecondaryBtn{text: "Units & conversions"}
                                    settings_hub_pickers := StackSecondaryBtn{text: "Areas · stations · tax rates"}
                                    settings_hub_printers := StackSecondaryBtn{text: "Kitchen printing & printers"}
                                    settings_hub_bill := StackSecondaryBtn{text: "Bill format & schemes"}
                                    settings_hub_team := StackSecondaryBtn{text: "Team & PINs"}
                                    settings_hub_alerts := StackSecondaryBtn{text: "Alerts"}
                                    settings_hub_backup := StackSecondaryBtn{text: "Backup & restore"}
                                    settings_hub_diag := StackSecondaryBtn{text: "Diagnostics"}
                                    settings_hub_sidebar := StackSecondaryBtn{text: "Sidebar"}
                                }
                                settings_detail_back := StackLightBtn{text: "Back" visible: false}
                                settings_panel_language := View{
                                    width: Fill height: Fit
                                    flow: Down spacing: 12
                                    visible: false
                                    View{
                                        width: Fill height: Fit
                                        flow: Down spacing: 8
                                        settings_title := WrapLabel{body: "Language & currency" font_color: #x260B08 font_size: 20.}
                                    }
                                    locale_label := mod.widgets.ReportText{}
                                    h_language := WrapLabel{body: "Language" font_color: #x260B08 font_size: 15.}
                                    h_language_setup_hint := WrapLabel{
                                        width: Fill
                                        body: "Follow this device’s language, or pick one yourself."
                                        font_color: #x260B08
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
                                        font_color: #x260B08
                                        font_size: 13.
                                    }
                                    language_list := mod.widgets.LanguageList{}
                                    h_currency := WrapLabel{body: "Currency" font_color: #x260B08 font_size: 15.}
                                    h_currency_use_help := WrapLabel{
                                        width: Fill
                                        body: "Scroll and tap Use for any supported currency."
                                        font_color: #x260B08
                                        font_size: 12.
                                    }
                                    currency_list := mod.widgets.CurrencyList{}
                                }
                                settings_panel_country := View{
                                    width: Fill height: Fit
                                    flow: Down spacing: 12
                                    visible: false
                                    h_country_tax := WrapLabel{body: "Country & tax" font_color: #x260B08 font_size: 20.}
                                    h_country_pack_help := WrapLabel{
                                        width: Fill
                                        body: ""
                                        font_color: #x260B08
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
                                    }
                                    country_apply := StackPrimaryBtn{text: "Apply country pack"}
                                    country_hint := mod.widgets.ReportText{}
                                    }
                                    FormCard{
                                    // Where tax sits: one choice of two, the current one filled.
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
                                }
                                settings_panel_tills := View{
                                    width: Fill height: Fit
                                    flow: Down spacing: 12
                                    visible: false
View{
                                    width: Fill height: Fit
                                    flow: Right spacing: 6
                                    align: Align{y: 0.5}
                                    h_extra_tills_heading := WrapLabel{
                                        width: Fit
                                        body: "Extra tills"
                                        font_color: #x260B08
                                        font_size: 20.
                                    }
                                    server_pair_help := InfoHelp{
                                        width: Fit
                                        tip: "On the other device: Client Mode → Find server → enter this pairing code."
                                    }
                                }
                                h_other_devices_help_heading := WrapLabel{
                                    body: "Set up other devices"
                                    font_color: #x260B08
                                    font_size: 15.
                                    visible: false
                                }
                                h_other_devices_help := WrapLabel{
                                    width: Fill
                                    body: " "
                                    font_color: #x6B4A3F
                                    font_size: 12.
                                    visible: false
                                }
                                // Big digits stay visible even when firewall/listen notes fill the status line.
                                pairing_code_value := WrapLabel{
                                    width: Fit
                                    body: " "
                                    font_color: #x260B08
                                    font_size: 28.
                                    selectable: true
                                    visible: false
                                }
                                server_pair_label := WrapLabel{
                                    width: Fill
                                    body: " "
                                    font_color: #x6B4A3F
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
                                h_start_template := WrapLabel{body: "Product" font_color: #x260B08 font_size: 20.}
                                settings_tmpl_label := WrapLabel{
                                    body: "Khaynjo"
                                    font_color: #x6B4A3F
                                    font_size: 13.
                                }
                                h_product_only_khaynjo := WrapLabel{
                                    width: Fill
                                    body: ""
                                    font_color: #x6B4A3F
                                    font_size: 12.
                                    visible: false
                                }
                                                                }
                                settings_panel_brand := View{
                                    width: Fill height: Fit
                                    flow: Down spacing: 12
                                    visible: false
                                FormCard{
h_your_brand := WrapLabel{body: "Your brand (in-app)" font_color: #x260B08 font_size: 20.}
                                h_brand_shown_help := WrapLabel{
                                    width: Fill
                                    body: ""
                                    font_color: #x260B08
                                    font_size: 12.
                                    visible: false
                                }
                                venue_title_input := FieldInput{empty_text: "Name of this place"}
                                org_name_input := FieldInput{empty_text: "Business name"}
                                View{
                                    width: Fill height: Fit
                                    flow: Right spacing: 8
                                    align: Align{y: 0.5}
                                    h_venue_icon := WrapLabel{
                                                    width: Fit
                                        body: "App icon"
                                        font_color: #x260B08
                                        font_size: 15.
                                    }
                                    venue_icon_help := InfoHelp{
                                        width: Fit
                                        tip: "Shown on screens inside the app. The installed app name and icon on this device stay as the platform listing."
                                    }
                                }
                                venue_icon_status := WrapLabel{
                                    body: " "
                                    font_color: #x6B4A3F
                                    font_size: 12.
                                }
                                ActionStack{
                                    venue_icon_pick := RowSecondaryBtn{text: "Choose icon"}
                                    venue_icon_clear := RowSecondaryBtn{text: "Clear icon"}
                                }
                                venue_brand_save := StackPrimaryBtn{text: "Save brand"}
                                }
                                                                }
                                settings_panel_units := View{
                                    width: Fill height: Fit
                                    flow: Down spacing: 12
                                    visible: false
                                FormCard{
h_units_heading := WrapLabel{body: "Units" font_color: #x260B08 font_size: 20.}
                                h_units_help := WrapLabel{
                                    width: Fill
                                    body: ""
                                    font_color: #x6B4A3F
                                    font_size: 12.
                                    visible: false
                                }
                                h_units_picker_help := WrapLabel{
                                    width: Fill
                                    body: ""
                                    font_color: #x6B4A3F
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
                                                                }
                                settings_panel_pickers := View{
                                    width: Fill height: Fit
                                    flow: Down spacing: 12
                                    visible: false
sibling_pickers_heading := WrapLabel{body: "Areas · stations · tax rates" font_color: #x260B08 font_size: 18.}
                                sibling_pickers_help := WrapLabel{
                                    width: Fill
                                    body: ""
                                    font_color: #x6B4A3F
                                    font_size: 12.
                                    visible: false
                                }
                                sibling_pickers_label := mod.widgets.ReportText{}
                                FormCard{
                                station_manage_heading := WrapLabel{
                                    body: "Add or remove a kitchen station"
                                    font_color: #x260B08
                                    font_size: 15.
                                }
                                // Own ids (Kitchen screen already has station_*).
                                settings_station_name_input := FieldInput{empty_text: "Station name (e.g. Tandoor, Fryer, Cold)"}
                                settings_station_add := StackPrimaryBtn{text: "Add station"}
                                settings_station_remove_input := FieldInput{empty_text: "Station name to remove"}
                                settings_station_remove_suggest := mod.widgets.SuggestList{field: "settings_station_remove"}
                                settings_station_remove := StackSecondaryBtn{text: "Remove station"}
                                }
                                FormCard{
                                bulk_price_heading := WrapLabel{ width: Fit
                                                    body: "Bulk menu price" font_color: #x260B08 font_size: 18.}
                                FormFieldRow{
                                    bulk_cat_input := FieldInput{empty_text: "Category name"}
                                    bulk_pct_input := FieldInput{empty_text: "% change (e.g. 10 or -5)"}
                                    bulk_set_input := FieldInput{empty_text: "Or set ₹ amount"}
                                }
                                bulk_cat_suggest := mod.widgets.SuggestList{field: "bulk_cat"}
                                bulk_price_apply := RowSecondaryBtn{text: "Apply bulk price"}
                                }
                                settings_lan_help := InfoHelp{
                                    width: Fit
                                    tip: "On this shop’s Wi‑Fi, guest pages work for display board, tokens, expo, kiosk, table QR, and feedback. Guests on mobile data need Cloud later. Card terminal / payment link is optional when the internet is up — it never blocks the counter."
                                }
                                                                }
                                settings_panel_printers := View{
                                    width: Fill height: Fit
                                    flow: Down spacing: 12
                                    visible: false
                                    // Two steps only: (1) name the printers this
                                    // device can reach, (2) tap Bill/kitchen then
                                    // tap a printer. No route text field, no Up/Down.
                                    FormCard{
                                    h_kitchen_printing := WrapLabel{body: "Kitchen printing & printers" font_color: #x260B08 font_size: 20.}
                                    h_kot_print_help_khaynjo := WrapLabel{
                                        width: Fill
                                        body: ""
                                        font_color: #x6B4A3F
                                        font_size: 12.
                                        visible: false
                                    }
                                    View{
                                        width: Fill height: Fit
                                        flow: Right spacing: 6
                                        View{width: Fill height: Fit flow: Overlay
                                            kot_print_off := SegBtn{text: "Kitchen printing off"}
                                            kot_print_off_on := SegBtnOn{text: "Kitchen printing off" visible: false}
                                        }
                                        View{width: Fill height: Fit flow: Overlay
                                            kot_print_on := SegBtn{text: "Kitchen printing on"}
                                            kot_print_on_on := SegBtnOn{text: "Kitchen printing on" visible: false}
                                        }
                                    }
                                    h_printers_pool := WrapLabel{body: "1. Your printers" font_color: #x260B08 font_size: 16.}
                                    h_printers_help := WrapLabel{
                                        width: Fill
                                        body: ""
                                        font_color: #x6B4A3F
                                        font_size: 12.
                                        visible: false
                                    }
                                    find_printers_btn := StackSecondaryBtn{text: "Find printers"}
                                    printers_found_label := WrapLabel{
                                        width: Fill
                                        body: ""
                                        font_color: #x6B4A3F
                                        font_size: 12.
                                    }
                                    printers_found_list := mod.widgets.ReportText{}
                                    printers_pool_label := mod.widgets.ReportText{}
                                    printer_label_input := FieldInput{empty_text: "Name (e.g. Kitchen)"}
                                    printer_target_input := FieldInput{empty_text: "Address or system name"}
                                    printer_target_suggest := mod.widgets.SuggestList{field: "printer_target"}
                                    h_printer_wire := WrapLabel{
                                        width: Fill
                                        body: "How to send"
                                        font_color: #x6B4A3F
                                        font_size: 12.
                                    }
                                    View{
                                        width: Fill height: Fit
                                        flow: Right spacing: 6
                                        View{width: Fill height: Fit flow: Overlay
                                            printer_wire_escpos := SegBtn{text: "Thermal ESC/POS"}
                                            printer_wire_escpos_on := SegBtnOn{text: "Thermal ESC/POS" visible: false}
                                        }
                                        View{width: Fill height: Fit flow: Overlay
                                            printer_wire_os := SegBtn{text: "System print"}
                                            printer_wire_os_on := SegBtnOn{text: "System print" visible: false}
                                        }
                                    }
                                    ActionStack{
                                        printer_add := StackSecondaryBtn{text: "Add printer"}
                                        printer_remove := StackSecondaryBtn{text: "Remove"}
                                    }
                                    h_printers_routes := WrapLabel{body: "2. What prints where" font_color: #x260B08 font_size: 16.}
                                    h_printers_routes_help := WrapLabel{
                                        width: Fill
                                        body: ""
                                        font_color: #x6B4A3F
                                        font_size: 12.
                                        visible: false
                                    }
                                    printer_routes_label := mod.widgets.ReportText{}
                                    route_editing_label := WrapLabel{
                                        width: Fill
                                        body: ""
                                        font_color: #x260B08
                                        font_size: 13.
                                    }
                                    // Optional: type/pick Bill or a kitchen (list tap also works).
                                    route_pick_input := FieldInput{empty_text: "Bill or a kitchen station"}
                                    route_pick_suggest := mod.widgets.SuggestList{field: "route_pick"}
                                    route_add_printers := ActionStack{
                                        route_add_prn_0 := StackSecondaryBtn{text: "" visible: false}
                                        route_add_prn_1 := StackSecondaryBtn{text: "" visible: false}
                                        route_add_prn_2 := StackSecondaryBtn{text: "" visible: false}
                                        route_add_prn_3 := StackSecondaryBtn{text: "" visible: false}
                                        route_add_prn_4 := StackSecondaryBtn{text: "" visible: false}
                                        route_add_prn_5 := StackSecondaryBtn{text: "" visible: false}
                                        route_add_prn_6 := StackSecondaryBtn{text: "" visible: false}
                                        route_add_prn_7 := StackSecondaryBtn{text: "" visible: false}
                                    }
                                    ActionStack{
                                        route_remove := StackSecondaryBtn{text: "Remove last printer"}
                                        route_clear := StackSecondaryBtn{text: "Clear printers for this place"}
                                    }
                                    // Up/Down: reorder when more than one printer is on a route.
                                    route_up := StackSecondaryBtn{text: "Up" visible: false}
                                    route_down := StackSecondaryBtn{text: "Down" visible: false}
                                    ActionStack{
                                        printers_save := StackPrimaryBtn{text: "Save printers"}
                                        print_test_btn := StackSecondaryBtn{text: "Print a test page"}
                                    }
                                    // Legacy heading id still filled for RTL align.
                                    h_printers := WrapLabel{body: "" visible: false}
                                    }
                                                                }
                                settings_panel_bill := View{
                                    width: Fill height: Fit
                                    flow: Down spacing: 12
                                    visible: false
                                FormCard{
h_bill_format := WrapLabel{body: "Bill format" font_color: #x260B08 font_size: 20.}
                                h_bill_format_help := WrapLabel{
                                    width: Fill
                                    body: ""
                                    font_color: #x6B4A3F
                                    font_size: 12.
                                    visible: false
                                }
                                bill_header_input := FieldInput{empty_text: "Address | phone"}
                                fssai_input := FieldInput{empty_text: "FSSAI licence no. (India)"}
                                upi_vpa_input := FieldInput{empty_text: "UPI ID (VPA)"}
                                upi_payee_input := FieldInput{empty_text: "UPI payee name (optional)"}
                                bill_footer_input := FieldInput{empty_text: "Footer (e.g. Thank you, visit again)"}
                                View{
                                    width: Fill height: Fit
                                    flow: Right spacing: 8
                                    align: Align{y: 0.5}
                                    h_print_brand := WrapLabel{
                                                    width: Fit
                                                    body: "Print brand kit" font_color: #x260B08 font_size: 18.}
                                    print_brand_help := InfoHelp{
                                        width: Fit
                                        tip: "Optional stamp and payment QR images on bill faces and the LAN guest display. Dynamic UPI QR from the VPA above still works when no payment QR image is set."
                                    }
                                }
                                View{
                                    width: Fill height: Fit
                                    flow: Right spacing: 8
                                    align: Align{y: 0.5}
                                    h_stamp := WrapLabel{
                                                    width: Fit
                                                    body: "Stamp" font_color: #x260B08 font_size: 15.}
                                    print_stamp_help := InfoHelp{
                                        width: Fit
                                        tip: "Optional stamp image on printed bills and the guest display."
                                    }
                                }
                                print_stamp_status := WrapLabel{
                                    body: " "
                                    font_color: #x6B4A3F
                                    font_size: 12.
                                }
                                ActionStack{
                                    print_stamp_pick := RowSecondaryBtn{text: "Choose stamp"}
                                    print_stamp_clear := RowSecondaryBtn{text: "Clear stamp"}
                                }
                                View{
                                    width: Fill height: Fit
                                    flow: Right spacing: 8
                                    align: Align{y: 0.5}
                                    h_payment_qr := WrapLabel{
                                                    width: Fit
                                                    body: "Payment QR" font_color: #x260B08 font_size: 15.}
                                    payment_qr_image_help := InfoHelp{
                                        width: Fit
                                        tip: "Static QR on bills and guest display; otherwise paying uses a dynamic UPI QR from the VPA above."
                                    }
                                }
                                payment_qr_status := WrapLabel{
                                    body: " "
                                    font_color: #x6B4A3F
                                    font_size: 12.
                                }
                                ActionStack{
                                    payment_qr_pick := RowSecondaryBtn{text: "Choose payment QR"}
                                    payment_qr_clear := RowSecondaryBtn{text: "Clear payment QR"}
                                }
                                bill_format_save := StackPrimaryBtn{text: "Save bill format"}
                                }
                                FormCard{
                                View{
                                    width: Fill height: Fit
                                    flow: Right spacing: 8
                                    align: Align{x: 0.0 y: 0.5}
                                    h_aggregator_import := WrapLabel{
                                                    width: Fit
                                                    body: "Aggregator orders (file)" font_color: #x260B08 font_size: 18.}
                                    settings_aggregator_help := InfoHelp{
                                        width: Fit
                                        tip: "Optional: put import_aggregator_orders.json in the backups folder, then tap Import. Never blocks the counter; live delivery apps are Cloud."
                                    }
                                }
                                aggregator_import_btn := StackSecondaryBtn{text: "Import aggregator file"}
                                View{
                                    width: Fill height: Fit
                                    flow: Right spacing: 8
                                    align: Align{x: 0.0 y: 0.5}
                                    h_zero_sales := WrapLabel{
                                                    width: Fit
                                                    body: "Zero-value sales" font_color: #x260B08 font_size: 20.}
                                    settings_zero_sales_help := InfoHelp{
                                        width: Fit
                                        tip: "Allow dishes priced at zero and bills settled at zero (complimentary). On by default so free covers still appear in Frequent and day books."
                                    }
                                }
                                View{
                                    width: Fill height: Fit
                                    flow: Right spacing: 6
                                    View{width: Fill height: Fit flow: Overlay
                                        zero_sales_on_btn := SegBtn{text: "Allow ₹0"}
                                        zero_sales_on_btn_on := SegBtnOn{text: "Allow ₹0" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        zero_sales_off_btn := SegBtn{text: "Require price"}
                                        zero_sales_off_btn_on := SegBtnOn{text: "Require price" visible: false}
                                    }
                                }
                                }
                                FormCard{
                                View{
                                    width: Fill height: Fit
                                    flow: Right spacing: 8
                                    align: Align{x: 0.0 y: 0.5}
                                    h_schemes := WrapLabel{
                                                    width: Fit
                                                    body: "Schemes / offers" font_color: #x260B08 font_size: 20.}
                                    settings_schemes_help := InfoHelp{
                                        width: Fit
                                        tip: "Buy X get Y — applied automatically when you settle. Example: Buy 2 Thali get 1 Chaas."
                                    }
                                }
                                schemes_label := mod.widgets.ReportText{}
                                scheme_bogo_input := FieldInput{empty_text: "Buy 2 Thali get 1 Chaas"}
                                scheme_add := StackPrimaryBtn{text: "Add BOGO"}
                                }
                                                                }
                                settings_panel_alerts := View{
                                    width: Fill height: Fit
                                    flow: Down spacing: 12
                                    visible: false
                                    FormCard{
                                        h_alerts := WrapLabel{body: "Alerts" font_color: #x260B08 font_size: 20.}
                                        h_alerts_help := WrapLabel{
                                            width: Fill
                                            body: "How long status messages stay on screen for you (this login)."
                                            font_color: #x6B4A3F
                                            font_size: 12.
                                        }
                                        snack_secs_input := FieldInput{empty_text: "Seconds (3–60, default 10)"}
                                        snack_secs_save := StackPrimaryBtn{text: "Save alert time"}
                                    }
                                }
                                settings_panel_backup := View{
                                    width: Fill height: Fit
                                    flow: Down spacing: 12
                                    visible: false
h_backup := WrapLabel{body: "Backup" font_color: #x260B08 font_size: 20.}
                                backup_auto_label := WrapLabel{
                                    width: Fill
                                    body: ""
                                    font_color: #x6B4A3F
                                    font_size: 12.
                                }
                                backup_copy_btn := NavBtn{text: "Also copy backups to a folder…"}
                                db_path_label := WrapLabel{body: "" font_color: #x260B08}
                                audit_label := mod.widgets.ReportText{}
                                backup_btn := StackPrimaryBtn{text: "Create backup"}
                                backup_path_label := WrapLabel{body: "" font_color: #x260B08}
                                h_restore_help := WrapLabel{
                                    width: Fill
                                    body: "Restore: put restore.sqlite3 in the backups folder, then tap Restore."
                                    font_color: #x260B08
                                }
                                restore_btn := NavBtn{text: "Restore backup"}
                                }
                                settings_panel_diag := View{
                                    width: Fill height: Fit
                                    flow: Down spacing: 12
                                    visible: false
                                    View{
                                        width: Fill height: Fit
                                        flow: Right spacing: 8
                                        align: Align{y: 0.5}
                                        h_diag := WrapLabel{
                                                    width: Fit
                                                    body: "Diagnostics" font_color: #x260B08 font_size: 20.}
                                        diag_privacy_help := InfoHelp{
                                            width: Fit
                                            tip: "Logs stay on this device. Secrets (PIN, passwords) are never stored. Sharing a package off the device needs your explicit approval each time — never sold or given to third parties."
                                        }
                                    }
                                    diag_status_label := WrapLabel{
                                        width: Fill
                                        body: ""
                                        font_color: #x260B08
                                        font_size: 12.
                                    }
                                    h_diag_level := WrapLabel{body: "Detail level" font_color: #x260B08 font_size: 15.}
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
                                        diag_support_start := StackSecondaryBtn{text: "Start 30-min support session"}
                                        diag_support_stop := StackSecondaryBtn{text: "End support session"}
                                    }
                                    diag_export_local := StackSecondaryBtn{text: "Export local copy"}
                                    diag_send_help := InfoHelp{
                                        width: Fit
                                        tip: "Approval is one package, then expires. Temporary — used only to fix or improve the product. Not shared with third parties. Works on Linux, Windows, macOS, and Android."
                                    }
                                    diag_approve_send := StackPrimaryBtn{text: "Approve share off this device"}
                                    diag_export_send := StackSecondaryBtn{text: "Create share package (needs approval)"}
                                    diag_clear := NavBtn{text: "Clear diagnostic logs"}
                                }
                            }
                        }

                        sidebar_settings_screen := ScrollYView{
                            width: Fill height: Fill
                            visible: false
                            View{
                                width: Fill height: Fit
                                flow: Down spacing: 12
                                padding: Inset{left: 16. right: 16. top: 8. bottom: 120.}
                                View{
                                    width: Fill height: Fit
                                    flow: Right spacing: 10
                                    align: Align{y: 0.5}
                                    sidebar_settings_back := QuietFitBtn{text: "Back"}
                                }
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
                                    width: Fit
                                    tip: "Recording a day close saves this summary (and drawer variance if counted) to the audit log. It does not stop selling."
                                }
                                day_close_label := mod.widgets.ReportText{}
                                // Morning float, then drawer handovers between billers
                                // (BUSINESS_BOOKS.md § Shift and day-end cash).
                                h_opening_float := WrapLabel{body: "Opening cash (float)" font_color: #x260B08 font_size: 15.}
                                opening_float_input := FieldInput{empty_text: "Cash put in the drawer this morning"}
                                opening_float_save := StackPrimaryBtn{text: "Save opening cash"}
                                h_shift_close := WrapLabel{body: "Shift handover" font_color: #x260B08 font_size: 15.}
                                shift_so_far_label := mod.widgets.ReportText{}
                                shift_counted_input := FieldInput{empty_text: "Cash counted at handover"}
                                shift_close_btn := StackPrimaryBtn{text: "Close shift and print slip"}
                                h_drawer_count := WrapLabel{ width: Fit
                                                    body: "Drawer count (optional)" font_color: #x260B08 font_size: 15.}
                                day_close_counted_input := FieldInput{empty_text: "Counted cash in drawer"}
                                day_close_copy := NavBtn{text: "Copy day close"}
                                day_close_commit := StackPrimaryBtn{text: "Record day close"}
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
                                // Which report: one segmented row, not four big buttons.
                                reports_books_btns := View{
                                    width: Fill height: Fit
                                    flow: Right spacing: 6
                                    View{width: Fill height: Fit flow: Overlay
                                        reports_today := SegBtn{text: "Today books"}
                                        reports_today_on := SegBtnOn{text: "Today books" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        reports_month := SegBtn{text: "This month"}
                                        reports_month_on := SegBtnOn{text: "This month" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        reports_aging := SegBtn{text: "Outstanding / Dr-Cr"}
                                        reports_aging_on := SegBtnOn{text: "Outstanding / Dr-Cr" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        reports_balance_sheet := SegBtn{text: "Balance sheet"}
                                        reports_balance_sheet_on := SegBtnOn{text: "Balance sheet" visible: false}
                                    }
                                }
                                reports_label := mod.widgets.ReportText{}
                                expense_attach_hint := WrapLabel{
                                    body: "(log an expense first)"
                                    font_color: #x6B4A3F
                                    font_size: 12.
                                }
                                expense_attach_list := mod.widgets.ReportText{}
                                reports_bank_heading := WrapLabel{body: "Bank / cashbook" font_color: #x260B08 font_size: 15.}
                                bank_amount_input := FieldInput{empty_text: "Amount"}
                                bank_note_input := FieldInput{empty_text: "Note (deposit, withdrawal, …)"}
                                ActionStack{
                                    bank_till_to_bank := RowSecondaryBtn{text: "Till → bank"}
                                    bank_bank_to_till := RowSecondaryBtn{text: "Bank → till"}
                                    bank_in := RowSecondaryBtn{text: "Bank in"}
                                    bank_out := RowSecondaryBtn{text: "Bank out"}
                                }
                                reports_copy := NavBtn{text: "Copy report"}
                                h_bank_reconciliation := WrapLabel{body: "Bank reconciliation (tick off against your bank statement)" font_color: #x260B08 font_size: 15.}
                                bank_recon_label := mod.widgets.ReportText{}
                                bank_clear_input := FieldInput{empty_text: "Uncleared # above"}
                                ActionStack{
                                    bank_mark_cleared := RowSecondaryBtn{text: "Mark cleared"}
                                    bank_mark_uncleared := RowSecondaryBtn{text: "Unmark"}
                                }
                                h_spreadsheet_csv := WrapLabel{body: "Spreadsheet (CSV)" font_color: #x260B08 font_size: 15.}
                                ActionStack{
                                    csv_copy_day := RowSecondaryBtn{text: "Copy day book CSV"}
                                    csv_copy_outstanding := RowSecondaryBtn{text: "Copy outstanding CSV"}
                                }
                                ActionStack{
                                    csv_copy_parties := RowSecondaryBtn{text: "Copy parties CSV"}
                                    csv_copy_items := RowSecondaryBtn{text: "Copy items CSV"}
                                }
                                ActionStack{
                                    csv_copy_ledger := RowSecondaryBtn{text: "Copy party ledger CSV"}
                                    csv_import_parties := RowSecondaryBtn{text: "Import parties CSV…"}
                                }
                                csv_import_items := NavBtn{text: "Import items CSV…"}
                            }
                        }

                        // What the kitchen has: change stock in one card, then
                        // every stock item as a row (tap one to pick it), then the
                        // latest changes. Purchases are recorded on Buy.
                        inventory_screen := ScrollYView{
                            width: Fill height: Fill
                            visible: false
                            View{
                                width: Fill height: Fit
                                flow: Down spacing: 14
                                padding: Inset{left: 22. right: 22. top: 16. bottom: 120.}
                                inventory_help := InfoHelp{
                                    width: Fit
                                    tip: "Type the item name (part of it is enough), a quantity, then adjust. Items that are not counted yet need Start tracking once."
                                }
                                FormCard{
                                    inv_item_input := FieldInput{empty_text: "Item name"}
                                    inv_item_suggest := mod.widgets.SuggestList{field: "inv_item"}
                                    View{
                                        width: Fill height: Fit
                                        flow: Right spacing: 8
                                        View{
                                            width: 180 height: Fit
                                            inv_qty_input := FieldInput{empty_text: "Quantity"}
                                        }
                                        inv_note_input := FieldInput{empty_text: "Reason / note (wastage, count, …)"}
                                    }
                                    View{
                                        width: Fill height: Fit
                                        flow: Right spacing: 8
                                        inv_adjust_plus := StackPrimaryBtn{text: "+ Stock"}
                                        inv_adjust_minus := StackLightBtn{text: "− Stock"}
                                    }
                                    // When this item counts as low (Day book and Buy say so).
                                    View{
                                        width: Fill height: Fit
                                        flow: Right spacing: 8
                                        align: Align{y: 0.5}
                                        View{
                                            width: 180 height: Fit
                                            inv_reorder_input := FieldInput{empty_text: "Buy more at"}
                                        }
                                        inv_reorder_save := StackLightBtn{text: "Set reorder level"}
                                    }
                                }
                                stock_list := mod.widgets.StockList{}
                                stock_empty := WrapLabel{
                                    width: Fit
                                    visible: false
                                    body: " "
                                    font_color: #x6B4A3F
                                    font_size: 14.
                                }
                                inventory_label := mod.widgets.ReportText{}
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
                                // Business name is edited under Settings → Your brand
                                // (single-outlet shops never see this Branches tab).
                                branches_label := mod.widgets.ReportText{}
                                // A branch is its own restaurant (vision §5): open
                                // another in Server Mode, never switch inside these
                                // books. These controls stay hidden; export/import
                                // below compares separate restaurants.
                                View{
                                    width: Fill height: Fit
                                    flow: Down spacing: 10
                                    visible: false
                                    branch_name_input := FieldInput{empty_text: "New branch name"}
                                    branch_code_input := FieldInput{empty_text: "Short code (MG02)"}
                                    branch_add := StackPrimaryBtn{text: "Add branch"}
                                    branch_switch_input := FieldInput{empty_text: "Branch name to sell from"}
                                    branch_switch_suggest := mod.widgets.SuggestList{field: "branch_switch"}
                                    branch_switch := NavBtn{text: "Switch this device"}
                                }
                                h_branch_export_title := WrapLabel{
                                    body: "Outlet day transfer"
                                    font_color: #x260B08
                                    font_size: 15.
                                }
                                ActionStack{
                                    branch_export := RowSecondaryBtn{text: "Export day"}
                                    branch_import := RowSecondaryBtn{text: "Import day"}
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
                            draw_bg.color: #x69090C
                            draw_bg.border_radius: 12.
                            snack_label_info := WrapLabel{width: Fill body: "" font_color: #xffffff font_size: 13.}
                            snack_dismiss_info := Button{
                                text: "✕" width: 36 height: 36
                                draw_bg +: { color: #xB43A12 color_hover: #xB43A12 color_down: #x69090C color_focus: #xB43A12 color_2: #xB43A12 color_2_hover: #xB43A12 color_2_down: #x69090C color_2_focus: #xB43A12 border_radius: 10. border_size: 0. }
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
                                draw_text +: { color: #xffffff color_hover: #xffffff color_down: #xffffff color_focus: #xffffff text_style +: {font_size: 14.} }
                            }
                        }
                        snack_face_warn := RoundedView{
                            visible: false
                            width: Fill height: Fit
                            padding: Inset{left: 12. right: 8. top: 10. bottom: 10.}
                            margin: Inset{left: 8. right: 8. bottom: 4.}
                            flow: Right spacing: 8
                            align: Align{y: 0.5}
                            draw_bg.color: #xB43A12
                            draw_bg.border_radius: 12.
                            snack_label_warn := WrapLabel{width: Fill body: "" font_color: #xffffff font_size: 13.}
                            snack_dismiss_warn := Button{
                                text: "✕" width: 36 height: 36
                                draw_bg +: { color: #x69090C color_hover: #x69090C color_down: #x260B08 color_focus: #x69090C color_2: #x69090C color_2_hover: #x69090C color_2_down: #x260B08 color_2_focus: #x69090C border_radius: 10. border_size: 0. }
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
                        draw_bg.color: #x9A0E10
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
                                color_down: #xEFD2BC
                                color_focus: #xF7B043
                                color_2: #xF7B043
                                color_2_hover: #xffffff
                                color_2_down: #xEFD2BC
                                color_2_focus: #xF7B043
                                border_radius: 10.
                                border_size: 0.
                            }
                            draw_text +: {
                                color: #x69090C
                                color_hover: #x69090C
                                color_down: #x69090C
                                color_focus: #x69090C
                                text_style +: {font_size: 13.}
                            }
                        }
                        undo_snack_dismiss := Button{
                            text: "✕"
                            width: 36 height: 36
                            draw_bg +: {
                                color: #xB43A12
                                color_hover: #xB43A12
                                color_down: #x69090C
                                color_focus: #xB43A12
                                color_2: #xB43A12
                                color_2_hover: #xB43A12
                                color_2_down: #x69090C
                                color_2_focus: #xB43A12
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

                    // Phone chrome: all peers in one scroll row (Books · Order · Buy …).
                    // Selected peer = *Active face visible (paint_nav_tabs), not focus.
                    bottom_nav := SolidView{
                        width: Fill height: Fit
                        padding: Inset{left: 4. right: 4. top: 2. bottom: 6.}
                        draw_bg.color: #x69090C
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
                                        nav_sell := NavChip{text: "Order"}
                                        nav_sell_on := NavChipActive{text: "Order" visible: false}
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
                                        nav_buy := NavChip{text: "Buy"}
                                        nav_buy_on := NavChipActive{text: "Buy" visible: false}
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
                                    nav_menu_slot := View{
                                        width: Fit{min: FitBound.Abs(72.)} height: Fit
                                        flow: Overlay
                                        nav_menu := NavChip{text: "Menu"}
                                        nav_menu_on := NavChipActive{text: "Menu" visible: false}
                                        View{
                                            width: Fit height: 56
                                            align: Align{x: 0.0 y: 0.5}
                                            padding: Inset{left: 10. right: 0. top: 0. bottom: 0.}
                                            nav_menu_icon := Image{
                                                width: 22 height: 22
                                                fit: ImageFit.Stretch
                                                src: crate_resource("self:resources/nav/menu.webp")
                                            }
                                        }
                                    }
                                    nav_floor_slot := View{
                                        width: Fit{min: FitBound.Abs(72.)} height: Fit
                                        flow: Overlay
                                        nav_floor := NavChip{text: "Floor"}
                                        nav_floor_on := NavChipActive{text: "Floor" visible: false}
                                        View{
                                            width: Fit height: 56
                                            align: Align{x: 0.0 y: 0.5}
                                            padding: Inset{left: 10. right: 0. top: 0. bottom: 0.}
                                            nav_floor_icon := Image{
                                                width: 22 height: 22
                                                fit: ImageFit.Stretch
                                                src: crate_resource("self:resources/nav/floor.webp")
                                            }
                                        }
                                    }
                                    nav_kitchen_slot := View{
                                        width: Fit{min: FitBound.Abs(72.)} height: Fit
                                        flow: Overlay
                                        nav_kitchen := NavChip{text: "Kitchen"}
                                        nav_kitchen_on := NavChipActive{text: "Kitchen" visible: false}
                                        View{
                                            width: Fit height: 56
                                            align: Align{x: 0.0 y: 0.5}
                                            padding: Inset{left: 10. right: 0. top: 0. bottom: 0.}
                                            nav_kitchen_icon := Image{
                                                width: 22 height: 22
                                                fit: ImageFit.Stretch
                                                src: crate_resource("self:resources/nav/kitchen.webp")
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
                                    // Team is Settings → Team (not a chrome peer).
                                    nav_team_slot := View{
                                        visible: false
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
                                chev_l := ChevronBtn{draw_bg.point: -1.0 height: 46 visible: false}
                            }
                            View{
                                width: Fill height: Fill
                                align: Align{x: 1.0 y: 0.5}
                                chev_r := ChevronBtn{height: 46 visible: false}
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
                                draw_bg.color: #x9A0E10E6
                                draw_bg.border_radius: 12.
                                draw_bg.border_color: #xF2B183
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

                    // The order desk's menu: search, category tabs and the dishes as
                    // tiles (several per row), over the order. Add dishes opens it; it
                    // stays open while dishes are tapped in, Done / Escape / Back close it.
                    upi_qr_modal := Modal{
                        content +: {
                            width: Fill height: Fill
                            align: Align{x: 0.5 y: 0.5}
                            padding: Inset{left: 16. right: 16. top: 20. bottom: 20.}
                            RoundedView{
                                width: Fill{max: 420.} height: Fit
                                flow: Down spacing: 12
                                padding: Inset{left: 20. right: 20. top: 18. bottom: 18.}
                                draw_bg.color: #xF9F4F0
                                draw_bg.border_radius: 18.
                                View{
                                    width: Fill height: Fit
                                    flow: Right spacing: 8
                                    align: Align{x: 0.0 y: 0.5}
                                    upi_qr_title := WrapLabel{
                                                    width: Fit
                                                    body: "UPI" font_color: #x260B08 font_size: 20.}
                                    upi_qr_title_help := InfoHelp{
                                        width: Fit
                                        tip: "Guest scans and pays. Tap Take payment after they pay."
                                    }
                                }
                                upi_qr_amount := WrapLabel{body: " " font_color: #x69090C font_size: 18.}
                                upi_qr_image := Image{
                                    width: 240 height: 240
                                    fit: ImageFit.Smallest
                                }
                                upi_qr_help := WrapLabel{
                                    width: Fill
                                    body: "Guest scans and pays. Tap Take payment after they pay."
                                    font_color: #x6B4A3F
                                    font_size: 12.
                                    visible: false
                                }
                                upi_qr_close := StackPrimaryBtn{text: "Close"}
                            }
                        }
                    }

                    menu_modal := Modal{
                        content +: {
                            width: Fill height: Fill
                            align: Align{x: 0.5 y: 0.5}
                            padding: Inset{left: 16. right: 16. top: 20. bottom: 20.}
                            menu_panel := RoundedView{
                                width: Fill{max: 980.} height: Fill
                                flow: Down spacing: 10
                                padding: Inset{left: 16. right: 16. top: 14. bottom: 16.}
                                draw_bg.color: #xF9F4F0
                                draw_bg.border_radius: 18.
                                View{
                                    width: Fill height: Fit
                                    flow: Right spacing: 10
                                    align: Align{y: 0.5}
                                    View{
                                        width: Fill height: Fit
                                        flow: Down spacing: 2
                                        menu_modal_title := Label{
                                            width: Fill
                                            text: "Add dishes"
                                            draw_text.color: #x260B08
                                            draw_text.text_style.font_size: 18.
                                        }
                                        // How the order stands, so nobody leaves the window to check.
                                        menu_modal_summary := Label{
                                            width: Fill
                                            text: " "
                                            draw_text.color: #x6B4A3F
                                            draw_text.text_style.font_size: 13.
                                        }
                                    }
                                    menu_pick_done := RowSecondaryBtn{text: "Done" width: 120}
                                }
                                // Name, alias or short code (aliases double as short codes,
                                // e.g. "101"); typing looks through every category.
                                menu_search_input := FieldInput{
                                    empty_text: "Search item or short code"
                                    draw_text +: {text_style +: {font_size: 15.}}
                                }
                                // Categories as one scrollable strip: All, Frequent,
                                // then the shop's own, in its order.
                                cat_strip := View{
                                    width: Fill height: Fit
                                    flow: Overlay
                                    tabs := ChipStrip{
                                        ChipStripRow{
                                            View{width: Fit height: 40 flow: Overlay
                                                cat_all := ChipBtn{text: "All items"}
                                                cat_all_on := ChipBtnActive{text: "All items" visible: false}
                                            }
                                            View{width: Fit height: 40 flow: Overlay
                                                cat_frequent := ChipBtn{text: "Frequent"}
                                                cat_frequent_on := ChipBtnActive{text: "Frequent" visible: false}
                                            }
                                            View{width: Fit height: 40 flow: Overlay
                                                cat_0 := ChipBtn{text: " "}
                                                cat_0_on := ChipBtnActive{text: " " visible: false}
                                            }
                                            View{width: Fit height: 40 flow: Overlay
                                                cat_1 := ChipBtn{text: " "}
                                                cat_1_on := ChipBtnActive{text: " " visible: false}
                                            }
                                            View{width: Fit height: 40 flow: Overlay
                                                cat_2 := ChipBtn{text: " "}
                                                cat_2_on := ChipBtnActive{text: " " visible: false}
                                            }
                                            View{width: Fit height: 40 flow: Overlay
                                                cat_3 := ChipBtn{text: " "}
                                                cat_3_on := ChipBtnActive{text: " " visible: false}
                                            }
                                            View{width: Fit height: 40 flow: Overlay
                                                cat_4 := ChipBtn{text: " "}
                                                cat_4_on := ChipBtnActive{text: " " visible: false}
                                            }
                                            View{width: Fit height: 40 flow: Overlay
                                                cat_5 := ChipBtn{text: " "}
                                                cat_5_on := ChipBtnActive{text: " " visible: false}
                                            }
                                            View{width: Fit height: 40 flow: Overlay
                                                cat_6 := ChipBtn{text: " "}
                                                cat_6_on := ChipBtnActive{text: " " visible: false}
                                            }
                                            View{width: Fit height: 40 flow: Overlay
                                                cat_7 := ChipBtn{text: " "}
                                                cat_7_on := ChipBtnActive{text: " " visible: false}
                                            }
                                            View{width: Fit height: 40 flow: Overlay
                                                cat_8 := ChipBtn{text: " "}
                                                cat_8_on := ChipBtnActive{text: " " visible: false}
                                            }
                                            View{width: Fit height: 40 flow: Overlay
                                                cat_9 := ChipBtn{text: " "}
                                                cat_9_on := ChipBtnActive{text: " " visible: false}
                                            }
                                            View{width: Fit height: 40 flow: Overlay
                                                cat_10 := ChipBtn{text: " "}
                                                cat_10_on := ChipBtnActive{text: " " visible: false}
                                            }
                                            View{width: Fit height: 40 flow: Overlay
                                                cat_11 := ChipBtn{text: " "}
                                                cat_11_on := ChipBtnActive{text: " " visible: false}
                                            }
                                            View{width: Fit height: 40 flow: Overlay
                                                cat_12 := ChipBtn{text: " "}
                                                cat_12_on := ChipBtnActive{text: " " visible: false}
                                            }
                                            View{width: Fit height: 40 flow: Overlay
                                                cat_13 := ChipBtn{text: " "}
                                                cat_13_on := ChipBtnActive{text: " " visible: false}
                                            }
                                            View{width: Fit height: 40 flow: Overlay
                                                cat_14 := ChipBtn{text: " "}
                                                cat_14_on := ChipBtnActive{text: " " visible: false}
                                            }
                                            View{width: Fit height: 40 flow: Overlay
                                                cat_15 := ChipBtn{text: " "}
                                                cat_15_on := ChipBtnActive{text: " " visible: false}
                                            }
                                            View{width: Fit height: 40 flow: Overlay
                                                cat_16 := ChipBtn{text: " "}
                                                cat_16_on := ChipBtnActive{text: " " visible: false}
                                            }
                                            View{width: Fit height: 40 flow: Overlay
                                                cat_17 := ChipBtn{text: " "}
                                                cat_17_on := ChipBtnActive{text: " " visible: false}
                                            }
                                            View{width: Fit height: 40 flow: Overlay
                                                cat_18 := ChipBtn{text: " "}
                                                cat_18_on := ChipBtnActive{text: " " visible: false}
                                            }
                                            View{width: Fit height: 40 flow: Overlay
                                                cat_19 := ChipBtn{text: " "}
                                                cat_19_on := ChipBtnActive{text: " " visible: false}
                                            }
                                            View{width: Fit height: 40 flow: Overlay
                                                cat_20 := ChipBtn{text: " "}
                                                cat_20_on := ChipBtnActive{text: " " visible: false}
                                            }
                                            View{width: Fit height: 40 flow: Overlay
                                                cat_21 := ChipBtn{text: " "}
                                                cat_21_on := ChipBtnActive{text: " " visible: false}
                                            }
                                            View{width: Fit height: 40 flow: Overlay
                                                cat_22 := ChipBtn{text: " "}
                                                cat_22_on := ChipBtnActive{text: " " visible: false}
                                            }
                                            View{width: Fit height: 40 flow: Overlay
                                                cat_23 := ChipBtn{text: " "}
                                                cat_23_on := ChipBtnActive{text: " " visible: false}
                                            }
                                        }
                                    }
                                    View{
                                        width: Fill height: 52
                                        align: Align{x: 0.0 y: 0.5}
                                        chev_l := ChevronBtn{draw_bg.point: -1.0 visible: false}
                                    }
                                    View{
                                        width: Fill height: 52
                                        align: Align{x: 1.0 y: 0.5}
                                        chev_r := ChevronBtn{visible: false}
                                    }
                                }
                                // Shown only on the Frequent tab — period + how many.
                                // Apply sits under the fields (ActionStack): Stack*
                                // beside Fit inputs collapses to a vertical letter stack.
                                frequent_config := View{
                                    visible: false
                                    width: Fill height: Fit
                                    flow: Down spacing: 6
                                    frequent_config_help := WrapLabel{
                                        width: Fill
                                        body: "Most sold dishes. Change how many days to look at and how many to show."
                                        font_color: #x6B4A3F
                                        font_size: 12.
                                    }
                                    View{
                                        width: Fill height: Fit
                                        flow: Right spacing: 8
                                        align: Align{y: 0.5}
                                        frequent_days_input := FieldInput{
                                            width: Fill
                                            empty_text: "Days (1)"
                                        }
                                        frequent_limit_input := FieldInput{
                                            width: Fill
                                            empty_text: "Items (24)"
                                        }
                                    }
                                    ActionStack{
                                        frequent_config_save := StackLightBtn{text: "Apply"}
                                    }
                                }
                                menu_no_match := Label{
                                    visible: false
                                    width: Fill
                                    text: " "
                                    draw_text.color: #x6B4A3F
                                    draw_text.text_style.font_size: 14.
                                }
                                product_list := mod.widgets.ProductList{height: Fill}
                            }
                        }
                    }

                    // Host for every InfoHelp ℹ tip (hover / tap).
                    app_callout := CalloutTooltip{}
                }
            }
        }
    }
}
