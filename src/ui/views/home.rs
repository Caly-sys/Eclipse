use crate::audio::PlayerController;
use crate::library::model::Track;
use crate::ui::art::AlbumArtManager;
use crate::ui::components::{format_badge, pill_badge};
use crate::ui::icons::{paint_icon, Icon};
use crate::ui::theme::Theme;
use egui::{pos2, vec2, Color32, Layout, Rect, RichText, Rounding, ScrollArea, Stroke, Ui};
use std::collections::HashSet;

#[allow(clippy::too_many_arguments)]
pub fn render_home(
    ui: &mut Ui,
    tracks: &[Track],
    recent_history: &[Track],
    player: &mut PlayerController,
    theme: &Theme,
    on_play_track: &mut Option<Track>,
    art_manager: &mut AlbumArtManager,
    on_trigger_scan: &mut bool,
) {
    ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing = vec2(0.0, 18.0);

            // =================================================================
            // 1. HERO HEADER BANNER WITH QUICK ACTIONS
            // =================================================================
            let mut hero_frame = theme.glass_frame(14.0);
            hero_frame.inner_margin = egui::Margin::symmetric(24.0, 20.0);

            hero_frame.show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(
                            RichText::new("ECLIPSE MUSIC")
                                .size(24.0)
                                .color(theme.text_primary)
                                .strong(),
                        );
                        ui.add_space(3.0);
                        ui.label(
                            RichText::new("A calm, local-first lossless audio player designed for deep listening.")
                                .size(13.0)
                                .color(theme.text_secondary),
                        );
                        ui.add_space(14.0);

                        // Quick Actions
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing = vec2(10.0, 0.0);

                            // Quick Action 1: Play All
                            let play_btn = egui::Button::new(
                                RichText::new("Play All")
                                    .color(Color32::from_rgb(11, 9, 20))
                                    .size(13.0)
                                    .strong(),
                            )
                            .fill(theme.accent)
                            .rounding(Rounding::same(8.0))
                            .min_size(vec2(110.0, 34.0));

                            if ui.add(play_btn).clicked() && !tracks.is_empty() {
                                player.shuffle = false;
                                player.load_queue(tracks.to_vec(), 0);
                            }

                            // Quick Action 2: Shuffle Library
                            let shuffle_btn = egui::Button::new(
                                RichText::new("Shuffle Library")
                                    .color(theme.text_primary)
                                    .size(13.0)
                                    .strong(),
                            )
                            .fill(theme.card)
                            .stroke(Stroke::new(1.0_f32, theme.border_accent))
                            .rounding(Rounding::same(8.0))
                            .min_size(vec2(130.0, 34.0));

                            if ui.add(shuffle_btn).clicked() && !tracks.is_empty() {
                                player.shuffle = true;
                                player.load_queue(tracks.to_vec(), 0);
                            }

                            // Quick Action 3: Scan Folder
                            let scan_btn = egui::Button::new(
                                RichText::new("Scan Folder")
                                    .color(theme.text_secondary)
                                    .size(13.0)
                                    .strong(),
                            )
                            .fill(theme.card)
                            .stroke(Stroke::new(1.0_f32, theme.border))
                            .rounding(Rounding::same(8.0))
                            .min_size(vec2(110.0, 34.0));

                            if ui.add(scan_btn).clicked() {
                                *on_trigger_scan = true;
                            }
                        });
                    });

                    // Ambient Right Badge
                    ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                        let (icon_box, _) = ui.allocate_exact_size(vec2(64.0, 64.0), egui::Sense::hover());
                        ui.painter().circle_filled(
                            icon_box.center(),
                            30.0,
                            Color32::from_rgba_premultiplied(theme.accent.r(), theme.accent.g(), theme.accent.b(), 25),
                        );
                        ui.painter().circle_stroke(
                            icon_box.center(),
                            30.0,
                            Stroke::new(1.0_f32, theme.border_accent),
                        );
                        let inner_icon = Rect::from_center_size(icon_box.center(), vec2(28.0, 28.0));
                        paint_icon(ui.painter(), Icon::Headphones, inner_icon, theme.accent);
                    });
                });
            });

            // =================================================================
            // 2. 4-CARD STATS GRID: Tracks, Albums, Artists, Lossless / Playtime
            // =================================================================
            let total_tracks = tracks.len();
            let total_albums = tracks
                .iter()
                .map(|t| {
                    if t.album.trim().is_empty() {
                        "Unknown Album"
                    } else {
                        t.album.as_str()
                    }
                })
                .collect::<HashSet<_>>()
                .len();

            let total_artists = tracks
                .iter()
                .map(|t| t.display_artist())
                .collect::<HashSet<_>>()
                .len();

            let lossless_count = tracks.iter().filter(|t| t.is_lossless()).count();
            let total_secs: f64 = tracks.iter().map(|t| t.duration_secs).sum();
            let total_hours = total_secs / 3600.0;
            let playtime_str = if total_hours >= 1.0 {
                format!("{:.1} hrs", total_hours)
            } else {
                format!("{} mins", (total_secs / 60.0).round() as u32)
            };

            let spacing = 12.0_f32;
            let available_w = ui.available_width();
            let card_w = ((available_w - 3.0 * spacing) / 4.0).max(110.0);
            let card_h = 76.0_f32;

            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = vec2(spacing, 0.0);

                // Card 1: Total Tracks
                render_stat_card(
                    ui,
                    card_w,
                    card_h,
                    "TRACKS",
                    &format!("{}", total_tracks),
                    "Library catalog",
                    Icon::Music,
                    theme.text_primary,
                    theme,
                );

                // Card 2: Total Albums
                render_stat_card(
                    ui,
                    card_w,
                    card_h,
                    "ALBUMS",
                    &format!("{}", total_albums),
                    "Curated collection",
                    Icon::Disc,
                    theme.text_primary,
                    theme,
                );

                // Card 3: Total Artists
                render_stat_card(
                    ui,
                    card_w,
                    card_h,
                    "ARTISTS",
                    &format!("{}", total_artists),
                    "Distinct creators",
                    Icon::Mic,
                    theme.text_primary,
                    theme,
                );

                // Card 4: Lossless Count / Playtime
                render_stat_card(
                    ui,
                    card_w,
                    card_h,
                    "LOSSLESS / DURATION",
                    &format!("{} · {}", lossless_count, playtime_str),
                    "Hi-Fi listening",
                    Icon::Sparkle,
                    theme.accent_warm,
                    theme,
                );
            });

            // =================================================================
            // 3. RECENTLY PLAYED HORIZONTAL SHELF / GRID
            // =================================================================
            if !recent_history.is_empty() {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("RECENTLY PLAYED")
                            .size(13.5)
                            .color(theme.text_secondary)
                            .strong(),
                    );
                    ui.label(
                        RichText::new("— Jump back into your listening history")
                            .size(11.5)
                            .color(theme.text_muted),
                    );
                });

                // Auto-wrapping responsive cards for recently played
                let shelf_card_w = 210.0_f32;
                let cols = ((ui.available_width() + 10.0) / (shelf_card_w + 10.0)).floor().max(1.0) as usize;
                let actual_w = ((ui.available_width() - (cols.saturating_sub(1) as f32 * 10.0)) / cols as f32).max(160.0);

                let recent_items: Vec<&Track> = recent_history.iter().take(8).collect();
                for chunk in recent_items.chunks(cols) {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing = vec2(10.0, 0.0);

                        for track in chunk {
                            let mut card_frame = theme.card_frame(8.0);
                            card_frame.inner_margin = egui::Margin::symmetric(10.0, 8.0);

                            let resp = card_frame.show(ui, |ui| {
                                ui.allocate_ui_with_layout(
                                    vec2(actual_w - 20.0, 44.0),
                                    Layout::left_to_right(egui::Align::Center),
                                    |ui| {
                                        // 40x40 Album Art Thumbnail
                                        let (art_box, _) = ui.allocate_exact_size(vec2(40.0, 40.0), egui::Sense::hover());
                                        let mut painted = false;
                                        if let Some(tex) = art_manager.get_or_load(ui.ctx(), &track.path) {
                                            ui.painter().image(
                                                tex.id(),
                                                art_box,
                                                Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                                                Color32::WHITE,
                                            );
                                            ui.painter().rect_stroke(art_box, Rounding::same(6.0_f32), Stroke::new(1.0_f32, theme.border_accent));
                                            painted = true;
                                        }
                                        if !painted {
                                            ui.painter().rect_filled(art_box, Rounding::same(6.0_f32), theme.surface);
                                            ui.painter().rect_stroke(art_box, Rounding::same(6.0_f32), Stroke::new(1.0_f32, theme.border));
                                            let icon_box = Rect::from_center_size(art_box.center(), vec2(18.0, 18.0));
                                            paint_icon(ui.painter(), Icon::Disc, icon_box, theme.accent);
                                        }

                                        ui.add_space(8.0);

                                        // Title & Artist
                                        ui.vertical(|ui| {
                                            let char_limit = ((actual_w - 70.0) / 7.5).max(8.0) as usize;
                                            ui.label(
                                                RichText::new(truncate_str(track.display_title(), char_limit))
                                                    .color(theme.text_primary)
                                                    .size(12.5)
                                                    .strong(),
                                            );
                                            ui.horizontal(|ui| {
                                                ui.label(
                                                    RichText::new(truncate_str(track.display_artist(), char_limit + 2))
                                                        .color(theme.text_secondary)
                                                        .size(11.0),
                                                );
                                                format_badge(ui, &track.format, track.is_lossless(), theme);
                                            });
                                        });
                                    },
                                );
                            });

                            let interact = resp.response.interact(egui::Sense::click());
                            if interact.hovered() {
                                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                            }
                            if interact.clicked() {
                                *on_play_track = Some((*track).clone());
                            }
                        }
                    });
                }
            }

            // =================================================================
            // 4. RECENTLY ADDED & MOST PLAYED SECTIONS (2 COLUMNS)
            // =================================================================
            ui.columns(2, |cols| {
                // Left Column: Recently Added
                cols[0].vertical(|ui| {
                    ui.label(
                        RichText::new("RECENTLY ADDED")
                            .size(13.0)
                            .color(theme.text_secondary)
                            .strong(),
                    );
                    ui.add_space(4.0);

                    let mut sorted_recent = tracks.to_vec();
                    sorted_recent.sort_by_key(|t| std::cmp::Reverse(t.id));

                    for track in sorted_recent.iter().take(5) {
                        render_compact_track_row(ui, track, theme, on_play_track, art_manager);
                    }
                });

                // Right Column: Most Played
                cols[1].vertical(|ui| {
                    ui.label(
                        RichText::new("MOST PLAYED")
                            .size(13.0)
                            .color(theme.text_secondary)
                            .strong(),
                    );
                    ui.add_space(4.0);

                    let mut sorted_most = tracks.to_vec();
                    sorted_most.sort_by_key(|t| std::cmp::Reverse(t.play_count));

                    for (rank, track) in sorted_most.iter().take(5).enumerate() {
                        render_ranked_track_row(ui, rank + 1, track, theme, on_play_track, art_manager);
                    }
                });
            });

            // =================================================================
            // 5. FAVORITES QUICK ACCESS GRID
            // =================================================================
            let favorites: Vec<&Track> = tracks.iter().filter(|t| t.favorite).collect();
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("FAVORITES")
                        .size(13.5)
                        .color(theme.text_secondary)
                        .strong(),
                );
                ui.label(
                    RichText::new(format!("— {} pinned songs", favorites.len()))
                        .size(11.5)
                        .color(theme.text_muted),
                );
            });

            if favorites.is_empty() {
                let mut empty_frame = theme.card_frame(8.0);
                empty_frame.inner_margin = egui::Margin::symmetric(16.0, 14.0);
                empty_frame.show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let (icon_box, _) = ui.allocate_exact_size(vec2(20.0, 20.0), egui::Sense::hover());
                        paint_icon(ui.painter(), Icon::HeartOutline, icon_box, theme.text_muted);
                        ui.add_space(6.0);
                        ui.label(
                            RichText::new("No favorites yet. Click the heart icon on any track or player bar to pin favorites here.")
                                .size(12.0)
                                .color(theme.text_muted),
                        );
                    });
                });
            } else {
                let fav_card_w = 200.0_f32;
                let cols = ((ui.available_width() + 10.0) / (fav_card_w + 10.0)).floor().max(1.0) as usize;
                let actual_w = ((ui.available_width() - (cols.saturating_sub(1) as f32 * 10.0)) / cols as f32).max(150.0);

                for chunk in favorites.chunks(cols) {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing = vec2(10.0, 0.0);

                        for track in chunk {
                            let mut card_frame = theme.card_frame(8.0);
                            card_frame.inner_margin = egui::Margin::symmetric(10.0, 8.0);

                            let resp = card_frame.show(ui, |ui| {
                                ui.allocate_ui_with_layout(
                                    vec2(actual_w - 20.0, 44.0),
                                    Layout::left_to_right(egui::Align::Center),
                                    |ui| {
                                        let (art_box, _) = ui.allocate_exact_size(vec2(38.0, 38.0), egui::Sense::hover());
                                        let mut painted = false;
                                        if let Some(tex) = art_manager.get_or_load(ui.ctx(), &track.path) {
                                            ui.painter().image(
                                                tex.id(),
                                                art_box,
                                                Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                                                Color32::WHITE,
                                            );
                                            ui.painter().rect_stroke(art_box, Rounding::same(6.0_f32), Stroke::new(1.0_f32, theme.border_accent));
                                            painted = true;
                                        }
                                        if !painted {
                                            ui.painter().rect_filled(art_box, Rounding::same(6.0_f32), theme.surface);
                                            ui.painter().rect_stroke(art_box, Rounding::same(6.0_f32), Stroke::new(1.0_f32, theme.border));
                                            let icon_box = Rect::from_center_size(art_box.center(), vec2(18.0, 18.0));
                                            paint_icon(ui.painter(), Icon::Disc, icon_box, theme.accent);
                                        }

                                        ui.add_space(8.0);

                                        ui.vertical(|ui| {
                                            let char_limit = ((actual_w - 74.0) / 7.5).max(8.0) as usize;
                                            ui.label(
                                                RichText::new(truncate_str(track.display_title(), char_limit))
                                                    .color(theme.text_primary)
                                                    .size(12.5)
                                                    .strong(),
                                            );
                                            ui.horizontal(|ui| {
                                                ui.label(
                                                    RichText::new(truncate_str(track.display_artist(), char_limit + 2))
                                                        .color(theme.text_secondary)
                                                        .size(11.0),
                                                );
                                                format_badge(ui, &track.format, track.is_lossless(), theme);
                                            });
                                        });

                                        ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                                            let (heart_box, _) = ui.allocate_exact_size(vec2(16.0, 16.0), egui::Sense::hover());
                                            paint_icon(ui.painter(), Icon::Heart, heart_box, theme.accent);
                                        });
                                    },
                                );
                            });

                            let interact = resp.response.interact(egui::Sense::click());
                            if interact.hovered() {
                                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                            }
                            if interact.clicked() {
                                let fav_list: Vec<Track> = favorites.iter().map(|&t| t.clone()).collect();
                                let idx = fav_list.iter().position(|t| t.id == track.id).unwrap_or(0);
                                player.load_queue(fav_list, idx);
                            }
                        }
                    });
                }
            }
        });
}

