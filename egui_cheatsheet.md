# egui 0.29 Cheatsheet

## Project Setup (Cargo.toml)
```toml
[dependencies]
eframe = "0.29"
egui = "0.29"
egui_extras = { version = "0.29", features = ["table"] }
egui_plot = "0.29"
rusqlite = { version = "0.34", features = ["bundled"] }
```

## Entry Point (main.rs)
```rust
mod app;
mod config;
mod db;
mod export;
mod redactor;
mod ui;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 900.0]),
        ..Default::default()
    };
    eframe::run_native("App Title", options,
        Box::new(|_cc| Ok(Box::new(app::App::new()))))
}
```

## App Trait Implementation
```rust
impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_visuals(nord_light());
        // Panels render first, data load after
        self.ui_top_panel(ctx);
        self.ui_sidebar(ctx);
        self.ui_central_panel(ctx);
        self.ui_status_bar(ctx);

        if self.needs_refresh {
            self.needs_refresh = false;
            self.needs_load = true;
            ctx.request_repaint();
        }
        if self.needs_load {
            self.needs_load = false;
            self.refresh_data();
        }
    }
}
```

## Panels

### SidePanel (left)
```rust
egui::SidePanel::left("sidebar")
    .resizable(true)
    .default_width(220.0)
    .show_separator_line(true)
    .frame(egui::Frame {
        inner_margin: egui::Margin::same(8.0),
        outer_margin: egui::Margin::default(),
        rounding: egui::Rounding::ZERO,
        shadow: egui::Shadow::NONE,
        fill: C_BG,
        stroke: egui::Stroke::NONE,
    })
    .show(ctx, |ui| { /* widgets */ });
```

### TopBottomPanel (top bar)
```rust
egui::TopBottomPanel::top("top_panel")
    .frame(egui::Frame { fill: C_SURF, .. })
    .show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.heading("Title");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Button").clicked() { }
            });
        });
    });
```

### TopBottomPanel (status bar)
```rust
egui::TopBottomPanel::bottom("status_bar")
    .frame(egui::Frame { fill: C_SURF, .. })
    .show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("text").color(C_GREY));
            ui.separator();
            ui.label("more text");
        });
    });
```

### CentralPanel
```rust
egui::CentralPanel::default()
    .frame(egui::Frame {
        inner_margin: egui::Margin::symmetric(10.0, 8.0),
        fill: C_BG, ..
    })
    .show(ctx, |ui| { /* main content */ });
```

## Window (modal/dialog)
```rust
let mut open = true;
egui::Window::new("Window Title")
    .open(&mut open)
    .default_size([600.0, 400.0])
    .show(ctx, |ui| { /* content */ });
```

## Common Widgets

### Label
```rust
ui.label("Plain text");
ui.label(egui::RichText::new("Styled").color(C_BLUE).size(14.0).strong());
```

### Button
```rust
if ui.button("Click").clicked() { action(); }
if ui.button("Export").on_hover_text("Tooltip").clicked() { }
```

### ComboBox
```rust
egui::ComboBox::from_id_salt("my_combo")
    .selected_text(current_value.replace('_', " "))
    .show_ui(ui, |ui| {
        ui.selectable_value(&mut value, "opt1", "Option 1");
        ui.selectable_value(&mut value, "opt2", "Option 2");
    });
```

### TextEdit (single/multi line)
```rust
ui.text_edit_singleline(&mut string);
ui.text_edit_multiline(&mut string);

// Read-only display
ui.add(egui::TextEdit::multiline(&mut output).interactive(false));
```

### Checkbox
```rust
ui.checkbox(&mut my_bool, "Label");
```

### Slider
```rust
ui.add(egui::Slider::new(&mut f32_val, 0.0..=100.0).text("label"));
```

### CollapsingHeader
```rust
egui::CollapsingHeader::new("Header")
    .default_open(true)
    .show(ui, |ui| { /* hidden content */ });
```

### Separator
```rust
ui.separator();
```

### Spinner
```rust
ui.add(egui::Spinner::new().size(32.0));
```

### ScrollArea
```rust
egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| { });
```

## Table (egui_extras)
```rust
use egui_extras::{Column, TableBuilder};

let mut table = TableBuilder::new(ui)
    .striped(true)
    .resizable(true)
    .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
    .min_scrolled_height(200.0);
for _ in &column_names {
    table = table.column(Column::initial(100.0));
}
table
    .header(22.0, |mut header| {
        for col in &column_names { header.col(|ui| { ui.strong(col); }); }
    })
    .body(|mut body| {
        for row in &rows {
            body.row(18.0, |mut row| {
                for col in &column_names {
                    row.col(|ui| { ui.label(&val); });
                }
            });
        }
    });
```

