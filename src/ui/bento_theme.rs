use egui::{Color32, FontId, Frame, Margin, Response, Rounding, Sense, Stroke, Ui, Vec2};

// ============================================================================
// 1. Surface Ramp, Color Palette & Spacing Tokens
// ============================================================================

/// Outermost window canvas visible through 8.0px bento gaps (#09090D)
pub const BG_CANVAS: Color32 = Color32::from_rgb(9, 9, 13);

/// Bento surface fill (Left rail, Center stage, Right inspector) (#101016)
pub const BG_PANEL: Color32 = Color32::from_rgb(16, 16, 22);

/// Elevated sub-surface (search inputs, segmented tracks, active pills) (#171720)
pub const BG_CARD: Color32 = Color32::from_rgb(23, 23, 32);

/// Subtle interactive hover highlight for table rows and buttons (#20202C)
pub const BG_CARD_HOVER: Color32 = Color32::from_rgb(32, 32, 44);

/// Floating bottom player dock surface (#12121A)
pub const BG_DOCK: Color32 = Color32::from_rgb(18, 18, 26);

/// 1.0px hairline edge for bento frames and dividers
/// Value matches `Color32::from_white_alpha(16)` (rgba(255, 255, 255, 16))
pub const BORDER_SUBTLE: Color32 = Color32::from_rgba_premultiplied(71, 71, 71, 16);

/// Primary vibrant brand accent (play button, seek fill, active dot) (#9C6CFF)
pub const ACCENT_PURPLE: Color32 = Color32::from_rgb(156, 108, 255);

/// Soft pill fill for active navigation items and active tabs
/// Value matches `Color32::from_rgba_unmultiplied(156, 108, 255, 34)`
pub const ACCENT_PURPLE_SOFT: Color32 = Color32::from_rgba_premultiplied(59, 39, 102, 34);

/// High-contrast crisp text for track titles and active lyrics (#F2F2F8)
pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(242, 242, 248);

/// Muted secondary text for artists, album names, and section headers (#9898A8)
pub const TEXT_MUTED: Color32 = Color32::from_rgb(152, 152, 168);

/// Faint timestamps, inactive lyrics, and shortcut keys (#585868)
pub const TEXT_FAINT: Color32 = Color32::from_rgb(88, 88, 104);

// Hi-Res Badge Tokens (Dark Amber-Gold Glass)
/// Matches `Color32::from_rgba_unmultiplied(245, 194, 66, 36)`
pub const BADGE_HIRES_BG: Color32 = Color32::from_rgba_premultiplied(101, 78, 21, 36);
/// Matches `Color32::from_rgba_unmultiplied(245, 194, 66, 170)`
pub const BADGE_HIRES_BORDER: Color32 = Color32::from_rgba_premultiplied(205, 162, 54, 170);
/// Text: #FFE082 (rgb(255, 224, 130))
pub const BADGE_HIRES_TEXT: Color32 = Color32::from_rgb(255, 224, 130);

// Standard Lossless Badge Tokens (Purple Glass)
/// Matches `Color32::from_rgba_unmultiplied(156, 108, 255, 34)`
pub const BADGE_LOSSLESS_BG: Color32 = Color32::from_rgba_premultiplied(59, 39, 102, 34);
/// Matches `Color32::from_rgba_unmultiplied(156, 108, 255, 140)`
pub const BADGE_LOSSLESS_BORDER: Color32 = Color32::from_rgba_premultiplied(118, 81, 196, 140);
/// Text: #DCC8FF (rgb(220, 200, 255))
pub const BADGE_LOSSLESS_TEXT: Color32 = Color32::from_rgb(220, 200, 255);

// Spacing & Corner Radii
pub const BENTO_GAP: f32 = 8.0_f32;
pub const BENTO_RADIUS: f32 = 12.0_f32;
pub const CARD_RADIUS: f32 = 8.0_f32;
pub const ART_RADIUS: f32 = 10.0_f32;

