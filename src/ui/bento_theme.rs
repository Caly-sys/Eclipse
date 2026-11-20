use egui::{Color32, FontId, Frame, Margin, Response, Rounding, Sense, Stroke, Ui, Vec2};

// ============================================================================
// 1. Feishin Matte-Slate Surface Ramp & Design Tokens
// ============================================================================

/// Top chrome bar & bottom player bar surface (#12151A)
pub const BG_BAR: Color32 = Color32::from_rgb(18, 21, 26);

/// Left navigation rail & right inspector surface (#161A20)
pub const BG_SIDEBAR: Color32 = Color32::from_rgb(22, 26, 32);

/// Main center viewport surface (#1D2129)
pub const BG_CENTER: Color32 = Color32::from_rgb(29, 33, 41);

/// Borderless elevated cards, search input, secondary buttons (#252A34)
pub const BG_CARD: Color32 = Color32::from_rgb(37, 42, 52);

/// Hover highlight for elevated cards and buttons (#2F3542)
pub const BG_CARD_HOVER: Color32 = Color32::from_rgb(47, 53, 66);

/// Ultra-subtle dark pane divider - NEVER white (#262B35)
pub const SEAM_DARK: Color32 = Color32::from_rgb(38, 43, 53);

/// Primary vibrant brand accent (play button, seek fill, active dot) (#9C6CFF)
pub const ACCENT_PRIMARY: Color32 = Color32::from_rgb(156, 108, 255);

/// Soft pill fill for active navigation items and active tabs
/// Matches Color32::from_rgba_unmultiplied(156, 108, 255, 34)
pub const ACCENT_SOFT: Color32 = Color32::from_rgba_premultiplied(59, 39, 102, 34);

/// High-contrast crisp text for track titles and active lyrics (#F0F2F6)
pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(240, 242, 246);

/// Secondary text for artists, album names, and subheadings (#A8AEBC)
pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(168, 174, 188);

/// Muted secondary text for metadata and section headers (#767C8A)
pub const TEXT_MUTED: Color32 = Color32::from_rgb(118, 124, 138);

/// Faint timestamps, inactive lyrics, and shortcut keys (#565C69)
pub const TEXT_FAINT: Color32 = Color32::from_rgb(86, 92, 105);

// Compatibility aliases for bento components and external consumers
pub const BG_CANVAS: Color32 = BG_BAR;
pub const BG_PANEL: Color32 = BG_CENTER;
pub const BG_DOCK: Color32 = BG_BAR;
pub const BORDER_SUBTLE: Color32 = SEAM_DARK;
pub const ACCENT_PURPLE: Color32 = ACCENT_PRIMARY;
pub const ACCENT_PURPLE_SOFT: Color32 = ACCENT_SOFT;

// Quality Badge Tokens (Hi-Res: Amber-Gold Glass, Lossless: Purple Glass)
/// Matches Color32::from_rgba_unmultiplied(245, 194, 66, 32)
pub const BADGE_HIRES_BG: Color32 = Color32::from_rgba_premultiplied(95, 74, 20, 32);
/// Matches Color32::from_rgba_unmultiplied(245, 194, 66, 90)
pub const BADGE_HIRES_BORDER: Color32 = Color32::from_rgba_premultiplied(154, 121, 38, 90);
/// Text: rgb(255, 224, 130) (#FFE082)
pub const BADGE_HIRES_TEXT: Color32 = Color32::from_rgb(255, 224, 130);

/// Matches Color32::from_rgba_unmultiplied(156, 108, 255, 36)
pub const BADGE_LOSSLESS_BG: Color32 = Color32::from_rgba_premultiplied(61, 40, 105, 36);
/// Transparent border for lossless (zero border)
pub const BADGE_LOSSLESS_BORDER: Color32 = Color32::TRANSPARENT;
/// Text: rgb(212, 194, 255) (#D4C2FF)
pub const BADGE_LOSSLESS_TEXT: Color32 = Color32::from_rgb(212, 194, 255);

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

