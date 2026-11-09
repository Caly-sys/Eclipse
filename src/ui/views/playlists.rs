use crate::audio::PlayerController;
use crate::library::model::{Playlist, Track};
use crate::ui::art::AlbumArtManager;
use crate::ui::components::{dark_pill_button, glass_button, white_pill_button};
use crate::ui::icons::{icon_button, paint_icon, Icon};
use crate::ui::theme::Theme;
use crate::ui::views::songs::render_track_table;
use egui::{
    pos2, vec2, Color32, Layout, Rect, RichText, Rounding, ScrollArea, Stroke, Ui, UiBuilder,
};
use rand::seq::SliceRandom;

#[derive(Default, Clone)]
pub struct PlaylistsViewState {
    pub selected_playlist_id: Option<i64>,
    pub new_playlist_name: String,
    pub is_creating: bool,
    pub filter_query: String,
    pub selected_track_id: Option<i64>,
    pub show_more_menu: bool,
}

#[allow(clippy::too_many_arguments)]
pub fn render_playlists(
    ui: &mut Ui,
    playlists: &[Playlist],
    playlist_tracks: &[Track],
    state: &mut PlaylistsViewState,
    player: &mut PlayerController,
    theme: &Theme,
    art_manager: &mut AlbumArtManager,
    on_toggle_fav: &mut Option<i64>,
    on_create_playlist: &mut Option<String>,
    on_delete_playlist: &mut Option<i64>,
) {
    if playlists.is_empty() {
        ui.vertical_centered(|ui| {
            ui.add_space(60.0);
            let (icon_rect, _) = ui.allocate_exact_size(vec2(48.0, 48.0), egui::Sense::hover());
            paint_icon(ui.painter(), Icon::Playlist, icon_rect, theme.text_muted);
            ui.add_space(12.0);
            ui.label(RichText::new("No Playlists Yet").size(20.0).color(theme.text_primary).strong());
            ui.label(RichText::new("Create custom playlists to organize your favorite tracks.").size(13.0).color(theme.text_muted));
            ui.add_space(16.0);

            if !state.is_creating {
                let create_btn = white_pill_button(ui, "Create Playlist", Some(Icon::Plus));
                if create_btn.clicked() {
                    state.is_creating = true;
                }
            } else {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Name:").color(theme.text_secondary));
                    ui.add(egui::TextEdit::singleline(&mut state.new_playlist_name).desired_width(200.0));
                    let save_btn = egui::Button::new(RichText::new("Save").color(Color32::from_rgb(11, 9, 20)).strong())
                        .fill(theme.accent)
                        .rounding(Rounding::same(6.0));
                    if ui.add(save_btn).clicked() {
                        let name = state.new_playlist_name.trim();
                        if !name.is_empty() {
                            *on_create_playlist = Some(name.to_string());
                            state.new_playlist_name.clear();
                            state.is_creating = false;
                        }
                    }
                    if ui.button("Cancel").clicked() {
                        state.is_creating = false;
                    }
                });
            }
        });
        return;
    }

    // Ensure a playlist is selected
    if state.selected_playlist_id.is_none()
        || !playlists.iter().any(|p| Some(p.id) == state.selected_playlist_id)
    {
        state.selected_playlist_id = Some(playlists[0].id);
    }

    // Top playlist switcher bar
    ui.horizontal(|ui| {
        ScrollArea::horizontal()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing = vec2(6.0, 0.0);
                for pl in playlists {
                    let is_active = state.selected_playlist_id == Some(pl.id);
                    let label = format!("{} ({})", pl.name, pl.track_count);
                    let pill_resp = glass_button(ui, &label, is_active, theme);
                    if pill_resp.clicked() {
                        state.selected_playlist_id = Some(pl.id);
                    }
                }
            });

        ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
            let add_btn = dark_pill_button(ui, "New Playlist", Some(Icon::Plus));
            if add_btn.clicked() {
                state.is_creating = !state.is_creating;
            }
        });
    });

    if state.is_creating {
        ui.add_space(6.0);
        let mut frame = theme.glass_frame(8.0);
        frame.inner_margin = egui::Margin::same(10.0);
        frame.show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("Name:").color(theme.text_secondary));
                ui.add(egui::TextEdit::singleline(&mut state.new_playlist_name).desired_width(200.0));
                let create_btn = egui::Button::new(RichText::new("Create").color(Color32::from_rgb(11, 9, 20)).strong())
                    .fill(theme.accent)
                    .rounding(Rounding::same(6.0));
                if ui.add(create_btn).clicked() {
                    let name = state.new_playlist_name.trim();
                    if !name.is_empty() {
                        *on_create_playlist = Some(name.to_string());
                        state.new_playlist_name.clear();
                        state.is_creating = false;
                    }
                }
                if ui.button("Cancel").clicked() {
                    state.is_creating = false;
                }
            });
        });
    }

    ui.add_space(10.0);

    let selected_pl = playlists
        .iter()
        .find(|p| Some(p.id) == state.selected_playlist_id)
        .unwrap_or(&playlists[0]);

    render_playlist_detail(
        ui,
        selected_pl,
        playlist_tracks,
        state,
        player,
        theme,
        art_manager,
        on_toggle_fav,
        on_delete_playlist,
    );
}

