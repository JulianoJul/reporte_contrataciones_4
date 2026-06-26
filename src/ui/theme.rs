use eframe::egui::{Color32, Rounding, Stroke, Visuals};

// Nord Light palette
pub const C_BG: Color32 = Color32::from_rgb(236, 239, 244);
pub const C_SURF: Color32 = Color32::from_rgb(229, 233, 240);
pub const C_MUTED: Color32 = Color32::from_rgb(216, 222, 233);
pub const C_TEXT: Color32 = Color32::from_rgb(46, 52, 64);
pub const C_GREY: Color32 = Color32::from_rgb(76, 86, 106);
pub const C_BLUE: Color32 = Color32::from_rgb(136, 192, 208);
pub const C_DKBLUE: Color32 = Color32::from_rgb(129, 161, 193);
pub const C_SEL: Color32 = Color32::from_rgb(94, 129, 172);
pub const C_RED: Color32 = Color32::from_rgb(191, 97, 106);
pub const C_GREEN: Color32 = Color32::from_rgb(163, 190, 140);
pub const C_ORANGE: Color32 = Color32::from_rgb(208, 135, 112);

pub fn nord_colors() -> Vec<Color32> {
    vec![
        Color32::from_rgb(136, 192, 208),
        Color32::from_rgb(163, 190, 140),
        Color32::from_rgb(208, 135, 112),
        Color32::from_rgb(191, 97, 106),
        Color32::from_rgb(129, 161, 193),
        Color32::from_rgb(143, 188, 187),
        Color32::from_rgb(180, 142, 173),
        Color32::from_rgb(235, 203, 139),
    ]
}

pub fn nord_light() -> Visuals {
    use eframe::egui::style::{HandleShape, NumericColorSpace, Selection, TextCursorStyle, WidgetVisuals, Widgets};
    let r4 = Rounding { nw: 4.0, ne: 4.0, sw: 4.0, se: 4.0 };
    let wv = |fill| WidgetVisuals {
        bg_fill: fill, weak_bg_fill: fill,
        bg_stroke: Stroke::new(1.0, C_MUTED),
        fg_stroke: Stroke::new(1.0, C_TEXT),
        rounding: r4, expansion: 0.0,
    };
    Visuals {
        dark_mode: false,
        override_text_color: Some(C_TEXT),
        widgets: Widgets {
            noninteractive: wv(C_SURF), inactive: wv(C_SURF),
            hovered: wv(C_BLUE), active: wv(C_DKBLUE), open: wv(C_SURF),
        },
        selection: Selection { bg_fill: C_SEL.linear_multiply(0.4), stroke: Stroke::new(1.0, C_SEL) },
        hyperlink_color: C_DKBLUE,
        faint_bg_color: C_BG, extreme_bg_color: C_MUTED, code_bg_color: C_SURF,
        warn_fg_color: C_ORANGE, error_fg_color: C_RED,
        window_rounding: r4,
        window_shadow: eframe::egui::Shadow::NONE, window_fill: C_BG,
        window_stroke: Stroke::new(1.0, C_MUTED), window_highlight_topmost: false,
        menu_rounding: r4, panel_fill: C_BG,
        popup_shadow: eframe::egui::Shadow::NONE, resize_corner_size: 4.0,
        text_cursor: TextCursorStyle::default(), clip_rect_margin: 0.0,
        button_frame: true, collapsing_header_frame: false, indent_has_left_vline: false,
        striped: false, slider_trailing_fill: false, handle_shape: HandleShape::Circle,
        interact_cursor: None, image_loading_spinners: false,
        numeric_color_space: NumericColorSpace::GammaByte,
    }
}
