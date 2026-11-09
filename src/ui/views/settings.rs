use crate::config::{AppConfig, VisualizerStyle};
use crate::ui::icons::{icon_button, paint_icon, render_icon, Icon};
use crate::ui::theme::Theme;
use egui::{vec2, Color32, Layout, Rect, RichText, Rounding, ScrollArea, Stroke, Ui};

pub fn render_settings(
    ui: &mut Ui,
    cfg: &mut AppConfig,
    theme: &Theme,
    on_rescan: &mut bool,
    on_add_folder: &mut bool,
    on_theme_changed: &mut bool,
) {
    ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing = vec2(0.0, 16.0);

            ui.horizontal(|ui| {
                render_icon(ui, Icon::Settings, 22.0, theme.accent);
                ui.add_space(4.0);
                ui.label(RichText::new("Settings").size(22.0).color(theme.text_primary).strong());
            });

            // 1. APPEARANCE
            let mut app_frame = theme.glass_frame(12.0);
            app_frame.inner_margin = egui::Margin::same(16.0);

            app_frame.show(ui, |ui| {
                ui.horizontal(|ui| {
                    render_icon(ui, Icon::Palette, 16.0, theme.accent);
                    ui.add_space(4.0);
                    ui.label(RichText::new("Appearance & Themes").size(16.0).color(theme.text_primary).strong());
                });
                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    ui.label(RichText::new("Theme:").color(theme.text_secondary));
                    for theme_name in &["Eclipse", "Obsidian", "Glacier", "Nebula", "Verdant", "Ember"] {
                        let is_active = cfg.theme == *theme_name;
                        if ui.selectable_label(is_active, *theme_name).clicked() {
                            cfg.theme = theme_name.to_string();
                            *on_theme_changed = true;
                        }
                    }
                });

                ui.add_space(10.0);

                // Accent colors
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Accent Color:").color(theme.text_secondary));

                    let presets = [
                        ("Violet (Signature)", [179, 136, 255]),
                        ("Amber (Warm)", [255, 178, 92]),
                        ("Cyan", [100, 181, 246]),
                        ("Emerald", [105, 240, 174]),
                        ("Coral", [255, 82, 82]),
                    ];

                    for (label, rgb) in &presets {
                        let c = Color32::from_rgb(rgb[0], rgb[1], rgb[2]);
                        let (rect, resp) = ui.allocate_exact_size(vec2(22.0, 22.0), egui::Sense::click());
                        ui.painter().rect_filled(rect, Rounding::same(11.0), c);

                        if cfg.accent_color == *rgb {
                            ui.painter().rect_stroke(rect, Rounding::same(11.0), Stroke::new(2.0_f32, Color32::WHITE));
                        }

                        if resp.on_hover_text(*label).clicked() {
                            cfg.accent_color = *rgb;
                            *on_theme_changed = true;
                        }
                    }
                });
            });

            // 2. AUDIO & VISUALIZER
            let mut audio_frame = theme.glass_frame(12.0);
            audio_frame.inner_margin = egui::Margin::same(16.0);

            audio_frame.show(ui, |ui| {
                ui.horizontal(|ui| {
                    render_icon(ui, Icon::BarChart, 16.0, theme.accent);
                    ui.add_space(4.0);
                    ui.label(RichText::new("Audio & Visualizer").size(16.0).color(theme.text_primary).strong());
                });
                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    ui.label(RichText::new("Visualizer Style:").color(theme.text_secondary));
                    if ui.selectable_label(cfg.visualizer_style == VisualizerStyle::Bars, "Bars").clicked() {
                        cfg.visualizer_style = VisualizerStyle::Bars;
                    }
                    if ui.selectable_label(cfg.visualizer_style == VisualizerStyle::Mirror, "Mirror").clicked() {
                        cfg.visualizer_style = VisualizerStyle::Mirror;
                    }
                    if ui.selectable_label(cfg.visualizer_style == VisualizerStyle::Scope, "Scope").clicked() {
                        cfg.visualizer_style = VisualizerStyle::Scope;
                    }
                    if ui.selectable_label(cfg.visualizer_style == VisualizerStyle::Radial, "Radial").clicked() {
                        cfg.visualizer_style = VisualizerStyle::Radial;
                    }
                });

                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    ui.label(RichText::new("Spectrum Bands:").color(theme.text_secondary));
                    let slider = egui::Slider::new(&mut cfg.visualizer_bands, 16..=64);
                    ui.add(slider);
                });
            });

            // 3. LIBRARY & FOLDERS
            let mut lib_frame = theme.glass_frame(12.0);
            lib_frame.inner_margin = egui::Margin::same(16.0);

            lib_frame.show(ui, |ui| {
                ui.horizontal(|ui| {
                    render_icon(ui, Icon::Folder, 16.0, theme.accent);
                    ui.add_space(4.0);
                    ui.label(RichText::new("Library Monitored Folders").size(16.0).color(theme.text_primary).strong());

                    ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                        let (btn_rect, btn_resp) = ui.allocate_exact_size(vec2(106.0, 26.0), egui::Sense::click());
                        let btn_bg = if btn_resp.hovered() { theme.surface_hover } else { theme.card };
                        let btn_stroke = if btn_resp.hovered() { Stroke::new(1.0_f32, theme.border_accent) } else { Stroke::new(1.0_f32, theme.border) };
                        ui.painter().rect_filled(btn_rect, Rounding::same(6.0), btn_bg);
                        ui.painter().rect_stroke(btn_rect, Rounding::same(6.0), btn_stroke);

                        let icon_r = Rect::from_center_size(egui::pos2(btn_rect.min.x + 14.0, btn_rect.center().y), vec2(11.0, 11.0));
                        paint_icon(ui.painter(), Icon::Plus, icon_r, theme.accent);

                        ui.painter().text(
                            egui::pos2(btn_rect.min.x + 26.0, btn_rect.center().y),
                            egui::Align2::LEFT_CENTER,
                            "Add Folder",
                            egui::FontId::proportional(11.5),
                            theme.accent,
                        );

                        if btn_resp.hovered() {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                        }
                        if btn_resp.clicked() {
                            *on_add_folder = true;
                        }
                    });
                });

                ui.add_space(8.0);

                let mut to_remove = None;
                for (i, folder) in cfg.music_folders.iter().enumerate() {
                    ui.horizontal(|ui| {
                        render_icon(ui, Icon::Folder, 14.0, theme.accent);
                        ui.label(RichText::new(folder).color(theme.text_primary).size(12.5));

                        ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                            if icon_button(ui, Icon::Close, 11.0, theme.text_muted).on_hover_text("Remove Folder").clicked() {
                                to_remove = Some(i);
                            }
                        });
                    });
                }

                if let Some(idx) = to_remove {
                    cfg.music_folders.remove(idx);
                }

                ui.add_space(10.0);

                let (scan_rect, scan_resp) = ui.allocate_exact_size(vec2(210.0, 30.0), egui::Sense::click());
                let scan_bg = if scan_resp.hovered() { theme.surface_hover } else { theme.card };
                let scan_stroke = if scan_resp.hovered() { Stroke::new(1.0_f32, theme.border_accent) } else { Stroke::new(1.0_f32, theme.border) };
                ui.painter().rect_filled(scan_rect, Rounding::same(8.0), scan_bg);
                ui.painter().rect_stroke(scan_rect, Rounding::same(8.0), scan_stroke);

                let icon_r = Rect::from_center_size(egui::pos2(scan_rect.min.x + 18.0, scan_rect.center().y), vec2(13.0, 13.0));
                paint_icon(ui.painter(), Icon::Refresh, icon_r, theme.accent);

                ui.painter().text(
                    egui::pos2(scan_rect.min.x + 34.0, scan_rect.center().y),
                    egui::Align2::LEFT_CENTER,
                    "Rescan Entire Library Now",
                    egui::FontId::proportional(12.5),
                    theme.accent,
                );

                if scan_resp.hovered() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                }
                if scan_resp.clicked() {
                    *on_rescan = true;
                }
            });

            // 4. ABOUT
            let mut about_frame = theme.glass_frame(12.0);
            about_frame.inner_margin = egui::Margin::same(16.0);

            about_frame.show(ui, |ui| {
                ui.horizontal(|ui| {
                    render_icon(ui, Icon::Sparkle, 16.0, theme.accent);
                    ui.add_space(4.0);
                    ui.label(RichText::new("About Umbra").size(16.0).color(theme.text_primary).strong());
                });
                ui.add_space(6.0);
                ui.label(
                    RichText::new("Umbra 1.2.0 (Rust Edition)")
                        .size(13.0)
                        .color(theme.accent)
                        .strong(),
                );
                ui.label(
                    RichText::new("Built with Rust, eframe/egui, rodio, symphonia & lofty.")
                        .size(12.0)
                        .color(theme.text_secondary),
                );
                ui.label(
                    RichText::new("Original project: github.com/sadult/umbra · music, in the dark.")
                        .size(11.5)
                        .color(theme.text_muted),
                );
            });
        });
}