#[allow(clippy::too_many_arguments)]
pub fn render_playlist_detail(
    ui: &mut Ui,
    playlist: &Playlist,
    tracks: &[Track],
    state: &mut PlaylistsViewState,
    player: &mut PlayerController,
    theme: &Theme,
    art_manager: &mut AlbumArtManager,
    on_toggle_fav: &mut Option<i64>,
    on_delete_playlist: &mut Option<i64>,
) {
    ui.vertical(|ui| {
        // 1. Playlist Hero Header
        render_playlist_hero_header(ui, playlist, tracks, theme, art_manager);

        ui.add_space(14.0);

        // 2. Action Bar
        render_playlist_action_bar(
            ui,
            tracks,
            state,
            player,
            theme,
            on_delete_playlist,
            playlist.id,
        );

        ui.add_space(10.0);

        // 3. Multi-Column Track Table
        if tracks.is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(48.0);
                let (empty_icon_rect, _) = ui.allocate_exact_size(vec2(36.0, 36.0), egui::Sense::hover());
                paint_icon(ui.painter(), Icon::Music, empty_icon_rect, theme.text_muted);
                ui.add_space(8.0);
                ui.label(RichText::new("Playlist is Empty").size(16.0).color(theme.text_secondary).strong());
                ui.label(RichText::new("Add songs to this playlist from your library.").size(12.0).color(theme.text_muted));
            });
        } else {
            let query = state.filter_query.trim().to_lowercase();
            let filtered_tracks: Vec<Track> = if query.is_empty() {
                tracks.to_vec()
            } else {
                tracks
                    .iter()
                    .filter(|t| {
                        t.title.to_lowercase().contains(&query)
                            || t.artist.to_lowercase().contains(&query)
                            || t.album.to_lowercase().contains(&query)
                    })
                    .cloned()
                    .collect()
            };

            if filtered_tracks.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.add_space(40.0);
                    let (empty_icon_rect, _) = ui.allocate_exact_size(vec2(32.0, 32.0), egui::Sense::hover());
                    paint_icon(ui.painter(), Icon::Search, empty_icon_rect, theme.text_muted);
                    ui.add_space(6.0);
                    ui.label(RichText::new(format!("No tracks matching \"{}\"", query)).size(15.0).color(theme.text_muted));
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
        }
    });
}