// Layout Dimensions
pub const SIDEBAR_EXPANDED_WIDTH: f32 = 220.0_f32;
pub const SIDEBAR_COLLAPSED_WIDTH: f32 = 64.0_f32;
pub const RIGHT_INSPECTOR_WIDTH: f32 = 310.0_f32;
pub const RIGHT_INSPECTOR_MIN_WINDOW_WIDTH: f32 = 1080.0_f32;
pub const PLAYER_DOCK_HEIGHT: f32 = 76.0_f32;
pub const TOP_BAR_HEIGHT: f32 = 44.0_f32;

// ============================================================================
// 2. Reusable Bento Frame Constructors
// ============================================================================

/// An egui::Frame with fill: BG_PANEL, rounding: Rounding::same(BENTO_RADIUS),
/// stroke: Stroke::new(1.0, BORDER_SUBTLE), and inner_margin: Margin::same(10.0).
pub fn bento_panel_frame() -> Frame {
    Frame::none()
        .fill(BG_PANEL)
        .rounding(Rounding::same(BENTO_RADIUS))
        .stroke(Stroke::new(1.0_f32, BORDER_SUBTLE))
        .inner_margin(Margin::same(10.0_f32))
}

/// An egui::Frame with fill: BG_DOCK, rounding: Rounding::same(BENTO_RADIUS),
/// stroke: Stroke::new(1.0, BORDER_SUBTLE), and inner_margin: Margin::symmetric(14.0, 8.0).
pub fn bento_dock_frame() -> Frame {
    Frame::none()
        .fill(BG_DOCK)
        .rounding(Rounding::same(BENTO_RADIUS))
        .stroke(Stroke::new(1.0_f32, BORDER_SUBTLE))
        .inner_margin(Margin::symmetric(14.0_f32, 8.0_f32))
}

/// An egui::Frame with fill: BG_CARD, rounding: Rounding::same(CARD_RADIUS),
/// stroke: Stroke::new(1.0, BORDER_SUBTLE), and inner_margin: Margin::same(8.0).
pub fn bento_card_frame() -> Frame {
    Frame::none()
        .fill(BG_CARD)
        .rounding(Rounding::same(CARD_RADIUS))
        .stroke(Stroke::new(1.0_f32, BORDER_SUBTLE))
        .inner_margin(Margin::same(8.0_f32))
}

// ============================================================================
// 3. Reusable Widgets & Theming
// ============================================================================

/// Configures ctx.set_visuals(...) with matte-dark theme, eliminating default
/// bright white outlines and setting slim translucent scrollbars, dark panel fills,
/// and muted selection colors.
pub fn apply_bento_visuals(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();

    // Dark matte fills matching the surface ramp
    visuals.panel_fill = BG_CANVAS;
    visuals.window_fill = BG_PANEL;
    visuals.extreme_bg_color = BG_CANVAS;
    visuals.faint_bg_color = BG_CARD;
    visuals.code_bg_color = BG_CARD;
    visuals.window_stroke = Stroke::new(1.0_f32, BORDER_SUBTLE);
    visuals.window_rounding = Rounding::same(BENTO_RADIUS);
    visuals.menu_rounding = Rounding::same(CARD_RADIUS);

    // Eliminate default bright white outlines on widgets
    visuals.widgets.noninteractive.bg_fill = BG_PANEL;
    visuals.widgets.noninteractive.weak_bg_fill = BG_PANEL;
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, BORDER_SUBTLE);
    visuals.widgets.noninteractive.rounding = Rounding::same(CARD_RADIUS);
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, TEXT_PRIMARY);

    visuals.widgets.inactive.bg_fill = BG_CARD;
    visuals.widgets.inactive.weak_bg_fill = Color32::TRANSPARENT;
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, BORDER_SUBTLE);
    visuals.widgets.inactive.rounding = Rounding::same(CARD_RADIUS);
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, TEXT_MUTED);

    visuals.widgets.hovered.bg_fill = BG_CARD_HOVER;
    visuals.widgets.hovered.weak_bg_fill = BG_CARD_HOVER;
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, BORDER_SUBTLE);
    visuals.widgets.hovered.rounding = Rounding::same(CARD_RADIUS);
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0_f32, TEXT_PRIMARY);

    visuals.widgets.active.bg_fill = ACCENT_PURPLE_SOFT;
    visuals.widgets.active.weak_bg_fill = ACCENT_PURPLE_SOFT;
    visuals.widgets.active.bg_stroke = Stroke::new(1.0_f32, ACCENT_PURPLE);
    visuals.widgets.active.rounding = Rounding::same(CARD_RADIUS);
    visuals.widgets.active.fg_stroke = Stroke::new(1.0_f32, TEXT_PRIMARY);

    visuals.widgets.open.bg_fill = BG_CARD;
    visuals.widgets.open.rounding = Rounding::same(CARD_RADIUS);

    // Muted brand selection colors
    visuals.selection.bg_fill = ACCENT_PURPLE_SOFT;
    visuals.selection.stroke = Stroke::new(1.0_f32, ACCENT_PURPLE);

    ctx.set_visuals(visuals);

    // Slim translucent scrollbars
    ctx.style_mut(|style| {
        style.spacing.scroll.bar_width = 6.0_f32;
        style.spacing.scroll.bar_inner_margin = 2.0_f32;
        style.spacing.scroll.bar_outer_margin = 2.0_f32;
        style.spacing.scroll.handle_min_length = 24.0_f32;
        style.spacing.scroll.dormant_background_opacity = 0.0_f32;
        style.spacing.scroll.active_background_opacity = 0.05_f32;
        style.spacing.scroll.interact_background_opacity = 0.1_f32;
        style.spacing.scroll.dormant_handle_opacity = 0.2_f32;
        style.spacing.scroll.active_handle_opacity = 0.4_f32;
        style.spacing.scroll.interact_handle_opacity = 0.6_f32;
    });
}

