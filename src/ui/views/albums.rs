use crate::audio::PlayerController;
use crate::library::model::Track;
use crate::ui::art::AlbumArtManager;
use crate::ui::icons::{paint_icon, Icon};
use crate::ui::theme::Theme;
use egui::{pos2, vec2, Color32, Layout, Rect, RichText, Rounding, ScrollArea, Stroke, Ui};
use std::collections::BTreeMap;

#[derive(Default)]
pub struct AlbumsViewState {
    pub selected_album: Option<String>,
}

pub fn render_albums(
    ui: &mut Ui,
    tracks: &[Track],
    search_query: &str,
    state: &mut AlbumsViewState,
    player: &mut PlayerController,
    theme: &Theme,
    art_manager: &mut AlbumArtManager,
) {
    // Group tracks by album
    let mut all_albums: BTreeMap<String, Vec<&Track>> = BTreeMap::new();
    for track in tracks {
        let key = if track.album.trim().is_empty() {
            "Unknown Album".to_string()
        } else {
            track.album.clone()
        };
        all_albums.entry(key).or_default().push(track);
    }

    let query = search_query.trim().to_lowercase();
    let albums: BTreeMap<String, Vec<&Track>> = all_albums
        .into_iter()
        .filter(|(album_name, a_tracks)| {
            if query.is_empty() {
                true
            } else {
                album_name.to_lowercase().contains(&query)
                    || a_tracks.iter().any(|t| t.display_artist().to_lowercase().contains(&query))
            }
        })
        .collect();

    // If an album is selected, show Album Detail View
    let selected_album_name = state.selected_album.clone();
    if let Some(selected) = selected_album_name {
        if let Some(album_tracks) = albums.get(&selected) {
            let mut back_clicked = false;
            ui.horizontal(|ui| {
                if ui.button(RichText::new("Back to Albums").color(theme.accent).size(13.0)).clicked() {
                    back_clicked = true;
                }
            });

            if back_clicked {
                state.selected_album = None;
                return;
            }

            ui.add_space(10.0);

            let artist = album_tracks.first().map(|t| t.display_artist()).unwrap_or("Unknown Artist");
            let year = album_tracks.first().map(|t| t.year).unwrap_or(0);

            // Album Header Card
            let mut header_frame = theme.glass_frame(12.0);
            header_frame.inner_margin = egui::Margin::same(16.0_f32);

            header_frame.show(ui, |ui| {
                ui.horizontal(|ui| {
                    let (rect, _) = ui.allocate_exact_size(vec2(100.0, 100.0), egui::Sense::hover());
                    let mut painted = false;
                    if let Some(first) = album_tracks.first() {
                        if let Some(tex) = art_manager.get_or_load(ui.ctx(), &first.path) {
                            ui.painter().image(
                                tex.id(),
                                rect,
                                Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                                Color32::WHITE,
                            );
                            ui.painter().rect_stroke(rect, Rounding::same(10.0_f32), Stroke::new(1.0_f32, theme.border_accent));
                            painted = true;
                        }
                    }

                    if !painted {
                        ui.painter().rect_filled(rect, Rounding::same(10.0_f32), theme.card);
                        ui.painter().rect_stroke(rect, Rounding::same(10.0_f32), Stroke::new(1.0_f32, theme.border));
                        let icon_box = Rect::from_center_size(rect.center(), vec2(42.0, 42.0));
                        paint_icon(ui.painter(), Icon::Disc, icon_box, theme.accent);
                    }

                    ui.add_space(14.0);

                    ui.vertical(|ui| {
                        ui.label(RichText::new(&selected).size(20.0).color(theme.text_primary).strong());
                        ui.label(RichText::new(artist).size(14.0).color(theme.text_secondary));
                        if year > 0 {
                            ui.label(RichText::new(format!("{} · {} Tracks", year, album_tracks.len())).color(theme.text_muted).size(12.0));
                        } else {
                            ui.label(RichText::new(format!("{} Tracks", album_tracks.len())).color(theme.text_muted).size(12.0));
                        }

                        ui.add_space(8.0);
                        let play_btn = egui::Button::new(RichText::new("Play Album").color(Color32::from_rgb(11, 9, 20)).strong())
                            .fill(theme.accent)
                            .rounding(Rounding::same(8.0_f32));
                        if ui.add(play_btn).clicked() {
                            let list: Vec<Track> = album_tracks.iter().map(|&t| t.clone()).collect();
                            player.load_queue(list, 0);
                        }
                    });
                });
            });

            ui.add_space(12.0);

            // Album Tracks List
            ScrollArea::vertical()
                .id_salt("album_detail_scroll")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    for (i, t) in album_tracks.iter().enumerate() {
                        let row = egui::Frame::none()
                            .fill(theme.card)
                            .stroke(Stroke::new(1.0_f32, theme.border))
                            .rounding(Rounding::same(6.0_f32))
                            .inner_margin(egui::Margin::symmetric(12.0_f32, 8.0_f32));

                        let resp = row.show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(format!("{}", i + 1)).size(12.0).color(theme.text_muted));
                                ui.add_space(8.0);
                                ui.label(RichText::new(t.display_title()).size(13.0).color(theme.text_primary).strong());
                                ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.label(RichText::new(t.formatted_duration()).size(12.0).color(theme.text_muted));
                                });
                            });
                        });

                        if resp.response.interact(egui::Sense::click()).clicked() {
                            let list: Vec<Track> = album_tracks.iter().map(|&track| track.clone()).collect();
                            player.load_queue(list, i);
                        }
                    }
                });
            return;
        }
    }

    // Auto-wrapping responsive grid of Albums
    ScrollArea::vertical()
        .id_salt("albums_grid_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let spacing = 14.0_f32;
            let min_card_w = 165.0_f32;
            let available_w = ui.available_width();
            let cols = ((available_w + spacing) / (min_card_w + spacing)).floor().max(1.0) as usize;
            let card_w = ((available_w - (cols.saturating_sub(1) as f32 * spacing)) / cols as f32).max(min_card_w * 0.75);
            let art_size = card_w; // 1:1 square aspect ratio
            let card_h = art_size + 62.0;

            let album_list: Vec<(&String, &Vec<&Track>)> = albums.iter().collect();

            ui.spacing_mut().item_spacing = vec2(spacing, spacing);

            for chunk in album_list.chunks(cols) {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = vec2(spacing, 0.0);

                    for (album_name, album_tracks) in chunk {
                        let artist = album_tracks.first().map(|t| t.display_artist()).unwrap_or("Unknown Artist");
                        let mut card = theme.card_frame(10.0);
                        card.inner_margin = egui::Margin::same(0.0);

                        card.show(ui, |ui| {
                            let (rect, resp) = ui.allocate_exact_size(vec2(card_w, card_h), egui::Sense::click());
                            if resp.hovered() {
                                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                                ui.painter().rect_stroke(rect, Rounding::same(10.0_f32), Stroke::new(1.0_f32, theme.border_accent));
                            }

                            // 1:1 square aspect ratio cover art with rounded corners
                            let art_rect = Rect::from_min_size(rect.min, vec2(card_w, art_size));
                            let mut painted = false;
                            if let Some(first) = album_tracks.first() {
                                if let Some(tex) = art_manager.get_or_load(ui.ctx(), &first.path) {
                                    ui.painter().image(
                                        tex.id(),
                                        art_rect,
                                        Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                                        Color32::WHITE,
                                    );
                                    ui.painter().rect_stroke(art_rect, Rounding::same(8.0_f32), Stroke::new(1.0_f32, theme.border_accent));
                                    painted = true;
                                }
                            }

                            if !painted {
                                ui.painter().rect_filled(art_rect, Rounding::same(8.0_f32), theme.surface);
                                ui.painter().rect_stroke(art_rect, Rounding::same(8.0_f32), Stroke::new(1.0_f32, theme.border));
                                let icon_box = Rect::from_center_size(art_rect.center(), vec2(art_size * 0.38, art_size * 0.38));
                                paint_icon(ui.painter(), Icon::Disc, icon_box, theme.accent);
                            }

                            // Title & Artist with truncation to prevent cross-card overlap
                            let char_limit = ((card_w - 16.0) / 7.5).max(10.0) as usize;
                            let display_title = truncate_str(album_name, char_limit);
                            let display_artist = truncate_str(artist, char_limit + 2);

                            let text_x = rect.min.x + 8.0;
                            let title_pos = pos2(text_x, art_rect.max.y + 8.0);
                            let artist_pos = pos2(text_x, title_pos.y + 17.0);
                            let count_pos = pos2(text_x, artist_pos.y + 16.0);

                            ui.painter().text(
                                title_pos,
                                egui::Align2::LEFT_TOP,
                                &display_title,
                                egui::FontId::proportional(12.5),
                                theme.text_primary,
                            );

                            ui.painter().text(
                                artist_pos,
                                egui::Align2::LEFT_TOP,
                                &display_artist,
                                egui::FontId::proportional(11.0),
                                theme.text_secondary,
                            );

                            ui.painter().text(
                                count_pos,
                                egui::Align2::LEFT_TOP,
                                format!("{} Tracks", album_tracks.len()),
                                egui::FontId::proportional(10.0),
                                theme.text_muted,
                            );

                            if resp.clicked() {
                                state.selected_album = Some((*album_name).clone());
                            }
                        });
                    }
                });
            }
        });
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
    fn test_truncate_str() {
        assert_eq!(truncate_str("Short", 10), "Short");
        assert_eq!(truncate_str("A very long title that exceeds", 10), "A very lo…");
    }

    #[test]
    fn test_albums_view_rendering() {
        let ctx = egui::Context::default();
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let theme = Theme::from_name("Eclipse", None);
                let cfg = AppConfig::default();
                let mut player = PlayerController::new(&cfg);
                let mut art_manager = AlbumArtManager::new();
                let mut state = AlbumsViewState::default();
                let tracks = vec![
                    Track {
                        id: 1,
                        path: "test1.flac".into(),
                        title: "Song One".into(),
                        artist: "Artist A".into(),
                        album: "Album X".into(),
                        genre: "Electronic".into(),
                        year: 2024,
                        track_number: 1,
                        duration_secs: 180.0,
                        format: "FLAC".into(),
                        bitrate_kbps: 1000,
                        sample_rate: 44100,
                        channels: 2,
                        favorite: false,
                        has_art: false,
                        folder: "".into(),
                        album_artist: "".into(),
                        play_count: 5,
                        file_size: 1024,
                    },
                ];

                render_albums(ui, &tracks, "", &mut state, &mut player, &theme, &mut art_manager);
            });
        });
    }
}