pub fn render_playlist_hero_header(
    ui: &mut Ui,
    playlist: &Playlist,
    tracks: &[Track],
    theme: &Theme,
    art_manager: &mut AlbumArtManager,
) {
    let header_height = 168.0;
    let (header_rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), header_height), egui::Sense::hover());

    // 1. Subtle ambient glow / gradient backdrop behind header
    ui.painter().rect_filled(
        header_rect,
        Rounding::same(14.0),
        Color32::from_rgba_premultiplied(16, 15, 22, 175),
    );
    // Radial ambient glow centered behind the cover art
    let glow_center = pos2(header_rect.min.x + 84.0, header_rect.min.y + 84.0);
    ui.painter().circle_filled(
        glow_center,
        110.0,
        Color32::from_rgba_premultiplied(
            theme.accent.r() / 5,
            theme.accent.g() / 5,
            theme.accent.b() / 5,
            35,
        ),
    );
    // Subtle hairline border around header
    ui.painter().rect_stroke(
        header_rect,
        Rounding::same(14.0),
        Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(255, 255, 255, 12)),
    );

    // 2. Large rounded Playlist Cover Art (136x136px, Rounding::same(12.0)) on left
    let cover_rect = Rect::from_min_size(
        pos2(header_rect.min.x + 16.0, header_rect.min.y + 16.0),
        vec2(136.0, 136.0),
    );

    let mut painted_cover = false;
    if let Some(first_track) = tracks.first() {
        if let Some(tex) = art_manager.get_or_load(ui.ctx(), &first_track.path) {
            let img = egui::Image::new(tex).rounding(Rounding::same(12.0));
            img.paint_at(ui, cover_rect);
            ui.painter().rect_stroke(
                cover_rect,
                Rounding::same(12.0),
                Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(255, 255, 255, 25)),
            );
            painted_cover = true;
        }
    }

    if !painted_cover {
        ui.painter().rect_filled(cover_rect, Rounding::same(12.0), Color32::from_rgb(22, 22, 26));
        ui.painter().rect_stroke(cover_rect, Rounding::same(12.0), Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(255, 255, 255, 15)));
        let icon_rect = Rect::from_center_size(cover_rect.center(), vec2(48.0, 48.0));
        paint_icon(ui.painter(), Icon::Playlist, icon_rect, theme.text_muted);
    }

    // 3. Right of cover: micro-label, 36px bold title, metadata summary
    let text_area = Rect::from_min_max(
        pos2(header_rect.min.x + 172.0, header_rect.min.y + 20.0),
        pos2(header_rect.max.x - 20.0, header_rect.min.y + 148.0),
    );
    ui.allocate_new_ui(UiBuilder::new().max_rect(text_area).layout(Layout::top_down(egui::Align::Min)), |text_ui| {
        text_ui.spacing_mut().item_spacing = vec2(0.0, 6.0);

        // Uppercase muted micro-label: PLAYLIST (11px, theme.text_muted, strong)
        text_ui.label(RichText::new("PLAYLIST").size(11.0).color(theme.text_muted).strong());

        // Large bold playlist title: 36px bold (#f4f4f5)
        text_ui.add(
            egui::Label::new(
                RichText::new(&playlist.name)
                    .size(36.0)
                    .color(Color32::from_rgb(244, 244, 245))
                    .strong(),
            )
            .truncate(),
        );

        // Metadata summary: "X tracks · Y h Z min"
        let total_duration = Playlist::formatted_total_duration(tracks);
        let count_str = if tracks.len() == 1 { "1 track" } else { &format!("{} tracks", tracks.len()) };
        let summary_text = format!("{} · {}", count_str, total_duration);
        text_ui.label(RichText::new(summary_text).size(13.0).color(theme.text_muted));
    });
}

