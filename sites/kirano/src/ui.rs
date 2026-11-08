// kirano.shop — one Makepad 2 `script_mod!` (included from main.rs).
// Story: apps/kirano/docs/KIRANO_WEBSITE.md §2 (Phase 1 grocery counter).
// Palette: docs/DESIGN.md "Color tokens — Kirano".
// Copied from sites/khaynjo and diverged: warm cream ground, orange chrome,
// leaf-green accents, rounded Baloo type. No Floor / Kitchen / KOT / tables.
script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    // ---- Tokens (Kirano icon / DESIGN.md) ---------------------------
    let Cream = #xFDF7EA
    let Well = #xFFFFFF
    let WellWarm = #xFCE1B4
    let WellDeep = #xF5D090
    let WellLine = #xE8CFA0
    let Ink = #x031303
    let Muted = #x1C460E
    let Hint = #x56575D
    let Orange = #xCD380E
    let OrangeHover = #xF96303
    let OrangeDeep = #xA82E0A
    let Accent = #xF96303
    let Leaf = #x1C460E
    let LeafSoft = #xA7A536
    let Paper = #xFFFDF8
    let BillFill = #xE8F5D8
    let BillLine = #x1C460E

    // ---- Type: Baloo 2, IBM Plex, Liberation Mono --------------------
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
            color: Leaf
            text_style: theme.font_bold{font_family: DisplayFont font_size: 48. line_spacing: 0.72}
        }
    }
    let Lede = Label{
        width: Fill
        padding: 0.
        draw_text +: {
            color: Hint
            text_style: theme.font_regular{font_family: TextFont font_size: 19. line_spacing: 1.45}
        }
    }
    let Body = Label{
        width: Fill
        padding: 0.
        draw_text +: {
            color: Hint
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
                return vec4(vec3(0.012, 0.075, 0.012) * a, a)
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

    let PrimaryBtn = Button{
        width: Fit height: Fit
        padding: Inset{left: 22. right: 22. top: 14. bottom: 14.}
        align: Align{x: 0.5 y: 0.5}
        grab_key_focus: false
        reset_hover_on_click: true
        label_walk: Walk{width: Fit height: Fit margin: Inset{left: 2. right: 2.}}
        icon_walk: Walk{width: 0. height: 0.}
        draw_bg +: {
            color: Orange
            color_hover: OrangeHover
            color_down: OrangeDeep
            color_focus: Orange
            color_2: Orange
            color_2_hover: OrangeHover
            color_2_down: OrangeDeep
            color_2_focus: Orange
            border_color: Orange
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
            color_down: #xE8C070
            color_focus: WellDeep
            color_2: WellDeep
            color_2_hover: WellWarm
            color_2_down: #xE8C070
            color_2_focus: WellDeep
            border_color: #xD9B878
            border_color_hover: #xD9B878
            border_color_down: #xD9B878
            border_color_focus: #xD9B878
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
                let base = mix(vec3(0.992, 0.969, 0.918), vec3(0.973, 0.925, 0.820), self.pos.y)
                let warm = mix(base, vec3(0.988, 0.882, 0.706), lamp * 0.45)
                let y = (1.0 - self.pos.y) * self.rect_size.y
                let runner = 1.0 - step(5.0, y)
                return vec4(mix(warm, vec3(0.804, 0.220, 0.055), runner), 1.0)
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
            draw_text +: {color: Hint text_style: theme.font_regular{font_family: TextFont font_size: 12.}}
        }
    }
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
                return vec4(vec3(0.655, 0.647, 0.212) * a, a)
            }
        }
        head := Strong{text: "Point"}
        line := Body{text: "Detail"}
    }

    let BillLine = View{
        width: Fill height: Fit
        flow: Right
        spacing: 8
        name := Body{width: Fill text: "Item"}
        amt := Ticket{width: Fit text: "₹ 0"}
    }

    let NavTile = View{
        width: 22 height: 26
        flow: Overlay
        align: Align{x: 0.5 y: 0.5}
        idle := RoundedView{width: 14 height: 14 draw_bg +: {color: #xFFFFFF border_color: #xE8CFA0 border_size: 1.5 border_radius: 4.}}
        here := RoundedView{visible: false width: 20 height: 20 draw_bg +: {color: #xFFFFFF border_color: LeafSoft border_size: 3. border_radius: 5.}}
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
                window.title: "Kirano — busy counter, no internet, still billing"
                window.inner_size: vec2(1280, 800)
                pass.clear_color: Cream
                body +: {
                    width: Fill
                    height: Fill
                    flow: Down

                    top_bar := RoundedView{
                        width: Fill height: 58
                        flow: Right
                        spacing: 12
                        align: Align{y: 0.5}
                        padding: Inset{left: 24. right: 20. top: 0. bottom: 0.}
                        draw_bg +: {color: Orange border_radius: 0.}
                        Label{
                            padding: 0.
                            text: "Kirano"
                            draw_text +: {color: Cream text_style: theme.font_bold{font_family: DisplayFont font_size: 25.}}
                        }
                        tagline := Label{
                            padding: 0.
                            text: "the grocery counter"
                            draw_text +: {color: WellWarm text_style: theme.font_regular{font_family: TextFont font_size: 13.}}
                        }
                        View{width: Fill height: 1}
                        chapter_name := Label{
                            padding: 0.
                            text: "Busy counter"
                            draw_text +: {color: #xFFE8C8 text_style: theme.font_regular{font_family: TextFont font_size: 13.}}
                        }
                        nav := View{
                            width: Fit height: Fit
                            flow: Right
                            spacing: 2
                            align: Align{y: 0.5}
                            n0 := NavTile{}
                            n1 := NavTile{idle.draw_bg.color: WellWarm here.draw_bg.color: WellWarm}
                            n2 := NavTile{idle.draw_bg.color: BillFill here.draw_bg.color: BillFill}
                            n3 := NavTile{idle.draw_bg.color: #xD4E8B8 here.draw_bg.color: #xD4E8B8}
                            n4 := NavTile{idle.draw_bg.color: LeafSoft here.draw_bg.color: LeafSoft}
                            n5 := NavTile{}
                            n6 := NavTile{}
                            n7 := NavTile{}
                        }
                        counter := Label{
                            padding: 0.
                            text: "1 / 8"
                            draw_text +: {color: #xFFE8C8 text_style: theme.font_bold{font_family: StrongFont font_size: 12.5}}
                        }
                    }

                    slides := SlidesView{
                        width: Fill
                        height: Fill
                        anim_speed: 0.86
                        axis: Vertical

                        // 1 — Brand + promise + counter bill toy
                        brand := Chapter{
                            brand_row := View{
                                width: Fill height: Fit
                                flow: Right
                                spacing: 24
                                align: Align{y: 0.5}
                                View{
                                    width: Fill height: Fit
                                    flow: Down
                                    spacing: 18
                                    Eyebrow{text: "KIRANO · THE GROCERY COUNTER"}
                                    Headline{text: "Busy counter. No internet. Still billing."}
                                    brand_lede := Lede{text: "Sell, khata, stock and the day book — on your own devices. Books · Sell · Buy · Items. No floor, no kitchen, no tables."}
                                    brand_cta := CtaPair{}
                                }
                                bill_card := WarmCard{
                                    width: 420
                                    spacing: 12
                                    View{
                                        width: Fill height: Fit
                                        flow: Right
                                        align: Align{y: 0.5}
                                        Strong{width: Fill text: "Tap the bill to walk a counter sale"}
                                        ExampleTag{}
                                    }
                                    bill := View{
                                        width: Fill height: Fit
                                        flow: Overlay
                                        RoundedView{
                                            width: Fill height: Fit
                                            flow: Down
                                            spacing: 8
                                            padding: Inset{left: 16. right: 16. top: 14. bottom: 14.}
                                            draw_bg +: {color: Paper border_color: WellLine border_size: 1. border_radius: 12.}
                                            View{
                                                width: Fill height: Fit
                                                flow: Right
                                                align: Align{y: 0.5}
                                                Strong{width: Fill text: "Cash memo"}
                                                RoundedView{
                                                    width: Fit height: Fit
                                                    padding: Inset{left: 8. right: 8. top: 3. bottom: 4.}
                                                    draw_bg +: {color: BillFill border_color: BillLine border_size: 1. border_radius: 8.}
                                                    status := Label{padding: 0. text: "Ready" draw_text +: {color: Ink text_style: theme.font_bold{font_family: StrongFont font_size: 12.}}}
                                                }
                                            }
                                            line1 := BillLine{visible: false}
                                            line2 := BillLine{visible: false}
                                            line3 := BillLine{visible: false}
                                            total_row := View{
                                                visible: false
                                                width: Fill height: Fit
                                                flow: Down
                                                spacing: 6
                                                TicketRule{}
                                                View{
                                                    width: Fill height: Fit
                                                    flow: Right
                                                    Strong{width: Fill text: "Total"}
                                                    total_amt := Ticket{width: Fit text: "₹ 0"}
                                                }
                                            }
                                            tender := Small{text: "Tap to add the first item"}
                                        }
                                        hit := HitBtn{}
                                    }
                                    bill_note := Small{text: "Example grocery lines — not a live till. Same Sell → settle → day book path as Kirano."}
                                }
                            }
                        }

                        // 2 — Counter sell in three taps
                        sell := Chapter{
                            Eyebrow{text: "2 · COUNTER SELL"}
                            Headline{text: "Item, quantity, pay — three taps."}
                            sell_row := View{
                                width: Fill height: Fit
                                flow: Right
                                spacing: 28
                                align: Align{y: 0.5}
                                ticket := RoundedView{
                                    width: 330 height: Fit
                                    flow: Down
                                    spacing: 4
                                    padding: Inset{left: 20. right: 20. top: 18. bottom: 20.}
                                    draw_bg +: {color: Paper border_color: WellLine border_size: 1. border_radius: 4.}
                                    Ticket{text: "SELL · COUNTER"}
                                    Ticket{text: "Cash memo · 12:42"}
                                    TicketRule{}
                                    Ticket{text: "1  Atta 5 kg          280"}
                                    Ticket{text: "1  Sunflower oil 1 L  145"}
                                    Ticket{text: "1  Toor dal 1 kg      160"}
                                    TicketRule{}
                                    Ticket{text: "Total                585"}
                                    Ticket{text: "UPI QR ready"}
                                    TicketRule{}
                                    Small{text: "Example bill"}
                                }
                                sell_points := View{
                                    width: Fill height: Fit
                                    flow: Down
                                    spacing: 16
                                    Point{head.text: "Search by name or short code" line.text: "Type atta, AT5 or an alias — the line lands on the bill."}
                                    Point{head.text: "kg, g, pc, L, ml" line.text: "Quantity in the unit the packet uses; price follows."}
                                    Point{head.text: "Cash, UPI or split" line.text: "Settle on the counter; the day book records the tender."}
                                }
                            }
                        }

                        // 3 — Khata
                        khata := Chapter{
                            Eyebrow{text: "3 · KHATA"}
                            Headline{text: "Credit without a CRM — put it on their khata."}
                            khata_row := View{
                                width: Fill height: Fit
                                flow: Right
                                spacing: 20
                                WarmCard{
                                    spacing: 8
                                    View{
                                        width: Fill height: Fit
                                        flow: Right
                                        align: Align{y: 0.5}
                                        Strong{width: Fill text: "Ramesh · Customer"}
                                        ExampleTag{}
                                    }
                                    View{width: Fill height: Fit flow: Right Body{width: Fill text: "Due from earlier"} Ticket{width: Fit text: "₹ 1,240"}}
                                    View{width: Fill height: Fit flow: Right Body{width: Fill text: "+ today's sale (Due)"} Ticket{width: Fit text: "₹ 585"}}
                                    RoundedView{width: Fill height: 1 draw_bg +: {color: WellLine border_radius: 0.}}
                                    View{width: Fill height: Fit flow: Right Strong{width: Fill text: "Balance"} Ticket{width: Fit text: "₹ 1,825"}}
                                    Small{text: "Settle later in Parties — part cash, part UPI, clear the line."}
                                }
                                khata_points := View{
                                    width: Fill height: Fit
                                    flow: Down
                                    spacing: 16
                                    Point{head.text: "Due on the bill" line.text: "What's left goes on the customer's khata — no separate ledger app."}
                                    Point{head.text: "One party, two roles" line.text: "Customer AR and supplier AP share the same party book."}
                                    Point{head.text: "Who, when, how much" line.text: "Every credit and settlement stays in the day book."}
                                }
                            }
                        }

                        // 4 — Buy → stock
                        buy := Chapter{
                            Eyebrow{text: "4 · BUY & STOCK"}
                            Headline{text: "Buy from the wholesaler — stock and who you owe update together."}
                            buy_row := View{
                                width: Fill height: Fit
                                flow: Right
                                spacing: 20
                                WarmCard{
                                    spacing: 8
                                    View{
                                        width: Fill height: Fit
                                        flow: Right
                                        align: Align{y: 0.5}
                                        Strong{width: Fill text: "Purchase · Sharma Wholesale"}
                                        ExampleTag{}
                                    }
                                    Strong{text: "Toor dal 50 kg"}
                                    Body{text: "Stock +50 kg · AP ₹ 7,200"}
                                    Strong{text: "Sunflower oil 12 × 1 L"}
                                    Body{text: "Stock +12 L · AP ₹ 1,560"}
                                    RoundedView{width: Fill height: 1 draw_bg +: {color: WellLine border_radius: 0.}}
                                    Small{text: "Resale stock — not kitchen raw. Optional batch and expiry on the buy."}
                                }
                                buy_points := View{
                                    width: Fill height: Fit
                                    flow: Down
                                    spacing: 16
                                    Point{head.text: "Buy creates stock" line.text: "Receive the goods; the catalogue quantity rises."}
                                    Point{head.text: "Who we owe" line.text: "Supplier balance sits on the same party as customer khata."}
                                    Point{head.text: "Low stock at a glance" line.text: "Inventory shows what is running out before the next rush."}
                                }
                            }
                        }

                        // 5 — Day book
                        daybook := Chapter{
                            Eyebrow{text: "5 · DAY BOOK"}
                            Headline{text: "Cash vs UPI, counted — and the difference named."}
                            day_row := View{
                                width: Fill height: Fit
                                flow: Right
                                spacing: 20
                                WarmCard{
                                    spacing: 6
                                    View{
                                        width: Fill height: Fit
                                        flow: Right
                                        align: Align{y: 0.5}
                                        Strong{width: Fill text: "Today's drawer"}
                                        ExampleTag{}
                                    }
                                    View{width: Fill height: Fit flow: Right Body{width: Fill text: "Opening float"} Ticket{width: Fit text: "  2,000"}}
                                    View{width: Fill height: Fit flow: Right Body{width: Fill text: "+ cash sales"} Ticket{width: Fit text: " 12,450"}}
                                    View{width: Fill height: Fit flow: Right Body{width: Fill text: "+ UPI sales"} Ticket{width: Fit text: "  8,320"}}
                                    View{width: Fill height: Fit flow: Right Body{width: Fill text: "− cash paid out"} Ticket{width: Fit text: "    600"}}
                                    RoundedView{width: Fill height: 1 draw_bg +: {color: WellLine border_radius: 0.}}
                                    View{width: Fill height: Fit flow: Right Strong{width: Fill text: "Expected cash"} Ticket{width: Fit text: "₹ 13,850"}}
                                    View{width: Fill height: Fit flow: Right Strong{width: Fill text: "Counted"} Ticket{width: Fit text: "₹ 13,800"}}
                                    View{width: Fill height: Fit flow: Right Strong{width: Fill draw_text.color: Orange text: "Short"} Ticket{width: Fit draw_text.color: Orange text: "₹ 50"}}
                                }
                                View{
                                    width: Fill height: Fit
                                    flow: Down
                                    spacing: 14
                                    Point{head.text: "Tenders stay separate" line.text: "Cash and UPI never mix into one mystery total."}
                                    Point{head.text: "Shift close" line.text: "Close the shift, print the slip, start the next one clean."}
                                    day_extra := Point{head.text: "GST-facing bills" line.text: "Tax invoice or cash memo — the day report carries both."}
                                    day_note := Small{text: "Books · Sell · Buy · Items — the counter chrome, not a restaurant floor."}
                                }
                            }
                        }

                        // 6 — Extra till on shop Wi‑Fi (LAN gated)
                        lan := Chapter{
                            Eyebrow{text: "6 · EXTRA TILL"}
                            Headline{text: "Another till on the shop Wi‑Fi."}
                            lan_row := View{
                                width: Fill height: Fit
                                flow: Right
                                spacing: 20
                                WarmCard{
                                    spacing: 12
                                    View{
                                        width: Fill height: Fit
                                        flow: Right
                                        spacing: 10
                                        align: Align{y: 0.5}
                                        RoundedView{width: 12 height: 12 draw_bg +: {color: Orange border_radius: 6.}}
                                        Strong{width: Fill text: "Router: no internet"}
                                    }
                                    View{
                                        width: Fill height: Fit
                                        flow: Right
                                        spacing: 10
                                        align: Align{y: 0.5}
                                        RoundedView{width: 12 height: 12 draw_bg +: {color: BillLine border_radius: 6.}}
                                        Strong{width: Fill text: "Main counter: selling and settling"}
                                    }
                                    lan_devices := View{
                                        visible: false
                                        width: Fill height: Fit
                                        flow: Right
                                        spacing: 10
                                        align: Align{y: 0.5}
                                        RoundedView{width: 12 height: 12 draw_bg +: {color: BillLine border_radius: 6.}}
                                        Strong{width: Fill text: "Second till: joined on your own Wi‑Fi"}
                                    }
                                }
                                View{
                                    width: Fill height: Fit
                                    flow: Down
                                    spacing: 14
                                    Point{head.text: "Books live on the shop devices" line.text: "When the internet drops mid-rush, nothing at the counter waits for it."}
                                    lan_extra := Point{visible: false head.text: "One Server, LAN Clients" line.text: "Extra tills share the same books over the shop network."}
                                    lan_note := Small{text: "Multi-device lines show after the owner's LAN pilot."}
                                }
                            }
                        }

                        // 7 — Free forever
                        free := Chapter{
                            Eyebrow{text: "7 · FREE FOREVER LOCALLY"}
                            Headline{text: "Free forever on your devices."}
                            free_row := View{
                                width: Fill height: Fit
                                flow: Right
                                spacing: 20
                                Point{head.text: "No per-bill charge" line.text: "No fee per sale, item, till or staff PIN on the local install."}
                                Point{head.text: "Android, Windows, Linux" line.text: "The same Kirano on the phone behind the counter and the desktop till."}
                                Point{head.text: "Cloud later, by usage" line.text: "Optional Kumkaj Cloud when you want it — billed for what you use."}
                            }
                        }

                        // 8 — Start / download
                        start := Chapter{
                            Eyebrow{text: "8 · START"}
                            Headline{text: "Put Kirano on the counter."}
                            Lede{text: "Download free, or WhatsApp for a walk-through with your shop type and city."}
                            close_cta := CtaPair{}
                            RoundedView{width: Fill height: 1 draw_bg +: {color: WellLine border_radius: 0.}}
                            footer := View{
                                width: Fill height: Fit
                                flow: Right
                                spacing: 12
                                align: Align{y: 0.5}
                                Small{width: Fill text: "Kirano is a Kumkaj product by Gotigin. · kirano.shop"}
                                to_kumkaj := SecondaryBtn{text: "kumkaj.com  →"}
                            }
                        }
                    }
                }
            }
        }
    }
}
