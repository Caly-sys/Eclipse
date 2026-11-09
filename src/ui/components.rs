use crate::config::VisualizerStyle;
use crate::ui::icons::{paint_icon, Icon};
use crate::ui::theme::Theme;
use egui::{
    pos2, vec2, Color32, Painter, Pos2, Rect, Response, Rounding, Sense, Stroke, Ui,
};

pub fn glass_button(ui: &mut Ui, label: &str, is_active: bool, theme: &Theme) -> Response {
    let text_color = if is_active {
        theme.text_primary
    } else {
        theme.text_secondary
    };

    let fill_color = if is_active {
        Color32::from_rgba_premultiplied(255, 255, 255, 18)
    } else {
        theme.card
    };

    let stroke = if is_active {
        Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(255, 255, 255, 25))
    } else {
        Stroke::new(1.0_f32, theme.border)
    };

    let button = egui::Button::new(
        egui::RichText::new(label)
            .color(text_color)
            .size(13.0)
            .strong(),
    )
    .fill(fill_color)
    .stroke(stroke)
    .rounding(Rounding::same(14.0_f32))
    .min_size(vec2(0.0, 30.0));

    ui.add(button)
}

pub fn accent_button(ui: &mut Ui, label: &str, theme: &Theme) -> Response {
    let button = egui::Button::new(
        egui::RichText::new(label)
            .color(Color32::from_rgb(10, 10, 12))
            .size(13.0)
            .strong(),
    )
    .fill(theme.accent)
    .rounding(Rounding::same(14.0_f32))
    .min_size(vec2(0.0, 30.0));

    ui.add(button)
}

/// Solid white #ffffff pill button with #0a0a0c text and icon.
pub fn white_pill_button(ui: &mut Ui, label: &str, icon: Option<Icon>) -> Response {
    let font_id = egui::FontId::proportional(12.5);
    let text_color = Color32::from_rgb(10, 10, 12);
    let galley = ui.painter().layout_no_wrap(label.to_string(), font_id, text_color);
    let icon_w = if icon.is_some() { 18.0 } else { 0.0 };
    let padding_x = 14.0;
    let height = 30.0;
    let width = galley.rect.width() + icon_w + (padding_x * 2.0);

    let (rect, resp) = ui.allocate_exact_size(vec2(width, height), Sense::click());
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }

    let bg_color = if resp.is_pointer_button_down_on() {
        Color32::from_rgb(220, 220, 220)
    } else if resp.hovered() {
        Color32::from_rgb(238, 238, 240)
    } else {
        Color32::WHITE
    };

    ui.painter().rect_filled(rect, Rounding::same(15.0), bg_color);

    let mut cur_x = rect.min.x + padding_x;
    let center_y = rect.center().y;

    if let Some(ic) = icon {
        let icon_rect = Rect::from_center_size(pos2(cur_x + 7.0, center_y), vec2(14.0, 14.0));
        paint_icon(ui.painter(), ic, icon_rect, text_color);
        cur_x += icon_w;
    }

    let text_pos = pos2(cur_x, center_y - galley.rect.height() / 2.0);
    ui.painter().galley(text_pos, galley, text_color);

    resp
}

/// Subtle dark secondary #222226 pill button with #e4e4e7 text and 1px rgba(255, 255, 255, 0.08) border.
pub fn dark_pill_button(ui: &mut Ui, label: &str, icon: Option<Icon>) -> Response {
    let font_id = egui::FontId::proportional(12.5);
    let text_color = Color32::from_rgb(228, 228, 231);
    let galley = ui.painter().layout_no_wrap(label.to_string(), font_id, text_color);
    let icon_w = if icon.is_some() { 18.0 } else { 0.0 };
    let padding_x = 14.0;
    let height = 30.0;
    let width = galley.rect.width() + icon_w + (padding_x * 2.0);

    let (rect, resp) = ui.allocate_exact_size(vec2(width, height), Sense::click());
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }

    let bg_color = if resp.is_pointer_button_down_on() {
        Color32::from_rgb(26, 26, 30)
    } else if resp.hovered() {
        Color32::from_rgb(42, 42, 47)
    } else {
        Color32::from_rgb(34, 34, 38)
    };

    let border_color = Color32::from_rgba_premultiplied(255, 255, 255, 20);

    ui.painter().rect_filled(rect, Rounding::same(15.0), bg_color);
    ui.painter().rect_stroke(rect, Rounding::same(15.0), Stroke::new(1.0_f32, border_color));

    let mut cur_x = rect.min.x + padding_x;
    let center_y = rect.center().y;

    if let Some(ic) = icon {
        let icon_rect = Rect::from_center_size(pos2(cur_x + 7.0, center_y), vec2(14.0, 14.0));
        paint_icon(ui.painter(), ic, icon_rect, text_color);
        cur_x += icon_w;
    }

    let text_pos = pos2(cur_x, center_y - galley.rect.height() / 2.0);
    ui.painter().galley(text_pos, galley, text_color);

    resp
}