/// Draws a clean quality badge pill:
/// - If `is_hires`: dark amber-gold glass (fill: rgba(245, 194, 66, 36), stroke: 1.0, rgba(245, 194, 66, 170), text rgb(255, 224, 130)).
/// - Else (standard lossless / format): purple glass (fill: rgba(156, 108, 255, 34), stroke: 1.0, rgba(156, 108, 255, 140), text rgb(220, 200, 255)).
/// - Text font: proportional 9.5px bold, rounding 5.0px.
pub fn quality_badge_pill(ui: &mut Ui, label: &str, is_hires: bool) -> Response {
    let (bg, border, text_col) = if is_hires {
        (
            BADGE_HIRES_BG,
            BADGE_HIRES_BORDER,
            BADGE_HIRES_TEXT,
        )
    } else {
        (
            BADGE_LOSSLESS_BG,
            BADGE_LOSSLESS_BORDER,
            BADGE_LOSSLESS_TEXT,
        )
    };

    let font_id = FontId::new(9.5_f32, egui::FontFamily::Proportional);
    let galley = ui.painter().layout_no_wrap(label.to_owned(), font_id, text_col);
    let text_size = galley.size();

    let h_padding = 6.0_f32;
    let v_padding = 2.0_f32;
    let pill_size = Vec2::new(
        text_size.x + h_padding * 2.0_f32,
        (text_size.y + v_padding * 2.0_f32).max(16.0_f32),
    );

    let (rect, response) = ui.allocate_exact_size(pill_size, Sense::hover());

    if ui.is_rect_visible(rect) {
        let rounding = Rounding::same(5.0_f32);
        ui.painter().rect_filled(rect, rounding, bg);
        ui.painter().rect_stroke(rect, rounding, Stroke::new(1.0_f32, border));

        let text_pos = egui::pos2(
            rect.min.x + (rect.width() - text_size.x) * 0.5_f32,
            rect.min.y + (rect.height() - text_size.y) * 0.5_f32,
        );
        ui.painter().galley(text_pos, galley, text_col);
    }

    response
}

