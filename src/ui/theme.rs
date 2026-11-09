use egui::{Color32, Frame, Margin, Rounding, Stroke};

/// Central Design Tokens module for Eclipse.
/// Defines colors, surfaces, borders, typography, and standard frame geometries
/// defining the signature dark purple glass aesthetic.
#[derive(Debug, Clone)]
pub struct Theme {
    pub name: String,
    // Backgrounds & Surfaces
    pub bg: Color32,
    pub surface: Color32,
    pub surface_hover: Color32,
    pub card: Color32,
    // Hairline Glass Borders
    pub border: Color32,
    pub border_accent: Color32,
    // Typography
    pub text_primary: Color32,
    pub text_secondary: Color32,
    pub text_muted: Color32,
    // Accents
    pub accent: Color32,
    pub accent_glow: Color32,
    pub accent_warm: Color32,
}

impl Theme {
    pub fn from_name(name: &str, custom_accent: Option<[u8; 3]>) -> Self {
        let accent_override = custom_accent.map(|[r, g, b]| Color32::from_rgb(r, g, b));

        match name {
            "Obsidian" => {
                let accent = accent_override.unwrap_or(Color32::from_rgb(179, 136, 255));
                Self {
                    name: "Obsidian".into(),
                    bg: Color32::from_rgb(10, 10, 14),
                    surface: Color32::from_rgba_premultiplied(18, 18, 26, 200),
                    surface_hover: Color32::from_rgba_premultiplied(30, 30, 42, 210),
                    card: Color32::from_rgba_premultiplied(22, 22, 32, 175),
                    border: Color32::from_rgba_premultiplied(255, 255, 255, 24),
                    border_accent: Color32::from_rgba_premultiplied(179, 136, 255, 80),
                    text_primary: Color32::from_rgb(240, 240, 248),
                    text_secondary: Color32::from_rgb(160, 160, 180),
                    text_muted: Color32::from_rgb(100, 100, 120),
                    accent,
                    accent_glow: Color32::from_rgba_premultiplied(accent.r(), accent.g(), accent.b(), 90),
                    accent_warm: Color32::from_rgb(255, 178, 92),
                }
            }
            "Glacier" => {
                let accent = accent_override.unwrap_or(Color32::from_rgb(100, 181, 246));
                Self {
                    name: "Glacier".into(),
                    bg: Color32::from_rgb(9, 14, 22),
                    surface: Color32::from_rgba_premultiplied(16, 25, 40, 195),
                    surface_hover: Color32::from_rgba_premultiplied(26, 40, 62, 210),
                    card: Color32::from_rgba_premultiplied(18, 28, 46, 170),
                    border: Color32::from_rgba_premultiplied(100, 181, 246, 36),
                    border_accent: Color32::from_rgba_premultiplied(100, 181, 246, 90),
                    text_primary: Color32::from_rgb(235, 245, 255),
                    text_secondary: Color32::from_rgb(140, 170, 205),
                    text_muted: Color32::from_rgb(85, 110, 140),
                    accent,
                    accent_glow: Color32::from_rgba_premultiplied(accent.r(), accent.g(), accent.b(), 90),
                    accent_warm: Color32::from_rgb(255, 178, 92),
                }
            }
            "Nebula" => {
                let accent = accent_override.unwrap_or(Color32::from_rgb(179, 136, 255));
                Self {
                    name: "Nebula".into(),
                    bg: Color32::from_rgb(13, 11, 24),
                    surface: Color32::from_rgba_premultiplied(24, 19, 42, 195),
                    surface_hover: Color32::from_rgba_premultiplied(40, 32, 68, 210),
                    card: Color32::from_rgba_premultiplied(28, 22, 50, 170),
                    border: Color32::from_rgba_premultiplied(179, 136, 255, 36),
                    border_accent: Color32::from_rgba_premultiplied(179, 136, 255, 90),
                    text_primary: Color32::from_rgb(245, 238, 255),
                    text_secondary: Color32::from_rgb(175, 155, 215),
                    text_muted: Color32::from_rgb(115, 100, 150),
                    accent,
                    accent_glow: Color32::from_rgba_premultiplied(accent.r(), accent.g(), accent.b(), 90),
                    accent_warm: Color32::from_rgb(255, 178, 92),
                }
            }
            "Verdant" => {
                let accent = accent_override.unwrap_or(Color32::from_rgb(105, 240, 174));
                Self {
                    name: "Verdant".into(),
                    bg: Color32::from_rgb(10, 18, 14),
                    surface: Color32::from_rgba_premultiplied(17, 32, 25, 195),
                    surface_hover: Color32::from_rgba_premultiplied(28, 52, 40, 210),
                    card: Color32::from_rgba_premultiplied(20, 38, 30, 170),
                    border: Color32::from_rgba_premultiplied(105, 240, 174, 36),
                    border_accent: Color32::from_rgba_premultiplied(105, 240, 174, 90),
                    text_primary: Color32::from_rgb(235, 255, 242),
                    text_secondary: Color32::from_rgb(145, 195, 165),
                    text_muted: Color32::from_rgb(85, 125, 105),
                    accent,
                    accent_glow: Color32::from_rgba_premultiplied(accent.r(), accent.g(), accent.b(), 90),
                    accent_warm: Color32::from_rgb(255, 178, 92),
                }
            }
            "Ember" => {
                let accent = accent_override.unwrap_or(Color32::from_rgb(255, 82, 82));
                Self {
                    name: "Ember".into(),
                    bg: Color32::from_rgb(20, 10, 12),
                    surface: Color32::from_rgba_premultiplied(36, 18, 20, 195),
                    surface_hover: Color32::from_rgba_premultiplied(56, 28, 32, 210),
                    card: Color32::from_rgba_premultiplied(42, 20, 24, 170),
                    border: Color32::from_rgba_premultiplied(255, 82, 82, 36),
                    border_accent: Color32::from_rgba_premultiplied(255, 82, 82, 90),
                    text_primary: Color32::from_rgb(255, 238, 240),
                    text_secondary: Color32::from_rgb(215, 155, 160),
                    text_muted: Color32::from_rgb(145, 95, 100),
                    accent,
                    accent_glow: Color32::from_rgba_premultiplied(accent.r(), accent.g(), accent.b(), 90),
                    accent_warm: Color32::from_rgb(255, 178, 92),
                }
            }
            // Signature Default: Eclipse (Deep Purple Glass Theme)
            _ => {
                let accent = accent_override.unwrap_or(Color32::from_rgb(179, 136, 255));
                Self {
                    name: "Eclipse".into(),
                    bg: Color32::from_rgb(11, 9, 20),
                    surface: Color32::from_rgba_premultiplied(20, 16, 34, 190),
                    surface_hover: Color32::from_rgba_premultiplied(38, 30, 65, 200),
                    card: Color32::from_rgba_premultiplied(28, 22, 48, 165),
                    border: Color32::from_rgba_premultiplied(180, 140, 255, 36),
                    border_accent: Color32::from_rgba_premultiplied(180, 140, 255, 90),
                    text_primary: Color32::from_rgb(244, 240, 255),
                    text_secondary: Color32::from_rgb(178, 168, 207),
                    text_muted: Color32::from_rgb(109, 100, 136),
                    accent,
                    accent_glow: Color32::from_rgba_premultiplied(accent.r(), accent.g(), accent.b(), 90),
                    accent_warm: Color32::from_rgb(255, 178, 92),
                }
            }
        }
    }

