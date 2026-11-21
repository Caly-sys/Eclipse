use egui::{Color32, FontId, Frame, Margin, Response, Rounding, Sense, Stroke, Ui, Vec2};
use crate::ui::theme::Theme;

// ============================================================================
// 1. Dynamic BentoPalette Engine (Settings Theme & Accent Driven)
// ============================================================================

/// Dynamic Feishin matte-slate design palette driven by Settings Theme & Accent color.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BentoPalette {
    /// Top chrome bar & bottom player dock surface
    pub bg_bar: Color32,
    /// Left navigation rail & right inspector surface
    pub bg_sidebar: Color32,
    /// Main center viewport surface
    pub bg_center: Color32,
    /// Borderless elevated cards, search inputs, secondary buttons
    pub bg_card: Color32,
    /// Hover highlight for elevated cards and buttons
    pub bg_card_hover: Color32,
    /// Ultra-subtle dark pane divider - NEVER white
    pub seam: Color32,
    /// Soft low-alpha atmospheric gradient glow matching active theme/accent
    pub header_glow: Color32,
    /// Primary brand accent (play button, seek fill, active indicators)
    pub accent: Color32,
    /// Soft accent tint for active pills, list selections, and lyric highlight (~34 alpha)
    pub accent_soft: Color32,
    /// Strong accent tint for borders and audio analyzer meters (~85 alpha)
    pub accent_strong: Color32,
    /// High-contrast text for titles and active lyrics
    pub text_primary: Color32,
    /// Secondary text for artists, album names, subheadings
    pub text_secondary: Color32,
    /// Muted secondary text for metadata and section headers
    pub text_muted: Color32,
}

impl BentoPalette {
    /// Constructs a dynamic palette from theme name and optional custom accent RGB.
    /// Supports all 6 themes: Eclipse, Obsidian, Ember, Verdant, Nebula, Glacier.
    pub fn from_settings(theme_name: &str, custom_accent: Option<[u8; 3]>) -> Self {
        let accent = match custom_accent {
            Some([r, g, b]) => Color32::from_rgb(r, g, b),
            None => match theme_name {
                "Glacier" => Color32::from_rgb(100, 181, 246),
                "Nebula" => Color32::from_rgb(179, 136, 255),
                "Verdant" => Color32::from_rgb(105, 240, 174),
                "Ember" => Color32::from_rgb(255, 82, 82),
                _ => Color32::from_rgb(156, 108, 255), // Eclipse / Obsidian purple
            },
        };

        let accent_soft = Color32::from_rgba_unmultiplied(accent.r(), accent.g(), accent.b(), 34);
        let accent_strong = Color32::from_rgba_unmultiplied(accent.r(), accent.g(), accent.b(), 85);
        let header_glow = Color32::from_rgba_unmultiplied(accent.r(), accent.g(), accent.b(), 22);

        let text_primary = Color32::from_rgb(240, 242, 246);
        let text_secondary = Color32::from_rgb(168, 174, 188);
        let text_muted = Color32::from_rgb(118, 124, 138);

        let (bg_bar, bg_sidebar, bg_center, bg_card, bg_card_hover, seam) = match theme_name {
            "Obsidian" => (
                Color32::from_rgb(10, 11, 14),   // #0A0B0E
                Color32::from_rgb(16, 18, 23),   // #101217
                Color32::from_rgb(21, 24, 31),   // #15181F
                Color32::from_rgb(30, 34, 43),   // #1E222B
                Color32::from_rgb(40, 45, 56),   // hover
                Color32::from_rgb(28, 32, 40),   // seam
            ),
            "Ember" => (
                Color32::from_rgb(26, 19, 19),   // #1A1313
                Color32::from_rgb(33, 24, 24),   // #211818
                Color32::from_rgb(41, 30, 30),   // #291E1E
                Color32::from_rgb(53, 39, 39),   // #352727
                Color32::from_rgb(66, 50, 50),   // hover
                Color32::from_rgb(51, 37, 37),   // seam
            ),
            "Verdant" => (
                Color32::from_rgb(16, 25, 22),   // #101916
                Color32::from_rgb(21, 33, 29),   // #15211D
                Color32::from_rgb(27, 42, 37),   // #1B2A25
                Color32::from_rgb(36, 55, 49),   // #243731
                Color32::from_rgb(46, 69, 62),   // hover
                Color32::from_rgb(35, 53, 47),   // seam
            ),
            "Nebula" => (
                Color32::from_rgb(19, 18, 32),   // #131220
                Color32::from_rgb(25, 24, 41),   // #191829
                Color32::from_rgb(33, 32, 54),   // #212036
                Color32::from_rgb(43, 42, 69),   // #2B2A45
                Color32::from_rgb(54, 53, 86),   // hover
                Color32::from_rgb(41, 40, 66),   // seam
            ),
            "Glacier" => (
                Color32::from_rgb(16, 23, 32),   // #101720
                Color32::from_rgb(22, 31, 43),   // #161F2B
                Color32::from_rgb(29, 40, 54),   // #1D2836
                Color32::from_rgb(38, 52, 70),   // #263446
                Color32::from_rgb(49, 66, 88),   // hover
                Color32::from_rgb(37, 51, 68),   // seam
            ),
            _ => (
                // Eclipse (default neutral cool slate)
                Color32::from_rgb(18, 21, 26),   // #12151A
                Color32::from_rgb(22, 26, 32),   // #161A20
                Color32::from_rgb(29, 33, 41),   // #1D2129
                Color32::from_rgb(37, 42, 52),   // #252A34
                Color32::from_rgb(47, 53, 66),   // hover
                Color32::from_rgb(36, 41, 50),   // seam
            ),
        };

        Self {
            bg_bar,
            bg_sidebar,
            bg_center,
            bg_card,
            bg_card_hover,
            seam,
            header_glow,
            accent,
            accent_soft,
            accent_strong,
            text_primary,
            text_secondary,
            text_muted,
        }
    }

