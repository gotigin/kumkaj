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

    // ---- Type: Baloo 2 (display Latin), IBM Plex + Noto for scripts
    // Script members must be listed so the shaper can fall through — Baloo alone
    // paints .notdef boxes for Devanagari/Thai. Native dish lines use TextFont.
    let DisplayFont = FontFamily{
        latin := FontMember{res: crate_resource("self:resources/fonts/Baloo2-Variable.ttf") asc: 0.0 desc: 0.0 weight: 720.0}
        ui := FontMember{res: crate_resource("self:resources/fonts/IBMPlexSans-SemiBold.ttf") asc: -0.1 desc: 0.0}
        gujarati := FontMember{res: crate_resource("self:resources/fonts/NotoSansGujarati-Bold.ttf") asc: 0.0 desc: 0.0}
        devanagari := FontMember{res: crate_resource("self:resources/fonts/NotoSansDevanagari-Regular.ttf") asc: 0.0 desc: 0.0}
        tamil := FontMember{res: crate_resource("self:resources/fonts/NotoSansTamil-Regular.ttf") asc: 0.0 desc: 0.0}
        bengali := FontMember{res: crate_resource("self:resources/fonts/NotoSansBengali-Regular.ttf") asc: 0.0 desc: 0.0}
        arabic := FontMember{res: crate_resource("self:resources/fonts/NotoSansArabic-Regular.ttf") asc: 0.0 desc: 0.0}
        hebrew := FontMember{res: crate_resource("self:resources/fonts/NotoSansHebrew-Regular.ttf") asc: 0.0 desc: 0.0}
        thai := FontMember{res: crate_resource("self:resources/fonts/NotoSansThai-Regular.ttf") asc: 0.0 desc: 0.0}
    }
    let StrongFont = FontFamily{
        latin := FontMember{res: crate_resource("self:resources/fonts/IBMPlexSans-SemiBold.ttf") asc: -0.1 desc: 0.0}
        gujarati := FontMember{res: crate_resource("self:resources/fonts/NotoSansGujarati-Bold.ttf") asc: 0.0 desc: 0.0}
        devanagari := FontMember{res: crate_resource("self:resources/fonts/NotoSansDevanagari-Regular.ttf") asc: 0.0 desc: 0.0}
        tamil := FontMember{res: crate_resource("self:resources/fonts/NotoSansTamil-Regular.ttf") asc: 0.0 desc: 0.0}
        bengali := FontMember{res: crate_resource("self:resources/fonts/NotoSansBengali-Regular.ttf") asc: 0.0 desc: 0.0}
        arabic := FontMember{res: crate_resource("self:resources/fonts/NotoSansArabic-Regular.ttf") asc: 0.0 desc: 0.0}
        hebrew := FontMember{res: crate_resource("self:resources/fonts/NotoSansHebrew-Regular.ttf") asc: 0.0 desc: 0.0}
        thai := FontMember{res: crate_resource("self:resources/fonts/NotoSansThai-Regular.ttf") asc: 0.0 desc: 0.0}
    }
    let TextFont = FontFamily{
        latin := FontMember{res: crate_resource("self:resources/fonts/IBMPlexSans-Text.ttf") asc: -0.1 desc: 0.0}
        gujarati := FontMember{res: crate_resource("self:resources/fonts/NotoSansGujarati-Regular.ttf") asc: 0.0 desc: 0.0}
        devanagari := FontMember{res: crate_resource("self:resources/fonts/NotoSansDevanagari-Regular.ttf") asc: 0.0 desc: 0.0}
        tamil := FontMember{res: crate_resource("self:resources/fonts/NotoSansTamil-Regular.ttf") asc: 0.0 desc: 0.0}
        bengali := FontMember{res: crate_resource("self:resources/fonts/NotoSansBengali-Regular.ttf") asc: 0.0 desc: 0.0}
        arabic := FontMember{res: crate_resource("self:resources/fonts/NotoSansArabic-Regular.ttf") asc: 0.0 desc: 0.0}
        hebrew := FontMember{res: crate_resource("self:resources/fonts/NotoSansHebrew-Regular.ttf") asc: 0.0 desc: 0.0}
        thai := FontMember{res: crate_resource("self:resources/fonts/NotoSansThai-Regular.ttf") asc: 0.0 desc: 0.0}
    }
    let TicketFont = FontFamily{
        latin := FontMember{res: crate_resource("self:resources/fonts/LiberationMono-Regular.ttf") asc: 0.0 desc: 0.0}
        ui := FontMember{res: crate_resource("self:resources/fonts/IBMPlexSans-Text.ttf") asc: -0.1 desc: 0.0}
    }

    // Cream on deep red is the only light-on-dark here; drop the SDF luma
    // bias so it never paints glyph boxes (vendor shader untouched).
    mod.widgets.Label.draw_text.sdf_luma_bias = 0.0
    mod.widgets.Button.draw_text.sdf_luma_bias = 0.0
    mod.widgets.Markdown.draw_text.sdf_luma_bias = 0.0

    // Markdown/TextFlow paint from text_style_* (not draw_text.text_style).
    // Same pin as apps/khaynjo — without it Indic/Thai become .notdef boxes.
    mod.widgets.Label.draw_text.text_style.font_family = TextFont
    mod.widgets.Button.draw_text.text_style.font_family = TextFont
    mod.widgets.Markdown.text_style_normal.font_family = TextFont
    mod.widgets.Markdown.text_style_italic.font_family = TextFont
    mod.widgets.Markdown.text_style_bold.font_family = StrongFont
    mod.widgets.Markdown.text_style_bold_italic.font_family = StrongFont
    mod.widgets.TextFlow.text_style_normal.font_family = TextFont
    mod.widgets.TextFlow.text_style_italic.font_family = TextFont
    mod.widgets.TextFlow.text_style_bold.font_family = StrongFont
    mod.widgets.TextFlow.text_style_bold_italic.font_family = StrongFont

    // TextFlow paints from `font_color` (not only draw_text.color) — set both.
    let Eyebrow = Markdown{
        selectable: true
        height: Fit
        paragraph_spacing: 0.
        pre_code_spacing: 0.
        draw_block +: { selection_color: #xF69D2CFF }
        width: Fill
        padding: 0.
        font_color: Accent
        font_size: 13.
        text_style_normal: theme.font_bold{font_family: StrongFont font_size: 13. line_spacing: 1.2}
        text_style_bold: theme.font_bold{font_family: StrongFont font_size: 13. line_spacing: 1.2}
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
        draw_block +: { selection_color: #xF69D2CFF }
        width: Fill
        padding: 0.
        font_color: Maroon
        font_size: 48.
        text_style_normal: theme.font_bold{font_family: DisplayFont font_size: 48. line_spacing: 0.72}
        text_style_bold: theme.font_bold{font_family: DisplayFont font_size: 48. line_spacing: 0.72}
        draw_text +: {
            color: Maroon
            text_style: theme.font_bold{font_family: DisplayFont font_size: 48. line_spacing: 0.72}
        }
    }
    let Lede = Markdown{
        selectable: true
        height: Fit
        paragraph_spacing: 0.
        pre_code_spacing: 0.
        draw_block +: { selection_color: #xF69D2CFF }
        width: Fill
        padding: 0.
        font_color: Muted
        font_size: 19.
        text_style_normal: theme.font_regular{font_family: TextFont font_size: 19. line_spacing: 1.45}
        draw_text +: {
            color: Muted
            text_style: theme.font_regular{font_family: TextFont font_size: 19. line_spacing: 1.45}
        }
    }
    let Body = Markdown{
        selectable: true
        height: Fit
        paragraph_spacing: 0.
        pre_code_spacing: 0.
        draw_block +: { selection_color: #xF69D2CFF }
        width: Fill
        padding: 0.
        font_color: Muted
        font_size: 15.5
        text_style_normal: theme.font_regular{font_family: TextFont font_size: 15.5 line_spacing: 1.45}
        draw_text +: {
            color: Muted
            text_style: theme.font_regular{font_family: TextFont font_size: 15.5 line_spacing: 1.45}
        }
    }
    let Strong = Markdown{
        selectable: true
        height: Fit
        paragraph_spacing: 0.
        pre_code_spacing: 0.
        draw_block +: { selection_color: #xF69D2CFF }
        width: Fill
        padding: 0.
        font_color: Ink
        font_size: 16.
        text_style_normal: theme.font_bold{font_family: StrongFont font_size: 16. line_spacing: 1.25}
        text_style_bold: theme.font_bold{font_family: StrongFont font_size: 16. line_spacing: 1.25}
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
        draw_block +: { selection_color: #xF69D2CFF }
        width: Fill
        padding: 0.
        font_color: Hint
        font_size: 13.
        text_style_normal: theme.font_regular{font_family: TextFont font_size: 13. line_spacing: 1.4}
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
    let Ticket = Markdown{
        selectable: true
        height: Fit
        paragraph_spacing: 0.
        pre_code_spacing: 0.
        draw_block +: { selection_color: #xF69D2CFF }
        width: Fill
        padding: 0.
        font_color: Ink
        font_size: 14.
        // TextFlow paints text_style_normal (not only draw_text) — same pin as Body/Strong.
        text_style_normal: theme.font_regular{font_family: TicketFont font_size: 14. line_spacing: 1.35}
        text_style_bold: theme.font_bold{font_family: TicketFont font_size: 14. line_spacing: 1.35}
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
        Markdown{
            padding: 0.
            selectable: true
            height: Fit
            paragraph_spacing: 0.
            pre_code_spacing: 0.
            body: "Example"
            font_color: Muted
            draw_text +: {color: Muted text_style: theme.font_regular{font_family: TextFont font_size: 12.}}
            draw_block +: { selection_color: #xF69D2CFF }
        }
    }
    // Country shelf card: photo · English country · local-script · English dish.
    let WorldDish = WarmCard{
        width: Fill
        spacing: 4
        padding: Inset{left: 10. right: 10. top: 10. bottom: 12.}
        photo := Image{
            width: Fill height: 72
            fit: ImageFit.CropToFill
            src: crate_resource("self:resources/dishes/japan-ramen.webp")
        }
        country := Strong{body: "Country"}
        // TextFlow paints text_style_normal (not draw_text.text_style) — pin both.
        native := Markdown{
            selectable: true
            height: Fit
            paragraph_spacing: 0.
            pre_code_spacing: 0.
            draw_block +: { selection_color: #xF69D2CFF }
            width: Fill
            padding: 0.
            font_color: Maroon
            font_size: 14.
            text_style_normal: theme.font_regular{font_family: TextFont font_size: 14. line_spacing: 1.25}
            text_style_bold: theme.font_bold{font_family: StrongFont font_size: 14. line_spacing: 1.25}
            draw_text +: {
                color: Maroon
                text_style: theme.font_regular{font_family: TextFont font_size: 14. line_spacing: 1.25}
            }
            body: "Local name"
        }
        dish := Body{body: "Famous dish"}
    }

    // Dense row for the country list (no photo) — fits many lines on one slide.
    let WorldLine = View{
        width: Fill height: Fit
        flow: Right
        spacing: 8
        align: Align{y: 0.5}
        padding: Inset{left: 10. right: 10. top: 5. bottom: 5.}
        show_bg: true
        draw_bg +: {color: Well border_radius: 8.}
        country := Strong{width: 96 font_size: 14. text_style_normal: theme.font_bold{font_family: StrongFont font_size: 14. line_spacing: 1.2} text_style_bold: theme.font_bold{font_family: StrongFont font_size: 14. line_spacing: 1.2} draw_text +: {text_style: theme.font_bold{font_family: StrongFont font_size: 14. line_spacing: 1.2}} body: "Country"}
        native := Markdown{
            selectable: true
            height: Fit
            paragraph_spacing: 0.
            pre_code_spacing: 0.
            draw_block +: { selection_color: #xF69D2CFF }
            width: Fill
            padding: 0.
            font_color: Maroon
            font_size: 13.
            text_style_normal: theme.font_regular{font_family: TextFont font_size: 13. line_spacing: 1.2}
            draw_text +: {
                color: Maroon
                text_style: theme.font_regular{font_family: TextFont font_size: 13. line_spacing: 1.2}
            }
            body: "Local · dish"
        }
        dish := Body{
            width: Fit
            font_size: 13.
            text_style_normal: theme.font_regular{font_family: TextFont font_size: 13. line_spacing: 1.2}
            draw_text +: {text_style: theme.font_regular{font_family: TextFont font_size: 13. line_spacing: 1.2}}
            body: "English dish"
        }
    }

    // Marketing banner — capped height so one chapter still fits a phone/laptop.
    let SiteBanner = Image{
        width: Fill
        height: 148
        fit: ImageFit.CropToFill
        src: crate_resource("self:resources/media/banner-1.webp")
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
        head := Strong{body: "Point"}
        line := Body{body: "Detail"}
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
        head := Markdown{padding: 0. selectable: true height: Fit paragraph_spacing: 0. pre_code_spacing: 0. body: "KOT" font_color: Ink draw_text +: {color: Ink text_style: theme.font_bold{font_family: StrongFont font_size: 14.}} draw_block +: { selection_color: #xF69D2CFF }}
        line := Markdown{width: Fill padding: 0. selectable: true height: Fit paragraph_spacing: 0. pre_code_spacing: 0. body: "Items" font_color: Muted draw_text +: {color: Muted text_style: theme.font_regular{font_family: TextFont font_size: 13.}} draw_block +: { selection_color: #xF69D2CFF }}
    }
    let KdsColumn = RoundedView{
        width: Fill height: Fit
        flow: Down
        spacing: 8
        padding: Inset{left: 10. right: 10. top: 10. bottom: 12.}
        draw_bg +: {color: WellWarm border_radius: 14.}
        title := Markdown{padding: 0. selectable: true height: Fit paragraph_spacing: 0. pre_code_spacing: 0. body: "Column" font_color: Maroon draw_text +: {color: Maroon text_style: theme.font_bold{font_family: DisplayFont font_size: 18.}} draw_block +: { selection_color: #xF69D2CFF }}
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
            download := PrimaryBtn{text: "Get it on Google Play"}
            whatsapp := SecondaryBtn{text: "WhatsApp for a demo"}
        }
        note := Small{body: "Free on Google Play · Windows and Linux packages coming."}
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
                        brand_icon := Image{
                            width: 36 height: 36
                            fit: ImageFit.Stretch
                            src: crate_resource("self:resources/media/app-icon.webp")
                        }
                        Markdown{
                            padding: 0.
                            selectable: true
                            height: Fit
                            paragraph_spacing: 0.
                            pre_code_spacing: 0.
                            body: "Khaynjo"
                            font_color: Cream
                            draw_text +: {color: Cream text_style: theme.font_bold{font_family: DisplayFont font_size: 25.}}
                            draw_block +: { selection_color: #xF69D2CFF }
                        }
                        tagline := Markdown{
                            padding: 0.
                            selectable: true
                            height: Fit
                            paragraph_spacing: 0.
                            pre_code_spacing: 0.
                            body: "the restaurant system"
                            font_color: GoldSoft
                            draw_text +: {color: GoldSoft text_style: theme.font_regular{font_family: TextFont font_size: 13.}}
                            draw_block +: { selection_color: #xF69D2CFF }
                        }
                        View{width: Fill height: 1}
                        chapter_name := Markdown{
                            padding: 0.
                            selectable: true
                            height: Fit
                            paragraph_spacing: 0.
                            pre_code_spacing: 0.
                            body: "Lunch rush"
                            font_color: #xF2D9C9
                            draw_text +: {color: #xF2D9C9 text_style: theme.font_regular{font_family: TextFont font_size: 13.}}
                            draw_block +: { selection_color: #xF69D2CFF }
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
                        counter := Markdown{
                            padding: 0.
                            selectable: true
                            height: Fit
                            paragraph_spacing: 0.
                            pre_code_spacing: 0.
                            body: "1 / 10"
                            font_color: #xF2D9C9
                            draw_text +: {color: #xF2D9C9 text_style: theme.font_bold{font_family: StrongFont font_size: 12.5}}
                            draw_block +: { selection_color: #xF69D2CFF }
                        }
                    }

                    slides := SlidesView{
                        width: Fill
                        height: Fill
                        anim_speed: 0.86
                        axis: Vertical

                        // 1 — Lunch rush (floor-states, table-cards) + world shelf on the first screen
                        rush := Chapter{
                            Eyebrow{body: "KHAYNJO · THE RESTAURANT SYSTEM"}
                            Headline{body: "Lunch rush. No internet. No problem."}
                            rush_lede := Lede{body: "From table to kitchen to bill, on your own devices — cafés, QSRs, diners and cloud kitchens from Tokyo to Mexico City, Mumbai to Madrid. Every table shows its minutes and its running bill at a glance."}
                            Small{body: "Drag to select · Ctrl+C / ⌘C to copy"}
                            // Country · local-script · English dish — first screen, not below the fold.
                            Strong{body: "Country · local name · famous dish"}
                            rush_world := View{
                                width: Fill height: Fit
                                flow: Down
                                spacing: 8
                                rush_world_a := View{
                                    width: Fill height: Fit
                                    flow: Right
                                    spacing: 10
                                    WorldDish{
                                        photo.src: crate_resource("self:resources/dishes/india-butter-chicken.webp")
                                        country.body: "India"
                                        native.body: "भारत · बटर चिकन"
                                        dish.body: "Butter chicken"
                                    }
                                    WorldDish{
                                        photo.src: crate_resource("self:resources/dishes/mexico-birria.webp")
                                        country.body: "Mexico"
                                        native.body: "México · Tacos de birria"
                                        dish.body: "Birria tacos"
                                    }
                                }
                                rush_world_b := View{
                                    width: Fill height: Fit
                                    flow: Right
                                    spacing: 10
                                    WorldDish{
                                        photo.src: crate_resource("self:resources/dishes/italy-margherita.webp")
                                        country.body: "Italy"
                                        native.body: "Italia · Pizza Margherita"
                                        dish.body: "Margherita pizza"
                                    }
                                    WorldDish{
                                        photo.src: crate_resource("self:resources/dishes/thailand-pad-thai.webp")
                                        country.body: "Thailand"
                                        native.body: "ประเทศไทย · ผัดไทย"
                                        dish.body: "Pad Thai"
                                    }
                                }
                            }
                            rush_scripts := Markdown{
                                padding: 0. selectable: true height: Fit paragraph_spacing: 0. pre_code_spacing: 0.
                                body: "પનીર ટિક્કા · پنیر تکہ · பன்னீர் டிக்கா · পনির টিক্কা · ประเทศไทย · ผัดไทย"
                                font_color: Maroon font_size: 14.
                                text_style_normal: theme.font_regular{font_family: TextFont font_size: 14.}
                                draw_text +: {color: Maroon text_style: theme.font_regular{font_family: TextFont font_size: 14.}}
                                draw_block +: { selection_color: #xF69D2CFF }
                            }
                            Small{body: "Sixteen countries on chapter 8 — same catalogue words captains already type."}
                            rush_row := View{
                                width: Fill height: Fit
                                flow: Right
                                spacing: 20
                                align: Align{y: 0.0}
                                View{
                                    width: Fill height: Fit
                                    flow: Down
                                    spacing: 12
                                    rush_cta := CtaPair{}
                                }
                                floor_card := WarmCard{
                                    width: 560
                                    spacing: 10
                                    View{
                                        width: Fill height: Fit
                                        flow: Right
                                        align: Align{y: 0.5}
                                        Strong{width: Fill body: "Tap a table to walk it through a visit"}
                                        ExampleTag{}
                                    }
                                    floor := View{
                                        width: Fill height: Fit
                                        flow: Down
                                        spacing: 8
                                        View{
                                            width: Fill height: Fit
                                            flow: Right
                                            spacing: 8
                                            t1 := TableTile{}
                                            t2 := TableTile{}
                                            t3 := TableTile{}
                                        }
                                        View{
                                            width: Fill height: Fit
                                            flow: Right
                                            spacing: 8
                                            t4 := TableTile{}
                                            t5 := TableTile{}
                                            t6 := TableTile{}
                                        }
                                    }
                                    floor_note := Small{body: "Same colours as Khaynjo's floor. Bill printed → Paid → Free."}
                                }
                            }
                            rush_banner := SiteBanner{
                                src: crate_resource("self:resources/media/banner-1.webp")
                            }
                        }

                        // 2 — Seated, order taken (seated, variations)
                        order := Chapter{
                            Eyebrow{body: "2 · SEATED, ORDER TAKEN"}
                            Headline{body: "Take the order at the table. Sizes, add-ons and notes, right there."}
                            order_row := View{
                                width: Fill height: Fit
                                flow: Right
                                spacing: 20
                                align: Align{y: 0.0}
                                // Dense top-packed card (same idea as the KOT ticket) — no stretch hole.
                                order_card := WarmCard{
                                    width: 360
                                    spacing: 6
                                    View{
                                        width: Fill height: Fit
                                        flow: Right
                                        spacing: 10
                                        align: Align{y: 0.5}
                                        RoundedView{
                                            width: Fit height: Fit
                                            padding: Inset{left: 10. right: 10. top: 4. bottom: 5.}
                                            draw_bg +: {color: SeatFill border_color: SeatLine border_size: 1.5 border_radius: 8.}
                                            Markdown{padding: 0. selectable: true height: Fit paragraph_spacing: 0. pre_code_spacing: 0. body: "T4 · Occupied" font_color: Ink font_size: 13. text_style_normal: theme.font_bold{font_family: StrongFont font_size: 13.} draw_text +: {color: Ink text_style: theme.font_bold{font_family: StrongFont font_size: 13.}} draw_block +: { selection_color: #xF69D2CFF }}
                                        }
                                        View{width: Fill height: Fit}
                                        ExampleTag{}
                                    }
                                    Strong{body: "Paneer tikka — Full"}
                                    Body{body: "+ extra mint chutney"}
                                    Strong{body: "Masala chaas × 2"}
                                    Body{body: "Note: less salt"}
                                    Strong{body: "Butter naan × 4"}
                                }
                                order_points := View{
                                    width: Fill height: Fit
                                    flow: Down
                                    spacing: 16
                                    Point{head.body: "Seat first" line.body: "Mark a table Occupied the moment guests sit — before any dish is ordered."}
                                    Point{head.body: "Sizes and add-ons" line.body: "Pick one size; add optional extras up to the limit you set."}
                                    Point{head.body: "A note per dish" line.body: "“Less spicy” goes to the kitchen with the dish it belongs to."}
                                }
                            }
                        }

                        // 3 — The KOT (kot-station, kot-no-prices; kot-autoprint gated)
                        kot := Chapter{
                            Eyebrow{body: "3 · THE KOT"}
                            Headline{body: "One ticket per kitchen station. Never a price on it."}
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
                                    Ticket{body: "KOT 27 · HOT KITCHEN"}
                                    Ticket{body: "Table T4 · by Captain 1"}
                                    Ticket{body: "12:42"}
                                    TicketRule{}
                                    Ticket{body: "1  Paneer tikka (Full)"}
                                    Ticket{body: "   + extra mint chutney"}
                                    Ticket{body: "2  Masala chaas"}
                                    Ticket{body: "   note: less salt"}
                                    Ticket{body: "4  Butter naan"}
                                    TicketRule{}
                                    Small{body: "Example ticket"}
                                }
                                kot_points := View{
                                    width: Fill height: Fit
                                    flow: Down
                                    spacing: 16
                                    Point{head.body: "Split by station" line.body: "Hot kitchen, cold counter, bar — each gets only its own dishes, numbered per day."}
                                    Point{head.body: "Who, which table, what notes" line.body: "The ticket carries the waiter's name, the table and every note."}
                                    Point{head.body: "No prices" line.body: "The kitchen cooks; the bill is the counter's business."}
                                    // Shown only after the owner's real-hardware LAN pilot passes.
                                    kot_lan := Point{visible: false head.body: "Prints itself" line.body: "Send, and the ticket prints at the right station — nobody carries it."}
                                }
                            }
                        }

                        // 4 — Kitchen in step (kds, all-day-counts, coursing)
                        kitchen := Chapter{
                            Eyebrow{body: "4 · KITCHEN IN STEP"}
                            Headline{body: "The kitchen sees what to cook now — and what can wait."}
                            kds := View{
                                width: Fill height: Fit
                                flow: Right
                                spacing: 12
                                KdsColumn{
                                    title.body: "New"
                                    KdsCard{head.body: "KOT 31 · T6 · 1 min" line.body: "2 Veg biryani · 1 Raita"}
                                    KdsCard{head.body: "KOT 30 · T2 · 3 min" line.body: "3 Masala dosa"}
                                }
                                KdsColumn{
                                    title.body: "Preparing"
                                    KdsCard{draw_bg.color: #xFFF4CC head.body: "KOT 27 · T4 · 9 min" line.body: "1 Paneer tikka · 4 Butter naan"}
                                    KdsCard{draw_bg.color: #xFFC48A head.body: "KOT 24 · T1 · 17 min" line.body: "2 Dal makhani — mains held"}
                                }
                                KdsColumn{
                                    title.body: "Ready"
                                    KdsCard{draw_bg.color: #xBFEBC8 head.body: "KOT 22 · T3 · 21 min" line.body: "2 Gulab jamun"}
                                }
                            }
                            kds_points := View{
                                width: Fill height: Fit
                                flow: Right
                                spacing: 20
                                Point{head.body: "New → Preparing → Ready" line.body: "One tap on a ticket moves it on; older tickets warm in colour."}
                                Point{head.body: "To cook now" line.body: "All-day counts: how many of each dish are waiting."}
                                Point{head.body: "Hold and fire" line.body: "Mains wait until the starters go; recall a ticket bumped too soon."}
                            }
                        }

                        // 5 — Service changes (move-merge, kot-cancel, nc)
                        changes := Chapter{
                            Eyebrow{body: "5 · SERVICE CHANGES"}
                            Headline{body: "Move, merge, cancel or comp — never silently."}
                            changes_grid := View{
                                width: Fill height: Fit
                                flow: Right
                                spacing: 16
                                WarmCard{
                                    Strong{body: "Move"}
                                    Body{body: "Guests change tables; the order and its minutes go with them."}
                                }
                                WarmCard{
                                    Strong{body: "Merge"}
                                    Body{body: "Two tables, one bill."}
                                }
                                WarmCard{
                                    Strong{body: "Cancel after the KOT"}
                                    Body{body: "With a reason, cooked or not, and who approved it. The kitchen gets a cancel slip."}
                                }
                                WarmCard{
                                    Strong{body: "Complimentary"}
                                    Body{body: "The dish stays on the bill at ₹ 0, with the reason and the approver."}
                                }
                            }
                            changes_note := Small{body: "Who, when and why is kept for every change — the day report lists cancels and comps."}
                        }

                        // 6 — Settle (split, settle, upi-qr, due-khata, free-on-pay)
                        settle := Chapter{
                            Eyebrow{body: "6 · SETTLE"}
                            Headline{body: "Cash, UPI, part, Due, tip — and the table frees only when it's paid."}
                            settle_row := View{
                                width: Fill height: Fit
                                flow: Right
                                spacing: 20
                                align: Align{y: 0.0}
                                bill := WarmCard{
                                    width: 360
                                    spacing: 6
                                    View{
                                        width: Fill height: Fit
                                        flow: Right
                                        spacing: 10
                                        align: Align{y: 0.5}
                                        RoundedView{
                                            width: Fit height: Fit
                                            padding: Inset{left: 10. right: 10. top: 4. bottom: 5.}
                                            draw_bg +: {color: BillFill border_color: BillLine border_size: 1.5 border_radius: 8.}
                                            Markdown{padding: 0. selectable: true height: Fit paragraph_spacing: 0. pre_code_spacing: 0. body: "T4 · Bill printed" font_color: Ink font_size: 13. text_style_normal: theme.font_bold{font_family: StrongFont font_size: 13.} draw_text +: {color: Ink text_style: theme.font_bold{font_family: StrongFont font_size: 13.}} draw_block +: { selection_color: #xF69D2CFF }}
                                        }
                                        View{width: Fill height: Fit}
                                        ExampleTag{}
                                    }
                                    View{
                                        width: Fill height: Fit
                                        flow: Right
                                        Body{width: Fill body: "Bill"}
                                        Ticket{width: Fit body: "₹ 1,240"}
                                    }
                                    View{
                                        width: Fill height: Fit
                                        flow: Right
                                        Body{width: Fill body: "Customer paid (cash)"}
                                        Ticket{width: Fit body: "₹ 1,500"}
                                    }
                                    View{
                                        width: Fill height: Fit
                                        flow: Right
                                        Strong{width: Fill body: "Return to customer"}
                                        Ticket{width: Fit body: "₹ 260"}
                                    }
                                    RoundedView{width: Fill height: 1 draw_bg +: {color: WellLine border_radius: 0.}}
                                    Small{body: "A tip is recorded apart from sales, never mixed into the bill."}
                                }
                                settle_points := View{
                                    width: Fill height: Fit
                                    flow: Down
                                    spacing: 14
                                    Point{head.body: "Split the bill" line.body: "By item, by seat, or evenly — each part settles on its own."}
                                    Point{head.body: "UPI QR with the amount" line.body: "The customer scans; the amount is already in it."}
                                    Point{head.body: "Part payment and Due" line.body: "What's left goes on the customer's khata in Parties."}
                                    // The headline says it too; short windows drop this one.
                                    settle_free := Point{head.body: "Free on pay" line.body: "A table with money still due cannot be freed."}
                                }
                            }
                        }

                        // 7 — Close the day (shift-close, day-report)
                        dayclose := Chapter{
                            Eyebrow{body: "7 · CLOSE THE DAY"}
                            Headline{body: "Day end you can trust: expected, counted, and the difference named."}
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
                                        Strong{width: Fill body: "Cash drawer, shift close"}
                                        ExampleTag{}
                                    }
                                    View{width: Fill height: Fit flow: Right Body{width: Fill body: "Opening float"} Ticket{width: Fit body: "  2,000"}}
                                    View{width: Fill height: Fit flow: Right Body{width: Fill body: "+ cash taken"} Ticket{width: Fit body: " 18,450"}}
                                    View{width: Fill height: Fit flow: Right Body{width: Fill body: "+ cash tips"} Ticket{width: Fit body: "    300"}}
                                    View{width: Fill height: Fit flow: Right Body{width: Fill body: "− cash paid out"} Ticket{width: Fit body: "    600"}}
                                    RoundedView{width: Fill height: 1 draw_bg +: {color: WellLine border_radius: 0.}}
                                    View{width: Fill height: Fit flow: Right Strong{width: Fill body: "Expected in the drawer"} Ticket{width: Fit body: "₹ 20,150"}}
                                    View{width: Fill height: Fit flow: Right Strong{width: Fill body: "Counted"} Ticket{width: Fit body: "₹ 20,050"}}
                                    View{width: Fill height: Fit flow: Right Strong{width: Fill draw_text.color: Red body: "Short"} Ticket{width: Fit draw_text.color: Red body: "₹ 100"}}
                                }
                                close_points := View{
                                    width: Fill height: Fit
                                    flow: Down
                                    spacing: 14
                                    Point{head.body: "Shift handover slip" line.body: "Close one shift, print the slip, and the next one starts clean."}
                                    Point{head.body: "The day report" line.body: "By staff, by order type, by hour and by dish — with cancels, comps and tips."}
                                }
                            }
                        }

                        // 8 — Your menu: full country list (English · local script · English dish)
                        menu := Chapter{
                            Eyebrow{body: "8 · YOUR MENU, YOUR WAY"}
                            Headline{body: "Every country at your counter — your words, your codes, your prices."}
                            Lede{body: "English country · name in their script · famous dish in English. Put those words on the menu — short codes and area prices still apply."}
                            // Two columns of 8 on wide screens; one stacked column when narrow.
                            menu_list := View{
                                width: Fill height: Fit
                                flow: Right
                                spacing: 8
                                menu_col_a := View{
                                    width: Fill height: Fit
                                    flow: Down
                                    spacing: 3
                                    WorldLine{country.body: "Japan" native.body: "日本 · ラーメン" dish.body: "Ramen"}
                                    WorldLine{country.body: "Korea" native.body: "한국 · 비빔밥" dish.body: "Bibimbap"}
                                    WorldLine{country.body: "China" native.body: "中国 · 北京烤鸭" dish.body: "Peking duck"}
                                    WorldLine{country.body: "Thailand" native.body: "ประเทศไทย · ผัดไทย" dish.body: "Pad Thai"}
                                    WorldLine{country.body: "Vietnam" native.body: "Việt Nam · Bánh mì" dish.body: "Bánh mì"}
                                    WorldLine{country.body: "Indonesia" native.body: "Indonesia · Rendang" dish.body: "Beef rendang"}
                                    WorldLine{country.body: "India" native.body: "भारत · बटर चिकन" dish.body: "Butter chicken"}
                                    WorldLine{country.body: "India" native.body: "இந்தியா · மசாலா தோசை" dish.body: "Masala dosa"}
                                }
                                menu_col_b := View{
                                    width: Fill height: Fit
                                    flow: Down
                                    spacing: 3
                                    WorldLine{country.body: "Turkey" native.body: "Türkiye · Adana kebap" dish.body: "Adana kebab"}
                                    WorldLine{country.body: "Lebanon" native.body: "لبنان · شاورما" dish.body: "Shawarma"}
                                    WorldLine{country.body: "Morocco" native.body: "المغرب · طاجين" dish.body: "Chicken tagine"}
                                    WorldLine{country.body: "Italy" native.body: "Italia · Pizza Margherita" dish.body: "Margherita pizza"}
                                    WorldLine{country.body: "France" native.body: "France · Croissant" dish.body: "Croissant"}
                                    WorldLine{country.body: "Spain" native.body: "España · Paella" dish.body: "Paella"}
                                    WorldLine{country.body: "Greece" native.body: "Ελλάδα · Μουσακάς" dish.body: "Moussaka"}
                                    WorldLine{country.body: "Mexico" native.body: "México · Tacos de birria" dish.body: "Birria tacos"}
                                }
                            }
                            menu_photos := View{
                                width: Fill height: Fit
                                flow: Right
                                spacing: 8
                                WorldDish{
                                    photo.src: crate_resource("self:resources/dishes/india-butter-chicken.webp")
                                    country.body: "India"
                                    native.body: "भारत · बटर चिकन"
                                    dish.body: "Butter chicken"
                                }
                                WorldDish{
                                    photo.src: crate_resource("self:resources/dishes/thailand-pad-thai.webp")
                                    country.body: "Thailand"
                                    native.body: "ประเทศไทย · ผัดไทย"
                                    dish.body: "Pad Thai"
                                }
                                WorldDish{
                                    photo.src: crate_resource("self:resources/dishes/lebanon-shawarma.webp")
                                    country.body: "Lebanon"
                                    native.body: "لبنان · شاورما"
                                    dish.body: "Shawarma"
                                }
                                WorldDish{
                                    photo.src: crate_resource("self:resources/dishes/mexico-birria.webp")
                                    country.body: "Mexico"
                                    native.body: "México · Tacos de birria"
                                    dish.body: "Birria tacos"
                                }
                            }
                            menu_script := WarmCard{
                                spacing: 6
                                Markdown{
                                    padding: 0. selectable: true height: Fit paragraph_spacing: 0. pre_code_spacing: 0.
                                    body: "પનીર ટિક્કા · پنیر تکہ · பன்னீர் டிக்கா · পনির টিক্কা"
                                    font_color: Maroon font_size: 15.
                                    text_style_normal: theme.font_regular{font_family: TextFont font_size: 15.}
                                    draw_text +: {color: Maroon text_style: theme.font_regular{font_family: TextFont font_size: 15.}}
                                    draw_block +: { selection_color: #xF69D2CFF }
                                }
                                Body{body: "Same dish, the names your captains already type — Gujarati, Urdu, Tamil, Bengali, or English."}
                            }
                            menu_note := Small{body: "Samples from the public Khaynjo catalogue · 29 languages on screen in the app."}
                        }

                        // 9 — Internet down (offline-lan; the multi-device half is gated)
                        offline := Chapter{
                            Eyebrow{body: "9 · INTERNET DOWN?"}
                            Headline{body: "Service goes on."}
                            Lede{body: "Orders, bills and the day book live on your own devices. When the internet drops in the middle of lunch, nothing in the restaurant waits for it."}
                            offline_card := WarmCard{
                                spacing: 12
                                View{
                                    width: Fill height: Fit
                                    flow: Right
                                    spacing: 10
                                    align: Align{y: 0.5}
                                    RoundedView{width: 12 height: 12 draw_bg +: {color: Red border_radius: 6.}}
                                    Strong{width: Fill body: "Router: no internet"}
                                }
                                View{
                                    width: Fill height: Fit
                                    flow: Right
                                    spacing: 10
                                    align: Align{y: 0.5}
                                    RoundedView{width: 12 height: 12 draw_bg +: {color: BillLine border_radius: 6.}}
                                    Strong{width: Fill body: "Counter: taking orders and settling bills"}
                                }
                                // Shown only after the owner's real-hardware LAN pilot passes.
                                offline_lan := View{
                                    visible: false
                                    width: Fill height: Fit
                                    flow: Right
                                    spacing: 10
                                    align: Align{y: 0.5}
                                    RoundedView{width: 12 height: 12 draw_bg +: {color: BillLine border_radius: 6.}}
                                    Strong{width: Fill body: "Captains' tablets and the kitchen screen: working together on your own Wi‑Fi"}
                                }
                            }
                        }

                        // 10 — Start (free-local): one CTA screen, no banner stack.
                        start := Chapter{
                            Eyebrow{body: "10 · START"}
                            Headline{body: "Free forever on your devices."}
                            Lede{body: "No charge per table, waiter, order or device. On Google Play now — Windows and Linux packages follow. Built in India, ready for restaurants anywhere."}
                            start_world := View{
                                width: Fill height: Fit
                                flow: Right
                                spacing: 10
                                WorldDish{
                                    photo.src: crate_resource("self:resources/dishes/india-butter-chicken.webp")
                                    country.body: "India"
                                    native.body: "भारत · बटर चिकन"
                                    dish.body: "Butter chicken"
                                }
                                WorldDish{
                                    photo.src: crate_resource("self:resources/dishes/thailand-pad-thai.webp")
                                    country.body: "Thailand"
                                    native.body: "ประเทศไทย · ผัดไทย"
                                    dish.body: "Pad Thai"
                                }
                                WorldDish{
                                    photo.src: crate_resource("self:resources/dishes/lebanon-shawarma.webp")
                                    country.body: "Lebanon"
                                    native.body: "لبنان · شاورما"
                                    dish.body: "Shawarma"
                                }
                                WorldDish{
                                    photo.src: crate_resource("self:resources/dishes/mexico-birria.webp")
                                    country.body: "Mexico"
                                    native.body: "México · Tacos de birria"
                                    dish.body: "Birria tacos"
                                }
                            }
                            Small{body: "Country · local-script name · English dish — full list on chapter 8."}
                            View{
                                width: Fill height: Fit
                                flow: Right
                                spacing: 14
                                align: Align{y: 0.5}
                                start_icon := Image{
                                    width: 56 height: 56
                                    fit: ImageFit.Stretch
                                    src: crate_resource("self:resources/media/app-icon.webp")
                                }
                                View{
                                    width: Fill height: Fit
                                    flow: Down
                                    spacing: 4
                                    Strong{body: "Khaynjo on Google Play"}
                                    Small{body: "com.khaynjo · free · no account required for day-to-day service"}
                                }
                            }
                            close_cta := CtaPair{}
                            RoundedView{width: Fill height: 1 draw_bg +: {color: WellLine border_radius: 0.}}
                            footer := View{
                                width: Fill height: Fit
                                flow: Right
                                spacing: 12
                                align: Align{y: 0.5}
                                Small{width: Fill body: "Khaynjo is a Kumkaj product by Gotigin."}
                                to_kumkaj := SecondaryBtn{text: "kumkaj.com  →"}
                            }
                        }
                    }
                }
            }
        }
    }
}
