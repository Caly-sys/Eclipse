use crate::audio::PlayerController;
use crate::library::model::Track;
use crate::ui::art::AlbumArtManager;
use crate::ui::components::{dark_pill_button, format_badge, white_pill_button};
use crate::ui::icons::{icon_button, paint_icon, Icon};
use crate::ui::theme::Theme;
use egui::{
    pos2, vec2, Align2, Color32, FontId, Layout, Rect, RichText, Rounding, ScrollArea, Stroke, Ui,
    UiBuilder,
};
use rand::seq::SliceRandom;

#[derive(Default, Clone)]
pub struct SongsViewState {
    pub selected_track_id: Option<i64>,
}

#[allow(clippy::too_many_arguments)]
pub fn render_songs(
    ui: &mut Ui,
    tracks: &[Track],
    search_query: &str,
    state: &mut SongsViewState,
    player: &mut PlayerController,
    theme: &Theme,
    art_manager: &mut AlbumArtManager,
    on_toggle_fav: &mut Option<i64>,
) {
    render_songs_view(
        ui,
        "All Songs",
        "Your full local music library",
        Icon::Music,
        tracks,
        search_query,
        state,
        player,
        theme,
        art_manager,
        on_toggle_fav,
    );
}

#[allow(clippy::too_many_arguments)]
pub fn render_songs_view(
    ui: &mut Ui,
    title: &str,
    description: &str,
    icon: Icon,
    tracks: &[Track],
    search_query: &str,
    state: &mut SongsViewState,
    player: &mut PlayerController,
    theme: &Theme,
    art_manager: &mut AlbumArtManager,
    on_toggle_fav: &mut Option<i64>,
) {
    let query = search_query.trim().to_lowercase();
    let filtered_tracks: Vec<Track> = tracks
        .iter()
        .filter(|t| {
            if query.is_empty() {
                true
            } else {
                t.title.to_lowercase().contains(&query)
                    || t.artist.to_lowercase().contains(&query)
                    || t.album.to_lowercase().contains(&query)
                    || t.genre.to_lowercase().contains(&query)
            }
        })
        .cloned()
        .collect();

    ui.vertical(|ui| {
        // Section header banner
        let mut header_banner = theme.glass_frame(10.0);
        header_banner.inner_margin = egui::Margin::symmetric(14.0, 10.0);

        header_banner.show(ui, |ui| {
            ui.horizontal(|ui| {
                let (icon_rect, _) = ui.allocate_exact_size(vec2(38.0, 38.0), egui::Sense::hover());
                ui.painter().rect_filled(icon_rect, Rounding::same(10.0), theme.card);
                ui.painter().rect_stroke(icon_rect, Rounding::same(10.0), Stroke::new(1.0_f32, theme.border));
                let vector_rect = Rect::from_center_size(icon_rect.center(), vec2(20.0, 20.0));
                paint_icon(ui.painter(), icon, vector_rect, theme.accent);

                ui.add_space(10.0);

                ui.vertical(|ui| {
                    ui.label(RichText::new(title).size(17.0).color(theme.text_primary).strong());
                    let desc_text = if !query.is_empty() {
                        format!(
                            "{} · {} of {} tracks match \"{}\"",
                            description,
                            filtered_tracks.len(),
                            tracks.len(),
                            query
                        )
                    } else {
                        format!("{} · {} tracks", description, tracks.len())
                    };
                    ui.label(RichText::new(desc_text).size(11.5).color(theme.text_muted));
                });

                if !filtered_tracks.is_empty() {
                    ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                        let shuf_resp = dark_pill_button(ui, "Shuffle", Some(Icon::Shuffle));
                        if shuf_resp.clicked() {
                            let mut list = filtered_tracks.clone();
                            let mut rng = rand::thread_rng();
                            list.shuffle(&mut rng);
                            player.load_queue(list, 0);
                        }

                        ui.add_space(8.0);

                        let play_resp = white_pill_button(ui, "Play All", Some(Icon::Play));
                        if play_resp.clicked() {
                            player.load_queue(filtered_tracks.clone(), 0);
                        }
                    });
                }
            });
        });

        ui.add_space(8.0);

        if filtered_tracks.is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(40.0);
                let (empty_icon_rect, _) = ui.allocate_exact_size(vec2(32.0, 32.0), egui::Sense::hover());
                paint_icon(ui.painter(), Icon::Music, empty_icon_rect, theme.text_muted);
                ui.add_space(6.0);
                if query.is_empty() {
                    ui.label(RichText::new("No Tracks Found").size(16.0).color(theme.text_muted));
                    ui.label(RichText::new("Add music folders in Settings to begin.").size(12.0).color(theme.text_muted));
                } else {
                    ui.label(RichText::new(format!("No results for \"{}\"", query)).size(16.0).color(theme.text_muted));
                }
            });
        } else {
            render_track_table(
                ui,
                &filtered_tracks,
                Some(tracks),
                &mut state.selected_track_id,
                player,
                theme,
                art_manager,
                on_toggle_fav,
            );
        }
    });
}

