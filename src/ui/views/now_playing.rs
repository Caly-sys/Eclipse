use crate::audio::PlayerController;
use crate::config::VisualizerStyle;
use crate::ui::art::AlbumArtManager;
use crate::ui::components::{format_badge, pill_badge, render_spectrum};
use crate::ui::icons::{paint_icon, render_icon, Icon};
use crate::ui::theme::Theme;
use egui::{pos2, vec2, Color32, Layout, Rect, RichText, Rounding, ScrollArea, Stroke, Ui};
use std::time::Duration;

pub fn render_now_playing(
    ui: &mut Ui,
    player: &mut PlayerController,
    theme: &Theme,
    art_manager: &mut AlbumArtManager,
) {
    if let Some(track) = player.current_track().cloned() {
        let pos_secs = player.engine.position().as_secs_f64();
        let pos_ms = (pos_secs * 1000.0) as u64;

        ui.columns(3, |cols| {
            // =================================================================
            // COLUMN 1 (LEFT): Artwork, Track Info, Technical Badges
            // =================================================================
            cols[0].vertical(|ui| {
                ui.spacing_mut().item_spacing = vec2(0.0, 12.0);

                let mut art_frame = theme.glass_frame(16.0);
                art_frame.inner_margin = egui::Margin::same(16.0);

                art_frame.show(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        let card_width = (ui.available_width() - 10.0).clamp(180.0, 260.0);
                        let art_size = vec2(card_width, card_width);

                        // Ambient glow & drop shadow behind album art card
                        let (rect, _) = ui.allocate_exact_size(art_size, egui::Sense::hover());

                        // Multi-layered specular violet halo
                        ui.painter().rect_filled(
                            rect.expand(10.0),
                            Rounding::same(18.0),
                            Color32::from_rgba_premultiplied(
                                theme.accent.r(),
                                theme.accent.g(),
                                theme.accent.b(),
                                18,
                            ),
                        );
                        ui.painter().rect_filled(
                            rect.expand(4.0),
                            Rounding::same(16.0),
                            Color32::from_rgba_premultiplied(
                                theme.accent.r(),
                                theme.accent.g(),
                                theme.accent.b(),
                                32,
                            ),
                        );

                        // Album Art Image or fallback placeholder
                        let mut painted_cover = false;
                        if let Some(texture) = art_manager.get_or_load(ui.ctx(), &track.path) {
                            ui.painter().image(
                                texture.id(),
                                rect,
                                Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                                Color32::WHITE,
                            );
                            ui.painter().rect_stroke(rect, Rounding::same(14.0), Stroke::new(1.0_f32, theme.border_accent));
                            painted_cover = true;
                        }

                        if !painted_cover {
                            ui.painter().rect_filled(rect, Rounding::same(14.0), theme.card);
                            ui.painter().rect_stroke(rect, Rounding::same(14.0), Stroke::new(1.0_f32, theme.border_accent));
                            let icon_box = Rect::from_center_size(rect.center(), vec2(art_size.x * 0.36, art_size.x * 0.36));
                            paint_icon(ui.painter(), Icon::Disc, icon_box, theme.accent);
                        }

                        ui.add_space(14.0);

                        // Track Title
                        ui.label(
                            RichText::new(track.display_title())
                                .size(19.0)
                                .color(theme.text_primary)
                                .strong(),
                        );
                        ui.add_space(2.0);

                        // Artist
                        ui.label(
                            RichText::new(track.display_artist())
                                .size(14.5)
                                .color(theme.text_secondary),
                        );
                        ui.add_space(2.0);

                        // Album and Year
                        let album_year = if track.year > 0 {
                            format!("{} · {}", track.display_album(), track.year)
                        } else {
                            track.display_album().to_string()
                        };
                        ui.label(
                            RichText::new(album_year)
                                .size(12.5)
                                .color(theme.text_muted),
                        );

                        ui.add_space(10.0);

                        // Format Badges matching target-umbra.png
                        ui.horizontal_wrapped(|ui| {
                            ui.spacing_mut().item_spacing = vec2(6.0, 6.0);

                            // High-contrast format badge (FLAC, MP3, etc.)
                            format_badge(ui, &track.format, track.is_lossless(), theme);

                            // Bit depth & Sample rate badge (e.g. 24-BIT 48 KHZ)
                            let bit_depth = if track.is_hires() { "24-BIT" } else { "16-BIT" };
                            let sr_khz = if track.sample_rate > 0 {
                                format!("{} KHZ", track.sample_rate / 1000)
                            } else {
                                "44.1 KHZ".to_string()
                            };
                            pill_badge(
                                ui,
                                &format!("{} {}", bit_depth, sr_khz),
                                Color32::from_rgba_unmultiplied(38, 30, 62, 215),
                                Color32::from_rgb(220, 210, 245),
                            );

                            // Play count badge
                            pill_badge(
                                ui,
                                &format!("{} plays", track.play_count),
                                Color32::from_rgba_unmultiplied(38, 30, 62, 190),
                                theme.text_secondary,
                            );

                            // Lossless or Hi-Res badge
                            if track.is_hires() {
                                pill_badge(
                                    ui,
                                    "HI-RES LOSSLESS",
                                    Color32::from_rgba_unmultiplied(theme.accent_warm.r(), theme.accent_warm.g(), theme.accent_warm.b(), 56),
                                    theme.accent_warm,
                                );
                            } else if track.is_lossless() {
                                pill_badge(
                                    ui,
                                    "LOSSLESS",
                                    Color32::from_rgba_unmultiplied(theme.accent.r(), theme.accent.g(), theme.accent.b(), 56),
                                    theme.accent,
                                );
                            }
                        });
                    });
                });
            });

            // =================================================================
            // COLUMN 2 (CENTER): Synced Lyrics with Auto-Scrolling & Click-Seek
            // =================================================================
            cols[1].vertical(|ui| {
                let mut lyrics_frame = theme.glass_frame(16.0);
                lyrics_frame.inner_margin = egui::Margin::same(16.0);

                lyrics_frame.show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let (icon_box, _) = ui.allocate_exact_size(vec2(16.0, 16.0), egui::Sense::hover());
                        paint_icon(ui.painter(), Icon::Mic, icon_box, theme.accent);
                        ui.add_space(4.0);
                        ui.label(
                            RichText::new("SYNCED LYRICS")
                                .size(13.0)
                                .color(theme.text_primary)
                                .strong(),
                        );

                        if let Some(ref lyrics) = player.lyrics {
                            ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                                pill_badge(ui, &lyrics.source, theme.card, theme.text_muted);
                            });
                        }
                    });

                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(8.0);

                    if player.is_loading_lyrics {
                        ui.centered_and_justified(|ui| {
                            ui.label(RichText::new("Loading synced lyrics...").color(theme.text_muted).size(13.5));
                        });
                    } else if let Some(ref lyrics) = player.lyrics {
                        if lyrics.lines.is_empty() {
                            ui.centered_and_justified(|ui| {
                                ui.label(RichText::new("No lyrics available for this song.").color(theme.text_muted).size(13.0));
                            });
                        } else {
                            let current_idx = lyrics.current_index(pos_ms);

                            ScrollArea::vertical()
                                .auto_shrink([false, false])
                                .show(ui, |ui| {
                                    ui.spacing_mut().item_spacing = vec2(0.0, 8.0);

                                    for (i, line) in lyrics.lines.iter().enumerate() {
                                        let is_current = current_idx == Some(i);

                                        let text_color = if is_current {
                                            theme.text_primary // #f4f0ff
                                        } else {
                                            theme.text_muted // #6d6488
                                        };

                                        let size = if is_current { 18.0 } else { 14.5 };
                                        let mut rich = RichText::new(&line.text)
                                            .color(text_color)
                                            .size(size);

                                        if is_current {
                                            rich = rich.strong();
                                        }

                                        let (fill, stroke) = if is_current {
                                            (
                                                Color32::from_rgba_premultiplied(
                                                    theme.accent.r(),
                                                    theme.accent.g(),
                                                    theme.accent.b(),
                                                    30,
                                                ),
                                                Stroke::new(1.0_f32, theme.border_accent),
                                            )
                                        } else {
                                            (Color32::TRANSPARENT, Stroke::NONE)
                                        };

                                        let btn = ui.add(
                                            egui::Button::new(rich)
                                                .fill(fill)
                                                .stroke(stroke)
                                                .rounding(Rounding::same(8.0))
                                                .wrap(),
                                        );

                                        // Auto-scroll current line into center of view
                                        if is_current {
                                            btn.scroll_to_me(Some(egui::Align::Center));
                                        }

                                        // Click lyric line to seek directly to timestamp!
                                        if btn.clicked() && lyrics.is_synced {
                                            player.engine.seek(Duration::from_millis(line.timestamp_ms));
                                        }
                                    }
                                });
                        }
                    } else {
                        ui.centered_and_justified(|ui| {
                            ui.label(RichText::new("No lyrics found.").color(theme.text_muted).size(13.0));
                        });
                    }
                });
            });

            // =================================================================
            // COLUMN 3 (RIGHT): Audio Analyzer Card
            // =================================================================
            cols[2].vertical(|ui| {
                ui.spacing_mut().item_spacing = vec2(0.0, 10.0);

                let mut analyzer_frame = theme.glass_frame(16.0);
                analyzer_frame.inner_margin = egui::Margin::same(14.0);

                analyzer_frame.show(ui, |ui| {
                    // 1. Format Header with glowing HI-RES LOSSLESS / LOSSLESS badge
                    ui.horizontal(|ui| {
                        render_icon(ui, Icon::BarChart, 14.0, theme.accent);
                        ui.add_space(4.0);
                        ui.label(
                            RichText::new("AUDIO ANALYZER")
                                .size(12.5)
                                .color(theme.text_secondary)
                                .strong(),
                        );

                        ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                            if track.is_hires() {
                                pill_badge(
                                    ui,
                                    "HI-RES LOSSLESS",
                                    Color32::from_rgba_premultiplied(
                                        theme.accent_warm.r(),
                                        theme.accent_warm.g(),
                                        theme.accent_warm.b(),
                                        45,
                                    ),
                                    theme.accent_warm,
                                );
                            } else if track.is_lossless() {
                                pill_badge(
                                    ui,
                                    "LOSSLESS",
                                    Color32::from_rgba_premultiplied(
                                        theme.accent_warm.r(),
                                        theme.accent_warm.g(),
                                        theme.accent_warm.b(),
                                        35,
                                    ),
                                    theme.accent_warm,
                                );
                            } else {
                                pill_badge(
                                    ui,
                                    &track.format.to_uppercase(),
                                    theme.card,
                                    theme.text_secondary,
                                );
                            }
                        });
                    });

                    ui.add_space(8.0);

                    // 2. Technical Stats Sub-Card
                    let specs_card = theme.card_frame(8.0);
                    specs_card.show(ui, |ui| {
                        ui.vertical(|ui| {
                            ui.spacing_mut().item_spacing = vec2(0.0, 4.0);

                            stat_row(ui, "Format / Codec", &track.format.to_uppercase(), theme);

                            let sr_val = if track.sample_rate > 0 {
                                format!("{} Hz", track.sample_rate)
                            } else {
                                "44,100 Hz".to_string()
                            };
                            stat_row(ui, "Sample Rate", &sr_val, theme);

                            let bit_depth = if track.is_hires() { "24-bit" } else { "16-bit" };
                            let br_val = if track.bitrate_kbps > 0 {
                                format!("{} · {} kbps", bit_depth, track.bitrate_kbps)
                            } else {
                                format!("{} · Lossless", bit_depth)
                            };
                            stat_row(ui, "Bit Depth / Rate", &br_val, theme);

                            let ch_val = if track.channels == 1 {
                                "Mono (1.0 ch)"
                            } else {
                                "Stereo (2.0 ch)"
                            };
                            stat_row(ui, "Channels", ch_val, theme);
                        });
                    });

                    ui.add_space(8.0);

                    // 3. Real-Time Audio Metrics Sub-Card (Peak dBFS, LUFS, Dynamic Range)
                    let metrics_card = theme.card_frame(8.0);
                    metrics_card.show(ui, |ui| {
                        ui.horizontal(|ui| {
                            let col_w = (ui.available_width() - 8.0) / 3.0;

                            // Metric 1: Peak Level (dBFS)
                            ui.allocate_ui_with_layout(vec2(col_w, 42.0), Layout::top_down(egui::Align::Center), |ui| {
                                ui.label(RichText::new("PEAK").size(9.5).color(theme.text_muted).strong());
                                let peak_str = if player.visualizer.peak_dbfs <= -55.0 {
                                    "-∞ dBFS".to_string()
                                } else {
                                    format!("{:.1} dBFS", player.visualizer.peak_dbfs)
                                };
                                let peak_col = if player.visualizer.peak_dbfs > -1.0 {
                                    theme.accent_warm
                                } else {
                                    theme.text_primary
                                };
                                ui.label(RichText::new(peak_str).size(12.5).color(peak_col).strong());
                            });

                            // Metric 2: LUFS
                            ui.allocate_ui_with_layout(vec2(col_w, 42.0), Layout::top_down(egui::Align::Center), |ui| {
                                ui.label(RichText::new("LUFS").size(9.5).color(theme.text_muted).strong());
                                let lufs_str = if player.visualizer.lufs <= -55.0 {
                                    "-∞".to_string()
                                } else {
                                    format!("{:.1}", player.visualizer.lufs)
                                };
                                ui.label(RichText::new(lufs_str).size(12.5).color(theme.text_primary).strong());
                            });

                            // Metric 3: Dynamic Range
                            ui.allocate_ui_with_layout(vec2(col_w, 42.0), Layout::top_down(egui::Align::Center), |ui| {
                                ui.label(RichText::new("DYN RANGE").size(9.5).color(theme.text_muted).strong());
                                let dr_str = if player.visualizer.dynamic_range < 1.0 {
                                    "--".to_string()
                                } else {
                                    format!("DR {:.0}", player.visualizer.dynamic_range)
                                };
                                ui.label(RichText::new(dr_str).size(12.5).color(theme.accent).strong());
                            });
                        });
                    });

                    ui.add_space(8.0);

                    // 4. Live L/R Stereo Volume Level Meters
                    let meter_card = theme.card_frame(8.0);
                    meter_card.show(ui, |ui| {
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("STEREO METERS").size(10.0).color(theme.text_muted).strong());
                                ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.label(RichText::new("0 dB").size(9.0).color(theme.text_muted));
                                });
                            });

                            ui.add_space(4.0);
                            render_channel_meter(ui, "L", player.visualizer.left_level, player.visualizer.left_peak, theme);
                            ui.add_space(3.0);
                            render_channel_meter(ui, "R", player.visualizer.right_level, player.visualizer.right_peak, theme);
                        });
                    });

                    ui.add_space(8.0);

                    // 5. 20Hz - 20kHz Responsive Audio Spectrum Visualizer
                    let spec_card = theme.card_frame(8.0);
                    spec_card.show(ui, |ui| {
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("SPECTRUM").size(10.0).color(theme.text_muted).strong());
                                ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.selectable_label(player.visualizer.style == VisualizerStyle::Scope, "Scope").clicked() {
                                        player.visualizer.style = VisualizerStyle::Scope;
                                    }
                                    if ui.selectable_label(player.visualizer.style == VisualizerStyle::Mirror, "Mirror").clicked() {
                                        player.visualizer.style = VisualizerStyle::Mirror;
                                    }
                                    if ui.selectable_label(player.visualizer.style == VisualizerStyle::Bars, "Bars").clicked() {
                                        player.visualizer.style = VisualizerStyle::Bars;
                                    }
                                });
                            });

                            ui.add_space(4.0);

                            // Responsive Spectrum Visualizer Canvas
                            let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 76.0), egui::Sense::hover());
                            render_spectrum(
                                ui.painter(),
                                rect,
                                &player.visualizer.band_values,
                                &player.visualizer.peak_values,
                                &player.visualizer.wave_buffer,
                                player.visualizer.style,
                                theme.accent,
                                theme,
                            );

                            ui.add_space(4.0);

                            // Frequency Axis Markers (20Hz - 20kHz) proportionally spaced across spectrum width
                            let (axis_rect, _) = ui.allocate_exact_size(vec2(rect.width(), 16.0), egui::Sense::hover());
                            let painter = ui.painter();

                            let freq_markers: [(&str, f32); 7] = [
                                ("20Hz", 0.00),
                                ("100Hz", 0.17),
                                ("500Hz", 0.36),
                                ("1kHz", 0.52),
                                ("5kHz", 0.70),
                                ("10kHz", 0.86),
                                ("20kHz", 1.00),
                            ];

                            for &(lbl, fraction) in &freq_markers {
                                let x = axis_rect.min.x + fraction * axis_rect.width();

                                // Subtle tick mark connecting spectrum to frequency label
                                painter.line_segment(
                                    [pos2(x, axis_rect.min.y), pos2(x, axis_rect.min.y + 3.0)],
                                    Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(180, 140, 255, 60)),
                                );

                                let align = if fraction <= 0.01 {
                                    egui::Align2::LEFT_TOP
                                } else if fraction >= 0.99 {
                                    egui::Align2::RIGHT_TOP
                                } else {
                                    egui::Align2::CENTER_TOP
                                };

                                painter.text(
                                    pos2(x, axis_rect.min.y + 4.0),
                                    align,
                                    lbl,
                                    egui::FontId::proportional(9.0),
                                    theme.text_muted,
                                );
                            }
                        });
                    });
                });
            });
        });
    } else {
        ui.centered_and_justified(|ui| {
            ui.vertical_centered(|ui| {
                let (icon_rect, _) = ui.allocate_exact_size(vec2(40.0, 40.0), egui::Sense::hover());
                paint_icon(ui.painter(), Icon::Disc, icon_rect, theme.text_muted);
                ui.add_space(8.0);
                ui.label(RichText::new("Nothing Playing").size(20.0).color(theme.text_muted));
                ui.add_space(8.0);
                ui.label(RichText::new("Select a song from Songs or Home to start listening.").color(theme.text_secondary));
            });
        });
    }
}