    /// Derives BentoPalette directly from an active Theme reference.
    pub fn from_theme(theme: &Theme) -> Self {
        Self::from_settings(&theme.name, Some([theme.accent.r(), theme.accent.g(), theme.accent.b()]))
    }

    /// Center stage panel frame with flat matte fill and Stroke::NONE.
    pub fn panel_frame(&self) -> Frame {
        Frame::none()
            .fill(self.bg_center)
            .rounding(Rounding::ZERO)
            .stroke(Stroke::NONE)
            .inner_margin(Margin::same(10.0_f32))
    }

    /// Sidebar rail and right inspector frame with flat matte fill and Stroke::NONE.
    pub fn sidebar_frame(&self) -> Frame {
        Frame::none()
            .fill(self.bg_sidebar)
            .rounding(Rounding::ZERO)
            .stroke(Stroke::NONE)
            .inner_margin(Margin::ZERO)
    }

    /// Floating bottom player bar frame with flat matte fill and Stroke::NONE.
    pub fn dock_frame(&self) -> Frame {
        Frame::none()
            .fill(self.bg_bar)
            .rounding(Rounding::ZERO)
            .stroke(Stroke::NONE)
            .inner_margin(Margin::symmetric(14.0_f32, 8.0_f32))
    }

    /// Borderless elevated card frame with fill and Stroke::NONE.
    pub fn card_frame(&self) -> Frame {
        Frame::none()
            .fill(self.bg_card)
            .rounding(Rounding::same(CARD_RADIUS))
            .stroke(Stroke::NONE)
            .inner_margin(Margin::same(8.0_f32))
    }
}

impl Default for BentoPalette {
    fn default() -> Self {
        Self::from_settings("Eclipse", None)
    }
}

// ============================================================================
// Legacy Static Constants (Backward Compatibility)
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
pub const SEAM_DARK: Color32 = Color32::from_rgb(36, 41, 50);

/// Primary vibrant brand accent (play button, seek fill, active dot) (#9C6CFF)
pub const ACCENT_PRIMARY: Color32 = Color32::from_rgb(156, 108, 255);

/// Soft pill fill for active navigation items and active tabs
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
pub const BADGE_HIRES_BG: Color32 = Color32::from_rgba_premultiplied(95, 74, 20, 32);
pub const BADGE_HIRES_BORDER: Color32 = Color32::from_rgba_premultiplied(154, 121, 38, 90);
pub const BADGE_HIRES_TEXT: Color32 = Color32::from_rgb(255, 224, 130);

pub const BADGE_LOSSLESS_BG: Color32 = Color32::from_rgba_premultiplied(61, 40, 105, 36);
pub const BADGE_LOSSLESS_BORDER: Color32 = Color32::TRANSPARENT;
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