#[allow(clippy::too_many_arguments)]
fn render_stat_card(
    ui: &mut Ui,
    w: f32,
    h: f32,
    category: &str,
    val: &str,
    sub: &str,
    icon: Icon,
    val_color: Color32,
    theme: &Theme,
) {
    let mut card = theme.card_frame(8.0);
    card.inner_margin = egui::Margin::symmetric(12.0, 10.0);

    card.show(ui, |ui| {
        let (rect, resp) = ui.allocate_exact_size(vec2(w - 24.0, h - 20.0), egui::Sense::hover());
        if resp.hovered() {
            ui.painter().rect_stroke(rect.expand(8.0), Rounding::same(8.0), Stroke::new(1.0_f32, theme.border_accent));
        }

        // Category label
        ui.painter().text(
            rect.min,
            egui::Align2::LEFT_TOP,
            category,
            egui::FontId::proportional(9.5),
            theme.text_muted,
        );

        // Value text
        ui.painter().text(
            pos2(rect.min.x, rect.min.y + 17.0),
            egui::Align2::LEFT_TOP,
            val,
            egui::FontId::proportional(16.0),
            val_color,
        );

        // Subtext
        ui.painter().text(
            pos2(rect.min.x, rect.min.y + 39.0),
            egui::Align2::LEFT_TOP,
            sub,
            egui::FontId::proportional(10.5),
            theme.text_muted,
        );

        // Top-right subtle vector icon
        let icon_box = Rect::from_min_size(pos2(rect.max.x - 18.0, rect.min.y), vec2(18.0, 18.0));
        paint_icon(
            ui.painter(),
            icon,
            icon_box,
            Color32::from_rgba_premultiplied(val_color.r(), val_color.g(), val_color.b(), 120),
        );
    });
}