## Layout Helpers
```rust
ui.horizontal(|ui| { /* widgets side by side */ });
ui.vertical(|ui| { /* stacked widgets */ });
ui.horizontal_wrapped(|ui| { /* wraps to next line */ });
ui.vertical_centered(|ui| { /* centered vertically */ });
ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| { });

// Custom spacing
ui.add_space(8.0);
ui.spacing_mut().item_spacing = egui::vec2(10.0, 2.0);
ui.set_min_width(200.0);
ui.set_min_height(100.0);
```

## Painting API
```rust
let painter = ui.painter();
let rect = egui::Rect::from_min_size(pos, size);

// Shapes
painter.rect_filled(rect, egui::Rounding::same(4.0), C_SURF);
painter.rect_stroke(rect, egui::Rounding::same(4.0),
    egui::Stroke::new(1.0, C_MUTED));
painter.circle_filled(center, radius, C_BLUE);
painter.circle_stroke(center, radius, egui::Stroke::new(1.0, C_MUTED));
painter.text(pos, egui::Align2::LEFT_TOP, "text",
    egui::FontId::proportional(12.0), C_TEXT);
painter.add(egui::Shape::convex_polygon(points, color,
    egui::Stroke::new(1.0, C_MUTED)));

// Allocate space for custom painting
let (rect, response) = ui.allocate_exact_size(size, egui::Sense::hover());
```

## RichText
```rust
egui::RichText::new("text")
    .size(14.0)
    .color(C_BLUE)
    .strong()
    .weak()
    .heading()
    .monospace()
    .background_color(C_SURF)
```

## Color Constants (Nord Light)
```rust
const C_BG:     Color32 = Color32::from_rgb(236, 239, 244); // bg
const C_SURF:   Color32 = Color32::from_rgb(229, 233, 240); // surface
const C_MUTED:  Color32 = Color32::from_rgb(216, 222, 233); // borders
const C_TEXT:   Color32 = Color32::from_rgb(46, 52, 64);    // text
const C_GREY:   Color32 = Color32::from_rgb(76, 86, 106);
const C_BLUE:   Color32 = Color32::from_rgb(136, 192, 208);
const C_DKBLUE: Color32 = Color32::from_rgb(129, 161, 193);
const C_SEL:    Color32 = Color32::from_rgb(94, 129, 172);
const C_RED:    Color32 = Color32::from_rgb(191, 97, 106);
const C_GREEN:  Color32 = Color32::from_rgb(163, 190, 140);
const C_ORANGE: Color32 = Color32::from_rgb(208, 135, 112);

// Color32 is const fn in Rust 1.00+
const C_MY_COLOR: Color32 = Color32::from_rgb(255, 0, 0);
// Color32::from_rgba_premultiplied(r, g, b, a)
// Color32::from_white_alpha(a) — white with alpha
// Color32::from_black_alpha(a) — black with alpha
```

## Theme (Nord Light Visuals)
```rust
fn nord_light() -> egui::Visuals {
    use egui::style::*;
    let r4 = egui::Rounding { nw: 4.0, ne: 4.0, sw: 4.0, se: 4.0 };
    let wv = |fill| WidgetVisuals {
        bg_fill: fill, weak_bg_fill: fill,
        bg_stroke: Stroke::new(1.0, C_MUTED),
        fg_stroke: Stroke::new(1.0, C_TEXT),
        rounding: r4, expansion: 0.0,
    };
    egui::Visuals {
        dark_mode: false,
        override_text_color: Some(C_TEXT),
        widgets: Widgets {
            noninteractive: wv(C_SURF), inactive: wv(C_SURF),
            hovered: wv(C_BLUE), active: wv(C_DKBLUE), open: wv(C_SURF),
        },
        selection: Selection {
            bg_fill: C_SEL.linear_multiply(0.4),
            stroke: Stroke::new(1.0, C_SEL),
        },
        hyperlink_color: C_DKBLUE,
        faint_bg_color: C_BG,
        extreme_bg_color: C_MUTED,
        code_bg_color: C_SURF,
        warn_fg_color: C_ORANGE,
        error_fg_color: C_RED,
        window_rounding: r4,
        window_fill: C_BG,
        window_stroke: Stroke::new(1.0, C_MUTED),
        panel_fill: C_BG,
        ..Default::default()
    }
}
```