pub fn icon_button(ui: &mut Ui, icon: &str, size: f32, theme: &Theme) -> Response {
    let button = egui::Button::new(
        egui::RichText::new(icon)
            .color(theme.text_primary)
            .size(size),
    )
    .fill(Color32::TRANSPARENT)
    .stroke(Stroke::NONE)
    .rounding(Rounding::same(6.0_f32));

    ui.add(button)
}

pub fn pill_badge(ui: &mut Ui, text: &str, fill: Color32, text_color: Color32) {
    ui.scope(|ui| {
        let padding = vec2(7.5, 2.5);
        let galley = ui.painter().layout_no_wrap(
            text.to_string(),
            egui::FontId::proportional(11.0),
            text_color,
        );
        let desired_size = galley.size() + padding * 2.0;
        let (rect, _) = ui.allocate_exact_size(desired_size, Sense::hover());

        let stroke_color = Color32::from_rgba_unmultiplied(
            text_color.r(),
            text_color.g(),
            text_color.b(),
            120,
        );

        ui.painter().rect_filled(rect, Rounding::same(6.0_f32), fill);
        ui.painter().rect_stroke(rect, Rounding::same(6.0_f32), Stroke::new(1.0_f32, stroke_color));
        ui.painter().galley(rect.min + padding, galley, text_color);
    });
}

/// Tiny, elegant, dark bordered pill for audio formats.
/// Fill: #18181b (Color32::from_rgb(24, 24, 27))
/// Border: 1px solid rgba(255, 255, 255, 0.08)
/// Text: #a1a1aa for standard, #f4f4f5 for Hi-Res/lossless.
pub fn format_badge(ui: &mut Ui, format: &str, is_lossless: bool, _theme: &Theme) {
    let fmt_str = if format.trim().is_empty() {
        "AUDIO"
    } else {
        format.trim()
    }
    .to_uppercase();

    let text_color = if is_lossless {
        Color32::from_rgb(244, 244, 245)
    } else {
        Color32::from_rgb(161, 161, 170)
    };
    let fill_color = Color32::from_rgb(24, 24, 27);
    let stroke_color = Color32::from_rgba_premultiplied(255, 255, 255, 20);

    ui.scope(|ui| {
        let padding = vec2(6.0, 2.0);
        let galley = ui.painter().layout_no_wrap(
            fmt_str,
            egui::FontId::proportional(10.5),
            text_color,
        );
        let desired_size = galley.size() + padding * 2.0;
        let (rect, _) = ui.allocate_exact_size(desired_size, Sense::hover());

        ui.painter().rect_filled(rect, Rounding::same(4.0_f32), fill_color);
        ui.painter().rect_stroke(rect, Rounding::same(4.0_f32), Stroke::new(1.0_f32, stroke_color));
        ui.painter().galley(rect.min + padding, galley, text_color);
    });
}

pub fn render_mini_visualizer(
    ui: &mut Ui,
    bands: &[f32],
    accent: Color32,
    size: egui::Vec2,
) {
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    let painter = ui.painter();

    let count = bands.len().min(16);
    if count == 0 {
        return;
    }

    let bar_width = (rect.width() / count as f32) - 1.5;
    let max_height = rect.height();

    for (i, &band) in bands.iter().enumerate().take(count) {
        let val = band.clamp(0.05, 1.0);
        let h = val * max_height;
        let x = rect.min.x + (i as f32) * (bar_width + 1.5);
        let y = rect.max.y - h;

        let bar_rect = Rect::from_min_size(pos2(x, y), vec2(bar_width.max(1.0), h));
        painter.rect_filled(bar_rect, Rounding::same(1.5_f32), accent);
    }
}