/// Center stage panel frame with flat matte fill BG_CENTER and Stroke::NONE.
pub fn bento_panel_frame() -> Frame {
    Frame::none()
        .fill(BG_CENTER)
        .rounding(Rounding::ZERO)
        .stroke(Stroke::NONE)
        .inner_margin(Margin::same(10.0_f32))
}

/// Sidebar rail and right inspector frame with flat matte fill BG_SIDEBAR and Stroke::NONE.
pub fn bento_sidebar_frame() -> Frame {
    Frame::none()
        .fill(BG_SIDEBAR)
        .rounding(Rounding::ZERO)
        .stroke(Stroke::NONE)
        .inner_margin(Margin::same(10.0_f32))
}

/// Floating bottom player bar frame with flat matte fill BG_BAR and Stroke::NONE.
pub fn bento_dock_frame() -> Frame {
    Frame::none()
        .fill(BG_BAR)
        .rounding(Rounding::ZERO)
        .stroke(Stroke::NONE)
        .inner_margin(Margin::symmetric(14.0_f32, 8.0_f32))
}

/// Borderless elevated card frame with fill BG_CARD and Stroke::NONE.
pub fn bento_card_frame() -> Frame {
    Frame::none()
        .fill(BG_CARD)
        .rounding(Rounding::same(CARD_RADIUS))
        .stroke(Stroke::NONE)
        .inner_margin(Margin::same(8.0_f32))
}

// ============================================================================
// 3. Reusable Widgets & Theming
// ============================================================================

/// Configures ctx.set_visuals(...) with Feishin matte-slate theme, eliminating default
/// bright white outlines and setting slim floating dark scrollbars, dark panel fills,
/// and muted selection colors.
pub fn apply_bento_visuals(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();

    // Matte-slate fills matching the surface ramp
    visuals.panel_fill = BG_CENTER;
    visuals.window_fill = BG_SIDEBAR;
    visuals.extreme_bg_color = BG_BAR;
    visuals.faint_bg_color = BG_CARD;
    visuals.code_bg_color = BG_CARD;
    visuals.window_stroke = Stroke::NONE;
    visuals.window_rounding = Rounding::same(BENTO_RADIUS);
    visuals.menu_rounding = Rounding::same(CARD_RADIUS);

    // Eliminate default bright white outlines on widgets
    visuals.widgets.noninteractive.bg_fill = BG_CARD;
    visuals.widgets.noninteractive.weak_bg_fill = BG_CENTER;
    visuals.widgets.noninteractive.bg_stroke = Stroke::NONE;
    visuals.widgets.noninteractive.rounding = Rounding::same(CARD_RADIUS);
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, TEXT_PRIMARY);

    visuals.widgets.inactive.bg_fill = BG_CARD;
    visuals.widgets.inactive.weak_bg_fill = BG_CARD;
    visuals.widgets.inactive.bg_stroke = Stroke::NONE;
    visuals.widgets.inactive.rounding = Rounding::same(CARD_RADIUS);
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, TEXT_MUTED);

    visuals.widgets.hovered.bg_fill = BG_CARD_HOVER;
    visuals.widgets.hovered.weak_bg_fill = BG_CARD_HOVER;
    visuals.widgets.hovered.bg_stroke = Stroke::NONE;
    visuals.widgets.hovered.rounding = Rounding::same(CARD_RADIUS);
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0_f32, TEXT_PRIMARY);

    visuals.widgets.active.bg_fill = ACCENT_SOFT;
    visuals.widgets.active.weak_bg_fill = ACCENT_SOFT;
    visuals.widgets.active.bg_stroke = Stroke::NONE;
    visuals.widgets.active.rounding = Rounding::same(CARD_RADIUS);
    visuals.widgets.active.fg_stroke = Stroke::new(1.0_f32, TEXT_PRIMARY);

    visuals.widgets.open.bg_fill = BG_CARD;
    visuals.widgets.open.weak_bg_fill = BG_CARD;
    visuals.widgets.open.bg_stroke = Stroke::NONE;
    visuals.widgets.open.rounding = Rounding::same(CARD_RADIUS);

    // Muted brand selection colors
    visuals.selection.bg_fill = ACCENT_SOFT;
    visuals.selection.stroke = Stroke::NONE;

    ctx.set_visuals(visuals);

    // Subtle floating dark scrollbars with zero bright borders
    ctx.style_mut(|style| {
        style.spacing.scroll.bar_width = 6.0_f32;
        style.spacing.scroll.bar_inner_margin = 2.0_f32;
        style.spacing.scroll.bar_outer_margin = 2.0_f32;
        style.spacing.scroll.handle_min_length = 24.0_f32;
        style.spacing.scroll.dormant_background_opacity = 0.0_f32;
        style.spacing.scroll.active_background_opacity = 0.0_f32;
        style.spacing.scroll.interact_background_opacity = 0.0_f32;
        style.spacing.scroll.dormant_handle_opacity = 0.08_f32;
        style.spacing.scroll.active_handle_opacity = 0.16_f32;
        style.spacing.scroll.interact_handle_opacity = 0.32_f32;
    });
}