/// Clean Multi-Column Track Table:
/// Columns:
/// - `#` (40px)
/// - `TITLE` (minmax 260px, 2.5fr) with 40x40 thumbnail and 2 stacked lines
/// - `ALBUM` (minmax 180px, 1.5fr, muted text #a1a1aa, properly separated column)
/// - `ADDED` (90px, relative date e.g. "2 d")
/// - `PLAYS` (70px, play count)
/// - `TIME` (75px, right-aligned duration e.g. "5:06" + heart toggle)
#[allow(clippy::too_many_arguments)]
pub fn render_track_table(
    ui: &mut Ui,
    tracks: &[Track],
    queue_tracks: Option<&[Track]>,
    selected_track_id: &mut Option<i64>,
    player: &mut PlayerController,
    theme: &Theme,
    art_manager: &mut AlbumArtManager,
    on_toggle_fav: &mut Option<i64>,
) {
    let right_padding = 24.0;
    let total_width = (ui.available_width() - right_padding).max(280.0);
    let col_num_w = 40.0;
    let col_added_w = 85.0;
    let col_plays_w = 65.0;
    let col_time_w = 75.0;
    let gap = 10.0;
    let total_gaps = gap * 5.0;
    let fixed_w = col_num_w + col_added_w + col_plays_w + col_time_w;
    let remaining_w = (total_width - fixed_w - total_gaps).max(100.0);

    let total_fr = 4.0;
    let col_title_w = remaining_w * 2.5 / total_fr;
    let col_album_w = remaining_w * 1.5 / total_fr;

    let x1 = col_num_w + gap;
    let x2 = x1 + col_title_w + gap;
    let x3 = x2 + col_album_w + gap;
    let x4 = x3 + col_added_w + gap;
    let x5 = x4 + col_plays_w + gap;

    // Multi-Column Table Header
    let (header_rect, _) = ui.allocate_exact_size(vec2(total_width, 28.0), egui::Sense::hover());
    ui.painter().line_segment(
        [
            pos2(header_rect.min.x, header_rect.max.y),
            pos2(header_rect.max.x, header_rect.max.y),
        ],
        Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(255, 255, 255, 15)),
    );

    ui.painter().text(
        pos2(header_rect.min.x + 20.0, header_rect.center().y),
        Align2::CENTER_CENTER,
        "#",
        FontId::proportional(11.0),
        theme.text_muted,
    );
    ui.painter().text(
        pos2(header_rect.min.x + x1 + 4.0, header_rect.center().y),
        Align2::LEFT_CENTER,
        "TITLE",
        FontId::proportional(11.0),
        theme.text_muted,
    );
    ui.painter().text(
        pos2(header_rect.min.x + x2, header_rect.center().y),
        Align2::LEFT_CENTER,
        "ALBUM",
        FontId::proportional(11.0),
        theme.text_muted,
    );
    ui.painter().text(
        pos2(header_rect.min.x + x3, header_rect.center().y),
        Align2::LEFT_CENTER,
        "ADDED",
        FontId::proportional(11.0),
        theme.text_muted,
    );
    ui.painter().text(
        pos2(header_rect.min.x + x4, header_rect.center().y),
        Align2::LEFT_CENTER,
        "PLAYS",
        FontId::proportional(11.0),
        theme.text_muted,
    );
    ui.painter().text(
        pos2(header_rect.max.x - 4.0, header_rect.center().y),
        Align2::RIGHT_CENTER,
        "TIME",
        FontId::proportional(11.0),
        theme.text_muted,
    );

    ui.add_space(4.0);

    // Track Rows in ScrollArea (independent playlist_scroll ID salt so it never couples with sidebar)
    ScrollArea::vertical()
        .id_salt("playlist_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing = vec2(0.0, 2.0);

            for (idx, track) in tracks.iter().enumerate() {
                let is_current = player.current_track().map(|t| t.id) == Some(track.id);
                let is_selected = *selected_track_id == Some(track.id);

                let row_height = 48.0;
                let (row_rect, row_resp) = ui.allocate_exact_size(vec2(total_width, row_height), egui::Sense::click());

                let row_bg = if is_current {
                    theme.accent_glow
                } else if row_resp.hovered() {
                    Color32::from_rgb(28, 28, 31) // theme.surface_hover #1c1c1f
                } else if is_selected {
                    Color32::from_rgba_premultiplied(180, 140, 255, 18)
                } else {
                    Color32::TRANSPARENT
                };

                if row_bg != Color32::TRANSPARENT {
                    ui.painter().rect_filled(row_rect, Rounding::same(6.0), row_bg);
                }

                // Col 0: # (40px)
                if is_current {
                    let play_state_icon = if player.engine.is_playing() { Icon::Play } else { Icon::Pause };
                    let icon_r = Rect::from_center_size(pos2(row_rect.min.x + 20.0, row_rect.center().y), vec2(14.0, 14.0));
                    paint_icon(ui.painter(), play_state_icon, icon_r, theme.accent);
                } else {
                    ui.painter().text(
                        pos2(row_rect.min.x + 20.0, row_rect.center().y),
                        Align2::CENTER_CENTER,
                        format!("{}", idx + 1),
                        FontId::proportional(12.0),
                        theme.text_muted,
                    );
                }

                // Col 1: TITLE (Square 40x40 thumbnail + 2 stacked lines)
                let title_start_x = row_rect.min.x + x1;
                let thumb_rect = Rect::from_min_size(
                    pos2(title_start_x, row_rect.center().y - 20.0),
                    vec2(40.0, 40.0),
                );

                let mut painted_thumb = false;
                if let Some(tex) = art_manager.get_or_load(ui.ctx(), &track.path) {
                    let img = egui::Image::new(tex).rounding(Rounding::same(6.0));
                    img.paint_at(ui, thumb_rect);
                    ui.painter().rect_stroke(
                        thumb_rect,
                        Rounding::same(6.0),
                        Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(255, 255, 255, 15)),
                    );
                    painted_thumb = true;
                }
                if !painted_thumb {
                    ui.painter().rect_filled(thumb_rect, Rounding::same(6.0), Color32::from_rgb(26, 26, 30));
                    ui.painter().rect_stroke(
                        thumb_rect,
                        Rounding::same(6.0),
                        Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(255, 255, 255, 10)),
                    );
                    let icon_r = Rect::from_center_size(thumb_rect.center(), vec2(16.0, 16.0));
                    paint_icon(ui.painter(), Icon::Music, icon_r, theme.text_muted);
                }

                // 2 stacked lines for title and artist/format badge
                let lines_rect = Rect::from_min_max(
                    pos2(title_start_x + 48.0, row_rect.min.y + 5.0),
                    pos2((title_start_x + col_title_w - 4.0).max(title_start_x + 50.0), row_rect.max.y - 5.0),
                );
                ui.allocate_new_ui(UiBuilder::new().max_rect(lines_rect).layout(Layout::top_down(egui::Align::Min)), |lines_ui| {
                    lines_ui.spacing_mut().item_spacing = vec2(0.0, 3.0);

                    let title_color = if is_current { theme.accent } else { Color32::from_rgb(244, 244, 245) };
                    lines_ui.add(egui::Label::new(RichText::new(track.display_title()).size(13.5).color(title_color).strong()).truncate());

                    lines_ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing = vec2(6.0, 0.0);
                        format_badge(ui, &track.format, track.is_lossless(), theme);
                        ui.add(egui::Label::new(RichText::new(track.display_artist()).size(11.5).color(Color32::from_rgb(161, 161, 170))).truncate());
                    });
                });

                // Col 2: ALBUM (properly separated column, muted text #a1a1aa)
                let album_rect = Rect::from_min_max(
                    pos2(row_rect.min.x + x2, row_rect.min.y),
                    pos2(row_rect.min.x + x2 + col_album_w, row_rect.max.y),
                );
                ui.allocate_new_ui(UiBuilder::new().max_rect(album_rect).layout(Layout::left_to_right(egui::Align::Center)), |album_ui| {
                    album_ui.add(egui::Label::new(RichText::new(track.display_album()).size(12.0).color(Color32::from_rgb(161, 161, 170))).truncate());
                });

                // Col 3: ADDED (90px)
                let added_rect = Rect::from_min_max(
                    pos2(row_rect.min.x + x3, row_rect.min.y),
                    pos2(row_rect.min.x + x3 + col_added_w, row_rect.max.y),
                );
                ui.allocate_new_ui(UiBuilder::new().max_rect(added_rect).layout(Layout::left_to_right(egui::Align::Center)), |added_ui| {
                    added_ui.label(RichText::new(track.relative_added_str()).size(12.0).color(Color32::from_rgb(161, 161, 170)));
                });

                // Col 4: PLAYS (70px)
                let plays_rect = Rect::from_min_max(
                    pos2(row_rect.min.x + x4, row_rect.min.y),
                    pos2(row_rect.min.x + x4 + col_plays_w, row_rect.max.y),
                );
                ui.allocate_new_ui(UiBuilder::new().max_rect(plays_rect).layout(Layout::left_to_right(egui::Align::Center)), |plays_ui| {
                    plays_ui.label(RichText::new(format!("{}", track.play_count)).size(12.0).color(Color32::from_rgb(161, 161, 170)));
                });

                // Col 5: TIME (right-aligned duration + favorite button, fits cleanly inside total_width)
                let time_rect = Rect::from_min_max(
                    pos2(row_rect.min.x + x5, row_rect.min.y),
                    pos2(row_rect.max.x - 4.0, row_rect.max.y),
                );
                let mut fav_clicked = false;
                ui.allocate_new_ui(UiBuilder::new().max_rect(time_rect).layout(Layout::right_to_left(egui::Align::Center)), |time_ui| {
                    time_ui.label(RichText::new(track.formatted_duration()).size(12.0).color(Color32::from_rgb(161, 161, 170)));

                    time_ui.add_space(8.0);

                    let (fav_icon, fav_color) = if track.favorite {
                        (Icon::Heart, Color32::from_rgb(239, 68, 68))
                    } else {
                        (Icon::HeartOutline, theme.text_muted)
                    };
                    let fav_resp = icon_button(time_ui, fav_icon, 13.0, fav_color);
                    if fav_resp.clicked() {
                        *on_toggle_fav = Some(track.id);
                        fav_clicked = true;
                    }
                });

                if row_resp.hovered() && !fav_clicked {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                }

                if row_resp.double_clicked() && !fav_clicked {
                    let q = queue_tracks.unwrap_or(tracks);
                    let q_idx = q.iter().position(|t| t.id == track.id).unwrap_or(idx);
                    player.load_queue(q.to_vec(), q_idx);
                } else if row_resp.clicked() && !fav_clicked {
                    *selected_track_id = Some(track.id);
                }
            }

            // Independent bottom padding so all tracks can scroll freely to the end
            ui.add_space(96.0);
        });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppConfig;

    fn sample_track(id: i64, title: &str, artist: &str, album: &str) -> Track {
        Track {
            id,
            path: format!("C:/music/{}.flac", title),
            title: title.to_string(),
            artist: artist.to_string(),
            album: album.to_string(),
            album_artist: artist.to_string(),
            genre: "Electronic".to_string(),
            year: 2024,
            track_number: id as u32,
            duration_secs: 215.0,
            bitrate_kbps: 920,
            sample_rate: 44100,
            channels: 2,
            format: "FLAC".to_string(),
            file_size: 25_000_000,
            play_count: 12,
            favorite: false,
            has_art: false,
            folder: "C:/music".to_string(),
        }
    }

    #[test]
    fn test_songs_view_rendering_empty() {
        let ctx = egui::Context::default();
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let theme = Theme::from_name("Eclipse", None);
                let cfg = AppConfig::default();
                let mut player = PlayerController::new(&cfg);
                let mut art_manager = AlbumArtManager::new();
                let mut state = SongsViewState::default();
                let mut on_toggle_fav = None;

                render_songs(
                    ui,
                    &[],
                    "",
                    &mut state,
                    &mut player,
                    &theme,
                    &mut art_manager,
                    &mut on_toggle_fav,
                );
            });
        });
    }

    #[test]
    fn test_track_table_renders_with_tracks() {
        let ctx = egui::Context::default();
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let theme = Theme::from_name("Eclipse", None);
                let cfg = AppConfig::default();
                let mut player = PlayerController::new(&cfg);
                let mut art_manager = AlbumArtManager::new();
                let mut selected_id = None;
                let mut on_toggle_fav = None;

                let tracks = vec![
                    sample_track(1, "Neon Horizon", "Solaris", "Midnight Echoes"),
                    sample_track(2, "Velvet Sky", "Komorebi", "Aether"),
                ];

                render_track_table(
                    ui,
                    &tracks,
                    None,
                    &mut selected_id,
                    &mut player,
                    &theme,
                    &mut art_manager,
                    &mut on_toggle_fav,
                );
            });
        });
    }
}