fn render_compact_track_row(
    ui: &mut Ui,
    track: &Track,
    theme: &Theme,
    on_play_track: &mut Option<Track>,
    art_manager: &mut AlbumArtManager,
) {
    let mut frame = theme.glass_frame(6.0);
    frame.inner_margin = egui::Margin::symmetric(8.0, 5.0);

    let resp = frame.show(ui, |ui| {
        ui.horizontal(|ui| {
            let (art_box, _) = ui.allocate_exact_size(vec2(28.0, 28.0), egui::Sense::hover());
            let mut painted = false;
            if let Some(tex) = art_manager.get_or_load(ui.ctx(), &track.path) {
                ui.painter().image(
                    tex.id(),
                    art_box,
                    Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                    Color32::WHITE,
                );
                painted = true;
            }
            if !painted {
                ui.painter().rect_filled(art_box, Rounding::same(4.0), theme.surface);
                let icon_box = Rect::from_center_size(art_box.center(), vec2(14.0, 14.0));
                paint_icon(ui.painter(), Icon::Music, icon_box, theme.accent);
            }

            ui.add_space(6.0);
            ui.vertical(|ui| {
                ui.label(
                    RichText::new(truncate_str(track.display_title(), 22))
                        .color(theme.text_primary)
                        .size(12.0)
                        .strong(),
                );
                ui.label(
                    RichText::new(truncate_str(track.display_artist(), 24))
                        .color(theme.text_secondary)
                        .size(10.5),
                );
            });

            ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(RichText::new(track.formatted_duration()).size(11.0).color(theme.text_muted));
                ui.add_space(6.0);
                format_badge(ui, &track.format, track.is_lossless(), theme);
            });
        });
    });

    let interact = resp.response.interact(egui::Sense::click());
    if interact.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    if interact.clicked() {
        *on_play_track = Some(track.clone());
    }
}