/// Draws a clean quality badge pill:
/// - If `is_hires`: dark amber-gold glass (fill: rgba(245, 194, 66, 32), stroke: 1.0, rgba(245, 194, 66, 90), text rgb(255, 224, 130)).
/// - Else (standard lossless / format): purple glass (fill: rgba(156, 108, 255, 36), stroke: Stroke::NONE, text rgb(212, 194, 255)).
/// - Text font: proportional 9.5px bold, rounding 5.0px.
pub fn quality_badge_pill(ui: &mut Ui, label: &str, is_hires: bool) -> Response {
    let (bg, border_stroke, text_col) = if is_hires {
        (
            BADGE_HIRES_BG,
            Stroke::new(1.0_f32, BADGE_HIRES_BORDER),
            BADGE_HIRES_TEXT,
        )
    } else {
        (
            BADGE_LOSSLESS_BG,
            Stroke::NONE,
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
        if border_stroke != Stroke::NONE {
            ui.painter().rect_stroke(rect, rounding, border_stroke);
        }

        let text_pos = egui::pos2(
            rect.min.x + (rect.width() - text_size.x) * 0.5_f32,
            rect.min.y + (rect.height() - text_size.y) * 0.5_f32,
        );
        ui.painter().galley(text_pos, galley, text_col);
    }

    response
}

/// Interactive pill tab switcher:
/// - If `is_active`: background `ACCENT_SOFT`, text `TEXT_PRIMARY` (bold, 12.0px), rounding `Rounding::same(6.0)`.
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
                ACCENT_SOFT,
                TEXT_PRIMARY,
                Stroke::NONE,
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
        assert_eq!(BG_BAR, Color32::from_rgb(18, 21, 26));
        assert_eq!(BG_SIDEBAR, Color32::from_rgb(22, 26, 32));
        assert_eq!(BG_CENTER, Color32::from_rgb(29, 33, 41));
        assert_eq!(BG_CARD, Color32::from_rgb(37, 42, 52));
        assert_eq!(BG_CARD_HOVER, Color32::from_rgb(47, 53, 66));
        assert_eq!(SEAM_DARK, Color32::from_rgb(38, 43, 53));
        assert_eq!(ACCENT_PRIMARY, Color32::from_rgb(156, 108, 255));
        assert_eq!(ACCENT_SOFT, Color32::from_rgba_unmultiplied(156, 108, 255, 34));
        assert_eq!(TEXT_PRIMARY, Color32::from_rgb(240, 242, 246));
        assert_eq!(TEXT_SECONDARY, Color32::from_rgb(168, 174, 188));
        assert_eq!(TEXT_MUTED, Color32::from_rgb(118, 124, 138));
        assert_eq!(TEXT_FAINT, Color32::from_rgb(86, 92, 105));

        // Compatibility aliases
        assert_eq!(BG_CANVAS, BG_BAR);
        assert_eq!(BG_PANEL, BG_CENTER);
        assert_eq!(BG_DOCK, BG_BAR);
        assert_eq!(BORDER_SUBTLE, SEAM_DARK);
        assert_eq!(ACCENT_PURPLE, ACCENT_PRIMARY);
        assert_eq!(ACCENT_PURPLE_SOFT, ACCENT_SOFT);
    }

    #[test]
    fn test_quality_badge_tokens() {
        assert_eq!(BADGE_HIRES_BG, Color32::from_rgba_unmultiplied(245, 194, 66, 32));
        assert_eq!(BADGE_HIRES_BORDER, Color32::from_rgba_unmultiplied(245, 194, 66, 90));
        assert_eq!(BADGE_HIRES_TEXT, Color32::from_rgb(255, 224, 130));

        assert_eq!(BADGE_LOSSLESS_BG, Color32::from_rgba_unmultiplied(156, 108, 255, 36));
        assert_eq!(BADGE_LOSSLESS_BORDER, Color32::TRANSPARENT);
        assert_eq!(BADGE_LOSSLESS_TEXT, Color32::from_rgb(212, 194, 255));
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
        assert_eq!(panel.fill, BG_CENTER);
        assert_eq!(panel.rounding, Rounding::ZERO);
        assert_eq!(panel.stroke, Stroke::NONE);
        assert_eq!(panel.inner_margin, Margin::same(10.0_f32));

        let sidebar = bento_sidebar_frame();
        assert_eq!(sidebar.fill, BG_SIDEBAR);
        assert_eq!(sidebar.rounding, Rounding::ZERO);
        assert_eq!(sidebar.stroke, Stroke::NONE);
        assert_eq!(sidebar.inner_margin, Margin::same(10.0_f32));

        let dock = bento_dock_frame();
        assert_eq!(dock.fill, BG_BAR);
        assert_eq!(dock.rounding, Rounding::ZERO);
        assert_eq!(dock.stroke, Stroke::NONE);
        assert_eq!(dock.inner_margin, Margin::symmetric(14.0_f32, 8.0_f32));

        let card = bento_card_frame();
        assert_eq!(card.fill, BG_CARD);
        assert_eq!(card.rounding, Rounding::same(CARD_RADIUS));
        assert_eq!(card.stroke, Stroke::NONE);
        assert_eq!(card.inner_margin, Margin::same(8.0_f32));
    }

    #[test]
    fn test_apply_bento_visuals() {
        let ctx = egui::Context::default();
        apply_bento_visuals(&ctx);

        let visuals = ctx.style().visuals.clone();
        assert!(visuals.dark_mode);
        assert_eq!(visuals.panel_fill, BG_CENTER);
        assert_eq!(visuals.window_fill, BG_SIDEBAR);
        assert_eq!(visuals.extreme_bg_color, BG_BAR);
        assert_eq!(visuals.window_stroke, Stroke::NONE);
        assert_eq!(visuals.window_rounding, Rounding::same(BENTO_RADIUS));
        assert_eq!(visuals.menu_rounding, Rounding::same(CARD_RADIUS));
        assert_eq!(visuals.widgets.noninteractive.bg_stroke, Stroke::NONE);
        assert_eq!(visuals.widgets.inactive.bg_stroke, Stroke::NONE);
        assert_eq!(visuals.widgets.hovered.bg_stroke, Stroke::NONE);
        assert_eq!(visuals.widgets.active.bg_stroke, Stroke::NONE);
        assert_eq!(visuals.widgets.open.bg_stroke, Stroke::NONE);
        assert_eq!(visuals.widgets.inactive.weak_bg_fill, BG_CARD);
        assert_eq!(visuals.widgets.hovered.weak_bg_fill, BG_CARD_HOVER);
        assert_eq!(visuals.widgets.hovered.bg_fill, BG_CARD_HOVER);
        assert_eq!(visuals.selection.bg_fill, ACCENT_SOFT);
        assert_eq!(visuals.selection.stroke, Stroke::NONE);

        let spacing = ctx.style().spacing.clone();
        assert_eq!(spacing.scroll.bar_width, 6.0_f32);
        assert_eq!(spacing.scroll.bar_inner_margin, 2.0_f32);
        assert_eq!(spacing.scroll.bar_outer_margin, 2.0_f32);
        assert_eq!(spacing.scroll.handle_min_length, 24.0_f32);
        assert_eq!(spacing.scroll.dormant_background_opacity, 0.0_f32);
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