/// Interactive pill tab switcher:
/// - If `is_active`: background `ACCENT_PURPLE_SOFT`, text `TEXT_PRIMARY` (bold, 12.0px), rounding `Rounding::same(6.0)`.
/// - Else: transparent background, text `TEXT_MUTED` (12.0px), with subtle `BG_CARD_HOVER` on hover.
pub fn segmented_tab_button(ui: &mut Ui, text: &str, is_active: bool) -> Response {
    let font_id = FontId::new(12.0_f32, egui::FontFamily::Proportional);

    let default_text_color = if is_active {
        TEXT_PRIMARY
    } else {
        TEXT_MUTED
    };

    let galley = ui.painter().layout_no_wrap(text.to_owned(), font_id.clone(), default_text_color);
    let text_size = galley.size();

    let h_padding = 12.0_f32;
    let v_padding = 5.0_f32;
    let desired_size = Vec2::new(
        text_size.x + h_padding * 2.0_f32,
        (text_size.y + v_padding * 2.0_f32).max(26.0_f32),
    );

    let (rect, response) = ui.allocate_exact_size(desired_size, Sense::click());

    if ui.is_rect_visible(rect) {
        let rounding = Rounding::same(6.0_f32);
        let hovered = response.hovered();

        let (bg_fill, text_color, border_stroke) = if is_active {
            (
                ACCENT_PURPLE_SOFT,
                TEXT_PRIMARY,
                Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(59, 39, 102, 90)),
            )
        } else if hovered {
            (
                BG_CARD_HOVER,
                TEXT_PRIMARY,
                Stroke::NONE,
            )
        } else {
            (
                Color32::TRANSPARENT,
                TEXT_MUTED,
                Stroke::NONE,
            )
        };

        if bg_fill != Color32::TRANSPARENT {
            ui.painter().rect_filled(rect, rounding, bg_fill);
        }
        if border_stroke != Stroke::NONE {
            ui.painter().rect_stroke(rect, rounding, border_stroke);
        }

        let render_galley = if text_color != default_text_color {
            ui.painter().layout_no_wrap(text.to_owned(), font_id, text_color)
        } else {
            galley
        };

        let text_pos = egui::pos2(
            rect.min.x + (rect.width() - text_size.x) * 0.5_f32,
            rect.min.y + (rect.height() - text_size.y) * 0.5_f32,
        );
        ui.painter().galley(text_pos, render_galley, text_color);
    }

    response
}