pub fn render_playlist_action_bar(
    ui: &mut Ui,
    tracks: &[Track],
    state: &mut PlaylistsViewState,
    player: &mut PlayerController,
    theme: &Theme,
    on_delete_playlist: &mut Option<i64>,
    playlist_id: i64,
) {
    ui.horizontal(|ui| {
        // Left: Solid white Play pill button
        let play_resp = white_pill_button(ui, "Play", Some(Icon::Play));
        if play_resp.clicked() && !tracks.is_empty() {
            player.load_queue(tracks.to_vec(), 0);
        }

        ui.add_space(8.0);

        // Dark Shuffle pill button
        let shuf_resp = dark_pill_button(ui, "Shuffle", Some(Icon::Shuffle));
        if shuf_resp.clicked() && !tracks.is_empty() {
            let mut list = tracks.to_vec();
            let mut rng = rand::thread_rng();
            list.shuffle(&mut rng);
            player.load_queue(list, 0);
        }

        ui.add_space(8.0);

        // More menu button (30x30 circle with vector Icon::MoreHorizontal)
        let (more_rect, more_resp) = ui.allocate_exact_size(vec2(30.0, 30.0), egui::Sense::click());
        if more_resp.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
        let more_bg = if more_resp.hovered() { Color32::from_rgb(42, 42, 47) } else { Color32::from_rgb(34, 34, 38) };
        let more_stroke = Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(255, 255, 255, 20));
        ui.painter().rect_filled(more_rect, Rounding::same(15.0), more_bg);
        ui.painter().rect_stroke(more_rect, Rounding::same(15.0), more_stroke);
        let dot_rect = Rect::from_center_size(more_rect.center(), vec2(14.0, 14.0));
        paint_icon(ui.painter(), Icon::MoreHorizontal, dot_rect, Color32::from_rgb(228, 228, 231));

        if more_resp.clicked() {
            state.show_more_menu = !state.show_more_menu;
        }

        // Right: Compact rounded Filter search input box
        ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
            render_filter_input(ui, &mut state.filter_query, theme);
        });
    });

    if state.show_more_menu {
        ui.add_space(6.0);
        let mut menu_frame = theme.card_frame(8.0);
        menu_frame.inner_margin = egui::Margin::symmetric(12.0, 8.0);
        menu_frame.show(ui, |ui| {
            ui.horizontal(|ui| {
                if icon_button(ui, Icon::Trash, 14.0, Color32::from_rgb(239, 68, 68)).clicked() {
                    *on_delete_playlist = Some(playlist_id);
                    state.show_more_menu = false;
                }
                ui.add_space(4.0);
                if ui.button(RichText::new("Delete Playlist").color(Color32::from_rgb(239, 68, 68)).size(12.5)).clicked() {
                    *on_delete_playlist = Some(playlist_id);
                    state.show_more_menu = false;
                }
            });
        });
    }
}