/// Configures ctx.set_visuals(...) with dynamic Feishin matte-slate theme driven by BentoPalette,
/// eliminating default bright white outlines and setting slim floating dark scrollbars,
/// dark panel fills, and muted selection colors.
pub fn apply_bento_visuals(ctx: &egui::Context, palette: &BentoPalette) {
    let mut visuals = egui::Visuals::dark();

    // Matte-slate fills matching the surface ramp
    visuals.panel_fill = palette.bg_center;
    visuals.window_fill = palette.bg_sidebar;
    visuals.extreme_bg_color = palette.bg_bar;
    visuals.faint_bg_color = palette.bg_card;
    visuals.code_bg_color = palette.bg_card;
    visuals.window_stroke = Stroke::NONE;
    visuals.window_rounding = Rounding::same(BENTO_RADIUS);
    visuals.menu_rounding = Rounding::same(CARD_RADIUS);

    // Eliminate default bright white outlines on widgets
    visuals.widgets.noninteractive.bg_fill = palette.bg_card;
    visuals.widgets.noninteractive.weak_bg_fill = palette.bg_center;
    visuals.widgets.noninteractive.bg_stroke = Stroke::NONE;
    visuals.widgets.noninteractive.rounding = Rounding::same(CARD_RADIUS);
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, palette.text_primary);

    visuals.widgets.inactive.bg_fill = palette.bg_card;
    visuals.widgets.inactive.weak_bg_fill = palette.bg_card;
    visuals.widgets.inactive.bg_stroke = Stroke::NONE;
    visuals.widgets.inactive.rounding = Rounding::same(CARD_RADIUS);
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, palette.text_muted);

    visuals.widgets.hovered.bg_fill = palette.bg_card_hover;
    visuals.widgets.hovered.weak_bg_fill = palette.bg_card_hover;
    visuals.widgets.hovered.bg_stroke = Stroke::NONE;
    visuals.widgets.hovered.rounding = Rounding::same(CARD_RADIUS);
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0_f32, palette.text_primary);

    visuals.widgets.active.bg_fill = palette.accent_soft;
    visuals.widgets.active.weak_bg_fill = palette.accent_soft;
    visuals.widgets.active.bg_stroke = Stroke::NONE;
    visuals.widgets.active.rounding = Rounding::same(CARD_RADIUS);
    visuals.widgets.active.fg_stroke = Stroke::new(1.0_f32, palette.text_primary);

    visuals.widgets.open.bg_fill = palette.bg_card;
    visuals.widgets.open.weak_bg_fill = palette.bg_card;
    visuals.widgets.open.bg_stroke = Stroke::NONE;
    visuals.widgets.open.rounding = Rounding::same(CARD_RADIUS);

    // Muted brand selection colors
    visuals.selection.bg_fill = palette.accent_soft;
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

/// Convenience overload using default Eclipse palette.
pub fn apply_bento_visuals_default(ctx: &egui::Context) {
    apply_bento_visuals(ctx, &BentoPalette::default());
}

