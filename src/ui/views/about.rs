use crate::ui::components::pill_badge;
use crate::ui::icons::{paint_icon, Icon};
use crate::ui::theme::Theme;
use egui::{vec2, Layout, Rect, RichText, Stroke, Ui};

pub fn render_about(ui: &mut Ui, theme: &Theme) {
    ui.vertical_centered(|ui| {
        ui.add_space(20.0);

        let mut card = theme.glass_frame(16.0);
        card.inner_margin = egui::Margin::symmetric(36.0, 28.0);

        card.show(ui, |ui| {
            ui.set_max_width(580.0);

            // Glowing Brand Icon Badge
            let (icon_rect, _) = ui.allocate_exact_size(vec2(64.0, 64.0), egui::Sense::hover());
            ui.painter().circle_filled(
                icon_rect.center(),
                30.0,
                theme.accent_glow,
            );
            ui.painter().circle_stroke(
                icon_rect.center(),
                30.0,
                Stroke::new(1.5_f32, theme.border_accent),
            );
            let sparkle_rect = Rect::from_center_size(icon_rect.center(), vec2(32.0, 32.0));
            paint_icon(ui.painter(), Icon::Sparkle, sparkle_rect, theme.accent);

            ui.add_space(14.0);

            ui.label(
                RichText::new("UMBRA MUSIC PLAYER")
                    .size(24.0)
                    .color(theme.text_primary)
                    .strong(),
            );

            ui.label(
                RichText::new("v1.2.0 · Audiophile Rust Edition")
                    .size(13.5)
                    .color(theme.accent)
                    .strong(),
            );

            ui.add_space(6.0);

            ui.label(
                RichText::new(
                    "Ultra-low latency, bit-perfect music player crafted with Rust and egui.\nDesigned for pristine sound reproduction and seamless library exploration.",
                )
                .size(13.0)
                .color(theme.text_secondary),
            );

            ui.add_space(18.0);

            // Feature Badges
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = vec2(8.0, 8.0);
                pill_badge(ui, "HI-RES LOSSLESS", theme.accent_glow, theme.accent_warm);
                pill_badge(ui, "20Hz - 20kHz SPECTRUM", theme.card, theme.accent);
                pill_badge(ui, "SYNCED LRC LYRICS", theme.card, theme.text_primary);
                pill_badge(ui, "SMART PLAYLISTS", theme.card, theme.text_secondary);
                pill_badge(ui, "CJK ZERO-TOFU", theme.card, theme.text_secondary);
                pill_badge(ui, "FRAMELESS GLASS", theme.card, theme.text_muted);
            });

            ui.add_space(20.0);

            // Architecture Specs Card
            let arch_card = theme.card_frame(10.0);
            arch_card.show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(RichText::new("CORE ARCHITECTURE").size(11.0).color(theme.text_muted).strong());
                        ui.add_space(6.0);
                        tech_row(ui, "Audio Pipeline", "rodio 0.19 · cpal WASAPI · symphonia", theme);
                        tech_row(ui, "Metadata Engine", "lofty 0.21 (pure Rust tag decoder)", theme);
                        tech_row(ui, "GUI & Rendering", "egui 0.29.1 · glow OpenGL ES", theme);
                        tech_row(ui, "Local Database", "rusqlite 0.32 (bundled SQLite WAL)", theme);
                        tech_row(ui, "Lyrics Sync", "LRCLIB REST API & local .lrc parser", theme);
                    });
                });
            });

            ui.add_space(16.0);

            // Acknowledgements Card
            let ack_card = theme.card_frame(10.0);
            ack_card.show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.label(RichText::new("ACKNOWLEDGEMENTS").size(11.0).color(theme.text_muted).strong());
                    ui.add_space(6.0);
                    ui.label(
                        RichText::new(
                            "Umbra Rust is an independent Rust reimplementation inspired by Umbra, originally created by Mersad Shahidi (@Bitologist).\n\nThe original project inspired the interface design, music-player concept, and overall user experience. The Rust implementation is independently developed and is not officially affiliated with the original author.",
                        )
                        .size(11.5)
                        .color(theme.text_secondary),
                    );
                    ui.add_space(6.0);
                    ui.label(
                        RichText::new("Original project: https://github.com/sadult/umbra · MIT License")
                            .size(11.0)
                            .color(theme.accent),
                    );
                });
            });

            ui.add_space(16.0);

            ui.horizontal(|ui| {
                ui.label(RichText::new("License: MIT").size(11.5).color(theme.text_muted));
                ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(RichText::new("Platform: Windows x86_64").size(11.5).color(theme.text_muted));
                });
            });
        });
    });
}

fn tech_row(ui: &mut Ui, label: &str, value: &str, theme: &Theme) {
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(label)
                .size(11.5)
                .color(theme.text_secondary)
                .strong(),
        );
        ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(RichText::new(value).size(11.5).color(theme.text_muted));
        });
    });
}