    /// Standard frosted glass frame for structural panels (sidebar, playerbar, header bars)
    pub fn glass_frame(&self, rounding: f32) -> Frame {
        Frame {
            inner_margin: Margin::same(12.0_f32),
            outer_margin: Margin::ZERO,
            rounding: Rounding::same(rounding),
            fill: self.surface,
            stroke: Stroke::new(1.0_f32, self.border),
            shadow: egui::epaint::Shadow::NONE,
        }
    }

    /// Standard frosted card frame for track cards, list containers, and popovers
    pub fn card_frame(&self, rounding: f32) -> Frame {
        Frame {
            inner_margin: Margin::same(12.0_f32),
            outer_margin: Margin::ZERO,
            rounding: Rounding::same(rounding),
            fill: self.card,
            stroke: Stroke::new(1.0_f32, self.border),
            shadow: egui::epaint::Shadow::NONE,
        }
    }

    /// Standard pill badge frame for lossless, hi-res, and format tags
    pub fn badge_frame(&self, bg: Color32, border: Color32) -> Frame {
        Frame {
            inner_margin: Margin::symmetric(6.0_f32, 2.0_f32),
            outer_margin: Margin::ZERO,
            rounding: Rounding::same(10.0_f32),
            fill: bg,
            stroke: Stroke::new(1.0_f32, border),
            shadow: egui::epaint::Shadow::NONE,
        }
    }