## Clipboard
```rust
// Copy text to clipboard
ui.output_mut(|o| o.copied_text = text.clone());

// Or via Context:
// ctx.copy_text(text);
```

## File Dialog (rfd)
```rust
if let Some(path) = rfd::FileDialog::new()
    .add_filter("SQLite", &["db", "sqlite", "sqlite3"])
    .pick_file()
{
    // use &path
}
```

## Metric Card (custom widget)
```rust
fn metric_card(ui: &mut egui::Ui, title: &str, value: u64, accent: Color32) {
    let height = 64.0;
    let (r, _) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), height), egui::Sense::hover());
    let bg_rect = r.shrink(2.0);
    ui.painter().rect_filled(bg_rect, egui::Rounding::same(6.0), C_SURF);
    ui.painter().rect_stroke(bg_rect, egui::Rounding::same(6.0),
        egui::Stroke::new(1.0, C_MUTED));
    // Accent left border
    let accent_rect = egui::Rect::from_min_size(
        egui::pos2(bg_rect.left(), bg_rect.top() + 4.0),
        egui::vec2(3.0, bg_rect.height() - 8.0));
    ui.painter().rect_filled(accent_rect, egui::Rounding::same(2.0), accent);
    // Title
    ui.painter().text(
        egui::pos2(bg_rect.left() + 12.0, bg_rect.top() + 16.0),
        egui::Align2::LEFT_TOP, title,
        egui::FontId::proportional(11.0), C_GREY);
    // Value
    ui.painter().text(
        egui::pos2(bg_rect.left() + 12.0, bg_rect.bottom() - 8.0),
        egui::Align2::LEFT_BOTTOM, &format!("{}", value),
        egui::FontId::proportional(22.0), accent);
}
```

## Frame Construction
```rust
egui::Frame {
    inner_margin: egui::Margin::same(8.0),
    outer_margin: egui::Margin::default(),
    rounding: egui::Rounding::ZERO,
    shadow: egui::Shadow::NONE,
    fill: C_BG,
    stroke: egui::Stroke::NONE,
}
// For grouping: egui::Frame::group(&ui.style()).show(ui, |ui| { });
```

## Bar Chart (egui_plot)
```rust
use egui_plot::{Bar, BarChart, Plot};

let bars: Vec<Bar> = data.iter().enumerate().map(|(i, (label, count))| {
    Bar::new(i as f64, *count as f64)
        .name(format!("{}: {}", label, count))
        .width(0.7)
        .fill(colors[i % colors.len()])
}).collect();
let chart = BarChart::new(bars).width(0.7);
Plot::new("plot_id")
    .height(200.0)
    .show(ui, |plot_ui| { plot_ui.bar_chart(chart); });
```

## Pie Chart (manual drawing)
```rust
let center = rect.center();
let radius = rect.height().min(rect.width()) / 2.0 - 10.0;
let painter = ui.painter();
let mut start_angle = -std::f32::consts::FRAC_PI_2;
let n = (30.0 * fraction).ceil().max(3.0) as usize;
let mut points = vec![center];
for j in 0..=n {
    let angle = start_angle + sweep * (j as f32 / n as f32);
    points.push(center + radius * egui::vec2(angle.cos(), angle.sin()));
}
points.push(center);
painter.add(egui::Shape::convex_polygon(points, color,
    egui::Stroke::new(1.0, C_MUTED)));
```

## Image
```rust
// From file
if let Some(texture) = &self.my_texture {
    ui.image(texture.id(), size);
}
// Load via:
// let texture = ctx.load_texture("name", color_image, Default::default());
```

## Multi-window / Viewport
```rust
// egui 0.29 supports multiple native windows
ctx.show_viewport_immediate(
    egui::ViewportId::from_hash_of("secondary"),
    egui::ViewportBuilder::default(),
    |ctx, class| { /* ui */ }
);
```

## Key Points
- `egui::FontId` does NOT have `.weight()` — use `egui::FontId::proportional(size)`
- `ui.output_mut(|o| o.copied_text = text)` sets clipboard (NOT `ctx.copy_to_clipboard`)
- `Rect::shrink(amount: f32)` exists on `Rect`, not on `Response`
- `allocate_exact_size` returns `(Rect, Response)` — destructure as `let (r, _) = …`
- `egui::Frame::new()` does NOT exist; use struct literal or `Frame::none()`/`Frame::group()`
