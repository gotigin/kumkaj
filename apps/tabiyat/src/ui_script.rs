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
        // CJK / icons / emoji live under this app's resources/fonts (desktop
        // packages ship only that tree — not makepad_widgets/resources).
        chinese := FontMember{res: crate_resource("self:resources/fonts/LXGWWenKaiRegular.ttf") asc: 0.0 desc: 0.0}
        icons := FontMember{res: crate_resource("self:resources/fonts/fa-solid-900.ttf") asc: 0.0 desc: 0.0}
        emoji := FontMember{res: crate_resource("self:resources/fonts/NotoColorEmoji.ttf") asc: 0.0 desc: 0.0}
    }
    // Card names/prices: Bold (one step under Black) + Gujarati Bold.
    let WorldFontsBold = FontFamily{
        latin := FontMember{res: crate_resource("self:resources/fonts/NotoSans-Bold.ttf") asc: -0.1 desc: 0.0}
        devanagari := FontMember{res: crate_resource("self:resources/fonts/NotoSansDevanagari-Regular.ttf") asc: 0.0 desc: 0.0}
        bengali := FontMember{res: crate_resource("self:resources/fonts/NotoSansBengali-Regular.ttf") asc: 0.0 desc: 0.0}
        tamil := FontMember{res: crate_resource("self:resources/fonts/NotoSansTamil-Regular.ttf") asc: 0.0 desc: 0.0}
        telugu := FontMember{res: crate_resource("self:resources/fonts/NotoSansTelugu-Regular.ttf") asc: 0.0 desc: 0.0}
        gujarati := FontMember{res: crate_resource("self:resources/fonts/NotoSansGujarati-Bold.ttf") asc: 0.0 desc: 0.0}
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
        chinese := FontMember{res: crate_resource("self:resources/fonts/LXGWWenKaiBold.ttf") asc: 0.0 desc: 0.0}
        icons := FontMember{res: crate_resource("self:resources/fonts/fa-solid-900.ttf") asc: 0.0 desc: 0.0}
        emoji := FontMember{res: crate_resource("self:resources/fonts/NotoColorEmoji.ttf") asc: 0.0 desc: 0.0}
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
    // Markdown / TextFlow / Html copy theme faces at class-def time (before
    // WorldFonts). Pin their text_style_* so shaping never sees an empty family.
    mod.widgets.Markdown.text_style_normal.font_family = WorldFonts
    mod.widgets.Markdown.text_style_italic.font_family = WorldFonts
    mod.widgets.Markdown.text_style_bold.font_family = WorldFontsBold
    mod.widgets.Markdown.text_style_bold_italic.font_family = WorldFontsBold
    mod.widgets.TextFlow.text_style_normal.font_family = WorldFonts
    mod.widgets.TextFlow.text_style_italic.font_family = WorldFonts
    mod.widgets.TextFlow.text_style_bold.font_family = WorldFontsBold
    mod.widgets.TextFlow.text_style_bold_italic.font_family = WorldFontsBold
    mod.widgets.Html.text_style_normal.font_family = WorldFonts
    mod.widgets.Html.text_style_italic.font_family = WorldFonts
    mod.widgets.Html.text_style_bold.font_family = WorldFontsBold
    mod.widgets.Html.text_style_bold_italic.font_family = WorldFontsBold
    // Contrast law: never pair two lights or two darks — theme OR local styles.
    // Palette from apps/tabiyat/assets/tabiyat-app-icon.png (teal pharmacy cross,
    // leaf green, cream — sampled with ImageMagick). See docs/DESIGN.md
    // § Color tokens — Tabiyat.
    // Light face → dark ink (#031303 / #1C460E) in EVERY state.
    // Orange chrome (#CD380E) → white ink in EVERY state.
    mod.theme.color_bg_app = #xFBFCFC
    mod.theme.color_app_caption_bar = #x023637
    mod.theme.color_text = #x023637
    mod.theme.color_text_hover = #x023637
    mod.theme.color_text_focus = #x023637
    mod.theme.color_text_down = #x023637
    mod.theme.color_text_hl = #x023637
    mod.theme.color_text_disabled = #x56575D
    mod.theme.color_text_val = #x074D3E
    mod.theme.color_text_meta = #x56575D
    mod.theme.color_label_inner = #x023637
    mod.theme.color_label_inner_hover = #x023637
    mod.theme.color_label_inner_down = #x023637
    mod.theme.color_label_inner_focus = #x023637
    mod.theme.color_label_inner_active = #x023637
    mod.theme.color_text_placeholder = #x56575D
    mod.theme.color_text_placeholder_hover = #x56575D
    mod.theme.color_text_cursor = #x074D3E
    mod.theme.color_cursor = #x074D3E
    mod.theme.color_cursor_focus = #x074D3E
    mod.theme.color_cursor_empty = #x074D3E
    mod.theme.color_cursor_border = #x074D3E

    // Stock TextInput caret + placeholder colors are baked from theme at
    // widget-def time (often white on gray wells). Re-bind after our theme so
    // EVERY TextInput shows a visible caret and readable placeholders.
    // Do NOT use set_type_default() here (Android Animator UAF).
    mod.widgets.TextInputFlat = mod.widgets.TextInputFlat{
        blink_speed: 0.5
        draw_cursor +: {
            color: #x025B60
        }
        draw_text +: {
            color: #x023637
            color_hover: #x023637
            color_focus: #x023637
            color_down: #x023637
            color_empty: #x56575D
            color_empty_hover: #x56575D
            color_empty_focus: #x56575D
        }
    }
    mod.widgets.TextInput = mod.widgets.TextInput{
        blink_speed: 0.5
        draw_cursor +: {
            color: #x025B60
        }
        draw_text +: {
            color: #x023637
            color_hover: #x023637
            color_focus: #x023637
            color_down: #x023637
            color_empty: #x56575D
            color_empty_hover: #x56575D
            color_empty_focus: #x56575D
        }
    }
    mod.theme.color_icon = #x023637
    mod.theme.color_icon_active = #x023637
    mod.theme.color_icon_inactive = #x56575D
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
    // Dark ink (#x023637) stays readable on this band.
    mod.theme.color_selection = #x37BE28FF
    mod.theme.color_selection_hover = #x37BE28FF
    mod.theme.color_selection_down = #xE8890CFF
    mod.theme.color_selection_focus = #x37BE28FF
    mod.theme.color_selection_empty = #x37BE28FF
    mod.theme.color_selection_disabled = #xEFD2BC

    // Default Label ink. Keep stock flow — forcing Down drops glyphs on Android GLES.
    mod.widgets.Label = set_type_default() do mod.widgets.Label{
        height: Fit
        draw_text +: {
            color: #x023637
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
            color_hover: #x023637
            color_down: #x025B60
            color_focus: #xffffff
            color_disabled: #xEFD2BC
            color_2: #xffffff
            color_2_hover: #x023637
            color_2_down: #x025B60
            color_2_focus: #xffffff
            color_2_disabled: #xEFD2BC
            border_color: #xD8BBA6
            border_color_hover: #x023637
            border_color_down: #x025B60
            border_color_focus: #xD8BBA6
            border_radius: 12.
            border_size: 1.
        }
        draw_text +: {
            color: #x023637
            color_hover: #xffffff
            color_down: #xffffff
            color_focus: #x023637
            color_disabled: #x56575D
            // Icon glyphs (✕ ‹ › − + ℹ FA) sit on the baseline without this —
            // every face inherits it; left-aligned hubs may still set false.
            ink_centered: true
        }
    }

    // NEVER call set_type_default on TextInput — it rebuilds Animator apply
    // objects and Android then panics with GenVec use-after-free on NextFrame,
    // freezing the whole UI (PIN keypad included). Style fields via theme +
    // per-instance blink_speed/padding only (see FieldInput).

    // Makepad 2 ScrollYView: wheel/trackpad + mouse/touch press-drag + keys.
    // Pages swipe-scroll on touch; a selectable WrapLabel takes its own drag.
    // The page's scrollbar must be seen: the stock handle colour is almost
    // the page colour. A thin handle that darkens on hover / drag.
    // Right padding clears the bar (bar_size 10 + gap) so inputs never sit
    // flush against the scrollbar.
    mod.widgets.ScrollYView = mod.widgets.ViewBase {
        padding: Inset{right: 16.}
        scroll_bars: mod.widgets.ScrollBars {
            show_scroll_x: false show_scroll_y: true
            scroll_bar_y: mod.widgets.ScrollBar{
                drag_scrolling: true
                // Press-drag over cart / buttons / filled panes (not only
                // whitespace). Nested PortalList still claims via nested_touch.
                mouse_capture_overload: true
                // A drag that runs sideways (over a sideways strip or the
                // Floor's tables) is not the page's: direction decides who
                // scrolls, never a 2px jitter (patch scroll_axis_lock).
                axis_lock: true
                bounce_at_start: false
                bounce_at_end: false
                bar_size: 10.
                min_handle_size: 40.
                draw_bg +: {
                    size: 6.
                    border_radius: 3.
                    color: #xC9AFA2
                    color_hover: #x023637
                    color_drag: #x025B60
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
    // chip. Desk panes that fill the strip set mouse_capture_overload.
    mod.widgets.ScrollXView = mod.widgets.ViewBase {
        scroll_bars: mod.widgets.ScrollBars {
            show_scroll_x: true show_scroll_y: false
            scroll_bar_x.drag_scrolling: true
            scroll_bar_x.axis_lock: true
            scroll_bar_x.bar_size: 4.
        }
    }

    // Custom window chrome (Wayland / Windows): stock Makepad uses black
    // min/max/close glyphs on a light caption — invisible on our navy UI.
    mod.widgets.Window = set_type_default() do mod.widgets.Window{
        pass.clear_color: #xFBFCFC
        caption_bar +: {
            draw_bg.color: #x023637
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
                        bg_color_hover: #x017B85
                        bg_color_down: #x025B60
                    }
                }
                max +: {
                    draw_bg +: {
                        color: #xffffff
                        color_hover: #xffffff
                        color_down: #xEBD5C2
                        bg_color: #00000000
                        bg_color_hover: #x017B85
                        bg_color_down: #x025B60
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
            border_color_hover: #x017B85
            border_color_down: #x023637
            border_color_focus: #xD8BBA6
            border_radius: 10.
            border_size: 1.
        }
        draw_text +: {
            color: #x025B60
            color_hover: #x025B60
            color_down: #x025B60
            color_focus: #x025B60
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
            color: #x023637
            color_hover: #x017B85
            color_down: #x025B60
            color_focus: #x023637
            color_2: #x023637
            color_2_hover: #x017B85
            color_2_down: #x025B60
            color_2_focus: #x023637
            border_color: #x023637
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
            color_hover: #x023637
            color_down: #x025B60
            color_focus: #x00000000
            color_disabled: #x00000000
            color_2: #x00000000
            color_2_hover: #x023637
            color_2_down: #x025B60
            color_2_focus: #x00000000
            border_color: #x00000000
            border_radius: 12.
            border_size: 0.
        }
        draw_text +: {
            color: #xA2CFCF
            color_hover: #xffffff
            color_down: #xffffff
            color_focus: #xA2CFCF
            color_disabled: #x56575D
            text_style +: {font_size: 13.}
        }
    }
    // Hide / show the whole side rail (tablet/desktop chrome only).
    // Drawn ›/‹ (not a glyph): glyphs sit on the text baseline and read low
    // inside the hover pill. `draw_bg.point`: 1 = ›, -1 = ‹.
    let RailToggleBtn = Button{
        width: 36 height: 36
        align: Align{x: 0.5 y: 0.5}
        grab_key_focus: false
        reset_hover_on_click: true
        margin: Inset{top: 0. right: 0. bottom: 0. left: 0.}
        padding: Inset{left: 0. right: 0. top: 0. bottom: 0.}
        text: ""
        label_walk: Walk{width: 0. height: 0.}
        icon_walk: Walk{width: 0. height: 0.}
        draw_bg +: {
            point: uniform(1.0)
            mark_color: uniform(#xA2CFCF)
            color: #x00000000
            color_hover: #x023637
            color_down: #x025B60
            color_focus: #x00000000
            color_disabled: #x00000000
            color_2: #x00000000
            color_2_hover: #x023637
            color_2_down: #x025B60
            color_2_focus: #x00000000
            border_color: #x00000000
            border_radius: 8.
            border_size: 0.
            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                sdf.box(0., 0., self.rect_size.x, self.rect_size.y, self.border_radius)
                let fill = self.color
                    .mix(self.color_hover, self.hover)
                    .mix(self.color_down, self.down)
                sdf.fill(fill)
                let c = self.rect_size * vec2(0.5, 0.5)
                let w = 4.0 * self.point
                let h = 7.5
                sdf.move_to(c.x - w, c.y - h)
                sdf.line_to(c.x + w, c.y)
                sdf.line_to(c.x - w, c.y + h)
                let mark = self.mark_color
                    .mix(#xffffff, self.hover)
                    .mix(#xffffff, self.down)
                sdf.stroke(mark, 2.4)
                return sdf.result
            }
        }
    }
    // One line + scroll (no wrap). Equal vertical pad sandwiches the label so
    // it lines up with the overlay icon; horizontal pad unchanged.
    // Fit min/max must be FitBound objects — bare numbers only work on Fill{min:}.
    let NavChip = Button{
        width: Fit{min: FitBound.Abs(96.)} height: Fit
        align: Align{x: 0.0 y: 0.5}
        flow: Right
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
            color_hover: #x023637
            color_down: #x025B60
            color_focus: #x00000000
            color_disabled: #x00000000
            color_2: #x00000000
            color_2_hover: #x023637
            color_2_down: #x025B60
            color_2_focus: #x00000000
            border_color: #x00000000
            border_radius: 12.
            border_size: 0.
        }
        draw_text +: {
            color: #xA2CFCF
            color_hover: #xffffff
            color_down: #xffffff
            color_focus: #xA2CFCF
            color_disabled: #x56575D
            text_style +: {font_size: 12.}
        }
    }
    let NavChipActive = Button{
        width: Fit{min: FitBound.Abs(96.)} height: Fit
        align: Align{x: 0.0 y: 0.5}
        flow: Right
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
            color: #x017B85
            color_hover: #x017B85
            color_down: #x023637
            color_focus: #x017B85
            color_disabled: #xEFD2BC
            color_2: #x017B85
            color_2_hover: #x017B85
            color_2_down: #x023637
            color_2_focus: #x017B85
            border_color: #x00000000
            border_radius: 12.
            border_size: 0.
        }
        draw_text +: {
            color: #xffffff
            color_hover: #xffffff
            color_down: #xffffff
            color_focus: #xffffff
            color_disabled: #x56575D
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
            color: #x023637
            color_hover: #x023637
            color_down: #x023637
            color_focus: #x023637
            color_disabled: #x023637
            color_2: #x023637
            color_2_hover: #x023637
            color_2_down: #x023637
            color_2_focus: #x023637
            color_2_disabled: #x023637
            border_color: #x023637
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
            color: #x017B85
            color_hover: #x023637
            color_down: #x025B60
            color_focus: #x017B85
            color_disabled: #xA9806E
            color_2: #x017B85
            color_2_hover: #x023637
            color_2_down: #x025B60
            color_2_focus: #x017B85
            color_2_disabled: #xA9806E
            border_color: #xA2CFCF
            border_color_hover: #xA2CFCF
            border_color_down: #xEFD2BC
            border_color_focus: #xA2CFCF
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
            color: #x023637
            color_hover: #x017B85
            color_down: #x025B60
            color_focus: #x023637
            color_2: #x023637
            color_2_hover: #x017B85
            color_2_down: #x025B60
            color_2_focus: #x023637
            border_color: #x023637
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
    // Prefer ActionStack when labels are longer than ~4 chars at the 375px floor.
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
            color: #x023637
            color_hover: #x017B85
            color_down: #x025B60
            color_focus: #x023637
            color_2: #x023637
            color_2_hover: #x017B85
            color_2_down: #x025B60
            color_2_focus: #x023637
            border_color: #x023637
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
            border_color_hover: #x017B85
            border_color_down: #x023637
            border_color_focus: #xD8BBA6
            border_radius: 8.
            border_size: 1.
        }
        draw_text +: {
            color: #x025B60
            color_hover: #x025B60
            color_down: #x025B60
            color_focus: #x025B60
            color_disabled: #xA9806E
            ink_centered: true
            text_style +: {font_size: 14.}
        }
    }
    // Icon-only eye next to a money figure (or the screen-level header eye).
    // FA glyph (eye ↔ lock via set_text); ink_centered keeps it in the face.
    let EyeIconBtn = Button{
        width: 28 height: 28
        margin: Inset{left: 0. right: 0. top: 0. bottom: 0.}
        padding: Inset{left: 0. right: 0. top: 0. bottom: 0.}
        align: Align{x: 0.5 y: 0.5}
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
            color: #x025B60
            color_hover: #x025B60
            color_down: #x025B60
            color_focus: #x025B60
            ink_centered: true
            text_style +: {font_size: 14.}
        }
        text: "\u{f06e}"
    }
    // Books / setup / floor accordion header — FA chevron (›/˅) + title.
    // Glyph comes from WorldFonts `icons` (fa-solid); Noto has no ▸/▾.
    // Alone (Books): Fill — one painted face. With a tip (Device setup /
    // Floor): Fit caption + Fit InfoHelp + Fill `_ext` as AccordionHdrHitBtn
    // (invisible face) so the row reads as one header, ℹ hugs the title, and
    // the whole card still toggles (DEV 7c) — never three painted segments.
    // label_walk Fit (bug 7).
    // Wrap — long / translated section titles must not crop (bug 3).
    let AccordionHdrBtn = Button{
        width: Fill height: Fit
        margin: Inset{left: 0. right: 0. top: 0. bottom: 0.}
        // right: 0 — tip_btn.margin.left is the only title→ℹ gap.
        padding: Inset{left: 6. right: 0. top: 10. bottom: 10.}
        align: Align{x: 0.0 y: 0.5}
        flow: Right{wrap: true}
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
            color: #x025B60
            color_hover: #x3D0507
            color_down: #x3D0507
            color_focus: #x025B60
            ink_centered: true
            text_style +: {font_size: 14.}
        }
        text: "TODAY"
    }
    // Tip-row accordion hit: same metrics/text as AccordionHdrBtn, but never
    // paints a hover slab — Fit caption + Fit InfoHelp + Fill `_ext`.
    // **No wrap** — Fit+wrap expands the caption and parks ℹ at the far end
    // (LANGUAGE / THIS DEVICE, 2026-11). Alone Fill headers use AccordionHdrBtn
    // (wrap OK).
    let AccordionHdrHitBtn = Button{
        width: Fill height: Fit
        margin: Inset{left: 0. right: 0. top: 0. bottom: 0.}
        padding: Inset{left: 6. right: 0. top: 10. bottom: 10.}
        align: Align{x: 0.0 y: 0.5}
        // Right, no wrap — wrap at Fit parks ℹ at the far end.
        flow: Right
        grab_key_focus: false
        reset_hover_on_click: true
        label_walk: Walk{width: Fit height: Fit margin: Inset{left: 0. right: 0.}}
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
            border_radius: 8.
            border_size: 0.
        }
        draw_text +: {
            color: #x025B60
            color_hover: #x3D0507
            color_down: #x3D0507
            color_focus: #x025B60
            ink_centered: true
            text_style +: {font_size: 14.}
        }
        text: ""
    }
    // InfoHelp DSL lives after WrapLabel (title is WrapLabel — Label+Fill+wrap
    // NaNs the turtle; see DEV bug 7). Register the type here so the widget
    // exists; set_type_default runs once WrapLabel is in scope.
    mod.widgets.InfoHelpBase = #(InfoHelp::register_widget(vm))
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
            border_color_hover: #x017B85
            border_color_down: #x023637
            border_color_focus: #xD8BBA6
            border_radius: 8.
            border_size: 1.
        }
        draw_text +: {
            color: #x025B60
            color_hover: #x025B60
            color_down: #x025B60
            color_focus: #x025B60
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
            border_color: #x023637
            border_color_hover: #x025B60
            border_color_down: #x4A0608
            border_color_focus: #x023637
            border_radius: 8.
            border_size: 1.5
        }
        draw_text +: {
            color: #x023637
            color_hover: #x025B60
            color_down: #x4A0608
            color_focus: #x023637
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
            color: #x023637
            color_hover: #x023637
            color_down: #x023637
            color_focus: #x023637
            ink_centered: false
            text_style +: {font_size: 15.}
        }
    }
    // Prefer ActionStack for CTAs. ActionRow only for 2 ultra-short labels
    // (≤4 chars) — Fill NavBtn/PayBtn in a Right row becomes "..." near the 375px floor.
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
    // Equal Fill fields only. Max two FieldInputs — a third long placeholder
    // wraps and makes the row look stair-stepped on Settings widths.
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
            color: #x023637
            color_hover: #x017B85
            color_down: #x025B60
            color_focus: #x023637
            color_2: #x023637
            color_2_hover: #x017B85
            color_2_down: #x025B60
            color_2_focus: #x023637
            border_color: #x023637
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
            color_hover: #x023637
            color_down: #x025B60
            color_focus: #xEFD2BC
            color_2: #xEFD2BC
            color_2_hover: #x023637
            color_2_down: #x025B60
            color_2_focus: #xEFD2BC
            border_color: #xD8BBA6
            border_radius: 8.
            border_size: 1.
        }
        draw_text +: {
            color: #x023637
            color_hover: #xffffff
            color_down: #xffffff
            color_focus: #x023637
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
            color: #x025B60
            color_hover: #x025B60
            color_down: #x025B60
            color_focus: #x025B60
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
            border_color: #x023637
            border_radius: 22.
            border_size: 1.5
        }
        draw_text +: {
            color: #x023637
            color_hover: #x023637
            color_down: #x023637
            color_focus: #x023637
            ink_centered: true
            text_style +: {font_size: 13.}
        }
    }
    // Full-width settings row. A real button, so a tap selects that row.
    let KotRowBtn = Button{
        width: Fill height: Fit
        margin: Inset{left: 0. right: 0. top: 0. bottom: 0.}
        padding: Inset{left: 12. right: 12. top: 8. bottom: 8.}
        align: Align{x: 0.0 y: 0.5}
        flow: Right
        label_walk: Walk{width: Fit height: Fit}
        icon_walk: Walk{width: 0. height: 0.}
        draw_bg +: {
            color: #xffffff
            color_hover: #xFFF6EE
            color_down: #xFBE3D3
            color_focus: #xffffff
            color_2: #xffffff
            color_2_hover: #xFFF6EE
            color_2_down: #xFBE3D3
            color_2_focus: #xffffff
            border_color: #xE3D3C8
            border_radius: 10.
            border_size: 1.
        }
        draw_text +: {
            color: #x023637
            color_hover: #x023637
            color_down: #x023637
            color_focus: #x023637
            ink_centered: false
            text_style +: {font_size: 14.}
        }
    }
    let KotRowBtnOn = KotRowBtn{
        draw_bg +: {
            color: #x023637
            color_hover: #x017B85
            color_down: #x025B60
            color_focus: #x023637
            color_2: #x023637
            color_2_hover: #x017B85
            color_2_down: #x025B60
            color_2_focus: #x023637
            border_color: #x023637
        }
        draw_text +: {
            color: #xffffff
            color_hover: #xffffff
            color_down: #xffffff
            color_focus: #xffffff
        }
    }

    let SegBtnOn = SegBtn{
        draw_bg +: {
            color: #x023637
            color_hover: #x017B85
            color_down: #x025B60
            color_focus: #x023637
            color_2: #x023637
            color_2_hover: #x017B85
            color_2_down: #x025B60
            color_2_focus: #x023637
            border_color: #x023637
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
            color: #x56575D
            color_hover: #x023637
            color_down: #x023637
            color_focus: #x56575D
            ink_centered: true
            text_style +: {font_size: 14.}
        }
    }
    let DeskTabOn = DeskTab{
        draw_text +: {
            color: #x023637
            color_hover: #x023637
            color_down: #x025B60
            color_focus: #x023637
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
            border_color: #x023637
            border_radius: 12.
            border_size: 1.5
        }
        draw_text +: {
            color: #x025B60
            color_hover: #x025B60
            color_down: #x025B60
            color_focus: #x025B60
            text_style +: {font_size: 15.}
        }
    }

    // FILTER CHIP — Fit width in a ChipStrip (scroll sideways for long bilingual
    // names). Soft tag, not DeskTab / SegBtn. Never cap/ellipsis — full label.
    // NEVER use for multi-word CTAs — ActionStack + StackPrimary/Secondary.
    let ChipBtn = Button{
        width: Fit height: 34
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
            color: #x074D3E
            color_hover: #x023637
            color_down: #x023637
            color_focus: #x074D3E
            ink_centered: true
            max_lines: 1
            text_style +: {font_size: 12.}
        }
    }
    let ChipBtnActive = Button{
        width: Fit height: 34
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
            color: #x023637
            color_hover: #x017B85
            color_down: #x025B60
            color_focus: #x023637
            color_2: #x023637
            color_2_hover: #x017B85
            color_2_down: #x025B60
            color_2_focus: #x023637
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
            mark_color: uniform(#x025B60)
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

    // Modifier pager ‹/› — same chrome as RoleChip, drawn mark (not glyph).
    let RoleChevronBtn = Button{
        width: 36 height: 40
        margin: Inset{left: 0. right: 0. top: 0. bottom: 0.}
        padding: Inset{left: 0. right: 0. top: 0. bottom: 0.}
        text: ""
        label_walk: Walk{width: 0. height: 0.}
        icon_walk: Walk{width: 0. height: 0.}
        draw_bg +: {
            point: uniform(1.0)
            mark_color: uniform(#x023637)
            color: #xEFD2BC
            color_hover: #x023637
            color_down: #x025B60
            color_focus: #xEFD2BC
            border_color: #xD8BBA6
            border_radius: 10.
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
                let w = 4.0 * self.point
                let h = 7.5
                sdf.move_to(c.x - w, c.y - h)
                sdf.line_to(c.x + w, c.y)
                sdf.line_to(c.x - w, c.y + h)
                let mark = self.mark_color
                    .mix(#xffffff, self.hover)
                    .mix(#xffffff, self.down)
                sdf.stroke(mark, 2.4)
                return sdf.result
            }
        }
    }

    // Icon-only ✕ — drawn, never a glyph (glyphs sit low in the face).
    // Override face/mark colours per call site; keep text empty.
    let CloseIconBtn = Button{
        width: 36 height: 36
        padding: Inset{left: 0. right: 0. top: 0. bottom: 0.}
        align: Align{x: 0.5 y: 0.5}
        text: ""
        label_walk: Walk{width: 0. height: 0.}
        icon_walk: Walk{width: 0. height: 0.}
        draw_bg +: {
            mark_color: uniform(#xffffff)
            color: #x017B85
            color_hover: #x023637
            color_down: #x025B60
            color_focus: #x017B85
            color_2: #x017B85
            color_2_hover: #x023637
            color_2_down: #x025B60
            color_2_focus: #x017B85
            border_color: #x00000000
            border_radius: 10.
            border_size: 0.
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
                // Always stroke the face path so it is consumed — skipping this
                // when border_size is 0 left the box SDF active and snack
                // dismisses drew a white ring with no ✕.
                sdf.fill_keep(fill)
                sdf.stroke(self.border_color, self.border_size)
                let c = self.rect_size * vec2(0.5, 0.5)
                let side = if self.rect_size.x < self.rect_size.y self.rect_size.x else self.rect_size.y
                let s = side * 0.22
                let mark = self.mark_color
                sdf.move_to(c.x - s, c.y - s)
                sdf.line_to(c.x + s, c.y + s)
                sdf.stroke(mark, 2.4)
                sdf.move_to(c.x + s, c.y - s)
                sdf.line_to(c.x - s, c.y + s)
                sdf.stroke(mark, 2.4)
                return sdf.result
            }
        }
    }

    let ChipStrip = ScrollXView{
        width: Fill height: 52
    }

    // Fill chips in a row: wrap (no ellipsis) so translated labels stay whole.
    let RoleChip = Button{
        width: Fill height: Fit
        margin: Inset{left: 0. right: 0. top: 0. bottom: 0.}
        padding: Inset{left: 10. right: 10. top: 8. bottom: 8.}
        align: Align{x: 0.5 y: 0.5}
        flow: Right{wrap: true}
        label_walk: Walk{
            width: Fit
            height: Fit
            margin: Inset{left: 2. right: 2.}
        }
        icon_walk: Walk{width: 0. height: 0.}
        draw_bg +: {
            color: #xEFD2BC
            color_hover: #x023637
            color_down: #x025B60
            color_focus: #xEFD2BC
            color_2: #xEFD2BC
            color_2_hover: #x023637
            color_2_down: #x025B60
            color_2_focus: #xEFD2BC
            border_color: #xD8BBA6
            border_radius: 10.
            border_size: 1.
        }
        draw_text +: {
            color: #x023637
            color_hover: #xffffff
            color_down: #xffffff
            color_focus: #x023637
            ink_centered: true
            text_style +: {font_size: 13.}
        }
    }

    let RoleChipActive = Button{
        width: Fill height: Fit
        margin: Inset{left: 0. right: 0. top: 0. bottom: 0.}
        padding: Inset{left: 10. right: 10. top: 8. bottom: 8.}
        align: Align{x: 0.5 y: 0.5}
        flow: Right{wrap: true}
        label_walk: Walk{
            width: Fit
            height: Fit
            margin: Inset{left: 2. right: 2.}
        }
        icon_walk: Walk{width: 0. height: 0.}
        draw_bg +: {
            color: #x023637
            color_hover: #x017B85
            color_down: #x025B60
            color_focus: #x023637
            color_2: #x023637
            color_2_hover: #x017B85
            color_2_down: #x025B60
            color_2_focus: #x023637
            border_color: #x023637
            border_radius: 10.
            border_size: 0.
        }
        draw_text +: {
            color: #xffffff
            color_hover: #xffffff
            color_down: #xffffff
            color_focus: #xffffff
            ink_centered: true
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
        // Fixed height so FormFieldRow peers stay level when one field is
        // focused (float caption) or a long placeholder would otherwise wrap.
        height: 56.
        // Extra vertical margin so scroll gestures miss the field (fewer
        // accidental keyboard opens mid-scroll).
        margin: Inset{left: 0. right: 0. top: 6. bottom: 6.}
        padding: Inset{left: 14. right: 14. top: 16. bottom: 12.}
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
            border_color_hover: #x023637
            border_color_focus: #x023637
            border_radius: 12.
            border_size: 1.5
        }
        draw_cursor +: {
            color: #x025B60
        }
        // Flat opaque selection — color_2 must match (no light gradient wash).
        draw_selection +: {
            gradient_fill_horizontal: 0.0
            color: #x37BE28FF
            color_hover: #x37BE28FF
            color_focus: #x37BE28FF
            color_down: #xE8890CFF
            color_empty: #x37BE28FF
            color_2: #x37BE28FF
            color_2_hover: #x37BE28FF
            color_2_focus: #x37BE28FF
            color_2_down: #xE8890CFF
            color_2_empty: #x37BE28FF
        }
        draw_text +: {
            // Typed ink stays #x023637; placeholders stay muted even on
            // hover/focus so empty fields never look pre-filled.
            color: #x023637
            color_hover: #x023637
            color_focus: #x023637
            color_down: #x023637
            color_empty: #x56575D
            color_empty_hover: #x56575D
            color_empty_focus: #x56575D
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
            border_color_hover: #x025B60
            border_color_focus: #x025B60
            border_radius: 10.
            border_size: 2.
        }
        draw_text +: {
            color: #x023637
            color_empty: #x56575D
            color_empty_hover: #x56575D
            color_empty_focus: #x56575D
            color_hover: #x023637
            color_down: #x023637
            color_focus: #x023637
            text_style +: {font_size: 15.}
        }
    }

    // Full-width Markdown TextFlow with real `visible` support — stock
    // Markdown/TextFlow ignore set_visible. Use `body:`.
    // Default: selectable (DESIGN.md § Text selection & copy). Set
    // `selectable: false` only inside tap targets (cards, nav, snack,
    // button faces) — select-drag there steals the press / page swipe.
    mod.widgets.WrapLabelBase = #(WrapLabel::register_widget(vm))
    mod.widgets.WrapLabel = set_type_default() do mod.widgets.WrapLabelBase{
        width: Fill
        height: Fit
        padding: 0.
        selectable: true
        paragraph_spacing: 0.
        pre_code_spacing: 0.
        font_size: 14.
        font_color: #x023637
        // Markdown copies theme faces before WorldFonts is assigned — pin every
        // text_style so shaping never sees an empty FontFamily (console WARNING).
        draw_text +: {
            color: #x023637
            text_style.font_family: WorldFonts
        }
        text_style_normal.font_family: WorldFonts
        text_style_italic.font_family: WorldFonts
        text_style_bold.font_family: WorldFontsBold
        text_style_bold_italic.font_family: WorldFontsBold
        draw_block +: {
            // Opaque mid-blue (RRGGBBAA) — readable dark ink on top, clear on white page.
            selection_color: #x37BE28FF
        }
    }
    let WrapLabel = mod.widgets.WrapLabel{}

    // Screen title + ℹ tip (CalloutTooltip via app_callout). Docs stay off the
    // work surface — tap/hover the icon when the tip is useful.
    // Titled: Fit title + tip + Fill trail — ℹ hugs the label on every screen
    // (never parks at the far end). Tip-only: trail hidden, walk Fit.
    // Never Label{wrap} (DEV bug 7). Parent rows beside InfoHelp use spacing: 0.
    mod.widgets.InfoHelp = set_type_default() do mod.widgets.InfoHelpBase{
        width: Fill
        height: Fit
        flow: Right
        spacing: 0
        align: Align{x: 0.0 y: 0.0}
        tip: ""
        icon := Image{
            width: 28 height: 28
            fit: ImageFit.Stretch
            visible: false
            margin: Inset{left: 0. right: 6. top: 0. bottom: 0.}
        }
        title := WrapLabel{
            width: Fit
            height: Fit
            body: ""
            // Hidden until set_title — empty title must not take layout.
            visible: false
            // Display title — selectable WrapLabel steals Android page swipe.
            selectable: false
            font_size: 20.
            font_color: #x023637
            margin: Inset{left: 0. right: 0. top: 0. bottom: 0.}
        }
        tip_btn := Button{
            width: 32 height: 32
            text: ""
            margin: Inset{left: 6. right: 0. top: 0. bottom: 0.}
            padding: Inset{left: 0. right: 0. top: 0. bottom: 0.}
            align: Align{x: 0.5 y: 0.5}
            label_walk: Walk{width: 0. height: 0.}
            icon_walk: Walk{width: 0. height: 0.}
            draw_bg +: {
                mark_color: uniform(#x025B60)
                color: #xFBEDE4
                color_hover: #x023637
                color_down: #x025B60
                color_focus: #xFBEDE4
                color_2: #xFBEDE4
                color_2_hover: #x023637
                color_2_down: #x025B60
                color_2_focus: #xFBEDE4
                border_color: #xEFD2BC
                border_radius: 16.
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
                    let ink = self.mark_color
                        .mix(#xffffff, self.hover)
                        .mix(#ffffff, self.down)
                    // Dot + stem ≈ ℹ, centred in the face.
                    sdf.circle(c.x, c.y - 5.5, 1.6)
                    sdf.fill(ink)
                    sdf.move_to(c.x, c.y - 2.0)
                    sdf.line_to(c.x, c.y + 6.0)
                    sdf.stroke(ink, 2.2)
                    return sdf.result
                }
            }
        }

        // Absorbs leftover row width so Fit title+ℹ stay packed on the left.
        trail := View{width: Fill height: Fit visible: false}
    }
    let InfoHelp = mod.widgets.InfoHelp{}

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
            color_hover: #x023637
            color_down: #x025B60
            color_focus: #xffffff
            color_disabled: #xF5EBE4
            color_2: #xffffff
            color_2_hover: #x023637
            color_2_down: #x025B60
            color_2_focus: #xffffff
            border_color: #xD8BBA6
            border_radius: 12.
            border_size: 1.
        }
        draw_text +: {
            color: #x023637
            color_hover: #xffffff
            color_down: #xffffff
            color_focus: #x023637
            color_disabled: #xA08070
            text_style +: {font_size: 13.}
        }
    }
    // Transparent hit over the pill only — exception to “whole row toggles”
    // (DEV 7c): the row face still drag-reorders; a full-row switch hit would
    // fight that.
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
                                draw_bg.color: #x023637
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
        draw_bg.color: #x023637
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
    // − / + on dish cards. Drawn mark (`draw_bg.mark`: 3 = +, 4 = −) — glyphs
    // sit low in the circle even with ink_centered.
    let CardStepBtn = Button{
        width: 34 height: 34
        padding: Inset{left: 0. right: 0. top: 0. bottom: 0.}
        align: Align{x: 0.5 y: 0.5}
        text: ""
        label_walk: Walk{width: 0. height: 0.}
        icon_walk: Walk{width: 0. height: 0.}
        draw_bg +: {
            mark: uniform(3.0)
            mark_color: uniform(#xffffff)
            color: #x023637
            color_hover: #x017B85
            color_down: #x025B60
            color_focus: #x023637
            color_2: #x023637
            color_2_hover: #x017B85
            color_2_down: #x025B60
            color_2_focus: #x023637
            border_color: #x023637
            border_color_hover: #x017B85
            border_color_down: #x025B60
            border_color_focus: #x023637
            border_radius: 17.
            border_size: 0.
            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                sdf.box(0., 0., self.rect_size.x, self.rect_size.y, self.border_radius)
                let fill = self.color
                    .mix(self.color_hover, self.hover)
                    .mix(self.color_down, self.down)
                sdf.fill(fill)
                let c = self.rect_size * vec2(0.5, 0.5)
                let s = 7.0
                sdf.move_to(c.x - s, c.y)
                sdf.line_to(c.x + s, c.y)
                if self.mark < 3.5 {
                    sdf.move_to(c.x, c.y - s)
                    sdf.line_to(c.x, c.y + s)
                }
                sdf.stroke(self.mark_color, 2.4)
                return sdf.result
            }
        }
    }

    // One dish in the menu window. The photo is the whole card (cropped to
    // fill, rounded with the card); a dark wash at the bottom carries the
    // name and, always on the same baseline, the price. No photo: a red
    // card with the dish's initials. A tap anywhere on the card adds one;
    // once the dish is on the order the − n + pill shows at the top and the
    // card wears a gold edge. A sold-out dish is dimmed. Layers are stacked and shown by state,
    // because a DSL colour cannot be restyled at runtime.
    // Experiment: one continuous card — left photo, right name/alias/price.
    // Dish tile: photo | text. Fixed 176 row (Fill+Fill collapsed the grid to
    // blank cards). −/qty/+ centered on photo. Name/alias ≤2 lines + Ellipsis,
    // natural LTR align, smaller type.
    let ProductCard = View{
        // Fixed height. Fit + Overlay children collapse while the grid
        // scrolls, and each row draws as a thick horizontal bar.
        width: Fill height: 176
        tile := View{
            cursor: MouseCursor.Hand
            width: Fill height: 176
            flow: Overlay
            RoundedView{
                width: Fill height: Fill
                draw_bg.color: #xFFF8F3
                draw_bg.color_2: #xF3E3DA
                draw_bg.border_radius: 16.
            }
            View{
                width: Fill height: 176
                flow: Right spacing: 0
                View{
                    width: Fill height: Fill
                    flow: Overlay
                    RoundedView{
                        width: Fill height: Fill
                        padding: Inset{top: 16.}
                        align: Align{x: 0.5 y: 0.}
                        draw_bg.color: #x017B85
                        draw_bg.color_2: #x025B60
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
                    View{
                        width: Fill height: Fill
                        align: Align{x: 0.5 y: 0.5}
                        stepper := RoundedView{
                            visible: false
                            width: Fit height: Fit
                            padding: Inset{left: 2. right: 2. top: 2. bottom: 2.}
                            flow: Right
                            align: Align{y: 0.5}
                            draw_bg.color: #x023637
                            draw_bg.border_radius: 19.
                            draw_bg.border_size: 1.5
                            draw_bg.border_color: #x37BE28
                            minus := CardStepBtn{draw_bg.mark: 4.0}
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
                            plus := CardStepBtn{draw_bg.mark: 3.0}
                        }
                    }
                }
                // Name/alias top; price pinned to the bottom so every card
                // lines up (centering the whole cluster put prices at different Y).
                View{
                    width: Fill height: Fill
                    flow: Down
                    padding: Inset{left: 12. right: 12. top: 10. bottom: 10.}
                    View{
                        width: Fill height: Fill
                        flow: Down spacing: 2
                        name := Label{
                            width: Fill
                            text: "Item"
                            flow: Right{wrap: true}
                            max_lines: 2
                            text_overflow: Ellipsis
                            draw_text.color: #x023637
                            draw_text.text_style: theme.font_bold{font_size: 13.}
                        }
                        alias := Label{
                            visible: false
                            width: Fill
                            text: ""
                            flow: Right{wrap: true}
                            max_lines: 2
                            text_overflow: Ellipsis
                            draw_text.color: #x074D3E
                            draw_text.text_style: theme.font_bold{font_size: 12.}
                        }
                    }
                    price := Label{
                        width: Fill
                        text: "0"
                        flow: Right{wrap: false}
                        max_lines: 1
                        draw_text.color: #x023637
                        draw_text.text_style: theme.font_bold{font_size: 14.}
                    }
                    price_off := Label{
                        visible: false
                        width: Fill
                        text: ""
                        flow: Right{wrap: false}
                        max_lines: 1
                        draw_text.color: #x074D3E
                        draw_text.text_style: theme.font_bold{font_size: 14.}
                    }
                }
            }
            dim := RoundedView{
                visible: false
                width: Fill height: Fill
                draw_bg.color: #x1A0505A6
                draw_bg.border_radius: 16.
            }
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
                draw_bg.border_color: #x37BE28
            }
        }
    }


        // Floor tab: one card per dining table — colour = visit state.
    // Icon actions only (no "Seat guests" text). Focus = stronger border.
    let FloorTileIconBtn = Button{
        width: 36 height: 32
        padding: Inset{left: 0. right: 0. top: 0. bottom: 0.}
        align: Align{x: 0.5 y: 0.5}
        grab_key_focus: false
        reset_hover_on_click: true
        text: ""
        label_walk: Walk{width: Fit height: Fit margin: Inset{left: 0. right: 0.}}
        icon_walk: Walk{width: 0. height: 0.}
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
            ink_centered: true
            text_style +: {font_size: 14.}
        }
    }
    let FloorTile = View{
        cursor: MouseCursor.Hand
        // One size on every screen (FLOOR_TILE_W × FLOOR_TILE_H): more
        // columns than fit scroll sideways instead of shrinking the cards.
        width: 280 height: 178
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
        bg_focus := RoundedView{
            width: Fill height: Fill
            visible: false
            draw_bg.color: #x00000000
            draw_bg.border_radius: 14.
            draw_bg.border_size: 3.
            draw_bg.border_color: #x023637
        }
        View{
            width: Fill height: Fill
            // Extra bottom inset so status never kisses the stroke.
            padding: Inset{left: 14. right: 14. top: 12. bottom: 16.}
            flow: Down spacing: 6
            View{
                width: Fill height: Fit
                flow: Right spacing: 6
                align: Align{y: 0.5}
                name := Label{
                    width: Fill
                    text: "T1"
                    flow: Right{wrap: false}
                    max_lines: 1
                    text_overflow: Ellipsis
                    draw_text.color: #x1F1A17
                    draw_text.text_style.font_size: 18.
                }
                // Print / view sit with the primary act — never a bottom row
                // that clips under the tile height.
                print_ico := FloorTileIconBtn{text: "\u{f02f}" visible: false}
                view_ico := FloorTileIconBtn{text: "\u{f06e}" visible: false}
                act := FloorTileIconBtn{text: "\u{f234}"}
            }
            // Never wrap:true + Fill on Labels — turtle coerces every draw and
            // froze Floor on phones with dozens of tables (2026-11).
            meta := Label{
                width: Fill
                text: ""
                flow: Right{wrap: false}
                max_lines: 1
                text_overflow: Ellipsis
                draw_text.color: #x1F1A17
                draw_text.text_style.font_size: 15.
            }
            state := Label{
                width: Fill
                text: "Free"
                flow: Right{wrap: false}
                max_lines: 1
                text_overflow: Ellipsis
                draw_text.color: #x3A2C26
                draw_text.text_style.font_size: 12.
            }
        }
    }

    // Kitchen tab: one card per open KOT — items + state, never prices.
    // Age colour via stacked backgrounds (same idea as FloorTile).
    // Fixed height (KOT_CARD_H). In a Fit Overlay a Fill layer drawn before
    // the content gets the height used so far — zero — so the ageing colour
    // never showed and rows collapsed while the list moved. The KitchenList
    // pitch (KOT_ROW) is this height plus the item's bottom pad.
    let KotCard = View{
        // The whole card is the kitchen's tap target, like its Start / Ready
        // / Done button (a View only hears the finger with a cursor). The
        // tap moves the ticket only from the state the card showed
        // (`advance_kot_from`), so a double tap never skips Preparing.
        cursor: MouseCursor.Hand
        width: Fill height: 200
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
            draw_bg.border_color: #x017B85
        }
        View{
            width: Fill height: Fill
            padding: Inset{left: 12. right: 12. top: 12. bottom: 12.}
            flow: Down spacing: 6
            // Card is a kitchen tap target — plain Labels never take the press.
            // Capped lines keep every card the same height; the full ticket
            // prints, and the kitchen reads the dishes from the summary.
            title := Label{
                width: Fill
                text: "Sections"
                flow: Right{wrap: true}
                max_lines: 2
                text_overflow: Ellipsis
                draw_text.color: #x025B60
                draw_text.text_style.font_size: 14.
            }
            View{
                width: Fill height: Fill
                summary := Label{
                    width: Fill
                    text: ""
                    flow: Right{wrap: true}
                    max_lines: 3
                    text_overflow: Ellipsis
                    draw_text.color: #x023637
                    draw_text.text_style.font_size: 15.
                }
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
                    text_overflow: Ellipsis
                    draw_text.color: #x9A6206
                    draw_text.text_style.font_size: 13.
                }
                advance := Button{
                    text: "Start"
                    width: Fit height: 44
                    padding: Inset{left: 18. right: 18. top: 10. bottom: 10.}
                    draw_bg +: {
                        color: #x017B85
                        color_hover: #x822801
                        color_down: #x6f2000
                        color_focus: #x017B85
                        color_2: #x017B85
                        color_2_hover: #x822801
                        color_2_down: #x6f2000
                        color_2_focus: #x017B85
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
        // Fixed height — Overlay+Fit collapses and empties MenuList.
        // Name/alias wrap inside (no ellipsis).
        width: Fill height: 152
        flow: Overlay
        RoundedView{
            width: Fill height: Fill
            align: Align{x: 0.5 y: 0.5}
            draw_bg.color: #x017B85
            draw_bg.color_2: #x025B60
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
        // Darker foot wash so name/price stay readable on bright photos.
        View{
            width: Fill height: Fill
            show_bg: true
            draw_bg +: {
                radius: uniform(16.0)
                pixel: fn() {
                    let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                    sdf.box(0., 0., self.rect_size.x, self.rect_size.y, self.radius)
                    let t = smoothstep(0.40, 0.88, self.pos.y)
                    sdf.fill(vec4(0.04, 0.01, 0.01, 0.97 * t))
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
                // Fixed photo card: wrap then clean Ellipsis — never mid-glyph crop.
                name := Label{
                    width: Fill
                    text: "Item"
                    flow: Right{wrap: true}
                    max_lines: 2
                    text_overflow: Ellipsis
                    draw_text.color: #xFFFFFF
                    draw_text.text_style: theme.font_bold{font_size: 16.}
                }
                alias := Label{
                    visible: false
                    width: Fill
                    text: ""
                    flow: Right{wrap: true}
                    max_lines: 1
                    text_overflow: Ellipsis
                    draw_text.color: #xF3E3DA
                    draw_text.text_style: theme.font_bold{font_size: 12.}
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
                        draw_text.text_style: theme.font_bold{font_size: 14.}
                    }
                    stock := Label{
                        width: Fill
                        text: " "
                        flow: Right{wrap: true}
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
                remove := CloseIconBtn{
                    width: 34 height: 34
                    draw_bg +: {
                        // Frosted dark — readable on any photo without a side wash.
                        mark_color: #xffffff
                        color: #x1A0A08CC
                        color_hover: #x023637
                        color_down: #x025B60
                        color_focus: #x1A0A08CC
                        color_2: #x1A0A08CC
                        color_2_hover: #x023637
                        color_2_down: #x025B60
                        color_2_focus: #x1A0A08CC
                        border_color: #xFFFFFF88
                        border_radius: 17.
                        border_size: 1.
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
                        color_hover: #x017B85
                        color_down: #x8A320C
                        color_focus: #x1A0A08CC
                        color_2: #x1A0A08CC
                        color_2_hover: #x017B85
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
                        color_hover: #x017B85
                        color_down: #x025B60
                        color_focus: #x1A0A08CC
                        color_2: #x1A0A08CC
                        color_2_hover: #x017B85
                        color_2_down: #x025B60
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
    // Drawn mark (`draw_bg.mark`: 3 = +, 4 = −).
    let TicketStepBtn = Button{
        width: 34 height: 34
        padding: Inset{left: 0. right: 0. top: 0. bottom: 0.}
        align: Align{x: 0.5 y: 0.5}
        text: ""
        label_walk: Walk{width: 0. height: 0.}
        icon_walk: Walk{width: 0. height: 0.}
        draw_bg +: {
            mark: uniform(3.0)
            mark_color: uniform(#x025B60)
            color: #xffffff
            color_hover: #x023637
            color_down: #x025B60
            color_focus: #xffffff
            color_2: #xffffff
            color_2_hover: #x023637
            color_2_down: #x025B60
            color_2_focus: #xffffff
            border_color: #xC9B0A2
            border_color_hover: #x023637
            border_color_down: #x025B60
            border_color_focus: #xC9B0A2
            border_radius: 9.
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
                sdf.stroke(
                    self.border_color
                        .mix(self.border_color_hover, self.hover)
                        .mix(self.border_color_down, self.down),
                    self.border_size
                )
                let c = self.rect_size * vec2(0.5, 0.5)
                let s = 7.0
                let ink = self.mark_color
                    .mix(#xffffff, self.hover)
                    .mix(#xffffff, self.down)
                sdf.move_to(c.x - s, c.y)
                sdf.line_to(c.x + s, c.y)
                if self.mark < 3.5 {
                    sdf.move_to(c.x, c.y - s)
                    sdf.line_to(c.x, c.y + s)
                }
                sdf.stroke(ink, 2.4)
                return sdf.result
            }
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
            color_hover: #x023637
            color_down: #x025B60
            color_focus: #xffffff
            color_2: #xffffff
            color_2_hover: #x023637
            color_2_down: #x025B60
            color_2_focus: #xffffff
            border_color: #xC9B0A2
            border_color_hover: #x023637
            border_color_down: #x025B60
            border_color_focus: #xC9B0A2
            border_radius: 9.
            border_size: 1.
        }
        draw_text +: {
            color: #x025B60
            color_hover: #xffffff
            color_down: #xffffff
            color_focus: #x025B60
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
            // Thumb + name/meta | (− qty + above amount) — both columns
            // share the row's vertical centre; amount keeps its right edge.
            align: Align{y: 0.5}
            // The dish's photo (or initials), so a busy ticket reads at a glance.
            ThumbAvatar{
                width: 44 height: 44
                margin: Inset{top: 0. bottom: 0. left: 0. right: 0.}
            }
            View{
                width: Fill height: Fit
                flow: Down spacing: 4
                label := WrapLabel{
                    selectable: false
                    body: "Item"
                    font_color: #x023637
                    font_size: 14.
                }
                line_meta := WrapLabel{
                    selectable: false
                    body: " "
                    font_color: #x074D3E
                    font_size: 11.
                }
                // Hold (unsent) / include-in-recovery-print (sent) under the meta.
                // Cancel sits here too — buried under tap-to-expand made "can't
                // cancel after KOT" feel broken (2026-11).
                View{
                    width: Fit height: Fit
                    flow: Right{wrap: true}
                    spacing: 6
                    hold_send_btn := TicketHoldBtn{text: "Hold"}
                    reprint_incl_btn := TicketHoldBtn{text: "In print" visible: false}
                    cancel_pick_btn := TicketHoldBtn{text: "Mark" visible: false}
                    line_cancel_sum := QuietFitBtn{text: "Cancel" visible: false}
                }
            }
            // Right column: same −/qty/+ as the menu card, stacked above the
            // line amount. Fit width keeps the amount's horizontal place;
            // column is vertically centred with the thumb/name block.
            View{
                width: Fit height: Fit
                flow: Down spacing: 6
                align: Align{x: 1.0 y: 0.5}
                stepper := View{
                    width: Fit height: Fit
                    flow: Right spacing: 4
                    align: Align{y: 0.5}
                    minus := TicketStepBtn{draw_bg.mark: 4.0}
                    // Centred in a fixed box so 1 and 12 sit in the same place.
                    View{
                        width: 36 height: 34
                        align: Align{x: 0.5 y: 0.5}
                        qty_label := Label{
                            width: Fit
                            text: "1"
                            draw_text.color: #x025B60
                            draw_text.text_style.font_size: 15.
                        }
                    }
                    plus := TicketStepBtn{draw_bg.mark: 3.0}
                }
                line_amount := Label{
                    width: Fit
                    text: "0"
                    draw_text.color: #x025B60
                    draw_text.text_style.font_size: 14.
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
                    draw_text.color: #x023637
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
                    draw_text.color: #x017B85
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
                mod_prev := RoleChevronBtn{draw_bg.point: -1.0}
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
                mod_next := RoleChevronBtn{}
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
            // Right, no wrap. Wrap on this row gave Cancel / NC / Move a
            // one-pixel label under the note (the faces stayed, the words
            // did not). Three short Fit buttons fit the ticket.
            sent_row := View{
                width: Fill height: Fit
                flow: Right
                spacing: 8
                align: Align{y: 0.5}
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
            color: #x023637
            color_hover: #x017B85
            color_down: #x025B60
            color_focus: #x023637
            color_disabled: #xA9806E
            color_2: #x023637
            color_2_hover: #x017B85
            color_2_down: #x025B60
            color_2_focus: #x023637
            color_2_disabled: #xA9806E
            border_color: #x023637
            border_color_hover: #x017B85
            border_color_down: #x025B60
            border_color_focus: #x023637
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
                draw_text.color: #x023637
                draw_text.text_style.font_size: 15.
            }
            label := Label{
                width: Fill
                text: ""
                draw_text.color: #x56575D
                draw_text.text_style.font_size: 12.
            }
        }
        sale_amount := Label{
            width: Fit
            text: ""
            draw_text.color: #x025B60
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
            // Pick-row face — do not steal press for select.
            label := WrapLabel{
                body: "customer"
                font_color: #x023637
                font_size: 14.
                selectable: false
            }
            sublabel := WrapLabel{
                body: "id"
                font_color: #x074D3E
                font_size: 11.
                selectable: false
            }
        }
        pick := Button{
            text: "Pick"
            width: 72 height: 44
            draw_bg +: {
                color: #x023637
                color_hover: #x017B85
                color_down: #x025B60
                color_focus: #x023637
                color_2: #x023637
                color_2_hover: #x017B85
                color_2_down: #x025B60
                color_2_focus: #x023637
                border_color: #x023637
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
            color_hover: #x023637
            color_down: #x025B60
            color_focus: #xffffff
            color_2: #xffffff
            color_2_hover: #x023637
            color_2_down: #x025B60
            color_2_focus: #xffffff
            border_color: #x5A4238
            border_color_hover: #x023637
            border_color_down: #x074D3E
            border_color_focus: #x5A4238
            border_radius: 14.
            border_size: 2.
        }
        draw_text +: {
            color: #x023637
            color_hover: #xffffff
            color_down: #xffffff
            color_focus: #x023637
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
        // Row pick face — Use button + row tap; keep select off the label.
        label := WrapLabel{
            width: Fill
            body: "locale"
            font_color: #x023637
            font_size: 14.
            selectable: false
        }
        pick := Button{
            text: "Use"
            width: 80 height: 48
            draw_bg +: {
                color: #x023637
                color_hover: #x017B85
                color_down: #x025B60
                color_focus: #x023637
                color_2: #x023637
                color_2_hover: #x017B85
                color_2_down: #x025B60
                color_2_focus: #x023637
                border_color: #x023637
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

    // Local server pick: name wraps; Open + Remove share the row (never crop).
    let ServerRow = RoundedView{
        cursor: MouseCursor.Hand
        width: Fill height: Fit
        padding: Inset{left: 16. right: 10. top: 10. bottom: 10.}
        flow: Down spacing: 8
        align: Align{y: 0.5}
        draw_bg.color: #xffffff
        draw_bg.color_hover: #xF0F4FF
        draw_bg.color_down: #xE0E8FF
        draw_bg.border_radius: 14.
        draw_bg.border_size: 2.
        draw_bg.border_color: #x5A4238
        label := WrapLabel{
            width: Fill
            body: "server"
            font_color: #x023637
            font_size: 14.
        }
        View{
            width: Fill height: Fit
            flow: Right spacing: 8
            pick := Button{
                text: "Open"
                width: Fill height: Fit
                padding: Inset{left: 12. right: 12. top: 10. bottom: 10.}
                align: Align{x: 0.5 y: 0.5}
                flow: Right{wrap: true}
                label_walk: Walk{width: Fit height: Fit margin: Inset{left: 2. right: 2.}}
                icon_walk: Walk{width: 0. height: 0.}
                draw_bg +: {
                    color: #x023637
                    color_hover: #x017B85
                    color_down: #x025B60
                    color_focus: #x023637
                    color_2: #x023637
                    color_2_hover: #x017B85
                    color_2_down: #x025B60
                    color_2_focus: #x023637
                    border_color: #x023637
                    border_radius: 12.
                    border_size: 0.
                }
                draw_text +: {
                    color: #xffffff
                    color_hover: #xffffff
                    color_down: #xffffff
                    color_focus: #xffffff
                    ink_centered: true
                    text_style +: {font_size: 14.}
                }
            }
            remove := Button{
                text: "Remove"
                width: Fill height: Fit
                padding: Inset{left: 12. right: 12. top: 10. bottom: 10.}
                align: Align{x: 0.5 y: 0.5}
                flow: Right{wrap: true}
                label_walk: Walk{width: Fit height: Fit margin: Inset{left: 2. right: 2.}}
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
                    border_color: #x023637
                    border_radius: 12.
                    border_size: 1.5
                }
                draw_text +: {
                    color: #x023637
                    color_hover: #x025B60
                    color_down: #x4A0608
                    color_focus: #x023637
                    ink_centered: true
                    text_style +: {font_size: 14.}
                }
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
        View{
            width: Fill height: Fit
            flow: Down spacing: 2
            label := WrapLabel{
                width: Fill
                body: "item"
                font_color: #x023637
                font_size: 14.
            }
            alias := Label{
                visible: false
                width: Fill
                text: ""
                flow: Right{wrap: true}
                draw_text.color: #x7A5A4E
                draw_text.text_style.font_size: 12.
            }
        }
        photo := Button{
            text: "Photo"
            width: Fit height: 40
            padding: Inset{left: 12. right: 12. top: 6. bottom: 6.}
            draw_bg +: {
                color: #xffffff
                color_hover: #x023637
                color_down: #x025B60
                color_focus: #xffffff
                color_2: #xffffff
                color_2_hover: #x023637
                color_2_down: #x025B60
                color_2_focus: #xffffff
                border_radius: 10.
                border_size: 2.
                border_color: #x023637
                border_color_hover: #x023637
                border_color_down: #x025B60
                border_color_focus: #x023637
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
                color: #x023637
                color_hover: #x017B85
                color_down: #x025B60
                color_focus: #x023637
                color_2: #x023637
                color_2_hover: #x017B85
                color_2_down: #x025B60
                color_2_focus: #x023637
                border_color: #x023637
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
    // Self-scrolling Fill viewport (no page ScrollY behind it): opt into
    // mouse press-drag so laptops without a wheel can still scroll dishes.
    mod.widgets.ProductList = set_type_default() do mod.widgets.ProductListBase{
        width: Fill height: Fit
        list := PortalList{
            width: Fill height: Fill
            spacing: 12
            drag_scrolling: true
            mouse_drag_scrolling: true
            bounce_at_start: false
            bounce_at_end: false
            capture_overload: true
            scroll_bar: ScrollBar{
                bar_size: 10.
                draw_bg +: {
                    size: 6.
                    border_radius: 3.
                    color: #xC9AFA2
                    color_hover: #x023637
                    color_drag: #x025B60
                }
            }
            Item := View{
                width: Fill height: Fit
                flow: Right spacing: 20
                // Row gap under each card row (across gap is `spacing` above).
                // Right: the scroll bar's own lane, so the bar never lies over
                // the last card's photo or price (PRODUCT_LIST_BAR_LANE).
                padding: Inset{right: 14. bottom: 24.}
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

    // One tile row inside an area's sideways scroller.
    // `width: Fit` — never Fill. Fill collapses to the ScrollX viewport
    // (~2×280 on a phone), so Columns=3 clipped the third tile with no
    // sideways scroll. Rust sets each row to floor_grid_width(cols).
    let FloorTileRow = View{
        width: Fit height: Fit
        padding: Inset{bottom: 10.}
        flow: Right spacing: 10
        texture_caching: false
        left_gap := View{width: 0 height: 1 visible: false}
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
    mod.widgets.FloorListBase = #(FloorList::register_widget(vm))
    // One PortalList item per area. Each area has its own ScrollXView so
    // sideways pan on Dine In does not move Parcel (and the reverse).
    mod.widgets.FloorList = set_type_default() do mod.widgets.FloorListBase{
        width: Fill height: Fit
        flow: Down
        top_gap := View{width: Fill height: 0}
        list_box := View{
            width: Fill height: 0
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
                    flow: Down spacing: 4
                    texture_caching: false
                    area_label := Label{
                        width: Fit
                        text: ""
                        padding: Inset{top: 6. bottom: 2.}
                        draw_text.color: #x025B60
                        draw_text.text_style.font_size: 16.
                    }
                    // Desk pane (not a chip strip): mouse + touch press-drag
                    // pans sideways over tiles, gaps, and pads. `axis_lock`
                    // waits ~8px before scroll owns the gesture — a short
                    // press stays a tile tap / icon click (finger_up_is_tap).
                    // Capture overload must be on: rows/PortalList cover the
                    // body, so "whitespace only" overload-off never saw the
                    // pads or the gaps between tables.
                    // Top pad = lane above tiles (18) + Columns↔tables gap (10)
                    // that must live *inside* this ScrollX (not parent spacing)
                    // so that whitespace pans with the Area like the tiles.
                    // Height is set in FloorList draw (body + pads). Fit+Fill
                    // PortalList used to leave this ~padding-tall so the X bar
                    // painted in the Columns↔tables gap and stole taps there.
                    area_scroll := ScrollXView{
                        width: Fill height: Fit
                        // Top: Columns↔tables gap + lane. Bottom: ≥ bar_size so
                        // the X handle sits under the tiles, never under Columns.
                        padding: Inset{top: 28. bottom: 22.}
                        scroll_bars: mod.widgets.ScrollBars {
                            show_scroll_x: true show_scroll_y: false
                            scroll_bar_x: mod.widgets.ScrollBar{
                                drag_scrolling: true
                                mouse_capture_overload: true
                                axis_lock: true
                                bounce_at_start: false
                                bounce_at_end: false
                                bar_size: 10.
                                min_handle_size: 40.
                                draw_bg +: {
                                    size: 6.
                                    border_radius: 3.
                                    color: #xC9AFA2
                                    color_hover: #x023637
                                    color_drag: #x025B60
                                }
                            }
                        }
                        area_body := View{
                            // Rust sets Fixed(floor_grid_width(cols)) so Columns=N
                            // is always N tiles wide (scroll sideways when needed).
                            width: Fit height: Fit
                            flow: Down
                            // Comfort pads above/below rows (same width as the
                            // grid) — press-drag here pans too.
                            padding: Inset{top: 10. bottom: 14.}
                            // Fill is fine once area_scroll + area_body get Fixed
                            // heights in FloorList draw (see product_menu_floor).
                            area_rows := PortalList{
                                width: Fill height: Fill
                                spacing: 10
                                drag_scrolling: false
                                bounce_at_start: false
                                bounce_at_end: false
                                capture_overload: false
                                scroll_bar: ScrollBar{bar_size: 0.}
                                Item := FloorTileRow{}
                            }
                        }
                    }
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
    // Page owns scroll; only on-screen KOT cards draw (bug 6c — full-height
    // PortalList froze Android on every touch-move redraw).
    mod.widgets.KitchenList = set_type_default() do mod.widgets.KitchenListBase{
        width: Fill height: Fit
        flow: Down
        top_gap := View{width: Fill height: 0}
        list_box := View{
        width: Fill height: 0
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
    // Full ticket height; outer Order ScrollYView scrolls. No page-window
    // slice — variable open/closed line heights made that flicker.
    mod.widgets.CartList = set_type_default() do mod.widgets.CartListBase{
        width: Fill height: Fit
        flow: Down
        top_gap := View{width: Fill height: 0}
        list_box := View{
            width: Fill height: 0
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
            drag_scrolling: true
            mouse_drag_scrolling: true
            bounce_at_start: false
            bounce_at_end: false
            scroll_bar: ScrollBar{bar_size: 6.}
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
        // As tall as the devices. The settings page scrolls.
        list := PortalList{
            width: Fill height: Fill
            spacing: 10
            drag_scrolling: false
            mouse_drag_scrolling: false
            scroll_bar: ScrollBar{bar_size: 0.}
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
            font_color: #x023637
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
        // Capped (~3 rows / 360px) — mouse press-drag when there is no wheel.
        list := PortalList{
            width: Fill height: Fill
            spacing: 10
            drag_scrolling: true
            mouse_drag_scrolling: true
            scroll_bar: ScrollBar{bar_size: 6.}
            Item := View{
                width: Fill height: Fit
                padding: Inset{top: 2. bottom: 8.}
                texture_caching: false
                ServerRow{}
            }
        }
    }

    // ClientPair: one full-width tap target per found shop (nested Button+label
    // rows miss taps inside PortalList on Android — same rule as DishRow).
    let FoundServerRow = Button{
        width: Fill height: Fit
        margin: Inset{left: 0. right: 0. top: 0. bottom: 0.}
        padding: Inset{left: 16. right: 16. top: 14. bottom: 14.}
        align: Align{x: 0.0 y: 0.5}
        flow: Right{wrap: true}
        grab_key_focus: false
        reset_hover_on_click: true
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
            border_color: #x5A4238
            border_radius: 14.
            border_size: 2.
        }
        draw_text +: {
            color: #x023637
            color_hover: #x023637
            color_down: #x023637
            color_focus: #x023637
            ink_centered: false
            text_style +: {font_size: 14.}
        }
    }

    mod.widgets.FoundServerListBase = #(FoundServerList::register_widget(vm))
    mod.widgets.FoundServerList = set_type_default() do mod.widgets.FoundServerListBase{
        width: Fill height: Fit
        margin: Inset{top: 4. bottom: 4.}
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
                pick := FoundServerRow{text: "shop"}
            }
        }
    }

    mod.widgets.CurrencyListBase = #(CurrencyList::register_widget(vm))
    mod.widgets.CurrencyList = set_type_default() do mod.widgets.CurrencyListBase{
        width: Fill height: Fit
        // Fixed 280px viewport — mouse press-drag when there is no wheel.
        list := PortalList{
            width: Fill height: Fill
            spacing: 6
            drag_scrolling: true
            mouse_drag_scrolling: true
            scroll_bar: ScrollBar{bar_size: 6.}
            Item := View{
                width: Fill height: Fit
                padding: Inset{bottom: 10.}
                texture_caching: false
                LocaleRow{}
            }
        }
    }

    // Dish / category pick row: thumb beside the name, type on the
    // cream card, Use centered. The photo never fills the row (type on
    // the food was unreadable, and Use sat on the bottom edge). Whole
    // row is the tap (nested Button clicks miss inside PortalList).
    let DishRow = View{
        cursor: MouseCursor.Hand
        // Fixed height — Overlay+Fit collapses pick lists.
        width: Fill height: 104
        flow: Overlay
        RoundedView{
            width: Fill height: Fill
            draw_bg.color: #xFFF8F3
            draw_bg.border_radius: 14.
            draw_bg.border_size: 1.
            draw_bg.border_color: #xD8C7BD
        }
        View{
            width: Fill height: Fill
            flow: Right spacing: 12
            padding: Inset{left: 12. right: 12. top: 12. bottom: 12.}
            align: Align{y: 0.5}
            View{
                width: 80 height: 80
                flow: Overlay
                RoundedView{
                    width: Fill height: Fill
                    align: Align{x: 0.5 y: 0.5}
                    draw_bg.color: #x017B85
                    draw_bg.color_2: #x025B60
                    draw_bg.border_radius: 12.
                    thumb_initials := Label{
                        width: Fit height: Fit
                        text: ""
                        draw_text.color: #xFFFFFF
                        draw_text.text_style.font_size: 18.
                    }
                }
                thumb := Image{
                    width: Fill height: Fill
                    fit: ImageFit.CropToFill
                    visible: false
                    draw_bg +: {
                        radius: uniform(12.0)
                        pixel: fn() {
                            let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                            sdf.box(0., 0., self.rect_size.x, self.rect_size.y, self.radius)
                            sdf.fill(self.get_color())
                            return sdf.result
                        }
                    }
                }
            }
            View{
                width: Fill height: Fit
                flow: Down spacing: 2
                title := Label{
                    width: Fill
                    text: ""
                    flow: Right{wrap: true}
                    max_lines: 2
                    text_overflow: Ellipsis
                    draw_text.color: #x023637
                    draw_text.text_style: theme.font_bold{font_size: 15.}
                }
                alias := Label{
                    visible: false
                    width: Fill
                    text: ""
                    flow: Right{wrap: true}
                    max_lines: 1
                    text_overflow: Ellipsis
                    draw_text.color: #x074D3E
                    draw_text.text_style.font_size: 13.
                }
                subtitle := Label{
                    width: Fill
                    text: ""
                    flow: Right{wrap: true}
                    max_lines: 1
                    text_overflow: Ellipsis
                    draw_text.color: #x074D3E
                    draw_text.text_style.font_size: 13.
                }
            }
            // Not a Button — the row is the tap. Centered with the thumb.
            RoundedView{
                width: Fit height: Fit
                padding: Inset{left: 14. right: 14. top: 8. bottom: 8.}
                align: Align{x: 0.5 y: 0.5}
                draw_bg.color: #x023637
                draw_bg.border_radius: 12.
                action := Label{
                    width: Fit
                    text: "Use"
                    draw_text.color: #xFFFFFF
                    draw_text.text_style.font_size: 13.
                }
            }
        }
    }

    // The item form's picker: its own scrolling viewport (height set in
    // draw_walk), so only the rows on screen draw — the shelf has ~1,400
    // dishes. A page-height list draws every row, which is how the old shelf
    // decoded a whole folder of photos in one frame and froze the app.
    // The item form's category picker: same rows and viewport as the
    // item picker.
    // One "type or pick" row: Labels on a clear RoundedView (Button does not
    // draw child widgets — empty pills). The list Item / ItemSel background is
    // the resting / selected face under the labels; hover/down from a clear
    // hit Button on top.
    let SuggestRowView = RoundedView{
        cursor: MouseCursor.Hand
        width: Fill height: Fit
        padding: Inset{left: 14. right: 14. top: 12. bottom: 12.}
        flow: Right spacing: 10
        align: Align{y: 0.5}
        draw_bg.color: #xffffff00
        draw_bg.border_radius: 10.
        draw_bg.border_size: 0.
        title := Label{
            width: Fill
            text: ""
            flow: Right{wrap: true}
            draw_text.color: #x023637
            draw_text.text_style.font_size: 15.
        }
        subtitle := Label{
            width: Fit
            text: ""
            flow: Right{wrap: true}
            draw_text.color: #x074D3E
            draw_text.text_style.font_size: 13.
        }
        action := Label{
            width: Fit
            text: "Use"
            draw_text.color: #x023637
            draw_text.text_style.font_size: 13.
        }
    }
    // Transparent hit face over SuggestRowView — hover / down wash only.
    let SuggestRowHit = Button{
        width: Fill height: Fill
        margin: Inset{left: 0. right: 0. top: 0. bottom: 0.}
        padding: Inset{left: 0. right: 0. top: 0. bottom: 0.}
        grab_key_focus: false
        reset_hover_on_click: true
        text: ""
        label_walk: Walk{width: 0. height: 0.}
        icon_walk: Walk{width: 0. height: 0.}
        draw_bg +: {
            color: #xffffff00
            color_hover: #xFBEDE4CC
            color_down: #xEFD2BCCC
            color_focus: #xffffff00
            color_2: #xffffff00
            color_2_hover: #xFBEDE4CC
            color_2_down: #xEFD2BCCC
            color_2_focus: #xffffff00
            border_radius: 10.
            border_size: 0.
        }
    }
    mod.widgets.SuggestListBase = #(SuggestList::register_widget(vm))
    mod.widgets.SuggestList = set_type_default() do mod.widgets.SuggestListBase{
        width: Fill height: Fit
        // Capped viewport (not expand-to-content): mouse press-drag scrolls
        // the list when there is no wheel (page still owns full-height lists).
        list := PortalList{
            width: Fill height: Fill
            spacing: 6
            drag_scrolling: true
            mouse_drag_scrolling: true
            scroll_bar: ScrollBar{bar_size: 6.}
            // The row's own background is its wash. Fill washes drawn before
            // the row in a Fit Overlay got the height used so far (zero), so
            // rows showed no card and could collapse while the list moved.
            // A selected row uses ItemSel. Right margin: the bar's lane.
            Item := RoundedView{
                width: Fill height: Fit
                margin: Inset{right: 8.}
                flow: Overlay
                texture_caching: false
                draw_bg.color: #xffffff
                draw_bg.border_radius: 10.
                draw_bg.border_size: 1.
                draw_bg.border_color: #xE3D3C8
                row := SuggestRowView{}
                // After the row: a Fill layer takes the row's height.
                hit := SuggestRowHit{}
            }
            ItemSel := RoundedView{
                width: Fill height: Fit
                margin: Inset{right: 8.}
                flow: Overlay
                texture_caching: false
                draw_bg.color: #xF6E3D3
                draw_bg.border_radius: 10.
                draw_bg.border_size: 1.
                draw_bg.border_color: #xE3D3C8
                row := SuggestRowView{}
                hit := SuggestRowHit{}
            }
        }
    }
    // Kitchen station row: drawn box + ✓ (Noto has ✓; ☐ was missing → tofu).
    // ↑↓ swap print order.
    let StationPrioBtn = Button{
        width: 28 height: 28
        padding: Inset{left: 0. right: 0. top: 0. bottom: 0.}
        align: Align{x: 0.5 y: 0.5}
        grab_key_focus: false
        reset_hover_on_click: true
        text: ""
        label_walk: Walk{width: Fit height: Fit}
        icon_walk: Walk{width: 0. height: 0.}
        draw_bg +: {
            color: #xF7EFE8
            color_hover: #xF4E8E0
            color_down: #xE8D6CA
            color_focus: #xF7EFE8
            color_2: #xF7EFE8
            color_2_hover: #xF4E8E0
            color_2_down: #xE8D6CA
            color_2_focus: #xF7EFE8
            border_radius: 8.
            border_size: 0.
        }
        draw_text +: {
            color: #x8A5A48
            color_hover: #x8A5A48
            color_down: #x8A5A48
            color_focus: #x8A5A48
            ink_centered: true
            text_style +: {font_size: 16.}
        }
    }
    // Whole row toggles the checkbox (finger_up on `row`); ↑↓ stay their own
    // hits so priority reorder is not swallowed by the check action.
    let StationCheckRow = RoundedView{
        cursor: MouseCursor.Hand
        width: Fill height: Fit
        padding: Inset{left: 8. right: 8. top: 10. bottom: 10.}
        flow: Right spacing: 8
        align: Align{y: 0.5}
        draw_bg.color: #xffffff
        draw_bg.border_radius: 10.
        draw_bg.border_size: 1.
        draw_bg.border_color: #xE3D3C8
        // Drawn checkbox — never depends on a missing Private-Use / ballot glyph.
        View{
            width: 22 height: 22
            flow: Overlay
            align: Align{x: 0.5 y: 0.5}
            check_off := RoundedView{
                width: 18 height: 18
                draw_bg.color: #xffffff
                draw_bg.border_radius: 3.
                draw_bg.border_size: 1.5
                draw_bg.border_color: #x8A5A48
            }
            check_on := RoundedView{
                width: 18 height: 18
                visible: false
                flow: Overlay
                align: Align{x: 0.5 y: 0.5}
                draw_bg.color: #x023637
                draw_bg.border_radius: 3.
                draw_bg.border_size: 0.
                Label{
                    text: "\u{2713}"
                    draw_text.color: #xffffff
                    draw_text.text_style.font_size: 12.
                }
            }
        }
        prio := Label{
            width: Fit
            text: ""
            draw_text.color: #x8A5A48
            draw_text.text_style.font_size: 13.
        }
        title := Label{
            width: Fill
            text: ""
            flow: Right{wrap: true}
            draw_text.color: #x023637
            draw_text.text_style.font_size: 15.
        }
        subtitle := Label{
            width: Fit
            text: ""
            flow: Right{wrap: true}
            draw_text.color: #x074D3E
            draw_text.text_style.font_size: 13.
        }
        prio_up := StationPrioBtn{text: "\u{2191}"}
        prio_down := StationPrioBtn{text: "\u{2193}"}
    }
    mod.widgets.StationCheckListBase = #(StationCheckList::register_widget(vm))
    mod.widgets.StationCheckList = set_type_default() do mod.widgets.StationCheckListBase{
        width: Fill height: Fit
        // Capped checklist (printers / Kitchen / Team stations): touch fling +
        // mouse press-drag (wheel still chains to page at the edges).
        list := PortalList{
            width: Fill height: Fill
            spacing: 6
            drag_scrolling: true
            mouse_drag_scrolling: true
            bounce_at_start: false
            bounce_at_end: false
            capture_overload: true
            scroll_bar: ScrollBar{bar_size: 6.}
            Item := View{
                width: Fill height: Fit
                texture_caching: false
                row := StationCheckRow{}
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
                        font_color: #x023637
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
                            font_color: #x074D3E
                            font_size: 14.
                        }
                        v := WrapLabel{
                            width: Fit
                            body: ""
                            font_color: #x023637
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
                            font_color: #x023637
                            font_size: 15.
                        }
                        k := WrapLabel{
                            width: Fill
                            body: ""
                            margin: Inset{right: 8.}
                            font_color: #x074D3E
                            font_size: 14.
                        }
                    }
                }
                Line := View{
                    width: Fill height: Fit
                    // `row` hit target — same as Kv (tap / drag-reorder).
                    row := View{
                        width: Fill height: Fit
                        padding: Inset{top: 4. bottom: 4.}
                        cursor: MouseCursor.Hand
                        line := WrapLabel{
                            body: ""
                            margin: Inset{right: 8.}
                            font_color: #x023637
                            font_size: 14.
                        }
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

    // Kitchen-slip preview. A tab is a real column: the header "Qty." and
    // the counts share one fixed right-hand box. This is not a report, so
    // a table code is never an amount and "Qty." is never a heading.
    mod.widgets.KotSlipPreviewBase = #(KotSlipPreview::register_widget(vm))
    mod.widgets.KotSlipPreview = set_type_default() do mod.widgets.KotSlipPreviewBase{
        width: Fill height: Fit
        card := RoundedView{
            width: Fill height: Fit
            padding: 0.
            draw_bg.color: #ffffff
            draw_bg.border_radius: 16.
            draw_bg.border_size: 1.
            draw_bg.border_color: #xE3D3C8
            slip := Image{
                width: Fill
                height: Fit
                fit: Stretch
            }
            list := PortalList{
                visible: false
                width: Fill height: 0.
                spacing: 0
                drag_scrolling: false
                bounce_at_start: false
                bounce_at_end: false
                capture_overload: false
                scroll_bar: ScrollBar{bar_size: 0.}
                Line := View{
                    width: Fill height: Fit
                    line := WrapLabel{
                        body: ""
                        margin: Inset{right: 8.}
                        font_color: #x023637
                        font_size: 14.
                    }
                }
                Qty := View{
                    width: Fill height: Fit
                    padding: Inset{top: 2. bottom: 2.}
                    flow: Right spacing: 8
                    align: Align{y: 0.5}
                    k := WrapLabel{
                        width: Fill
                        body: ""
                        font_color: #x023637
                        font_size: 14.
                    }
                    v := WrapLabel{
                        width: 80
                        body: ""
                        font_color: #x023637
                        font_size: 15.
                    }
                }
                QtyRtl := View{
                    width: Fill height: Fit
                    padding: Inset{top: 2. bottom: 2.}
                    flow: Right spacing: 8
                    align: Align{y: 0.5}
                    v := WrapLabel{
                        width: 80
                        body: ""
                        font_color: #x023637
                        font_size: 15.
                    }
                    k := WrapLabel{
                        width: Fill
                        body: ""
                        font_color: #x023637
                        font_size: 14.
                    }
                }
                Gap := View{
                    width: Fill height: 8
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
                draw_text.color: #x023637
                draw_text.text_style.font_size: 15.
            }
            stock_kind := Label{
                width: Fill
                text: ""
                draw_text.color: #x56575D
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
                draw_text.color: #x023637
                draw_text.text_style.font_size: 12.
            }
        }
        stock_qty := Label{
            width: Fit
            text: ""
            draw_text.color: #x023637
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
                part_remove := CloseIconBtn{
                    width: 40 height: 40
                    draw_bg +: {
                        mark_color: #x025B60
                        color: #xffffff
                        color_hover: #xFBE3D3
                        color_down: #xEFD2BC
                        color_focus: #xffffff
                        color_2: #xffffff
                        color_2_hover: #xFBE3D3
                        color_2_down: #xEFD2BC
                        color_2_focus: #xffffff
                        border_color: #xD8BBA6
                        border_radius: 10.
                        border_size: 1.
                    }
                }
            }
        }
    }

    mod.widgets.CategoryPickListBase = #(CategoryPickList::register_widget(vm))
    mod.widgets.CategoryPickList = set_type_default() do mod.widgets.CategoryPickListBase{
        width: Fill height: Fit
        margin: Inset{top: 8. bottom: 4.}
        // Capped viewport (DISH_PICK_MAX_HEIGHT) — mouse press-drag when no wheel.
        list := PortalList{
            width: Fill height: Fill
            spacing: 12
            drag_scrolling: true
            mouse_drag_scrolling: true
            scroll_bar: ScrollBar{bar_size: 6.}
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
        // Capped viewport (DISH_PICK_MAX_HEIGHT) — mouse press-drag when no wheel.
        list := PortalList{
            width: Fill height: Fill
            spacing: 12
            drag_scrolling: true
            mouse_drag_scrolling: true
            scroll_bar: ScrollBar{bar_size: 6.}
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
                // Layout content still never goes below 375 css px (app_shell).
                window.inner_size: vec2(1024, 768)
                pass.clear_color: #xFBFCFC
                caption_bar +: {
                    draw_bg.color: #x023637
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
                    // Min layout width 375 css px. Narrower viewports scroll horizontally
                    // instead of crushing faces below that floor.
                    app_hscroll := ScrollXView{
                        width: Fill height: Fill
                        app_shell := View{
                            width: Fill{min: 375.} height: Fill
                            // Wide (≥720): side rail. Narrow: bottom tabs. See apply_shell_layout.
                            flow: Right spacing: 0

                    // Tablet / desktop nav. More hub expanded into its destinations.
                    nav_rail := SolidView{
                        visible: false
                        width: 220 height: Fill
                        draw_bg.color: #x025B60
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
                                    rail_collapse_btn := RailToggleBtn{draw_bg.point: -1.0}
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
                                    rail_rates := TabBtn{text: "Medicines"}
                                    rail_rates_on := TabBtnActive{text: "Medicines" visible: false}
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
                                    rail_ops := TabBtn{text: "Shop"}
                                    rail_ops_on := TabBtnActive{text: "Shop" visible: false}
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
                                    rail_cod := TabBtn{text: "Sections"}
                                    rail_cod_on := TabBtnActive{text: "Sections" visible: false}
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
                        draw_bg.color: #x023637
                    }
                    main_column := View{
                        width: Fill height: Fill
                        flow: Down spacing: 0

                    header := SolidView{
                        width: Fill height: Fit
                        padding: Inset{left: 16. right: 16. top: 12. bottom: 12.}
                        flow: Down spacing: 8
                        draw_bg.color: #x025B60

                        View{
                            width: Fill height: Fit
                            flow: Right spacing: 10
                            align: Align{y: 0.5}
                            rail_expand_btn := RailToggleBtn{visible: false}
                            // Elegant back — left of the shop mark; body "Back" buttons stay hidden.
                            // Drawn ‹ (not FA glyph): glyphs sit low in the 36×36 hit target.
                            header_back := Button{
                                width: 36 height: 36
                                visible: false
                                align: Center
                                padding: Inset{left: 0. right: 0. top: 0. bottom: 0.}
                                margin: Inset{left: 0. right: 2. top: 0. bottom: 0.}
                                text: ""
                                label_walk: Walk{width: 0. height: 0.}
                                icon_walk: Walk{width: 0. height: 0.}
                                draw_bg +: {
                                    point: uniform(-1.0)
                                    mark_color: uniform(#xffffff)
                                    color: #x00000000
                                    color_hover: #x023637
                                    color_down: #x3D0507
                                    color_focus: #x00000000
                                    color_2: #x00000000
                                    color_2_hover: #x023637
                                    color_2_down: #x3D0507
                                    color_2_focus: #x00000000
                                    border_color: #x00000000
                                    border_radius: 18.
                                    border_size: 0.
                                    pixel: fn() {
                                        let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                                        sdf.box(0., 0., self.rect_size.x, self.rect_size.y, self.border_radius)
                                        let fill = self.color
                                            .mix(self.color_hover, self.hover)
                                            .mix(self.color_down, self.down)
                                        sdf.fill(fill)
                                        let c = self.rect_size * vec2(0.5, 0.5)
                                        let w = 4.0 * self.point
                                        let h = 7.5
                                        sdf.move_to(c.x - w, c.y - h)
                                        sdf.line_to(c.x + w, c.y)
                                        sdf.line_to(c.x - w, c.y + h)
                                        sdf.stroke(self.mark_color, 2.4)
                                        return sdf.result
                                    }
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
                                View{
                                    width: Fill height: Fit
                                    flow: Right spacing: 6
                                    align: Align{y: 0.5}
                                    // Selectable WrapLabel — font_color stays white on maroon.
                                    brand_title := WrapLabel{
                                        width: Fit
                                        body: "Tabiyat"
                                        font_color: #xffffff
                                        font_size: 18.
                                    }
                                    // Short server id — smaller than the shop name.
                                    brand_server_id := WrapLabel{
                                        width: Fit
                                        body: ""
                                        font_color: #xE8C4B0
                                        font_size: 11.
                                        visible: false
                                    }
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
                            font_color: #x023637
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
                                    draw_bg.color: #x023637
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
                                    draw_bg.color: #xFBFCFC
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
                                                View{
                                                    width: Fill height: Fit
                                                    flow: Right spacing: 6
                                                    align: Align{y: 0.5}
                                                    setup_brand_title := WrapLabel{
                                                        width: Fit
                                                        body: "Your shop"
                                                        font_color: #x025B60
                                                        font_size: 18.
                                                    }
                                                    setup_brand_server_id := WrapLabel{
                                                        width: Fit
                                                        body: ""
                                                        font_color: #x074D3E
                                                        font_size: 11.
                                                        visible: false
                                                    }
                                                }
                                                device_use_question := WrapLabel{
                                                    body: ""
                                                    font_color: #x074D3E
                                                    font_size: 12.
                                                    visible: false
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
                                            flow: Right spacing: 0
                                            align: Align{y: 0.5}
                                            // One header: invisible hits + ℹ (not three painted segments).
                                            setup_lang_hdr := AccordionHdrHitBtn{width: Fit text: "LANGUAGE"}
                                            setup_lang_help := InfoHelp{
                                                tip: "Follow this device’s language, or pick one yourself."
                                            }
                                            setup_lang_hdr_ext := AccordionHdrHitBtn{text: ""}
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
                                                font_color: #x023637
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
                                            flow: Right spacing: 0
                                            align: Align{y: 0.5}
                                            setup_role_hdr := AccordionHdrHitBtn{width: Fit text: "THIS DEVICE"}
                                            setup_device_help := InfoHelp{
                                                tip: "Server keeps the books here. Client pairs to a Server."
                                            }
                                            setup_role_hdr_ext := AccordionHdrHitBtn{text: ""}
                                        }
                                        setup_role_body := View{
                                            width: Fill height: Fit
                                            flow: Down spacing: 8
                                            visible: true
                                            device_role_actions := ActionStack{
                                                device_role_server := StackSecondaryBtn{text: "Server"}
                                                device_role_server_on := StackPrimaryBtn{text: "Server" visible: false}
                                                device_role_client := StackSecondaryBtn{text: "Client"}
                                                device_role_client_on := StackPrimaryBtn{text: "Client" visible: false}
                                            }
                                            // Busy: replace Server/Client with status + Cancel
                                            // (DEV.md 7b — never set_text on wrapping Stack* faces).
                                            device_role_busy_label := WrapLabel{
                                                visible: false
                                                body: "Opening client…"
                                                font_color: #x023637
                                                font_size: 15.
                                            }
                                            device_role_cancel := QuietFitBtn{text: "Cancel" visible: false}
                                            // Offline LAN: multi-server pick unused (cloud-only later).
                                            // Ids kept so live reload / old bindings do not break.
                                            server_pick_section := View{
                                                width: Fill height: Fit
                                                flow: Down spacing: 12
                                                visible: false
                                                View{
                                                    width: Fill height: Fit
                                                    flow: Right spacing: 0
                                                    align: Align{y: 0.5}
                                                    server_pick_title := WrapLabel{
                                                        width: Fit
                                                        body: "Choose a server"
                                                        font_color: #x023637
                                                        font_size: 14.
                                                    }
                                                    server_pick_help := InfoHelp{
                                                        tip: "Offline: one shop server per install. Multi-server is cloud-only later."
                                                    }
                                                }
                                                server_name_input := FieldInput{empty_text: "New server name (optional)"}
                                                server_create_new := NavBtn{text: "Create new server"}
                                                // Remove needs that shop's Owner PIN (vision §17) — not on Open.
                                                server_remove_pin := FieldInput{
                                                    empty_text: "Owner PIN to remove a server"
                                                    is_password: true
                                                }
                                                setup_server_list := mod.widgets.ServerList{}
                                                server_pick_cancel := BackBtn{text: "Back"}
                                            }
                                            // One-shot after new server create — before Owner PIN.
                                            // Apply: Submit → status + Cancel (same shape as Menu import).
                                            app_template_section := View{
                                                width: Fill height: Fit
                                                flow: Down spacing: 12
                                                visible: false
                                                app_template_heading := WrapLabel{
                                                    body: "Starter catalogue"
                                                    font_color: #x023637
                                                    font_size: 17.
                                                }
                                                app_template_blurb := WrapLabel{
                                                    body: "Not used on this counter shop — skip to set your Owner PIN."
                                                    font_color: #x074D3E
                                                    font_size: 13.
                                                }
                                                app_template_apply := NavBtn{text: "Starter catalogue"}
                                                app_template_busy := View{
                                                    width: Fill height: Fit
                                                    visible: false
                                                    flow: Right spacing: 8
                                                    app_template_status := StackPrimaryBtn{text: "Importing…"}
                                                    app_template_cancel := StackLightBtn{text: "Cancel"}
                                                }
                                                app_template_skip := QuietFitBtn{text: "Skip for now"}
                                            }
                                            device_setup_status := WrapLabel{
                                                body: " "
                                                font_color: #x017B85
                                                font_size: 12.
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
                                    draw_bg.color: #x023637
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
                                    draw_bg.color: #xFBFCFC
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
                                                View{
                                                    width: Fill height: Fit
                                                    flow: Right spacing: 6
                                                    align: Align{y: 0.5}
                                                    lock_brand_title := WrapLabel{
                                                        width: Fit
                                                        body: "Your shop"
                                                        font_color: #x025B60
                                                        font_size: 18.
                                                    }
                                                    lock_brand_server_id := WrapLabel{
                                                        width: Fit
                                                        body: ""
                                                        font_color: #x074D3E
                                                        font_size: 11.
                                                        visible: false
                                                    }
                                                }
                                                h_counter_locked := WrapLabel{
                                                    body: "Counter 01 · Locked"
                                                    font_color: #x074D3E
                                                    font_size: 12.
                                                }
                                            }
                                            h_shop := Label{
                                                text: "Shop"
                                                draw_text.color: #x017B85
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
                                            font_color: #x023637
                                            font_size: 11.
                                        }
                                        lock_staff_pick := StackPrimaryBtn{text: "Choose staff"}
                                        lock_staff_label := WrapLabel{
                                            width: Fill
                                            body: "No staff yet — unlock as Owner, then Team"
                                            font_color: #x023637
                                            font_size: 14.
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
                                            font_color: #x017B85
                                            font_size: 12.
                                        }
                                    }

                                    View{
                                        width: Fill height: Fit
                                        flow: Right spacing: 0
                                        align: Align{y: 0.5}
                                        lock_login_title := WrapLabel{
                                            width: Fit
                                            body: "Or unlock with login"
                                            font_color: #x023637
                                            font_size: 14.
                                        }
                                        lock_login_help := InfoHelp{
                                            tip: "Optional: unlock with login id and password instead of PIN."
                                        }
                                    }
                                    lock_login_id := FieldInput{empty_text: "Username"}
                                    lock_login_password := FieldInput{
                                        empty_text: "Password"
                                        is_password: true
                                    }
                                    lock_login_ok := StackPrimaryBtn{text: "Unlock with login"}
                                    // Client: reach Find / pick among Servers without unlocking.
                                    lock_client_connection := NavBtn{
                                        text: "Connection to the shop"
                                        visible: false
                                    }
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
                                // Auth header is hidden here — body Back like ClientPair
                                // (→ DeviceSetup / previous onboard step).
                                onboard_back := BackBtn{text: "Back"}
                                h_setup_app_tabiyat := WrapLabel{
                                    body: "Set up Tabiyat"
                                    font_color: #x023637
                                    font_size: 22.
                                }
                                onboard_status := WrapLabel{
                                    body: "Name this counter if you want, then Start fresh or Bring my book."
                                    font_color: #x074D3E
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
                                            flow: Right spacing: 0
                                            align: Align{y: 0.5}
                                            h_shop_logo_optional := WrapLabel{
                                                    width: Fit
                                                body: "Shop logo (optional)"
                                                font_color: #x023637
                                                font_size: 15.
                                            }
                                            onboard_icon_help := InfoHelp{
                                                tip: "Optional shop mark on screens in the app."
                                            }
                                        }
                                        onboard_icon_status := WrapLabel{
                                            body: " "
                                            font_color: #x074D3E
                                            font_size: 12.
                                        }
                                    }
                                }
                                onboard_icon_pick := NavBtn{text: "Choose logo"}
                                // Legacy id (hot-reload): was a redundant "Tabiyat" under Choose logo.
                                h_product_locked_tabiyat := WrapLabel{
                                    body: ""
                                    visible: false
                                    font_color: #x023637
                                    font_size: 15.
                                }
                                View{
                                    width: Fill height: Fit
                                    flow: Right spacing: 0
                                    align: Align{y: 0.5}
                                    h_onboard_layout := WrapLabel{
                                                    width: Fit
                                        body: "What you get"
                                        font_color: #x023637
                                        font_size: 15.
                                    }
                                    onboard_tmpl_help := InfoHelp{
                                        tip: " "
                                    }
                                }
                                h_how_start := WrapLabel{
                                    body: "How do you want to start?"
                                    font_color: #x023637
                                    font_size: 15.
                                }
                                View{
                                    width: Fill height: 52
                                    flow: Overlay
                                    onboard_fresh := RowSecondaryBtn{text: "Start fresh"}
                                    onboard_fresh_on := RowPrimaryBtn{text: "Start fresh" visible: false}
                                }
                                // Whole-shop App Template (Rajdhani) is offered
                                // once on DeviceSetup right after create server —
                                // before Owner PIN. Menu-only imports stay here
                                // on Menu (Client menu / shelf).
                                View{
                                    width: Fill height: 52
                                    flow: Overlay
                                    onboard_book := RowSecondaryBtn{text: "Bring my book"}
                                    onboard_book_on := RowPrimaryBtn{text: "Bring my book" visible: false}
                                }
                                h_fresh_book_tabiyat := WrapLabel{
                                    body: ""
                                    font_color: #x074D3E
                                    font_size: 12.
                                    visible: false
                                }
                                onboard_fresh_panel := View{
                                    width: Fill height: Fit
                                    flow: Down spacing: 8
                                    visible: false
                                    h_fresh_selected := WrapLabel{
                                        body: "Fresh selected — catalogue stays empty. Tap Done when ready."
                                        font_color: #x074D3E
                                        font_size: 12.
                                    }
                                }
                                onboard_book_panel := View{
                                    width: Fill height: Fit
                                    flow: Down spacing: 8
                                    visible: false
                                    h_book_add_dish := WrapLabel{body: "Book — add dish / item" font_color: #x023637 font_size: 15.}
                                    onboard_item_name := FieldInput{empty_text: "Item name"}
                                    onboard_item_price := FieldInput{empty_text: "Price"}
                                    onboard_add_item := NavBtn{text: "Add item"}
                                    h_book_opening_credit := WrapLabel{body: "Book — opening credit" font_color: #x023637 font_size: 15.}
                                    onboard_cust_name := FieldInput{empty_text: "Customer name"}
                                    onboard_cust_bal := FieldInput{empty_text: "Amount owed"}
                                    onboard_add_credit := NavBtn{text: "Add opening balance"}
                                }
                                onboard_done := PinOkKey{text: "Done — open books"}
                                h_later_country_pack := WrapLabel{
                                    body: ""
                                    font_color: #x074D3E
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
                                // Auth header is hidden here — body Back calls navigate_back
                                // (→ DeviceSetup). Unlocked screens use header_back instead.
                                client_pair_back := BackBtn{text: "Back"}
                                View{
                                    width: Fill height: Fit
                                    flow: Right spacing: 0
                                    align: Align{y: 0.5}
                                    h_connect_to_server := WrapLabel{
                                        width: Fit
                                        body: "Connect to the Server"
                                        font_color: #x023637
                                        font_size: 22.
                                    }
                                    client_pair_help := InfoHelp{
                                        tip: "Find the shop's server on Wi‑Fi, or type its address and pairing code."
                                    }
                                }
                                client_pair_status := WrapLabel{
                                    body: " "
                                    font_color: #x074D3E
                                    font_size: 13.
                                }
                                // No typing IPs: the shop's server answers on this Wi‑Fi.
                                ActionStack{
                                    client_find_btn := StackPrimaryBtn{text: "Find the shop's server"}
                                    client_find_cancel := StackLightBtn{text: "Cancel" visible: false}
                                }
                                client_found_label := WrapLabel{
                                    body: " "
                                    font_color: #x074D3E
                                    font_size: 13.
                                }
                                client_found_list := mod.widgets.FoundServerList{}
                                client_host_input := FieldInput{empty_text: "Server IP (e.g. 192.168.1.10)"}
                                // 6-digit shop code — numeric pad on Android (same idea as Lock PIN).
                                client_code_input := FieldInput{
                                    empty_text: "Pairing code"
                                    is_numeric_only: true
                                }
                                // How the shop's server lists this device (Forget picks it by this).
                                client_name_input := FieldInput{empty_text: "This device's name"}
                                client_pair_btn := PinOkKey{text: "Pair now"}
                                client_pair_progress := WrapLabel{
                                    visible: false
                                    body: "Connecting…"
                                    font_color: #x023637
                                    font_size: 15.
                                }
                                client_pair_cancel := QuietFitBtn{text: "Cancel" visible: false}
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
                                    tip: "Today’s position. Sell = counter sales. Buy = purchases & expenses. Parties = khata customers & suppliers."
                                }
                                books_expiry_alert := WrapLabel{body: " " font_color: #x017B85 font_size: 14. visible: false}
                                books_low_stock := WrapLabel{body: " " font_color: #x023637 font_size: 14. visible: false}
                                books_backup_warning := WrapLabel{body: " " font_color: #x023637 font_size: 14. visible: false}
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
                                            h_money_in_sales := Label{width: Fill height: Fit flow: Right{wrap: true} text: "Money in (sales)" draw_text.color: #x074D3E draw_text.text_style.font_size: 12.}
                                            View{width: Fit height: Fit flow: Right spacing: 4 align: Align{y: 0.5}
                                            books_in_value := Label{width: Fit height: Fit text: "—" draw_text.color: #x023637 draw_text.text_style.font_size: 13.}
                                            books_in_eye := EyeIconBtn{}
                                        }
                                        }
                                        View{width: Fill height: Fit flow: Right spacing: 8 align: Align{y: 0.5}
                                            h_purchases_cogs := Label{width: Fill height: Fit flow: Right{wrap: true} text: "Purchases (COGS)" draw_text.color: #x074D3E draw_text.text_style.font_size: 12.}
                                            View{width: Fit height: Fit flow: Right spacing: 4 align: Align{y: 0.5}
                                            books_purchases_value := Label{width: Fit height: Fit text: "—" draw_text.color: #x023637 draw_text.text_style.font_size: 13.}
                                            books_purchases_eye := EyeIconBtn{}
                                        }
                                        }
                                        View{width: Fill height: Fit flow: Right spacing: 8 align: Align{y: 0.5}
                                            h_expenses_word := Label{width: Fill height: Fit flow: Right{wrap: true} text: "Expenses" draw_text.color: #x074D3E draw_text.text_style.font_size: 12.}
                                            View{width: Fit height: Fit flow: Right spacing: 4 align: Align{y: 0.5}
                                            books_expenses_value := Label{width: Fit height: Fit text: "—" draw_text.color: #x023637 draw_text.text_style.font_size: 13.}
                                            books_expenses_eye := EyeIconBtn{}
                                        }
                                        }
                                        View{width: Fill height: Fit flow: Right spacing: 8 align: Align{y: 0.5}
                                            h_gross_word := Label{width: Fill height: Fit flow: Right{wrap: true} text: "Gross" draw_text.color: #x074D3E draw_text.text_style.font_size: 12.}
                                            View{width: Fit height: Fit flow: Right spacing: 4 align: Align{y: 0.5}
                                            books_gross_value := Label{width: Fit height: Fit text: "—" draw_text.color: #x023637 draw_text.text_style.font_size: 13.}
                                            books_gross_eye := EyeIconBtn{}
                                        }
                                        }
                                        View{width: Fill height: Fit flow: Right spacing: 8 align: Align{y: 0.5}
                                            h_net_word := Label{width: Fill height: Fit flow: Right{wrap: true} text: "Net" draw_text.color: #x023637 draw_text.text_style.font_size: 13.}
                                            View{width: Fit height: Fit flow: Right spacing: 4 align: Align{y: 0.5}
                                            books_net_value := Label{width: Fit height: Fit text: "—" draw_text.color: #x023637 draw_text.text_style.font_size: 15.}
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
                                            books_declared_name := Label{width: Fill height: Fit flow: Right{wrap: true} text: "Tax invoice" draw_text.color: #x074D3E draw_text.text_style.font_size: 12.}
                                            View{width: Fit height: Fit flow: Right spacing: 4 align: Align{y: 0.5}
                                            books_declared_value := Label{width: Fit height: Fit text: "—" draw_text.color: #x023637 draw_text.text_style.font_size: 13.}
                                            books_declared_eye := EyeIconBtn{}
                                        }
                                        }
                                        View{width: Fill height: Fit flow: Right spacing: 8 align: Align{y: 0.5}
                                            books_owner_name := Label{width: Fill height: Fit flow: Right{wrap: true} text: "Cash memo" draw_text.color: #x074D3E draw_text.text_style.font_size: 12.}
                                            View{width: Fit height: Fit flow: Right spacing: 4 align: Align{y: 0.5}
                                            books_owner_value := Label{width: Fit height: Fit text: "—" draw_text.color: #x023637 draw_text.text_style.font_size: 13.}
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
                                            h_cash := Label{width: Fill height: Fit flow: Right{wrap: true} text: "Cash" draw_text.color: #x074D3E draw_text.text_style.font_size: 12.}
                                            View{width: Fit height: Fit flow: Right spacing: 4 align: Align{y: 0.5}
                                            books_cash_value := Label{width: Fit height: Fit text: "—" draw_text.color: #x023637 draw_text.text_style.font_size: 13.}
                                            books_cash_eye := EyeIconBtn{}
                                        }
                                        }
                                        View{width: Fill height: Fit flow: Right spacing: 8 align: Align{y: 0.5}
                                            books_digital_name := Label{width: Fill height: Fit flow: Right{wrap: true} text: "UPI" draw_text.color: #x074D3E draw_text.text_style.font_size: 12.}
                                            View{width: Fit height: Fit flow: Right spacing: 4 align: Align{y: 0.5}
                                            books_digital_value := Label{width: Fit height: Fit text: "—" draw_text.color: #x023637 draw_text.text_style.font_size: 13.}
                                            books_digital_eye := EyeIconBtn{}
                                        }
                                        }
                                        View{width: Fill height: Fit flow: Right spacing: 8 align: Align{y: 0.5}
                                            h_card := Label{width: Fill height: Fit flow: Right{wrap: true} text: "Card" draw_text.color: #x074D3E draw_text.text_style.font_size: 12.}
                                            View{width: Fit height: Fit flow: Right spacing: 4 align: Align{y: 0.5}
                                            books_card_value := Label{width: Fit height: Fit text: "—" draw_text.color: #x023637 draw_text.text_style.font_size: 13.}
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
                                            h_they_owe_us := Label{width: Fill height: Fit flow: Right{wrap: true} text: "They owe us" draw_text.color: #x074D3E draw_text.text_style.font_size: 12.}
                                            View{width: Fit height: Fit flow: Right spacing: 4 align: Align{y: 0.5}
                                            books_they_owe_value := Label{width: Fit height: Fit text: "—" draw_text.color: #x137333 draw_text.text_style.font_size: 13.}
                                            books_they_owe_eye := EyeIconBtn{}
                                        }
                                        }
                                        View{width: Fill height: Fit flow: Right spacing: 8 align: Align{y: 0.5}
                                            h_we_owe_them := Label{width: Fill height: Fit flow: Right{wrap: true} text: "We owe them" draw_text.color: #x074D3E draw_text.text_style.font_size: 12.}
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
                                            h_bank_balance := Label{width: Fill height: Fit flow: Right{wrap: true} text: "Bank balance" draw_text.color: #x074D3E draw_text.text_style.font_size: 12.}
                                            View{width: Fit height: Fit flow: Right spacing: 4 align: Align{y: 0.5}
                                            books_bank_balance_value := Label{width: Fit height: Fit text: "—" draw_text.color: #x023637 draw_text.text_style.font_size: 13.}
                                            books_bank_balance_eye := EyeIconBtn{}
                                        }
                                        }
                                        View{width: Fill height: Fit flow: Right spacing: 8 align: Align{y: 0.5}
                                            h_cash_in_hand := Label{width: Fill height: Fit flow: Right{wrap: true} text: "Cash in hand (till)" draw_text.color: #x074D3E draw_text.text_style.font_size: 12.}
                                            View{width: Fit height: Fit flow: Right spacing: 4 align: Align{y: 0.5}
                                            books_till_value := Label{width: Fit height: Fit text: "—" draw_text.color: #x023637 draw_text.text_style.font_size: 13.}
                                            books_till_eye := EyeIconBtn{}
                                        }
                                        }
                                        View{width: Fill height: Fit flow: Right spacing: 8 align: Align{y: 0.5}
                                            h_bank_net_today := Label{width: Fill height: Fit flow: Right{wrap: true} text: "Bank net today" draw_text.color: #x074D3E draw_text.text_style.font_size: 12.}
                                            View{width: Fit height: Fit flow: Right spacing: 4 align: Align{y: 0.5}
                                            books_bank_net_value := Label{width: Fit height: Fit text: "—" draw_text.color: #x023637 draw_text.text_style.font_size: 13.}
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
                                        otype_dinein := SegBtn{text: "Counter"}
                                        otype_dinein_on := SegBtnOn{text: "Counter" visible: false}
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
                                    scroll_bar_x.axis_lock: true
                                    scroll_bar_x.bar_size: 8.
                                }
                                pos_ticket_row := View{
                                    width: Fit height: Fill
                                    flow: Right spacing: 16
                                pos_scroll := ScrollYView{
                                    width: 600 height: Fill
                                    // Mouse drag over cart rows / buttons must still
                                    // pan the Order page (touch already co-captures).
                                    scroll_bars: mod.widgets.ScrollBars {
                                        show_scroll_x: false show_scroll_y: true
                                        scroll_bar_y: mod.widgets.ScrollBar{
                                            drag_scrolling: true
                                            mouse_capture_overload: true
                                            axis_lock: true
                                            bounce_at_start: false
                                            bounce_at_end: false
                                            bar_size: 10.
                                            min_handle_size: 40.
                                            draw_bg +: {
                                                size: 6.
                                                border_radius: 3.
                                                color: #xC9AFA2
                                                color_hover: #x023637
                                                color_drag: #x025B60
                                            }
                                        }
                                    }
                                    pos_page := View{
                                        width: Fill height: Fit
                                        flow: Down spacing: 10
                                        padding: Inset{top: 4. bottom: 32.}

                                    // Scrolls with the ticket — not sticky.
                                    sell_help := InfoHelp{
                                        tip: "Tap an item to add it."
                                    }
                                    sticky_total := Label{
                                        width: Fill height: Fit
                                        text: "Total · ₹0.00"
                                        draw_text.color: #x023637
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
                                            font_color: #x074D3E
                                            font_size: 12.
                                        }
                                        empty_catalog := WrapLabel{
                                            body: "No items yet. Add your dishes and items to start taking orders."
                                            font_color: #x023637
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
                                                font_color: #x023637
                                                font_size: 12.
                                            }
                                            table_chip := WrapLabel{
                                                body: "Ready to order"
                                                font_color: #x023637
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
                                                font_color: #x074D3E
                                                font_size: 13.
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
                                                font_color: #x074D3E
                                                font_size: 12.
                                            }

                                            // Right under the lines just punched: pick a
                                            // station for dishes that have none, then send.
                                            kot_dest_help := InfoHelp{
                                                tip: " "
                                                visible: false
                                            }
                                            kot_station_wrap := View{
                                                width: Fill height: Fit
                                                visible: false
                                                flow: Down spacing: 6
                                                // Dropdown — opens checkbox station list (not a type-in field).
                                                kot_station_dd := StackSecondaryBtn{text: "Sections" flow: Right}
                                                // TextInput has no `visible` — collapse height; wrap hides the strip.
                                                kot_station_input := FieldInput{
                                                    height: 0.
                                                    margin: Inset{left: 0. right: 0. top: 0. bottom: 0.}
                                                    padding: Inset{left: 0. right: 0. top: 0. bottom: 0.}
                                                    empty_text: "Sections"
                                                }
                                                kot_station_suggest := mod.widgets.StationCheckList{field: "kot_station"}
                                            }
                                            send_stack := ActionStack{
                                                kot_set_station := StackLightBtn{text: "Use this station" visible: false}
                                                send_kot_btn := StackPrimaryBtn{text: "Send slip"}
                                                // Next step once the kitchen has everything:
                                                // the bill, without opening Table actions.
                                                print_bill_quick := StackSecondaryBtn{text: "Print bill" visible: false}
                                                reprint_kot_btn := StackLightBtn{text: "Print selected again" visible: false}
                                                batch_cancel_btn := StackLightBtn{text: "Cancel marked" visible: false}
                                                print_cancelled_kot_btn := StackLightBtn{text: "Print all cancelled" visible: false}
                                                next_order_btn := StackLightBtn{text: "Next order (clear desk)" visible: false}
                                            }
                                        }
                                        // Foot of the ticket column: Print bill · Move /
                                        // merge · split · coursing — under the lines and
                                        // Next order so the cart stays the hero.
                                        table_actions_wrap := RoundedView{
                                            width: Fill height: Fit
                                            visible: false
                                            padding: Inset{left: 14. right: 14. top: 10. bottom: 10.}
                                            flow: Down spacing: 7
                                            draw_bg.color: #xffffff
                                            draw_bg.border_radius: 14.
                                            table_actions_hdr := AccordionHdrBtn{text: "TABLE ACTIONS"}
                                            table_actions_body := View{
                                                width: Fill height: Fit
                                                flow: Down spacing: 8
                                                visible: false
                                                table_actions := ActionStack{
                                                    print_bill_btn := StackLightBtn{text: "Print bill"}
                                                    table_move_btn := StackLightBtn{text: "Move / merge table"}
                                                    table_split_btn := StackLightBtn{text: "Split bill"}
                                                    split_evenly_btn := StackLightBtn{text: "Split evenly"}
                                                    fire_course_btn := StackLightBtn{text: "Fire course"}
                                                    hold_item_btn := StackLightBtn{text: "Hold item"}
                                                    waiter_call_btn := StackLightBtn{text: "Call waiter"}
                                                }
                                            }
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
                                            draw_bg.border_color: #x017B85
                                            View{
                                                width: Fill height: Fit
                                                flow: Right spacing: 0
                                                align: Align{x: 0.0 y: 0.5}
                                                h_split_bill := WrapLabel{
                                                    width: Fit
                                                    body: "Split bill"
                                                    font_color: #x023637
                                                    font_size: 14.
                                                }
                                                split_bill_help := InfoHelp{
                                                    tip: "Tap the dishes one guest pays for, then make their separate bill."
                                                }
                                            }
                                            split_help := WrapLabel{
                                                body: " "
                                                font_color: #x023637
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
                                            draw_bg.border_color: #x017B85
                                            View{
                                                width: Fill height: Fit
                                                flow: Right spacing: 0
                                                align: Align{x: 0.0 y: 0.5}
                                                h_move_table := WrapLabel{
                                                    width: Fit
                                                    body: "Move / merge"
                                                    font_color: #x023637
                                                    font_size: 14.
                                                }
                                                move_table_help := InfoHelp{
                                                    tip: "Move this table's order to a free table, or merge it into a running one."
                                                }
                                            }
                                            move_help := WrapLabel{
                                                body: "Move this table's order to a free table, or merge it into a running one."
                                                font_color: #x023637
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
                                            draw_bg.border_color: #x017B85
                                            line_action_title := WrapLabel{
                                                body: "Cancel item"
                                                font_color: #x025B60
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
                                                draw_text.color: #x023637
                                                draw_text.text_style.font_size: 18.
                                            }
                                            cart_charge_caption := WrapLabel{
                                                selectable: false
                                                body: "ROUNDED"
                                                font_color: #x017B85
                                                font_size: 10.
                                            }
                                        }
                                        cart_total_input := CartField{
                                            width: 140
                                            empty_text: "0"
                                            text: "0"
                                            draw_bg +: {
                                                border_color: #x017B85
                                                border_color_hover: #x822801
                                                border_color_focus: #x025B60
                                                border_radius: 8.
                                                border_size: 1.
                                            }
                                            draw_text +: {
                                                color: #x822801
                                                color_hover: #x822801
                                                color_down: #x822801
                                                color_focus: #x822801
                                                color_empty: #x56575D
                                                color_empty_hover: #x56575D
                                                color_empty_focus: #x56575D
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
                                                    font_color: #x074D3E
                                                    font_size: 10.
                                                }
                                                cart_natural_label := WrapLabel{
                                                    body: "0"
                                                    font_color: #x023637
                                                    font_size: 14.
                                                }
                                            }
                                            View{
                                                width: Fill height: Fit
                                                flow: Down spacing: 2
                                                align: Align{x: 0.5}
                                                h_applied_total_cap := WrapLabel{
                                                    body: "APPLIED TOTAL"
                                                    font_color: #x074D3E
                                                    font_size: 10.
                                                }
                                                cart_applied_label := WrapLabel{
                                                    body: "0"
                                                    font_color: #x023637
                                                    font_size: 14.
                                                }
                                            }
                                        }
                                    }
                                    // Exclusive-tax shops quote net prices, so the strip above is
                                    // not what the customer pays — this line is.
                                    tax_estimate_label := WrapLabel{
                                        body: " "
                                        font_color: #x017B85
                                        font_size: 13.
                                    }

                                    // Bill discount: 10% or an amount, always with a reason;
                                    // a manager's PIN when the person's right asks for it.
                                    discount_label := WrapLabel{
                                        body: " "
                                        font_color: #x025B60
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
                                            flow: Right spacing: 0
                                            align: Align{x: 0.0 y: 0.5}
                                            h_split_payment := WrapLabel{
                                                    width: Fit
                                                body: "Split"
                                                font_color: #x023637
                                                font_size: 14.
                                            }
                                            payment_help := InfoHelp{
                                                tip: "Split across cash / UPI / card / credit if needed. Credit needs a customer above."
                                            }
                                        }
                                        h_mix_type_hint_tabiyat := WrapLabel{
                                            body: "Split across cash / UPI / card / credit if needed. Credit needs a customer above."
                                            font_color: #x074D3E
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
                                            font_color: #x023637
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
                                        font_color: #x023637
                                        font_size: 12.
                                        visible: false
                                    }
                                    credit_chip := WrapLabel{
                                        body: "No customer yet — type a name, then tap Credit"
                                        font_color: #x023637
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
                                        font_color: #x023637
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
                                            font_color: #x025B60
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
                                    draw_bg.color: #x023637
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
                                            color: #x025B60
                                            color_hover: #x025B60
                                            color_down: #x025B60
                                            color_focus: #x025B60
                                            text_style +: {font_size: 14.}
                                        }
                                    }
                                }
                                floor_help := InfoHelp{
                                    tip: "Tap a table to open its order. Icons on the card: seat · free · bill · print · view. Active table uses a stronger border."
                                }
                                // Kept hidden — tile border marks the active table; icons are on the card.
                                floor_toolbar := RoundedView{
                                    width: Fill height: Fit
                                    visible: false
                                    padding: Inset{left: 12. right: 12. top: 10. bottom: 10.}
                                    flow: Right spacing: 10
                                    align: Align{x: 0.0 y: 0.0}
                                    draw_bg.color: #xFFF6EE
                                    draw_bg.border_radius: 14.
                                    draw_bg.border_size: 1.
                                    draw_bg.border_color: #xE3D3C8
                                    // Wrap — never crop a long table name beside the icons.
                                    floor_tb_name := WrapLabel{
                                        width: Fill
                                        body: ""
                                        font_color: #x023637
                                        font_size: 15.
                                    }
                                    floor_tb_print := FloorTileIconBtn{text: "\u{f02f}"}
                                    floor_tb_view := FloorTileIconBtn{text: "\u{f06e}"}
                                    floor_tb_open := FloorTileIconBtn{text: "\u{f07c}"}
                                }
                                floor_bill_view := RoundedView{
                                    width: Fill height: Fit
                                    visible: false
                                    padding: Inset{left: 8. right: 8. top: 8. bottom: 12.}
                                    flow: Down spacing: 8
                                    draw_bg.color: #xffffff
                                    draw_bg.border_radius: 14.
                                    draw_bg.border_size: 1.
                                    draw_bg.border_color: #xE3D3C8
                                    // Same ReportText layout as printed day books —
                                    // WrapLabel flattened newlines into one paragraph.
                                    floor_bill_text := mod.widgets.ReportText{}
                                    floor_bill_close := StackSecondaryBtn{text: "Close bill" flow: Right}
                                }
                                // At a glance: busy tables, bills to collect, money on
                                // the floor.
                                floor_summary := WrapLabel{
                                    body: " "
                                    font_color: #x3A2C26
                                    font_size: 14.
                                }
                                // Colour key — one line; scroll sideways if the
                                // labels do not fit (never wrap to a second line).
                                floor_legend := ScrollXView{
                                    width: Fill height: Fit
                                    padding: Inset{bottom: 4.}
                                    scroll_bars: mod.widgets.ScrollBars {
                                        show_scroll_x: true show_scroll_y: false
                                        scroll_bar_x: mod.widgets.ScrollBar{
                                            drag_scrolling: true
                                            mouse_capture_overload: true
                                            axis_lock: true
                                            bounce_at_start: false
                                            bounce_at_end: false
                                            bar_size: 6.
                                            min_handle_size: 24.
                                            draw_bg +: {
                                                size: 4.
                                                border_radius: 2.
                                                color: #xC9AFA2
                                                color_hover: #x023637
                                                color_drag: #x025B60
                                            }
                                        }
                                    }
                                    View{
                                        width: Fit height: Fit
                                        flow: Right{wrap: false}
                                        spacing: 14
                                        align: Align{y: 0.5}
                                        View{
                                            width: Fit height: Fit
                                            flow: Right spacing: 6
                                            align: Align{y: 0.5}
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
                                }
                                // Areas as a sideways tab strip (‹ › when needed).
                                // Each tab's body below = that area's Columns + tables.
                                floor_area_strip := View{
                                    width: Fill height: Fit
                                    flow: Overlay
                                    visible: false
                                    tabs := ChipStrip{
                                        ChipStripRow{
                                            View{width: Fit height: 40 flow: Overlay
                                                floor_area_0 := ChipBtn{text: " "}
                                                floor_area_0_on := ChipBtnActive{text: " " visible: false}
                                            }
                                            View{width: Fit height: 40 flow: Overlay
                                                floor_area_1 := ChipBtn{text: " "}
                                                floor_area_1_on := ChipBtnActive{text: " " visible: false}
                                            }
                                            View{width: Fit height: 40 flow: Overlay
                                                floor_area_2 := ChipBtn{text: " "}
                                                floor_area_2_on := ChipBtnActive{text: " " visible: false}
                                            }
                                            View{width: Fit height: 40 flow: Overlay
                                                floor_area_3 := ChipBtn{text: " "}
                                                floor_area_3_on := ChipBtnActive{text: " " visible: false}
                                            }
                                            View{width: Fit height: 40 flow: Overlay
                                                floor_area_4 := ChipBtn{text: " "}
                                                floor_area_4_on := ChipBtnActive{text: " " visible: false}
                                            }
                                            View{width: Fit height: 40 flow: Overlay
                                                floor_area_5 := ChipBtn{text: " "}
                                                floor_area_5_on := ChipBtnActive{text: " " visible: false}
                                            }
                                            View{width: Fit height: 40 flow: Overlay
                                                floor_area_6 := ChipBtn{text: " "}
                                                floor_area_6_on := ChipBtnActive{text: " " visible: false}
                                            }
                                            View{width: Fit height: 40 flow: Overlay
                                                floor_area_7 := ChipBtn{text: " "}
                                                floor_area_7_on := ChipBtnActive{text: " " visible: false}
                                            }
                                            View{width: Fit height: 40 flow: Overlay
                                                floor_area_8 := ChipBtn{text: " "}
                                                floor_area_8_on := ChipBtnActive{text: " " visible: false}
                                            }
                                            View{width: Fit height: 40 flow: Overlay
                                                floor_area_9 := ChipBtn{text: " "}
                                                floor_area_9_on := ChipBtnActive{text: " " visible: false}
                                            }
                                            View{width: Fit height: 40 flow: Overlay
                                                floor_area_10 := ChipBtn{text: " "}
                                                floor_area_10_on := ChipBtnActive{text: " " visible: false}
                                            }
                                            View{width: Fit height: 40 flow: Overlay
                                                floor_area_11 := ChipBtn{text: " "}
                                                floor_area_11_on := ChipBtnActive{text: " " visible: false}
                                            }
                                            View{width: Fit height: 40 flow: Overlay
                                                floor_area_12 := ChipBtn{text: " "}
                                                floor_area_12_on := ChipBtnActive{text: " " visible: false}
                                            }
                                            View{width: Fit height: 40 flow: Overlay
                                                floor_area_13 := ChipBtn{text: " "}
                                                floor_area_13_on := ChipBtnActive{text: " " visible: false}
                                            }
                                            View{width: Fit height: 40 flow: Overlay
                                                floor_area_14 := ChipBtn{text: " "}
                                                floor_area_14_on := ChipBtnActive{text: " " visible: false}
                                            }
                                            View{width: Fit height: 40 flow: Overlay
                                                floor_area_15 := ChipBtn{text: " "}
                                                floor_area_15_on := ChipBtnActive{text: " " visible: false}
                                            }
                                            View{width: Fit height: 40 flow: Overlay
                                                floor_area_16 := ChipBtn{text: " "}
                                                floor_area_16_on := ChipBtnActive{text: " " visible: false}
                                            }
                                            View{width: Fit height: 40 flow: Overlay
                                                floor_area_17 := ChipBtn{text: " "}
                                                floor_area_17_on := ChipBtnActive{text: " " visible: false}
                                            }
                                            View{width: Fit height: 40 flow: Overlay
                                                floor_area_18 := ChipBtn{text: " "}
                                                floor_area_18_on := ChipBtnActive{text: " " visible: false}
                                            }
                                            View{width: Fit height: 40 flow: Overlay
                                                floor_area_19 := ChipBtn{text: " "}
                                                floor_area_19_on := ChipBtnActive{text: " " visible: false}
                                            }
                                            View{width: Fit height: 40 flow: Overlay
                                                floor_area_20 := ChipBtn{text: " "}
                                                floor_area_20_on := ChipBtnActive{text: " " visible: false}
                                            }
                                            View{width: Fit height: 40 flow: Overlay
                                                floor_area_21 := ChipBtn{text: " "}
                                                floor_area_21_on := ChipBtnActive{text: " " visible: false}
                                            }
                                            View{width: Fit height: 40 flow: Overlay
                                                floor_area_22 := ChipBtn{text: " "}
                                                floor_area_22_on := ChipBtnActive{text: " " visible: false}
                                            }
                                            View{width: Fit height: 40 flow: Overlay
                                                floor_area_23 := ChipBtn{text: " "}
                                                floor_area_23_on := ChipBtnActive{text: " " visible: false}
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
                                // Columns + tables for the selected area. spacing: 0
                                // so the Columns↔tables whitespace is *inside*
                                // area_scroll's top pad (not a dead parent gap) —
                                // press-drag there pans with the Area. Keep the
                                // visual gap; see FloorList area_scroll padding.
                                floor_area_body := View{
                                    width: Fill height: Fit
                                    flow: Down spacing: 0
                                    // Columns + zoom (− / + / Auto) for this area (this device).
                                    // The pick list lives in floor_cols_dd_modal (not here)
                                    // so opening it cannot stretch this row or shove the grid.
                                    // Start side of the body (physical left). Window
                                    // chrome is not mirrored for RTL (DEV.md); only
                                    // the field's text + chevron trailing edge flip
                                    // in paint_floor_cols_field_dir.
                                    floor_cols_row := View{
                                        width: Fill height: Fit
                                        flow: Right spacing: 8
                                        align: Align{x: 0.0 y: 0.5}
                                        padding: Inset{top: 2. bottom: 2.}
                                        floor_cols_box := View{
                                            width: Fixed(176.) height: Fit
                                            flow: Overlay
                                            align: Align{x: 0.0 y: 0.5}
                                            floor_cols_input := FieldInput{
                                                margin: Inset{left: 0. right: 0. top: 2. bottom: 2.}
                                                // Symmetric side pad so the chevron
                                                // (trailing edge: right LTR / left RTL)
                                                // never sits on the placeholder.
                                                padding: Inset{left: 36. right: 36. top: 11. bottom: 11.}
                                                empty_text: "Columns"
                                                is_numeric_only: true
                                            }
                                            floor_cols_chev_row := View{
                                                width: Fill height: Fit
                                                flow: Right
                                                align: Align{x: 1.0 y: 0.5}
                                                floor_cols_chev := Label{
                                                    width: Fit
                                                    margin: Inset{right: 14.}
                                                    text: "\u{f078}"
                                                    draw_text.color: #x074D3E
                                                    draw_text.text_style.font_size: 11.
                                                }
                                            }
                                        }
                                        View{
                                            width: Fixed(44.) height: Fit
                                            floor_zoom_out := SegBtn{text: "\u{f068}"}
                                        }
                                        View{
                                            width: Fixed(44.) height: Fit
                                            floor_zoom_in := SegBtn{text: "\u{f067}"}
                                        }
                                        View{
                                            width: Fixed(72.) height: Fit
                                            flow: Overlay
                                            floor_zoom_auto := SegBtn{text: "Auto"}
                                            floor_zoom_auto_on := SegBtnOn{text: "Auto" visible: false}
                                        }
                                    }
                                    // Tables for the selected area; sideways ScrollXView
                                    // inside FloorList when columns exceed the viewport.
                                    floor_scroll := View{
                                        width: Fill height: Fit
                                        padding: Inset{bottom: 8.}
                                        floor_list := mod.widgets.FloorList{}
                                    }
                                }
                                // Drawn-layout editor removed from Floor chrome —
                                // widgets kept off-screen so old ids still resolve.
                                floor_layout_list := mod.widgets.FloorLayoutList{visible: false}
                                floor_layout_editor := View{
                                    width: Fill height: Fit
                                    flow: Down spacing: 8
                                    visible: false
                                    floor_layout_edit_heading := WrapLabel{
                                        body: "Layout"
                                        font_color: #x023637
                                        font_size: 15.
                                    }
                                    FormFieldRow{
                                        layout_x_input := FieldInput{empty_text: "X"}
                                        layout_y_input := FieldInput{empty_text: "Y"}
                                    }
                                    FormFieldRow{
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
                                        waitlist_heading := WrapLabel{body: "Waitlist" font_color: #x023637 font_size: 15. visible: false}
                                        FormCard{
                                            wait_name_input := FieldInput{empty_text: "Customer name"}
                                            FormFieldRow{
                                                wait_party_input := FieldInput{empty_text: "Party size"}
                                                wait_mins_input := FieldInput{empty_text: "Wait (min)"}
                                            }
                                            ActionStack{
                                                waitlist_add := StackPrimaryBtn{text: "Add to waitlist"}
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
                                    resv_hdr := AccordionHdrBtn{text: "RESERVATIONS"}
                                    resv_body := View{
                                        width: Fill height: Fit
                                        flow: Down spacing: 8
                                        visible: false
                                        resv_heading := WrapLabel{body: "Reservations" font_color: #x023637 font_size: 15. visible: false}
                                        resv_list := mod.widgets.ResvList{}
                                        resv_empty := WrapLabel{body: " " font_color: #x074D3E font_size: 13.}
                                        FormCard{
                                            FormFieldRow{
                                                resv_name_input := FieldInput{empty_text: "Customer name"}
                                                resv_phone_input := FieldInput{empty_text: "Phone"}
                                            }
                                            FormFieldRow{
                                                resv_guests_input := FieldInput{empty_text: "People"}
                                                resv_when_input := FieldInput{empty_text: "When"}
                                            }
                                            resv_table_input := FieldInput{empty_text: "Table (optional)"}
                                            resv_table_suggest := mod.widgets.SuggestList{field: "resv_table"}
                                            ActionStack{
                                                resv_add := StackPrimaryBtn{text: "Book"}
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
                                    View{
                                        width: Fill height: Fit
                                        flow: Right spacing: 0
                                        align: Align{y: 0.5}
                                        floor_add_hdr := AccordionHdrHitBtn{width: Fit text: "ADD TABLES"}
                                        floor_areas_help := InfoHelp{
                                            tip: "Areas are optional shop zones. Counter shops usually leave this unused."
                                        }
                                        floor_add_hdr_ext := AccordionHdrHitBtn{text: ""}
                                    }
                                    floor_add_body := View{
                                        width: Fill height: Fit
                                        flow: Down spacing: 8
                                        visible: false
                                        floor_add_heading := WrapLabel{
                                            width: Fit
                                            body: "Add a table"
                                            font_color: #x023637
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
                                        // Fresh shops start with T1..T24 + Parcel P1..P24;
                                        // a floor of its own stays clear after Remove.
                                        table_remove_input := FieldInput{empty_text: "Remove: T1-T24, P1-P24, or one name"}
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
                                    tip: "Ticket board — unused on a counter shop."
                                }
                                // All-day counts: how much of each dish is still to cook.
                                kitchen_counts := WrapLabel{body: " " font_color: #x025B60 font_size: 15. visible: false}
                                kitchen_list := mod.widgets.KitchenList{}
                                kitchen_empty := WrapLabel{
                                    body: "No open slips."
                                    font_color: #x074D3E
                                    font_size: 13.
                                }
                                recently_done_heading := WrapLabel{body: "Recently done" font_color: #x023637 font_size: 15. visible: false}
                                recently_done_label := mod.widgets.ReportText{}
                                recall_kot_input := FieldInput{empty_text: "Slip number to recall"}
                                recall_kot_btn := NavBtn{text: "Recall"}
                                RoundedView{
                                    width: Fill height: Fit
                                    padding: Inset{left: 14. right: 14. top: 10. bottom: 10.}
                                    flow: Down spacing: 7
                                    draw_bg.color: #xffffff
                                    draw_bg.border_radius: 14.
                                    View{
                                        width: Fill height: Fit
                                        flow: Right spacing: 0
                                        align: Align{y: 0.5}
                                        stations_hdr := AccordionHdrHitBtn{width: Fit text: "KITCHEN STATIONS"}
                                        stations_help := InfoHelp{
                                            tip: "Stations — unused on a counter shop. Bill printers live under Settings → Printers."
                                        }
                                        stations_hdr_ext := AccordionHdrHitBtn{text: ""}
                                    }
                                    stations_body := View{
                                        width: Fill height: Fit
                                        flow: Down spacing: 8
                                        visible: false
                                        stations_heading := WrapLabel{
                                            width: Fit
                                            body: "Dispensary sections"
                                            font_color: #x023637
                                            font_size: 15.
                                            visible: false
                                        }
                                        stations_label := mod.widgets.ReportText{}
                                        station_add_heading := WrapLabel{body: "Add a dispensary section" font_color: #x023637 font_size: 15.}
                                        station_name_input := FieldInput{empty_text: "Section name (e.g. OTC, Fridge, Counter)"}
                                        station_add := StackPrimaryBtn{text: "Add station"}
                                        station_remove_heading := WrapLabel{body: "Remove a dispensary section" font_color: #x023637 font_size: 15.}
                                        station_remove_input := FieldInput{empty_text: "Tap a station in the list to remove it"}
                                        station_remove_suggest := mod.widgets.SuggestList{field: "station_remove"}
                                        // Pick removes immediately; button kept for typed + tap.
                                        station_remove := StackSecondaryBtn{text: "Remove station"}
                                        View{
                                            width: Fill height: Fit
                                            flow: Right spacing: 0
                                            align: Align{y: 0.5}
                                            h_station_print := WrapLabel{
                                                width: Fit
                                                body: "Printers for this station"
                                                font_color: #x023637
                                                font_size: 15.
                                            }
                                            station_print_help := InfoHelp{
                                                tip: "Pick a station, then attach one or more printers. ↑↓ sets try order. First that works stops after the first success; Print on all sends to every attached printer."
                                            }
                                        }
                                        station_print_input := FieldInput{empty_text: "Station to set printers"}
                                        station_print_suggest := mod.widgets.SuggestList{field: "station_print"}
                                        station_print_mode_row := View{
                                            width: Fill height: Fit
                                            flow: Right spacing: 6
                                            View{width: Fill height: Fit flow: Overlay
                                                station_print_first := SegBtn{text: "First that works"}
                                                station_print_first_on := SegBtnOn{text: "First that works" visible: false}
                                            }
                                            View{width: Fill height: Fit flow: Overlay
                                                station_print_all := SegBtn{text: "Print on all"}
                                                station_print_all_on := SegBtnOn{text: "Print on all" visible: false}
                                            }
                                        }
                                        ActionStack{
                                            station_vps_dd := StackSecondaryBtn{text: "Printers for this station" flow: Down}
                                        }
                                        station_vps_input := FieldInput{
                                            height: 0.
                                            margin: Inset{left: 0. right: 0. top: 0. bottom: 0.}
                                            padding: Inset{left: 0. right: 0. top: 0. bottom: 0.}
                                            empty_text: ""
                                        }
                                        station_vps_suggest := mod.widgets.StationCheckList{field: "station_vps"}
                                        station_print_summary := WrapLabel{
                                            width: Fill
                                            body: ""
                                            font_color: #x074D3E
                                            font_size: 12.
                                        }
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
                                    tip: "Tap an item to edit it. New item clears the form. Search our dishes or type your own name and photo, set a price (0 is fine), then Add. Sales already recorded keep the name they were charged with."
                                }
                                // Form errors and confirmations have to be visible
                                // where the form is: this screen had no status line,
                                // so "Name required" / "Set a price …" failed
                                // silently (2026-10-22).
                                menu_status := WrapLabel{
                                    body: " "
                                    font_color: #x025B60
                                    font_size: 13.
                                }
                                // The whole provided menu in one go: every category and
                                // dish we ship, with photos; prices are the owner's.
                                template_card := FormCard{
                                    template_heading := Label{
                                        text: "Our menu template"
                                        draw_text.color: #x023637
                                        draw_text.text_style.font_size: 17.
                                    }
                                    template_info := Label{
                                        width: Fill
                                        text: " "
                                        flow: Right{wrap: true}
                                        draw_text.color: #x074D3E
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
                                // Named client menus (assets/templates/<id>/). One card;
                                // chips pick which template when more than one ships.
                                client_template_card := FormCard{
                                    client_template_heading := Label{
                                        text: "Client menu"
                                        draw_text.color: #x023637
                                        draw_text.text_style.font_size: 17.
                                    }
                                    client_template_chips := View{
                                        width: Fill height: Fit
                                        flow: Right{wrap: true} spacing: 8
                                        // Fixed chip slots — paint shows as many as we ship
                                        // (add another LiveId here when a fourth template arrives).
                                        // ChipBtn / ChipBtnActive — same faces as category strip
                                        // (ChoiceChip was never defined → script errors every boot).
                                        client_tmpl_0 := ChipBtn{text: " " visible: false}
                                        client_tmpl_0_on := ChipBtnActive{text: " " visible: false}
                                        client_tmpl_1 := ChipBtn{text: " " visible: false}
                                        client_tmpl_1_on := ChipBtnActive{text: " " visible: false}
                                        client_tmpl_2 := ChipBtn{text: " " visible: false}
                                        client_tmpl_2_on := ChipBtnActive{text: " " visible: false}
                                    }
                                    client_template_info := Label{
                                        width: Fill
                                        text: " "
                                        flow: Right{wrap: true}
                                        draw_text.color: #x074D3E
                                        draw_text.text_style.font_size: 13.
                                    }
                                    client_template_import := NavBtn{text: "Import this menu"}
                                    client_template_confirm := View{
                                        width: Fill height: Fit
                                        visible: false
                                        flow: Right spacing: 8
                                        client_template_yes := StackPrimaryBtn{text: "Yes, import"}
                                        client_template_no := StackLightBtn{text: "Cancel"}
                                    }
                                }
                                menu_mode_label := WrapLabel{body: "Adding a new item" font_color: #x025B60 font_size: 14.}
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
                                        menu_kind_dish := SegBtn{text: "Don't count stock"}
                                        menu_kind_dish_on := SegBtnOn{text: "Don't count stock" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        menu_kind_resale := SegBtn{text: "Bought medicine"}
                                        menu_kind_resale_on := SegBtnOn{text: "Bought medicine" visible: false}
                                    }
                                }
                                menu_aliases_input := FieldInput{
                                    empty_text: "Aliases — other names, comma separated"
                                }
                                FormFieldRow{
                                    menu_barcode_input := FieldInput{empty_text: "Barcode / EAN"}
                                    menu_hsn_input := FieldInput{empty_text: "HSN (optional)"}
                                    menu_composition_input := FieldInput{empty_text: "Salt / composition (searchable)"}
                                }
                                View{
                                    width: Fill height: Fit
                                    flow: Down spacing: 8
                                    View{width: Fill height: Fit flow: Overlay
                                        menu_schedule_h := SegBtn{text: "Schedule H — off (owner flag)"}
                                        menu_schedule_h_on := SegBtnOn{text: "Schedule H — on (owner flag)" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        menu_rx_required := SegBtn{text: "Rx expected — off (owner flag)"}
                                        menu_rx_required_on := SegBtnOn{text: "Rx expected — on (owner flag)" visible: false}
                                    }
                                }
                                menu_rx_note_input := FieldInput{
                                    empty_text: "Prescription note (optional — your wording on the bill)"
                                }
                                FormFieldRow{
                                    menu_price_input := FieldInput{empty_text: "Price"}
                                    menu_unit_input := FieldInput{empty_text: "Unit (blank = each)"}
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
                                menu_station_input := FieldInput{empty_text: "Shelf / location (optional)"}
                                menu_station_suggest := mod.widgets.SuggestList{field: "menu_station"}
                                // Variations (* pick one) and add-ons (+ multi), or a flat add-on list.
                                menu_modifiers_input := FieldInput{empty_text: "Options — Size*: Small=199, Large=399 | Toppings+3: Cheese=40"}
                                // Quick entry: Half / Full become items of their own
                                // (own UUID, price, stock, station), grouped by name.
                                menu_variations_input := FieldInput{empty_text: "Sizes as separate items (optional) — Half=120, Full=200"}
                                menu_group_input := FieldInput{empty_text: "Strength / pack group (optional)"}
                                menu_group_suggest := mod.widgets.SuggestList{field: "menu_group"}
                                // A dearer area (AC) or a parcel price; other areas pay
                                // the item price.
                                menu_area_prices_input := FieldInput{empty_text: "Alt prices (optional)"}
                                menu_schedule_input := FieldInput{empty_text: "Hours — Mon-Fri 07:00-11:00 or Evening 17:00-19:00=150"}
                                // What one portion takes off the shelf.
                                menu_recipe_input := FieldInput{empty_text: "Recipe (optional — unused on pharmacy counter)"}
                                View{
                                    width: Fill height: Fit
                                    flow: Right spacing: 0
                                    align: Align{y: 0.5}
                                    h_item_photo := WrapLabel{
                                                    width: Fit
                                        body: "Photo"
                                        font_color: #x023637
                                        font_size: 15.
                                    }
                                    item_photo_help := InfoHelp{
                                        tip: "Optional. Without a photo we show initials from the item name."
                                    }
                                }
                                image_status := WrapLabel{
                                    body: " "
                                    font_color: #x074D3E
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
                                    font_color: #x023637
                                    font_size: 14.
                                }
                                // A full menu is hundreds of dishes: find one by name.
                                menu_list_search := FieldInput{empty_text: "Find in your menu"}
                                // Shown while dishes have no price yet (a fresh
                                // template import): how many, and a filter to
                                // price them one after another.
                                menu_unpriced_note := WrapLabel{
                                    body: " "
                                    font_color: #x023637
                                    font_size: 13.
                                    visible: false
                                }
                                menu_unpriced_btn := RowSecondaryBtn{text: "Show only dishes without a price" visible: false}
                                product_list_menu := mod.widgets.MenuList{}
                                // Remove is an archive, not a delete — these are
                                // the items taken off the menu, restorable
                                // exactly as they were. Hidden while none are.
                                h_removed_items := WrapLabel{body: "Removed items" font_color: #x023637 font_size: 15.}
                                removed_help := WrapLabel{
                                    body: "Restore puts the item back on the menu exactly as it was — price, photo and category. Our shelf of dishes is never removed, so a dish can always be added again."
                                    font_color: #x074D3E
                                    font_size: 12.
                                }
                                removed_list := mod.widgets.RemovedList{}
                                h_your_categories := WrapLabel{body: "Your categories" font_color: #x023637 font_size: 15.}
                                categories_help := WrapLabel{
                                    width: Fit
                                    body: "Tap Use to file the item above under that category. Photo gives the category a picture; without one it shows its initials."
                                    font_color: #x074D3E
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
                                    tip: "Expense vouchers and supplier bills (money out). On-account stays on the party ledger."
                                }
                                buy_low_stock := WrapLabel{body: " " font_color: #x023637 font_size: 14. visible: false}
                                FormCard{
                                    h_goods_received := WrapLabel{body: "Goods received" font_color: #x023637 font_size: 17.}
                                    buy_item_input := FieldInput{empty_text: "Item name"}
                                    buy_item_suggest := mod.widgets.SuggestList{field: "buy_item"}
                                    FormFieldRow{
                                        buy_qty_input := FieldInput{empty_text: "Qty"}
                                        buy_unit_input := FieldInput{empty_text: "Unit"}
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
                                    FormFieldRow{
                                        buy_batch_input := FieldInput{empty_text: "Batch no. (optional)"}
                                        buy_expiry_input := FieldInput{empty_text: "Expiry YYYY-MM-DD (optional)"}
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
                                    h_buy_expense := WrapLabel{body: "Expense" font_color: #x023637 font_size: 17.}
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
                                            draw_text.color: #x074D3E
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
                                                color: #x025B60
                                                color_hover: #x025B60
                                                color_down: #x025B60
                                                color_focus: #x025B60
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
                                    tip: "Select a sale to amend, void, or refund. Amendments keep full history. Rounded Total starts equal to Applied — edit only to round. Void and refund may need an Owner or Manager PIN."
                                }
                                // The day's sales first; the picked sale's details
                                // (copy, print, amend, void, refund, notes) under them.
                                sale_list := mod.widgets.SaleList{}
                                sale_detail := FormCard{
                                    visible: false
                                    sale_selected_label := WrapLabel{
                                        body: "No sale selected"
                                        font_color: #x023637
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
                                        font_color: #x023637
                                        font_size: 13.
                                    }
                                    amend_applied_label := WrapLabel{
                                        body: "Applied Total 0"
                                        font_color: #x023637
                                        font_size: 13.
                                    }
                                        View{
                                            width: Fill height: Fit
                                            flow: Right spacing: 8
                                            align: Align{y: 0.5}
                                            h_rounded_total := WrapLabel{
                                                body: "Rounded Total"
                                                font_color: #x017B85
                                                font_size: 14.
                                            }
                                            amend_total_input := CartField{
                                            padding: Inset{left: 14. right: 14. top: 14. bottom: 14.}
                                            empty_text: "0"
                                            text: "0"
                                            draw_bg +: {
                                                border_color: #x017B85
                                                border_color_hover: #x822801
                                                border_color_focus: #x025B60
                                                border_radius: 12.
                                                border_size: 2.
                                            }
                                            draw_text +: {
                                                color: #x822801
                                                color_hover: #x822801
                                                color_down: #x822801
                                                color_focus: #x822801
                                                color_empty: #x56575D
                                                color_empty_hover: #x56575D
                                                color_empty_focus: #x56575D
                                                text_style +: {font_size: 16.}
                                            }
                                        }
                                    }
                                    amend_round_hint := WrapLabel{
                                        body: "Rounded Total starts equal to Applied Total. Edit only to round the bill."
                                        font_color: #x023637
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
                                        font_color: #x074D3E
                                        font_size: 12.
                                        visible: false
                                    }
                                    ActionStack{
                                        amend_btn := StackPrimaryBtn{text: "Amend"}
                                        void_btn := DangerBtn{text: "Void"}
                                        refund_btn := DangerBtn{text: "Refund"}
                                    }
                                    sale_history_label := mod.widgets.ReportText{}
                                    h_notes_attach_sale := WrapLabel{body: "Notes & attachments (selected sale)" font_color: #x023637 font_size: 15.}
                                    sale_attach_note := FieldInput{empty_text: "Extra note on this bill / sale"}
                                    ActionStack{
                                        sale_attach_add_note := RowSecondaryBtn{text: "Add note"}
                                        sale_attach_photo := RowSecondaryBtn{text: "Attach photo"}
                                        sale_attach_pdf := RowSecondaryBtn{text: "Attach PDF"}
                                    }
                                    sale_attach_hint := WrapLabel{
                                        width: Fit
                                        body: "(select a sale)"
                                        font_color: #x074D3E
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
                                    tip: "Each party has a unique id (names may repeat). A Customer runs a khata tab (owes us); a Supplier sells us stock (we owe them); Both does the two. Khata and payables stay separate; net who-owes-whom is in Reports."
                                }
                                credit_summary_label := WrapLabel{
                                    body: ""
                                    font_color: #x023637
                                    font_size: 13.
                                }
                                credit_recent_list := mod.widgets.ReportText{}
                                // New party, or the picked one being edited.
                                FormCard{
                                    party_selected_label := WrapLabel{
                                        body: "New party — fill the form and tap Add party, or Pick a party from the list to edit it."
                                        font_color: #x023637
                                        font_size: 13.
                                    }
                                    FormFieldRow{
                                        credit_add_name := FieldInput{empty_text: "Party name"}
                                        credit_add_phone := FieldInput{empty_text: "Phone"}
                                    }
                                    credit_add_birthday := FieldInput{empty_text: "Birthday (MM-DD or YYYY-MM-DD)"}
                                    h_account_kind := WrapLabel{
                                        body: "Account kind (tap one)"
                                        font_color: #x074D3E
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
                                    crm_heading := WrapLabel{body: "Guests & messages" font_color: #x023637 font_size: 15.}
                                    crm_help := WrapLabel{
                                        width: Fill
                                        body: ""
                                        font_color: #x074D3E
                                        font_size: 12.
                                        visible: false
                                    }
                                    // SegBtn is width:Fill — cannot sit in wrap:true (freezes UI).
                                    // Overlay idle/on pairs so the selected segment stays lit.
                                    ActionStack{
                                        View{width: Fill height: Fit flow: Overlay
                                            crm_seg_bday_week := SegBtn{text: "Birthday week"}
                                            crm_seg_bday_week_on := SegBtnOn{text: "Birthday week" visible: false}
                                        }
                                        View{width: Fill height: Fit flow: Overlay
                                            crm_seg_bday_month := SegBtn{text: "Birthday month"}
                                            crm_seg_bday_month_on := SegBtnOn{text: "Birthday month" visible: false}
                                        }
                                        View{width: Fill height: Fit flow: Overlay
                                            crm_seg_phone := SegBtn{text: "With phone"}
                                            crm_seg_phone_on := SegBtnOn{text: "With phone" visible: false}
                                        }
                                        View{width: Fill height: Fit flow: Overlay
                                            crm_seg_loyalty := SegBtn{text: "Loyalty"}
                                            crm_seg_loyalty_on := SegBtnOn{text: "Loyalty" visible: false}
                                        }
                                        View{width: Fill height: Fit flow: Overlay
                                            crm_seg_inactive := SegBtn{text: "Inactive"}
                                            crm_seg_inactive_on := SegBtnOn{text: "Inactive" visible: false}
                                        }
                                        View{width: Fill height: Fit flow: Overlay
                                            crm_seg_top := SegBtn{text: "Top spenders"}
                                            crm_seg_top_on := SegBtnOn{text: "Top spenders" visible: false}
                                        }
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
                                    h_credit_limit_heading := WrapLabel{body: "Credit limit" font_color: #x023637 font_size: 15.}
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
                                    h_pay_selected_heading := WrapLabel{body: "Pay selected (settle debt / take advance)" font_color: #x023637 font_size: 15.}
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
                                    h_write_off_heading := WrapLabel{body: "Write off (clear without payment)" font_color: #x023637 font_size: 15.}
                                    writeoff_amount_input := FieldInput{empty_text: "Amount (blank = full balance)"}
                                    writeoff_reason_input := FieldInput{empty_text: "Reason (required)"}
                                    credit_writeoff := NavBtn{text: "Write off / nil balance"}
                                    h_pay_ap_heading := WrapLabel{body: "Pay what we owe them (AP)" font_color: #x023637 font_size: 15.}
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
                                    h_notes_attach_party := WrapLabel{body: "Notes & attachments" font_color: #x023637 font_size: 15.}
                                    party_attach_note := FieldInput{empty_text: "Note / GSTIN / agreement reminder"}
                                    ActionStack{
                                        party_attach_add_note := RowSecondaryBtn{text: "Add note"}
                                        party_attach_photo := RowSecondaryBtn{text: "Attach photo"}
                                        party_attach_pdf := RowSecondaryBtn{text: "Attach PDF"}
                                    }
                                    party_attach_hint := WrapLabel{
                                        width: Fit
                                        body: "(select a party)"
                                        font_color: #x074D3E
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
                                    tip: "Pick a job (Cashier / Manager), optional copy from someone who already works here, then set what they can do."
                                }
                                // Hire / edit others — hidden when the signed-in
                                // person only has My PIN (Waiter, Kitchen, …).
                                team_admin_block := View{
                                width: Fill height: Fit
                                flow: Down spacing: 10
                                staff_label := mod.widgets.ReportText{}
                                team_pick_input := FieldInput{empty_text: "Pick a person to edit — leave empty to add someone"}
                                team_pick_suggest := mod.widgets.SuggestList{field: "team_pick"}
                                team_pick_hint := WrapLabel{
                                    body: "Leave this empty to add someone. Pick a person to edit that one form."
                                    font_color: #x074D3E
                                    font_size: 12.
                                }
                                team_hire_card := View{
                                width: Fill height: Fit
                                flow: Down spacing: 10
                                FormCard{
                                staff_name_input := FieldInput{empty_text: "Staff name"}
                                staff_title_input := FieldInput{empty_text: "Role title (e.g. Captain, Head cook)"}
                                staff_pin_input := FieldInput{empty_text: "Starting PIN (4–12 digits)"}
                                h_permission_preset := WrapLabel{
                                    body: "Job"
                                    font_color: #x023637
                                    font_size: 13.
                                }
                                h_preset_help_tabiyat := WrapLabel{
                                    width: Fill
                                    body: ""
                                    font_color: #x074D3E
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
                                        staff_role_kitchen := SegBtn{text: "Store"}
                                        staff_role_kitchen_on := SegBtnOn{text: "Store" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        staff_role_manager := SegBtn{text: "Manager"}
                                        staff_role_manager_on := SegBtnOn{text: "Manager" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        staff_role_admin := SegBtn{text: "Admin"}
                                        staff_role_admin_on := SegBtnOn{text: "Admin" visible: false}
                                    }
                                }
                                // Hire only: optional copy of role/title/rights into
                                // this form — then Add creates a new id. Remove is
                                // not here (it belongs with the selected person below).
                                staff_copy_hire_input := FieldInput{
                                    empty_text: "Copy setup from (optional)"
                                }
                                staff_copy_hire_suggest := mod.widgets.SuggestList{field: "team_copy_hire"}
                                staff_copy_hire_btn := RowSecondaryBtn{text: "Use this person's setup"}
                                // Hire-only optional login — used by Add staff, not Apply.
                                h_optional_login_help := WrapLabel{
                                    body: ""
                                    font_color: #x074D3E
                                    font_size: 12.
                                    visible: false
                                }
                                staff_login_input := FieldInput{empty_text: "Username (required, unique)"}
                                staff_password_input := FieldInput{empty_text: "Starting password (optional)"}
                                staff_add_cashier := StackPrimaryBtn{text: "Add staff"}
                                }
                                } // team_hire_card
                                FormCard{
                                // One form for the person you picked. Name, username,
                                // and password save with the job — not as extra forms.
                                // Hidden until a person is selected.
                                team_selected_setup := View{
                                width: Fill height: Fit
                                flow: Down spacing: 10
                                visible: false
                                team_selected_label := WrapLabel{
                                    body: " "
                                    font_color: #x025B60
                                    font_size: 13.
                                }
                                staff_edit_name_input := FieldInput{empty_text: "Display name"}
                                staff_edit_login_input := FieldInput{empty_text: "Username"}
                                staff_edit_password_input := FieldInput{empty_text: "New password — leave blank to keep"}
                                // Ids kept so an older session still finds them. Save does this work.
                                View{width: Fill height: Fit visible: false
                                    staff_rename_btn := RowSecondaryBtn{text: "Save name"}
                                    staff_set_login := RowSecondaryBtn{text: "Apply login to selected"}
                                }
                                staff_copy_edit_input := FieldInput{
                                    empty_text: "Copy setup from another person"
                                }
                                staff_copy_edit_suggest := mod.widgets.SuggestList{field: "team_copy_edit"}
                                staff_copy_edit_btn := RowSecondaryBtn{text: "Copy their setup onto selected"}
                                staff_deactivate := DangerBtn{text: "Remove selected"}
                                h_feature_access_help := WrapLabel{
                                    body: "What they can open (optional)"
                                    font_color: #x023637
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
                                                feat_floor := ChipBtn{text: "Shop"}
                                                feat_floor_on := ChipBtnActive{text: "Shop" visible: false}
                                            }
                                            View{width: Fit height: 40 flow: Overlay
                                                feat_kitchen := ChipBtn{text: "Sections"}
                                                feat_kitchen_on := ChipBtnActive{text: "Sections" visible: false}
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
                                h_rights_help := WrapLabel{
                                    body: ""
                                    font_color: #x023637
                                    font_size: 13.
                                }
                                // Each right: label + three SegBtn levels (tap sets, no cycle).
                                right_settle_lbl := WrapLabel{body: "Take payment" font_color: #x023637 font_size: 13.}
                                View{width: Fill height: Fit flow: Right spacing: 6
                                    View{width: Fill height: Fit flow: Overlay
                                        right_settle_allowed := SegBtn{text: "Allowed"}
                                        right_settle_allowed_on := SegBtnOn{text: "Allowed" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        right_settle_ask := SegBtn{text: "Ask"}
                                        right_settle_ask_on := SegBtnOn{text: "Ask" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        right_settle_deny := SegBtn{text: "No"}
                                        right_settle_deny_on := SegBtnOn{text: "No" visible: false}
                                    }
                                }
                                right_cancel_lbl := WrapLabel{body: "Cancel a sent item" font_color: #x023637 font_size: 13.}
                                View{width: Fill height: Fit flow: Right spacing: 6
                                    View{width: Fill height: Fit flow: Overlay
                                        right_cancel_allowed := SegBtn{text: "Allowed"}
                                        right_cancel_allowed_on := SegBtnOn{text: "Allowed" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        right_cancel_ask := SegBtn{text: "Ask"}
                                        right_cancel_ask_on := SegBtnOn{text: "Ask" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        right_cancel_deny := SegBtn{text: "No"}
                                        right_cancel_deny_on := SegBtnOn{text: "No" visible: false}
                                    }
                                }
                                right_nc_lbl := WrapLabel{body: "Complimentary (NC)" font_color: #x023637 font_size: 13.}
                                View{width: Fill height: Fit flow: Right spacing: 6
                                    View{width: Fill height: Fit flow: Overlay
                                        right_nc_allowed := SegBtn{text: "Allowed"}
                                        right_nc_allowed_on := SegBtnOn{text: "Allowed" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        right_nc_ask := SegBtn{text: "Ask"}
                                        right_nc_ask_on := SegBtnOn{text: "Ask" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        right_nc_deny := SegBtn{text: "No"}
                                        right_nc_deny_on := SegBtnOn{text: "No" visible: false}
                                    }
                                }
                                right_move_lbl := WrapLabel{body: "Move / merge tables" font_color: #x023637 font_size: 13.}
                                View{width: Fill height: Fit flow: Right spacing: 6
                                    View{width: Fill height: Fit flow: Overlay
                                        right_move_allowed := SegBtn{text: "Allowed"}
                                        right_move_allowed_on := SegBtnOn{text: "Allowed" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        right_move_ask := SegBtn{text: "Ask"}
                                        right_move_ask_on := SegBtnOn{text: "Ask" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        right_move_deny := SegBtn{text: "No"}
                                        right_move_deny_on := SegBtnOn{text: "No" visible: false}
                                    }
                                }
                                right_reprint_lbl := WrapLabel{body: "Print bill again" font_color: #x023637 font_size: 13.}
                                View{width: Fill height: Fit flow: Right spacing: 6
                                    View{width: Fill height: Fit flow: Overlay
                                        right_reprint_allowed := SegBtn{text: "Allowed"}
                                        right_reprint_allowed_on := SegBtnOn{text: "Allowed" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        right_reprint_ask := SegBtn{text: "Ask"}
                                        right_reprint_ask_on := SegBtnOn{text: "Ask" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        right_reprint_deny := SegBtn{text: "No"}
                                        right_reprint_deny_on := SegBtnOn{text: "No" visible: false}
                                    }
                                }
                                right_discount_lbl := WrapLabel{body: "Give a discount" font_color: #x023637 font_size: 13.}
                                View{width: Fill height: Fit flow: Right spacing: 6
                                    View{width: Fill height: Fit flow: Overlay
                                        right_discount_allowed := SegBtn{text: "Allowed"}
                                        right_discount_allowed_on := SegBtnOn{text: "Allowed" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        right_discount_ask := SegBtn{text: "Ask"}
                                        right_discount_ask_on := SegBtnOn{text: "Ask" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        right_discount_deny := SegBtn{text: "No"}
                                        right_discount_deny_on := SegBtnOn{text: "No" visible: false}
                                    }
                                }
                                // Kitchen stations this person may send / print KOTs to.
                                // Bill reprint above is bills only — not kitchen.
                                h_team_stations_help := WrapLabel{
                                    body: "Dispensary sections they may use"
                                    font_color: #x023637
                                    font_size: 13.
                                }
                                // Always-open checklist while this person is selected
                                // (see SuggestField::TeamKotStations + sync_chrome).
                                // Hidden type-in anchor — FieldInput has no `visible`.
                                View{
                                    width: Fill height: 0.
                                    clip_y: true
                                    team_stations_anchor := FieldInput{
                                        height: 0.
                                    }
                                }
                                team_stations_list := mod.widgets.StationCheckList{field: "team_kot_stations"}
                                feat_save := StackPrimaryBtn{text: "Save job setup"}
                                } // team_selected_setup
                                }
                                } // team_admin_block
                                FormCard{
                                h_change_my_pin_heading := WrapLabel{
                                    body: "My sign-in"
                                    font_color: #x023637
                                    font_size: 15.
                                }
                                my_pin_input := FieldInput{empty_text: "New PIN (4–12 digits) — leave blank to keep"}
                                my_password_input := FieldInput{empty_text: "New password — leave blank to keep"}
                                change_my_pin := StackPrimaryBtn{text: "Update my sign-in"}
                                View{width: Fill height: Fit visible: false
                                    change_my_password := RowSecondaryBtn{text: "Update my password"}
                                }
                                }
                                // Owner-only. FieldInput ignores set_visible — hide
                                // the whole card with this wrapper (Waiter must never
                                // see Emergency PIN fields).
                                duress_owner_block := View{
                                    width: Fill height: Fit
                                    flow: Down spacing: 0
                                    visible: false
                                // Emergency unlock PIN: looks like a normal login
                                // but wipes books entries first (sales/ledgers/
                                // history). Menu, staff, settings, schema stay —
                                // never deletes the local server.
                                FormCard{
                                View{
                                    width: Fill height: Fit
                                    flow: Right spacing: 0
                                    align: Align{y: 0.5}
                                    h_duress_heading := WrapLabel{
                                        width: Fit
                                        body: "Emergency unlock PIN (Owner only)"
                                        font_color: #x023637
                                        font_size: 15.
                                    }
                                    h_duress_help := InfoHelp{
                                        tip: "Optional. If this PIN is typed at unlock instead of yours, all books entries are wiped (sales, ledgers, history) and the till opens as Owner looking normal — menu, prices, tables, staff and settings stay. Schema and config are not deleted. Use only if you are forced to unlock. Leave blank and save to turn off. Never the same as anyone's PIN."
                                    }
                                }
                                duress_status := WrapLabel{
                                    width: Fit
                                    body: " "
                                    font_color: #x025B60
                                    font_size: 12.
                                }
                                duress_pin_input := FieldInput{
                                    empty_text: "Emergency PIN (4–12 digits)"
                                    is_password: true
                                }
                                duress_pin_confirm := FieldInput{
                                    empty_text: "Type again to confirm"
                                    is_password: true
                                }
                                save_duress_pin := StackPrimaryBtn{text: "Save emergency PIN"}
                                clear_duress_pin := RowSecondaryBtn{text: "Turn off emergency PIN"}
                                }
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
                                        tip: "Open one area at a time — language, printers, bill format, backup, and more."
                                    }
                                    settings_hub_language := StackSecondaryBtn{text: "Language & currency"}
                                    settings_hub_country := StackSecondaryBtn{text: "Country & tax"}
                                    settings_hub_tills := StackSecondaryBtn{text: "Other devices"}
                                    settings_hub_brand := StackSecondaryBtn{text: "Your brand"}
                                    settings_hub_units := StackSecondaryBtn{text: "Units & conversions"}
                                    settings_hub_pickers := StackSecondaryBtn{text: "Areas · stations · tax rates"}
                                    settings_hub_printers := StackSecondaryBtn{text: "Printers"}
                                    settings_hub_bill := StackSecondaryBtn{text: "Bill & KOT format"}
                                    settings_hub_selling := StackSecondaryBtn{text: "Schemes & sales"}
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
                                        settings_title := WrapLabel{body: "Language & currency" font_color: #x023637 font_size: 20.}
                                    }
                                    locale_label := mod.widgets.ReportText{}
                                    h_language := WrapLabel{body: "Language" font_color: #x023637 font_size: 15.}
                                    h_language_setup_hint := WrapLabel{
                                        width: Fill
                                        body: "Follow this device’s language, or pick one yourself."
                                        font_color: #x023637
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
                                        font_color: #x023637
                                        font_size: 13.
                                    }
                                    language_list := mod.widgets.LanguageList{}
                                    h_currency := WrapLabel{body: "Currency" font_color: #x023637 font_size: 15.}
                                    h_currency_use_help := WrapLabel{
                                        width: Fill
                                        body: "Scroll and tap Use for any supported currency."
                                        font_color: #x023637
                                        font_size: 12.
                                    }
                                    currency_list := mod.widgets.CurrencyList{}
                                }
                                settings_panel_country := View{
                                    width: Fill height: Fit
                                    flow: Down spacing: 12
                                    visible: false
                                    h_country_tax := WrapLabel{body: "Country & tax" font_color: #x023637 font_size: 20.}
                                    h_country_pack_help := WrapLabel{
                                        width: Fill
                                        body: ""
                                        font_color: #x023637
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
                                            tax_inclusive_btn := SegBtn{text: "Tax inclusive"}
                                            tax_inclusive_btn_on := SegBtnOn{text: "Tax inclusive" visible: false}
                                        }
                                        View{width: Fill height: Fit flow: Overlay
                                            tax_exclusive_btn := SegBtn{text: "Tax exclusive"}
                                            tax_exclusive_btn_on := SegBtnOn{text: "Tax exclusive" visible: false}
                                        }
                                    }
                                    business_tax_id_input := FieldInput{empty_text: "Business tax id"}
                                    service_bps_input := FieldInput{
                                        empty_text: "Service % optional 0/10/12"
                                    }
                                    fiscal_save := StackPrimaryBtn{text: "Save business id"}
                                    }
                                    FormCard{
                                    h_datetime_appearance := WrapLabel{
                                        width: Fill
                                        body: "Date & time appearance"
                                        font_color: #x023637
                                        font_size: 15.
                                    }
                                    datetime_appearance_help := WrapLabel{
                                        width: Fill
                                        body: "Books keep year-first RFC3339 inside. These only change how dates and clocks look on bills, KOT, reports, and the screen."
                                        font_color: #x074D3E
                                        font_size: 12.
                                    }
                                    datetime_example_label := WrapLabel{
                                        width: Fill
                                        body: ""
                                        font_color: #x023637
                                        font_size: 13.
                                    }
                                    h_date_order := WrapLabel{body: "Date order" font_color: #x023637 font_size: 13.}
                                    View{
                                        width: Fill height: Fit
                                        flow: Right spacing: 6
                                        View{width: Fill height: Fit flow: Overlay
                                            date_order_dmy := SegBtn{text: "D/M/Y"}
                                            date_order_dmy_on := SegBtnOn{text: "D/M/Y" visible: false}
                                        }
                                        View{width: Fill height: Fit flow: Overlay
                                            date_order_mdy := SegBtn{text: "M/D/Y"}
                                            date_order_mdy_on := SegBtnOn{text: "M/D/Y" visible: false}
                                        }
                                        View{width: Fill height: Fit flow: Overlay
                                            date_order_ymd := SegBtn{text: "Y-M-D"}
                                            date_order_ymd_on := SegBtnOn{text: "Y-M-D" visible: false}
                                        }
                                    }
                                    h_time_format := WrapLabel{body: "Clock" font_color: #x023637 font_size: 13.}
                                    View{
                                        width: Fill height: Fit
                                        flow: Right spacing: 6
                                        View{width: Fill height: Fit flow: Overlay
                                            time_format_24h := SegBtn{text: "24-hour"}
                                            time_format_24h_on := SegBtnOn{text: "24-hour" visible: false}
                                        }
                                        View{width: Fill height: Fit flow: Overlay
                                            time_format_12h := SegBtn{text: "12-hour"}
                                            time_format_12h_on := SegBtnOn{text: "12-hour" visible: false}
                                        }
                                    }
                                    }
                                }
                                settings_panel_tills := View{
                                    width: Fill height: Fit
                                    flow: Down spacing: 12
                                    visible: false
View{
                                    width: Fill height: Fit
                                    flow: Right spacing: 0
                                    align: Align{y: 0.5}
                                    h_extra_tills_heading := WrapLabel{
                                        width: Fit
                                        body: "Extra tills"
                                        font_color: #x023637
                                        font_size: 20.
                                    }
                                    server_pair_help := InfoHelp{
                                        tip: "On the other device: Client Mode → Find server → enter this pairing code."
                                    }
                                }
                                h_other_devices_help_heading := WrapLabel{
                                    body: "Set up other devices"
                                    font_color: #x023637
                                    font_size: 15.
                                    visible: false
                                }
                                h_other_devices_help := WrapLabel{
                                    width: Fill
                                    body: " "
                                    font_color: #x074D3E
                                    font_size: 12.
                                    visible: false
                                }
                                // Big digits stay visible even when firewall/listen notes fill the status line.
                                // Display/copy only — not a TextInput (no soft keyboard). Tap Copy for clipboard.
                                pairing_code_value := WrapLabel{
                                    width: Fit
                                    body: " "
                                    font_color: #x023637
                                    font_size: 28.
                                    selectable: true
                                    visible: false
                                }
                                server_pair_label := WrapLabel{
                                    width: Fill
                                    body: " "
                                    font_color: #x074D3E
                                    font_size: 13.
                                }
                                ActionStack{
                                    allow_extra_tills := RowPrimaryBtn{text: "Allow extra tills"}
                                    stop_extra_tills := RowSecondaryBtn{text: "Stop extra tills"}
                                    allow_firewall := RowPrimaryBtn{text: "Allow other devices to connect" visible: false}
                                }
                                autostart_btn := RowSecondaryBtn{text: "Start with this computer" visible: false}
                                copy_pair_code := RowSecondaryBtn{text: "Copy pairing code" visible: false}
                                new_pair_code := RowSecondaryBtn{text: "New pairing code" visible: false}
                                paired_list := mod.widgets.PairedList{}
                                // A paired device: its link, what waits, what was not accepted.
                                client_connection_btn := RowSecondaryBtn{text: "Connection to the shop" visible: false}
                                h_start_template := WrapLabel{body: "Product" font_color: #x023637 font_size: 20.}
                                settings_tmpl_label := WrapLabel{
                                    body: "Tabiyat"
                                    font_color: #x074D3E
                                    font_size: 13.
                                }
                                h_product_only_tabiyat := WrapLabel{
                                    width: Fill
                                    body: ""
                                    font_color: #x074D3E
                                    font_size: 12.
                                    visible: false
                                }
                                                                }
                                settings_panel_brand := View{
                                    width: Fill height: Fit
                                    flow: Down spacing: 12
                                    visible: false
                                FormCard{
h_your_brand := WrapLabel{body: "Your brand (in-app)" font_color: #x023637 font_size: 20.}
                                h_brand_shown_help := WrapLabel{
                                    width: Fill
                                    body: ""
                                    font_color: #x023637
                                    font_size: 12.
                                    visible: false
                                }
                                venue_title_input := FieldInput{empty_text: "Name of this place"}
                                org_name_input := FieldInput{empty_text: "Business name"}
                                View{
                                    width: Fill height: Fit
                                    flow: Right spacing: 0
                                    align: Align{y: 0.5}
                                    h_venue_icon := WrapLabel{
                                                    width: Fit
                                        body: "App icon"
                                        font_color: #x023637
                                        font_size: 15.
                                    }
                                    venue_icon_help := InfoHelp{
                                        tip: "Shown on screens inside the app. The installed app name and icon on this device stay as the platform listing."
                                    }
                                }
                                venue_icon_status := WrapLabel{
                                    body: " "
                                    font_color: #x074D3E
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
                                    flow: Right spacing: 0
                                    align: Align{y: 0.5}
                                    h_print_brand := WrapLabel{
                                                    width: Fit
                                                    body: "Print brand kit" font_color: #x023637 font_size: 18.}
                                    print_brand_help := InfoHelp{
                                        tip: "Optional stamp and payment QR images on bill faces and the LAN guest display. Dynamic UPI QR from the VPA on the bill still works when no payment QR image is set."
                                    }
                                }
                                View{
                                    width: Fill height: Fit
                                    flow: Right spacing: 0
                                    align: Align{y: 0.5}
                                    h_stamp := WrapLabel{
                                                    width: Fit
                                                    body: "Stamp" font_color: #x023637 font_size: 15.}
                                    print_stamp_help := InfoHelp{
                                        tip: "Optional stamp image on printed bills and the guest display."
                                    }
                                }
                                print_stamp_status := WrapLabel{
                                    body: " "
                                    font_color: #x074D3E
                                    font_size: 12.
                                }
                                ActionStack{
                                    print_stamp_pick := StackSecondaryBtn{text: "Choose stamp"}
                                    print_stamp_clear := StackSecondaryBtn{text: "Clear stamp"}
                                }
                                View{
                                    width: Fill height: Fit
                                    flow: Right spacing: 0
                                    align: Align{y: 0.5}
                                    h_payment_qr := WrapLabel{
                                                    width: Fit
                                                    body: "Payment QR" font_color: #x023637 font_size: 15.}
                                    payment_qr_image_help := InfoHelp{
                                        tip: "Static QR on bills and guest display; otherwise paying uses a dynamic UPI QR from the VPA on the bill."
                                    }
                                }
                                payment_qr_status := WrapLabel{
                                    body: " "
                                    font_color: #x074D3E
                                    font_size: 12.
                                }
                                ActionStack{
                                    payment_qr_pick := StackSecondaryBtn{text: "Choose payment QR"}
                                    payment_qr_clear := StackSecondaryBtn{text: "Clear payment QR"}
                                }
                                }
                                                                }
                                settings_panel_units := View{
                                    width: Fill height: Fit
                                    flow: Down spacing: 12
                                    visible: false
                                FormCard{
h_units_heading := WrapLabel{body: "Units" font_color: #x023637 font_size: 20.}
                                h_units_help := WrapLabel{
                                    width: Fill
                                    body: ""
                                    font_color: #x074D3E
                                    font_size: 12.
                                    visible: false
                                }
                                h_units_picker_help := WrapLabel{
                                    width: Fill
                                    body: ""
                                    font_color: #x074D3E
                                    font_size: 12.
                                    visible: false
                                }
                                units_label := mod.widgets.ReportText{}
                                FormFieldRow{
                                    unit_code_input := FieldInput{empty_text: "Code (bag)"}
                                    unit_name_input := FieldInput{empty_text: "Name (Bag)"}
                                }
                                ActionStack{
                                    unit_add := StackPrimaryBtn{text: "Add unit"}
                                }
                                }
                                FormCard{
                                conv_from_input := FieldInput{empty_text: "From code (bag)"}
                                conv_from_suggest := mod.widgets.SuggestList{field: "conv_from"}
                                conv_to_input := FieldInput{empty_text: "To code (kg)"}
                                conv_to_suggest := mod.widgets.SuggestList{field: "conv_to"}
                                conv_factor_input := FieldInput{empty_text: "Factor (1 bag → kg)"}
                                ActionStack{
                                    conv_add := StackPrimaryBtn{text: "Add conversion"}
                                }
                                }
                                                                }
                                settings_panel_pickers := View{
                                    width: Fill height: Fit
                                    flow: Down spacing: 12
                                    visible: false
sibling_pickers_heading := WrapLabel{body: "Areas · sections · tax rates" font_color: #x023637 font_size: 18.}
                                sibling_pickers_help := WrapLabel{
                                    width: Fill
                                    body: ""
                                    font_color: #x074D3E
                                    font_size: 12.
                                    visible: false
                                }
                                sibling_pickers_label := mod.widgets.ReportText{}
                                FormCard{
                                station_manage_heading := WrapLabel{
                                    body: "Add or remove a dispensary section"
                                    font_color: #x023637
                                    font_size: 15.
                                }
                                // Own ids (Kitchen screen already has station_*).
                                settings_station_name_input := FieldInput{empty_text: "Section name (e.g. OTC, Fridge, Counter)"}
                                settings_station_add := StackPrimaryBtn{text: "Add station"}
                                settings_station_remove_input := FieldInput{empty_text: "Station name to remove"}
                                settings_station_remove_suggest := mod.widgets.SuggestList{field: "settings_station_remove"}
                                settings_station_remove := StackSecondaryBtn{text: "Remove station"}
                                }
                                FormCard{
                                bulk_price_heading := WrapLabel{ width: Fit
                                                    body: "Bulk medicine price" font_color: #x023637 font_size: 18.}
                                bulk_cat_input := FieldInput{empty_text: "Category name"}
                                bulk_cat_suggest := mod.widgets.SuggestList{field: "bulk_cat"}
                                FormFieldRow{
                                    bulk_pct_input := FieldInput{empty_text: "% change"}
                                    bulk_set_input := FieldInput{empty_text: "Or set ₹"}
                                }
                                ActionStack{
                                    bulk_price_apply := StackPrimaryBtn{text: "Apply bulk price"}
                                }
                                }
                                settings_lan_help := InfoHelp{
                                    tip: "On this shop’s Wi‑Fi, guest pages work for display board, tokens, expo, kiosk, table QR, and feedback. Guests on mobile data need Cloud later. Card terminal / payment link is optional when the internet is up — it never blocks the counter."
                                }
                                                                }
                                settings_panel_printers := View{
                                    width: Fill height: Fit
                                    flow: Down spacing: 12
                                    visible: false
                                    FormCard{
                                    printers_panel_help := InfoHelp{
                                        tip: "Find printers, then open Printers for a test page on the device to try."
                                    }
                                    View{width: Fit height: Fit flow: Overlay visible: false
                                        kot_print_off := ChipBtn{text: "Auto-print kitchen tickets: Off"}
                                        kot_print_on := ChipBtnActive{text: "Auto-print kitchen tickets: On" visible: false}
                                    }
                                    // Legacy id — must stay hidden (no second “Printers” title).
                                    printers_find_help := InfoHelp{
                                        tip: ""
                                        visible: false
                                    }
                                    ActionStack{
                                        find_printers_btn := StackSecondaryBtn{text: "Find printers" flow: Down}
                                    }
                                    find_printers_cancel := QuietFitBtn{text: "Cancel" visible: false}
                                    printers_found_label := WrapLabel{
                                        width: Fill
                                        body: ""
                                        font_color: #x074D3E
                                        font_size: 12.
                                    }
                                    printers_pool_label := mod.widgets.ReportText{}
                                    // Legacy — Find used to mirror the pool in a second list.
                                    printers_found_list := mod.widgets.ReportText{visible: false}
                                    printers_pool_help := InfoHelp{tip: " " visible: false}
                                    printers_test_dd := StackSecondaryBtn{text: "Printers for a test page" flow: Down}
                                    printers_test_input := FieldInput{
                                        height: 0.
                                        empty_text: ""
                                        margin: Inset{left: 0. right: 0. top: 0. bottom: 0.}
                                        padding: Inset{left: 0. right: 0. top: 0. bottom: 0.}
                                    }
                                    printers_test_suggest := mod.widgets.StationCheckList{field: "test_printers"}
                                    // Legacy ids (hot-reload) — Find owns the pool; no manual Add/Remove.
                                    printer_manual_legacy := View{width: Fill height: Fit visible: false
                                        printer_target_input := FieldInput{height: 0. empty_text: ""}
                                        printer_target_suggest := mod.widgets.SuggestList{field: "printer_target"}
                                        printer_add := StackPrimaryBtn{text: "Add printer" flow: Down}
                                        printer_remove := StackSecondaryBtn{text: "Remove" flow: Down}
                                        printer_add_cancel := QuietFitBtn{text: "Cancel"}
                                    }
                                    printers_routes_help := InfoHelp{tip: " " visible: false}
                                    View{width: Fill height: Fit visible: false
                                        printer_place_dd := StackSecondaryBtn{text: "Bill" flow: Down}
                                    }
                                    // FieldInput has no `visible`. A hidden wrapper keeps the
                                    // legacy id without a script error on every launch.
                                    View{width: Fill height: Fit visible: false
                                        printer_place_input := FieldInput{
                                            height: 0.
                                            margin: Inset{left: 0. right: 0. top: 0. bottom: 0.}
                                            padding: Inset{left: 0. right: 0. top: 0. bottom: 0.}
                                            empty_text: ""
                                        }
                                        printer_place_suggest := mod.widgets.SuggestList{field: "printer_place"}
                                    }
                                    // Legacy ids kept for hot-reload — no named Config catalog.
                                    View{width: Fill height: Fit visible: false
                                        h_profiles := WrapLabel{
                                            body: ""
                                            font_color: #x023637
                                            font_size: 14.
                                        }
                                        profiles_list := mod.widgets.ReportText{}
                                        profile_name_input := FieldInput{
                                            empty_text: ""
                                            height: 0.
                                            margin: Inset{left: 0. right: 0. top: 0. bottom: 0.}
                                            padding: Inset{left: 0. right: 0. top: 0. bottom: 0.}
                                        }
                                        ActionStack{
                                            profile_new_btn := StackSecondaryBtn{text: "" flow: Down}
                                            profile_delete_btn := StackLightBtn{text: "" flow: Down}
                                        }
                                    }
                                    // Leftover virtual-printer form. A text field ignores set_visible,
                                    // so the whole block stays inside a hidden view.
                                    virtual_printer_legacy := View{
                                        width: Fill height: Fit
                                        visible: false
                                    h_station_group := WrapLabel{
                                        body: "Virtual printers"
                                        font_color: #x023637
                                        font_size: 14.
                                    }
                                    route_assigned_empty := WrapLabel{
                                        width: Fill
                                        body: ""
                                        font_color: #x074D3E
                                        font_size: 12.
                                        margin: Inset{left: 0. right: 0. top: 6. bottom: 4.}
                                    }
                                    route_assigned_list := mod.widgets.ReportText{}
                                    station_group_name_input := FieldInput{
                                        empty_text: "Virtual printer name (optional)"
                                    }
                                    ActionStack{
                                        printer_assign_dd := StackSecondaryBtn{text: "Select real printer" flow: Down}
                                    }
                                    printer_assign_input := FieldInput{
                                        height: 0.
                                        margin: Inset{left: 0. right: 0. top: 0. bottom: 0.}
                                        padding: Inset{left: 0. right: 0. top: 0. bottom: 0.}
                                        empty_text: ""
                                    }
                                    printer_assign_suggest := mod.widgets.StationCheckList{field: "printer_assign"}
                                    h_printer_profile := WrapLabel{
                                        body: "Layout"
                                        font_color: #x023637
                                        font_size: 14.
                                    }
                                    printer_profile_row := View{
                                        width: Fill height: Fit
                                        flow: Right{wrap: true}
                                        spacing: 6
                                        View{width: Fit height: Fit flow: Overlay visible: false
                                            printer_wire_escpos := ChipBtn{text: ""}
                                            printer_wire_escpos_on := ChipBtnActive{text: "" visible: false}
                                        }
                                        View{width: Fit height: Fit flow: Overlay
                                            printer_wire_os := ChipBtn{text: "Portrait"}
                                            printer_wire_os_on := ChipBtnActive{text: "Portrait" visible: false}
                                        }
                                        View{width: Fit height: Fit flow: Overlay
                                            printer_wire_os_landscape := ChipBtn{text: "Landscape"}
                                            printer_wire_os_landscape_on := ChipBtnActive{text: "Landscape" visible: false}
                                        }
                                    }
                                    // Legacy hot-reload ids — content decides size/wrap/margins, not VP form.
                                    View{width: Fill height: Fit visible: false
                                        h_vp_type_size := WrapLabel{body: ""}
                                        vp_type_size_row := View{width: Fit height: Fit
                                            vp_size_s := SegBtn{text: ""}
                                            vp_size_s_on := SegBtnOn{text: "" visible: false}
                                            vp_size_m := SegBtn{text: ""}
                                            vp_size_m_on := SegBtnOn{text: "" visible: false}
                                            vp_size_l := SegBtn{text: ""}
                                            vp_size_l_on := SegBtnOn{text: "" visible: false}
                                        }
                                        h_vp_wrap := WrapLabel{body: ""}
                                        vp_wrap_row := View{width: Fit height: Fit
                                            vp_wrap_32 := SegBtn{text: ""}
                                            vp_wrap_32_on := SegBtnOn{text: "" visible: false}
                                            vp_wrap_40 := SegBtn{text: ""}
                                            vp_wrap_40_on := SegBtnOn{text: "" visible: false}
                                            vp_wrap_48 := SegBtn{text: ""}
                                            vp_wrap_48_on := SegBtnOn{text: "" visible: false}
                                        }
                                        h_vp_margin := WrapLabel{body: ""}
                                        vp_margin_row := View{width: Fit height: Fit
                                            vp_margin_tight := SegBtn{text: ""}
                                            vp_margin_tight_on := SegBtnOn{text: "" visible: false}
                                            vp_margin_normal := SegBtn{text: ""}
                                            vp_margin_normal_on := SegBtnOn{text: "" visible: false}
                                            vp_margin_narrow := SegBtn{text: ""}
                                            vp_margin_narrow_on := SegBtnOn{text: "" visible: false}
                                        }
                                    }
                                    View{
                                        width: Fill height: Fit
                                        flow: Right spacing: 0
                                        align: Align{y: 0.5}
                                        h_vp_paper_width := WrapLabel{
                                            width: Fit
                                            body: "Content width (mm)"
                                            font_color: #x023637
                                            font_size: 14.
                                        }
                                        vp_paper_width_help := InfoHelp{
                                            tip: "How wide the printed content is on this virtual printer. Bills use this."
                                        }
                                    }
                                    vp_paper_width_choices := View{
                                        width: Fill height: Fit
                                        flow: Down spacing: 6
                                        View{width: Fill height: Fit flow: Overlay
                                            vp_paper_58 := SegBtn{text: "58 mm — narrow slip"}
                                            vp_paper_58_on := SegBtnOn{text: "58 mm — narrow slip" visible: false}
                                        }
                                        View{width: Fill height: Fit flow: Overlay
                                            vp_paper_80 := SegBtn{text: "80 mm — wide slip"}
                                            vp_paper_80_on := SegBtnOn{text: "80 mm — wide slip" visible: false}
                                        }
                                        View{width: Fill height: Fit flow: Overlay
                                            vp_paper_112 := SegBtn{text: "112 mm — 4 inch slip"}
                                            vp_paper_112_on := SegBtnOn{text: "112 mm — 4 inch slip" visible: false}
                                        }
                                        View{width: Fill height: Fit flow: Overlay
                                            vp_paper_210 := SegBtn{text: "210 mm — page"}
                                            vp_paper_210_on := SegBtnOn{text: "210 mm — page" visible: false}
                                        }
                                    }
                                    vp_paper_width_input := TextInput{
                                        width: Fill height: Fit
                                        empty_text: "Or type a content width, 40–210 mm"
                                    }
                                    // Tip hugs the section label — never a lone ℹ on its own Down row.
                                    View{
                                        width: Fill height: Fit
                                        flow: Right spacing: 0
                                        align: Align{y: 0.5}
                                        h_vp_delivery := WrapLabel{
                                            width: Fit
                                            body: "Delivery"
                                            font_color: #x023637
                                            font_size: 14.
                                        }
                                        vp_delivery_help := InfoHelp{
                                            tip: "How this virtual printer sends jobs — OS document, raw bytes, or ESC/POS."
                                        }
                                    }
                                    // Down + Fill SegBtns — never Right{wrap} (empty slab / crop).
                                    printer_mode_row := View{
                                        width: Fill height: Fit
                                        flow: Down spacing: 6
                                        View{width: Fill height: Fit flow: Overlay
                                            printer_mode_first := SegBtn{text: "OS document (system spooler)"}
                                            printer_mode_first_on := SegBtnOn{text: "OS document (system spooler)" visible: false}
                                        }
                                        View{width: Fill height: Fit flow: Overlay
                                            printer_mode_all := SegBtn{text: "Raw bytes to device"}
                                            printer_mode_all_on := SegBtnOn{text: "Raw bytes to device" visible: false}
                                        }
                                        View{width: Fill height: Fit flow: Overlay
                                            vp_deliv_escpos := SegBtn{text: "ESC/POS thermal commands"}
                                            vp_deliv_escpos_on := SegBtnOn{text: "ESC/POS thermal commands" visible: false}
                                        }
                                    }
                                    ActionStack{
                                        virtual_printer_new_btn := StackLightBtn{text: "New virtual printer" flow: Down}
                                        station_group_save_btn := StackSecondaryBtn{text: "Save virtual printer" flow: Down}
                                        station_group_apply_btn := StackSecondaryBtn{text: "Delete virtual printer" flow: Down}
                                    }
                                    }
                                    route_place_btns := View{width: Fill height: Fit visible: false
                                        route_place_slot_0 := View{width: Fit height: Fit flow: Overlay
                                            route_place_0 := ChipBtn{text: "Bill"}
                                            route_place_0_on := ChipBtnActive{text: "Bill" visible: false}
                                        }
                                        route_place_slot_1 := View{width: Fit height: Fit flow: Overlay visible: false
                                            route_place_1 := ChipBtn{text: ""}
                                            route_place_1_on := ChipBtnActive{text: "" visible: false}
                                        }
                                        route_place_slot_2 := View{width: Fit height: Fit flow: Overlay visible: false
                                            route_place_2 := ChipBtn{text: ""}
                                            route_place_2_on := ChipBtnActive{text: "" visible: false}
                                        }
                                        route_place_slot_3 := View{width: Fit height: Fit flow: Overlay visible: false
                                            route_place_3 := ChipBtn{text: ""}
                                            route_place_3_on := ChipBtnActive{text: "" visible: false}
                                        }
                                        route_place_slot_4 := View{width: Fit height: Fit flow: Overlay visible: false
                                            route_place_4 := ChipBtn{text: ""}
                                            route_place_4_on := ChipBtnActive{text: "" visible: false}
                                        }
                                        route_place_slot_5 := View{width: Fit height: Fit flow: Overlay visible: false
                                            route_place_5 := ChipBtn{text: ""}
                                            route_place_5_on := ChipBtnActive{text: "" visible: false}
                                        }
                                        route_place_slot_6 := View{width: Fit height: Fit flow: Overlay visible: false
                                            route_place_6 := ChipBtn{text: ""}
                                            route_place_6_on := ChipBtnActive{text: "" visible: false}
                                        }
                                        route_place_slot_7 := View{width: Fit height: Fit flow: Overlay visible: false
                                            route_place_7 := ChipBtn{text: ""}
                                            route_place_7_on := ChipBtnActive{text: "" visible: false}
                                        }
                                    }
                                    printer_routes_label := mod.widgets.ReportText{visible: false}
                                    route_editing_label := WrapLabel{body: "" visible: false}
                                    printer_wire_legacy := View{width: Fit height: Fit visible: false}
                                    // Legacy +chips — assign is printer_assign_dd checklist now.
                                    route_add_printers := View{
                                        width: Fill height: Fit
                                        visible: false
                                        flow: Right{wrap: true}
                                        spacing: 8
                                        route_add_prn_0 := QuietFitBtn{text: "" visible: false}
                                        route_add_prn_1 := QuietFitBtn{text: "" visible: false}
                                        route_add_prn_2 := QuietFitBtn{text: "" visible: false}
                                        route_add_prn_3 := QuietFitBtn{text: "" visible: false}
                                        route_add_prn_4 := QuietFitBtn{text: "" visible: false}
                                        route_add_prn_5 := QuietFitBtn{text: "" visible: false}
                                        route_add_prn_6 := QuietFitBtn{text: "" visible: false}
                                        route_add_prn_7 := QuietFitBtn{text: "" visible: false}
                                        route_add_prn_8 := QuietFitBtn{text: "" visible: false}
                                        route_add_prn_9 := QuietFitBtn{text: "" visible: false}
                                        route_add_prn_10 := QuietFitBtn{text: "" visible: false}
                                        route_add_prn_11 := QuietFitBtn{text: "" visible: false}
                                        route_add_prn_12 := QuietFitBtn{text: "" visible: false}
                                        route_add_prn_13 := QuietFitBtn{text: "" visible: false}
                                        route_add_prn_14 := QuietFitBtn{text: "" visible: false}
                                        route_add_prn_15 := QuietFitBtn{text: "" visible: false}
                                    }
                                    // Legacy place-route actions — not on Printers (PRINTER.md).
                                    ActionStack{
                                        visible: false
                                        route_clear := StackSecondaryBtn{text: "Clear" flow: Down}
                                    }
                                    ActionStack{
                                        print_test_btn := StackSecondaryBtn{text: "Print a test page" flow: Down}
                                    }
                                    // Legacy ids kept for RTL / paint paths.
                                    h_kitchen_printing := WrapLabel{body: "" visible: false}
                                    h_kot_print_help_tabiyat := WrapLabel{body: "" visible: false}
                                    h_printers_find := WrapLabel{body: "" visible: false}
                                    h_printers_help := WrapLabel{body: "" visible: false}
                                    h_printers_pool := WrapLabel{body: "" visible: false}
                                    h_printers_routes := WrapLabel{body: "" visible: false}
                                    h_printers_routes_help := WrapLabel{body: "" visible: false}
                                    h_route_pick := WrapLabel{body: "" visible: false}
                                    h_printer_wire := WrapLabel{body: "" visible: false}
                                    h_route_assigned := WrapLabel{body: "" visible: false}
                                    h_route_add := WrapLabel{body: "" visible: false}
                                    h_printers := WrapLabel{body: "" visible: false}
                                    }
                                                                }
                                settings_panel_bill := View{
                                    width: Fill height: Fit
                                    flow: Down spacing: 12
                                    visible: false
                                    // Screen title + ℹ — same pattern as books_help (not buried in a card).
                                    bill_kot_panel_help := InfoHelp{
                                        tip: "Bill layout and content width. Cashier is who issued or settled the bill."
                                    }
                                    // Bill | KOT tabs under the section title.
                                    View{
                                        width: Fill height: Fit
                                        flow: Right spacing: 6
                                        View{width: Fill height: Fit flow: Overlay
                                            bill_kot_tab_bill := SegBtn{text: "Bill"}
                                            bill_kot_tab_bill_on := SegBtnOn{text: "Bill" visible: false}
                                        }
                                        View{width: Fill height: Fit flow: Overlay
                                            bill_kot_tab_kot := SegBtn{text: "Slip"}
                                            bill_kot_tab_kot_on := SegBtnOn{text: "Slip" visible: false}
                                        }
                                    }
                                    View{
                                        width: Fill height: Fit
                                        flow: Right spacing: 0
                                        align: Align{y: 0.5}
                                        h_content_width := WrapLabel{
                                            width: Fit
                                            body: "Content width (mm)"
                                            font_color: #x023637
                                            font_size: 14.
                                        }
                                        content_width_help := InfoHelp{
                                            tip: "How wide this slip prints. Bill keeps its own width."
                                        }
                                    }
                                    content_width_choices := View{
                                        width: Fill height: Fit
                                        flow: Down spacing: 6
                                        View{width: Fill height: Fit flow: Overlay
                                            content_width_58 := SegBtn{text: "58 mm — narrow slip"}
                                            content_width_58_on := SegBtnOn{text: "58 mm — narrow slip" visible: false}
                                        }
                                        View{width: Fill height: Fit flow: Overlay
                                            content_width_80 := SegBtn{text: "80 mm — wide slip"}
                                            content_width_80_on := SegBtnOn{text: "80 mm — wide slip" visible: false}
                                        }
                                        View{width: Fill height: Fit flow: Overlay
                                            content_width_112 := SegBtn{text: "112 mm — 4 inch slip"}
                                            content_width_112_on := SegBtnOn{text: "112 mm — 4 inch slip" visible: false}
                                        }
                                        View{width: Fill height: Fit flow: Overlay
                                            content_width_210 := SegBtn{text: "210 mm — page"}
                                            content_width_210_on := SegBtnOn{text: "210 mm — page" visible: false}
                                        }
                                    }
                                    content_width_input := FieldInput{
                                        empty_text: "Or type a content width, 40–210 mm"
                                    }
                                bill_format_body := View{
                                    width: Fill height: Fit
                                    flow: Down spacing: 12
                                FormCard{
                                h_bill_layout := WrapLabel{body: "Layout" font_color: #x023637 font_size: 14.}
                                View{width: Fill height: Fit flow: Right spacing: 6
                                    View{width: Fill height: Fit flow: Overlay
                                        bill_layout_portrait := SegBtn{text: "Portrait"}
                                        bill_layout_portrait_on := SegBtnOn{text: "Portrait" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        bill_layout_landscape := SegBtn{text: "Landscape"}
                                        bill_layout_landscape_on := SegBtnOn{text: "Landscape" visible: false}
                                    }
                                }
                                View{width: Fill height: Fit flow: Right spacing: 0 align: Align{y: 0.5}
                                    h_bill_delivery := WrapLabel{width: Fit body: "How the job is sent" font_color: #x023637 font_size: 14.}
                                    bill_delivery_help := InfoHelp{tip: "OS document, raw bytes, or ESC/POS for the bill. Route to any real printer."}
                                }
                                View{width: Fill height: Fit flow: Down spacing: 6
                                    View{width: Fill height: Fit flow: Overlay
                                        bill_deliv_os := SegBtn{text: "OS document (system spooler)"}
                                        bill_deliv_os_on := SegBtnOn{text: "OS document (system spooler)" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        bill_deliv_raw := SegBtn{text: "Raw bytes to device"}
                                        bill_deliv_raw_on := SegBtnOn{text: "Raw bytes to device" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        bill_deliv_escpos := SegBtn{text: "ESC/POS thermal commands"}
                                        bill_deliv_escpos_on := SegBtnOn{text: "ESC/POS thermal commands" visible: false}
                                    }
                                }
                                bill_printers_dd := StackSecondaryBtn{text: "Printers for the bill" flow: Down}
                                bill_printers_input := FieldInput{
                                    height: 0.
                                    empty_text: ""
                                    margin: Inset{left: 0. right: 0. top: 0. bottom: 0.}
                                    padding: Inset{left: 0. right: 0. top: 0. bottom: 0.}
                                }
                                bill_printers_suggest := mod.widgets.StationCheckList{field: "bill_printers"}
                                }
                                FormCard{
View{width: Fill height: Fit flow: Right spacing: 8 align: Align{y: 0.5}
                                    h_bill_format := WrapLabel{width: Fit body: "Bill format" font_color: #x023637 font_size: 20.}
                                    bill_apply_all := QuietFitBtn{text: "Apply to all"}
                                }
                                h_bill_format_help := WrapLabel{
                                    width: Fill
                                    body: "Pick a line. Move up or down sets the print order. Show, size, weight, and alignment apply to that line."
                                    font_color: #x074D3E
                                    font_size: 12.
                                }
                                // One list. The selected row is the line you are editing.
                                View{width: Fill height: Fit flow: Down spacing: 4
                                    View{width: Fill height: Fit flow: Overlay
                                        bill_pick_shop := KotRowBtn{text: "Shop"}
                                        bill_pick_shop_on := KotRowBtnOn{text: "Shop" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        bill_pick_header := KotRowBtn{text: "Address"}
                                        bill_pick_header_on := KotRowBtnOn{text: "Address" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        bill_pick_fssai := KotRowBtn{text: "FSSAI"}
                                        bill_pick_fssai_on := KotRowBtnOn{text: "FSSAI" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        bill_pick_title := KotRowBtn{text: "Title"}
                                        bill_pick_title_on := KotRowBtnOn{text: "Title" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        bill_pick_customer := KotRowBtn{text: "Customer"}
                                        bill_pick_customer_on := KotRowBtnOn{text: "Customer" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        bill_pick_billno := KotRowBtn{text: "Bill no"}
                                        bill_pick_billno_on := KotRowBtnOn{text: "Bill no" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        bill_pick_time := KotRowBtn{text: "Time"}
                                        bill_pick_time_on := KotRowBtnOn{text: "Time" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        bill_pick_cashier := KotRowBtn{text: "Cashier"}
                                        bill_pick_cashier_on := KotRowBtnOn{text: "Cashier" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        bill_pick_table := KotRowBtn{text: "Table"}
                                        bill_pick_table_on := KotRowBtnOn{text: "Table" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        bill_pick_items := KotRowBtn{text: "Items"}
                                        bill_pick_items_on := KotRowBtnOn{text: "Items" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        bill_pick_total := KotRowBtn{text: "Total"}
                                        bill_pick_total_on := KotRowBtnOn{text: "Total" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        bill_pick_footer := KotRowBtn{text: "Footer"}
                                        bill_pick_footer_on := KotRowBtnOn{text: "Footer" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        bill_pick_upi := KotRowBtn{text: "UPI"}
                                        bill_pick_upi_on := KotRowBtnOn{text: "UPI" visible: false}
                                    }
                                }
                                View{width: Fill height: Fit flow: Right spacing: 6
                                    bill_field_up := StackSecondaryBtn{text: "Move up" flow: Down}
                                    bill_field_down := StackSecondaryBtn{text: "Move down" flow: Down}
                                }
                                View{width: Fill height: Fit flow: Right spacing: 6
                                    View{width: Fill height: Fit flow: Overlay
                                        bill_field_show := SegBtn{text: "Show"}
                                        bill_field_show_on := SegBtnOn{text: "Show" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        bill_field_hide := SegBtn{text: "Hide"}
                                        bill_field_hide_on := SegBtnOn{text: "Hide" visible: false}
                                    }
                                }
                                // Per-piece toggles for Items / Total — only
                                // visible while that line is open.
                                bill_item_pieces := View{
                                    width: Fill height: Fit
                                    flow: Down spacing: 8
                                    visible: false
                                    h_bill_item_layout := WrapLabel{body: "Item layout" font_color: #x023637 font_size: 14.}
                                    View{width: Fill height: Fit flow: Right spacing: 6
                                        View{width: Fill height: Fit flow: Overlay
                                            bill_item_layout_columns := SegBtn{text: "Columns"}
                                            bill_item_layout_columns_on := SegBtnOn{text: "Columns" visible: false}
                                        }
                                        View{width: Fill height: Fit flow: Overlay
                                            bill_item_layout_stacked := SegBtn{text: "Stacked"}
                                            bill_item_layout_stacked_on := SegBtnOn{text: "Stacked" visible: false}
                                        }
                                    }
                                    h_bill_item_header := WrapLabel{body: "Column headers" font_color: #x023637 font_size: 14.}
                                    View{width: Fill height: Fit flow: Right spacing: 6
                                        View{width: Fill height: Fit flow: Overlay
                                            bill_item_header_on := SegBtn{text: "On"}
                                            bill_item_header_on_on := SegBtnOn{text: "On" visible: false}
                                        }
                                        View{width: Fill height: Fit flow: Overlay
                                            bill_item_header_off := SegBtn{text: "Off"}
                                            bill_item_header_off_on := SegBtnOn{text: "Off" visible: false}
                                        }
                                    }
                                    bill_item_header_labels := View{
                                        width: Fill height: Fit
                                        flow: Down spacing: 8
                                        h_bill_item_hdr_item := WrapLabel{body: "Item header" font_color: #x023637 font_size: 14.}
                                        bill_item_hdr_item_input := FieldInput{empty_text: "Item"}
                                        h_bill_item_hdr_qty := WrapLabel{body: "Qty header" font_color: #x023637 font_size: 14.}
                                        bill_item_hdr_qty_input := FieldInput{empty_text: "Qty."}
                                        h_bill_item_hdr_price := WrapLabel{body: "Price header" font_color: #x023637 font_size: 14.}
                                        bill_item_hdr_price_input := FieldInput{empty_text: "Price"}
                                        h_bill_item_hdr_amount := WrapLabel{body: "Amount header" font_color: #x023637 font_size: 14.}
                                        bill_item_hdr_amount_input := FieldInput{empty_text: "Amount"}
                                    }
                                    h_bill_item_name := WrapLabel{body: "Dish name" font_color: #x023637 font_size: 14.}
                                    View{width: Fill height: Fit flow: Right spacing: 6
                                        View{width: Fill height: Fit flow: Overlay
                                            bill_item_name_on := SegBtn{text: "On"}
                                            bill_item_name_on_on := SegBtnOn{text: "On" visible: false}
                                        }
                                        View{width: Fill height: Fit flow: Overlay
                                            bill_item_name_off := SegBtn{text: "Off"}
                                            bill_item_name_off_on := SegBtnOn{text: "Off" visible: false}
                                        }
                                    }
                                    h_bill_item_qty := WrapLabel{body: "Qty" font_color: #x023637 font_size: 14.}
                                    View{width: Fill height: Fit flow: Right spacing: 6
                                        View{width: Fill height: Fit flow: Overlay
                                            bill_item_qty_on := SegBtn{text: "On"}
                                            bill_item_qty_on_on := SegBtnOn{text: "On" visible: false}
                                        }
                                        View{width: Fill height: Fit flow: Overlay
                                            bill_item_qty_off := SegBtn{text: "Off"}
                                            bill_item_qty_off_on := SegBtnOn{text: "Off" visible: false}
                                        }
                                    }
                                    h_bill_item_price := WrapLabel{body: "Price" font_color: #x023637 font_size: 14.}
                                    View{width: Fill height: Fit flow: Right spacing: 6
                                        View{width: Fill height: Fit flow: Overlay
                                            bill_item_price_on := SegBtn{text: "On"}
                                            bill_item_price_on_on := SegBtnOn{text: "On" visible: false}
                                        }
                                        View{width: Fill height: Fit flow: Overlay
                                            bill_item_price_off := SegBtn{text: "Off"}
                                            bill_item_price_off_on := SegBtnOn{text: "Off" visible: false}
                                        }
                                    }
                                    h_bill_item_amount := WrapLabel{body: "Amount" font_color: #x023637 font_size: 14.}
                                    View{width: Fill height: Fit flow: Right spacing: 6
                                        View{width: Fill height: Fit flow: Overlay
                                            bill_item_amount_on := SegBtn{text: "On"}
                                            bill_item_amount_on_on := SegBtnOn{text: "On" visible: false}
                                        }
                                        View{width: Fill height: Fit flow: Overlay
                                            bill_item_amount_off := SegBtn{text: "Off"}
                                            bill_item_amount_off_on := SegBtnOn{text: "Off" visible: false}
                                        }
                                    }
                                    h_bill_item_note := WrapLabel{body: "Dish note" font_color: #x023637 font_size: 14.}
                                    View{width: Fill height: Fit flow: Right spacing: 6
                                        View{width: Fill height: Fit flow: Overlay
                                            bill_item_note_on := SegBtn{text: "On"}
                                            bill_item_note_on_on := SegBtnOn{text: "On" visible: false}
                                        }
                                        View{width: Fill height: Fit flow: Overlay
                                            bill_item_note_off := SegBtn{text: "Off"}
                                            bill_item_note_off_on := SegBtnOn{text: "Off" visible: false}
                                        }
                                    }
                                    // Column air / reserved widths — only meaningful for Columns layout.
                                    bill_item_col_metrics := View{
                                        width: Fill height: Fit
                                        flow: Down spacing: 8
                                        h_bill_item_col_order := WrapLabel{body: "Column order" font_color: #x023637 font_size: 14.}
                                        View{width: Fill height: Fit flow: Down spacing: 4
                                            View{width: Fill height: Fit flow: Overlay
                                                bill_col_ord_0 := KotRowBtn{text: "Dish name"}
                                                bill_col_ord_0_on := KotRowBtnOn{text: "Dish name" visible: false}
                                            }
                                            View{width: Fill height: Fit flow: Overlay
                                                bill_col_ord_1 := KotRowBtn{text: "Qty"}
                                                bill_col_ord_1_on := KotRowBtnOn{text: "Qty" visible: false}
                                            }
                                            View{width: Fill height: Fit flow: Overlay
                                                bill_col_ord_2 := KotRowBtn{text: "Price"}
                                                bill_col_ord_2_on := KotRowBtnOn{text: "Price" visible: false}
                                            }
                                            View{width: Fill height: Fit flow: Overlay
                                                bill_col_ord_3 := KotRowBtn{text: "Amount"}
                                                bill_col_ord_3_on := KotRowBtnOn{text: "Amount" visible: false}
                                            }
                                        }
                                        View{width: Fill height: Fit flow: Right spacing: 6
                                            bill_col_ord_up := StackSecondaryBtn{text: "Move up" flow: Down}
                                            bill_col_ord_down := StackSecondaryBtn{text: "Move down" flow: Down}
                                        }
                                        h_bill_item_col_sep := WrapLabel{body: "Column rules" font_color: #x023637 font_size: 14.}
                                        View{width: Fill height: Fit flow: Right spacing: 6
                                            View{width: Fill height: Fit flow: Overlay
                                                bill_item_sep_on := SegBtn{text: "On"}
                                                bill_item_sep_on_on := SegBtnOn{text: "On" visible: false}
                                            }
                                            View{width: Fill height: Fit flow: Overlay
                                                bill_item_sep_off := SegBtn{text: "Off"}
                                                bill_item_sep_off_on := SegBtnOn{text: "Off" visible: false}
                                            }
                                        }
                                        h_bill_item_col_gap := WrapLabel{body: "Column gap" font_color: #x023637 font_size: 14.}
                                        View{width: Fill height: Fit flow: Right spacing: 6
                                            View{width: Fill height: Fit flow: Overlay
                                                bill_item_gap_tight := SegBtn{text: "Tight"}
                                                bill_item_gap_tight_on := SegBtnOn{text: "Tight" visible: false}
                                            }
                                            View{width: Fill height: Fit flow: Overlay
                                                bill_item_gap_comfort := SegBtn{text: "Comfortable"}
                                                bill_item_gap_comfort_on := SegBtnOn{text: "Comfortable" visible: false}
                                            }
                                            View{width: Fill height: Fit flow: Overlay
                                                bill_item_gap_wide := SegBtn{text: "Wide"}
                                                bill_item_gap_wide_on := SegBtnOn{text: "Wide" visible: false}
                                            }
                                        }
                                        h_bill_item_num_width := WrapLabel{body: "Number columns" font_color: #x023637 font_size: 14.}
                                        View{width: Fill height: Fit flow: Right spacing: 6
                                            View{width: Fill height: Fit flow: Overlay
                                                bill_item_num_compact := SegBtn{text: "Compact"}
                                                bill_item_num_compact_on := SegBtnOn{text: "Compact" visible: false}
                                            }
                                            View{width: Fill height: Fit flow: Overlay
                                                bill_item_num_comfort := SegBtn{text: "Comfortable"}
                                                bill_item_num_comfort_on := SegBtnOn{text: "Comfortable" visible: false}
                                            }
                                            View{width: Fill height: Fit flow: Overlay
                                                bill_item_num_wide := SegBtn{text: "Wide"}
                                                bill_item_num_wide_on := SegBtnOn{text: "Wide" visible: false}
                                            }
                                        }
                                        h_bill_item_num_align := WrapLabel{body: "Numbers align" font_color: #x023637 font_size: 14.}
                                        View{width: Fill height: Fit flow: Right spacing: 6
                                            View{width: Fill height: Fit flow: Overlay
                                                bill_item_num_start := SegBtn{text: "Start"}
                                                bill_item_num_start_on := SegBtnOn{text: "Start" visible: false}
                                            }
                                            View{width: Fill height: Fit flow: Overlay
                                                bill_item_num_center := SegBtn{text: "Center"}
                                                bill_item_num_center_on := SegBtnOn{text: "Center" visible: false}
                                            }
                                            View{width: Fill height: Fit flow: Overlay
                                                bill_item_num_end := SegBtn{text: "End"}
                                                bill_item_num_end_on := SegBtnOn{text: "End" visible: false}
                                            }
                                        }
                                        h_bill_item_gap_em := WrapLabel{body: "Gap (em)" font_color: #x023637 font_size: 14.}
                                        bill_item_gap_em_input := FieldInput{empty_text: "e.g. 1.15"}
                                        h_bill_item_qty_min := WrapLabel{body: "Qty min (em)" font_color: #x023637 font_size: 14.}
                                        bill_item_qty_min_input := FieldInput{empty_text: "e.g. 3.5"}
                                        h_bill_item_price_min := WrapLabel{body: "Price min (em)" font_color: #x023637 font_size: 14.}
                                        bill_item_price_min_input := FieldInput{empty_text: "e.g. 7.5"}
                                        h_bill_item_amount_min := WrapLabel{body: "Amount min (em)" font_color: #x023637 font_size: 14.}
                                        bill_item_amount_min_input := FieldInput{empty_text: "e.g. 8.5"}
                                    }
                                }
                                bill_total_pieces := View{
                                    width: Fill height: Fit
                                    flow: Down spacing: 8
                                    visible: false
                                    h_bill_total_qty := WrapLabel{body: "Total qty" font_color: #x023637 font_size: 14.}
                                    View{width: Fill height: Fit flow: Right spacing: 6
                                        View{width: Fill height: Fit flow: Overlay
                                            bill_total_qty_on := SegBtn{text: "On"}
                                            bill_total_qty_on_on := SegBtnOn{text: "On" visible: false}
                                        }
                                        View{width: Fill height: Fit flow: Overlay
                                            bill_total_qty_off := SegBtn{text: "Off"}
                                            bill_total_qty_off_on := SegBtnOn{text: "Off" visible: false}
                                        }
                                    }
                                    h_bill_total_qty_label := WrapLabel{body: "Total qty label" font_color: #x023637 font_size: 14.}
                                    bill_total_qty_label_input := FieldInput{empty_text: "Total Qty"}
                                    h_bill_total_subtotal := WrapLabel{body: "Sub total" font_color: #x023637 font_size: 14.}
                                    View{width: Fill height: Fit flow: Right spacing: 6
                                        View{width: Fill height: Fit flow: Overlay
                                            bill_total_subtotal_on := SegBtn{text: "On"}
                                            bill_total_subtotal_on_on := SegBtnOn{text: "On" visible: false}
                                        }
                                        View{width: Fill height: Fit flow: Overlay
                                            bill_total_subtotal_off := SegBtn{text: "Off"}
                                            bill_total_subtotal_off_on := SegBtnOn{text: "Off" visible: false}
                                        }
                                    }
                                    h_bill_total_subtotal_label := WrapLabel{body: "Sub total label" font_color: #x023637 font_size: 14.}
                                    bill_total_subtotal_label_input := FieldInput{empty_text: "Sub Total"}
                                    h_bill_total_grand := WrapLabel{body: "Grand total" font_color: #x023637 font_size: 14.}
                                    View{width: Fill height: Fit flow: Right spacing: 6
                                        View{width: Fill height: Fit flow: Overlay
                                            bill_total_grand_on := SegBtn{text: "On"}
                                            bill_total_grand_on_on := SegBtnOn{text: "On" visible: false}
                                        }
                                        View{width: Fill height: Fit flow: Overlay
                                            bill_total_grand_off := SegBtn{text: "Off"}
                                            bill_total_grand_off_on := SegBtnOn{text: "Off" visible: false}
                                        }
                                    }
                                    h_bill_total_grand_label := WrapLabel{body: "Grand total label" font_color: #x023637 font_size: 14.}
                                    bill_total_grand_label_input := FieldInput{empty_text: "Grand Total"}
                                    h_bill_total_paid_label := WrapLabel{body: "Paid total label" font_color: #x023637 font_size: 14.}
                                    bill_total_paid_label_input := FieldInput{empty_text: "Total"}
                                }
                                bill_meta_pair := View{
                                    width: Fill height: Fit
                                    flow: Down spacing: 8
                                    visible: false
                                    h_bill_meta_pair := WrapLabel{body: "Share line with" font_color: #x023637 font_size: 14.}
                                    View{width: Fill height: Fit flow: Right spacing: 6
                                        View{width: Fill height: Fit flow: Overlay
                                            bill_pair_alone := SegBtn{text: "Alone"}
                                            bill_pair_alone_on := SegBtnOn{text: "Alone" visible: false}
                                        }
                                        View{width: Fill height: Fit flow: Overlay
                                            bill_pair_customer := SegBtn{text: "Customer"}
                                            bill_pair_customer_on := SegBtnOn{text: "Customer" visible: false}
                                        }
                                        View{width: Fill height: Fit flow: Overlay
                                            bill_pair_billno := SegBtn{text: "Bill no"}
                                            bill_pair_billno_on := SegBtnOn{text: "Bill no" visible: false}
                                        }
                                    }
                                    View{width: Fill height: Fit flow: Right spacing: 6
                                        View{width: Fill height: Fit flow: Overlay
                                            bill_pair_time := SegBtn{text: "Time"}
                                            bill_pair_time_on := SegBtnOn{text: "Time" visible: false}
                                        }
                                        View{width: Fill height: Fit flow: Overlay
                                            bill_pair_cashier := SegBtn{text: "Cashier"}
                                            bill_pair_cashier_on := SegBtnOn{text: "Cashier" visible: false}
                                        }
                                        View{width: Fill height: Fit flow: Overlay
                                            bill_pair_table := SegBtn{text: "Table"}
                                            bill_pair_table_on := SegBtnOn{text: "Table" visible: false}
                                        }
                                    }
                                }
                                h_bill_field_size := WrapLabel{body: "Font size" font_color: #x023637 font_size: 14.}
                                bill_field_size_input := FieldInput{empty_text: "e.g. 16, 20, 24, 28, 32…"}
                                View{width: Fill height: Fit flow: Right spacing: 6
                                    View{width: Fill height: Fit flow: Overlay
                                        bill_w_light := SegBtn{text: "Light"}
                                        bill_w_light_on := SegBtnOn{text: "Light" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        bill_w_reg := SegBtn{text: "Regular"}
                                        bill_w_reg_on := SegBtnOn{text: "Regular" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        bill_w_bold := SegBtn{text: "Bold"}
                                        bill_w_bold_on := SegBtnOn{text: "Bold" visible: false}
                                    }
                                }
                                h_bill_field_align := WrapLabel{body: "Align" font_color: #x023637 font_size: 14.}
                                View{width: Fill height: Fit flow: Right spacing: 6
                                    View{width: Fill height: Fit flow: Overlay
                                        bill_a_start := SegBtn{text: "Start"}
                                        bill_a_start_on := SegBtnOn{text: "Start" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        bill_a_center := SegBtn{text: "Center"}
                                        bill_a_center_on := SegBtnOn{text: "Center" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        bill_a_end := SegBtn{text: "End"}
                                        bill_a_end_on := SegBtnOn{text: "End" visible: false}
                                    }
                                }
                                // One box. Its hint is the selected line (address, FSSAI, title, footer, UPI).
                                bill_field_value := FieldInput{empty_text: "Address | phone"}
                                // Kept so older saves and hot reload still find the ids.
                                View{width: Fill height: Fit visible: false
                                    bill_header_input := FieldInput{empty_text: "Address | phone"}
                                    fssai_input := FieldInput{empty_text: "FSSAI licence no. (India)"}
                                    upi_vpa_input := FieldInput{empty_text: "UPI ID (VPA)"}
                                    upi_payee_input := FieldInput{empty_text: "UPI payee name (optional)"}
                                    bill_footer_input := FieldInput{empty_text: "Footer (e.g. Thank you, visit again)"}
                                }
                                View{width: Fill height: Fit visible: false
                                h_bill_field_order := WrapLabel{body: "Print order" font_color: #x023637 font_size: 14.}
                                View{width: Fill height: Fit flow: Down spacing: 4
                                    View{width: Fill height: Fit flow: Overlay
                                        bill_ord_0 := KotRowBtn{text: ""}
                                        bill_ord_0_on := KotRowBtnOn{text: "" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        bill_ord_1 := KotRowBtn{text: ""}
                                        bill_ord_1_on := KotRowBtnOn{text: "" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        bill_ord_2 := KotRowBtn{text: ""}
                                        bill_ord_2_on := KotRowBtnOn{text: "" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        bill_ord_3 := KotRowBtn{text: ""}
                                        bill_ord_3_on := KotRowBtnOn{text: "" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        bill_ord_4 := KotRowBtn{text: ""}
                                        bill_ord_4_on := KotRowBtnOn{text: "" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        bill_ord_5 := KotRowBtn{text: ""}
                                        bill_ord_5_on := KotRowBtnOn{text: "" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        bill_ord_6 := KotRowBtn{text: ""}
                                        bill_ord_6_on := KotRowBtnOn{text: "" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        bill_ord_7 := KotRowBtn{text: ""}
                                        bill_ord_7_on := KotRowBtnOn{text: "" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        bill_ord_8 := KotRowBtn{text: ""}
                                        bill_ord_8_on := KotRowBtnOn{text: "" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        bill_ord_9 := KotRowBtn{text: ""}
                                        bill_ord_9_on := KotRowBtnOn{text: "" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        bill_ord_10 := KotRowBtn{text: ""}
                                        bill_ord_10_on := KotRowBtnOn{text: "" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        bill_ord_11 := KotRowBtn{text: ""}
                                        bill_ord_11_on := KotRowBtnOn{text: "" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        bill_ord_12 := KotRowBtn{text: ""}
                                        bill_ord_12_on := KotRowBtnOn{text: "" visible: false}
                                    }
                                }
                                }
                                h_bill_copies := WrapLabel{body: "Copies" font_color: #x023637 font_size: 14.}
                                View{width: Fill height: Fit flow: Right spacing: 6
                                    View{width: Fill height: Fit flow: Overlay
                                        bill_copies_1 := SegBtn{text: "1"}
                                        bill_copies_1_on := SegBtnOn{text: "1" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        bill_copies_2 := SegBtn{text: "2"}
                                        bill_copies_2_on := SegBtnOn{text: "2" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        bill_copies_3 := SegBtn{text: "3"}
                                        bill_copies_3_on := SegBtnOn{text: "3" visible: false}
                                    }
                                }
                                h_bill_preview := WrapLabel{body: "Preview" font_color: #x023637 font_size: 14.}
                                bill_format_preview_card := View{
                                    width: Fill height: Fit
                                    bill_format_preview := mod.widgets.KotSlipPreview{}
                                }
                                bill_format_save := StackPrimaryBtn{text: "Save bill format"}
                                bill_format_test := StackSecondaryBtn{text: "Test bill"}
                                }
                                FormCard{
                                visible: false
                                View{
                                    width: Fill height: Fit
                                    flow: Right spacing: 0
                                    align: Align{x: 0.0 y: 0.5}
                                    h_std_print := WrapLabel{
                                                    width: Fit
                                                    body: "Bill print size" font_color: #x023637 font_size: 20.}
                                    std_print_help := InfoHelp{
                                        tip: "Font size for Standard Portrait and Landscape bill jobs on OS / document printers (enter points)."
                                    }
                                }
                                h_std_size := WrapLabel{
                                    body: "Font size"
                                    font_color: #x023637
                                    font_size: 14.
                                }
                                std_size_input := FieldInput{empty_text: "e.g. 18, 20, 26…"}
                                // Legacy hot-reload ids — hidden; numeric size replaces Small/Normal/Large.
                                View{width: Fill height: Fit visible: false
                                    std_size_s := SegBtn{text: ""}
                                    std_size_s_on := SegBtnOn{text: "" visible: false}
                                    std_size_m := SegBtn{text: ""}
                                    std_size_m_on := SegBtnOn{text: "" visible: false}
                                    std_size_l := SegBtn{text: ""}
                                    std_size_l_on := SegBtnOn{text: "" visible: false}
                                }
                                std_print_save := StackPrimaryBtn{text: "Save bill print size"}
                                }
                                } // bill_format_body
                                kot_format_body := View{
                                    width: Fill height: Fit
                                    flow: Down spacing: 12
                                    visible: false
                                FormCard{
                                h_kot_layout := WrapLabel{body: "Layout" font_color: #x023637 font_size: 14.}
                                View{width: Fill height: Fit flow: Right spacing: 6
                                    View{width: Fill height: Fit flow: Overlay
                                        kot_layout_portrait := SegBtn{text: "Portrait"}
                                        kot_layout_portrait_on := SegBtnOn{text: "Portrait" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        kot_layout_landscape := SegBtn{text: "Landscape"}
                                        kot_layout_landscape_on := SegBtnOn{text: "Landscape" visible: false}
                                    }
                                }
                                View{width: Fill height: Fit flow: Right spacing: 0 align: Align{y: 0.5}
                                    h_kot_delivery := WrapLabel{width: Fit body: "How the job is sent" font_color: #x023637 font_size: 14.}
                                    kot_delivery_help := InfoHelp{tip: "OS document, raw bytes, or ESC/POS for the bill. Route to any printer."}
                                }
                                View{width: Fill height: Fit flow: Down spacing: 6
                                    View{width: Fill height: Fit flow: Overlay
                                        kot_deliv_os := SegBtn{text: "OS document (system spooler)"}
                                        kot_deliv_os_on := SegBtnOn{text: "OS document (system spooler)" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        kot_deliv_raw := SegBtn{text: "Raw bytes to device"}
                                        kot_deliv_raw_on := SegBtnOn{text: "Raw bytes to device" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        kot_deliv_escpos := SegBtn{text: "ESC/POS thermal commands"}
                                        kot_deliv_escpos_on := SegBtnOn{text: "ESC/POS thermal commands" visible: false}
                                    }
                                }
                                }
                                FormCard{
                                // Kind first — styles below apply to the selected kind.
                                h_kot_kind := WrapLabel{
                                    body: "KOT kind"
                                    font_color: #x023637
                                    font_size: 14.
                                }
                                View{
                                    width: Fill height: Fit
                                    flow: Right spacing: 6
                                    View{width: Fill height: Fit flow: Overlay
                                        kot_kind_original := SegBtn{text: "Original"}
                                        kot_kind_original_on := SegBtnOn{text: "Original" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        kot_kind_duplicate := SegBtn{text: "Duplicate"}
                                        kot_kind_duplicate_on := SegBtnOn{text: "Duplicate" visible: false}
                                    }
                                }
                                View{
                                    width: Fill height: Fit
                                    flow: Right spacing: 6
                                    View{width: Fill height: Fit flow: Overlay
                                        kot_kind_modified := SegBtn{text: "Modified"}
                                        kot_kind_modified_on := SegBtnOn{text: "Modified" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        kot_kind_cancelled := SegBtn{text: "Cancelled"}
                                        kot_kind_cancelled_on := SegBtnOn{text: "Cancelled" visible: false}
                                    }
                                }
                                h_kot_banner := WrapLabel{
                                    body: "Kind banner"
                                    font_color: #x023637
                                    font_size: 14.
                                }
                                kot_banner_input := FieldInput{empty_text: "Banner (blank = none)"}
                                h_kot_item_prefix := WrapLabel{
                                    body: "Item prefix"
                                    font_color: #x023637
                                    font_size: 14.
                                }
                                kot_item_prefix_input := FieldInput{empty_text: "e.g. [CANCELLED] "}
                                h_kot_strike := WrapLabel{
                                    body: "Strike dish name"
                                    font_color: #x023637
                                    font_size: 14.
                                }
                                View{
                                    width: Fill height: Fit
                                    flow: Right spacing: 6
                                    View{width: Fill height: Fit flow: Overlay
                                        kot_strike_on := SegBtn{text: "On"}
                                        kot_strike_on_on := SegBtnOn{text: "On" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        kot_strike_off := SegBtn{text: "Off"}
                                        kot_strike_off_on := SegBtnOn{text: "Off" visible: false}
                                    }
                                }
                                h_kot_strike_note := WrapLabel{
                                    body: "Strike dish note"
                                    font_color: #x023637
                                    font_size: 14.
                                }
                                View{
                                    width: Fill height: Fit
                                    flow: Right spacing: 6
                                    View{width: Fill height: Fit flow: Overlay
                                        kot_strike_note_on := SegBtn{text: "On"}
                                        kot_strike_note_on_on := SegBtnOn{text: "On" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        kot_strike_note_off := SegBtn{text: "Off"}
                                        kot_strike_note_off_on := SegBtnOn{text: "Off" visible: false}
                                    }
                                }
                                // One place per field: pick field → show + font size + weight.
                                View{width: Fill height: Fit flow: Right spacing: 8 align: Align{y: 0.5}
                                    h_kot_format_fields := WrapLabel{
                                        width: Fit
                                        body: "Fields"
                                        font_color: #x023637
                                        font_size: 14.
                                    }
                                    kot_apply_all := QuietFitBtn{text: "Apply to all"}
                                }
                                kot_format_help := InfoHelp{
                                    tip: "Pick a field. Set show/hide, font size (points), and weight. Content width is above."
                                    visible: false
                                }
                                kot_fields_summary := mod.widgets.ReportText{visible: false}
                                View{
                                    width: Fill height: Fit
                                    flow: Down spacing: 4
                                    View{width: Fill height: Fit flow: Overlay
                                        kot_pick_title := KotRowBtn{text: "Title"}
                                        kot_pick_title_on := KotRowBtnOn{text: "Title" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        kot_pick_shop := KotRowBtn{text: "Shop"}
                                        kot_pick_shop_on := KotRowBtnOn{text: "Shop" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        kot_pick_time := KotRowBtn{text: "Time"}
                                        kot_pick_time_on := KotRowBtnOn{text: "Time" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        kot_pick_waiter := KotRowBtn{text: "Waiter"}
                                        kot_pick_waiter_on := KotRowBtnOn{text: "Waiter" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        kot_pick_station := KotRowBtn{text: "Station"}
                                        kot_pick_station_on := KotRowBtnOn{text: "Station" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        kot_pick_table := KotRowBtn{text: "Table"}
                                        kot_pick_table_on := KotRowBtnOn{text: "Table" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        kot_pick_no := KotRowBtn{text: "Ticket no"}
                                        kot_pick_no_on := KotRowBtnOn{text: "Ticket no" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        kot_pick_area := KotRowBtn{text: "Area"}
                                        kot_pick_area_on := KotRowBtnOn{text: "Area" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        kot_pick_qty := KotRowBtn{text: "Item / Qty"}
                                        kot_pick_qty_on := KotRowBtnOn{text: "Item / Qty" visible: false}
                                    }
                                }
                                // Legacy hot-reload ids — tap a field row instead of cycling Next.
                                View{width: Fill height: Fit visible: false
                                    kot_style_field_cycle := StackLightBtn{text: "" flow: Down}
                                }
                                kot_style_field_label := mod.widgets.ReportText{visible: false}
                                View{
                                    width: Fill height: Fit
                                    flow: Right spacing: 6
                                    View{width: Fill height: Fit flow: Overlay
                                        kot_field_show := SegBtn{text: "Show"}
                                        kot_field_show_on := SegBtnOn{text: "Show" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        kot_field_hide := SegBtn{text: "Hide"}
                                        kot_field_hide_on := SegBtnOn{text: "Hide" visible: false}
                                    }
                                }
                                h_kot_field_size := WrapLabel{
                                    body: "Font size"
                                    font_color: #x023637
                                    font_size: 14.
                                }
                                kot_field_size_input := FieldInput{empty_text: "e.g. 16, 20, 24, 28, 32…"}
                                View{
                                    width: Fill height: Fit
                                    flow: Right spacing: 6
                                    View{width: Fill height: Fit flow: Overlay
                                        kot_field_weight_light := SegBtn{text: "Light"}
                                        kot_field_weight_light_on := SegBtnOn{text: "Light" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        kot_field_weight_reg := SegBtn{text: "Regular"}
                                        kot_field_weight_reg_on := SegBtnOn{text: "Regular" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        kot_field_weight_bold := SegBtn{text: "Bold"}
                                        kot_field_weight_bold_on := SegBtnOn{text: "Bold" visible: false}
                                    }
                                }
                                h_kot_field_align := WrapLabel{
                                    body: "Align"
                                    font_color: #x023637
                                    font_size: 14.
                                }
                                View{
                                    width: Fill height: Fit
                                    flow: Right spacing: 6
                                    View{width: Fill height: Fit flow: Overlay
                                        kot_align_start := SegBtn{text: "Start"}
                                        kot_align_start_on := SegBtnOn{text: "Start" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        kot_align_center := SegBtn{text: "Center"}
                                        kot_align_center_on := SegBtnOn{text: "Center" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        kot_align_end := SegBtn{text: "End"}
                                        kot_align_end_on := SegBtnOn{text: "End" visible: false}
                                    }
                                }
                                // Item / Qty column air — only while that field is open.
                                kot_item_col_metrics := View{
                                    width: Fill height: Fit
                                    flow: Down spacing: 8
                                    visible: false
                                    h_kot_item_col_gap := WrapLabel{body: "Column gap" font_color: #x023637 font_size: 14.}
                                    View{width: Fill height: Fit flow: Right spacing: 6
                                        View{width: Fill height: Fit flow: Overlay
                                            kot_item_gap_tight := SegBtn{text: "Tight"}
                                            kot_item_gap_tight_on := SegBtnOn{text: "Tight" visible: false}
                                        }
                                        View{width: Fill height: Fit flow: Overlay
                                            kot_item_gap_comfort := SegBtn{text: "Comfortable"}
                                            kot_item_gap_comfort_on := SegBtnOn{text: "Comfortable" visible: false}
                                        }
                                        View{width: Fill height: Fit flow: Overlay
                                            kot_item_gap_wide := SegBtn{text: "Wide"}
                                            kot_item_gap_wide_on := SegBtnOn{text: "Wide" visible: false}
                                        }
                                    }
                                    h_kot_item_qty_width := WrapLabel{body: "Qty column" font_color: #x023637 font_size: 14.}
                                    View{width: Fill height: Fit flow: Right spacing: 6
                                        View{width: Fill height: Fit flow: Overlay
                                            kot_item_num_compact := SegBtn{text: "Compact"}
                                            kot_item_num_compact_on := SegBtnOn{text: "Compact" visible: false}
                                        }
                                        View{width: Fill height: Fit flow: Overlay
                                            kot_item_num_comfort := SegBtn{text: "Comfortable"}
                                            kot_item_num_comfort_on := SegBtnOn{text: "Comfortable" visible: false}
                                        }
                                        View{width: Fill height: Fit flow: Overlay
                                            kot_item_num_wide := SegBtn{text: "Wide"}
                                            kot_item_num_wide_on := SegBtnOn{text: "Wide" visible: false}
                                        }
                                    }
                                    h_kot_item_num_align := WrapLabel{body: "Qty align" font_color: #x023637 font_size: 14.}
                                    View{width: Fill height: Fit flow: Right spacing: 6
                                        View{width: Fill height: Fit flow: Overlay
                                            kot_item_num_start := SegBtn{text: "Start"}
                                            kot_item_num_start_on := SegBtnOn{text: "Start" visible: false}
                                        }
                                        View{width: Fill height: Fit flow: Overlay
                                            kot_item_num_center := SegBtn{text: "Center"}
                                            kot_item_num_center_on := SegBtnOn{text: "Center" visible: false}
                                        }
                                        View{width: Fill height: Fit flow: Overlay
                                            kot_item_num_end := SegBtn{text: "End"}
                                            kot_item_num_end_on := SegBtnOn{text: "End" visible: false}
                                        }
                                    }
                                    h_kot_item_hdr_item := WrapLabel{body: "Item header" font_color: #x023637 font_size: 14.}
                                    kot_item_hdr_item_input := FieldInput{empty_text: "Item"}
                                    h_kot_item_hdr_qty := WrapLabel{body: "Qty header" font_color: #x023637 font_size: 14.}
                                    kot_item_hdr_qty_input := FieldInput{empty_text: "Qty."}
                                    h_kot_item_gap_em := WrapLabel{body: "Gap (em)" font_color: #x023637 font_size: 14.}
                                    kot_item_gap_em_input := FieldInput{empty_text: "e.g. 0.7"}
                                    h_kot_item_qty_min := WrapLabel{body: "Qty min (em)" font_color: #x023637 font_size: 14.}
                                    kot_item_qty_min_input := FieldInput{empty_text: "e.g. 3.5"}
                                }
                                // Legacy hot-reload ids — kept hidden so Live reload does not break.
                                View{width: Fill height: Fit visible: false
                                    h_kot_format := WrapLabel{body: ""}
                                    kot_shop_on := SegBtn{text: ""}
                                    kot_shop_on_on := SegBtnOn{text: "" visible: false}
                                    kot_shop_off := SegBtn{text: ""}
                                    kot_shop_off_on := SegBtnOn{text: "" visible: false}
                                    kot_time_on := SegBtn{text: ""}
                                    kot_time_on_on := SegBtnOn{text: "" visible: false}
                                    kot_time_off := SegBtn{text: ""}
                                    kot_time_off_on := SegBtnOn{text: "" visible: false}
                                    kot_waiter_on := SegBtn{text: ""}
                                    kot_waiter_on_on := SegBtnOn{text: "" visible: false}
                                    kot_waiter_off := SegBtn{text: ""}
                                    kot_waiter_off_on := SegBtnOn{text: "" visible: false}
                                    kot_station_on := SegBtn{text: ""}
                                    kot_station_on_on := SegBtnOn{text: "" visible: false}
                                    kot_station_off := SegBtn{text: ""}
                                    kot_station_off_on := SegBtnOn{text: "" visible: false}
                                    kot_table_on := SegBtn{text: ""}
                                    kot_table_on_on := SegBtnOn{text: "" visible: false}
                                    kot_table_off := SegBtn{text: ""}
                                    kot_table_off_on := SegBtnOn{text: "" visible: false}
                                    kot_no_on := SegBtn{text: ""}
                                    kot_no_on_on := SegBtnOn{text: "" visible: false}
                                    kot_no_off := SegBtn{text: ""}
                                    kot_no_off_on := SegBtnOn{text: "" visible: false}
                                    kot_area_on := SegBtn{text: ""}
                                    kot_area_on_on := SegBtnOn{text: "" visible: false}
                                    kot_area_off := SegBtn{text: ""}
                                    kot_area_off_on := SegBtnOn{text: "" visible: false}
                                    kot_item_hdr_on := SegBtn{text: ""}
                                    kot_item_hdr_on_on := SegBtnOn{text: "" visible: false}
                                    kot_item_hdr_off := SegBtn{text: ""}
                                    kot_item_hdr_off_on := SegBtnOn{text: "" visible: false}
                                    kot_title_on := SegBtn{text: ""}
                                    kot_title_on_on := SegBtnOn{text: "" visible: false}
                                    kot_title_off := SegBtn{text: ""}
                                    kot_title_off_on := SegBtnOn{text: "" visible: false}
                                    kot_field_size_s := SegBtn{text: ""}
                                    kot_field_size_s_on := SegBtnOn{text: "" visible: false}
                                    kot_field_size_m := SegBtn{text: ""}
                                    kot_field_size_m_on := SegBtnOn{text: "" visible: false}
                                    kot_field_size_l := SegBtn{text: ""}
                                    kot_field_size_l_on := SegBtnOn{text: "" visible: false}
                                    h_kot_width := WrapLabel{body: ""}
                                    kot_width_58 := SegBtn{text: ""}
                                    kot_width_58_on := SegBtnOn{text: "" visible: false}
                                    kot_width_80 := SegBtn{text: ""}
                                    kot_width_80_on := SegBtnOn{text: "" visible: false}
                                    h_kot_style := WrapLabel{body: ""}
                                }
                                // Print order for header lines — same selected field as above (no second picker).
                                h_kot_field_order := WrapLabel{
                                    body: "Print order"
                                    font_color: #x023637
                                    font_size: 14.
                                }
                                kot_field_order_label := mod.widgets.ReportText{visible: false}
                                View{
                                    width: Fill height: Fit
                                    flow: Down spacing: 4
                                    View{width: Fill height: Fit flow: Overlay
                                        kot_ord_0 := KotRowBtn{text: ""}
                                        kot_ord_0_on := KotRowBtnOn{text: "" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        kot_ord_1 := KotRowBtn{text: ""}
                                        kot_ord_1_on := KotRowBtnOn{text: "" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        kot_ord_2 := KotRowBtn{text: ""}
                                        kot_ord_2_on := KotRowBtnOn{text: "" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        kot_ord_3 := KotRowBtn{text: ""}
                                        kot_ord_3_on := KotRowBtnOn{text: "" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        kot_ord_4 := KotRowBtn{text: ""}
                                        kot_ord_4_on := KotRowBtnOn{text: "" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        kot_ord_5 := KotRowBtn{text: ""}
                                        kot_ord_5_on := KotRowBtnOn{text: "" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        kot_ord_6 := KotRowBtn{text: ""}
                                        kot_ord_6_on := KotRowBtnOn{text: "" visible: false}
                                    }
                                }
                                View{
                                    width: Fill height: Fit
                                    flow: Right spacing: 6
                                    kot_field_up := StackSecondaryBtn{text: "Move up" flow: Down}
                                    kot_field_down := StackSecondaryBtn{text: "Move down" flow: Down}
                                }
                                // Legacy hot-reload id — style field cycle is the only picker.
                                View{width: Fill height: Fit visible: false
                                    kot_field_cycle := StackLightBtn{text: "" flow: Down}
                                }
                                h_kot_copies := WrapLabel{
                                    body: "Copies"
                                    font_color: #x023637
                                    font_size: 14.
                                }
                                View{
                                    width: Fill height: Fit
                                    flow: Right spacing: 6
                                    View{width: Fill height: Fit flow: Overlay
                                        kot_copies_1 := SegBtn{text: "1"}
                                        kot_copies_1_on := SegBtnOn{text: "1" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        kot_copies_2 := SegBtn{text: "2"}
                                        kot_copies_2_on := SegBtnOn{text: "2" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        kot_copies_3 := SegBtn{text: "3"}
                                        kot_copies_3_on := SegBtnOn{text: "3" visible: false}
                                    }
                                }
                                kot_title_input := FieldInput{empty_text: "Title (blank = KOT - KITCHEN ORDER)"}
                                kot_footer_input := FieldInput{empty_text: "Slip note (optional, no prices)"}
                                h_kot_preview := WrapLabel{
                                    body: "Preview"
                                    font_color: #x023637
                                    font_size: 14.
                                }
                                // Paper strip width updates with the printer.
                                // KotSlipPreview: Item and Qty. share one right column.
                                kot_format_preview_card := View{
                                    width: Fill height: Fit
                                    kot_format_preview := mod.widgets.KotSlipPreview{}
                                }
                                kot_format_save := StackPrimaryBtn{text: "Save KOT format"}
                                kot_test_printers_dd := StackSecondaryBtn{text: "Printers for Test KOT" flow: Down}
                                kot_test_printers_input := FieldInput{
                                    height: 0.
                                    empty_text: ""
                                    margin: Inset{left: 0. right: 0. top: 0. bottom: 0.}
                                    padding: Inset{left: 0. right: 0. top: 0. bottom: 0.}
                                }
                                kot_test_printers_suggest := mod.widgets.StationCheckList{field: "kot_test_printers"}
                                kot_format_test := StackSecondaryBtn{text: "Test KOT"}
                                }
                                } // kot_format_body
                                                                }
                                settings_panel_selling := View{
                                    width: Fill height: Fit
                                    flow: Down spacing: 12
                                    visible: false
                                    FormCard{
                                        View{
                                            width: Fill height: Fit
                                            flow: Right spacing: 0
                                            align: Align{x: 0.0 y: 0.5}
                                            h_schemes := WrapLabel{
                                                width: Fit
                                                body: "Schemes / offers" font_color: #x023637 font_size: 20.}
                                            settings_schemes_help := InfoHelp{
                                                tip: "Buy X get Y — applied automatically when you settle. Example: Buy 2 paracetamol get 1 vitamin."
                                            }
                                        }
                                        schemes_label := mod.widgets.ReportText{}
                                        scheme_bogo_input := FieldInput{empty_text: "Buy 2 paracetamol get 1 vitamin"}
                                        scheme_add := StackPrimaryBtn{text: "Add BOGO"}
                                    }
                                    FormCard{
                                        View{
                                            width: Fill height: Fit
                                            flow: Right spacing: 0
                                            align: Align{x: 0.0 y: 0.5}
                                            h_zero_sales := WrapLabel{
                                                width: Fit
                                                body: "Zero-value sales" font_color: #x023637 font_size: 20.}
                                            settings_zero_sales_help := InfoHelp{
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
                                            flow: Right spacing: 0
                                            align: Align{x: 0.0 y: 0.5}
                                            h_aggregator_import := WrapLabel{
                                                width: Fit
                                                body: "Aggregator orders (file)" font_color: #x023637 font_size: 18.}
                                            settings_aggregator_help := InfoHelp{
                                                tip: "Optional: put import_aggregator_orders.json in the backups folder, then tap Import. Never blocks the counter; live delivery apps are Cloud."
                                            }
                                        }
                                        ActionStack{
                                            aggregator_import_btn := StackSecondaryBtn{text: "Import aggregator file"}
                                            aggregator_import_cancel := StackLightBtn{text: "Cancel" visible: false}
                                            aggregator_import_progress := WrapLabel{visible: false body: "" font_color: #x023637 font_size: 15.}
                                        }
                                    }
                                }
                                settings_panel_alerts := View{
                                    width: Fill height: Fit
                                    flow: Down spacing: 12
                                    visible: false
                                    FormCard{
                                        h_alerts := WrapLabel{body: "Alerts" font_color: #x023637 font_size: 20.}
                                        h_alerts_help := WrapLabel{
                                            width: Fill
                                            body: "How long status messages stay on screen for you (this login)."
                                            font_color: #x074D3E
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
h_backup := WrapLabel{body: "Backup" font_color: #x023637 font_size: 20.}
                                backup_auto_label := WrapLabel{
                                    width: Fill
                                    body: ""
                                    font_color: #x074D3E
                                    font_size: 12.
                                }
                                ActionStack{
                                    backup_copy_btn := NavBtn{text: "Also copy backups to a folder…"}
                                    backup_copy_cancel := StackLightBtn{text: "Cancel" visible: false}
                                    backup_copy_progress := WrapLabel{visible: false body: "" font_color: #x023637 font_size: 15.}
                                }
                                db_path_label := WrapLabel{body: "" font_color: #x023637}
                                audit_label := mod.widgets.ReportText{}
                                ActionStack{
                                    backup_btn := StackPrimaryBtn{text: "Create backup"}
                                    backup_cancel := StackLightBtn{text: "Cancel" visible: false}
                                    backup_progress := WrapLabel{visible: false body: "" font_color: #x023637 font_size: 15.}
                                }
                                backup_path_label := WrapLabel{body: "" font_color: #x023637}
                                h_restore_help := WrapLabel{
                                    width: Fill
                                    body: "Restore: put restore.sqlite3 in the backups folder, then tap Restore."
                                    font_color: #x023637
                                }
                                ActionStack{
                                    restore_btn := NavBtn{text: "Restore backup"}
                                    restore_cancel := StackLightBtn{text: "Cancel" visible: false}
                                    restore_progress := WrapLabel{visible: false body: "" font_color: #x023637 font_size: 15.}
                                }
                                h_history_retention := WrapLabel{
                                    width: Fill
                                    body: "Trading history"
                                    font_color: #x023637
                                    font_size: 17.
                                }
                                history_retention_help := WrapLabel{
                                    width: Fill
                                    body: "Auto-clear old sales and KOTs. Menu, staff, and settings stay. Clear now wipes all trading rows and keeps setup."
                                    font_color: #x074D3E
                                    font_size: 12.
                                }
                                View{
                                    width: Fill height: Fit
                                    flow: Right spacing: 6
                                    View{width: Fill height: Fit flow: Overlay
                                        history_off := SegBtn{text: "Keep"}
                                        history_off_on := SegBtnOn{text: "Keep" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        history_day_close := SegBtn{text: "On day close"}
                                        history_day_close_on := SegBtnOn{text: "On day close" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        history_after_hours := SegBtn{text: "After hours"}
                                        history_after_hours_on := SegBtnOn{text: "After hours" visible: false}
                                    }
                                }
                                history_hours_input := FieldInput{empty_text: "Hours (1–168, default 4)"}
                                history_hours_save := StackPrimaryBtn{text: "Save hours"}
                                history_clear_now := DangerBtn{text: "Clear history now"}
                                h_wipe_data := WrapLabel{
                                    width: Fill
                                    body: "Delete local data"
                                    font_color: #x023637
                                    font_size: 17.
                                }
                                wipe_data_help := WrapLabel{
                                    width: Fill
                                    body: "Owner only — same choices as Windows uninstall. Business records keeps menu/setup (like emergency unlock PIN). Shop setup removes the books. Confirm with Owner PIN. Team emergency PIN is separate."
                                    font_color: #x074D3E
                                    font_size: 12.
                                }
                                View{
                                    width: Fill height: Fit
                                    flow: Down spacing: 6
                                    View{width: Fill height: Fit flow: Overlay
                                        wipe_trading := SegBtn{text: "Business records"}
                                        wipe_trading_on := SegBtnOn{text: "Business records" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        wipe_schema := SegBtn{text: "Shop setup"}
                                        wipe_schema_on := SegBtnOn{text: "Shop setup" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        wipe_ui := SegBtn{text: "Screen preferences"}
                                        wipe_ui_on := SegBtnOn{text: "Screen preferences" visible: false}
                                    }
                                }
                                wipe_owner_pin := FieldInput{
                                    empty_text: "Owner PIN to delete"
                                    is_password: true
                                }
                                wipe_data_btn := DangerBtn{text: "Delete selected"}
                                }
                                settings_panel_diag := View{
                                    width: Fill height: Fit
                                    flow: Down spacing: 12
                                    visible: false
                                    View{
                                        width: Fill height: Fit
                                        flow: Right spacing: 0
                                        align: Align{y: 0.5}
                                        h_diag := WrapLabel{
                                                    width: Fit
                                                    body: "Diagnostics" font_color: #x023637 font_size: 20.}
                                        diag_privacy_help := InfoHelp{
                                            tip: "Logs stay on this device. Secrets (PIN, passwords) are never stored. Sharing a package off the device needs your explicit approval each time — never sold or given to third parties."
                                        }
                                    }
                                    diag_status_label := WrapLabel{
                                        width: Fill
                                        body: ""
                                        font_color: #x023637
                                        font_size: 12.
                                    }
                                    h_diag_level := WrapLabel{body: "Detail level" font_color: #x023637 font_size: 15.}
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
                                    // Start/End: idle = quiet, on = filled primary for the
                                    // live session state (select-in-place faces, not two CTAs).
                                    View{width: Fill height: Fit flow: Overlay
                                        diag_support_start := StackSecondaryBtn{text: "Start 30-min support session"}
                                        diag_support_start_on := StackPrimaryBtn{text: "Start 30-min support session" visible: false}
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        diag_support_stop := StackSecondaryBtn{text: "End support session"}
                                        diag_support_stop_on := StackPrimaryBtn{text: "End support session" visible: false}
                                    }
                                    ActionStack{
                                        diag_export_local := StackSecondaryBtn{text: "Export local copy"}
                                        diag_export_local_cancel := StackLightBtn{text: "Cancel" visible: false}
                                        diag_export_local_progress := WrapLabel{visible: false body: "" font_color: #x023637 font_size: 15.}
                                    }
                                    diag_send_help := InfoHelp{
                                        tip: "Approval is one package, then expires. Temporary — used only to fix or improve the product. Not shared with third parties. Works on Linux, Windows, macOS, and Android."
                                    }
                                    View{width: Fill height: Fit flow: Overlay
                                        diag_approve_send := StackPrimaryBtn{text: "Approve share off this device"}
                                        diag_approve_send_on := StackSecondaryBtn{text: "Share approved" visible: false}
                                    }
                                    ActionStack{
                                        diag_export_send := StackSecondaryBtn{text: "Create share package (needs approval)"}
                                        diag_export_send_cancel := StackLightBtn{text: "Cancel" visible: false}
                                        diag_export_send_progress := WrapLabel{visible: false body: "" font_color: #x023637 font_size: 15.}
                                    }
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
                                    flow: Right spacing: 0
                                    align: Align{y: 0.5}
                                    sidebar_settings_back := QuietFitBtn{text: "Back"}
                                }
                                sidebar_settings_help := InfoHelp{
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
                                // Morning float, then drawer handovers between billers
                                // (BUSINESS_BOOKS.md § Shift and day-end cash).
                                h_opening_float := WrapLabel{body: "Opening cash (float)" font_color: #x023637 font_size: 15.}
                                opening_float_input := FieldInput{empty_text: "Cash put in the drawer this morning"}
                                opening_float_save := StackPrimaryBtn{text: "Save opening cash"}
                                h_shift_close := WrapLabel{body: "Shift handover" font_color: #x023637 font_size: 15.}
                                shift_so_far_label := mod.widgets.ReportText{}
                                shift_counted_input := FieldInput{empty_text: "Cash counted at handover"}
                                shift_close_btn := StackPrimaryBtn{text: "Close shift and print slip"}
                                h_drawer_count := WrapLabel{ width: Fit
                                                    body: "Drawer count (optional)" font_color: #x023637 font_size: 15.}
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
                                    font_color: #x074D3E
                                    font_size: 12.
                                }
                                expense_attach_list := mod.widgets.ReportText{}
                                reports_bank_heading := WrapLabel{body: "Bank / cashbook" font_color: #x023637 font_size: 15.}
                                bank_amount_input := FieldInput{empty_text: "Amount"}
                                bank_note_input := FieldInput{empty_text: "Note (deposit, withdrawal, …)"}
                                ActionStack{
                                    bank_till_to_bank := RowSecondaryBtn{text: "Till → bank"}
                                    bank_bank_to_till := RowSecondaryBtn{text: "Bank → till"}
                                    bank_in := RowSecondaryBtn{text: "Bank in"}
                                    bank_out := RowSecondaryBtn{text: "Bank out"}
                                }
                                reports_copy := NavBtn{text: "Copy report"}
                                h_bank_reconciliation := WrapLabel{body: "Bank reconciliation (tick off against your bank statement)" font_color: #x023637 font_size: 15.}
                                bank_recon_label := mod.widgets.ReportText{}
                                bank_clear_input := FieldInput{empty_text: "Uncleared # above"}
                                ActionStack{
                                    bank_mark_cleared := RowSecondaryBtn{text: "Mark cleared"}
                                    bank_mark_uncleared := RowSecondaryBtn{text: "Unmark"}
                                }
                                h_spreadsheet_csv := WrapLabel{body: "Spreadsheet (CSV)" font_color: #x023637 font_size: 15.}
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
                                ActionStack{
                                    csv_import_items := NavBtn{text: "Import items CSV…"}
                                    csv_import_cancel := StackLightBtn{text: "Cancel" visible: false}
                                    csv_import_progress := WrapLabel{visible: false body: "" font_color: #x023637 font_size: 15.}
                                }
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
                                    font_color: #x074D3E
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
                                    font_color: #x023637
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
                    // Snack is a tap/dismiss surface — keep selectable:false (DESIGN exception).
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
                            draw_bg.color: #x025B60
                            draw_bg.border_radius: 12.
                            snack_label_info := WrapLabel{width: Fill body: "" font_color: #xffffff font_size: 13. selectable: false}
                            snack_dismiss_info := CloseIconBtn{
                                draw_bg +: { mark_color: #xffffff color: #x017B85 color_hover: #x017B85 color_down: #x025B60 color_focus: #x017B85 color_2: #x017B85 color_2_hover: #x017B85 color_2_down: #x025B60 color_2_focus: #x017B85 border_radius: 10. border_size: 0. }
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
                            snack_label_ok := WrapLabel{width: Fill body: "" font_color: #ffffff font_size: 13. selectable: false}
                            snack_dismiss_ok := CloseIconBtn{
                                draw_bg +: { mark_color: #xffffff color: #x1E3D28 color_hover: #x1E3D28 color_down: #x152A1C color_focus: #x1E3D28 color_2: #x1E3D28 color_2_hover: #x1E3D28 color_2_down: #x152A1C color_2_focus: #x1E3D28 border_radius: 10. border_size: 0. }
                            }
                        }
                        snack_face_warn := RoundedView{
                            visible: false
                            width: Fill height: Fit
                            padding: Inset{left: 12. right: 8. top: 10. bottom: 10.}
                            margin: Inset{left: 8. right: 8. bottom: 4.}
                            flow: Right spacing: 8
                            align: Align{y: 0.5}
                            draw_bg.color: #x017B85
                            draw_bg.border_radius: 12.
                            snack_label_warn := WrapLabel{width: Fill body: "" font_color: #ffffff font_size: 13. selectable: false}
                            snack_dismiss_warn := CloseIconBtn{
                                draw_bg +: { mark_color: #xffffff color: #x025B60 color_hover: #x025B60 color_down: #x023637 color_focus: #x025B60 color_2: #x025B60 color_2_hover: #x025B60 color_2_down: #x023637 color_2_focus: #x025B60 border_radius: 10. border_size: 0. }
                            }
                        }
                        snack_face_err := RoundedView{
                            visible: false
                            width: Fill height: Fit
                            padding: Inset{left: 12. right: 8. top: 10. bottom: 10.}
                            margin: Inset{left: 8. right: 8. bottom: 4.}
                            flow: Right spacing: 8
                            align: Align{y: 0.5}
                            draw_bg.color: #x023637
                            draw_bg.border_radius: 12.
                            snack_label_err := WrapLabel{width: Fill body: "" font_color: #ffffff font_size: 13. selectable: false}
                            snack_dismiss_err := CloseIconBtn{
                                draw_bg +: { mark_color: #xffffff color: #x017B85 color_hover: #x017B85 color_down: #x025B60 color_focus: #x017B85 color_2: #x017B85 color_2_hover: #x017B85 color_2_down: #x025B60 color_2_focus: #x017B85 border_radius: 10. border_size: 0. }
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
                        draw_bg.color: #x023637
                        draw_bg.border_radius: 12.
                        undo_snack_label := WrapLabel{
                            width: Fill
                            body: "Sale recorded"
                            font_color: #ffffff
                            font_size: 13.
                            selectable: false
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
                                color: #x025B60
                                color_hover: #x025B60
                                color_down: #x025B60
                                color_focus: #x025B60
                                text_style +: {font_size: 13.}
                            }
                        }
                        undo_snack_dismiss := CloseIconBtn{
                            draw_bg +: {
                                mark_color: #xffffff
                                color: #x017B85
                                color_hover: #x017B85
                                color_down: #x025B60
                                color_focus: #x017B85
                                color_2: #x017B85
                                color_2_hover: #x017B85
                                color_2_down: #x025B60
                                color_2_focus: #x017B85
                                border_radius: 10.
                                border_size: 0.
                            }
                        }
                    }

                    // Phone chrome: all peers in one scroll row (Books · Order · Buy …).
                    // Selected peer = *Active face visible (paint_nav_tabs), not focus.
                    bottom_nav := SolidView{
                        width: Fill height: Fit
                        // Bottom pad 0 so the ScrollX scrollbar sits on the
                        // chrome edge (lift_footer adds safe-area only).
                        // Top pad keeps air above the chips.
                        padding: Inset{left: 4. right: 4. top: 6. bottom: 0.}
                        draw_bg.color: #x025B60
                        bottom_nav_tabs := View{
                            width: Fill height: Fit
                            flow: Overlay
                            bottom_nav_scroll := ScrollXView{
                                width: Fill height: Fit
                                // Dark bar on maroon — stock handle reads as a
                                // cream strip at the footer floor.
                                scroll_bars: mod.widgets.ScrollBars {
                                    show_scroll_x: true show_scroll_y: false
                                    scroll_bar_x: mod.widgets.ScrollBar{
                                        drag_scrolling: true
                                        axis_lock: true
                                        bar_size: 4.
                                        draw_bg +: {
                                            size: 3.
                                            border_radius: 2.
                                            color: #x3D0507
                                            color_hover: #x023637
                                            color_drag: #xfe8357
                                        }
                                    }
                                }
                                // Whitespace lanes around the chips (top,
                                // bottom, ends, gaps): a press there belongs
                                // to the scroll view, a press on a chip is a
                                // tap or a drag — see pointer_nav_drag.
                                // End pad ≥ ChevronBtn (46) so first/last tabs
                                // can scroll clear of overlay ‹ › (2026-11).
                                // Strip bottom ≥ bar_size: ScrollX paints the
                                // bar over this pad (chips stay clear). Parent
                                // bottom_nav.padding.bottom stays 0 — nothing
                                // under the bar.
                                View{
                                    width: Fit height: Fit
                                    padding: Inset{left: 48. right: 48. top: 6. bottom: 8.}
                                    flow: Right spacing: 16
                                    nav_books_slot := View{
                                        width: Fit{min: FitBound.Abs(72.)} height: Fit
                                        flow: Overlay
                                        nav_books := NavChip{text: "Books"}
                                        nav_books_on := NavChipActive{text: "Books" visible: false}
                                        View{
                                            width: Fit height: Fill
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
                                        nav_sell := NavChip{text: "Sell"}
                                        nav_sell_on := NavChipActive{text: "Sell" visible: false}
                                        View{
                                            width: Fit height: Fill
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
                                            width: Fit height: Fill
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
                                        nav_menu := NavChip{text: "Medicines"}
                                        nav_menu_on := NavChipActive{text: "Medicines" visible: false}
                                        View{
                                            width: Fit height: Fill
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
                                        nav_floor := NavChip{text: "Shop"}
                                        nav_floor_on := NavChipActive{text: "Shop" visible: false}
                                        View{
                                            width: Fit height: Fill
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
                                        nav_kitchen := NavChip{text: "Sections"}
                                        nav_kitchen_on := NavChipActive{text: "Sections" visible: false}
                                        View{
                                            width: Fit height: Fill
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
                                            width: Fit height: Fill
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
                                            width: Fit height: Fill
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
                                            width: Fit height: Fill
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
                                            width: Fit height: Fill
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
                                            width: Fit height: Fill
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
                                            width: Fit height: Fill
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
                                            width: Fit height: Fill
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
                                            width: Fit height: Fill
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
                                            width: Fit height: Fill
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
                    // Floor Columns pick list — Modal overlay draw list so the
                    // open menu cannot stretch floor_cols_row / shove tables.
                    // Transparent dim; panel is positioned under the field in Rust.
                    floor_cols_dd_modal := Modal{
                        can_dismiss: true
                        bg_view +: {
                            draw_bg +: {color: #x00000000}
                        }
                        content +: {
                            width: Fill height: Fill
                            flow: Overlay
                            floor_cols_dd_panel := RoundedView{
                                visible: false
                                width: Fixed(176.) height: Fit
                                padding: Inset{left: 6. right: 6. top: 6. bottom: 6.}
                                flow: Down
                                draw_bg.color: #xFFF8F3
                                draw_bg.border_radius: 12.
                                draw_bg.border_size: 1.
                                draw_bg.border_color: #xE3D3C8
                                floor_cols_suggest := mod.widgets.SuggestList{
                                    field: "floor_cols"
                                    width: Fill
                                    height: Fit
                                }
                            }
                        }
                    }
                    // Lock → pick who is unlocking (replaces Prev/Next). Fit
                    // panel so outside press dismisses (DEV bug 7e).
                    lock_staff_modal := Modal{
                        can_dismiss: true
                        content +: {
                            width: Fill height: Fill
                            align: Align{x: 0.5 y: 0.5}
                            padding: Inset{left: 24. right: 24. top: 24. bottom: 24.}
                            lock_staff_sheet := RoundedView{
                                width: Fixed(320.) height: Fit
                                padding: Inset{left: 16. right: 16. top: 16. bottom: 16.}
                                flow: Down spacing: 8
                                draw_bg.color: #xFFF8F3
                                draw_bg.border_radius: 14.
                                draw_bg.border_size: 1.
                                draw_bg.border_color: #xE3D3C8
                                lock_staff_sheet_title := WrapLabel{
                                    body: "Who is unlocking?"
                                    font_color: #x023637
                                    font_size: 15.
                                }
                                ActionStack{
                                    lock_s0 := NavBtn{text: "" visible: false}
                                    lock_s1 := NavBtn{text: "" visible: false}
                                    lock_s2 := NavBtn{text: "" visible: false}
                                    lock_s3 := NavBtn{text: "" visible: false}
                                    lock_s4 := NavBtn{text: "" visible: false}
                                    lock_s5 := NavBtn{text: "" visible: false}
                                    lock_s6 := NavBtn{text: "" visible: false}
                                    lock_s7 := NavBtn{text: "" visible: false}
                                    lock_s8 := NavBtn{text: "" visible: false}
                                    lock_s9 := NavBtn{text: "" visible: false}
                                    lock_s10 := NavBtn{text: "" visible: false}
                                    lock_s11 := NavBtn{text: "" visible: false}
                                }
                                lock_staff_sheet_done := StackSecondaryBtn{text: "Done"}
                            }
                        }
                    }
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
                                draw_bg.color: #x023637E6
                                draw_bg.border_radius: 12.
                                draw_bg.border_color: #xA2CFCF
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
                            // The visible sheet: a press outside it closes the
                            // modal (content fills the window, so Makepad's own
                            // outside test never fires — see `sheet_press`).
                            upi_qr_sheet := RoundedView{
                                width: Fill{max: 420.} height: Fit
                                flow: Down spacing: 12
                                padding: Inset{left: 20. right: 20. top: 18. bottom: 18.}
                                draw_bg.color: #xFBFCFC
                                draw_bg.border_radius: 18.
                                View{
                                    width: Fill height: Fit
                                    flow: Right spacing: 0
                                    align: Align{x: 0.0 y: 0.5}
                                    upi_qr_title := WrapLabel{
                                                    width: Fit
                                                    body: "UPI" font_color: #x023637 font_size: 20.}
                                    upi_qr_title_help := InfoHelp{
                                        tip: "Guest scans and pays. Tap Take payment after they pay."
                                    }
                                }
                                upi_qr_amount := WrapLabel{body: " " font_color: #x025B60 font_size: 18.}
                                upi_qr_image := Image{
                                    width: 240 height: 240
                                    fit: ImageFit.Smallest
                                }
                                upi_qr_help := WrapLabel{
                                    width: Fill
                                    body: "Guest scans and pays. Tap Take payment after they pay."
                                    font_color: #x074D3E
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
                                // Header (Done) is tested before the search and the
                                // dish grid. The default last-first order let those
                                // take the press, so Done never closed the window.
                                event_order: Down
                                padding: Inset{left: 16. right: 16. top: 14. bottom: 16.}
                                draw_bg.color: #xFBFCFC
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
                                            draw_text.color: #x023637
                                            draw_text.text_style.font_size: 18.
                                        }
                                        // How the order stands, so nobody leaves the window to check.
                                        menu_modal_summary := Label{
                                            width: Fill
                                            text: " "
                                            draw_text.color: #x074D3E
                                            draw_text.text_style.font_size: 13.
                                        }
                                    }
                                    // Not RowSecondaryBtn: that face wraps, and its hit box was not
                                    // the pill on screen, so Done never received the press.
                                    menu_pick_done := Button{
                                        text: "Done"
                                        width: 140
                                        height: 44
                                        grab_key_focus: false
                                        align: Align{x: 0.5 y: 0.5}
                                        padding: Inset{left: 14. right: 14. top: 10. bottom: 10.}
                                        draw_bg +: {
                                            color: #xffffff
                                            color_hover: #xFBEDE4
                                            color_down: #xEFD2BC
                                            color_focus: #xffffff
                                            border_color: #xD8BBA6
                                            border_radius: 8.
                                            border_size: 1.
                                        }
                                        draw_text +: {
                                            color: #x025B60
                                            color_hover: #x025B60
                                            color_down: #x025B60
                                            color_focus: #x025B60
                                            ink_centered: true
                                            text_style +: {font_size: 14.}
                                        }
                                    }
                                }
                                // Name, alias or short code (aliases double as short codes,
                                // e.g. "101"); typing looks through every category.
                                menu_search_input := FieldInput{
                                    empty_text: "Search medicine, salt, short code or barcode"
                                    draw_text +: {text_style +: {font_size: 15.}}
                                }
                                // Categories as one scrollable strip: All, Frequent,
                                // Added Item(s), then the shop's own, in its order.
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
                                                cat_added := ChipBtn{text: "Added Item(s)"}
                                                cat_added_on := ChipBtnActive{text: "Added Item(s)" visible: false}
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
                                        font_color: #x074D3E
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
                                    draw_text.color: #x074D3E
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