fn render_ranked_track_row(
    ui: &mut Ui,
    rank: usize,
    track: &Track,
    theme: &Theme,
    on_play_track: &mut Option<Track>,
    art_manager: &mut AlbumArtManager,
) {
    let mut frame = theme.glass_frame(6.0);
    frame.inner_margin = egui::Margin::symmetric(8.0, 5.0);

    let resp = frame.show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(format!("{}", rank))
                    .size(11.5)
                    .color(if rank <= 3 { theme.accent } else { theme.text_muted })
                    .strong(),
            );
            ui.add_space(4.0);

            let (art_box, _) = ui.allocate_exact_size(vec2(28.0, 28.0), egui::Sense::hover());
            let mut painted = false;
            if let Some(tex) = art_manager.get_or_load(ui.ctx(), &track.path) {
                ui.painter().image(
                    tex.id(),
                    art_box,
                    Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                    Color32::WHITE,
                );
                painted = true;
            }
            if !painted {
                ui.painter().rect_filled(art_box, Rounding::same(4.0), theme.surface);
                let icon_box = Rect::from_center_size(art_box.center(), vec2(14.0, 14.0));
                paint_icon(ui.painter(), Icon::Disc, icon_box, theme.accent);
            }

            ui.add_space(6.0);
            ui.vertical(|ui| {
                ui.label(
                    RichText::new(truncate_str(track.display_title(), 20))
                        .color(theme.text_primary)
                        .size(12.0)
                        .strong(),
                );
                ui.label(
                    RichText::new(truncate_str(track.display_artist(), 22))
                        .color(theme.text_secondary)
                        .size(10.5),
                );
            });

            ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                if track.play_count > 0 {
                    pill_badge(ui, &format!("{} plays", track.play_count), theme.card, theme.text_secondary);
                } else {
                    ui.label(RichText::new(track.formatted_duration()).size(11.0).color(theme.text_muted));
                }
            });
        });
    });

    let interact = resp.response.interact(egui::Sense::click());
    if interact.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    if interact.clicked() {
        *on_play_track = Some(track.clone());
    }
}

