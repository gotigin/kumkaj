// tabiyat.shop — one Makepad 2 `script_mod!` (included from main.rs).
// Story: apps/tabiyat/docs/TABIYAT_WEBSITE.md (Phase 1 pharmacy counter).
// Palette: docs/DESIGN.md "Color tokens — Tabiyat".
// Copied from sites/kirano / sites/khaynjo and diverged: cool white ground,
// teal chrome, lime accents. No Floor / Kitchen / KOT / tables.
// Never claims Drugs Act compliance — owner bookkeeping only.
script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    // ---- Tokens (Tabiyat icon / DESIGN.md) ---------------------------
    let Cream = #xFBFCFC
    let Well = #xFFFFFF
    let WellWarm = #xE8F4F4
    let WellDeep = #xA2CFCF
    let WellLine = #xC5DEDE
    let Ink = #x023637
    let Muted = #x025B60
    let Hint = #x074D3E
    let Teal = #x023637
    let TealMid = #x025B60
    let TealBright = #x017B85
    let Accent = #x017B85
    let Lime = #x37BE28
    let Paper = #xFFFFFF
    let BillFill = #xE8F8E6
    let BillLine = #x074D3E

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

    let Eyebrow = Markdown{
        selectable: true
        height: Fit
        paragraph_spacing: 0.
        pre_code_spacing: 0.
        draw_block +: { selection_color: #x37BE28FF }
        width: Fill
        padding: 0.
        draw_text +: {
            color: Accent
            text_style: theme.font_bold{font_family: StrongFont font_size: 13. line_spacing: 1.2}
        }
    }
    let Headline = Markdown{
        selectable: true
        height: Fit
        paragraph_spacing: 0.
        pre_code_spacing: 0.
        draw_block +: { selection_color: #x37BE28FF }
        width: Fill
        padding: 0.
        draw_text +: {
            color: Teal
            text_style: theme.font_bold{font_family: DisplayFont font_size: 48. line_spacing: 0.72}
        }
    }
    let Lede = Markdown{
        selectable: true
        height: Fit
        paragraph_spacing: 0.
        pre_code_spacing: 0.
        draw_block +: { selection_color: #x37BE28FF }
        width: Fill
        padding: 0.
        draw_text +: {
            color: Hint
            text_style: theme.font_regular{font_family: TextFont font_size: 19. line_spacing: 1.45}
        }
    }
    let Body = Markdown{
        selectable: true
        height: Fit
        paragraph_spacing: 0.
        pre_code_spacing: 0.
        draw_block +: { selection_color: #x37BE28FF }
        width: Fill
        padding: 0.
        draw_text +: {
            color: Hint
            text_style: theme.font_regular{font_family: TextFont font_size: 15.5 line_spacing: 1.45}
        }
    }
    let Strong = Markdown{
        selectable: true
        height: Fit
        paragraph_spacing: 0.
        pre_code_spacing: 0.
        draw_block +: { selection_color: #x37BE28FF }
        width: Fill
        padding: 0.
        draw_text +: {
            color: Ink
            text_style: theme.font_bold{font_family: StrongFont font_size: 16. line_spacing: 1.25}
        }
    }
    let Small = Markdown{
        selectable: true
        height: Fit
        paragraph_spacing: 0.
        pre_code_spacing: 0.
        draw_block +: { selection_color: #x37BE28FF }
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
                return vec4(vec3(0.008, 0.212, 0.216) * a, a)
            }
        }
    }
    let Ticket = Markdown{
        selectable: true
        height: Fit
        paragraph_spacing: 0.
        pre_code_spacing: 0.
        draw_block +: { selection_color: #x37BE28FF }
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
            color: Teal
            color_hover: TealMid
            color_down: TealBright
            color_focus: Teal
            color_2: Teal
            color_2_hover: TealMid
            color_2_down: TealBright
            color_2_focus: Teal
            border_color: Teal
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
            color: WellWarm
            color_hover: #xD5EBEB
            color_down: WellDeep
            color_focus: WellWarm
            color_2: WellWarm
            color_2_hover: #xD5EBEB
            color_2_down: WellDeep
            color_2_focus: WellWarm
            border_color: WellLine
            border_color_hover: WellLine
            border_color_down: WellLine
            border_color_focus: WellLine
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
                let base = mix(vec3(0.984, 0.988, 0.988), vec3(0.910, 0.957, 0.957), self.pos.y)
                let cool = mix(base, vec3(0.851, 0.945, 0.945), lamp * 0.40)
                let y = (1.0 - self.pos.y) * self.rect_size.y
                let runner = 1.0 - step(5.0, y)
                return vec4(mix(cool, vec3(0.008, 0.212, 0.216), runner), 1.0)
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
        Markdown{
            selectable: true
            height: Fit
            paragraph_spacing: 0.
            pre_code_spacing: 0.
            draw_block +: { selection_color: #x37BE28FF }

            padding: 0.
            body: "Example"
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
                return vec4(vec3(0.216, 0.745, 0.157) * a, a)
            }
        }
        head := Strong{body: "Point"}
        line := Body{body: "Detail"}
    }

    let BillLine = View{
        width: Fill height: Fit
        flow: Right
        spacing: 8
        name := Body{width: Fill body: "Item"}
        amt := Ticket{width: Fit body: "₹ 0"}
    }

    let NavTile = View{
        width: 22 height: 26
        flow: Overlay
        align: Align{x: 0.5 y: 0.5}
        idle := RoundedView{width: 14 height: 14 draw_bg +: {color: #xFFFFFF border_color: #xC5DEDE border_size: 1.5 border_radius: 4.}}
        here := RoundedView{visible: false width: 20 height: 20 draw_bg +: {color: #xFFFFFF border_color: Lime border_size: 3. border_radius: 5.}}
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
        note := Small{body: "Free on Android, Windows and Linux."}
    }

    startup() do #(App::script_component(vm)){
        ui: Root{
            main_window := Window{
                window.title: "Tabiyat — pharmacy counter, offline, your books"
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
                        draw_bg +: {color: Teal border_radius: 0.}
                        Markdown{
            selectable: true
            height: Fit
            paragraph_spacing: 0.
            pre_code_spacing: 0.
            draw_block +: { selection_color: #x37BE28FF }

                            padding: 0.
                            body: "Tabiyat"
                            draw_text +: {color: Cream text_style: theme.font_bold{font_family: DisplayFont font_size: 25.}}
                        
        }
                        tagline := Markdown{
            selectable: true
            height: Fit
            paragraph_spacing: 0.
            pre_code_spacing: 0.
            draw_block +: { selection_color: #x37BE28FF }

                            padding: 0.
                            body: "the pharmacy counter"
                            draw_text +: {color: Lime text_style: theme.font_regular{font_family: TextFont font_size: 13.}}
                        
        }
                        View{width: Fill height: 1}
                        chapter_name := Markdown{
            selectable: true
            height: Fit
            paragraph_spacing: 0.
            pre_code_spacing: 0.
            draw_block +: { selection_color: #x37BE28FF }

                            padding: 0.
                            body: "Pharmacy counter"
                            draw_text +: {color: #xB8E4E6 text_style: theme.font_regular{font_family: TextFont font_size: 13.}}
                        
        }
                        nav := View{
                            width: Fit height: Fit
                            flow: Right
                            spacing: 2
                            align: Align{y: 0.5}
                            n0 := NavTile{}
                            n1 := NavTile{idle.draw_bg.color: WellWarm here.draw_bg.color: WellWarm}
                            n2 := NavTile{idle.draw_bg.color: BillFill here.draw_bg.color: BillFill}
                            n3 := NavTile{idle.draw_bg.color: WellDeep here.draw_bg.color: WellDeep}
                            n4 := NavTile{idle.draw_bg.color: Lime here.draw_bg.color: Lime}
                            n5 := NavTile{}
                            n6 := NavTile{}
                            n7 := NavTile{}
                        }
                        counter := Markdown{
            selectable: true
            height: Fit
            paragraph_spacing: 0.
            pre_code_spacing: 0.
            draw_block +: { selection_color: #x37BE28FF }

                            padding: 0.
                            body: "1 / 8"
                            draw_text +: {color: #xB8E4E6 text_style: theme.font_bold{font_family: StrongFont font_size: 12.5}}
                        
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
                                    Eyebrow{body: "TABIYAT · THE PHARMACY COUNTER"}
                                    Headline{body: "Pharmacy counter. Offline. Your books."}
                                    brand_lede := Lede{body: "Medicines, barcode and salt search, batch and expiry for your own books — on your devices. Owner flags only; never fake Drugs Act compliance."}
                                    brand_cta := CtaPair{}
                                }
                                bill_card := WarmCard{
                                    width: 420
                                    spacing: 12
                                    View{
                                        width: Fill height: Fit
                                        flow: Right
                                        align: Align{y: 0.5}
                                        Strong{width: Fill body: "Tap the bill to walk a counter sale"}
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
                                                Strong{width: Fill body: "Cash memo"}
                                                RoundedView{
                                                    width: Fit height: Fit
                                                    padding: Inset{left: 8. right: 8. top: 3. bottom: 4.}
                                                    draw_bg +: {color: BillFill border_color: BillLine border_size: 1. border_radius: 8.}
                                                    status := Markdown{padding: 0. selectable: true height: Fit paragraph_spacing: 0. pre_code_spacing: 0. body: "Ready" font_color: Ink draw_text +: {color: Ink text_style: theme.font_bold{font_family: StrongFont font_size: 12.}} draw_block +: { selection_color: #x37BE28FF }}
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
                                                    Strong{width: Fill body: "Total"}
                                                    total_amt := Ticket{width: Fit body: "₹ 0"}
                                                }
                                            }
                                            tender := Small{body: "Tap to add the first medicine"}
                                        }
                                        hit := HitBtn{}
                                    }
                                    bill_note := Small{body: "Example pharmacy lines — not a live till. Sell → settle → day book."}
                                }
                            }
                        }

                        // 2 — Three-tap sell
                        sell := Chapter{
                            Eyebrow{body: "2 · COUNTER SELL"}
                            Headline{body: "Medicine, quantity, pay — three taps."}
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
                                    Ticket{body: "SELL · COUNTER"}
                                    Ticket{body: "Cash memo · 12:42"}
                                    TicketRule{}
                                    Ticket{body: "1  Paracetamol 500     28"}
                                    Ticket{body: "1  Amoxicillin 250     86"}
                                    Ticket{body: "4  ORS sachets         48"}
                                    TicketRule{}
                                    Ticket{body: "Total               162"}
                                    Ticket{body: "UPI QR ready"}
                                    TicketRule{}
                                    Small{body: "Example bill"}
                                }
                                sell_points := View{
                                    width: Fill height: Fit
                                    flow: Down
                                    spacing: 16
                                    Point{head.body: "Fast counter billing" line.body: "Add the line, set the qty, settle — cash, UPI or split."}
                                    Point{head.body: "GST Tax invoice or cash memo" line.body: "The bill lane you need for the sale."}
                                    Point{head.body: "Day book follows" line.body: "Every tender lands in Books when you close the sale."}
                                }
                            }
                        }

                        // 3 — Barcode / salt search
                        search := Chapter{
                            Eyebrow{body: "3 · BARCODE & SALT"}
                            Headline{body: "Scan the barcode — or find it by salt."}
                            search_row := View{
                                width: Fill height: Fit
                                flow: Right
                                spacing: 20
                                WarmCard{
                                    spacing: 8
                                    View{
                                        width: Fill height: Fit
                                        flow: Right
                                        align: Align{y: 0.5}
                                        Strong{width: Fill body: "Search"}
                                        ExampleTag{}
                                    }
                                    Strong{body: "8901030… · Paracetamol 500 mg"}
                                    Body{body: "Composition: Paracetamol · strip of 10"}
                                    Strong{body: "Salt: amoxicillin"}
                                    Body{body: "Ranks barcode, name and composition together."}
                                    RoundedView{width: Fill height: 1 draw_bg +: {color: WellLine border_radius: 0.}}
                                    Small{body: "Wedge scanner or type Enter — the line lands on the bill."}
                                }
                                search_points := View{
                                    width: Fill height: Fit
                                    flow: Down
                                    spacing: 16
                                    Point{head.body: "Barcode first" line.body: "Scan or paste the code; the catalogue match comes up."}
                                    Point{head.body: "Salt / composition" line.body: "Type the salt when the brand name is not what the customer said."}
                                    Point{head.body: "Owner Rx / Schedule H flags" line.body: "Reminders you set — not a statutory register, not medical advice."}
                                }
                            }
                        }

                        // 4 — Batch & expiry
                        batch := Chapter{
                            Eyebrow{body: "4 · BATCH & EXPIRY"}
                            Headline{body: "Batch and expiry on your books — alerts for the owner."}
                            batch_row := View{
                                width: Fill height: Fit
                                flow: Right
                                spacing: 20
                                WarmCard{
                                    spacing: 8
                                    View{
                                        width: Fill height: Fit
                                        flow: Right
                                        align: Align{y: 0.5}
                                        Strong{width: Fill body: "Buy · batch A12"}
                                        ExampleTag{}
                                    }
                                    Strong{body: "Amoxicillin 250 mg · 10 × 10"}
                                    Body{body: "Expires 2027-03 · alert in 7 days"}
                                    Strong{body: "ORS sachets · batch B04"}
                                    Body{body: "Expires 2026-11 · near-expiry strip on Books"}
                                    RoundedView{width: Fill height: 1 draw_bg +: {color: WellLine border_radius: 0.}}
                                    Small{body: "Shop bookkeeping only — not Drugs Act compliance."}
                                }
                                batch_points := View{
                                    width: Fill height: Fit
                                    flow: Down
                                    spacing: 16
                                    Point{head.body: "Batch on buy" line.body: "Record batch and expiry when stock comes in."}
                                    Point{head.body: "Near-expiry alerts" line.body: "Books and Sell can hint before the date passes."}
                                    batch_extra := Point{head.body: "Owner practice" line.body: "FEFO hints and Rx notes are yours to follow — Tabiyat records what you enter."}
                                }
                            }
                        }

                        // 5 — Khata
                        khata := Chapter{
                            Eyebrow{body: "5 · KHATA"}
                            Headline{body: "Customer credit and supplier dues in one party book."}
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
                                        Strong{width: Fill body: "Mehta Clinic · Customer"}
                                        ExampleTag{}
                                    }
                                    View{width: Fill height: Fit flow: Right Body{width: Fill body: "Due from earlier"} Ticket{width: Fit body: "₹ 2,480"}}
                                    View{width: Fill height: Fit flow: Right Body{width: Fill body: "+ today's sale (Due)"} Ticket{width: Fit body: "₹ 162"}}
                                    RoundedView{width: Fill height: 1 draw_bg +: {color: WellLine border_radius: 0.}}
                                    View{width: Fill height: Fit flow: Right Strong{width: Fill body: "Balance"} Ticket{width: Fit body: "₹ 2,642"}}
                                    khata_note := Small{body: "Settle later — part cash, part UPI. Supplier AP shares the same Parties screen."}
                                }
                                View{
                                    width: Fill height: Fit
                                    flow: Down
                                    spacing: 16
                                    Point{head.body: "Due on the bill" line.body: "Leave a balance on khata without a separate ledger app."}
                                    Point{head.body: "Supplier buys too" line.body: "Who you owe updates when stock arrives."}
                                    Point{head.body: "Auditable" line.body: "Who, when and how much stays in the day book."}
                                }
                            }
                        }

                        // 6 — Extra till / offline
                        lan := Chapter{
                            Eyebrow{body: "6 · EXTRA TILL"}
                            Headline{body: "Another till on the shop Wi‑Fi."}
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
                                        RoundedView{width: 12 height: 12 draw_bg +: {color: TealBright border_radius: 6.}}
                                        Strong{width: Fill body: "Router: no internet"}
                                    }
                                    View{
                                        width: Fill height: Fit
                                        flow: Right
                                        spacing: 10
                                        align: Align{y: 0.5}
                                        RoundedView{width: 12 height: 12 draw_bg +: {color: Lime border_radius: 6.}}
                                        Strong{width: Fill body: "Main counter: selling and settling"}
                                    }
                                    lan_devices := View{
                                        visible: false
                                        width: Fill height: Fit
                                        flow: Right
                                        spacing: 10
                                        align: Align{y: 0.5}
                                        RoundedView{width: 12 height: 12 draw_bg +: {color: Lime border_radius: 6.}}
                                        Strong{width: Fill body: "Second till: joined on your own Wi‑Fi"}
                                    }
                                }
                                View{
                                    width: Fill height: Fit
                                    flow: Down
                                    spacing: 14
                                    Point{head.body: "Books live on the shop devices" line.body: "When the internet drops, nothing at the pharmacy counter waits for it."}
                                    lan_extra := Point{visible: false head.body: "One Server, LAN Clients" line.body: "Extra tills share the same books over the shop network."}
                                    lan_note := Small{body: "Multi-device lines show after the owner's LAN pilot."}
                                }
                            }
                        }

                        // 7 — Free forever
                        free := Chapter{
                            Eyebrow{body: "7 · FREE FOREVER LOCALLY"}
                            Headline{body: "Free forever on your devices."}
                            free_row := View{
                                width: Fill height: Fit
                                flow: Right
                                spacing: 20
                                Point{head.body: "No per-bill charge" line.body: "No fee per sale, medicine, till or staff PIN on the local install."}
                                Point{head.body: "Android, Windows, Linux" line.body: "The same Tabiyat on the phone and the desktop till."}
                                Point{head.body: "Cloud later, by usage" line.body: "Optional Kumkaj Cloud when you want it — billed for what you use."}
                            }
                        }

                        // 8 — Start
                        start := Chapter{
                            Eyebrow{body: "8 · START"}
                            Headline{body: "Put Tabiyat on the counter."}
                            Lede{body: "Download free, or WhatsApp for a walk-through with your pharmacy and city."}
                            close_cta := CtaPair{}
                            RoundedView{width: Fill height: 1 draw_bg +: {color: WellLine border_radius: 0.}}
                            footer := View{
                                width: Fill height: Fit
                                flow: Right
                                spacing: 12
                                align: Align{y: 0.5}
                                Small{width: Fill body: "Tabiyat is a Kumkaj product by Gotigin. · tabiyat.shop"}
                                to_kumkaj := SecondaryBtn{text: "kumkaj.com  →"}
                            }
                        }
                    }
                }
            }
        }
    }
}
