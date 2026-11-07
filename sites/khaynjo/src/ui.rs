// khaynjo.com — one Makepad 2 `script_mod!` (included from main.rs).
// Story: apps/khaynjo/docs/KHAYNJO_WEBSITE.md §3 ("One table, one
// service"). Palette: docs/DESIGN.md "Color tokens — Khaynjo".
// Started as a copy of sites/kumkaj and diverged: warm cream ground,
// deep-red chrome, gold light, rounded Baloo type, table-tile navigation in
// the app's own visit colours. Feature sentences cite KHAYNJO.md §4b/§4d
// (claim ids in comments); `lan-pilot` sentences wait for the owner.
script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    // ---- Tokens (Khaynjo icon / DESIGN.md) ---------------------------
    let Cream = #xF9F4F0
    let Well = #xFFFFFF
    let WellWarm = #xF6E3D3
    let WellDeep = #xEFD2BC
    let WellLine = #xE2CBB9
    let Ink = #x260B08
    let Muted = #x6B4A3F
    let Hint = #x8A6A5C
    let Red = #x9A0E10
    let RedHover = #xB43A12
    let Maroon = #x69090C
    let Accent = #xB43A12
    let Gold = #xF69D2C
    let GoldSoft = #xF7B043
    let Paper = #xFFFDF8

    // Visit colours — copied from Khaynjo's FloorTile (fill, border).
    let FreeFill = #xFFFFFF
    let FreeLine = #xD8C7BD
    let SeatFill = #xFFF4CC
    let SeatLine = #xE8C200
    let KitchenFill = #xFFE9A8
    let KitchenLine = #xD9A400
    let ReadyFill = #xFFC48A
    let ReadyLine = #xE06A00
    let ServedFill = #xCFE3FF
    let ServedLine = #x3B7DD8
    let BillFill = #xBFEBC8
    let BillLine = #x2E9E4F
    let PaidFill = #xE3DCD7
    let PaidLine = #x8C7B72

    // ---- Type: Baloo 2 (rounded, like the logo), IBM Plex, Liberation Mono
    let DisplayFont = FontFamily{
        latin := FontMember{res: crate_resource("self:resources/fonts/Baloo2-Variable.ttf") asc: 0.0 desc: 0.0 weight: 720.0}
        ui := FontMember{res: crate_resource("self:resources/fonts/IBMPlexSans-SemiBold.ttf") asc: -0.1 desc: 0.0}
        gujarati := FontMember{res: crate_resource("self:resources/fonts/NotoSansGujarati-Bold.ttf") asc: 0.0 desc: 0.0}
    }
    let StrongFont = FontFamily{
        latin := FontMember{res: crate_resource("self:resources/fonts/IBMPlexSans-SemiBold.ttf") asc: -0.1 desc: 0.0}
        gujarati := FontMember{res: crate_resource("self:resources/fonts/NotoSansGujarati-Bold.ttf") asc: 0.0 desc: 0.0}
        devanagari := FontMember{res: crate_resource("self:resources/fonts/NotoSansDevanagari-Regular.ttf") asc: 0.0 desc: 0.0}
    }
    let TextFont = FontFamily{
        latin := FontMember{res: crate_resource("self:resources/fonts/IBMPlexSans-Text.ttf") asc: -0.1 desc: 0.0}
        gujarati := FontMember{res: crate_resource("self:resources/fonts/NotoSansGujarati-Regular.ttf") asc: 0.0 desc: 0.0}
        devanagari := FontMember{res: crate_resource("self:resources/fonts/NotoSansDevanagari-Regular.ttf") asc: 0.0 desc: 0.0}
    }
    let TicketFont = FontFamily{
        latin := FontMember{res: crate_resource("self:resources/fonts/LiberationMono-Regular.ttf") asc: 0.0 desc: 0.0}
        ui := FontMember{res: crate_resource("self:resources/fonts/IBMPlexSans-Text.ttf") asc: -0.1 desc: 0.0}
    }

    // Cream on deep red is the only light-on-dark here; drop the SDF luma
    // bias so it never paints glyph boxes (vendor shader untouched).
    mod.widgets.Label.draw_text.sdf_luma_bias = 0.0
    mod.widgets.Button.draw_text.sdf_luma_bias = 0.0

    let Eyebrow = Label{
        width: Fill
        padding: 0.
        draw_text +: {
            color: Accent
            text_style: theme.font_bold{font_family: StrongFont font_size: 13. line_spacing: 1.2}
        }
    }
    let Headline = Label{
        width: Fill
        padding: 0.
        draw_text +: {
            color: Maroon
            text_style: theme.font_bold{font_family: DisplayFont font_size: 48. line_spacing: 0.72}
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
            text_style: theme.font_bold{font_family: StrongFont font_size: 16. line_spacing: 1.25}
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
    let TicketRule = View{
        width: Fill height: 9
        show_bg: true
        draw_bg +: {
            pixel: fn() {
                let x = self.pos.x * self.rect_size.x
                let y = self.pos.y * self.rect_size.y
                let dash = step(0.45, fract(x / 7.0))
                let row = 1.0 - step(1.2, abs(y - 4.5))
                let a = dash * row * 0.75
                return vec4(vec3(0.149, 0.043, 0.031) * a, a)
            }
        }
    }
    let Ticket = Label{
        width: Fill
        padding: 0.
        draw_text +: {
            color: Ink
            text_style: theme.font_regular{font_family: TicketFont font_size: 14. line_spacing: 1.35}
        }
    }

    // ---- Buttons: deep red primary (white ink), cream secondary --------
    let PrimaryBtn = Button{
        width: Fit height: Fit
        padding: Inset{left: 22. right: 22. top: 14. bottom: 14.}
        align: Align{x: 0.5 y: 0.5}
        grab_key_focus: false
        reset_hover_on_click: true
        label_walk: Walk{width: Fit height: Fit margin: Inset{left: 2. right: 2.}}
        icon_walk: Walk{width: 0. height: 0.}
        draw_bg +: {
            color: Red
            color_hover: RedHover
            color_down: Maroon
            color_focus: Red
            color_2: Red
            color_2_hover: RedHover
            color_2_down: Maroon
            color_2_focus: Red
            border_color: Red
            border_radius: 14.
            border_size: 0.
        }
        draw_text +: {
            color: #xFFFFFF
            color_hover: #xFFFFFF
            color_down: #xFFFFFF
            color_focus: #xFFFFFF
            text_style: theme.font_bold{font_family: StrongFont font_size: 15.5}
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
            color: WellDeep
            color_hover: WellWarm
            color_down: #xE4C2A8
            color_focus: WellDeep
            color_2: WellDeep
            color_2_hover: WellWarm
            color_2_down: #xE4C2A8
            color_2_focus: WellDeep
            border_color: #xD8BBA6
            border_color_hover: #xD8BBA6
            border_color_down: #xD8BBA6
            border_color_focus: #xD8BBA6
            border_radius: 14.
            border_size: 1.
        }
        draw_text +: {
            color: Ink
            color_hover: Ink
            color_down: Ink
            color_focus: Ink
            text_style: theme.font_bold{font_family: StrongFont font_size: 15.5}
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

    // ---- Chapter ground: cream, a pool of warm lamp light ---------------
    let Chapter = View{
        width: Fill height: Fill
        flow: Down
        spacing: 20
        align: Align{x: 0.0 y: 0.5}
        padding: Inset{left: 64. right: 64. top: 30. bottom: 36.}
        show_bg: true
        draw_bg +: {
            pixel: fn() {
                let aspect = self.rect_size.x / max(1.0, self.rect_size.y)
                let d = length(vec2((self.pos.x - 0.12) * aspect, self.pos.y - 0.0))
                let lamp = 1.0 - smoothstep(0.0, 1.25, d)
                let base = mix(vec3(0.976, 0.957, 0.941), vec3(0.957, 0.906, 0.871), self.pos.y)
                let warm = mix(base, vec3(0.992, 0.890, 0.733), lamp * 0.55)
                // A deep-red rule along the very bottom, like a table runner.
                let y = (1.0 - self.pos.y) * self.rect_size.y
                let runner = 1.0 - step(5.0, y)
                return vec4(mix(warm, vec3(0.604, 0.055, 0.063), runner), 1.0)
            }
        }
    }

    let WarmCard = RoundedView{
        width: Fill height: Fit
        flow: Down
        spacing: 10
        padding: Inset{left: 20. right: 20. top: 18. bottom: 20.}
        draw_bg +: {
            color: Well
            border_color: WellLine
            border_size: 1.
            border_radius: 18.
        }
    }
    let ExampleTag = RoundedView{
        width: Fit height: Fit
        padding: Inset{left: 10. right: 10. top: 4. bottom: 5.}
        draw_bg +: {color: WellWarm border_radius: 10.}
        Label{
            padding: 0.
            text: "Example"
            draw_text +: {color: Muted text_style: theme.font_regular{font_family: TextFont font_size: 12.}}
        }
    }
    // A point: gold dot, bold line, plain line (direct children for overrides).
    let Point = View{
        width: Fill height: Fit
        flow: Down
        spacing: 3
        padding: Inset{left: 20. right: 0. top: 0. bottom: 2.}
        show_bg: true
        draw_bg +: {
            pixel: fn() {
                let p = self.pos * self.rect_size
                let d = length(p - vec2(6.0, 11.0))
                let a = 1.0 - smoothstep(4.0, 5.2, d)
                return vec4(vec3(0.965, 0.616, 0.173) * a, a)
            }
        }
        head := Strong{text: "Point"}
        line := Body{text: "Detail"}
    }

    // Floor toy tile: the app's seven visit layers, name / word / meta, hit.
    let TableTile = View{
        width: 160 height: 100
        flow: Overlay
        bg_free := RoundedView{width: Fill height: Fill draw_bg +: {color: FreeFill border_color: FreeLine border_size: 2. border_radius: 14.}}
        bg_seated := RoundedView{visible: false width: Fill height: Fill draw_bg +: {color: SeatFill border_color: SeatLine border_size: 2. border_radius: 14.}}
        bg_kitchen := RoundedView{visible: false width: Fill height: Fill draw_bg +: {color: KitchenFill border_color: KitchenLine border_size: 2. border_radius: 14.}}
        bg_ready := RoundedView{visible: false width: Fill height: Fill draw_bg +: {color: ReadyFill border_color: ReadyLine border_size: 2. border_radius: 14.}}
        bg_served := RoundedView{visible: false width: Fill height: Fill draw_bg +: {color: ServedFill border_color: ServedLine border_size: 2. border_radius: 14.}}
        bg_bill := RoundedView{visible: false width: Fill height: Fill draw_bg +: {color: BillFill border_color: BillLine border_size: 2. border_radius: 14.}}
        bg_paid := RoundedView{visible: false width: Fill height: Fill draw_bg +: {color: PaidFill border_color: PaidLine border_size: 2. border_radius: 14.}}
        View{
            width: Fill height: Fill
            flow: Down
            spacing: 2
            padding: Inset{left: 12. right: 8. top: 8. bottom: 8.}
            name := Label{padding: 0. draw_text +: {color: Ink text_style: theme.font_bold{font_family: DisplayFont font_size: 20. line_spacing: 0.7}} text: "T1"}
            word := Label{padding: 0. draw_text +: {color: Ink text_style: theme.font_bold{font_family: StrongFont font_size: 14.}} text: "Free"}
            meta := Label{padding: 0. draw_text +: {color: Muted text_style: theme.font_regular{font_family: TextFont font_size: 13.}} text: ""}
        }
        hit := HitBtn{}
    }

    // Kitchen column card (static): KOT, table, items, age.
    let KdsCard = RoundedView{
        width: Fill height: Fit
        flow: Down
        spacing: 2
        padding: Inset{left: 12. right: 12. top: 10. bottom: 10.}
        draw_bg +: {color: Well border_color: WellLine border_size: 1. border_radius: 12.}
        head := Label{padding: 0. draw_text +: {color: Ink text_style: theme.font_bold{font_family: StrongFont font_size: 14.}} text: "KOT"}
        line := Label{width: Fill padding: 0. draw_text +: {color: Muted text_style: theme.font_regular{font_family: TextFont font_size: 13.}} text: "Items"}
    }
    let KdsColumn = RoundedView{
        width: Fill height: Fit
        flow: Down
        spacing: 8
        padding: Inset{left: 10. right: 10. top: 10. bottom: 12.}
        draw_bg +: {color: WellWarm border_radius: 14.}
        title := Label{padding: 0. draw_text +: {color: Maroon text_style: theme.font_bold{font_family: DisplayFont font_size: 18.}} text: "Column"}
    }

    // Top-bar chapter tile: a small table in that chapter's visit colour.
    let NavTile = View{
        width: 22 height: 26
        flow: Overlay
        align: Align{x: 0.5 y: 0.5}
        idle := RoundedView{width: 14 height: 14 draw_bg +: {color: #xFFFFFF border_color: #xD8C7BD border_size: 1.5 border_radius: 4.}}
        here := RoundedView{visible: false width: 20 height: 20 draw_bg +: {color: #xFFFFFF border_color: Gold border_size: 3. border_radius: 5.}}
        hit := HitBtn{}
    }

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
                window.title: "Khaynjo — lunch rush, no internet, no problem"
                window.inner_size: vec2(1280, 800)
                pass.clear_color: Cream
                body +: {
                    width: Fill
                    height: Fill
                    flow: Down

                    // RoundedView: a Fit/fixed plain View may skip its background.
                    top_bar := RoundedView{
                        width: Fill height: 58
                        flow: Right
                        spacing: 12
                        align: Align{y: 0.5}
                        padding: Inset{left: 24. right: 20. top: 0. bottom: 0.}
                        draw_bg +: {color: Red border_radius: 0.}
                        Label{
                            padding: 0.
                            text: "Khaynjo"
                            draw_text +: {color: Cream text_style: theme.font_bold{font_family: DisplayFont font_size: 25.}}
                        }
                        tagline := Label{
                            padding: 0.
                            text: "the restaurant system"
                            draw_text +: {color: GoldSoft text_style: theme.font_regular{font_family: TextFont font_size: 13.}}
                        }
                        View{width: Fill height: 1}
                        chapter_name := Label{
                            padding: 0.
                            text: "Lunch rush"
                            draw_text +: {color: #xF2D9C9 text_style: theme.font_regular{font_family: TextFont font_size: 13.}}
                        }
                        nav := View{
                            width: Fit height: Fit
                            flow: Right
                            spacing: 2
                            align: Align{y: 0.5}
                            n0 := NavTile{}
                            n1 := NavTile{idle.draw_bg.color: SeatFill here.draw_bg.color: SeatFill}
                            n2 := NavTile{idle.draw_bg.color: KitchenFill here.draw_bg.color: KitchenFill}
                            n3 := NavTile{idle.draw_bg.color: ReadyFill here.draw_bg.color: ReadyFill}
                            n4 := NavTile{idle.draw_bg.color: ServedFill here.draw_bg.color: ServedFill}
                            n5 := NavTile{idle.draw_bg.color: BillFill here.draw_bg.color: BillFill}
                            n6 := NavTile{idle.draw_bg.color: PaidFill here.draw_bg.color: PaidFill}
                            n7 := NavTile{}
                            n8 := NavTile{}
                            n9 := NavTile{}
                        }
                        counter := Label{
                            padding: 0.
                            text: "1 / 10"
                            draw_text +: {color: #xF2D9C9 text_style: theme.font_bold{font_family: StrongFont font_size: 12.5}}
                        }
                    }

                    slides := SlidesView{
                        width: Fill
                        height: Fill
                        anim_speed: 0.86
                        axis: Vertical

                        // 1 — Lunch rush (floor-states, table-cards)
                        rush := Chapter{
                            rush_row := View{
                                width: Fill height: Fit
                                flow: Right
                                spacing: 24
                                align: Align{y: 0.5}
                                View{
                                    width: Fill height: Fit
                                    flow: Down
                                    spacing: 18
                                    Eyebrow{text: "KHAYNJO · THE RESTAURANT SYSTEM"}
                                    Headline{text: "Lunch rush. No internet. No problem."}
                                    rush_lede := Lede{text: "From table to kitchen to bill, on your own devices. Every table shows its minutes and its running bill at a glance."}
                                    rush_cta := CtaPair{}
                                }
                                floor_card := WarmCard{
                                    width: 560
                                    spacing: 12
                                    View{
                                        width: Fill height: Fit
                                        flow: Right
                                        align: Align{y: 0.5}
                                        Strong{width: Fill text: "Tap a table to walk it through a visit"}
                                        ExampleTag{}
                                    }
                                    // Two fixed rows (a wrap row mis-sizes overlay tiles).
                                    floor := View{
                                        width: Fill height: Fit
                                        flow: Down
                                        spacing: 10
                                        View{
                                            width: Fill height: Fit
                                            flow: Right
                                            spacing: 10
                                            t1 := TableTile{}
                                            t2 := TableTile{}
                                            t3 := TableTile{}
                                        }
                                        View{
                                            width: Fill height: Fit
                                            flow: Right
                                            spacing: 10
                                            t4 := TableTile{}
                                            t5 := TableTile{}
                                            t6 := TableTile{}
                                        }
                                    }
                                    floor_note := Small{text: "Same colours and words as Khaynjo's floor. Bill printed → Paid → Free: a table frees only once it is paid."}
                                }
                            }
                        }

                        // 2 — Seated, order taken (seated, variations)
                        order := Chapter{
                            Eyebrow{text: "2 · SEATED, ORDER TAKEN"}
                            Headline{text: "Take the order at the table. Sizes, add-ons and notes, right there."}
                            order_row := View{
                                width: Fill height: Fit
                                flow: Right
                                spacing: 20
                                WarmCard{
                                    spacing: 8
                                    View{
                                        width: Fill height: Fit
                                        flow: Right
                                        spacing: 10
                                        align: Align{y: 0.5}
                                        RoundedView{
                                            width: Fit height: Fit
                                            padding: Inset{left: 10. right: 10. top: 4. bottom: 5.}
                                            draw_bg +: {color: SeatFill border_color: SeatLine border_size: 1.5 border_radius: 8.}
                                            Label{padding: 0. text: "T4 · Occupied" draw_text +: {color: Ink text_style: theme.font_bold{font_family: StrongFont font_size: 13.}}}
                                        }
                                        View{width: Fill height: 1}
                                        ExampleTag{}
                                    }
                                    Strong{text: "Paneer tikka — Full"}
                                    Body{text: "+ extra mint chutney"}
                                    Strong{text: "Masala chaas × 2"}
                                    Body{text: "Note: less salt"}
                                    Strong{text: "Butter naan × 4"}
                                }
                                order_points := View{
                                    width: Fill height: Fit
                                    flow: Down
                                    spacing: 16
                                    Point{head.text: "Seat first" line.text: "Mark a table Occupied the moment guests sit — before any dish is ordered."}
                                    Point{head.text: "Sizes and add-ons" line.text: "Pick one size; add optional extras up to the limit you set."}
                                    Point{head.text: "A note per dish" line.text: "“Less spicy” goes to the kitchen with the dish it belongs to."}
                                }
                            }
                        }

                        // 3 — The KOT (kot-station, kot-no-prices; kot-autoprint gated)
                        kot := Chapter{
                            Eyebrow{text: "3 · THE KOT"}
                            Headline{text: "One ticket per kitchen station. Never a price on it."}
                            kot_row := View{
                                width: Fill height: Fit
                                flow: Right
                                spacing: 28
                                align: Align{y: 0.5}
                                ticket := RoundedView{
                                    width: 330 height: Fit
                                    flow: Down
                                    spacing: 4
                                    padding: Inset{left: 20. right: 20. top: 18. bottom: 20.}
                                    draw_bg +: {color: Paper border_color: #xD9CBBE border_size: 1. border_radius: 4.}
                                    Ticket{text: "KOT 27 · HOT KITCHEN"}
                                    Ticket{text: "Table T4 · by Captain 1"}
                                    Ticket{text: "12:42"}
                                    TicketRule{}
                                    Ticket{text: "1  Paneer tikka (Full)"}
                                    Ticket{text: "   + extra mint chutney"}
                                    Ticket{text: "2  Masala chaas"}
                                    Ticket{text: "   note: less salt"}
                                    Ticket{text: "4  Butter naan"}
                                    TicketRule{}
                                    Small{text: "Example ticket"}
                                }
                                kot_points := View{
                                    width: Fill height: Fit
                                    flow: Down
                                    spacing: 16
                                    Point{head.text: "Split by station" line.text: "Hot kitchen, cold counter, bar — each gets only its own dishes, numbered per day."}
                                    Point{head.text: "Who, which table, what notes" line.text: "The ticket carries the waiter's name, the table and every note."}
                                    Point{head.text: "No prices" line.text: "The kitchen cooks; the bill is the counter's business."}
                                    // Shown only after the owner's real-hardware LAN pilot passes.
                                    kot_lan := Point{visible: false head.text: "Prints itself" line.text: "Send, and the ticket prints at the right station — nobody carries it."}
                                }
                            }
                        }

                        // 4 — Kitchen in step (kds, all-day-counts, coursing)
                        kitchen := Chapter{
                            Eyebrow{text: "4 · KITCHEN IN STEP"}
                            Headline{text: "The kitchen sees what to cook now — and what can wait."}
                            kds := View{
                                width: Fill height: Fit
                                flow: Right
                                spacing: 12
                                KdsColumn{
                                    title.text: "New"
                                    KdsCard{head.text: "KOT 31 · T6 · 1 min" line.text: "2 Veg biryani · 1 Raita"}
                                    KdsCard{head.text: "KOT 30 · T2 · 3 min" line.text: "3 Masala dosa"}
                                }
                                KdsColumn{
                                    title.text: "Preparing"
                                    KdsCard{draw_bg.color: #xFFF4CC head.text: "KOT 27 · T4 · 9 min" line.text: "1 Paneer tikka · 4 Butter naan"}
                                    KdsCard{draw_bg.color: #xFFC48A head.text: "KOT 24 · T1 · 17 min" line.text: "2 Dal makhani — mains held"}
                                }
                                KdsColumn{
                                    title.text: "Ready"
                                    KdsCard{draw_bg.color: #xBFEBC8 head.text: "KOT 22 · T3 · 21 min" line.text: "2 Gulab jamun"}
                                }
                            }
                            kds_points := View{
                                width: Fill height: Fit
                                flow: Right
                                spacing: 20
                                Point{head.text: "New → Preparing → Ready" line.text: "One tap on a ticket moves it on; older tickets warm in colour."}
                                Point{head.text: "To cook now" line.text: "All-day counts: how many of each dish are waiting."}
                                Point{head.text: "Hold and fire" line.text: "Mains wait until the starters go; recall a ticket bumped too soon."}
                            }
                        }

                        // 5 — Service changes (move-merge, kot-cancel, nc)
                        changes := Chapter{
                            Eyebrow{text: "5 · SERVICE CHANGES"}
                            Headline{text: "Move, merge, cancel or comp — never silently."}
                            changes_grid := View{
                                width: Fill height: Fit
                                flow: Right
                                spacing: 16
                                WarmCard{
                                    Strong{text: "Move"}
                                    Body{text: "Guests change tables; the order and its minutes go with them."}
                                }
                                WarmCard{
                                    Strong{text: "Merge"}
                                    Body{text: "Two tables, one bill."}
                                }
                                WarmCard{
                                    Strong{text: "Cancel after the KOT"}
                                    Body{text: "With a reason, cooked or not, and who approved it. The kitchen gets a cancel slip."}
                                }
                                WarmCard{
                                    Strong{text: "Complimentary"}
                                    Body{text: "The dish stays on the bill at ₹ 0, with the reason and the approver."}
                                }
                            }
                            changes_note := Small{text: "Who, when and why is kept for every change — the day report lists cancels and comps."}
                        }

                        // 6 — Settle (split, settle, upi-qr, due-khata, free-on-pay)
                        settle := Chapter{
                            Eyebrow{text: "6 · SETTLE"}
                            Headline{text: "Cash, UPI, part, Due, tip — and the table frees only when it's paid."}
                            settle_row := View{
                                width: Fill height: Fit
                                flow: Right
                                spacing: 20
                                bill := WarmCard{
                                    spacing: 8
                                    View{
                                        width: Fill height: Fit
                                        flow: Right
                                        spacing: 10
                                        align: Align{y: 0.5}
                                        RoundedView{
                                            width: Fit height: Fit
                                            padding: Inset{left: 10. right: 10. top: 4. bottom: 5.}
                                            draw_bg +: {color: BillFill border_color: BillLine border_size: 1.5 border_radius: 8.}
                                            Label{padding: 0. text: "T4 · Bill printed" draw_text +: {color: Ink text_style: theme.font_bold{font_family: StrongFont font_size: 13.}}}
                                        }
                                        View{width: Fill height: 1}
                                        ExampleTag{}
                                    }
                                    View{
                                        width: Fill height: Fit
                                        flow: Right
                                        Body{width: Fill text: "Bill"}
                                        Ticket{width: Fit text: "₹ 1,240"}
                                    }
                                    View{
                                        width: Fill height: Fit
                                        flow: Right
                                        Body{width: Fill text: "Customer paid (cash)"}
                                        Ticket{width: Fit text: "₹ 1,500"}
                                    }
                                    View{
                                        width: Fill height: Fit
                                        flow: Right
                                        Strong{width: Fill text: "Return to customer"}
                                        Ticket{width: Fit text: "₹ 260"}
                                    }
                                    RoundedView{width: Fill height: 1 draw_bg +: {color: WellLine border_radius: 0.}}
                                    Small{text: "A tip is recorded apart from sales, never mixed into the bill."}
                                }
                                settle_points := View{
                                    width: Fill height: Fit
                                    flow: Down
                                    spacing: 14
                                    Point{head.text: "Split the bill" line.text: "By item, by seat, or evenly — each part settles on its own."}
                                    Point{head.text: "UPI QR with the amount" line.text: "The customer scans; the amount is already in it."}
                                    Point{head.text: "Part payment and Due" line.text: "What's left goes on the customer's khata in Parties."}
                                    // The headline says it too; short windows drop this one.
                                    settle_free := Point{head.text: "Free on pay" line.text: "A table with money still due cannot be freed."}
                                }
                            }
                        }

                        // 7 — Close the day (shift-close, day-report)
                        dayclose := Chapter{
                            Eyebrow{text: "7 · CLOSE THE DAY"}
                            Headline{text: "Day end you can trust: expected, counted, and the difference named."}
                            close_row := View{
                                width: Fill height: Fit
                                flow: Right
                                spacing: 20
                                drawer := WarmCard{
                                    spacing: 6
                                    View{
                                        width: Fill height: Fit
                                        flow: Right
                                        align: Align{y: 0.5}
                                        Strong{width: Fill text: "Cash drawer, shift close"}
                                        ExampleTag{}
                                    }
                                    View{width: Fill height: Fit flow: Right Body{width: Fill text: "Opening float"} Ticket{width: Fit text: "  2,000"}}
                                    View{width: Fill height: Fit flow: Right Body{width: Fill text: "+ cash taken"} Ticket{width: Fit text: " 18,450"}}
                                    View{width: Fill height: Fit flow: Right Body{width: Fill text: "+ cash tips"} Ticket{width: Fit text: "    300"}}
                                    View{width: Fill height: Fit flow: Right Body{width: Fill text: "− cash paid out"} Ticket{width: Fit text: "    600"}}
                                    RoundedView{width: Fill height: 1 draw_bg +: {color: WellLine border_radius: 0.}}
                                    View{width: Fill height: Fit flow: Right Strong{width: Fill text: "Expected in the drawer"} Ticket{width: Fit text: "₹ 20,150"}}
                                    View{width: Fill height: Fit flow: Right Strong{width: Fill text: "Counted"} Ticket{width: Fit text: "₹ 20,050"}}
                                    View{width: Fill height: Fit flow: Right Strong{width: Fill draw_text.color: Red text: "Short"} Ticket{width: Fit draw_text.color: Red text: "₹ 100"}}
                                }
                                close_points := View{
                                    width: Fill height: Fit
                                    flow: Down
                                    spacing: 14
                                    Point{head.text: "Shift handover slip" line.text: "Close one shift, print the slip, and the next one starts clean."}
                                    Point{head.text: "The day report" line.text: "By staff, by order type, by hour and by dish — with cancels, comps and tips."}
                                }
                            }
                        }

                        // 8 — Your menu (aliases-short-codes, area-prices, menu-timings, languages)
                        menu := Chapter{
                            Eyebrow{text: "8 · YOUR MENU, YOUR WAY"}
                            Headline{text: "Your menu in your words — with short codes, area prices and timed prices."}
                            menu_row := View{
                                width: Fill height: Fit
                                flow: Right
                                spacing: 16
                                WarmCard{
                                    spacing: 4
                                    Label{padding: 0. text: "પનીર ટિક્કા" draw_text +: {color: Maroon text_style: theme.font_bold{font_family: DisplayFont font_size: 26.}}}
                                    Strong{text: "Paneer tikka · code PT"}
                                    Body{text: "AC room ₹ 220 · everywhere else ₹ 200"}
                                }
                                WarmCard{
                                    spacing: 4
                                    Label{padding: 0. text: "મસાલા છાશ" draw_text +: {color: Maroon text_style: theme.font_bold{font_family: DisplayFont font_size: 26.}}}
                                    Strong{text: "Masala chaas · code MC"}
                                    Body{text: "Happy hour 17:00–19:00 ₹ 40 · otherwise ₹ 60"}
                                }
                                WarmCard{
                                    spacing: 4
                                    Label{padding: 0. text: "पोहा" draw_text +: {color: Maroon text_style: theme.font_bold{font_family: DisplayFont font_size: 26.}}}
                                    Strong{text: "Poha · code PO"}
                                    Body{text: "On the menu Mon–Fri, 07:00–11:00"}
                                }
                            }
                            menu_note := Small{text: "Type a short code or any name — Gujarati, Hindi or English — and the dish comes first. Example menu; 29 languages on screen."}
                        }

                        // 9 — Internet down (offline-lan; the multi-device half is gated)
                        offline := Chapter{
                            Eyebrow{text: "9 · INTERNET DOWN?"}
                            Headline{text: "Service goes on."}
                            Lede{text: "Orders, bills and the day book live on your own devices. When the internet drops in the middle of lunch, nothing in the restaurant waits for it."}
                            offline_card := WarmCard{
                                spacing: 12
                                View{
                                    width: Fill height: Fit
                                    flow: Right
                                    spacing: 10
                                    align: Align{y: 0.5}
                                    RoundedView{width: 12 height: 12 draw_bg +: {color: Red border_radius: 6.}}
                                    Strong{width: Fill text: "Router: no internet"}
                                }
                                View{
                                    width: Fill height: Fit
                                    flow: Right
                                    spacing: 10
                                    align: Align{y: 0.5}
                                    RoundedView{width: 12 height: 12 draw_bg +: {color: BillLine border_radius: 6.}}
                                    Strong{width: Fill text: "Counter: taking orders and settling bills"}
                                }
                                // Shown only after the owner's real-hardware LAN pilot passes.
                                offline_lan := View{
                                    visible: false
                                    width: Fill height: Fit
                                    flow: Right
                                    spacing: 10
                                    align: Align{y: 0.5}
                                    RoundedView{width: 12 height: 12 draw_bg +: {color: BillLine border_radius: 6.}}
                                    Strong{width: Fill text: "Captains' tablets and the kitchen screen: working together on your own Wi‑Fi"}
                                }
                            }
                        }

                        // 10 — Start (free-local)
                        start := Chapter{
                            Eyebrow{text: "10 · START"}
                            Headline{text: "Free forever on your devices."}
                            Lede{text: "No charge per table, waiter, order or device. Khaynjo runs on Android, Windows and Linux."}
                            close_cta := CtaPair{}
                            RoundedView{width: Fill height: 1 draw_bg +: {color: WellLine border_radius: 0.}}
                            footer := View{
                                width: Fill height: Fit
                                flow: Right
                                spacing: 12
                                align: Align{y: 0.5}
                                Small{width: Fill text: "Khaynjo is a Kumkaj product by Gotigin."}
                                to_kumkaj := SecondaryBtn{text: "kumkaj.com  →"}
                            }
                        }
                    }
                }
            }
        }
    }
}