fn truncate_str(s: &str, max_chars: usize) -> String {
    if s.chars().count() > max_chars {
        let truncated: String = s.chars().take(max_chars.saturating_sub(1)).collect();
        format!("{truncated}…")
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppConfig;

    #[test]
    fn test_home_view_rendering() {
        let ctx = egui::Context::default();
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let theme = Theme::from_name("Eclipse", None);
                let cfg = AppConfig::default();
                let mut player = PlayerController::new(&cfg);
                let mut art_manager = AlbumArtManager::new();
                let mut on_play = None;
                let mut on_scan = false;

                let tracks = vec![
                    Track {
                        id: 1,
                        path: "song.flac".into(),
                        title: "Home Track".into(),
                        artist: "Home Artist".into(),
                        album: "Home Album".into(),
                        genre: "Ambient".into(),
                        year: 2024,
                        track_number: 1,
                        duration_secs: 240.0,
                        format: "FLAC".into(),
                        bitrate_kbps: 1411,
                        sample_rate: 48000,
                        channels: 2,
                        favorite: true,
                        has_art: false,
                        folder: "".into(),
                        album_artist: "".into(),
                        play_count: 12,
                        file_size: 4096,
                    },
                ];

                render_home(
                    ui,
                    &tracks,
                    &tracks,
                    &mut player,
                    &theme,
                    &mut on_play,
                    &mut art_manager,
                    &mut on_scan,
                );
            });
        });
    }
}
