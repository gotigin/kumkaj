// kumkaj.com — the whole page, one Makepad 2 `script_mod!` (included from
// main.rs so `App` is in scope). Story: docs/WEBSITES.md §9. Palette: §4.
// Claims: only Ship / Have / Done rows of MOKLANJO.md / KHAYNJO.md.
// Desk sizes are the DSL defaults; App::apply_responsive shrinks type,
// stacks rows and hides detail on phones (every chapter fits one screen).
script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    // ---- Tokens ------------------------------------------------------
    let Navy = #x041534
    let NavyDeep = #x020B1F
    let Panel = #x0B1F4A
    let PanelHi = #x12295C
    let Rule = #x24407A
    let Ink = #xF9F9FC
    let Muted = #xC2CADC
    let Faint = #x93A0BE
    let Terracotta = #xA23E18
    let TerracottaHover = #x8C3414
    let TerracottaDown = #x6E2A10
    // Terracotta lifted for text on navy (6.9:1); the fill stays #A23E18.
    let Clay = #xE8875C
    let Good = #x6CCB94
    let Bad = #xF2846F
    let MokNavy = #x0A3A8C
    let MokOrange = #xDF5E16
    let KhaRed = #x9A0E10
    let KhaGold = #xF69D2C

    // ---- Type: Fraunces display, IBM Plex text, Noto for Indic/Arabic --
    let DisplayFont = FontFamily{
        latin := FontMember{res: crate_resource("self:resources/fonts/Fraunces-Variable.ttf") asc: 0.0 desc: 0.0 weight: 620.0}
        ui := FontMember{res: crate_resource("self:resources/fonts/IBMPlexSans-SemiBold.ttf") asc: -0.1 desc: 0.0}
    }
    let StrongFont = FontFamily{
        latin := FontMember{res: crate_resource("self:resources/fonts/IBMPlexSans-SemiBold.ttf") asc: -0.1 desc: 0.0}
        devanagari := FontMember{res: crate_resource("self:resources/fonts/NotoSansDevanagari-Regular.ttf") asc: 0.0 desc: 0.0}
        gujarati := FontMember{res: crate_resource("self:resources/fonts/NotoSansGujarati-Regular.ttf") asc: 0.0 desc: 0.0}
        tamil := FontMember{res: crate_resource("self:resources/fonts/NotoSansTamil-Regular.ttf") asc: 0.0 desc: 0.0}
        bengali := FontMember{res: crate_resource("self:resources/fonts/NotoSansBengali-Regular.ttf") asc: 0.0 desc: 0.0}
        arabic := FontMember{res: crate_resource("self:resources/fonts/NotoSansArabic-Regular.ttf") asc: 0.0 desc: 0.0}
    }
    let TextFont = FontFamily{
        latin := FontMember{res: crate_resource("self:resources/fonts/IBMPlexSans-Text.ttf") asc: -0.1 desc: 0.0}
        devanagari := FontMember{res: crate_resource("self:resources/fonts/NotoSansDevanagari-Regular.ttf") asc: 0.0 desc: 0.0}
        gujarati := FontMember{res: crate_resource("self:resources/fonts/NotoSansGujarati-Regular.ttf") asc: 0.0 desc: 0.0}
        tamil := FontMember{res: crate_resource("self:resources/fonts/NotoSansTamil-Regular.ttf") asc: 0.0 desc: 0.0}
        bengali := FontMember{res: crate_resource("self:resources/fonts/NotoSansBengali-Regular.ttf") asc: 0.0 desc: 0.0}
        arabic := FontMember{res: crate_resource("self:resources/fonts/NotoSansArabic-Regular.ttf") asc: 0.0 desc: 0.0}
    }

    // The vendored SDF text shader adds `sdf_luma_bias` coverage to the whole
    // glyph quad for light ink, which paints faint boxes behind white type on
    // navy. This page is light-on-dark everywhere, so drop the bias here (the
    // apps' dark-on-light faces never hit it; vendor shader untouched).
    mod.widgets.Label.draw_text.sdf_luma_bias = 0.0
    mod.widgets.Button.draw_text.sdf_luma_bias = 0.0

    let Eyebrow = Markdown{
        selectable: true
        height: Fit
        paragraph_spacing: 0.
        pre_code_spacing: 0.
        draw_block +: { selection_color: #x2E5BADFF }
        width: Fill
        padding: 0.
        draw_text +: {
            color: Clay
            text_style: theme.font_bold{font_family: StrongFont font_size: 12.5 line_spacing: 1.3}
        }
    }
    let Headline = Markdown{
        selectable: true
        height: Fit
        paragraph_spacing: 0.
        pre_code_spacing: 0.
        draw_block +: { selection_color: #x2E5BADFF }
        width: Fill
        padding: 0.
        draw_text +: {
            color: Ink
            text_style: theme.font_bold{font_family: DisplayFont font_size: 52. line_spacing: 0.94}
        }
    }
    let Lede = Markdown{
        selectable: true
        height: Fit
        paragraph_spacing: 0.
        pre_code_spacing: 0.
        draw_block +: { selection_color: #x2E5BADFF }
        width: Fill
        padding: 0.
        draw_text +: {
            color: Muted
            text_style: theme.font_regular{font_family: TextFont font_size: 20. line_spacing: 1.45}
        }
    }
    let Body = Markdown{
        selectable: true
        height: Fit
        paragraph_spacing: 0.
        pre_code_spacing: 0.
        draw_block +: { selection_color: #x2E5BADFF }
        width: Fill
        padding: 0.
        draw_text +: {
            color: Muted
            text_style: theme.font_regular{font_family: TextFont font_size: 16. line_spacing: 1.45}
        }
    }
    let Strong = Markdown{
        selectable: true
        height: Fit
        paragraph_spacing: 0.
        pre_code_spacing: 0.
        draw_block +: { selection_color: #x2E5BADFF }
        width: Fill
        padding: 0.
        draw_text +: {
            color: Ink
            text_style: theme.font_bold{font_family: StrongFont font_size: 16. line_spacing: 1.3}
        }
    }
    let Small = Markdown{
        selectable: true
        height: Fit
        paragraph_spacing: 0.
        pre_code_spacing: 0.
        draw_block +: { selection_color: #x2E5BADFF }
        width: Fill
        padding: 0.
        draw_text +: {
            color: Faint
            text_style: theme.font_regular{font_family: TextFont font_size: 13. line_spacing: 1.4}
        }
    }
    let CardName = Markdown{
        selectable: true
        height: Fit
        paragraph_spacing: 0.
        pre_code_spacing: 0.
        draw_block +: { selection_color: #x2E5BADFF }
        width: Fill
        padding: 0.
        draw_text +: {
            color: Ink
            text_style: theme.font_bold{font_family: DisplayFont font_size: 30. line_spacing: 1.0}
        }
    }

    // ---- Buttons (one face each; white ink only on terracotta) --------
    let PrimaryBtn = Button{
        width: Fit height: Fit
        padding: Inset{left: 22. right: 22. top: 14. bottom: 14.}
        align: Align{x: 0.5 y: 0.5}
        grab_key_focus: false
        reset_hover_on_click: true
        label_walk: Walk{width: Fit height: Fit margin: Inset{left: 2. right: 2.}}
        icon_walk: Walk{width: 0. height: 0.}
        draw_bg +: {
            color: Terracotta
            color_hover: TerracottaHover
            color_down: TerracottaDown
            color_focus: Terracotta
            color_2: Terracotta
            color_2_hover: TerracottaHover
            color_2_down: TerracottaDown
            color_2_focus: Terracotta
            border_color: Terracotta
            border_radius: 10.
            border_size: 0.
        }
        draw_text +: {
            color: #xFFFFFF
            color_hover: #xFFFFFF
            color_down: #xFFFFFF
            color_focus: #xFFFFFF
            text_style: theme.font_bold{font_family: StrongFont font_size: 15.}
        }
    }
    let GhostBtn = Button{
        width: Fit height: Fit
        padding: Inset{left: 20. right: 20. top: 13. bottom: 13.}
        align: Align{x: 0.5 y: 0.5}
        grab_key_focus: false
        reset_hover_on_click: true
        label_walk: Walk{width: Fit height: Fit margin: Inset{left: 2. right: 2.}}
        icon_walk: Walk{width: 0. height: 0.}
        draw_bg +: {
            color: #x00000000
            color_hover: PanelHi
            color_down: Panel
            color_focus: #x00000000
            color_2: #x00000000
            color_2_hover: PanelHi
            color_2_down: Panel
            color_2_focus: #x00000000
            border_color: #x4A64A0
            border_color_hover: #x6E86C0
            border_color_down: #x6E86C0
            border_color_focus: #x4A64A0
            border_radius: 10.
            border_size: 1.5
        }
        draw_text +: {
            color: Ink
            color_hover: Ink
            color_down: Ink
            color_focus: Ink
            text_style: theme.font_bold{font_family: StrongFont font_size: 15.}
        }
    }
    // Invisible hit face laid over a painted shape (chapter dots).
    let HitBtn = Button{
        width: Fill height: Fill
        padding: 0.
        text: ""
        grab_key_focus: false
        reset_hover_on_click: true
        label_walk: Walk{width: 0. height: 0.}
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
            border_size: 0.
        }
        draw_text +: {
            color: #x00000000
            color_hover: #x00000000
            color_down: #x00000000
            color_focus: #x00000000
        }
    }

    // ---- Chapter ground: navy fall-off, a warm glow, faint ledger rules --
    let Chapter = View{
        width: Fill height: Fill
        flow: Down
        spacing: 22
        align: Align{x: 0.0 y: 0.5}
        padding: Inset{left: 64. right: 64. top: 36. bottom: 44.}
        show_bg: true
        draw_bg +: {
            top: instance(#x041534)
            bottom: instance(#x01081A)
            glow: instance(#xA23E18)
            glow_x: instance(0.92)
            glow_y: instance(0.95)
            pixel: fn() {
                let base = mix(self.top, self.bottom, self.pos.y)
                let aspect = self.rect_size.x / max(1.0, self.rect_size.y)
                let d = length(vec2((self.pos.x - self.glow_x) * aspect, self.pos.y - self.glow_y))
                let g = 0.30 * (1.0 - smoothstep(0.0, 0.95, d))
                let y = self.pos.y * self.rect_size.y
                let m = fract(y / 44.0) * 44.0
                let rule = (1.0 - smoothstep(0.0, 1.1, min(m, 44.0 - m))) * 0.035
                return vec4(mix(base.xyz, self.glow.xyz, g) + vec3(rule, rule, rule * 1.4), 1.0)
            }
        }
    }

    let Card = RoundedView{
        width: Fill height: Fit
        flow: Down
        spacing: 10
        padding: Inset{left: 22. right: 22. top: 20. bottom: 22.}
        draw_bg +: {
            color: #x0B1F4AE6
            border_color: Rule
            border_size: 1.
            border_radius: 14.
        }
    }

    // One row of the counter-status prop: dot · what · state.
    let StatusRow = View{
        width: Fill height: Fit
        flow: Right
        spacing: 12
        align: Align{y: 0.5}
        dot := RoundedView{width: 10 height: 10 draw_bg +: {color: Good border_radius: 5.}}
        what := Markdown{
            selectable: true
            height: Fit
            paragraph_spacing: 0.
            pre_code_spacing: 0.
            draw_block +: { selection_color: #x2E5BADFF }

            width: Fill
            padding: 0.
            draw_text +: {color: Ink text_style: theme.font_regular{font_family: TextFont font_size: 15.}}
            body: "Row"
        
        }
        state := Markdown{
            selectable: true
            height: Fit
            paragraph_spacing: 0.
            pre_code_spacing: 0.
            draw_block +: { selection_color: #x2E5BADFF }

            width: Fit
            padding: 0.
            draw_text +: {color: Good text_style: theme.font_bold{font_family: StrongFont font_size: 14.}}
            body: "Working"
        
        }
    }

    let Bullet = View{
        width: Fill height: Fit
        flow: Right
        spacing: 10
        align: Align{y: 0.0}
        tick := Markdown{
            selectable: true
            height: Fit
            paragraph_spacing: 0.
            pre_code_spacing: 0.
            draw_block +: { selection_color: #x2E5BADFF }

            width: Fit
            padding: Inset{top: 1. bottom: 0. left: 0. right: 0.}
            draw_text +: {color: Clay text_style: theme.font_bold{font_family: StrongFont font_size: 14.}}
            body: "—"
        
        }
        line := Markdown{
            selectable: true
            height: Fit
            paragraph_spacing: 0.
            pre_code_spacing: 0.
            draw_block +: { selection_color: #x2E5BADFF }

            width: Fill
            padding: 0.
            draw_text +: {color: Muted text_style: theme.font_regular{font_family: TextFont font_size: 15. line_spacing: 1.35}}
            body: "Bullet"
        
        }
    }

    // A "Food ready" chip: language name over the word, in its script.
    let WordChip = RoundedView{
        width: Fit height: Fit
        flow: Down
        spacing: 4
        padding: Inset{left: 18. right: 18. top: 12. bottom: 14.}
        draw_bg +: {color: Panel border_color: Rule border_size: 1. border_radius: 12.}
        lang := Markdown{
            selectable: true
            height: Fit
            paragraph_spacing: 0.
            pre_code_spacing: 0.
            draw_block +: { selection_color: #x2E5BADFF }

            width: Fit
            padding: 0.
            draw_text +: {color: Faint text_style: theme.font_regular{font_family: TextFont font_size: 13.}}
            body: "Language"
        
        }
        word := Markdown{
            selectable: true
            height: Fit
            paragraph_spacing: 0.
            pre_code_spacing: 0.
            draw_block +: { selection_color: #x2E5BADFF }
            width: Fit
            padding: 0.
            draw_text +: {color: Ink text_style: theme.font_bold{font_family: StrongFont font_size: 24.}}
            body: "Word"
        }
    }

    // A node in the Gotigin → Kumkaj → products map.
    let MapNode = RoundedView{
        width: Fill height: Fit
        flow: Down
        spacing: 4
        // Tight sides: four nodes and two arrows share 900 px at 1024 wide.
        padding: Inset{left: 14. right: 12. top: 14. bottom: 16.}
        draw_bg +: {color: Panel border_color: Rule border_size: 1. border_radius: 12.}
        role := Markdown{
            selectable: true
            height: Fit
            paragraph_spacing: 0.
            pre_code_spacing: 0.
            draw_block +: { selection_color: #x2E5BADFF }

            width: Fill
            padding: 0.
            draw_text +: {color: Faint text_style: theme.font_bold{font_family: StrongFont font_size: 12.}}
            body: "ROLE"
        
        }
        name := Markdown{
            selectable: true
            height: Fit
            paragraph_spacing: 0.
            pre_code_spacing: 0.
            draw_block +: { selection_color: #x2E5BADFF }
            width: Fill
            padding: 0.
            draw_text +: {color: Ink text_style: theme.font_bold{font_family: DisplayFont font_size: 24.}}
            body: "Name"
        }
    }
    let MapArrow = Markdown{
        selectable: true
        height: Fit
        paragraph_spacing: 0.
        pre_code_spacing: 0.
        draw_block +: { selection_color: #x2E5BADFF }
        width: Fit
        padding: Inset{left: 2. right: 2. top: 0. bottom: 0.}
        draw_text +: {color: Clay text_style: theme.font_bold{font_family: StrongFont font_size: 22.}}
        body: "→"
    }

    // Chapter dot in the top bar: idle dot, active pill, invisible hit.
    let Dot = View{
        width: 20 height: 24
        flow: Overlay
        align: Align{x: 0.5 y: 0.5}
        idle := RoundedView{width: 7 height: 7 draw_bg +: {color: #x3A4F80 border_radius: 3.5}}
        on := RoundedView{visible: false width: 16 height: 7 draw_bg +: {color: Clay border_radius: 3.5}}
        hit := HitBtn{}
    }

    startup() do #(App::script_component(vm)){
        ui: Root{
            main_window := Window{
                window.title: "Kumkaj — the counter keeps working"
                window.inner_size: vec2(1280, 800)
                pass.clear_color: Navy
                body +: {
                    width: Fill
                    height: Fill
                    flow: Down

                    top_bar := View{
                        width: Fill height: 58
                        flow: Right
                        spacing: 14
                        align: Align{y: 0.5}
                        padding: Inset{left: 24. right: 20. top: 0. bottom: 0.}
                        show_bg: true
                        draw_bg +: {color: NavyDeep}
                        View{
                            width: Fit height: Fit
                            flow: Right
                            spacing: 10
                            align: Align{y: 0.6}
                            Markdown{
            selectable: true
            height: Fit
            paragraph_spacing: 0.
            pre_code_spacing: 0.
            draw_block +: { selection_color: #x2E5BADFF }

                                padding: 0.
                                body: "Kumkaj"
                                draw_text +: {color: Ink text_style: theme.font_bold{font_family: DisplayFont font_size: 21.}}
                            
        }
                            byline := Markdown{
            selectable: true
            height: Fit
            paragraph_spacing: 0.
            pre_code_spacing: 0.
            draw_block +: { selection_color: #x2E5BADFF }

                                padding: 0.
                                body: "by Gotigin"
                                draw_text +: {color: Faint text_style: theme.font_regular{font_family: TextFont font_size: 12.5}}
                            
        }
                        }
                        View{width: Fill height: 1}
                        chapter_name := Markdown{
            selectable: true
            height: Fit
            paragraph_spacing: 0.
            pre_code_spacing: 0.
            draw_block +: { selection_color: #x2E5BADFF }

                            padding: 0.
                            body: "The promise"
                            draw_text +: {color: Muted text_style: theme.font_regular{font_family: TextFont font_size: 13.5}}
                        
        }
                        dots := View{
                            width: Fit height: Fit
                            flow: Right
                            spacing: 0
                            align: Align{y: 0.5}
                            d0 := Dot{}
                            d1 := Dot{}
                            d2 := Dot{}
                            d3 := Dot{}
                            d4 := Dot{}
                            d5 := Dot{}
                            d6 := Dot{}
                        }
                        counter := Markdown{
            selectable: true
            height: Fit
            paragraph_spacing: 0.
            pre_code_spacing: 0.
            draw_block +: { selection_color: #x2E5BADFF }

                            padding: 0.
                            body: "1 / 7"
                            draw_text +: {color: Faint text_style: theme.font_bold{font_family: StrongFont font_size: 12.5}}
                        
        }
                    }

                    slides := SlidesView{
                        width: Fill
                        height: Fill
                        anim_speed: 0.86
                        axis: Vertical

                        // 01 — the promise
                        hero := Chapter{
                            hero_row := View{
                                width: Fill height: Fit
                                flow: Right
                                spacing: 48
                                align: Align{y: 0.5}
                                View{
                                    width: Fill height: Fit
                                    flow: Down
                                    spacing: 22
                                    Eyebrow{body: "KUMKAJ · SOFTWARE FOR THE SHOP COUNTER"}
                                    Headline{body: "The internet went down.\nThe counter didn't."}
                                    Lede{body: "Kumkaj makes business software that runs on your own devices — courier, restaurant, grocery and pharmacy counters, each built for its trade."}
                                    hero_ctas := View{
                                        width: Fill height: Fit
                                        flow: Right
                                        spacing: 12
                                        margin: Inset{top: 6. bottom: 0. left: 0. right: 0.}
                                        to_trade := PrimaryBtn{text: "Choose your trade  ↓"}
                                        to_offline := GhostBtn{text: "Watch it keep working"}
                                    }
                                }
                                status_card := Card{
                                    width: 360
                                    spacing: 16
                                    Small{body: "THIS COUNTER, RIGHT NOW"}
                                    net_row := StatusRow{
                                        dot.draw_bg.color: Bad
                                        what.body: "Internet"
                                        state.body: "Down"
                                        state.draw_text.color: Bad
                                    }
                                    StatusRow{what.body: "Taking orders" state.body: "Yes"}
                                    StatusRow{what.body: "Bills and dockets" state.body: "Saving"}
                                    StatusRow{what.body: "Day book" state.body: "Adding up"}
                                    Small{body: "Nothing on this counter waits for the cloud."}
                                }
                            }
                            hint := Small{body: "Scroll, swipe up, or press ↓ to read on."}
                        }

                        // 02 — choose your trade
                        trade := Chapter{
                            Eyebrow{body: "02 · BUILT FOR YOUR TRADE"}
                            Headline{
                                draw_text.text_style.font_size: 42.
                                body: "A desk for each trade.\nNot one app with a switch."
                            }
                            trade_lede := Lede{body: "Not one app with a trade switch. Each product is designed around one kind of counter, in that trade's own words."}
                            cards := View{
                                width: Fill height: Fit
                                flow: Down
                                spacing: 20
                                View{
                                    width: Fill height: Fit
                                    flow: Right
                                    spacing: 20
                                    mok := Card{
                                        draw_bg.border_color: #x2A4F9A
                                        RoundedView{width: 44 height: 5 draw_bg +: {color: MokOrange border_radius: 2.5}}
                                        mok_kicker := Small{body: "COURIER OFFICES · FRANCHISE COUNTERS · OWN RIDERS"}
                                        CardName{body: "Moklanjo"}
                                        Strong{body: "Book → label → hand over → track → settle."}
                                        mok_points := View{
                                            width: Fill height: Fit
                                            flow: Down
                                            spacing: 6
                                            Bullet{line.body: "Next booking in one tap"}
                                            Bullet{line.body: "Actual or volumetric weight, rounded your way"}
                                            Bullet{line.body: "Rider cash on delivery: expected against handed in"}
                                        }
                                        to_moklanjo := GhostBtn{text: "Visit moklanjo.com  →"}
                                    }
                                    kha := Card{
                                        draw_bg.border_color: #x7A2A2A
                                        RoundedView{width: 44 height: 5 draw_bg +: {color: KhaGold border_radius: 2.5}}
                                        kha_kicker := Small{body: "RESTAURANTS · CAFÉS · QUICK SERVICE · BAKERIES"}
                                        CardName{body: "Khaynjo"}
                                        Strong{body: "Table → kitchen → bill → day close."}
                                        kha_points := View{
                                            width: Fill height: Fit
                                            flow: Down
                                            spacing: 6
                                            Bullet{line.body: "Every table's minutes and bill at a glance"}
                                            Bullet{line.body: "One KOT per kitchen station — never prices"}
                                            Bullet{line.body: "Cash, UPI, part payment, Due and tip"}
                                        }
                                        to_khaynjo := GhostBtn{text: "Visit khaynjo.com  →"}
                                    }
                                }
                                View{
                                    width: Fill height: Fit
                                    flow: Right
                                    spacing: 20
                                    kir := Card{
                                        draw_bg.border_color: #xCD380E
                                        RoundedView{width: 44 height: 5 draw_bg +: {color: #xF96303 border_radius: 2.5}}
                                        Small{body: "KIRANA · GROCERY · NEIGHBOURHOOD SHOPS"}
                                        CardName{body: "Kirano"}
                                        Strong{body: "Sell → khata → stock → day book."}
                                        View{
                                            width: Fill height: Fit
                                            flow: Down
                                            spacing: 6
                                            Bullet{line.body: "Fast counter sell with short codes"}
                                            Bullet{line.body: "Customer credit and supplier dues"}
                                            Bullet{line.body: "Buy into stock — offline forever"}
                                        }
                                        to_kirano := GhostBtn{text: "Visit kirano.shop  →"}
                                    }
                                    tab := Card{
                                        draw_bg.border_color: #x023637
                                        RoundedView{width: 44 height: 5 draw_bg +: {color: #x37BE28 border_radius: 2.5}}
                                        Small{body: "CHEMISTS · PHARMACY COUNTERS"}
                                        CardName{body: "Tabiyat"}
                                        Strong{body: "Medicine → batch → bill → day close."}
                                        View{
                                            width: Fill height: Fit
                                            flow: Down
                                            spacing: 6
                                            Bullet{line.body: "Barcode and salt / composition search"}
                                            Bullet{line.body: "Batch and expiry alerts on purchases"}
                                            Bullet{line.body: "Owner Schedule H / Rx notes — not fake compliance"}
                                        }
                                        to_tabiyat := GhostBtn{text: "Visit tabiyat.shop  →"}
                                    }
                                }
                            }
                        }

                        // 03 — your books stay with you
                        books := Chapter{
                            Eyebrow{body: "03 · YOUR BOOKS STAY WITH YOU"}
                            Headline{body: "Your books live in your shop."}
                            books_lede := Lede{body: "Sales, parties and the day book are written on the device at your counter — not on a server you rent."}
                            tiles := View{
                                width: Fill height: Fit
                                flow: Right
                                spacing: 18
                                Card{
                                    Strong{body: "On your device"}
                                    Body{body: "Every sale, payment and docket is saved on the computer or tablet you own."}
                                }
                                Card{
                                    Strong{body: "No account with us"}
                                    Body{body: "Install it and start. No sign-up, no card on file."}
                                }
                                Card{
                                    Strong{body: "Copies you keep"}
                                    Body{body: "Save a backup copy whenever you like. It is a file, and it is yours."}
                                }
                            }
                            // Shown only after the owner's real-hardware LAN pilot passes (main.rs LAN_PILOT_PASSED).
                            books_lan := Body{visible: false body: "Extra counters and tablets join over your own Wi‑Fi — with no internet in between."}
                        }

                        // 04 — the toy: cut the internet
                        offline := Chapter{
                            Eyebrow{body: "04 · TRY IT"}
                            Headline{body: "Pull the plug on the internet."}
                            offline_lede := Lede{body: "Kumkaj apps don't phone home to work. Cut the internet and watch what changes at the counter."}
                            toy := Card{
                                width: Fill
                                spacing: 14
                                toy_head := View{
                                    width: Fill height: Fit
                                    flow: Right
                                    spacing: 14
                                    align: Align{y: 0.5}
                                    toggle := PrimaryBtn{text: "Cut the internet"}
                                    net_state := Strong{width: Fill body: "Internet: on"}
                                }
                                StatusRow{what.body: "Book a parcel or take an order" state.body: "Working"}
                                StatusRow{what.body: "Save the bill or docket" state.body: "Working"}
                                StatusRow{what.body: "Day book totals" state.body: "Working"}
                                StatusRow{what.body: "Find an old sale" state.body: "Working"}
                                verdict := Strong{body: "The internet is on — and the counter isn't using it."}
                            }
                            offline_note := Small{body: "Things that leave the shop, like a WhatsApp receipt, still need the internet to arrive."}
                        }

                        // 05 — price
                        free := Chapter{
                            Eyebrow{body: "05 · PRICE"}
                            Headline{body: "Free forever on your devices."}
                            Lede{body: "No charge per seat, user, table, rider, parcel or local device. Not a trial."}
                            prices := Card{
                                width: Fill
                                spacing: 16
                                View{
                                    width: Fill height: Fit
                                    flow: Right
                                    spacing: 16
                                    align: Align{y: 0.5}
                                    Strong{width: Fill body: "On your own devices"}
                                    Markdown{
            selectable: true
            height: Fit
            paragraph_spacing: 0.
            pre_code_spacing: 0.
            draw_block +: { selection_color: #x2E5BADFF }

                                        padding: 0.
                                        body: "₹0, forever"
                                        draw_text +: {color: Good text_style: theme.font_bold{font_family: DisplayFont font_size: 26.}}
                                    
        }
                                }
                                RoundedView{width: Fill height: 1 draw_bg +: {color: Rule border_radius: 0.}}
                                View{
                                    width: Fill height: Fit
                                    flow: Right
                                    spacing: 16
                                    align: Align{y: 0.5}
                                    View{
                                        width: Fill height: Fit
                                        flow: Down
                                        spacing: 4
                                        Strong{body: "Kumkaj Cloud"}
                                        Body{body: "Pay only for what you use — the same rates for everyone."}
                                    }
                                    RoundedView{
                                        width: Fit height: Fit
                                        padding: Inset{left: 12. right: 12. top: 6. bottom: 7.}
                                        draw_bg +: {color: #x00000000 border_color: Clay border_size: 1.2 border_radius: 12.}
                                        Markdown{
            selectable: true
            height: Fit
            paragraph_spacing: 0.
            pre_code_spacing: 0.
            draw_block +: { selection_color: #x2E5BADFF }

                                            padding: 0.
                                            body: "Planned"
                                            draw_text +: {color: Clay text_style: theme.font_bold{font_family: StrongFont font_size: 13.}}
                                        
        }
                                    }
                                }
                            }
                            Small{body: "Cloud is planned and optional. The counter works without it."}
                        }

                        // 06 — languages (real Khaynjo floor words, locale_chrome tile_food_ready)
                        languages := Chapter{
                            Eyebrow{body: "06 · LANGUAGES"}
                            Headline{body: "Speaks your staff's language."}
                            Lede{body: "29 languages on screen, with Arabic and Urdu right to left. One floor word — “Food ready” — as Khaynjo shows it:"}
                            words := View{
                                width: Fill height: Fit
                                flow: Right{wrap: true}
                                spacing: 12
                                WordChip{lang.body: "हिन्दी" word.body: "खाना तैयार"}
                                WordChip{lang.body: "ગુજરાતી" word.body: "જમવાનું તૈયાર"}
                                WordChip{lang.body: "தமிழ்" word.body: "உணவு தயார்"}
                                WordChip{lang.body: "বাংলা" word.body: "খাবার তৈরি"}
                                WordChip{lang.body: "اردو" word.body: "کھانا تیار"}
                                WordChip{lang.body: "English" word.body: "Food ready"}
                            }
                            Small{body: "Same shaping engine as the apps, so joined letters and right-to-left lines come out right."}
                        }

                        // 07 — who makes it, and the choice again
                        gotigin := Chapter{
                            Eyebrow{body: "07 · WHO MAKES IT"}
                            Headline{body: "Made by Gotigin."}
                            gotigin_lede := Lede{body: "Gotigin is the company. Kumkaj is its family of business software. Moklanjo and Khaynjo are the products you install."}
                            map := View{
                                width: Fill height: Fit
                                flow: Right
                                spacing: 10
                                align: Align{y: 0.5}
                                MapNode{role.body: "COMPANY" name.body: "Gotigin"}
                                arrow_1 := MapArrow{}
                                MapNode{role.body: "THE BRAND" name.body: "Kumkaj"}
                                arrow_2 := MapArrow{}
                                MapNode{
                                    draw_bg.border_color: #x2A4F9A
                                    role.body: "FOR COURIER OFFICES"
                                    name.body: "Moklanjo"
                                }
                                MapNode{
                                    draw_bg.border_color: #x7A2A2A
                                    role.body: "FOR RESTAURANTS"
                                    name.body: "Khaynjo"
                                }
                            }
                            close_ctas := View{
                                width: Fill height: Fit
                                flow: Right
                                spacing: 12
                                close_moklanjo := PrimaryBtn{text: "Moklanjo — courier desk  →"}
                                close_khaynjo := PrimaryBtn{text: "Khaynjo — restaurant system  →"}
                                close_kirano := PrimaryBtn{text: "Kirano — grocery counter  →"}
                                close_tabiyat := PrimaryBtn{text: "Tabiyat — pharmacy counter  →"}
                            }
                            Small{body: "Moklanjo and Khaynjo are Kumkaj products by Gotigin."}
                        }
                    }
                }
            }
        }
    }
}
