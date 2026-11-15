// khaynjo.com — one Makepad 2 `script_mod!` (included from main.rs).
// A scrolling product page (2026-11-12 redesign; was a 10-slide deck):
// hero with the live floor toy → how a service runs (5 steps, the app's
// visit colours) → features → menus (catalogue templates + world shelf) →
// offline → pricing → FAQ → closing CTA + footer. Story and claims:
// apps/khaynjo/docs/KHAYNJO_WEBSITE.md (claim ids in comments); palette:
// docs/DESIGN.md "Color tokens — Khaynjo"; `lan-pilot` sentences stay
// hidden until the owner's switch. Illustrations carry an Example tag;
// icons are the app's own sidebar art (copied, not imported).
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
    // World shelf (chapter 8) native names: TextFont plus every script the
    // 199-country list needs. Its own family so these fonts load only when
    // chapter 8 draws (a family loads all members on first use).
    let WorldFont = FontFamily{
        latin := FontMember{res: crate_resource("self:resources/fonts/IBMPlexSans-Text.ttf") asc: -0.1 desc: 0.0}
        gujarati := FontMember{res: crate_resource("self:resources/fonts/NotoSansGujarati-Regular.ttf") asc: 0.0 desc: 0.0}
        devanagari := FontMember{res: crate_resource("self:resources/fonts/NotoSansDevanagari-Regular.ttf") asc: 0.0 desc: 0.0}
        tamil := FontMember{res: crate_resource("self:resources/fonts/NotoSansTamil-Regular.ttf") asc: 0.0 desc: 0.0}
        bengali := FontMember{res: crate_resource("self:resources/fonts/NotoSansBengali-Regular.ttf") asc: 0.0 desc: 0.0}
        arabic := FontMember{res: crate_resource("self:resources/fonts/NotoSansArabic-Regular.ttf") asc: 0.0 desc: 0.0}
        hebrew := FontMember{res: crate_resource("self:resources/fonts/NotoSansHebrew-Regular.ttf") asc: 0.0 desc: 0.0}
        thai := FontMember{res: crate_resource("self:resources/fonts/NotoSansThai-Regular.ttf") asc: 0.0 desc: 0.0}
        armenian := FontMember{res: crate_resource("self:resources/fonts/NotoSansArmenian-Regular.ttf") asc: 0.0 desc: 0.0}
        ethiopic := FontMember{res: crate_resource("self:resources/fonts/NotoSansEthiopic-Regular.ttf") asc: 0.0 desc: 0.0}
        georgian := FontMember{res: crate_resource("self:resources/fonts/NotoSansGeorgian-Regular.ttf") asc: 0.0 desc: 0.0}
        khmer := FontMember{res: crate_resource("self:resources/fonts/NotoSansKhmer-Regular.ttf") asc: 0.0 desc: 0.0}
        lao := FontMember{res: crate_resource("self:resources/fonts/NotoSansLao-Regular.ttf") asc: 0.0 desc: 0.0}
        myanmar := FontMember{res: crate_resource("self:resources/fonts/NotoSansMyanmar-Regular.ttf") asc: 0.0 desc: 0.0}
        sinhala := FontMember{res: crate_resource("self:resources/fonts/NotoSansSinhala-Regular.ttf") asc: 0.0 desc: 0.0}
        thaana := FontMember{res: crate_resource("self:resources/fonts/NotoSansThaana-Regular.ttf") asc: 0.0 desc: 0.0}
        tibetan := FontMember{res: crate_resource("self:resources/fonts/NotoSerifTibetan-Regular.ttf") asc: 0.0 desc: 0.0}
        // 12 KB subset of Noto Sans CJK: only the Han / kana / Hangul the
        // shelf uses (build-world-shelf.sh checks coverage).
        cjk := FontMember{res: crate_resource("self:resources/fonts/NotoSansCJK-WorldShelf.otf") asc: 0.0 desc: 0.0}
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

    // Selection band under ink: light ServedFill (T5) so maroon/ink stay readable.
    // draw_selection.color is what TextFlow paints (not draw_block.selection_color).
    mod.widgets.TextFlow.draw_selection.color = #xCFE3FFFF
    mod.widgets.Markdown.draw_selection.color = #xCFE3FFFF
    // Text selection (DESIGN.md § Text selection & copy — same model as the
    // app): every Markdown block selects on its own (drag; double-click a word;
    // long-press on touch); the `page` root adds Ctrl/Cmd+A, Copy of all and the
    // right-click Copy / Select all menu. Tap targets (floor tiles, buttons) and
    // chrome marks keep `selectable: false`.
    mod.widgets.TextFlow.selectable = true
    mod.widgets.Markdown.selectable = true

    // TextFlow paints from `font_color` (not only draw_text.color) — set both.
    let Eyebrow = Markdown{
        selectable: true
        height: Fit
        paragraph_spacing: 0.
        pre_code_spacing: 0.
        draw_selection +: { color: #xCFE3FFFF }
        draw_block +: { selection_color: #xCFE3FFFF }
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
        draw_selection +: { color: #xCFE3FFFF }
        draw_block +: { selection_color: #xCFE3FFFF }
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
        draw_selection +: { color: #xCFE3FFFF }
        draw_block +: { selection_color: #xCFE3FFFF }
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
        draw_selection +: { color: #xCFE3FFFF }
        draw_block +: { selection_color: #xCFE3FFFF }
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
        draw_selection +: { color: #xCFE3FFFF }
        draw_block +: { selection_color: #xCFE3FFFF }
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
        draw_selection +: { color: #xCFE3FFFF }
        draw_block +: { selection_color: #xCFE3FFFF }
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
        draw_selection +: { color: #xCFE3FFFF }
        draw_block +: { selection_color: #xCFE3FFFF }
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

    // ---- Section type ---------------------------------------------------
    // Section titles (H2) and card titles (H3) — Baloo, maroon on cream.
    let H2 = Markdown{
        selectable: true
        height: Fit
        paragraph_spacing: 0.
        pre_code_spacing: 0.
        draw_selection +: { color: #xCFE3FFFF }
        draw_block +: { selection_color: #xCFE3FFFF }
        width: Fill
        padding: 0.
        font_color: Maroon
        font_size: 38.
        text_style_normal: theme.font_bold{font_family: DisplayFont font_size: 38. line_spacing: 0.8}
        text_style_bold: theme.font_bold{font_family: DisplayFont font_size: 38. line_spacing: 0.8}
        draw_text +: {
            color: Maroon
            text_style: theme.font_bold{font_family: DisplayFont font_size: 38. line_spacing: 0.8}
        }
    }
    let H3 = Markdown{
        selectable: true
        height: Fit
        paragraph_spacing: 0.
        pre_code_spacing: 0.
        draw_selection +: { color: #xCFE3FFFF }
        draw_block +: { selection_color: #xCFE3FFFF }
        width: Fill
        padding: 0.
        font_color: Maroon
        font_size: 24.
        text_style_normal: theme.font_bold{font_family: DisplayFont font_size: 24. line_spacing: 0.85}
        text_style_bold: theme.font_bold{font_family: DisplayFont font_size: 24. line_spacing: 0.85}
        draw_text +: {
            color: Maroon
            text_style: theme.font_bold{font_family: DisplayFont font_size: 24. line_spacing: 0.85}
        }
    }
    let HeroTitle = Headline{
        font_size: 56.
        text_style_normal: theme.font_bold{font_family: DisplayFont font_size: 56. line_spacing: 0.72}
        text_style_bold: theme.font_bold{font_family: DisplayFont font_size: 56. line_spacing: 0.72}
        draw_text +: {text_style: theme.font_bold{font_family: DisplayFont font_size: 56. line_spacing: 0.72}}
    }
    let CardTitle = Strong{
        font_size: 17.
        text_style_normal: theme.font_bold{font_family: StrongFont font_size: 17. line_spacing: 1.25}
        text_style_bold: theme.font_bold{font_family: StrongFont font_size: 17. line_spacing: 1.25}
        draw_text +: {text_style: theme.font_bold{font_family: StrongFont font_size: 17. line_spacing: 1.25}}
    }
    // Light ink for the deep-red bands (offline, closing CTA, footer).
    let OnRedTitle = H2{
        font_color: #xFFFFFF
        draw_text +: {color: #xFFFFFF}
        // Light ink: a dark band (light blue would hide it).
        draw_selection +: { color: #x3D0507FF }
    }
    let OnRedBody = Body{
        font_color: #xF6DCCB
        draw_text +: {color: #xF6DCCB}
        draw_selection +: { color: #x3D0507FF }
    }
    let OnRedEyebrow = Eyebrow{
        font_color: GoldSoft
        draw_text +: {color: GoldSoft}
        draw_selection +: { color: #x3D0507FF }
    }

    // ---- Buttons ---------------------------------------------------------
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
            color: Well
            color_hover: WellWarm
            color_down: WellDeep
            color_focus: Well
            color_2: Well
            color_2_hover: WellWarm
            color_2_down: WellDeep
            color_2_focus: Well
            border_color: #xD8BBA6
            border_color_hover: #xC9A58C
            border_color_down: #xC9A58C
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
    // Gold CTA for the deep-red bands (white-on-red primary would vanish).
    // grab_key_focus false: avoid stealing focus from page scroll / copy.
    let GoldBtn = PrimaryBtn{
        grab_key_focus: false
        draw_bg +: {
            color: Gold
            color_hover: GoldSoft
            color_down: #xE08A1A
            color_focus: Gold
            color_2: Gold
            color_2_hover: GoldSoft
            color_2_down: #xE08A1A
            color_2_focus: Gold
            border_color: Gold
        }
        draw_text +: {
            color: Maroon
            color_hover: Maroon
            color_down: Maroon
            color_focus: Maroon
        }
    }
    let GhostOnRedBtn = SecondaryBtn{
        draw_bg +: {
            color: #x00000000
            color_hover: #xFFFFFF22
            color_down: #xFFFFFF33
            color_focus: #x00000000
            color_2: #x00000000
            color_2_hover: #xFFFFFF22
            color_2_down: #xFFFFFF33
            color_2_focus: #x00000000
            border_color: #xF6DCCB
            border_color_hover: #xFFFFFF
            border_color_down: #xFFFFFF
            border_color_focus: #xF6DCCB
        }
        draw_text +: {
            color: #xFFFFFF
            color_hover: #xFFFFFF
            color_down: #xFFFFFF
            color_focus: #xFFFFFF
        }
    }
    // Top-bar section link: quiet text on the red bar.
    let NavLink = Button{
        width: Fit height: Fit
        padding: Inset{left: 10. right: 10. top: 8. bottom: 8.}
        align: Align{x: 0.5 y: 0.5}
        grab_key_focus: false
        reset_hover_on_click: true
        label_walk: Walk{width: Fit height: Fit}
        icon_walk: Walk{width: 0. height: 0.}
        draw_bg +: {
            color: #x00000000
            color_hover: #xFFFFFF1F
            color_down: #xFFFFFF33
            color_focus: #x00000000
            color_2: #x00000000
            color_2_hover: #xFFFFFF1F
            color_2_down: #xFFFFFF33
            color_2_focus: #x00000000
            border_color: #x00000000
            border_radius: 10.
            border_size: 0.
        }
        draw_text +: {
            color: #xF6DCCB
            color_hover: #xFFFFFF
            color_down: #xFFFFFF
            color_focus: #xF6DCCB
            text_style: theme.font_regular{font_family: StrongFont font_size: 14.}
        }
    }
    let BarCta = GoldBtn{
        padding: Inset{left: 16. right: 16. top: 9. bottom: 9.}
        draw_text +: {text_style: theme.font_bold{font_family: StrongFont font_size: 14.}}
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
    let PagerBtn = SecondaryBtn{
        padding: Inset{left: 14. right: 14. top: 9. bottom: 9.}
        draw_text +: {text_style: theme.font_bold{font_family: StrongFont font_size: 14.}}
    }

    // ---- Grounds --------------------------------------------------------
    // Hero ground: cream with a pool of warm lamp light top-left.
    let LampBand = View{
        width: Fill height: Fit
        flow: Down
        align: Align{x: 0.5 y: 0.0}
        show_bg: true
        draw_bg +: {
            pixel: fn() {
                let aspect = self.rect_size.x / max(1.0, self.rect_size.y)
                let d = length(vec2((self.pos.x - 0.15) * aspect, self.pos.y - 0.05))
                let lamp = 1.0 - smoothstep(0.0, 1.3, d)
                let base = mix(vec3(0.976, 0.957, 0.941), vec3(0.965, 0.925, 0.894), self.pos.y)
                return vec4(mix(base, vec3(0.992, 0.890, 0.733), lamp * 0.5), 1.0)
            }
        }
    }
    // Flat section bands (RoundedView radius 0 — a Fit View skips show_bg).
    let CreamBand = RoundedView{
        width: Fill height: Fit
        flow: Down
        align: Align{x: 0.5 y: 0.0}
        draw_bg +: {color: Cream border_radius: 0.}
    }
    let PaperBand = CreamBand{draw_bg +: {color: Paper}}
    let WarmBand = CreamBand{draw_bg +: {color: #xF8EADF}}
    let RedBand = CreamBand{draw_bg +: {color: Red}}
    let MaroonBand = CreamBand{draw_bg +: {color: Maroon}}
    // Content column inside a band; side padding + max width set in Rust.
    let Wrap = View{
        width: Fill height: Fit
        flow: Down
        spacing: 28
        padding: Inset{left: 64. right: 64. top: 72. bottom: 72.}
    }
    let SecHead = View{
        width: Fill height: Fit
        flow: Down
        spacing: 12
    }
    let Rule = RoundedView{width: Fill height: 1 draw_bg +: {color: WellLine border_radius: 0.}}

    // ---- Cards, pills, props ---------------------------------------------
    let WarmCard = RoundedView{
        width: Fill height: Fit
        flow: Down
        spacing: 10
        padding: Inset{left: 22. right: 22. top: 20. bottom: 22.}
        draw_bg +: {
            color: Well
            border_color: WellLine
            border_size: 1.
            border_radius: 18.
        }
    }
    let Pill = RoundedView{
        width: Fit height: Fit
        padding: Inset{left: 12. right: 12. top: 6. bottom: 7.}
        draw_bg +: {color: WellWarm border_color: #xEBCDB6 border_size: 1. border_radius: 14.}
        label := Markdown{
            selectable: true
            width: Fit height: Fit
            padding: 0.
            paragraph_spacing: 0.
            pre_code_spacing: 0.
            body: "Pill"
            font_color: Maroon
            font_size: 13.
            text_style_normal: theme.font_bold{font_family: StrongFont font_size: 13.}
            text_style_bold: theme.font_bold{font_family: StrongFont font_size: 13.}
            draw_text +: {color: Maroon text_style: theme.font_bold{font_family: StrongFont font_size: 13.}}
            draw_selection +: { color: #xCFE3FFFF }
            draw_block +: { selection_color: #xCFE3FFFF }
        }
    }
    // Trust line: a green check dot + short claim.
    let Trust = View{
        width: Fit height: Fit
        flow: Right
        spacing: 8
        align: Align{y: 0.5}
        RoundedView{width: 10 height: 10 draw_bg +: {color: BillLine border_radius: 5.}}
        label := Markdown{
            selectable: true
            width: Fit height: Fit
            padding: 0.
            paragraph_spacing: 0.
            pre_code_spacing: 0.
            body: "Claim"
            font_color: Ink
            font_size: 14.
            text_style_normal: theme.font_bold{font_family: StrongFont font_size: 14.}
            text_style_bold: theme.font_bold{font_family: StrongFont font_size: 14.}
            draw_text +: {color: Ink text_style: theme.font_bold{font_family: StrongFont font_size: 14.}}
            draw_selection +: { color: #xCFE3FFFF }
            draw_block +: { selection_color: #xCFE3FFFF }
        }
    }
    let ExampleTag = RoundedView{
        width: Fit height: Fit
        padding: Inset{left: 10. right: 10. top: 4. bottom: 5.}
        draw_bg +: {color: WellWarm border_radius: 10.}
        Markdown{
            selectable: true
            width: Fit height: Fit
            padding: 0.
            paragraph_spacing: 0.
            pre_code_spacing: 0.
            body: "Example"
            font_color: Muted
            font_size: 12.
            text_style_normal: theme.font_regular{font_family: TextFont font_size: 12.}
            draw_text +: {color: Muted text_style: theme.font_regular{font_family: TextFont font_size: 12.}}
            draw_selection +: { color: #xCFE3FFFF }
            draw_block +: { selection_color: #xCFE3FFFF }
        }
    }
    // Step number disc for the service story.
    let StepDisc = RoundedView{
        width: 34 height: 34
        align: Align{x: 0.5 y: 0.5}
        draw_bg +: {color: Red border_radius: 17.}
        num := Markdown{
            // Step number on a red disc: a mark, not copy.
            selectable: false
            width: Fit height: Fit
            padding: 0.
            paragraph_spacing: 0.
            pre_code_spacing: 0.
            body: "1"
            font_color: #xFFFFFF
            font_size: 15.
            text_style_normal: theme.font_bold{font_family: StrongFont font_size: 15.}
            text_style_bold: theme.font_bold{font_family: StrongFont font_size: 15.}
            draw_text +: {color: #xFFFFFF text_style: theme.font_bold{font_family: StrongFont font_size: 15.}}
            draw_selection +: { color: #xCFE3FFFF }
            draw_block +: { selection_color: #xCFE3FFFF }
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
        head := Strong{body: "Point"}
        line := Body{body: "Detail"}
    }
    // Feature card: the app's own sidebar icon, a title, two lines.
    let FeatureCard = WarmCard{
        spacing: 10
        icon := Image{
            width: 44 height: 44
            fit: ImageFit.Stretch
            src: crate_resource("self:resources/icons/sell.webp")
        }
        title := CardTitle{body: "Feature"}
        line := Body{body: "Detail"}
    }
    let FaqItem = WarmCard{
        spacing: 8
        q := CardTitle{body: "Question?"}
        a := Body{body: "Answer."}
    }
    // Price card numbers.
    let BigPrice = Markdown{
        selectable: true
        height: Fit
        paragraph_spacing: 0.
        pre_code_spacing: 0.
        draw_selection +: { color: #xCFE3FFFF }
        draw_block +: { selection_color: #xCFE3FFFF }
        width: Fill
        padding: 0.
        font_color: Red
        font_size: 64.
        text_style_normal: theme.font_bold{font_family: DisplayFont font_size: 64. line_spacing: 0.7}
        draw_text +: {
            color: Red
            text_style: theme.font_bold{font_family: DisplayFont font_size: 64. line_spacing: 0.7}
        }
    }

    // Floor toy tile: the app's seven visit layers, name / word / meta, hit.
    let TableTile = View{
        width: 150 height: 108
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
            // Tap target (HitBtn on top owns the tap): tile words are not copy,
            // so they stay out of drag and Select all (DESIGN tap-target rule).
            name := Markdown{
                selectable: false
                width: Fit height: Fit
                padding: 0.
                paragraph_spacing: 0.
                pre_code_spacing: 0.
                body: "T1"
                font_color: Ink
                font_size: 20.
                text_style_normal: theme.font_bold{font_family: DisplayFont font_size: 20. line_spacing: 0.7}
                text_style_bold: theme.font_bold{font_family: DisplayFont font_size: 20. line_spacing: 0.7}
                draw_text +: {color: Ink text_style: theme.font_bold{font_family: DisplayFont font_size: 20. line_spacing: 0.7}}
                draw_selection +: { color: #xCFE3FFFF }
                draw_block +: { selection_color: #xCFE3FFFF }
            }
            word := Markdown{
                selectable: false
                width: Fit height: Fit
                padding: 0.
                paragraph_spacing: 0.
                pre_code_spacing: 0.
                body: "Free"
                font_color: Ink
                font_size: 14.
                text_style_normal: theme.font_bold{font_family: StrongFont font_size: 14.}
                text_style_bold: theme.font_bold{font_family: StrongFont font_size: 14.}
                draw_text +: {color: Ink text_style: theme.font_bold{font_family: StrongFont font_size: 14.}}
                draw_selection +: { color: #xCFE3FFFF }
                draw_block +: { selection_color: #xCFE3FFFF }
            }
            meta := Markdown{
                selectable: false
                width: Fit height: Fit
                padding: 0.
                paragraph_spacing: 0.
                pre_code_spacing: 0.
                body: ""
                font_color: Muted
                font_size: 13.
                text_style_normal: theme.font_regular{font_family: TextFont font_size: 13.}
                draw_text +: {color: Muted text_style: theme.font_regular{font_family: TextFont font_size: 13.}}
                draw_selection +: { color: #xCFE3FFFF }
                draw_block +: { selection_color: #xCFE3FFFF }
            }
        }
        hit := HitBtn{}
    }
    // Visit-colour legend chip (the app's floor key).
    let KeyChip = View{
        width: Fit height: Fit
        flow: Right
        spacing: 6
        align: Align{y: 0.5}
        dot := RoundedView{width: 12 height: 12 draw_bg +: {color: FreeFill border_color: FreeLine border_size: 1.5 border_radius: 6.}}
        word := Markdown{
            selectable: true
            width: Fit height: Fit
            padding: 0.
            paragraph_spacing: 0.
            pre_code_spacing: 0.
            body: "Free"
            font_color: Muted
            font_size: 12.5
            text_style_normal: theme.font_regular{font_family: TextFont font_size: 12.5}
            draw_text +: {color: Muted text_style: theme.font_regular{font_family: TextFont font_size: 12.5}}
            draw_selection +: { color: #xCFE3FFFF }
            draw_block +: { selection_color: #xCFE3FFFF }
        }
    }
    // Kitchen column card (static): KOT, table, items, age.
    let KdsCard = RoundedView{
        width: Fill height: Fit
        flow: Down
        spacing: 2
        padding: Inset{left: 12. right: 12. top: 10. bottom: 10.}
        draw_bg +: {color: Well border_color: WellLine border_size: 1. border_radius: 12.}
        head := Markdown{padding: 0. selectable: true height: Fit paragraph_spacing: 0. pre_code_spacing: 0. body: "KOT" font_color: Ink draw_text +: {color: Ink text_style: theme.font_bold{font_family: StrongFont font_size: 14.}} draw_selection +: { color: #xCFE3FFFF }
        draw_block +: { selection_color: #xCFE3FFFF }}
        line := Markdown{width: Fill padding: 0. selectable: true height: Fit paragraph_spacing: 0. pre_code_spacing: 0. body: "Items" font_color: Muted draw_text +: {color: Muted text_style: theme.font_regular{font_family: TextFont font_size: 13.}} draw_selection +: { color: #xCFE3FFFF }
        draw_block +: { selection_color: #xCFE3FFFF }}
    }
    let KdsColumn = RoundedView{
        width: Fill height: Fit
        flow: Down
        spacing: 8
        padding: Inset{left: 10. right: 10. top: 10. bottom: 12.}
        draw_bg +: {color: WellWarm border_radius: 14.}
        title := Markdown{padding: 0. selectable: true height: Fit paragraph_spacing: 0. pre_code_spacing: 0. body: "Column" font_color: Maroon draw_text +: {color: Maroon text_style: theme.font_bold{font_family: DisplayFont font_size: 18.}} draw_selection +: { color: #xCFE3FFFF }
        draw_block +: { selection_color: #xCFE3FFFF }}
    }
    // Row: label left, amount right (bill, drawer).
    let AmountRow = View{
        width: Fill height: Fit
        flow: Right
        spacing: 8
        what := Body{width: Fill body: "Line"}
        amt := Ticket{width: Fit body: "0"}
    }

    // World shelf card (pager). Text and photo come from world_shelf.rs at
    // runtime: photo, or an honest "photo coming" tile.
    let ShelfCard = WarmCard{
        width: Fill
        spacing: 5
        padding: Inset{left: 10. right: 10. top: 10. bottom: 11.}
        pic_frame := RoundedView{
            width: Fill height: 96
            flow: Overlay
            draw_bg +: {
                color: WellDeep
                border_color: WellLine
                border_size: 1.
                border_radius: 10.
            }
            photo := Image{
                width: Fill height: Fill
                fit: ImageFit.CropToFill
            }
            soon := RoundedView{
                visible: false
                width: Fill height: Fill
                align: Align{x: 0.5 y: 0.5}
                draw_bg +: {
                    color: WellWarm
                    border_color: WellLine
                    border_size: 0.
                    border_radius: 10.
                }
                Small{
                    width: Fit
                    body: "Photo coming"
                    font_size: 11.
                    text_style_normal: theme.font_regular{font_family: TextFont font_size: 11.}
                    draw_text +: {text_style: theme.font_regular{font_family: TextFont font_size: 11.}}
                }
            }
        }
        country := Strong{body: "Country"}
        // TextFlow paints text_style_normal — pin both, on WorldFont. In a View:
        // Markdown ignores set_visible, and some countries have no native line.
        native_wrap := View{
        width: Fill height: Fit
        native := Markdown{
            selectable: true
            height: Fit
            paragraph_spacing: 0.
            pre_code_spacing: 0.
            draw_selection +: { color: #xCFE3FFFF }
        draw_block +: { selection_color: #xCFE3FFFF }
            width: Fill
            padding: 0.
            font_color: Maroon
            font_size: 12.
            text_style_normal: theme.font_regular{font_family: WorldFont font_size: 12. line_spacing: 1.2}
            text_style_bold: theme.font_bold{font_family: WorldFont font_size: 12. line_spacing: 1.2}
            draw_text +: {
                color: Muted
                text_style: theme.font_regular{font_family: WorldFont font_size: 12. line_spacing: 1.2}
            }
            body: "Local name"
        }
        }
        dish := Body{
            body: "Famous dish"
            font_size: 14.
            text_style_normal: theme.font_regular{font_family: TextFont font_size: 14. line_spacing: 1.25}
            draw_text +: {
                color: Ink
                text_style: theme.font_regular{font_family: TextFont font_size: 14. line_spacing: 1.25}
            }
        }
    }
    // Keeps its Fill share of the row when its card hides (last page).
    let ShelfSlot = View{
        width: Fill height: Fit
        card := ShelfCard{}
    }

    let CtaRow = View{
        width: Fill height: Fit
        flow: Right
        spacing: 12
        download := PrimaryBtn{text: "Get it on Google Play"}
        whatsapp := SecondaryBtn{text: "WhatsApp for a demo"}
    }

    startup() do #(App::script_component(vm)){
        ui: Root{
            main_window := Window{
                window.title: "Khaynjo — restaurant POS that keeps working without internet"
                window.inner_size: vec2(1280, 800)
                pass.clear_color: Cream
                body +: {
                    width: Fill
                    height: Fill
                    flow: Down

                    // ---- Sticky top bar: brand · section links · CTA -------------
                    // Labels (not Markdown): Fit width and a real `visible`.
                    top_bar := RoundedView{
                        width: Fill height: 60
                        flow: Right
                        spacing: 10
                        align: Align{y: 0.5}
                        padding: Inset{left: 24. right: 20. top: 0. bottom: 0.}
                        draw_bg +: {color: Red border_radius: 0.}
                        brand := View{
                            width: Fit height: Fit
                            flow: Overlay
                            View{
                                width: Fit height: Fit
                                flow: Right
                                spacing: 10
                                align: Align{y: 0.5}
                                Image{
                                    width: 36 height: 36
                                    fit: ImageFit.Stretch
                                    src: crate_resource("self:resources/media/app-icon.webp")
                                }
                                brand_name := Label{
                                    width: Fit
                                    text: "Khaynjo"
                                    draw_text +: {color: Cream text_style: theme.font_bold{font_family: DisplayFont font_size: 25.}}
                                }
                            }
                            brand_hit := HitBtn{}
                        }
                        View{width: Fill height: 1}
                        nav_links := View{
                            width: Fit height: Fit
                            flow: Right
                            spacing: 2
                            align: Align{y: 0.5}
                            go_how := NavLink{text: "How it works"}
                            go_features := NavLink{text: "Features"}
                            go_menus := NavLink{text: "Menus"}
                            go_offline := NavLink{text: "Offline"}
                            go_pricing := NavLink{text: "Pricing"}
                            go_faq := NavLink{text: "FAQ"}
                        }
                        bar_cta := BarCta{text: "Get it free"}
                    }

                    page := ScrollYView{
                        // Selection root: drag across blocks, Ctrl/Cmd+A, Copy, right-click
                        // menu; on a web page a drag may start in a margin too.
                        selection_root: true
                        select_from_gaps: true
                        width: Fill
                        height: Fill
                        flow: Down

                        // ---- 1. Hero (floor-states, table-cards, free-local) -----
                        hero := LampBand{
                            hero_wrap := Wrap{
                                padding: Inset{left: 64. right: 64. top: 56. bottom: 64.}
                                hero_row := View{
                                    width: Fill height: Fit
                                    flow: Right
                                    spacing: 44
                                    // Top-align: y:0.5 left a dead band under the
                                    // shorter copy column beside the tall floor card
                                    // and broke drag-select into the next section.
                                    align: Align{y: 0.0}
                                    hero_copy := View{
                                        width: Fill height: Fit
                                        flow: Down
                                        spacing: 20
                                        Pill{label.body: "Restaurant POS · free forever"}
                                        HeroTitle{body: "Lunch rush. No internet. No problem."}
                                        Lede{body: "Khaynjo runs your floor, kitchen and counter on devices you already own — from table to kitchen to bill, with or without the internet."}
                                        hero_cta := CtaRow{}
                                        cta_note := View{width: Fill height: Fit cta_note_text := Small{body: "Android on Google Play today · Windows and Linux packages coming."}}
                                        trust_row := View{
                                            width: Fill height: Fit
                                            flow: Down
                                            spacing: 10
                                            padding: Inset{top: 6.}
                                            Trust{label.body: "Works with no internet"}
                                            Trust{label.body: "Your books stay on your devices"}
                                            trust_langs := Trust{label.body: "115 languages on screen"}
                                        }
                                    }
                                    // The product's own mechanics as the hero visual.
                                    floor_card := WarmCard{
                                        width: 520
                                        spacing: 14
                                        padding: Inset{left: 20. right: 20. top: 18. bottom: 18.}
                                        View{
                                            width: Fill height: Fit
                                            flow: Right
                                            spacing: 10
                                            align: Align{y: 0.5}
                                            Strong{width: Fill body: "Your floor at a glance"}
                                            ExampleTag{}
                                        }
                                        floor := View{
                                            width: Fill height: Fit
                                            flow: Down
                                            spacing: 8
                                            floor_r0 := View{
                                                width: Fill height: Fit
                                                flow: Right
                                                spacing: 8
                                                t1 := TableTile{}
                                                t2 := TableTile{}
                                                t3 := TableTile{}
                                            }
                                            floor_r1 := View{
                                                width: Fill height: Fit
                                                flow: Right
                                                spacing: 8
                                                t4 := TableTile{}
                                                t5 := TableTile{}
                                                t6 := TableTile{}
                                            }
                                        }
                                        Small{body: "Tap a table to walk it through a visit. Same colours and words as the app — and a table frees only once it is paid."}
                                    }
                                }
                            }
                        }

                        // ---- 2. How a service runs ----------------------------------
                        how := PaperBand{
                            how_wrap := Wrap{
                                spacing: 56
                                SecHead{
                                    Eyebrow{body: "HOW A SERVICE RUNS"}
                                    H2{body: "One table, start to finish."}
                                    Lede{body: "Watch a table go from Seated to Paid — the same steps and colours your staff see in the app."}
                                    // One line when wide; three short lines on a phone.
                                    visit_key := View{
                                        width: Fit height: Fit
                                        flow: Right
                                        spacing: 14
                                        padding: Inset{top: 6.}
                                        key_r0 := View{width: Fit height: Fit flow: Right spacing: 14 KeyChip{} KeyChip{dot.draw_bg.color: SeatFill dot.draw_bg.border_color: SeatLine word.body: "Occupied"} KeyChip{dot.draw_bg.color: KitchenFill dot.draw_bg.border_color: KitchenLine word.body: "In kitchen"}}
                                        key_r1 := View{width: Fit height: Fit flow: Right spacing: 14 KeyChip{dot.draw_bg.color: ReadyFill dot.draw_bg.border_color: ReadyLine word.body: "Food ready"} KeyChip{dot.draw_bg.color: ServedFill dot.draw_bg.border_color: ServedLine word.body: "Served"}}
                                        key_r2 := View{width: Fit height: Fit flow: Right spacing: 14 KeyChip{dot.draw_bg.color: BillFill dot.draw_bg.border_color: BillLine word.body: "Bill printed"} KeyChip{dot.draw_bg.color: PaidFill dot.draw_bg.border_color: PaidLine word.body: "Paid"}}
                                    }
                                }

                                // Step 1 — seated, order taken (seated, variations)
                                step_order := View{
                                    width: Fill height: Fit
                                    flow: Right
                                    spacing: 48
                                    // Top-align: y:0.5 left a dead band above “1” that
                                    // broke drag-select from the section head into step 1.
                                    align: Align{y: 0.0}
                                    View{
                                        width: Fill height: Fit
                                        flow: Down
                                        spacing: 14
                                        StepDisc{num.body: "1"}
                                        H3{body: "Seat the table, take the order"}
                                        Body{body: "Mark the table Occupied the moment guests sit. Add dishes, pick a size, add extras, and type a note for the kitchen — right at the table."}
                                        View{
                                            width: Fill height: Fit
                                            flow: Down
                                            spacing: 10
                                            Point{head.body: "Sizes and add-ons" line.body: "Pick one size; add optional extras up to the limit you set."}
                                            Point{head.body: "A note per dish" line.body: "“Less spicy” travels with the dish it belongs to."}
                                        }
                                    }
                                    order_card := WarmCard{
                                        width: 380
                                        spacing: 6
                                        View{width: Fill height: Fit flow: Right align: Align{y: 0.5} Strong{width: Fill body: "T4 · Occupied"} ExampleTag{}}
                                        Rule{}
                                        Strong{body: "Paneer tikka — Full"}
                                        Body{body: "+ extra mint chutney"}
                                        Strong{body: "Masala chaas × 2"}
                                        Body{body: "Note: less salt"}
                                        Strong{body: "Butter naan × 4"}
                                    }
                                }

                                // Step 2 — the KOT (kot-station, kot-no-prices; kot-autoprint gated)
                                step_kot := View{
                                    width: Fill height: Fit
                                    flow: Right
                                    spacing: 48
                                    align: Align{y: 0.0}
                                    View{
                                        width: Fill height: Fit
                                        flow: Down
                                        spacing: 14
                                        StepDisc{num.body: "2"}
                                        H3{body: "One ticket per kitchen station"}
                                        Body{body: "Send the order and each station gets only its own dishes — numbered per day, with the table, the waiter and every note. Never a price: the kitchen cooks, the counter bills."}
                                        // Shown only after the owner's real-hardware LAN pilot passes.
                                        kot_lan := Point{visible: false head.body: "Prints itself" line.body: "Send, and the ticket prints at the right station — nobody carries it."}
                                    }
                                    ticket := RoundedView{
                                        width: 340 height: Fit
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
                                }

                                // Step 3 — kitchen in step (kds, all-day-counts, coursing)
                                step_kitchen := View{
                                    width: Fill height: Fit
                                    flow: Down
                                    spacing: 20
                                    kitchen_head := View{
                                        width: Fill height: Fit
                                        flow: Right
                                        spacing: 48
                                        View{
                                            width: Fill height: Fit
                                            flow: Down
                                            spacing: 14
                                            StepDisc{num.body: "3"}
                                            H3{body: "The kitchen sees what to cook now"}
                                            Body{body: "On the kitchen screen tickets move New → Preparing → Ready with one tap. Older tickets warm in colour, “To cook now” counts every dish waiting, and mains can hold until the starters go."}
                                        }
                                        View{width: 1 height: 1}
                                    }
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
                                    Small{body: "Example kitchen screen"}
                                }

                                // Step 4 — settle (split, settle, upi-qr, due-khata, free-on-pay)
                                step_settle := View{
                                    width: Fill height: Fit
                                    flow: Right
                                    spacing: 48
                                    align: Align{y: 0.0}
                                    View{
                                        width: Fill height: Fit
                                        flow: Down
                                        spacing: 14
                                        StepDisc{num.body: "4"}
                                        H3{body: "Settle any way the guest pays"}
                                        Body{body: "Cash with change to return, UPI with the amount already in the QR, part payment, or Due on the customer's khata. Split by item, by seat or evenly. The table frees only when its bill is paid."}
                                        Point{head.body: "Tips stay apart" line.body: "A tip is recorded on its own — never mixed into sales."}
                                    }
                                    bill := WarmCard{
                                        width: 380
                                        spacing: 8
                                        View{width: Fill height: Fit flow: Right align: Align{y: 0.5} Strong{width: Fill body: "T4 · Bill printed"} ExampleTag{}}
                                        Rule{}
                                        AmountRow{what.body: "Bill" amt.body: "₹ 1,240"}
                                        AmountRow{what.body: "Customer paid (cash)" amt.body: "₹ 1,500"}
                                        Rule{}
                                        AmountRow{what.body: "Return to customer" amt.body: "₹ 260"}
                                    }
                                }

                                // Step 5 — close the day (shift-close, day-report)
                                step_close := View{
                                    width: Fill height: Fit
                                    flow: Right
                                    spacing: 48
                                    align: Align{y: 0.0}
                                    View{
                                        width: Fill height: Fit
                                        flow: Down
                                        spacing: 14
                                        StepDisc{num.body: "5"}
                                        H3{body: "Close the day you can trust"}
                                        Body{body: "Count the drawer against the float, cash taken, tips and cash paid out — short or over is named. Print the shift slip, then open the day report by staff, order type, hour and dish, with cancels, comps and tips."}
                                    }
                                    drawer := WarmCard{
                                        width: 380
                                        spacing: 6
                                        View{width: Fill height: Fit flow: Right align: Align{y: 0.5} Strong{width: Fill body: "Shift close"} ExampleTag{}}
                                        Rule{}
                                        AmountRow{what.body: "Opening float" amt.body: "2,000"}
                                        AmountRow{what.body: "+ cash taken" amt.body: "18,450"}
                                        AmountRow{what.body: "+ cash tips" amt.body: "300"}
                                        AmountRow{what.body: "− cash paid out" amt.body: "600"}
                                        Rule{}
                                        AmountRow{what.body: "Expected in the drawer" amt.body: "₹ 20,150"}
                                        AmountRow{what.body: "Counted" amt.body: "₹ 20,050"}
                                        AmountRow{what.body: "Short" what.draw_text.color: Red amt.body: "₹ 100" amt.draw_text.color: Red}
                                    }
                                }
                            }
                        }

                        // ---- 3. Features grid ------------------------------------------
                        features := CreamBand{
                            features_wrap := Wrap{
                                SecHead{
                                    Eyebrow{body: "EVERYTHING ELSE THE FLOOR NEEDS"}
                                    H2{body: "Built for a real service."}
                                    Lede{body: "The changes that happen every lunch — handled in the open, with who, when and why kept."}
                                }
                                feat_grid := View{
                                    width: Fill height: Fit
                                    flow: Down
                                    spacing: 16
                                    // move-merge, kot-cancel
                                    feat_r0 := View{
                                        width: Fill height: Fit
                                        flow: Right
                                        spacing: 16
                                        FeatureCard{icon.src: crate_resource("self:resources/icons/floor.webp") title.body: "Move and merge tables" line.body: "Guests change tables — the order and its minutes go with them. Two tables, one bill."}
                                        FeatureCard{icon.src: crate_resource("self:resources/icons/kitchen.webp") title.body: "Cancel after the KOT" line.body: "With a reason, cooked or not, and who approved it. The kitchen gets a cancel slip."}
                                    }
                                    // nc, rights
                                    feat_r1 := View{
                                        width: Fill height: Fit
                                        flow: Right
                                        spacing: 16
                                        FeatureCard{icon.src: crate_resource("self:resources/icons/sales.webp") title.body: "Complimentary dishes" line.body: "A comp stays on the bill at ₹ 0, with the reason and the approver."}
                                        FeatureCard{icon.src: crate_resource("self:resources/icons/team.webp") title.body: "Rights per person" line.body: "Each action is allowed, allowed with a manager PIN, or not allowed — person by person."}
                                    }
                                    // aliases-short-codes, area-prices, menu-timings
                                    feat_r2 := View{
                                        width: Fill height: Fit
                                        flow: Right
                                        spacing: 16
                                        FeatureCard{icon.src: crate_resource("self:resources/icons/menu.webp") title.body: "Your menu, your words" line.body: "Search by name or short code, with aliases in another language — “ptk” finds Paneer tikka."}
                                        FeatureCard{icon.src: crate_resource("self:resources/icons/rates.webp") title.body: "Area prices and happy hour" line.body: "A dearer price in the AC room; breakfast menus and happy-hour prices by the clock."}
                                    }
                                    // due-khata, day-report
                                    feat_r3 := View{
                                        width: Fill height: Fit
                                        flow: Right
                                        spacing: 16
                                        FeatureCard{icon.src: crate_resource("self:resources/icons/credit.webp") title.body: "Customer khata" line.body: "Part payments and Due go on the customer's account in Parties, settled later."}
                                        FeatureCard{icon.src: crate_resource("self:resources/icons/reports.webp") title.body: "Reports that answer questions" line.body: "Sales by staff, order type, hour and dish — with cancels, comps and tips listed."}
                                    }
                                }
                            }
                        }

                        // ---- 4. Menus: shop templates + world shelf -----------------
                        menus := PaperBand{
                            menus_wrap := Wrap{
                                SecHead{
                                    Eyebrow{body: "START WITH A MENU"}
                                    H2{body: "Your menu, ready in minutes."}
                                    Lede{body: "Set up your counter and pick a starter menu with photos — then change names, prices and codes to your own. Free to download from khaynjo.com."}
                                }
                                // Catalogue: static/v1/catalogue/public/index.json (pinned by tests).
                                tmpl_row := View{
                                    width: Fill height: Fit
                                    flow: Right
                                    spacing: 16
                                    tmpl_starter := WarmCard{
                                        spacing: 8
                                        Pill{label.body: "Starter menu"}
                                        H3{body: "Khaynjo starter menu"}
                                        Body{body: "World cuisines — Japanese, Mexican, Italian, Indian, Thai and more — each dish with its photo."}
                                        Small{body: "14 dishes · 11 categories"}
                                    }
                                    tmpl_cafe := WarmCard{
                                        spacing: 8
                                        Pill{label.body: "Starter menu"}
                                        H3{body: "Café starter"}
                                        Body{body: "Tea, coffee, cold drinks, bakery and snacks — ready for a counter café."}
                                        Small{body: "22 dishes · 4 categories"}
                                    }
                                    tmpl_items := WarmCard{
                                        spacing: 8
                                        Pill{label.body: "Photo shelf"}
                                        H3{body: "Khaynjo dish photos"}
                                        Body{body: "The full public photo shelf — pick the picture for any dish you add."}
                                        Small{body: "1,459 dish photos · 25 kinds of kitchen"}
                                    }
                                }
                                Small{body: "Have a menu made for your restaurant? Your Khaynjo contact gives you a private 12-digit code to load it on setup."}

                                Rule{}
                                world_head := SecHead{
                                    padding: Inset{top: 12.}
                                    H3{body: "Every country at your counter"}
                                    Body{body: "One famous dish from every country — the English name, the name in their own script, and the dish in English. Put those words on your menu."}
                                }
                                // World shelf pager (world_shelf.rs, 199 countries): 8 cards
                                // a page in a 4 × 2 grid when wide, 4 (2 × 2) when narrow.
                                shelf_bar := View{
                                    width: Fill height: Fit
                                    flow: Right
                                    spacing: 10
                                    align: Align{y: 0.5}
                                    shelf_nav := View{
                                        width: Fit height: Fit
                                        flow: Right
                                        spacing: 8
                                        align: Align{y: 0.5}
                                        shelf_prev := PagerBtn{text: "‹ Prev"}
                                        shelf_page := Markdown{
                                            selectable: true
                                            width: Fit height: Fit
                                            padding: 0.
                                            paragraph_spacing: 0.
                                            pre_code_spacing: 0.
                                            body: "1 / 1"
                                            font_color: Muted
                                            font_size: 14.
                                            text_style_normal: theme.font_bold{font_family: StrongFont font_size: 14.}
                                            text_style_bold: theme.font_bold{font_family: StrongFont font_size: 14.}
                                            draw_text +: {color: Muted text_style: theme.font_bold{font_family: StrongFont font_size: 14.}}
                                            draw_selection +: { color: #xCFE3FFFF }
                                            draw_block +: { selection_color: #xCFE3FFFF }
                                        }
                                        shelf_next := PagerBtn{text: "Next ›"}
                                    }
                                }
                                shelf_grid := View{
                                    width: Fill height: Fit
                                    flow: Down
                                    spacing: 12
                                    shelf_q0 := View{
                                        width: Fill height: Fit
                                        flow: Right
                                        spacing: 12
                                        View{width: Fill height: Fit flow: Right spacing: 12 s0 := ShelfSlot{} s1 := ShelfSlot{}}
                                        View{width: Fill height: Fit flow: Right spacing: 12 s2 := ShelfSlot{} s3 := ShelfSlot{}}
                                    }
                                    shelf_q1 := View{
                                        width: Fill height: Fit
                                        flow: Right
                                        spacing: 12
                                        View{width: Fill height: Fit flow: Right spacing: 12 s4 := ShelfSlot{} s5 := ShelfSlot{}}
                                        View{width: Fill height: Fit flow: Right spacing: 12 s6 := ShelfSlot{} s7 := ShelfSlot{}}
                                    }
                                }
                                menu_note := View{width: Fill height: Fit menu_note_text := Small{body: "One famous dish per country."}}
                            }
                        }

                        // ---- 5. Offline (offline-lan; the multi-device half is gated) ---
                        offline := RedBand{
                            offline_wrap := Wrap{
                                offline_row := View{
                                    width: Fill height: Fit
                                    flow: Right
                                    spacing: 48
                                    // Top-align with the status card (same dead-band
                                    // issue as hero / how-service step rows).
                                    align: Align{y: 0.0}
                                    View{
                                        width: Fill height: Fit
                                        flow: Down
                                        spacing: 16
                                        OnRedEyebrow{body: "INTERNET DOWN?"}
                                        OnRedTitle{body: "The internet went down. The counter didn't."}
                                        OnRedBody{body: "Orders, kitchen tickets, bills and the day book live on your own devices. When the internet drops in the middle of lunch, nothing in your restaurant waits for it — and there is no account with us to sign in to."}
                                    }
                                    status_card := WarmCard{
                                        width: 400
                                        spacing: 14
                                        Strong{body: "Right now, mid-lunch"}
                                        View{
                                            width: Fill height: Fit
                                            flow: Right
                                            spacing: 10
                                            align: Align{y: 0.5}
                                            RoundedView{width: 12 height: 12 draw_bg +: {color: Red border_radius: 6.}}
                                            Body{width: Fill body: "Router: no internet"}
                                        }
                                        View{
                                            width: Fill height: Fit
                                            flow: Right
                                            spacing: 10
                                            align: Align{y: 0.5}
                                            RoundedView{width: 12 height: 12 draw_bg +: {color: BillLine border_radius: 6.}}
                                            Body{width: Fill body: "Counter: taking orders and settling bills"}
                                        }
                                        View{
                                            width: Fill height: Fit
                                            flow: Right
                                            spacing: 10
                                            align: Align{y: 0.5}
                                            RoundedView{width: 12 height: 12 draw_bg +: {color: BillLine border_radius: 6.}}
                                            Body{width: Fill body: "Day book: every bill saved on this device"}
                                        }
                                        // Shown only after the owner's real-hardware LAN pilot passes.
                                        offline_lan := View{
                                            visible: false
                                            width: Fill height: Fit
                                            flow: Right
                                            spacing: 10
                                            align: Align{y: 0.5}
                                            RoundedView{width: 12 height: 12 draw_bg +: {color: BillLine border_radius: 6.}}
                                            Body{width: Fill body: "Captains' tablets and the kitchen screen: working together on your own Wi‑Fi"}
                                        }
                                        ExampleTag{}
                                    }
                                }
                            }
                        }

                        // ---- 6. Pricing (free-local; vision §21) ------------------------
                        pricing := CreamBand{
                            pricing_wrap := Wrap{
                                SecHead{
                                    Eyebrow{body: "PRICING"}
                                    H2{body: "Free forever on your devices."}
                                    Lede{body: "Software capability is not what we sell. Run the whole of Khaynjo on your own devices at no cost — nothing is locked behind a plan."}
                                }
                                price_row := View{
                                    width: Fill height: Fit
                                    flow: Right
                                    spacing: 16
                                    WarmCard{
                                        spacing: 12
                                        draw_bg +: {border_color: Red border_size: 2.}
                                        Pill{label.body: "Khaynjo on your devices"}
                                        BigPrice{body: "₹0"}
                                        Small{body: "forever"}
                                        Rule{}
                                        Point{head.body: "Every feature" line.body: "Floor, kitchen, billing, day close, reports — the full app."}
                                        Point{head.body: "No per-seat charge" line.body: "No charge per table, waiter, order or device."}
                                        Point{head.body: "No account with us" line.body: "Day-to-day service needs no sign-up and no internet."}
                                    }
                                    WarmCard{
                                        spacing: 12
                                        draw_bg +: {color: #xFBF4EE}
                                        Pill{label.body: "Kumkaj Cloud · planned"}
                                        H3{body: "Pay only for the cloud you use"}
                                        Body{body: "When Kumkaj Cloud arrives it is optional: you pay for the cloud resources you actually consume. Same rates for everyone — a single café and a chain see the same price sheet."}
                                        Small{body: "Your local Khaynjo keeps working with or without it."}
                                    }
                                }
                            }
                        }

                        // ---- 7. FAQ ---------------------------------------------------
                        faq := PaperBand{
                            faq_wrap := Wrap{
                                SecHead{
                                    Eyebrow{body: "QUESTIONS"}
                                    H2{body: "Asked before every first service."}
                                }
                                faq_grid := View{
                                    width: Fill height: Fit
                                    flow: Down
                                    spacing: 16
                                    faq_r0 := View{
                                        width: Fill height: Fit
                                        flow: Right
                                        spacing: 16
                                        FaqItem{q.body: "Do I need the internet?" a.body: "No. Orders, KOTs, bills and the day book work with no internet. You need it once to download the app and, if you want one, a starter menu."}
                                        FaqItem{q.body: "Where are my bills kept?" a.body: "On your own device, in your shop — not on our servers. Make backup copies from Settings and keep them wherever you like."}
                                    }
                                    faq_r1 := View{
                                        width: Fill height: Fit
                                        flow: Right
                                        spacing: 16
                                        FaqItem{q.body: "Which devices does it run on?" a.body: "Android phones and tablets today, from Google Play. Windows and Linux packages are on their way."}
                                        FaqItem{q.body: "Does it print KOTs and bills?" a.body: "Yes — on thermal printers you set up in Settings → Printers, one kitchen ticket per station."}
                                    }
                                    faq_r2 := View{
                                        width: Fill height: Fit
                                        flow: Right
                                        spacing: 16
                                        faq_langs := FaqItem{q.body: "Can my staff use their own language?" a.body: "Yes. The screens come in 115 languages, Arabic and Urdu right-to-left, and dish names can carry aliases in another script."}
                                        FaqItem{q.body: "What does it cost?" a.body: "Nothing, on your own devices — every feature, forever. Only the optional Kumkaj Cloud, when it arrives, is paid by usage."}
                                    }
                                }
                            }
                        }

                        // ---- 8. Closing CTA + footer -------------------------------------
                        closing := RedBand{
                            closing_wrap := Wrap{
                                spacing: 20
                                padding: Inset{left: 64. right: 64. top: 64. bottom: 64.}
                                OnRedEyebrow{body: "READY FOR LUNCH?"}
                                OnRedTitle{body: "Run your next service on Khaynjo."}
                                OnRedBody{body: "Free on Google Play. Want a walk-through first? Message us on WhatsApp."}
                                close_cta := View{
                                    width: Fill height: Fit
                                    flow: Right
                                    spacing: 12
                                    download := GoldBtn{text: "Get it on Google Play"}
                                    whatsapp := GhostOnRedBtn{text: "WhatsApp for a demo"}
                                }
                                close_note := View{width: Fill height: Fit close_note_text := OnRedBody{body: " "}}
                            }
                        }
                        footer := MaroonBand{
                            footer_wrap := Wrap{
                                spacing: 18
                                padding: Inset{left: 64. right: 64. top: 36. bottom: 40.}
                                footer_row := View{
                                    width: Fill height: Fit
                                    flow: Right
                                    spacing: 24
                                    align: Align{y: 0.5}
                                    View{
                                        width: Fill height: Fit
                                        flow: Right
                                        spacing: 12
                                        align: Align{y: 0.5}
                                        Image{
                                            width: 40 height: 40
                                            fit: ImageFit.Stretch
                                            src: crate_resource("self:resources/media/app-icon.webp")
                                        }
                                        View{
                                            width: Fill height: Fit
                                            flow: Down
                                            spacing: 2
                                            // Footer brand mark (cream on red): chrome, not copy.
                                            Markdown{
                                                selectable: false
                                                width: Fit height: Fit
                                                padding: 0.
                                                paragraph_spacing: 0.
                                                pre_code_spacing: 0.
                                                body: "Khaynjo"
                                                font_color: Cream
                                                font_size: 22.
                                                text_style_normal: theme.font_bold{font_family: DisplayFont font_size: 22.}
                                                text_style_bold: theme.font_bold{font_family: DisplayFont font_size: 22.}
                                                draw_text +: {color: Cream text_style: theme.font_bold{font_family: DisplayFont font_size: 22.}}
                                                draw_selection +: { color: #xCFE3FFFF }
                                                draw_block +: { selection_color: #xCFE3FFFF }
                                            }
                                            OnRedBody{body: "Khaynjo is a Kumkaj product by Gotigin."}
                                        }
                                    }
                                    footer_links := View{
                                        width: Fit height: Fit
                                        flow: Right
                                        spacing: 4
                                        align: Align{y: 0.5}
                                        to_privacy := NavLink{text: "Privacy"}
                                        to_terms := NavLink{text: "Terms"}
                                        to_play := NavLink{text: "Google Play"}
                                        to_kumkaj := NavLink{text: "kumkaj.com"}
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