    /// Application canvas frame
    pub fn panel_frame(&self) -> Frame {
        self.panel_frame_with_state(true)
    }

    /// Application canvas frame adapting to floating (rounded corners with luminous border)
    /// or maximized/fullscreen (flush square edges)
    pub fn panel_frame_with_state(&self, is_floating: bool) -> Frame {
        let (rounding, stroke, inner_margin) = if is_floating {
            (
                Rounding::same(12.0_f32),
                Stroke::new(1.0_f32, self.border_accent),
                Margin::same(10.0_f32),
            )
        } else {
            (
                Rounding::ZERO,
                Stroke::NONE,
                Margin::same(8.0_f32),
            )
        };

        Frame {
            inner_margin,
            outer_margin: Margin::ZERO,
            rounding,
            fill: self.bg,
            stroke,
            shadow: egui::epaint::Shadow::NONE,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_eclipse_theme_tokens() {
        let theme = Theme::from_name("Eclipse", None);
        assert_eq!(theme.bg, Color32::from_rgb(11, 9, 20));
        assert_eq!(theme.surface, Color32::from_rgba_premultiplied(20, 16, 34, 190));
        assert_eq!(theme.card, Color32::from_rgba_premultiplied(28, 22, 48, 165));
        assert_eq!(theme.surface_hover, Color32::from_rgba_premultiplied(38, 30, 65, 200));
        assert_eq!(theme.border, Color32::from_rgba_premultiplied(180, 140, 255, 36));
        assert_eq!(theme.border_accent, Color32::from_rgba_premultiplied(180, 140, 255, 90));
        assert_eq!(theme.accent, Color32::from_rgb(179, 136, 255));
        assert_eq!(theme.accent_glow, Color32::from_rgba_premultiplied(179, 136, 255, 90));
        assert_eq!(theme.accent_warm, Color32::from_rgb(255, 178, 92));
        assert_eq!(theme.text_primary, Color32::from_rgb(244, 240, 255));
        assert_eq!(theme.text_secondary, Color32::from_rgb(178, 168, 207));
        assert_eq!(theme.text_muted, Color32::from_rgb(109, 100, 136));
    }

    #[test]
    fn test_theme_frames_helper_methods() {
        let theme = Theme::from_name("Eclipse", None);

        let glass = theme.glass_frame(12.0);
        assert_eq!(glass.fill, theme.surface);
        assert_eq!(glass.stroke.color, theme.border);
        assert_eq!(glass.rounding, Rounding::same(12.0));

        let card = theme.card_frame(10.0);
        assert_eq!(card.fill, theme.card);
        assert_eq!(card.stroke.color, theme.border);
        assert_eq!(card.rounding, Rounding::same(10.0));

        let badge = theme.badge_frame(theme.card, theme.accent_warm);
        assert_eq!(badge.fill, theme.card);
        assert_eq!(badge.stroke.color, theme.accent_warm);

        let panel = theme.panel_frame();
        assert_eq!(panel.fill, theme.bg);
        assert_eq!(panel.rounding, Rounding::same(12.0));
        assert_eq!(panel.stroke, Stroke::new(1.0_f32, theme.border_accent));

        let panel_max = theme.panel_frame_with_state(false);
        assert_eq!(panel_max.fill, theme.bg);
        assert_eq!(panel_max.rounding, Rounding::ZERO);
        assert_eq!(panel_max.stroke, Stroke::NONE);
    }

    #[test]
    fn test_custom_accent_override() {
        let custom = [100, 200, 250];
        let theme = Theme::from_name("Eclipse", Some(custom));
        assert_eq!(theme.accent, Color32::from_rgb(100, 200, 250));
        assert_eq!(theme.accent_glow, Color32::from_rgba_premultiplied(100, 200, 250, 90));
    }
}
