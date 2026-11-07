// moklanjo.com — one Makepad 2 `script_mod!` (included from main.rs).
// Story: apps/moklanjo/docs/MOKLANJO_WEBSITE.md §3 ("One parcel, from
// counter to settled"). Palette: docs/DESIGN.md "Color tokens (live)".
// Started as a copy of sites/kumkaj and diverged: light desk ground,
// navy chrome, orange speed streaks, a tracking-bead rail for chapters.
// Every feature sentence is a Ship row in MOKLANJO.md (claim ids in
// comments); multi-device sentences wait for the LAN pilot.
script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    // ---- Tokens (Moklanjo icon / DESIGN.md) --------------------------
    let Surface = #xD6E0EC
    let SurfaceHi = #xE7EEF6
    let Well = #xFFFFFF
    let WellSoft = #xEEF3F8
    let WellLine = #xC3D0E2
    let Ink = #x0B1220
    let Muted = #x3A4554
    let Hint = #x56575D
    let NavyBlack = #x02030B
    let Navy = #x072058
    let NavyHi = #x0A3A8C
    let Orange = #xC44E10
    let OrangeLogo = #xDF5E16
    let Yellow = #xF7B043
    let RailInk = #xC9D5EE
    let GoodInk = #x1D7A45
    let BadInk = #xB3261E

    // ---- Type: Anek Latin (fast, tight) + IBM Plex + Liberation Mono --
    let DisplayFont = FontFamily{
        latin := FontMember{res: crate_resource("self:resources/fonts/AnekLatin-Variable.ttf") asc: 0.0 desc: 0.0 weight: 760.0}
        ui := FontMember{res: crate_resource("self:resources/fonts/IBMPlexSans-Text.ttf") asc: -0.1 desc: 0.0}
    }
    let StrongFont = FontFamily{
        latin := FontMember{res: crate_resource("self:resources/fonts/AnekLatin-Variable.ttf") asc: 0.0 desc: 0.0 weight: 640.0}
        ui := FontMember{res: crate_resource("self:resources/fonts/IBMPlexSans-Text.ttf") asc: -0.1 desc: 0.0}
        devanagari := FontMember{res: crate_resource("self:resources/fonts/NotoSansDevanagari-Regular.ttf") asc: 0.0 desc: 0.0}
        gujarati := FontMember{res: crate_resource("self:resources/fonts/NotoSansGujarati-Regular.ttf") asc: 0.0 desc: 0.0}
    }
    let TextFont = FontFamily{
        latin := FontMember{res: crate_resource("self:resources/fonts/IBMPlexSans-Text.ttf") asc: -0.1 desc: 0.0}
        devanagari := FontMember{res: crate_resource("self:resources/fonts/NotoSansDevanagari-Regular.ttf") asc: 0.0 desc: 0.0}
        gujarati := FontMember{res: crate_resource("self:resources/fonts/NotoSansGujarati-Regular.ttf") asc: 0.0 desc: 0.0}
    }
    let MonoFont = FontFamily{
        latin := FontMember{res: crate_resource("self:resources/fonts/LiberationMono-Regular.ttf") asc: 0.0 desc: 0.0}
        ui := FontMember{res: crate_resource("self:resources/fonts/IBMPlexSans-Text.ttf") asc: -0.1 desc: 0.0}
    }

    // White ink only ever sits on navy; keep the SDF luma bias off so light
    // type on navy draws without faint glyph boxes (vendor shader untouched).
    mod.widgets.Label.draw_text.sdf_luma_bias = 0.0
    mod.widgets.Button.draw_text.sdf_luma_bias = 0.0

    let Eyebrow = Label{
        width: Fill
        padding: 0.
        draw_text +: {
            color: Orange
            text_style: theme.font_bold{font_family: StrongFont font_size: 13. line_spacing: 1.2}
        }
    }
    let Headline = Label{
        width: Fill
        padding: 0.
        draw_text +: {
            color: Navy
            text_style: theme.font_bold{font_family: DisplayFont font_size: 50. line_spacing: 0.95}
        }
    }
    let Lede = Label{
        width: Fill
        padding: 0.
        draw_text +: {
            color: Muted
            text_style: theme.font_regular{font_family: TextFont font_size: 19. line_spacing: 1.45}
        }
    }
    let Body = Label{
        width: Fill
        padding: 0.
        draw_text +: {
            color: Muted
            text_style: theme.font_regular{font_family: TextFont font_size: 15.5 line_spacing: 1.45}
        }
    }
    let Strong = Label{
        width: Fill
        padding: 0.
        draw_text +: {
            color: Ink
            text_style: theme.font_bold{font_family: StrongFont font_size: 17. line_spacing: 1.2}
        }
    }
    let Small = Label{
        width: Fill
        padding: 0.
        draw_text +: {
            color: Hint
            text_style: theme.font_regular{font_family: TextFont font_size: 13. line_spacing: 1.4}
        }
    }
    let Mono = Label{
        width: Fit
        padding: 0.
        draw_text +: {
            color: Ink
            text_style: theme.font_regular{font_family: MonoFont font_size: 15.}
        }
    }

    // ---- Buttons: navy primary (white ink), white secondary (ink) -------
    let PrimaryBtn = Button{
        width: Fit height: Fit
        padding: Inset{left: 22. right: 22. top: 14. bottom: 14.}
        align: Align{x: 0.5 y: 0.5}
        grab_key_focus: false
        reset_hover_on_click: true
        label_walk: Walk{width: Fit height: Fit margin: Inset{left: 2. right: 2.}}
        icon_walk: Walk{width: 0. height: 0.}
        draw_bg +: {
            color: Navy
            color_hover: NavyHi
            color_down: NavyBlack
            color_focus: Navy
            color_2: Navy
            color_2_hover: NavyHi
            color_2_down: NavyBlack
            color_2_focus: Navy
            border_color: Navy
            border_radius: 8.
            border_size: 0.
        }
        draw_text +: {
            color: #xFFFFFF
            color_hover: #xFFFFFF
            color_down: #xFFFFFF
            color_focus: #xFFFFFF
            text_style: theme.font_bold{font_family: StrongFont font_size: 16.}
        }
    }
    let SecondaryBtn = Button{
        width: Fit height: Fit
        padding: Inset{left: 20. right: 20. top: 13. bottom: 13.}
        align: Align{x: 0.5 y: 0.5}
        grab_key_focus: false
        reset_hover_on_click: true
        label_walk: Walk{width: Fit height: Fit margin: Inset{left: 2. right: 2.}}
        icon_walk: Walk{width: 0. height: 0.}
        draw_bg +: {
            color: Well
            color_hover: WellSoft
            color_down: #xDCE5F0
            color_focus: Well
            color_2: Well
            color_2_hover: WellSoft
            color_2_down: #xDCE5F0
            color_2_focus: Well
            border_color: Navy
            border_color_hover: Navy
            border_color_down: Navy
            border_color_focus: Navy
            border_radius: 8.
            border_size: 1.5
        }
        draw_text +: {
            color: Navy
            color_hover: Navy
            color_down: Navy
            color_focus: Navy
            text_style: theme.font_bold{font_family: StrongFont font_size: 16.}
        }
    }
    // Square ± key for the weight toy.
    let StepBtn = Button{
        width: 40 height: 40
        padding: 0.
        align: Align{x: 0.5 y: 0.5}
        grab_key_focus: false
        reset_hover_on_click: true
        label_walk: Walk{width: Fit height: Fit}
        icon_walk: Walk{width: 0. height: 0.}
        draw_bg +: {
            color: WellSoft
            color_hover: #xDCE5F0
            color_down: #xC9D6E7
            color_focus: WellSoft
            color_2: WellSoft
            color_2_hover: #xDCE5F0
            color_2_down: #xC9D6E7
            color_2_focus: WellSoft
            border_color: WellLine
            border_color_hover: WellLine
            border_color_down: WellLine
            border_color_focus: WellLine
            border_radius: 8.
            border_size: 1.
        }
        draw_text +: {
            color: Navy
            color_hover: Navy
            color_down: Navy
            color_focus: Navy
            text_style: theme.font_bold{font_family: StrongFont font_size: 20.}
        }
    }
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

    // ---- Chapter ground: cool desk light + the icon's speed streaks ----
    let Chapter = View{
        width: Fill height: Fill
        flow: Down
        spacing: 20
        align: Align{x: 0.0 y: 0.5}
        padding: Inset{left: 64. right: 64. top: 30. bottom: 36.}
        show_bg: true
        draw_bg +: {
            top: instance(#xE3EAF3)
            bottom: instance(#xD2DDEA)
            streak: fn(p, y, x0, thick) {
                let line = 1.0 - smoothstep(thick * 0.5, thick * 0.5 + 1.2, abs(p.y - y))
                let fade_in = smoothstep(x0, x0 + 260.0, p.x)
                return line * fade_in
            }
            pixel: fn() {
                let p = self.pos * self.rect_size
                let w = self.rect_size.x
                let h = self.rect_size.y
                var c = mix(self.top.xyz, self.bottom.xyz, self.pos.y)
                let s1 = self.streak(p, h * 0.12, w * 0.58, 7.0)
                let s2 = self.streak(p, h * 0.12 + 16.0, w * 0.70, 3.0)
                let s3 = self.streak(p, h * 0.12 + 30.0, w * 0.64, 12.0)
                let s4 = self.streak(p, h * 0.12 + 50.0, w * 0.80, 3.0)
                c = mix(c, vec3(0.874, 0.369, 0.086), s1 * 0.55)
                c = mix(c, vec3(0.969, 0.690, 0.263), s2 * 0.75)
                c = mix(c, vec3(0.039, 0.227, 0.549), s3 * 0.16)
                c = mix(c, vec3(0.874, 0.369, 0.086), s4 * 0.45)
                return vec4(c, 1.0)
            }
        }
    }

    let WellCard = RoundedView{
        width: Fill height: Fit
        flow: Down
        spacing: 10
        padding: Inset{left: 20. right: 20. top: 18. bottom: 20.}
        draw_bg +: {
            color: Well
            border_color: WellLine
            border_size: 1.
            border_radius: 12.
        }
    }
    // A key fact: orange bar (painted), bold line, plain line. `head` and
    // `line` are direct children so `Fact{head.text: … line.text: …}` works.
    let Fact = View{
        width: Fill height: Fit
        flow: Down
        spacing: 3
        padding: Inset{left: 16. right: 0. top: 2. bottom: 4.}
        show_bg: true
        draw_bg +: {
            bar: instance(#xDF5E16)
            pixel: fn() {
                let x = self.pos.x * self.rect_size.x
                let on = 1.0 - step(4.0, x)
                return vec4(self.bar.xyz * on, on)
            }
        }
        head := Strong{text: "Fact"}
        line := Body{text: "Detail"}
    }
    let ExampleTag = RoundedView{
        width: Fit height: Fit
        padding: Inset{left: 10. right: 10. top: 4. bottom: 5.}
        draw_bg +: {color: WellSoft border_color: WellLine border_size: 1. border_radius: 10.}
        Label{
            padding: 0.
            text: "Example"
            draw_text +: {color: Hint text_style: theme.font_regular{font_family: TextFont font_size: 12.}}
        }
    }

    // Weight toy: name, −, value, +.
    let Stepper = View{
        width: Fill height: Fit
        flow: Right
        spacing: 10
        align: Align{y: 0.5}
        name := Label{
            width: Fill
            padding: 0.
            draw_text +: {color: Muted text_style: theme.font_regular{font_family: TextFont font_size: 15.}}
            text: "Length"
        }
        minus := StepBtn{text: "−"}
        value := Label{
            width: 92
            padding: 0.
            align: Align{x: 0.5}
            draw_text +: {color: Ink text_style: theme.font_regular{font_family: MonoFont font_size: 17.}}
            text: "30 cm"
        }
        plus := StepBtn{text: "+"}
    }
    // Two-way choice: painted off/on faces, invisible hit on top.
    let Choice = View{
        width: Fit height: 38
        flow: Overlay
        off := RoundedView{
            width: Fit height: Fill
            padding: Inset{left: 14. right: 14. top: 0. bottom: 0.}
            align: Align{y: 0.5}
            draw_bg +: {color: Well border_color: WellLine border_size: 1. border_radius: 19.}
            txt := Label{padding: 0. draw_text +: {color: Navy text_style: theme.font_bold{font_family: StrongFont font_size: 14.}} text: "Choice"}
        }
        on := RoundedView{
            visible: false
            width: Fit height: Fill
            padding: Inset{left: 14. right: 14. top: 0. bottom: 0.}
            align: Align{y: 0.5}
            draw_bg +: {color: Navy border_radius: 19.}
            txt := Label{padding: 0. draw_text +: {color: #xFFFFFF text_style: theme.font_bold{font_family: StrongFont font_size: 14.}} text: "Choice"}
        }
        hit := HitBtn{}
    }

    // Tracking timeline bead (chapter 6 prop): dot + status word.
    let TrackStep = View{
        width: Fit height: Fit
        flow: Down
        spacing: 6
        align: Align{x: 0.5}
        dot := RoundedView{width: 18 height: 18 draw_bg +: {color: Navy border_radius: 9.}}
        word := Label{padding: 0. draw_text +: {color: Ink text_style: theme.font_bold{font_family: StrongFont font_size: 14.}} text: "Step"}
    }
    let TrackLine = RoundedView{width: Fill height: 3 margin: Inset{top: 8. bottom: 0. left: 0. right: 0.} draw_bg +: {color: Navy border_radius: 1.5}}

    // Chapter rail bead: hollow (ahead), yellow (passed), orange (here).
    let Bead = View{
        width: 78 height: 42
        flow: Overlay
        col := View{
            width: Fill height: Fit
            flow: Down
            spacing: 5
            align: Align{x: 0.5}
            padding: Inset{top: 3. bottom: 0. left: 0. right: 0.}
            face := View{
                width: 16 height: 16
                flow: Overlay
                align: Align{x: 0.5 y: 0.5}
                ahead := RoundedView{width: 12 height: 12 draw_bg +: {color: Navy border_color: #x6F8AC4 border_size: 2. border_radius: 6.}}
                passed := RoundedView{visible: false width: 12 height: 12 draw_bg +: {color: Yellow border_radius: 6.}}
                here := RoundedView{visible: false width: 16 height: 16 draw_bg +: {color: OrangeLogo border_color: #xFFFFFF border_size: 2. border_radius: 8.}}
            }
            name := Label{
                padding: 0.
                draw_text +: {color: RailInk text_style: theme.font_regular{font_family: TextFont font_size: 11.5}}
                text: "Step"
            }
        }
        hit := HitBtn{}
    }
    let RailGap = View{width: Fill height: 1}

    // Download / WhatsApp pair + the note that answers them.
    let CtaPair = View{
        width: Fill height: Fit
        flow: Down
        spacing: 10
        buttons := View{
            width: Fill height: Fit
            flow: Right
            spacing: 12
            download := PrimaryBtn{text: "Download free"}
            whatsapp := SecondaryBtn{text: "WhatsApp for a demo"}
        }
        note := Small{text: "Free on Android, Windows and Linux."}
    }

    startup() do #(App::script_component(vm)){
        ui: Root{
            main_window := Window{
                window.title: "Moklanjo — the courier desk"
                window.inner_size: vec2(1280, 800)
                pass.clear_color: Surface
                body +: {
                    width: Fill
                    height: Fill
                    flow: Down

                    // RoundedView, not View: a Fit-height plain View does not
                    // paint its background in this Makepad build.
                    top_bar := RoundedView{
                        width: Fill height: Fit
                        flow: Down
                        draw_bg +: {color: Navy border_radius: 0.}
                        brand_row := View{
                            width: Fill height: 50
                            flow: Right
                            spacing: 12
                            align: Align{y: 0.5}
                            padding: Inset{left: 24. right: 24. top: 0. bottom: 0.}
                            Label{
                                padding: 0.
                                text: "Moklanjo"
                                draw_text +: {color: #xFFFFFF text_style: theme.font_bold{font_family: DisplayFont font_size: 22.}}
                            }
                            tagline := Label{
                                padding: 0.
                                text: "the courier desk"
                                draw_text +: {color: Yellow text_style: theme.font_regular{font_family: TextFont font_size: 13.}}
                            }
                            View{width: Fill height: 1}
                            step_name := Label{
                                padding: 0.
                                text: "1 / 9 · Walk-in"
                                draw_text +: {color: RailInk text_style: theme.font_regular{font_family: TextFont font_size: 13.}}
                            }
                        }
                        // The parcel's tracking line is the chapter rail: one
                        // continuous track under evenly spaced beads.
                        rail := View{
                            width: Fill height: Fit
                            flow: Overlay
                            padding: Inset{left: 20. right: 20. top: 2. bottom: 8.}
                            rail_line := RoundedView{
                                width: Fill height: 2
                                margin: Inset{left: 39. right: 39. top: 10. bottom: 0.}
                                draw_bg +: {color: #x2B4C8F border_radius: 1.}
                            }
                            beads := View{
                                width: Fill height: Fit
                                flow: Right
                                b0 := Bead{col.name.text: "Walk-in"}
                                RailGap{}
                                b1 := Bead{col.name.text: "Weigh"}
                                RailGap{}
                                b2 := Bead{col.name.text: "Price"}
                                RailGap{}
                                b3 := Bead{col.name.text: "Label"}
                                RailGap{}
                                b4 := Bead{col.name.text: "Hand over"}
                                RailGap{}
                                b5 := Bead{col.name.text: "Track"}
                                RailGap{}
                                b6 := Bead{col.name.text: "COD"}
                                RailGap{}
                                b7 := Bead{col.name.text: "Offline"}
                                RailGap{}
                                b8 := Bead{col.name.text: "Start"}
                            }
                        }
                    }

                    slides := SlidesView{
                        width: Fill
                        height: Fill
                        anim_speed: 0.84
                        axis: Vertical

                        // 1 — Walk-in (walkin-loop)
                        walkin := Chapter{
                            hero_row := View{
                                width: Fill height: Fit
                                flow: Right
                                spacing: 44
                                align: Align{y: 0.5}
                                View{
                                    width: Fill height: Fit
                                    flow: Down
                                    spacing: 20
                                    Eyebrow{text: "MOKLANJO · A COURIER DESK, NOT A CASH REGISTER"}
                                    Headline{text: "Book a parcel before the next customer reaches the counter."}
                                    Lede{text: "Book, label, hand over, track and settle — one desk on your own devices, internet or not."}
                                    hero_cta := CtaPair{}
                                }
                                loop_card := WellCard{
                                    width: 340
                                    spacing: 14
                                    Small{text: "THE WALK-IN LOOP"}
                                    Fact{head.text: "Weigh" line.text: "Actual and volumetric, side by side."}
                                    Fact{head.text: "Charge" line.text: "Your rate card, cash or account."}
                                    Fact{head.text: "Next booking" line.text: "One tap clears the desk for the next person."}
                                }
                            }
                            Small{text: "Scroll, swipe up, or press ↓ — the rail above follows your parcel."}
                        }

                        // 2 — Weigh it right (chargeable-weight, units)
                        weigh := Chapter{
                            Eyebrow{text: "2 · WEIGH IT RIGHT"}
                            weigh_head := Headline{text: "Actual or volumetric — whichever is higher, rounded your way."}
                            weigh_row := View{
                                width: Fill height: Fit
                                flow: Right
                                spacing: 20
                                WellCard{
                                    spacing: 12
                                    View{
                                        width: Fill height: Fit
                                        flow: Right
                                        spacing: 10
                                        align: Align{y: 0.5}
                                        toy_title := Strong{width: Fill text: "Try a box"}
                                        dom := Choice{off.txt.text: "Domestic" on.txt.text: "Domestic"}
                                        intl := Choice{off.txt.text: "International" on.txt.text: "International"}
                                    }
                                    len := Stepper{name.text: "Length"}
                                    bre := Stepper{name.text: "Breadth"}
                                    hei := Stepper{name.text: "Height"}
                                    act := Stepper{name.text: "Weight on the scale"}
                                }
                                WellCard{
                                    spacing: 10
                                    draw_bg.color: Navy
                                    draw_bg.border_color: Navy
                                    Label{
                                        padding: 0.
                                        text: "CHARGEABLE WEIGHT"
                                        draw_text +: {color: Yellow text_style: theme.font_bold{font_family: StrongFont font_size: 13.}}
                                    }
                                    charge := Label{
                                        padding: 0.
                                        text: "2.0 kg"
                                        draw_text +: {color: #xFFFFFF text_style: theme.font_bold{font_family: DisplayFont font_size: 56.}}
                                    }
                                    winner := Label{
                                        width: Fill
                                        padding: 0.
                                        text: "The box's size sets the charge."
                                        draw_text +: {color: #xFFFFFF text_style: theme.font_bold{font_family: StrongFont font_size: 16.}}
                                    }
                                    working := Label{
                                        width: Fill
                                        padding: 0.
                                        text: "30 × 20 × 15 ÷ 5000 = 1.8 kg"
                                        draw_text +: {color: RailInk text_style: theme.font_regular{font_family: MonoFont font_size: 15.}}
                                    }
                                    actual := Label{
                                        width: Fill
                                        padding: 0.
                                        text: "On the scale: 1.2 kg"
                                        draw_text +: {color: RailInk text_style: theme.font_regular{font_family: MonoFont font_size: 15.}}
                                    }
                                    rule := Label{
                                        width: Fill
                                        padding: 0.
                                        text: "Higher of the two, rounded up to the next 0.5 kg."
                                        draw_text +: {color: RailInk text_style: theme.font_regular{font_family: TextFont font_size: 14. line_spacing: 1.4}}
                                    }
                                }
                            }
                            weigh_note := Small{text: "In the app the divisor, the rounding step and your units (cm or inches, kg or lb) are the shop's to set. The International chip switches ÷6000 for one booking without touching Settings."}
                        }

                        // 3 — Price it (rate-master, party-rates, carrier-compare)
                        price := Chapter{
                            Eyebrow{text: "3 · PRICE IT"}
                            Headline{text: "Your rate cards, your party rates — and your margin on every docket."}
                            price_row := View{
                                width: Fill height: Fit
                                flow: Right
                                spacing: 20
                                rates := WellCard{
                                    spacing: 8
                                    View{
                                        width: Fill height: Fit
                                        flow: Right
                                        align: Align{y: 0.5}
                                        Strong{width: Fill text: "Rate card"}
                                        ExampleTag{}
                                    }
                                    View{
                                        width: Fill height: Fit
                                        flow: Right
                                        spacing: 8
                                        Small{width: Fill text: "Zone · service"}
                                        Small{width: 96 text: "First 0.5 kg"}
                                        Small{width: 100 text: "Each 0.5 kg"}
                                    }
                                    RoundedView{width: Fill height: 1 draw_bg +: {color: WellLine border_radius: 0.}}
                                    View{
                                        width: Fill height: Fit
                                        flow: Right
                                        spacing: 8
                                        Body{width: Fill text: "Local · Surface"}
                                        Mono{width: 96 text: "₹ 40"}
                                        Mono{width: 100 text: "₹ 20"}
                                    }
                                    View{
                                        width: Fill height: Fit
                                        flow: Right
                                        spacing: 8
                                        Body{width: Fill text: "Within state · Express"}
                                        Mono{width: 96 text: "₹ 60"}
                                        Mono{width: 100 text: "₹ 35"}
                                    }
                                    View{
                                        width: Fill height: Fit
                                        flow: Right
                                        spacing: 8
                                        Body{width: Fill text: "Metro · Air"}
                                        Mono{width: 96 text: "₹ 95"}
                                        Mono{width: 100 text: "₹ 60"}
                                    }
                                }
                                price_facts := View{
                                    width: Fill height: Fit
                                    flow: Down
                                    spacing: 16
                                    Fact{head.text: "Zone × service × weight slab" line.text: "First slab and each extra slab, per zone and service."}
                                    Fact{head.text: "A consignor's own card" line.text: "Regular senders get their rates; the default card covers everyone else."}
                                    Fact{head.text: "Margin at the desk" line.text: "Your sell price against the carrier's price, on every docket."}
                                }
                            }
                        }

                        // 4 — Label it (awb-pools, label-scan)
                        labels := Chapter{
                            Eyebrow{text: "4 · LABEL IT"}
                            Headline{text: "AWB from your own number pools. Label, scan, find."}
                            label_row := View{
                                width: Fill height: Fit
                                flow: Right
                                spacing: 28
                                align: Align{y: 0.5}
                                awb_label := RoundedView{
                                    width: 380 height: Fit
                                    flow: Down
                                    spacing: 10
                                    padding: Inset{left: 22. right: 22. top: 20. bottom: 20.}
                                    draw_bg +: {color: Well border_color: #x9FB0C8 border_size: 1.5 border_radius: 6.}
                                    View{
                                        width: Fill height: Fit
                                        flow: Right
                                        align: Align{y: 0.5}
                                        Label{
                                            width: Fill
                                            padding: 0.
                                            text: "AWB"
                                            draw_text +: {color: Ink text_style: theme.font_bold{font_family: StrongFont font_size: 15.}}
                                        }
                                        ExampleTag{}
                                    }
                                    // Code 128-style bars, drawn (illustration, not a scannable code).
                                    View{
                                        width: Fill height: 74
                                        show_bg: true
                                        draw_bg +: {
                                            pixel: fn() {
                                                let x = self.pos.x * self.rect_size.x
                                                let cell = floor(x / 3.0)
                                                let r = fract(sin(cell * 12.9898 + 78.233) * 43758.5453)
                                                let quiet = step(10.0, x) * step(x, self.rect_size.x - 10.0)
                                                let bar = step(0.48, r) * quiet
                                                return vec4(mix(vec3(1.0, 1.0, 1.0), vec3(0.043, 0.071, 0.125), bar), 1.0)
                                            }
                                        }
                                    }
                                    Label{
                                        width: Fill
                                        padding: 0.
                                        align: Align{x: 0.5}
                                        text: "7004 1182 35"
                                        draw_text +: {color: Ink text_style: theme.font_regular{font_family: MonoFont font_size: 20.}}
                                    }
                                    RoundedView{width: Fill height: 1 draw_bg +: {color: WellLine border_radius: 0.}}
                                    Small{text: "TO  Pune 411001  ·  1 piece  ·  2.0 kg chargeable"}
                                }
                                label_facts := View{
                                    width: Fill height: Fit
                                    flow: Down
                                    spacing: 16
                                    Fact{head.text: "Number pools per carrier" line.text: "Enter each carrier's start–end series once; every booking takes the next number."}
                                    Fact{head.text: "Code 128 labels" line.text: "The AWB goes on the box as a barcode and in plain digits."}
                                    Fact{head.text: "Scan to find" line.text: "Scan a label to open its docket and act on it."}
                                }
                            }
                        }

                        // 5 — Two roads (manifest, rider-runs)
                        roads := Chapter{
                            Eyebrow{text: "5 · HAND OVER"}
                            Headline{text: "Hand it to a carrier, or send your own rider. Same desk."}
                            roads_row := View{
                                width: Fill height: Fit
                                flow: Right
                                spacing: 20
                                WellCard{
                                    spacing: 12
                                    RoundedView{width: 44 height: 5 draw_bg +: {color: OrangeLogo border_radius: 2.5}}
                                    Strong{text: "Into a carrier bag"}
                                    Body{text: "The manifest holds exactly the parcels you picked — all of them or none. A parcel booked with another carrier cannot slip into this bag."}
                                }
                                WellCard{
                                    spacing: 12
                                    RoundedView{width: 44 height: 5 draw_bg +: {color: Navy border_radius: 2.5}}
                                    Strong{text: "Onto your own rider's run"}
                                    Body{text: "Pick the parcels, print the run sheet, and the rider works from the same app in rider mode — proof of delivery included."}
                                }
                            }
                        }

                        // 6 — Track and prove (tracking-rules, pod, ndr-rto)
                        track := Chapter{
                            Eyebrow{text: "6 · TRACK AND PROVE"}
                            Headline{text: "Every move recorded — and only moves that make sense."}
                            timeline := WellCard{
                                spacing: 16
                                track_line := View{
                                    width: Fill height: Fit
                                    flow: Right
                                    align: Align{y: 0.0}
                                    TrackStep{word.text: "Booked"}
                                    TrackLine{}
                                    TrackStep{word.text: "With carrier"}
                                    TrackLine{}
                                    TrackStep{word.text: "Out for delivery"}
                                    TrackLine{}
                                    TrackStep{dot.draw_bg.color: GoodInk word.text: "Delivered"}
                                }
                                // Phones: the four steps as one line (no room for beads).
                                track_compact := Strong{visible: false text: "Booked → With carrier → Out for delivery → Delivered"}
                                RoundedView{width: Fill height: 1 draw_bg +: {color: WellLine border_radius: 0.}}
                                branch := View{
                                    width: Fill height: Fit
                                    flow: Right
                                    spacing: 10
                                    align: Align{y: 0.5}
                                    RoundedView{width: 12 height: 12 draw_bg +: {color: BadInk border_radius: 6.}}
                                    Strong{width: Fit text: "Failed"}
                                    Body{width: Fill text: "→ reason → out again, or return to sender."}
                                }
                            }
                            track_facts := View{
                                width: Fill height: Fit
                                flow: Right
                                spacing: 20
                                Fact{head.text: "Impossible moves refused" line.text: "Delivered, returned and cancelled are final."}
                                Fact{head.text: "Proof of delivery" line.text: "A photo and the receiver's signature on the docket."}
                                Fact{head.text: "Failed deliveries" line.text: "Reasons, attempts, out again, or back to the sender."}
                            }
                        }

                        // 7 — Money comes home (cod-register, rider-settle, day-book, period-bill, gst-csv)
                        cod := Chapter{
                            Eyebrow{text: "7 · MONEY COMES HOME"}
                            Headline{text: "COD you can see: expected, handed in, owed to each consignor."}
                            cod_row := View{
                                width: Fill height: Fit
                                flow: Right
                                spacing: 20
                                settle := WellCard{
                                    spacing: 8
                                    View{
                                        width: Fill height: Fit
                                        flow: Right
                                        align: Align{y: 0.5}
                                        Strong{width: Fill text: "Rider cash, end of run"}
                                        ExampleTag{}
                                    }
                                    View{
                                        width: Fill height: Fit
                                        flow: Right
                                        spacing: 8
                                        Small{width: Fill text: "Rider"}
                                        Small{width: 96 text: "Expected"}
                                        Small{width: 96 text: "Handed in"}
                                    }
                                    RoundedView{width: Fill height: 1 draw_bg +: {color: WellLine border_radius: 0.}}
                                    View{
                                        width: Fill height: Fit
                                        flow: Right
                                        spacing: 8
                                        Body{width: Fill text: "Rider A · 14 drops"}
                                        Mono{width: 96 text: "₹ 4,250"}
                                        Mono{width: 96 text: "₹ 4,250"}
                                    }
                                    View{
                                        width: Fill height: Fit
                                        flow: Right
                                        spacing: 8
                                        Body{width: Fill text: "Rider B · 9 drops"}
                                        Mono{width: 96 text: "₹ 3,100"}
                                        Mono{width: 96 draw_text.color: BadInk text: "₹ 2,900"}
                                    }
                                    Small{text: "Short ₹ 200 on Rider B — named, not lost in a total."}
                                }
                                cod_facts := View{
                                    width: Fill height: Fit
                                    flow: Down
                                    spacing: 16
                                    Fact{head.text: "COD register" line.text: "Collected against paid out, per consignor, with remittance batches."}
                                    Fact{head.text: "Rider settlement" line.text: "Expected against handed in, and the commission slip."}
                                    Fact{head.text: "Books that close" line.text: "Day book, month bill per account, and a GST summary CSV for your accountant."}
                                }
                            }
                        }

                        // 8 — Internet down (offline-lan; the multi-device half is gated)
                        offline := Chapter{
                            Eyebrow{text: "8 · INTERNET DOWN?"}
                            Headline{text: "Keep booking."}
                            Lede{text: "Booking, labels, scanning, COD and the books all run on the desk itself. When the internet drops, nothing at the counter waits for it."}
                            offline_row := View{
                                width: Fill height: Fit
                                flow: Right
                                spacing: 20
                                WellCard{
                                    spacing: 12
                                    View{
                                        width: Fill height: Fit
                                        flow: Right
                                        spacing: 10
                                        align: Align{y: 0.5}
                                        RoundedView{width: 12 height: 12 draw_bg +: {color: BadInk border_radius: 6.}}
                                        Strong{width: Fill text: "Office router: no internet"}
                                    }
                                    View{
                                        width: Fill height: Fit
                                        flow: Right
                                        spacing: 10
                                        align: Align{y: 0.5}
                                        RoundedView{width: 12 height: 12 draw_bg +: {color: GoodInk border_radius: 6.}}
                                        Strong{width: Fill text: "Desk: still booking"}
                                    }
                                    // Shown only after the owner's real-hardware LAN pilot passes.
                                    offline_lan := View{
                                        visible: false
                                        width: Fill height: Fit
                                        flow: Right
                                        spacing: 10
                                        align: Align{y: 0.5}
                                        RoundedView{width: 12 height: 12 draw_bg +: {color: GoodInk border_radius: 6.}}
                                        Strong{width: Fill text: "Second counter and rider phone: joined over the office Wi‑Fi"}
                                    }
                                }
                                Fact{head.text: "Free forever on your devices" line.text: "Nothing is counted per rider, user or parcel. No subscription."}
                            }
                        }

                        // 9 — Start (free-local)
                        start := Chapter{
                            Eyebrow{text: "9 · START"}
                            Headline{text: "One desk. Your parcels. Your books."}
                            Lede{text: "Moklanjo runs on Android, Windows and Linux. A rider uses the same app in rider mode."}
                            close_cta := CtaPair{}
                            RoundedView{width: Fill height: 1 draw_bg +: {color: WellLine border_radius: 0.}}
                            footer := View{
                                width: Fill height: Fit
                                flow: Right
                                spacing: 12
                                align: Align{y: 0.5}
                                Small{width: Fill text: "Moklanjo is a Kumkaj product by Gotigin."}
                                to_kumkaj := SecondaryBtn{text: "kumkaj.com  →"}
                            }
                        }
                    }
                }
            }
        }
    }
}