// ============================================================================
// 4. Comprehensive Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_tokens() {
        assert_eq!(BG_CANVAS, Color32::from_rgb(9, 9, 13));
        assert_eq!(BG_PANEL, Color32::from_rgb(16, 16, 22));
        assert_eq!(BG_CARD, Color32::from_rgb(23, 23, 32));
        assert_eq!(BG_CARD_HOVER, Color32::from_rgb(32, 32, 44));
        assert_eq!(BG_DOCK, Color32::from_rgb(18, 18, 26));
        assert_eq!(BORDER_SUBTLE, Color32::from_white_alpha(16));
        assert_eq!(ACCENT_PURPLE, Color32::from_rgb(156, 108, 255));
        assert_eq!(ACCENT_PURPLE_SOFT, Color32::from_rgba_unmultiplied(156, 108, 255, 34));
        assert_eq!(TEXT_PRIMARY, Color32::from_rgb(242, 242, 248));
        assert_eq!(TEXT_MUTED, Color32::from_rgb(152, 152, 168));
        assert_eq!(TEXT_FAINT, Color32::from_rgb(88, 88, 104));
    }

    #[test]
    fn test_quality_badge_tokens() {
        assert_eq!(BADGE_HIRES_BG, Color32::from_rgba_unmultiplied(245, 194, 66, 36));
        assert_eq!(BADGE_HIRES_BORDER, Color32::from_rgba_unmultiplied(245, 194, 66, 170));
        assert_eq!(BADGE_HIRES_TEXT, Color32::from_rgb(255, 224, 130));

        assert_eq!(BADGE_LOSSLESS_BG, Color32::from_rgba_unmultiplied(156, 108, 255, 34));
        assert_eq!(BADGE_LOSSLESS_BORDER, Color32::from_rgba_unmultiplied(156, 108, 255, 140));
        assert_eq!(BADGE_LOSSLESS_TEXT, Color32::from_rgb(220, 200, 255));
    }

    #[test]
    fn test_spacing_and_radius_tokens() {
        assert_eq!(BENTO_GAP, 8.0_f32);
        assert_eq!(BENTO_RADIUS, 12.0_f32);
        assert_eq!(CARD_RADIUS, 8.0_f32);
        assert_eq!(ART_RADIUS, 10.0_f32);
        assert_eq!(SIDEBAR_EXPANDED_WIDTH, 220.0_f32);
        assert_eq!(SIDEBAR_COLLAPSED_WIDTH, 64.0_f32);
        assert_eq!(RIGHT_INSPECTOR_WIDTH, 310.0_f32);
        assert_eq!(RIGHT_INSPECTOR_MIN_WINDOW_WIDTH, 1080.0_f32);
        assert_eq!(PLAYER_DOCK_HEIGHT, 76.0_f32);
        assert_eq!(TOP_BAR_HEIGHT, 44.0_f32);
    }

    #[test]
    fn test_bento_frames() {
        let panel = bento_panel_frame();
        assert_eq!(panel.fill, BG_PANEL);
        assert_eq!(panel.rounding, Rounding::same(BENTO_RADIUS));
        assert_eq!(panel.stroke, Stroke::new(1.0_f32, BORDER_SUBTLE));
        assert_eq!(panel.inner_margin, Margin::same(10.0_f32));

        let dock = bento_dock_frame();
        assert_eq!(dock.fill, BG_DOCK);
        assert_eq!(dock.rounding, Rounding::same(BENTO_RADIUS));
        assert_eq!(dock.stroke, Stroke::new(1.0_f32, BORDER_SUBTLE));
        assert_eq!(dock.inner_margin, Margin::symmetric(14.0_f32, 8.0_f32));

        let card = bento_card_frame();
        assert_eq!(card.fill, BG_CARD);
        assert_eq!(card.rounding, Rounding::same(CARD_RADIUS));
        assert_eq!(card.stroke, Stroke::new(1.0_f32, BORDER_SUBTLE));
        assert_eq!(card.inner_margin, Margin::same(8.0_f32));
    }

    #[test]
    fn test_apply_bento_visuals() {
        let ctx = egui::Context::default();
        apply_bento_visuals(&ctx);

        let visuals = ctx.style().visuals.clone();
        assert!(visuals.dark_mode);
        assert_eq!(visuals.panel_fill, BG_CANVAS);
        assert_eq!(visuals.window_fill, BG_PANEL);
        assert_eq!(visuals.window_stroke, Stroke::new(1.0_f32, BORDER_SUBTLE));
        assert_eq!(visuals.window_rounding, Rounding::same(BENTO_RADIUS));
        assert_eq!(visuals.menu_rounding, Rounding::same(CARD_RADIUS));
        assert_eq!(visuals.widgets.hovered.bg_fill, BG_CARD_HOVER);
        assert_eq!(visuals.selection.bg_fill, ACCENT_PURPLE_SOFT);

        let spacing = ctx.style().spacing.clone();
        assert_eq!(spacing.scroll.bar_width, 6.0_f32);
        assert_eq!(spacing.scroll.bar_inner_margin, 2.0_f32);
        assert_eq!(spacing.scroll.bar_outer_margin, 2.0_f32);
        assert_eq!(spacing.scroll.handle_min_length, 24.0_f32);
    }

    #[test]
    fn test_widgets_rendering() {
        let ctx = egui::Context::default();
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                // Hi-Res pill
                let res_hires = quality_badge_pill(ui, "FLAC 24-bit/96kHz", true);
                assert!(res_hires.rect.width() > 0.0);
                assert!(res_hires.rect.height() >= 16.0);

                // Lossless pill
                let res_lossless = quality_badge_pill(ui, "LOSSLESS", false);
                assert!(res_lossless.rect.width() > 0.0);
                assert!(res_lossless.rect.height() >= 16.0);

                // Active segmented tab button
                let res_active = segmented_tab_button(ui, "Queue", true);
                assert!(res_active.rect.width() > 0.0);
                assert!(res_active.rect.height() >= 26.0);

                // Inactive segmented tab button
                let res_inactive = segmented_tab_button(ui, "Lyrics", false);
                assert!(res_inactive.rect.width() > 0.0);
                assert!(res_inactive.rect.height() >= 26.0);
            });
        });
    }
}
