// kumkaj.com — the whole page, one Makepad 2 `script_mod!` (included from
// main.rs so `App` is in scope). Story: docs/WEBSITES.md §9. Palette: §4.
// Claims: only Ship / Have / Done rows of MOKLANJO.md / KHAYNJO.md.
// One scrolling page (was a SlidesView deck); App::apply_responsive shrinks
// type and stacks rows; body copy stays visible on every tier.
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
    // Kumkaj mark petals — cyan stem, warm orange arm (not Gotigin amber clone).
    let PetalCyan = #x5A8FFF
    let PetalCyanDim = #x2A6BFF
    // Terracotta lifted for text on navy (6.9:1); the fill stays #A23E18.
    let Clay = #xE8875C
    let Good = #x6CCB94
    let Bad = #xF2846F
    let MokNavy = #x0A3A8C
    let MokOrange = #xDF5E16
    let KhaRed = #x9A0E10
    let KhaGold = #xF69D2C

    // ---- Type: Latin chrome first; script faces only on WordChip (WorldFont).
    // Do NOT put Noto back into StrongFont/TextFont — Makepad downloads every
    // FontFamily member on first use, and that bloated first paint. Do NOT
    // remove WorldFont while §06 WordChips still draw Hindi/Gujarati/Tamil/
    // Bengali/Urdu — that is exactly how tofu boxes return.
    let DisplayFont = FontFamily{
        latin := FontMember{res: crate_resource("self:resources/fonts/Fraunces-Variable.ttf") asc: 0.0 desc: 0.0 weight: 620.0}
        ui := FontMember{res: crate_resource("self:resources/fonts/IBMPlexSans-SemiBold.ttf") asc: -0.1 desc: 0.0}
    }
    let StrongFont = FontFamily{
        latin := FontMember{res: crate_resource("self:resources/fonts/IBMPlexSans-SemiBold.ttf") asc: -0.1 desc: 0.0}
    }
    let TextFont = FontFamily{
        latin := FontMember{res: crate_resource("self:resources/fonts/IBMPlexSans-Text.ttf") asc: -0.1 desc: 0.0}
    }
    // Same script faces as apps WorldFonts — every catalogue language's
    // native_name must paint. Latin chrome stays on TextFont/StrongFont.
    let WorldFont = FontFamily{
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
        // Catalogue-only CJK / Hangul subsets (not full LXGW / Noto CJK).
        cjk := FontMember{res: crate_resource("self:resources/fonts/CatalogueCJK.ttf") asc: 0.0 desc: 0.0}
        hangul := FontMember{res: crate_resource("self:resources/fonts/CatalogueHangul.ttf") asc: 0.0 desc: 0.0}
    }

    // The vendored SDF text shader adds `sdf_luma_bias` coverage to the whole
    // glyph quad for light ink, which paints faint boxes behind white type on
    // navy. This page is light-on-dark everywhere, so drop the bias here (the
    // apps' dark-on-light faces never hit it; vendor shader untouched).
    mod.widgets.Label.draw_text.sdf_luma_bias = 0.0
    mod.widgets.Button.draw_text.sdf_luma_bias = 0.0
    mod.widgets.Markdown.draw_text.sdf_luma_bias = 0.0

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

    // Selection band under light ink on navy (#2E5BAD / DESIGN § selection).
    // draw_selection.color is what TextFlow paints (not draw_block.selection_color).
    mod.widgets.TextFlow.draw_selection.color = #x2E5BADFF
    mod.widgets.Markdown.draw_selection.color = #x2E5BADFF
    // Text selection (DESIGN.md § Text selection & copy — same model as the
    // app): every Markdown block selects on its own; the `page` root adds
    // Ctrl/Cmd+A, Copy of all and the right-click Copy / Select all menu.
    // Chrome marks and tap targets keep `selectable: false`.
    mod.widgets.TextFlow.selectable = true
    mod.widgets.Markdown.selectable = true

    let Eyebrow = Markdown{
        selectable: true
        height: Fit
        paragraph_spacing: 0.
        pre_code_spacing: 0.
        draw_selection +: { color: #x2E5BADFF }
        draw_block +: { selection_color: #x2E5BADFF }
        width: Fill
        padding: 0.
        font_color: PetalCyan
        font_size: 12.5
        text_style_normal: theme.font_bold{font_family: StrongFont font_size: 12.5 line_spacing: 1.3}
        text_style_bold: theme.font_bold{font_family: StrongFont font_size: 12.5 line_spacing: 1.3}
        draw_text +: {
            color: PetalCyan
            text_style: theme.font_bold{font_family: StrongFont font_size: 12.5 line_spacing: 1.3}
        }
    }
    let Headline = Markdown{
        selectable: true
        height: Fit
        paragraph_spacing: 0.
        pre_code_spacing: 0.
        draw_selection +: { color: #x2E5BADFF }
        draw_block +: { selection_color: #x2E5BADFF }
        width: Fill
        padding: 0.
        font_color: Ink
        font_size: 52.
        text_style_normal: theme.font_bold{font_family: DisplayFont font_size: 52. line_spacing: 0.94}
        text_style_bold: theme.font_bold{font_family: DisplayFont font_size: 52. line_spacing: 0.94}
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
        draw_selection +: { color: #x2E5BADFF }
        draw_block +: { selection_color: #x2E5BADFF }
        width: Fill
        padding: 0.
        font_color: Muted
        // Lede sits under headlines; keep it above Body but not phone-huge
        // (was 20 — read larger than Khaynjo/Moklanjo on the same handset).
        font_size: 18.
        text_style_normal: theme.font_regular{font_family: TextFont font_size: 18. line_spacing: 1.45}
        text_style_bold: theme.font_bold{font_family: StrongFont font_size: 18. line_spacing: 1.45}
        draw_text +: {
            color: Muted
            text_style: theme.font_regular{font_family: TextFont font_size: 18. line_spacing: 1.45}
        }
    }
    // Normal body across Makepad sites: 15.5 (Khaynjo/Moklanjo target).
    let Body = Markdown{
        selectable: true
        height: Fit
        paragraph_spacing: 0.
        pre_code_spacing: 0.
        draw_selection +: { color: #x2E5BADFF }
        draw_block +: { selection_color: #x2E5BADFF }
        width: Fill
        padding: 0.
        font_color: Muted
        font_size: 15.5
        text_style_normal: theme.font_regular{font_family: TextFont font_size: 15.5 line_spacing: 1.45}
        text_style_bold: theme.font_bold{font_family: StrongFont font_size: 15.5 line_spacing: 1.45}
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
        draw_selection +: { color: #x2E5BADFF }
        draw_block +: { selection_color: #x2E5BADFF }
        width: Fill
        padding: 0.
        font_color: Ink
        font_size: 16.
        text_style_normal: theme.font_bold{font_family: StrongFont font_size: 16. line_spacing: 1.3}
        text_style_bold: theme.font_bold{font_family: StrongFont font_size: 16. line_spacing: 1.3}
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
        draw_selection +: { color: #x2E5BADFF }
        draw_block +: { selection_color: #x2E5BADFF }
        width: Fill
        padding: 0.
        font_color: Faint
        font_size: 13.
        text_style_normal: theme.font_regular{font_family: TextFont font_size: 13. line_spacing: 1.4}
        text_style_bold: theme.font_bold{font_family: StrongFont font_size: 13. line_spacing: 1.4}
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
        draw_selection +: { color: #x2E5BADFF }
        draw_block +: { selection_color: #x2E5BADFF }
        width: Fill
        padding: 0.
        font_color: Ink
        font_size: 30.
        text_style_normal: theme.font_bold{font_family: DisplayFont font_size: 30. line_spacing: 1.0}
        text_style_bold: theme.font_bold{font_family: DisplayFont font_size: 30. line_spacing: 1.0}
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
    // Footer brand names: same face as Small on the navy band.
    let FooterBrandMark = Button{
        width: Fit height: Fit
        padding: 0.
        align: Align{x: 0.0 y: 0.5}
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
            color: Faint
            color_hover: #xFFFFFF
            color_down: #xFFFFFF
            color_focus: Faint
            text_style: theme.font_regular{font_family: TextFont font_size: 13. line_spacing: 1.4}
        }
    }
    let FooterBrandSep = Markdown{
        selectable: false
        width: Fit height: Fit
        padding: 0.
        paragraph_spacing: 0.
        pre_code_spacing: 0.
        body: " · "
        font_color: Faint
        font_size: 13.
        text_style_normal: theme.font_regular{font_family: TextFont font_size: 13. line_spacing: 1.4}
        draw_text +: {
            color: Faint
            text_style: theme.font_regular{font_family: TextFont font_size: 13. line_spacing: 1.4}
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
            border_color: #x3A5AA8
            border_color_hover: PetalCyanDim
            border_color_down: PetalCyanDim
            border_color_focus: #x3A5AA8
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
    // Full-tile hit (Gotigin / Khaynjo pattern): hover/press wash over the card.
    let CardHit = Button{
        width: Fill height: Fill
        padding: 0.
        text: ""
        grab_key_focus: false
        reset_hover_on_click: true
        label_walk: Walk{width: 0. height: 0.}
        icon_walk: Walk{width: 0. height: 0.}
        draw_bg +: {
            color: #x00000000
            color_hover: #x5A8FFF33
            color_down: #x5A8FFF55
            color_focus: #x00000000
            color_2: #x00000000
            color_2_hover: #x5A8FFF33
            color_2_down: #x5A8FFF55
            color_2_focus: #x00000000
            border_color: #x00000000
            border_color_hover: #x5A8FFF88
            border_color_down: #x5A8FFFCC
            border_color_focus: #x00000000
            border_radius: 14.
            border_size: 1.5
        }
        draw_text +: {
            color: #x00000000
            color_hover: #x00000000
            color_down: #x00000000
            color_focus: #x00000000
        }
    }
    let ProductCard = View{
        width: Fill height: Fit
        flow: Overlay
    }
    let CardLink = Markdown{
        selectable: false
        height: Fit
        width: Fill
        padding: 0.
        paragraph_spacing: 0.
        pre_code_spacing: 0.
        font_color: Clay
        font_size: 14.
        text_style_normal: theme.font_bold{font_family: StrongFont font_size: 14. line_spacing: 1.3}
        text_style_bold: theme.font_bold{font_family: StrongFont font_size: 14. line_spacing: 1.3}
        draw_text +: { color: Clay text_style: theme.font_bold{font_family: StrongFont font_size: 14. line_spacing: 1.3} }
        draw_selection +: { color: #x2E5BADFF }
        draw_block +: { selection_color: #x2E5BADFF }
    }

    // ---- Chapter ground: navy fall-off, a warm glow, faint ledger rules --
    let Chapter = View{
        width: Fill height: Fit
        flow: Down
        spacing: 22
        align: Align{x: 0.0 y: 0.0}
        padding: Inset{left: 64. right: 64. top: 36. bottom: 44.}
        show_bg: true
        draw_bg +: {
            top: instance(#x041534)
            bottom: instance(#x01081A)
            glow: instance(#x2A6BFF)
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
            draw_selection +: { color: #x2E5BADFF }
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
            draw_selection +: { color: #x2E5BADFF }
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
            draw_selection +: { color: #x2E5BADFF }
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
            draw_selection +: { color: #x2E5BADFF }
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
        // Markdown shapes via text_style_* (TextFlow), not only draw_text.
        // draw_text alone left chips on Latin TextFont/StrongFont → tofu.
        // Match khaynjo shelf native: set both.
        lang := Markdown{
            selectable: true
            height: Fit
            paragraph_spacing: 0.
            pre_code_spacing: 0.
            draw_selection +: { color: #x2E5BADFF }
            draw_block +: { selection_color: #x2E5BADFF }

            width: Fit
            padding: 0.
            font_color: Faint
            font_size: 13.
            text_style_normal: theme.font_regular{font_family: WorldFont font_size: 13. line_spacing: 1.2}
            text_style_bold: theme.font_bold{font_family: WorldFont font_size: 13. line_spacing: 1.2}
            draw_text +: {
                color: Faint
                text_style: theme.font_regular{font_family: WorldFont font_size: 13. line_spacing: 1.2}
            }
            body: "Language"
        
        }
        word := Markdown{
            selectable: true
            height: Fit
            paragraph_spacing: 0.
            pre_code_spacing: 0.
            draw_selection +: { color: #x2E5BADFF }
            draw_block +: { selection_color: #x2E5BADFF }
            width: Fit
            padding: 0.
            font_color: Ink
            font_size: 24.
            text_style_normal: theme.font_regular{font_family: WorldFont font_size: 24. line_spacing: 1.15}
            text_style_bold: theme.font_bold{font_family: WorldFont font_size: 24. line_spacing: 1.15}
            draw_text +: {
                color: Ink
                text_style: theme.font_bold{font_family: WorldFont font_size: 24. line_spacing: 1.15}
            }
            body: "Word"
        }
    }

    let AppIcon = Image{
        width: 52 height: 52
        fit: ImageFit.Stretch
    }
    let BrandIcon = Image{
        width: 36 height: 36
        fit: ImageFit.Stretch
    }

    // Chapter dot in the top bar: idle dot, active pill, invisible hit.
    let Dot = View{
        width: 20 height: 24
        flow: Overlay
        align: Align{x: 0.5 y: 0.5}
        idle := RoundedView{width: 7 height: 7 draw_bg +: {color: #x3A4F80 border_radius: 3.5}}
        on := RoundedView{visible: false width: 16 height: 7 draw_bg +: {color: PetalCyan border_radius: 3.5}}
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
                        brand_mark := View{
                            width: Fit height: Fit
                            flow: Right
                            spacing: 10
                            align: Align{y: 0.5}
                            brand_icon := BrandIcon{
                                src: crate_resource("self:resources/media/brand-logo.webp")
                            }
                            brand_name := Label{
                                width: Fit
                                text: "Kumkaj"
                                draw_text +: {color: Ink text_style: theme.font_bold{font_family: DisplayFont font_size: 21.}}
                            }
                        }
                        View{width: Fill height: 1}
                        chapter_name := Label{
                            width: Fit
                            text: "The promise"
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
                        counter := Label{
                            width: Fit
                            text: "1 / 7"
                            draw_text +: {color: Faint text_style: theme.font_bold{font_family: StrongFont font_size: 12.5}}
                        }
                    }

                    page := ScrollYView{
                        selection_root: true
                        select_from_gaps: true
                        width: Fill
                        height: Fill
                        flow: Down

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
                                    Eyebrow{body: "KUMKAJ · BUSINESS OPERATING SYSTEMS"}
                                    Headline{body: "The internet went down.\nThe counter didn't."}
                                    Lede{body: "Kumkaj is the brand for concrete business operating system apps — complete business management software that runs on your own devices. Each product is built for one trade: courier, restaurant, grocery, pharmacy."}
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
                            hint := Small{body: "Scroll or use the dots to jump between sections."}
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
                                pair_top := View{
                                    width: Fill height: Fit
                                    flow: Right
                                    spacing: 20
                                    mok := ProductCard{
                                        Card{
                                            draw_bg.border_color: #x2A4F9A
                                            AppIcon{src: crate_resource("self:resources/media/moklanjo-app-icon.webp")}
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
                                            CardLink{body: "Visit moklanjo.com  →"}
                                        }
                                        to_moklanjo := CardHit{}
                                    }
                                    kha := ProductCard{
                                        Card{
                                            draw_bg.border_color: #x7A2A2A
                                            AppIcon{src: crate_resource("self:resources/media/khaynjo-app-icon.webp")}
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
                                            CardLink{body: "Visit khaynjo.com  →"}
                                        }
                                        to_khaynjo := CardHit{}
                                    }
                                }
                                pair_bot := View{
                                    width: Fill height: Fit
                                    flow: Right
                                    spacing: 20
                                    kir := ProductCard{
                                        Card{
                                            draw_bg.border_color: #xCD380E
                                            AppIcon{src: crate_resource("self:resources/media/kirano-app-icon.webp")}
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
                                            CardLink{body: "Visit kirano.shop  →"}
                                        }
                                        to_kirano := CardHit{}
                                    }
                                    tab := ProductCard{
                                        Card{
                                            draw_bg.border_color: #x023637
                                            AppIcon{src: crate_resource("self:resources/media/tabiyat-app-icon.webp")}
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
                                            CardLink{body: "Visit tabiyat.shop  →"}
                                        }
                                        to_tabiyat := CardHit{}
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
                            // The gate is the View: a Markdown has no `visible`, so on the
                            // block itself the switch did nothing and the sentence was live.
                            books_lan := View{
                                visible: false
                                width: Fill height: Fit
                                Body{body: "Extra counters and tablets join over your own Wi‑Fi — with no internet in between."}
                            }
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
                                    // Fit: the price sits at the right edge of the row.
                                    // With no width it took half the row and started mid-card.
                                    Markdown{
                                        selectable: true
                                        width: Fit height: Fit
                                        paragraph_spacing: 0.
                                        pre_code_spacing: 0.
                                        draw_selection +: { color: #x2E5BADFF }
                                        draw_block +: { selection_color: #x2E5BADFF }
                                        padding: 0.
                                        font_color: Good
                                        font_size: 26.
                                        body: "₹0, forever"
                                        draw_text +: {color: Good text_style: theme.font_bold{font_family: DisplayFont font_size: 26.}}
                                    }
                                }
                                RoundedView{width: Fill height: 1 draw_bg +: {color: Rule border_radius: 0.}}
                                // ROWS stacks this under ~520 content px so the
                                // Planned pill is never clipped on landscape phones.
                                cloud_row := View{
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
                                            // Fit, like the pill around it: with no width the text took Fill
                                            // inside a Fit pill and collapsed to an empty sliver.
                                            width: Fit height: Fit
                                            paragraph_spacing: 0.
                                            pre_code_spacing: 0.
                                            draw_selection +: { color: #x2E5BADFF }
                                            draw_block +: { selection_color: #x2E5BADFF }
                                            padding: 0.
                                            font_color: PetalCyan
                                            font_size: 13.
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
                            Lede{body: "The same 115-language catalogue as every Kumkaj app — Arabic and Urdu right to left. Native names below; “Food ready” samples for Hindi, Gujarati, Tamil, Bengali, Urdu, and English:"}
                            words := View{
                                width: Fill height: Fit
                                flow: Right{wrap: true}
                                spacing: 12
                                // BEGIN CATALOGUE_LANG_CHIPS
                                WordChip{lang.body: "English" word.body: "Food ready"}
                                WordChip{lang.body: "हिन्दी" word.body: "खाना तैयार"}
                                WordChip{lang.body: "বাংলা" word.body: "খাবার তৈরি"}
                                WordChip{lang.body: "తెలుగు" word.body: "తెలుగు"}
                                WordChip{lang.body: "मराठी" word.body: "मराठी"}
                                WordChip{lang.body: "தமிழ்" word.body: "உணவு தயார்"}
                                WordChip{lang.body: "اردو" word.body: "کھانا تیار"}
                                WordChip{lang.body: "ગુજરાતી" word.body: "જમવાનું તૈયાર"}
                                WordChip{lang.body: "ಕನ್ನಡ" word.body: "ಕನ್ನಡ"}
                                WordChip{lang.body: "മലയാളം" word.body: "മലയാളം"}
                                WordChip{lang.body: "ଓଡ଼ିଆ" word.body: "ଓଡ଼ିଆ"}
                                WordChip{lang.body: "ਪੰਜਾਬੀ" word.body: "ਪੰਜਾਬੀ"}
                                WordChip{lang.body: "অসমীয়া" word.body: "অসমীয়া"}
                                WordChip{lang.body: "मैथिली" word.body: "मैथिली"}
                                WordChip{lang.body: "संस्कृतम्" word.body: "संस्कृतम्"}
                                WordChip{lang.body: "كٲشُر" word.body: "كٲشُر"}
                                WordChip{lang.body: "नेपाली" word.body: "नेपाली"}
                                WordChip{lang.body: "سنڌي" word.body: "سنڌي"}
                                WordChip{lang.body: "कोंकणी" word.body: "कोंकणी"}
                                WordChip{lang.body: "डोगरी" word.body: "डोगरी"}
                                WordChip{lang.body: "ꯃꯤꯇꯩꯂꯣꯟ" word.body: "ꯃꯤꯇꯩꯂꯣꯟ"}
                                WordChip{lang.body: "ᱥᱟᱱᱛᱟᱲᱤ" word.body: "ᱥᱟᱱᱛᱟᱲᱤ"}
                                WordChip{lang.body: "बर'" word.body: "बर'"}
                                WordChip{lang.body: "भोजपुरी" word.body: "भोजपुरी"}
                                WordChip{lang.body: "अवधी" word.body: "अवधी"}
                                WordChip{lang.body: "मगही" word.body: "मगही"}
                                WordChip{lang.body: "छत्तीसगढ़ी" word.body: "छत्तीसगढ़ी"}
                                WordChip{lang.body: "राजस्थानी" word.body: "राजस्थानी"}
                                WordChip{lang.body: "ತುಳು" word.body: "ತುಳು"}
                                WordChip{lang.body: "ককবরক" word.body: "ককবরক"}
                                WordChip{lang.body: "Mizo ṭawng" word.body: "Mizo ṭawng"}
                                WordChip{lang.body: "Khasi" word.body: "Khasi"}
                                WordChip{lang.body: "गोंयची कोंकणी" word.body: "गोंयची कोंकणी"}
                                WordChip{lang.body: "བོད་སྐད་" word.body: "བོད་སྐད་"}
                                WordChip{lang.body: "සිංහල" word.body: "සිංහල"}
                                WordChip{lang.body: "ދިވެހި" word.body: "ދިވެހި"}
                                WordChip{lang.body: "پښتو" word.body: "پښتو"}
                                WordChip{lang.body: "فارسی" word.body: "فارسی"}
                                WordChip{lang.body: "中文" word.body: "中文"}
                                WordChip{lang.body: "繁體中文" word.body: "繁體中文"}
                                WordChip{lang.body: "粵語" word.body: "粵語"}
                                WordChip{lang.body: "日本語" word.body: "日本語"}
                                WordChip{lang.body: "한국어" word.body: "한국어"}
                                WordChip{lang.body: "Tiếng Việt" word.body: "Tiếng Việt"}
                                WordChip{lang.body: "ไทย" word.body: "ไทย"}
                                WordChip{lang.body: "Bahasa Indonesia" word.body: "Bahasa Indonesia"}
                                WordChip{lang.body: "Bahasa Melayu" word.body: "Bahasa Melayu"}
                                WordChip{lang.body: "Filipino" word.body: "Filipino"}
                                WordChip{lang.body: "မြန်မာ" word.body: "မြန်မာ"}
                                WordChip{lang.body: "ខ្មែរ" word.body: "ខ្មែរ"}
                                WordChip{lang.body: "ລາວ" word.body: "ລາວ"}
                                WordChip{lang.body: "العربية" word.body: "العربية"}
                                WordChip{lang.body: "עברית" word.body: "עברית"}
                                WordChip{lang.body: "Türkçe" word.body: "Türkçe"}
                                WordChip{lang.body: "Azərbaycan" word.body: "Azərbaycan"}
                                WordChip{lang.body: "Қазақша" word.body: "Қазақша"}
                                WordChip{lang.body: "Oʻzbekcha" word.body: "Oʻzbekcha"}
                                WordChip{lang.body: "Кыргызча" word.body: "Кыргызча"}
                                WordChip{lang.body: "Тоҷикӣ" word.body: "Тоҷикӣ"}
                                WordChip{lang.body: "Türkmençe" word.body: "Türkmençe"}
                                WordChip{lang.body: "Монгол" word.body: "Монгол"}
                                WordChip{lang.body: "Español" word.body: "Español"}
                                WordChip{lang.body: "Français" word.body: "Français"}
                                WordChip{lang.body: "Português" word.body: "Português"}
                                WordChip{lang.body: "Deutsch" word.body: "Deutsch"}
                                WordChip{lang.body: "Italiano" word.body: "Italiano"}
                                WordChip{lang.body: "Русский" word.body: "Русский"}
                                WordChip{lang.body: "Українська" word.body: "Українська"}
                                WordChip{lang.body: "Polski" word.body: "Polski"}
                                WordChip{lang.body: "Nederlands" word.body: "Nederlands"}
                                WordChip{lang.body: "Română" word.body: "Română"}
                                WordChip{lang.body: "Magyar" word.body: "Magyar"}
                                WordChip{lang.body: "Čeština" word.body: "Čeština"}
                                WordChip{lang.body: "Slovenčina" word.body: "Slovenčina"}
                                WordChip{lang.body: "Български" word.body: "Български"}
                                WordChip{lang.body: "Српски" word.body: "Српски"}
                                WordChip{lang.body: "Hrvatski" word.body: "Hrvatski"}
                                WordChip{lang.body: "Bosanski" word.body: "Bosanski"}
                                WordChip{lang.body: "Slovenščina" word.body: "Slovenščina"}
                                WordChip{lang.body: "Ελληνικά" word.body: "Ελληνικά"}
                                WordChip{lang.body: "Svenska" word.body: "Svenska"}
                                WordChip{lang.body: "Norsk" word.body: "Norsk"}
                                WordChip{lang.body: "Dansk" word.body: "Dansk"}
                                WordChip{lang.body: "Suomi" word.body: "Suomi"}
                                WordChip{lang.body: "Íslenska" word.body: "Íslenska"}
                                WordChip{lang.body: "Gaeilge" word.body: "Gaeilge"}
                                WordChip{lang.body: "Cymraeg" word.body: "Cymraeg"}
                                WordChip{lang.body: "Català" word.body: "Català"}
                                WordChip{lang.body: "Euskara" word.body: "Euskara"}
                                WordChip{lang.body: "Galego" word.body: "Galego"}
                                WordChip{lang.body: "Lietuvių" word.body: "Lietuvių"}
                                WordChip{lang.body: "Latviešu" word.body: "Latviešu"}
                                WordChip{lang.body: "Eesti" word.body: "Eesti"}
                                WordChip{lang.body: "Malti" word.body: "Malti"}
                                WordChip{lang.body: "Shqip" word.body: "Shqip"}
                                WordChip{lang.body: "Македонски" word.body: "Македонски"}
                                WordChip{lang.body: "Беларуская" word.body: "Беларуская"}
                                WordChip{lang.body: "ქართული" word.body: "ქართული"}
                                WordChip{lang.body: "Հայերեն" word.body: "Հայերեն"}
                                WordChip{lang.body: "Kiswahili" word.body: "Kiswahili"}
                                WordChip{lang.body: "አማርኛ" word.body: "አማርኛ"}
                                WordChip{lang.body: "Hausa" word.body: "Hausa"}
                                WordChip{lang.body: "Yorùbá" word.body: "Yorùbá"}
                                WordChip{lang.body: "Igbo" word.body: "Igbo"}
                                WordChip{lang.body: "isiZulu" word.body: "isiZulu"}
                                WordChip{lang.body: "isiXhosa" word.body: "isiXhosa"}
                                WordChip{lang.body: "Afrikaans" word.body: "Afrikaans"}
                                WordChip{lang.body: "Soomaali" word.body: "Soomaali"}
                                WordChip{lang.body: "Ikinyarwanda" word.body: "Ikinyarwanda"}
                                WordChip{lang.body: "Malagasy" word.body: "Malagasy"}
                                WordChip{lang.body: "Português (Brasil)" word.body: "Português (Brasil)"}
                                WordChip{lang.body: "Español (México)" word.body: "Español (México)"}
                                WordChip{lang.body: "Kreyòl ayisyen" word.body: "Kreyòl ayisyen"}
                                WordChip{lang.body: "Runa Simi" word.body: "Runa Simi"}
                                WordChip{lang.body: "Avañe'ẽ" word.body: "Avañe'ẽ"}
                                // END CATALOGUE_LANG_CHIPS
                            }
                            Small{body: "Same shaping engine as the apps, so joined letters and right-to-left lines come out right."}
                        }

                        // 07 — who makes it, and the product choice again
                        gotigin := Chapter{
                            Eyebrow{body: "07 · WHO MAKES IT"}
                            Headline{body: "Made under Gotigin."}
                            names := View{
                                width: Fill height: Fit
                                flow: Down
                                spacing: 16
                                View{
                                    width: Fill height: Fit
                                    flow: Down
                                    spacing: 4
                                    Eyebrow{body: "TRADEMARK"}
                                    Strong{body: "Gotigin"}
                                    Small{body: "Memon for “find it” · pronounced go-tee-geen"}
                                    Small{body: "Where Technology Meets Discovery"}
                                }
                                View{
                                    width: Fill height: Fit
                                    flow: Down
                                    spacing: 4
                                    Eyebrow{body: "COMPANY"}
                                    Strong{body: "GOTIGIN SOFTWARE & HARDWARE PRIVATE LIMITED"}
                                    Small{body: "support@gotigin.com"}
                                    Small{body: "Director · mearaj@gmail.com"}
                                }
                                View{
                                    width: Fill height: Fit
                                    flow: Down
                                    spacing: 4
                                    Eyebrow{body: "BRAND"}
                                    Strong{body: "Kumkaj"}
                                    Small{body: "Concrete business operating system apps"}
                                }
                                View{
                                    width: Fill height: Fit
                                    flow: Down
                                    spacing: 4
                                    Eyebrow{body: "PRODUCTS"}
                                    Strong{body: "Moklanjo · Khaynjo · Kirano · Tabiyat"}
                                    Small{body: "Courier · restaurant · grocery · pharmacy"}
                                }
                            }
                            product_grid := View{
                                width: Fill height: Fit
                                flow: Down
                                spacing: 16
                                product_row_1 := View{
                                    width: Fill height: Fit
                                    flow: Right
                                    spacing: 16
                                    pick_mok := ProductCard{
                                        Card{
                                            draw_bg.border_color: #x2A4F9A
                                            AppIcon{src: crate_resource("self:resources/media/moklanjo-app-icon.webp")}
                                            CardName{body: "Moklanjo"}
                                            Body{body: "Courier desk — book, label, hand over, settle."}
                                            CardLink{body: "Visit moklanjo.com  →"}
                                        }
                                        close_moklanjo := CardHit{}
                                    }
                                    pick_kha := ProductCard{
                                        Card{
                                            draw_bg.border_color: #x7A2A2A
                                            AppIcon{src: crate_resource("self:resources/media/khaynjo-app-icon.webp")}
                                            CardName{body: "Khaynjo"}
                                            Body{body: "Restaurant system — table, kitchen, bill, day close."}
                                            CardLink{body: "Visit khaynjo.com  →"}
                                        }
                                        close_khaynjo := CardHit{}
                                    }
                                }
                                product_row_2 := View{
                                    width: Fill height: Fit
                                    flow: Right
                                    spacing: 16
                                    pick_kir := ProductCard{
                                        Card{
                                            draw_bg.border_color: #xCD380E
                                            AppIcon{src: crate_resource("self:resources/media/kirano-app-icon.webp")}
                                            CardName{body: "Kirano"}
                                            Body{body: "Grocery counter — sell, khata, stock, day book."}
                                            CardLink{body: "Visit kirano.shop  →"}
                                        }
                                        close_kirano := CardHit{}
                                    }
                                    pick_tab := ProductCard{
                                        Card{
                                            draw_bg.border_color: #x023637
                                            AppIcon{src: crate_resource("self:resources/media/tabiyat-app-icon.webp")}
                                            CardName{body: "Tabiyat"}
                                            Body{body: "Pharmacy counter — batch, expiry, bill, day close."}
                                            CardLink{body: "Visit tabiyat.shop  →"}
                                        }
                                        close_tabiyat := CardHit{}
                                    }
                                }
                            }
                        }

                        footer := View{
                            width: Fill height: Fit
                            flow: Down
                            spacing: 0
                            align: Align{x: 0.0 y: 0.0}
                            show_bg: true
                            draw_bg +: {color: NavyDeep}
                            footer_row := View{
                                width: Fill height: Fit
                                flow: Right
                                spacing: 14
                                align: Align{y: 0.5}
                                footer_brand := View{
                                    width: Fill height: Fit
                                    flow: Right
                                    spacing: 10
                                    align: Align{y: 0.5}
                                    footer_icon := BrandIcon{
                                        src: crate_resource("self:resources/media/brand-logo.webp")
                                    }
                                    View{
                                        width: Fill height: Fit
                                        flow: Down
                                        spacing: 2
                                        footer_attribution := Small{
                                            width: Fill
                                            body: "© 2026 GOTIGIN SOFTWARE & HARDWARE PRIVATE LIMITED"
                                        }
                                        View{
                                            width: Fit height: Fit
                                            flow: Right
                                            spacing: 0
                                            align: Align{y: 0.5}
                                            foot_gotigin := FooterBrandMark{text: "Gotigin"}
                                            FooterBrandSep{}
                                            foot_kumkaj := FooterBrandMark{text: "Kumkaj"}
                                        }
                                    }
                                }
                                footer_links := View{
                                    width: Fit height: Fit
                                    flow: Right
                                    spacing: 10
                                    to_privacy := GhostBtn{text: "Privacy"}
                                    to_terms := GhostBtn{text: "Terms"}
                                    to_cancellation := GhostBtn{text: "Cancellation"}
                                    to_shop := GhostBtn{text: "Cloud shop"}
                                    to_account := GhostBtn{text: "Account"}
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