fn stat_row(ui: &mut Ui, label: &str, val: &str, theme: &Theme) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(label).size(11.0).color(theme.text_muted));
        ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(RichText::new(val).size(11.5).color(theme.text_primary).strong());
        });
    });
}

fn render_channel_meter(
    ui: &mut Ui,
    channel_lbl: &str,
    level: f32,
    peak: f32,
    theme: &Theme,
) {
    ui.horizontal(|ui| {
        ui.allocate_ui_with_layout(vec2(14.0, 14.0), Layout::left_to_right(egui::Align::Center), |ui| {
            ui.label(RichText::new(channel_lbl).size(10.0).color(theme.text_secondary).strong());
        });

        let meter_width = ui.available_width() - 52.0;
        let (rect, _) = ui.allocate_exact_size(vec2(meter_width, 8.0), egui::Sense::hover());

        // Background track
        ui.painter().rect_filled(
            rect,
            Rounding::same(2.0),
            Color32::from_rgba_premultiplied(180, 140, 255, 20),
        );

        // Filled level bar
        let clamped_level = level.clamp(0.0, 1.0);
        if clamped_level > 0.01 {
            let fill_w = (rect.width() * clamped_level).max(2.0);
            let fill_rect = Rect::from_min_size(rect.min, vec2(fill_w, rect.height()));

            let bar_color = if clamped_level > 0.88 {
                theme.accent_warm
            } else {
                theme.accent
            };

            ui.painter().rect_filled(fill_rect, Rounding::same(2.0), bar_color);
        }

        // Peak tick
        let clamped_peak = peak.clamp(0.0, 1.0);
        if clamped_peak > 0.02 {
            let peak_x = rect.min.x + rect.width() * clamped_peak;
            let peak_rect = Rect::from_min_size(pos2((peak_x - 1.5).max(rect.min.x), rect.min.y), vec2(2.0, rect.height()));
            let peak_color = if clamped_peak > 0.95 {
                Color32::from_rgb(239, 68, 68) // Red clip warning
            } else {
                theme.accent_warm
            };
            ui.painter().rect_filled(peak_rect, Rounding::same(1.0), peak_color);
        }

        // dB value display
        let db_val = if level > 0.001 {
            format!("{:.1} dB", (20.0 * level.log10()).clamp(-60.0, 0.0))
        } else {
            "-∞ dB".to_string()
        };
        ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(RichText::new(db_val).size(9.5).color(theme.text_muted));
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppConfig;
    use crate::library::model::Track;

    #[test]
    fn test_now_playing_rendering_with_frequency_axis() {
        let ctx = egui::Context::default();
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let theme = Theme::from_name("Eclipse", None);
                let cfg = AppConfig::default();
                let mut player = PlayerController::new(&cfg);
                let mut art_manager = AlbumArtManager::new();

                // Empty state test
                render_now_playing(ui, &mut player, &theme, &mut art_manager);

                // Playing track test
                let track = Track {
                    id: 1,
                    path: "hires.flac".into(),
                    title: "Hi-Res Master".into(),
                    artist: "Audiophile Artist".into(),
                    album: "Master Recording".into(),
                    genre: "Classical".into(),
                    year: 2024,
                    track_number: 1,
                    duration_secs: 320.0,
                    format: "FLAC".into(),
                    bitrate_kbps: 4608,
                    sample_rate: 96000,
                    channels: 2,
                    favorite: true,
                    has_art: false,
                    folder: "".into(),
                    album_artist: "".into(),
                    play_count: 7,
                    file_size: 8192,
                };
                player.load_queue(vec![track], 0);

                // Update visualizer values
                player.visualizer.band_values = vec![0.5; 32];
                player.visualizer.peak_values = vec![0.8; 32];
                player.visualizer.wave_buffer = vec![0.0; 128];
                player.visualizer.peak_dbfs = -3.2;
                player.visualizer.lufs = -14.1;
                player.visualizer.dynamic_range = 12.0;

                render_now_playing(ui, &mut player, &theme, &mut art_manager);
            });
        });
    }
}