pub fn render_filter_input(ui: &mut Ui, query: &mut String, theme: &Theme) {
    let width = 180.0;
    let height = 30.0;
    let (rect, _) = ui.allocate_exact_size(vec2(width, height), egui::Sense::hover());

    let bg_color = Color32::from_rgb(26, 26, 30);
    let stroke_color = Color32::from_rgba_premultiplied(255, 255, 255, 20);
    ui.painter().rect_filled(rect, Rounding::same(15.0), bg_color);
    ui.painter().rect_stroke(rect, Rounding::same(15.0), Stroke::new(1.0_f32, stroke_color));

    // Search vector icon on left
    let icon_rect = Rect::from_center_size(pos2(rect.min.x + 16.0, rect.center().y), vec2(13.0, 13.0));
    paint_icon(ui.painter(), Icon::Search, icon_rect, theme.text_muted);

    // TextEdit in remaining space
    let edit_rect = Rect::from_min_max(
        pos2(rect.min.x + 30.0, rect.min.y + 4.0),
        pos2(rect.max.x - 10.0, rect.max.y - 4.0),
    );
    ui.allocate_new_ui(UiBuilder::new().max_rect(edit_rect).layout(Layout::left_to_right(egui::Align::Center)), |child_ui| {
        let text_edit = egui::TextEdit::singleline(query)
            .hint_text(RichText::new("Filter...").size(12.0).color(theme.text_muted))
            .text_color(theme.text_primary)
            .frame(false)
            .desired_width(edit_rect.width());
        child_ui.add(text_edit);
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
            duration_secs: 245.0,
            bitrate_kbps: 920,
            sample_rate: 44100,
            channels: 2,
            format: "FLAC".to_string(),
            file_size: 25_000_000,
            play_count: 5,
            favorite: false,
            has_art: false,
            folder: "C:/music".to_string(),
        }
    }

    fn sample_playlist(id: i64, name: &str, count: usize) -> Playlist {
        Playlist {
            id,
            name: name.to_string(),
            created: 1700000000.0,
            description: "A sleek test playlist".to_string(),
            track_count: count,
        }
    }

    #[test]
    fn test_playlist_empty_state() {
        let ctx = egui::Context::default();
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let theme = Theme::from_name("Eclipse", None);
                let cfg = AppConfig::default();
                let mut player = PlayerController::new(&cfg);
                let mut art_manager = AlbumArtManager::new();
                let mut state = PlaylistsViewState::default();
                let mut on_create = None;
                let mut on_delete = None;
                let mut on_toggle_fav = None;

                render_playlists(
                    ui,
                    &[],
                    &[],
                    &mut state,
                    &mut player,
                    &theme,
                    &mut art_manager,
                    &mut on_toggle_fav,
                    &mut on_create,
                    &mut on_delete,
                );
            });
        });
    }

    #[test]
    fn test_playlist_hero_header_rendering() {
        let ctx = egui::Context::default();
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let theme = Theme::from_name("Eclipse", None);
                let mut art_manager = AlbumArtManager::new();
                let pl = sample_playlist(1, "Deep Focus", 2);
                let tracks = vec![
                    sample_track(1, "Neon Horizon", "Solaris", "Midnight Echoes"),
                    sample_track(2, "Velvet Sky", "Komorebi", "Aether"),
                ];

                render_playlist_hero_header(ui, &pl, &tracks, &theme, &mut art_manager);
            });
        });
    }

    #[test]
    fn test_playlist_detail_and_track_table_rendering() {
        let ctx = egui::Context::default();
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let theme = Theme::from_name("Eclipse", None);
                let cfg = AppConfig::default();
                let mut player = PlayerController::new(&cfg);
                let mut art_manager = AlbumArtManager::new();
                let mut state = PlaylistsViewState::default();
                let mut on_toggle_fav = None;
                let mut on_delete = None;

                let pl = sample_playlist(1, "Lo-Fi Beats", 2);
                let tracks = vec![
                    sample_track(10, "Rainy Coffee", "Kitsune", "Raindrops"),
                    sample_track(11, "Evening Glow", "Murasaki", "Sundown"),
                ];

                render_playlist_detail(
                    ui,
                    &pl,
                    &tracks,
                    &mut state,
                    &mut player,
                    &theme,
                    &mut art_manager,
                    &mut on_toggle_fav,
                    &mut on_delete,
                );
            });
        });
    }

    #[test]
    fn test_filter_tracks_in_playlist() {
        let ctx = egui::Context::default();
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let theme = Theme::from_name("Eclipse", None);
                let cfg = AppConfig::default();
                let mut player = PlayerController::new(&cfg);
                let mut art_manager = AlbumArtManager::new();
                let mut state = PlaylistsViewState {
                    filter_query: "Rainy".to_string(),
                    ..Default::default()
                };
                let mut on_toggle_fav = None;
                let mut on_delete = None;

                let pl = sample_playlist(1, "Lo-Fi Beats", 2);
                let tracks = vec![
                    sample_track(10, "Rainy Coffee", "Kitsune", "Raindrops"),
                    sample_track(11, "Evening Glow", "Murasaki", "Sundown"),
                ];

                render_playlist_detail(
                    ui,
                    &pl,
                    &tracks,
                    &mut state,
                    &mut player,
                    &theme,
                    &mut art_manager,
                    &mut on_toggle_fav,
                    &mut on_delete,
                );
            });
        });
    }
}