/// Draws a clean quality badge pill:
/// - If `is_hires`: dark amber-gold glass.
/// - Else (standard lossless / format): accent-tinted soft glass with Stroke::NONE.
/// - Text font: proportional 9.5px bold, rounding 5.0px.
pub fn quality_badge_pill(ui: &mut Ui, label: &str, is_hires: bool, palette: &BentoPalette) -> Response {
    let (bg, border_stroke, text_col) = if is_hires {
        (
            BADGE_HIRES_BG,
            Stroke::new(1.0_f32, BADGE_HIRES_BORDER),
            BADGE_HIRES_TEXT,
        )
    } else {
        (
            palette.accent_soft,
            Stroke::NONE,
            Color32::from_rgb(220, 200, 255),
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
/// - If `is_active`: background `palette.accent_soft`, text `palette.text_primary` (bold, 12.0px), rounding `Rounding::same(6.0)`.
/// - Else: transparent background, text `palette.text_muted` (12.0px), with subtle `palette.bg_card_hover` on hover.
pub fn segmented_tab_button(ui: &mut Ui, text: &str, is_active: bool, palette: &BentoPalette) -> Response {
    let font_id = FontId::new(12.0_f32, egui::FontFamily::Proportional);

    let default_text_color = if is_active {
        palette.text_primary
    } else {
        palette.text_muted
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
                palette.accent_soft,
                palette.text_primary,
                Stroke::NONE,
            )
        } else if hovered {
            (
                palette.bg_card_hover,
                palette.text_primary,
                Stroke::NONE,
            )
        } else {
            (
                Color32::TRANSPARENT,
                palette.text_muted,
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
    fn test_bento_palette_themes() {
        let themes = ["Eclipse", "Obsidian", "Ember", "Verdant", "Nebula", "Glacier"];
        for name in &themes {
            let pal = BentoPalette::from_settings(name, None);
            // Verify surfaces exist and are non-transparent
            assert_eq!(pal.bg_bar.a(), 255);
            assert_eq!(pal.bg_sidebar.a(), 255);
            assert_eq!(pal.bg_center.a(), 255);
            assert_eq!(pal.bg_card.a(), 255);
            assert_eq!(pal.bg_card_hover.a(), 255);
            assert_eq!(pal.seam.a(), 255);
            assert_eq!(pal.text_primary.a(), 255);
            assert!(pal.accent_soft.a() > 0 && pal.accent_soft.a() < 100);
            assert!(pal.header_glow.a() > 0 && pal.header_glow.a() < 60);

            // Verify frame constructors
            let panel_frame = pal.panel_frame();
            assert_eq!(panel_frame.fill, pal.bg_center);
            assert_eq!(panel_frame.stroke, Stroke::NONE);

            let card_frame = pal.card_frame();
            assert_eq!(card_frame.fill, pal.bg_card);
            assert_eq!(card_frame.stroke, Stroke::NONE);
        }

        // Test custom accent propagation
        let custom_accent = Some([255, 82, 82]); // Coral red
        let pal_custom = BentoPalette::from_settings("Eclipse", custom_accent);
        assert_eq!(pal_custom.accent, Color32::from_rgb(255, 82, 82));

        // Test from_theme constructor
        let theme = Theme::from_name("Glacier", None);
        let pal_theme = BentoPalette::from_theme(&theme);
        assert_eq!(pal_theme.bg_bar, Color32::from_rgb(16, 23, 32));
        assert_eq!(pal_theme.accent, theme.accent);

        // Verify specific color ramp mappings for all 6 themes
        let eclipse = BentoPalette::from_settings("Eclipse", None);
        assert_eq!(eclipse.bg_bar, Color32::from_rgb(18, 21, 26));
        assert_eq!(eclipse.bg_sidebar, Color32::from_rgb(22, 26, 32));
        assert_eq!(eclipse.bg_center, Color32::from_rgb(29, 33, 41));
        assert_eq!(eclipse.bg_card, Color32::from_rgb(37, 42, 52));
        assert_eq!(eclipse.bg_card_hover, Color32::from_rgb(47, 53, 66));
        assert_eq!(eclipse.seam, Color32::from_rgb(36, 41, 50));

        let obsidian = BentoPalette::from_settings("Obsidian", None);
        assert_eq!(obsidian.bg_bar, Color32::from_rgb(10, 11, 14));
        assert_eq!(obsidian.bg_sidebar, Color32::from_rgb(16, 18, 23));
        assert_eq!(obsidian.bg_center, Color32::from_rgb(21, 24, 31));
        assert_eq!(obsidian.bg_card, Color32::from_rgb(30, 34, 43));
        assert_eq!(obsidian.bg_card_hover, Color32::from_rgb(40, 45, 56));
        assert_eq!(obsidian.seam, Color32::from_rgb(28, 32, 40));

        let ember = BentoPalette::from_settings("Ember", None);
        assert_eq!(ember.bg_bar, Color32::from_rgb(26, 19, 19));
        assert_eq!(ember.bg_sidebar, Color32::from_rgb(33, 24, 24));
        assert_eq!(ember.bg_center, Color32::from_rgb(41, 30, 30));
        assert_eq!(ember.bg_card, Color32::from_rgb(53, 39, 39));
        assert_eq!(ember.bg_card_hover, Color32::from_rgb(66, 50, 50));
        assert_eq!(ember.seam, Color32::from_rgb(51, 37, 37));

        let verdant = BentoPalette::from_settings("Verdant", None);
        assert_eq!(verdant.bg_bar, Color32::from_rgb(16, 25, 22));
        assert_eq!(verdant.bg_sidebar, Color32::from_rgb(21, 33, 29));
        assert_eq!(verdant.bg_center, Color32::from_rgb(27, 42, 37));
        assert_eq!(verdant.bg_card, Color32::from_rgb(36, 55, 49));
        assert_eq!(verdant.bg_card_hover, Color32::from_rgb(46, 69, 62));
        assert_eq!(verdant.seam, Color32::from_rgb(35, 53, 47));

        let nebula = BentoPalette::from_settings("Nebula", None);
        assert_eq!(nebula.bg_bar, Color32::from_rgb(19, 18, 32));
        assert_eq!(nebula.bg_sidebar, Color32::from_rgb(25, 24, 41));
        assert_eq!(nebula.bg_center, Color32::from_rgb(33, 32, 54));
        assert_eq!(nebula.bg_card, Color32::from_rgb(43, 42, 69));
        assert_eq!(nebula.bg_card_hover, Color32::from_rgb(54, 53, 86));
        assert_eq!(nebula.seam, Color32::from_rgb(41, 40, 66));

        let glacier = BentoPalette::from_settings("Glacier", None);
        assert_eq!(glacier.bg_bar, Color32::from_rgb(16, 23, 32));
        assert_eq!(glacier.bg_sidebar, Color32::from_rgb(22, 31, 43));
        assert_eq!(glacier.bg_center, Color32::from_rgb(29, 40, 54));
        assert_eq!(glacier.bg_card, Color32::from_rgb(38, 52, 70));
        assert_eq!(glacier.bg_card_hover, Color32::from_rgb(49, 66, 88));
        assert_eq!(glacier.seam, Color32::from_rgb(37, 51, 68));
    }

    #[test]
    fn test_color_tokens() {
        assert_eq!(BG_BAR, Color32::from_rgb(18, 21, 26));
        assert_eq!(BG_SIDEBAR, Color32::from_rgb(22, 26, 32));
        assert_eq!(BG_CENTER, Color32::from_rgb(29, 33, 41));
        assert_eq!(BG_CARD, Color32::from_rgb(37, 42, 52));
        assert_eq!(BG_CARD_HOVER, Color32::from_rgb(47, 53, 66));
        assert_eq!(SEAM_DARK, Color32::from_rgb(36, 41, 50));
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
        let palette = BentoPalette::default();
        apply_bento_visuals(&ctx, &palette);

        let visuals = ctx.style().visuals.clone();
        assert!(visuals.dark_mode);
        assert_eq!(visuals.panel_fill, palette.bg_center);
        assert_eq!(visuals.window_fill, palette.bg_sidebar);
        assert_eq!(visuals.extreme_bg_color, palette.bg_bar);
        assert_eq!(visuals.window_stroke, Stroke::NONE);
        assert_eq!(visuals.window_rounding, Rounding::same(BENTO_RADIUS));
        assert_eq!(visuals.menu_rounding, Rounding::same(CARD_RADIUS));
        assert_eq!(visuals.widgets.noninteractive.bg_stroke, Stroke::NONE);
        assert_eq!(visuals.widgets.inactive.bg_stroke, Stroke::NONE);
        assert_eq!(visuals.widgets.hovered.bg_stroke, Stroke::NONE);
        assert_eq!(visuals.widgets.active.bg_stroke, Stroke::NONE);
        assert_eq!(visuals.widgets.open.bg_stroke, Stroke::NONE);
        assert_eq!(visuals.widgets.inactive.weak_bg_fill, palette.bg_card);
        assert_eq!(visuals.widgets.hovered.weak_bg_fill, palette.bg_card_hover);
        assert_eq!(visuals.widgets.hovered.bg_fill, palette.bg_card_hover);
        assert_eq!(visuals.selection.bg_fill, palette.accent_soft);
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
        let palette = BentoPalette::default();
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                // Hi-Res pill
                let res_hires = quality_badge_pill(ui, "FLAC 24-bit/96kHz", true, &palette);
                assert!(res_hires.rect.width() > 0.0);
                assert!(res_hires.rect.height() >= 16.0);

                // Lossless pill
                let res_lossless = quality_badge_pill(ui, "LOSSLESS", false, &palette);
                assert!(res_lossless.rect.width() > 0.0);
                assert!(res_lossless.rect.height() >= 16.0);

                // Active segmented tab button
                let res_active = segmented_tab_button(ui, "Queue", true, &palette);
                assert!(res_active.rect.width() > 0.0);
                assert!(res_active.rect.height() >= 26.0);

                // Inactive segmented tab button
                let res_inactive = segmented_tab_button(ui, "Lyrics", false, &palette);
                assert!(res_inactive.rect.width() > 0.0);
                assert!(res_inactive.rect.height() >= 26.0);
            });
        });
    }
}