#[allow(clippy::too_many_arguments)]
pub fn render_spectrum(
    painter: &Painter,
    rect: Rect,
    bands: &[f32],
    peaks: &[f32],
    wave: &[f32],
    style: VisualizerStyle,
    accent: Color32,
    theme: &Theme,
) {
    match style {
        VisualizerStyle::Bars => {
            let count = bands.len();
            if count == 0 {
                return;
            }
            let total_gap = 2.0 * (count - 1) as f32;
            let bar_width = ((rect.width() - total_gap) / count as f32).max(1.0);

            for (i, &band) in bands.iter().enumerate() {
                let val = band.clamp(0.02, 1.0);
                let h = val * rect.height();
                let x = rect.min.x + (i as f32) * (bar_width + 2.0);
                let y = rect.max.y - h;

                // Main bar in neon violet
                let bar_rect = Rect::from_min_size(pos2(x, y), vec2(bar_width, h));
                let alpha = (120.0 + val * 135.0) as u8;
                let color = Color32::from_rgba_premultiplied(accent.r(), accent.g(), accent.b(), alpha);
                painter.rect_filled(bar_rect, Rounding::same(2.0_f32), color);

                // Peak dot in warm amber accent
                if i < peaks.len() {
                    let peak_y = rect.max.y - (peaks[i].clamp(0.0, 1.0) * rect.height());
                    let peak_rect = Rect::from_min_size(pos2(x, peak_y - 2.0), vec2(bar_width, 2.0));
                    painter.rect_filled(peak_rect, Rounding::same(1.0_f32), theme.accent_warm);
                }
            }
        }
        VisualizerStyle::Mirror => {
            let count = bands.len();
            if count == 0 {
                return;
            }
            let center_y = rect.center().y;
            let half_h = rect.height() * 0.48;
            let bar_width = (rect.width() / count as f32) - 1.5;

            for (i, &band) in bands.iter().enumerate() {
                let val = band.clamp(0.02, 1.0);
                let h = val * half_h;
                let x = rect.min.x + (i as f32) * (bar_width + 1.5);

                let bar_rect = Rect::from_min_max(
                    pos2(x, center_y - h),
                    pos2(x + bar_width.max(1.0), center_y + h),
                );
                let alpha = (140.0 + val * 115.0) as u8;
                let color = Color32::from_rgba_premultiplied(accent.r(), accent.g(), accent.b(), alpha);
                painter.rect_filled(bar_rect, Rounding::same(2.0_f32), color);
            }
        }
        VisualizerStyle::Scope => {
            let wave_len = wave.len();
            if wave_len < 2 {
                return;
            }
            let center_y = rect.center().y;
            let amp = rect.height() * 0.4;
            let step_x = rect.width() / (wave_len - 1) as f32;

            let mut points = Vec::with_capacity(wave_len);
            for (i, &w) in wave.iter().enumerate() {
                let x = rect.min.x + i as f32 * step_x;
                let y = center_y - w * amp;
                points.push(pos2(x, y));
            }

            for w in points.windows(2) {
                painter.line_segment([w[0], w[1]], Stroke::new(2.0_f32, accent));
            }
        }
        VisualizerStyle::Radial => {
            let center = rect.center();
            let radius = rect.height().min(rect.width()) * 0.35;
            let count = bands.len();

            for (i, &band) in bands.iter().enumerate().take(count) {
                let angle = (i as f32 / count as f32) * std::f32::consts::TAU;
                let val = band.clamp(0.05, 1.0);
                let r_out = radius + val * radius * 0.7;

                let p1 = Pos2::new(
                    center.x + angle.cos() * radius,
                    center.y + angle.sin() * radius,
                );
                let p2 = Pos2::new(
                    center.x + angle.cos() * r_out,
                    center.y + angle.sin() * r_out,
                );

                painter.line_segment([p1, p2], Stroke::new(2.5_f32, accent));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_badge_rendering() {
        let ctx = egui::Context::default();
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let theme = Theme::from_name("Eclipse", None);
                // Verify rendering of FLAC (lossless) badge
                format_badge(ui, "FLAC", true, &theme);
                // Verify rendering of MP3 (lossy) badge
                format_badge(ui, "MP3", false, &theme);
                // Verify fallback for empty format string
                format_badge(ui, "", false, &theme);
            });
        });
    }

    #[test]
    fn test_pill_badge_rendering() {
        let ctx = egui::Context::default();
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                pill_badge(ui, "24-BIT 48 KHZ", Color32::BLACK, Color32::WHITE);
            });
        });
    }

    #[test]
    fn test_pill_buttons_rendering() {
        let ctx = egui::Context::default();
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let resp_white = white_pill_button(ui, "Play All", Some(Icon::Play));
                assert!(resp_white.rect.width() > 30.0);

                let resp_dark = dark_pill_button(ui, "Shuffle", Some(Icon::Shuffle));
                assert!(resp_dark.rect.width() > 30.0);
            });
        });
    }
}
