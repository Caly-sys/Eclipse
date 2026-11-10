//! Stream Views for Eclipse (Spotify Public Link Importer & YouTube Music).
//! Reference image compliance: pure vector icons, dark purple glass aesthetic, zero login RFC 7636-free importer.

use crate::audio::PlayerController;
use crate::library::model::Track;
use crate::streams::controller::StreamController;
use crate::streams::credentials::CredentialStore;
use crate::streams::provider::StreamTrack;
use crate::ui::art::AlbumArtManager;
use crate::ui::components::{dark_pill_button, format_badge, pill_badge, white_pill_button};
use crate::ui::icons::{paint_icon, render_icon, Icon};
use crate::ui::sidebar::NavTab;
use crate::ui::theme::Theme;
use egui::{
    pos2, vec2, Align2, Color32, FontId, Layout, Rect, RichText, Rounding, ScrollArea, Sense,
    Stroke, TextEdit, Ui, UiBuilder,
};
use rand::seq::SliceRandom;

#[derive(Debug, Clone, Default)]
pub struct StreamsViewState {
    pub spotify_url_input: String,
    pub youtube_search_query: String,
    pub status_msg: Option<(String, bool)>, // (message, is_error)
    pub selected_track_id: Option<String>,
}

/// Render the Spotify Public Link Importer view (zero login, zero developer app).
#[allow(clippy::too_many_arguments)]
pub fn render_spotify_view(
    ui: &mut Ui,
    stream_controller: &mut StreamController,
    player: &mut PlayerController,
    theme: &Theme,
    state: &mut StreamsViewState,
    art_manager: &mut AlbumArtManager,
    on_navigate: &mut Option<NavTab>,
) {
    ScrollArea::vertical()
        .id_salt("spotify_main_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing = vec2(0.0, 14.0);

            // =================================================================
            // 1. HEADER GROUP (Reference Design)
            // =================================================================
            let header_frame = theme.glass_frame(14.0);
            header_frame.show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing = vec2(0.0, 12.0);

                    ui.horizontal(|ui| {
                        // 68x68 dark glass rounded square card with Spotify green vector icon
                        let (card_rect, _) = ui.allocate_exact_size(vec2(68.0, 68.0), Sense::hover());
                        ui.painter().rect_filled(
                            card_rect,
                            Rounding::same(14.0),
                            Color32::from_rgb(20, 20, 24),
                        );
                        ui.painter().rect_stroke(
                            card_rect,
                            Rounding::same(14.0),
                            Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(255, 255, 255, 20)),
                        );
                        let icon_box = Rect::from_center_size(card_rect.center(), vec2(36.0, 36.0));
                        paint_icon(ui.painter(), Icon::Spotify, icon_box, Color32::from_rgb(30, 215, 96));

                        ui.add_space(14.0);

                        // Right of card: Micro-tag, Title, Subtitle
                        ui.vertical(|ui| {
                            ui.spacing_mut().item_spacing = vec2(0.0, 2.0);

                            ui.label(
                                RichText::new("STREAM")
                                    .size(10.0)
                                    .color(theme.text_muted)
                                    .strong(),
                            );

                            ui.label(
                                RichText::new("Spotify")
                                    .size(29.0)
                                    .color(theme.text_primary)
                                    .strong(),
                            );

                            ui.label(
                                RichText::new(
                                    "Paste a public Spotify track, album or playlist link. Save it as a playlist, stream it (matched from YouTube — Spotify doesn't allow third-party streaming) or download it.",
                                )
                                .size(12.5)
                                .color(theme.text_secondary),
                            );
                        });
                    });

                    ui.add_space(4.0);

                    // =========================================================
                    // 2. INPUT BAR (Reference Design)
                    // =========================================================
                    let input_bar_frame = egui::Frame::none()
                        .fill(Color32::from_rgb(22, 22, 26))
                        .stroke(Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(255, 255, 255, 18)))
                        .rounding(Rounding::same(12.0))
                        .inner_margin(egui::Margin::symmetric(14.0, 8.0));

                    input_bar_frame.show(ui, |ui| {
                        ui.horizontal(|ui| {
                            render_icon(ui, Icon::Search, 16.0, theme.text_muted);
                            ui.add_space(8.0);

                            let text_w = (ui.available_width() - 175.0).max(120.0);
                            let text_resp = ui.add(
                                TextEdit::singleline(&mut state.spotify_url_input)
                                    .hint_text("Paste a Spotify link — playlist, album or track (open.spotify.com/...)")
                                    .desired_width(text_w)
                                    .frame(false)
                                    .margin(egui::Margin::symmetric(0.0, 4.0))
                                    .text_color(theme.text_primary),
                            );

                            // Right inside input bar: ENTER pill badge
                            pill_badge(
                                ui,
                                "ENTER",
                                Color32::from_rgba_premultiplied(255, 255, 255, 12),
                                theme.text_muted,
                            );

                            ui.add_space(8.0);

                            // Right button: White pill button "Import"
                            let import_btn = white_pill_button(ui, "Import", Some(Icon::Search));

                            let enter_hit = text_resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                            if (import_btn.clicked() || enter_hit) && !state.spotify_url_input.trim().is_empty() {
                                let url = state.spotify_url_input.trim().to_string();
                                stream_controller.import_spotify_link(&url);
                                state.status_msg = None;
                            }
                        });
                    });

                    // Feedback status banner if present
                    if let Some((ref msg, is_err)) = state.status_msg {
                        ui.add_space(2.0);
                        render_status_banner(ui, msg, is_err, theme);
                    }
                });
            });

            // =================================================================
            // 3. MAIN CONTENT: Loading, Error, Truncated, Empty, or Track Table
            // =================================================================
            if stream_controller.is_busy {
                render_loading_state(ui, theme);
            } else if let Some(ref import_res) = stream_controller.last_spotify_import.clone() {
                match import_res {
                    Ok(import) => {
                        // Truncation banner if needed
                        if import.is_truncated {
                            let note = import.truncation_message.as_deref().unwrap_or(
                                "Showing first 100 tracks from public Spotify embed",
                            );
                            render_truncation_notice(ui, note, theme);
                        }

                        // Summary & Action Buttons Bar
                        render_action_bar(
                            ui,
                            import.title.as_str(),
                            import.subtitle.as_deref().unwrap_or("Spotify Import"),
                            &import.tracks,
                            stream_controller,
                            theme,
                            on_navigate,
                        );

                        // Tracks Table
                        render_imported_track_table(
                            ui,
                            &import.tracks,
                            player,
                            stream_controller,
                            state,
                            theme,
                            art_manager,
                        );
                    }
                    Err(err) => {
                        render_error_card(ui, err, theme, state, stream_controller);
                        render_empty_guide(ui, state, stream_controller, theme);
                    }
                }
            } else {
                render_empty_guide(ui, state, stream_controller, theme);
            }

            ui.add_space(32.0);
        });
}

/// Render the Action Buttons Bar above the tracklist
fn render_action_bar(
    ui: &mut Ui,
    title: &str,
    subtitle: &str,
    tracks: &[StreamTrack],
    stream_controller: &mut StreamController,
    theme: &Theme,
    on_navigate: &mut Option<NavTab>,
) {
    let bar_frame = theme.card_frame(12.0);
    bar_frame.show(ui, |ui| {
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing = vec2(0.0, 10.0);

            // Summary Title & Track Count
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(title)
                        .size(18.0)
                        .color(theme.text_primary)
                        .strong(),
                );
                ui.label(
                    RichText::new(format!("· {subtitle}"))
                        .size(13.0)
                        .color(theme.text_muted),
                );

                ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                    pill_badge(
                        ui,
                        &format!("{} TRACKS", tracks.len()),
                        Color32::from_rgba_unmultiplied(30, 215, 96, 25),
                        Color32::from_rgb(30, 215, 96),
                    );
                });
            });

            ui.separator();

            // Actions: Play all, Shuffle, Save all as playlist, Find in my library, Recently streamed
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = vec2(10.0, 0.0);

                if white_pill_button(ui, "Play all", Some(Icon::Play)).clicked() && !tracks.is_empty() {
                    stream_controller.load_stream_queue(tracks.to_vec(), 0);
                }

                if dark_pill_button(ui, "Shuffle", Some(Icon::Shuffle)).clicked() && !tracks.is_empty() {
                    let mut shuffled = tracks.to_vec();
                    let mut rng = rand::thread_rng();
                    shuffled.shuffle(&mut rng);
                    stream_controller.load_stream_queue(shuffled, 0);
                }

                if dark_pill_button(ui, "Save all as playlist", Some(Icon::Playlist)).clicked() && !tracks.is_empty() {
                    let pl_name = title.to_string();
                    stream_controller.save_imported_playlist(pl_name, tracks.to_vec());
                }

                if dark_pill_button(ui, "Find in my library", Some(Icon::Search)).clicked() && !tracks.is_empty() {
                    stream_controller.match_library(tracks.to_vec());
                }

                ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                    let recent_btn = ui.add(
                        egui::Button::new(
                            RichText::new("Recently streamed")
                                .size(12.5)
                                .color(theme.text_secondary)
                                .underline(),
                        )
                        .fill(Color32::TRANSPARENT)
                        .stroke(Stroke::NONE),
                    );
                    if recent_btn.hovered() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    }
                    if recent_btn.clicked() {
                        *on_navigate = Some(NavTab::RecentlyPlayed);
                    }
                });
            });
        });
    });
}

/// Render the multi-column Track Table matching the reference layout
fn render_imported_track_table(
    ui: &mut Ui,
    tracks: &[StreamTrack],
    player: &mut PlayerController,
    stream_controller: &mut StreamController,
    state: &mut StreamsViewState,
    theme: &Theme,
    art_manager: &mut AlbumArtManager,
) {
    let right_padding = 20.0;
    let total_width = (ui.available_width() - right_padding).max(280.0);

    let col_num_w = 40.0;
    let col_added_w = 80.0;
    let col_plays_w = 60.0;
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
    let (header_rect, _) = ui.allocate_exact_size(vec2(total_width, 28.0), Sense::hover());
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

    // Track Rows in ScrollArea with unique ID salt
    ScrollArea::vertical()
        .id_salt("spotify_tracks_table_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing = vec2(0.0, 3.0);

            for (idx, track) in tracks.iter().enumerate() {
                let is_current = player.playback_source.is_stream()
                    && player.active_stream_track.as_ref().map(|t| &t.id) == Some(&track.id);
                let is_selected = state.selected_track_id.as_deref() == Some(&track.id);

                let row_height = 48.0;
                let (row_rect, row_resp) = ui.allocate_exact_size(vec2(total_width, row_height), Sense::click());

                let row_bg = if is_current {
                    theme.accent_glow
                } else if row_resp.hovered() {
                    Color32::from_rgb(28, 28, 31)
                } else if is_selected {
                    Color32::from_rgba_premultiplied(180, 140, 255, 18)
                } else {
                    Color32::TRANSPARENT
                };

                if row_bg != Color32::TRANSPARENT {
                    ui.painter().rect_filled(row_rect, Rounding::same(6.0), row_bg);
                }

                // Col 0: # / Play Icon on Hover
                if is_current {
                    let is_playing = stream_controller.active_state.as_ref().map(|s| s.is_playing).unwrap_or(true);
                    let play_icon = if is_playing { Icon::Play } else { Icon::Pause };
                    let icon_r = Rect::from_center_size(pos2(row_rect.min.x + 20.0, row_rect.center().y), vec2(14.0, 14.0));
                    paint_icon(ui.painter(), play_icon, icon_r, theme.accent);
                } else if row_resp.hovered() {
                    let icon_r = Rect::from_center_size(pos2(row_rect.min.x + 20.0, row_rect.center().y), vec2(13.0, 13.0));
                    paint_icon(ui.painter(), Icon::Play, icon_r, Color32::WHITE);
                } else {
                    ui.painter().text(
                        pos2(row_rect.min.x + 20.0, row_rect.center().y),
                        Align2::CENTER_CENTER,
                        format!("{}", idx + 1),
                        FontId::proportional(12.0),
                        theme.text_muted,
                    );
                }

                // Col 1: TITLE (40x40 thumbnail + 2 stacked lines)
                let title_start_x = row_rect.min.x + x1;
                let thumb_rect = Rect::from_min_size(
                    pos2(title_start_x, row_rect.center().y - 20.0),
                    vec2(40.0, 40.0),
                );

                let mut painted_thumb = false;
                if let Some(ref art_url) = track.art_url {
                    if let Some(tex) = art_manager.get_or_load(ui.ctx(), art_url) {
                        let img = egui::Image::new(tex).rounding(Rounding::same(6.0));
                        img.paint_at(ui, thumb_rect);
                        ui.painter().rect_stroke(
                            thumb_rect,
                            Rounding::same(6.0),
                            Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(255, 255, 255, 15)),
                        );
                        painted_thumb = true;
                    }
                }

                if !painted_thumb {
                    ui.painter().rect_filled(thumb_rect, Rounding::same(6.0), Color32::from_rgb(26, 26, 30));
                    ui.painter().rect_stroke(
                        thumb_rect,
                        Rounding::same(6.0),
                        Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(255, 255, 255, 10)),
                    );
                    let icon_r = Rect::from_center_size(thumb_rect.center(), vec2(16.0, 16.0));
                    paint_icon(ui.painter(), Icon::Spotify, icon_r, Color32::from_rgb(30, 215, 96));
                }

                // 2 stacked lines: Title and (SP badge + Artist)
                let lines_rect = Rect::from_min_max(
                    pos2(title_start_x + 48.0, row_rect.min.y + 5.0),
                    pos2((title_start_x + col_title_w - 4.0).max(title_start_x + 50.0), row_rect.max.y - 5.0),
                );
                ui.allocate_new_ui(UiBuilder::new().max_rect(lines_rect).layout(Layout::top_down(egui::Align::Min)), |lines_ui| {
                    lines_ui.spacing_mut().item_spacing = vec2(0.0, 3.0);

                    let title_color = if is_current { theme.accent } else { Color32::from_rgb(244, 244, 245) };
                    lines_ui.add(egui::Label::new(RichText::new(track.display_title()).size(13.5).color(title_color).strong()).truncate());

                    lines_ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing = vec2(5.0, 0.0);

                        // Official Spotify Green "SP" badge
                        pill_badge(
                            ui,
                            "SP",
                            Color32::from_rgba_unmultiplied(30, 215, 96, 28),
                            Color32::from_rgb(30, 215, 96),
                        );

                        // If matched in local library, display subtle lossless/local badge
                        if track.is_available_locally() {
                            format_badge(ui, "LOCAL", true, theme);
                        }

                        ui.add(egui::Label::new(RichText::new(track.display_artist()).size(11.5).color(Color32::from_rgb(161, 161, 170))).truncate());
                    });
                });

                // Col 2: ALBUM
                let album_rect = Rect::from_min_max(
                    pos2(row_rect.min.x + x2, row_rect.min.y),
                    pos2(row_rect.min.x + x2 + col_album_w, row_rect.max.y),
                );
                ui.allocate_new_ui(UiBuilder::new().max_rect(album_rect).layout(Layout::left_to_right(egui::Align::Center)), |album_ui| {
                    album_ui.add(egui::Label::new(RichText::new(track.display_album()).size(12.0).color(Color32::from_rgb(161, 161, 170))).truncate());
                });

                // Col 3: ADDED ("1 d")
                let added_rect = Rect::from_min_max(
                    pos2(row_rect.min.x + x3, row_rect.min.y),
                    pos2(row_rect.min.x + x3 + col_added_w, row_rect.max.y),
                );
                ui.allocate_new_ui(UiBuilder::new().max_rect(added_rect).layout(Layout::left_to_right(egui::Align::Center)), |added_ui| {
                    added_ui.label(RichText::new("1 d").size(12.0).color(Color32::from_rgb(161, 161, 170)));
                });

                // Col 4: PLAYS / Download arrow
                let plays_rect = Rect::from_min_max(
                    pos2(row_rect.min.x + x4, row_rect.min.y),
                    pos2(row_rect.min.x + x4 + col_plays_w, row_rect.max.y),
                );
                ui.allocate_new_ui(UiBuilder::new().max_rect(plays_rect).layout(Layout::left_to_right(egui::Align::Center)), |plays_ui| {
                    render_icon(plays_ui, Icon::Download, 13.0, theme.text_muted);
                });

                // Col 5: TIME
                let time_rect = Rect::from_min_max(
                    pos2(row_rect.min.x + x5, row_rect.min.y),
                    pos2(row_rect.max.x - 4.0, row_rect.max.y),
                );
                ui.allocate_new_ui(UiBuilder::new().max_rect(time_rect).layout(Layout::right_to_left(egui::Align::Center)), |time_ui| {
                    time_ui.label(RichText::new(track.formatted_duration()).size(12.0).color(Color32::from_rgb(161, 161, 170)));
                });

                if row_resp.hovered() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                }

                // Row Click / Double-Click Play Interaction
                if row_resp.double_clicked() || row_resp.clicked() {
                    state.selected_track_id = Some(track.id.clone());

                    // If track is already matched in local library, play local lossless file!
                    if track.is_available_locally() {
                        if let Some(ref path_str) = track.local_path {
                            let local_track = Track {
                                id: track.local_track_id.unwrap_or(0),
                                path: path_str.clone(),
                                title: track.title.clone(),
                                artist: track.artist.clone(),
                                album: track.album.clone(),
                                duration_secs: track.duration_secs,
                                ..Default::default()
                            };
                            player.load_queue(vec![local_track], 0);
                        }
                    } else {
                        // Stream via official embedded webview player
                        stream_controller.play_stream_track(track.clone());
                        if let Some(next_t) = tracks.get(idx + 1) {
                            stream_controller.prefetch_next_track(next_t.clone());
                        }
                    }
                }
            }

            ui.add_space(80.0);
        });
}

/// Render the Empty State Guide with clickable sample links
fn render_empty_guide(
    ui: &mut Ui,
    state: &mut StreamsViewState,
    stream_controller: &mut StreamController,
    theme: &Theme,
) {
    let guide_frame = theme.card_frame(14.0);
    guide_frame.show(ui, |ui| {
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing = vec2(0.0, 10.0);

            ui.horizontal(|ui| {
                render_icon(ui, Icon::Sparkle, 18.0, theme.accent);
                ui.add_space(6.0);
                ui.label(
                    RichText::new("Quick Import Guide — No Login Required")
                        .size(15.0)
                        .color(theme.text_primary)
                        .strong(),
                );
            });

            ui.label(
                RichText::new(
                    "Paste any public Spotify playlist, album, or track link into the bar above. \
                     Eclipse automatically parses the tracklist, links it to local library matches, \
                     and streams remote audio via official embedded playback with zero account sign-in.",
                )
                .size(12.5)
                .color(theme.text_secondary),
            );

            ui.add_space(6.0);
            ui.separator();
            ui.add_space(6.0);

            ui.label(
                RichText::new("SAMPLE LINKS (CLICK TO TEST)")
                    .size(11.0)
                    .color(theme.text_muted)
                    .strong(),
            );

            let samples = [
                ("Today's Top Hits (Playlist)", "https://open.spotify.com/playlist/37i9dQZF1DXcBWIGoYBM5M"),
                ("Random Access Memories (Album)", "https://open.spotify.com/album/4m2880jivSbbyEGAKfITCa"),
                ("Never Gonna Give You Up (Track)", "https://open.spotify.com/track/4cOdK2wGLETKBW3PvgPWqT"),
            ];

            for (label, sample_url) in samples {
                ui.horizontal(|ui| {
                    render_icon(ui, Icon::Music, 14.0, theme.text_muted);
                    ui.add_space(4.0);

                    let link_resp = ui.add(
                        egui::Button::new(
                            RichText::new(label)
                                .size(12.5)
                                .color(Color32::from_rgb(187, 247, 208))
                                .underline(),
                        )
                        .fill(Color32::TRANSPARENT)
                        .stroke(Stroke::NONE),
                    );

                    if link_resp.hovered() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    }

                    if link_resp.clicked() {
                        state.spotify_url_input = sample_url.to_string();
                        stream_controller.import_spotify_link(sample_url);
                    }

                    ui.label(
                        RichText::new(sample_url)
                            .size(11.5)
                            .color(theme.text_muted),
                    );
                });
            }
        });
    });
}

/// Render the Loading State Spinner Card
fn render_loading_state(ui: &mut Ui, theme: &Theme) {
    let load_frame = theme.card_frame(14.0);
    load_frame.show(ui, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(16.0);
            render_icon(ui, Icon::Hourglass, 24.0, theme.accent);
            ui.add_space(8.0);
            ui.label(
                RichText::new("Fetching Spotify metadata & resolving tracks...")
                    .size(14.5)
                    .color(theme.text_primary)
                    .strong(),
            );
            ui.label(
                RichText::new("Extracting public embed metadata off-thread without blocking the UI")
                    .size(12.0)
                    .color(theme.text_secondary),
            );
            ui.add_space(16.0);
        });
    });
}

/// Render Error Card with Retry Action
fn render_error_card(
    ui: &mut Ui,
    err_msg: &str,
    _theme: &Theme,
    state: &mut StreamsViewState,
    stream_controller: &mut StreamController,
) {
    let err_frame = egui::Frame::none()
        .fill(Color32::from_rgba_unmultiplied(239, 68, 68, 25))
        .stroke(Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(239, 68, 68, 60)))
        .rounding(Rounding::same(12.0))
        .inner_margin(egui::Margin::symmetric(16.0, 12.0));

    err_frame.show(ui, |ui| {
        ui.horizontal(|ui| {
            render_icon(ui, Icon::Info, 18.0, Color32::from_rgb(252, 165, 165));
            ui.add_space(8.0);
            ui.vertical(|ui| {
                ui.label(
                    RichText::new("Failed to import Spotify link")
                        .size(13.5)
                        .color(Color32::from_rgb(254, 202, 202))
                        .strong(),
                );
                ui.label(
                    RichText::new(err_msg)
                        .size(12.0)
                        .color(Color32::from_rgb(252, 165, 165)),
                );
            });

            ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                if dark_pill_button(ui, "Retry", Some(Icon::Refresh)).clicked() {
                    let url = state.spotify_url_input.trim().to_string();
                    if !url.is_empty() {
                        stream_controller.import_spotify_link(&url);
                    }
                }
            });
        });
    });
}

/// Render Truncation Notice Banner
fn render_truncation_notice(ui: &mut Ui, note: &str, _theme: &Theme) {
    let notice_frame = egui::Frame::none()
        .fill(Color32::from_rgba_unmultiplied(234, 179, 8, 22))
        .stroke(Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(234, 179, 8, 50)))
        .rounding(Rounding::same(10.0))
        .inner_margin(egui::Margin::symmetric(14.0, 8.0));

    notice_frame.show(ui, |ui| {
        ui.horizontal(|ui| {
            render_icon(ui, Icon::Info, 14.0, Color32::from_rgb(250, 204, 21));
            ui.add_space(6.0);
            ui.label(
                RichText::new(note)
                    .size(12.0)
                    .color(Color32::from_rgb(254, 240, 138)),
            );
            ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                pill_badge(
                    ui,
                    "PUBLIC EMBED LIMIT",
                    Color32::from_rgba_unmultiplied(234, 179, 8, 30),
                    Color32::from_rgb(250, 204, 21),
                );
            });
        });
    });
}

/// Render the YouTube Music search and playback view.
pub fn render_youtube_view(
    ui: &mut Ui,
    stream_controller: &mut StreamController,
    _player: &mut PlayerController,
    theme: &Theme,
    state: &mut StreamsViewState,
    on_navigate: &mut Option<NavTab>,
) {
    let has_api_key = CredentialStore::get_youtube_api_key()
        .ok()
        .flatten()
        .map(|k| !k.trim().is_empty())
        .unwrap_or(false);

    ScrollArea::vertical()
        .id_salt("youtube_main_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing = vec2(0.0, 14.0);

            // =================================================================
            // 1. HEADER GROUP
            // =================================================================
            let header_frame = theme.glass_frame(14.0);
            header_frame.show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing = vec2(0.0, 12.0);

                    ui.horizontal(|ui| {
                        let (card_rect, _) = ui.allocate_exact_size(vec2(68.0, 68.0), Sense::hover());
                        ui.painter().rect_filled(
                            card_rect,
                            Rounding::same(14.0),
                            Color32::from_rgb(20, 20, 24),
                        );
                        ui.painter().rect_stroke(
                            card_rect,
                            Rounding::same(14.0),
                            Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(255, 255, 255, 20)),
                        );
                        let icon_box = Rect::from_center_size(card_rect.center(), vec2(36.0, 36.0));
                        paint_icon(ui.painter(), Icon::YouTube, icon_box, Color32::from_rgb(255, 60, 60));

                        ui.add_space(14.0);

                        ui.vertical(|ui| {
                            ui.spacing_mut().item_spacing = vec2(0.0, 2.0);

                            ui.label(
                                RichText::new("STREAM")
                                    .size(10.0)
                                    .color(theme.text_muted)
                                    .strong(),
                            );

                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new("YouTube Music")
                                        .size(29.0)
                                        .color(theme.text_primary)
                                        .strong(),
                                );
                                pill_badge(
                                    ui,
                                    "OFFICIAL DATA API V3",
                                    Color32::from_rgba_unmultiplied(255, 60, 60, 30),
                                    Color32::from_rgb(255, 100, 100),
                                );
                            });

                            ui.label(
                                RichText::new(
                                    "Search music and play official embedded streams via YouTube Data API v3 and Windows WebView2.",
                                )
                                .size(12.5)
                                .color(theme.text_secondary),
                            );
                        });
                    });

                    ui.add_space(4.0);

                    // If API Key missing: show inline banner with link to Settings > Streaming
                    if !has_api_key {
                        let notice_frame = egui::Frame::none()
                            .fill(Color32::from_rgba_unmultiplied(234, 179, 8, 25))
                            .stroke(Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(234, 179, 8, 60)))
                            .rounding(Rounding::same(10.0))
                            .inner_margin(egui::Margin::symmetric(14.0, 10.0));

                        notice_frame.show(ui, |ui| {
                            ui.horizontal(|ui| {
                                render_icon(ui, Icon::Info, 16.0, Color32::from_rgb(250, 204, 21));
                                ui.add_space(8.0);
                                ui.vertical(|ui| {
                                    ui.label(
                                        RichText::new("YouTube Data API v3 key not configured")
                                            .size(13.0)
                                            .color(theme.text_primary)
                                            .strong(),
                                    );
                                    ui.label(
                                        RichText::new(
                                            "Add your free YouTube Data API v3 key in Settings > Streaming to enable full search and track matching.",
                                        )
                                        .size(12.0)
                                        .color(theme.text_secondary),
                                    );
                                });

                                ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                                    if white_pill_button(ui, "Settings > Streaming", Some(Icon::Settings)).clicked() {
                                        *on_navigate = Some(NavTab::Settings);
                                    }
                                });
                            });
                        });
                    } else {
                        // Search bar
                        let input_bar_frame = egui::Frame::none()
                            .fill(Color32::from_rgb(22, 22, 26))
                            .stroke(Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(255, 255, 255, 18)))
                            .rounding(Rounding::same(12.0))
                            .inner_margin(egui::Margin::symmetric(14.0, 8.0));

                        input_bar_frame.show(ui, |ui| {
                            ui.horizontal(|ui| {
                                render_icon(ui, Icon::Search, 16.0, theme.text_muted);
                                ui.add_space(8.0);

                                let text_w = (ui.available_width() - 175.0).max(120.0);
                                let edit_resp = ui.add(
                                    TextEdit::singleline(&mut state.youtube_search_query)
                                        .hint_text("Search music videos, songs, or artists on YouTube...")
                                        .desired_width(text_w)
                                        .frame(false)
                                        .margin(egui::Margin::symmetric(0.0, 4.0))
                                        .text_color(theme.text_primary),
                                );

                                pill_badge(
                                    ui,
                                    "ENTER",
                                    Color32::from_rgba_premultiplied(255, 255, 255, 12),
                                    theme.text_muted,
                                );

                                ui.add_space(8.0);

                                let do_search = edit_resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                                if (white_pill_button(ui, "Search", Some(Icon::Search)).clicked() || do_search)
                                    && !state.youtube_search_query.trim().is_empty()
                                {
                                    let q = state.youtube_search_query.trim().to_string();
                                    stream_controller.search("youtube", &q);
                                }
                            });
                        });
                    }
                });
            });

            // =================================================================
            // 2. YOUTUBE RESULTS LIST
            // =================================================================
            if let Some(results) = stream_controller.search_results.get("youtube").cloned() {
                if !results.is_empty() {
                    let results_frame = theme.card_frame(14.0);
                    results_frame.show(ui, |ui| {
                        ui.vertical(|ui| {
                            ui.spacing_mut().item_spacing = vec2(0.0, 8.0);

                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new("Search Results")
                                        .size(16.0)
                                        .color(theme.text_primary)
                                        .strong(),
                                );
                                ui.label(
                                    RichText::new(format!("· {} tracks found", results.len()))
                                        .size(12.5)
                                        .color(theme.text_muted),
                                );
                            });

                            ui.separator();

                            for track in results {
                                let (row_rect, row_resp) = ui.allocate_exact_size(
                                    vec2(ui.available_width(), 44.0),
                                    Sense::click(),
                                );
                                let bg = if row_resp.hovered() {
                                    Color32::from_rgb(28, 28, 31)
                                } else {
                                    Color32::TRANSPARENT
                                };
                                if bg != Color32::TRANSPARENT {
                                    ui.painter().rect_filled(row_rect, Rounding::same(6.0), bg);
                                }

                                ui.allocate_new_ui(UiBuilder::new().max_rect(row_rect).layout(Layout::left_to_right(egui::Align::Center)), |row_ui| {
                                    row_ui.add_space(8.0);
                                    render_icon(row_ui, Icon::YouTube, 16.0, Color32::from_rgb(255, 60, 60));
                                    row_ui.add_space(8.0);

                                    row_ui.vertical(|ui| {
                                        ui.spacing_mut().item_spacing = vec2(0.0, 2.0);
                                        ui.label(RichText::new(track.display_title()).size(13.0).color(theme.text_primary).strong());
                                        ui.label(RichText::new(track.display_artist()).size(11.0).color(theme.text_muted));
                                    });

                                    row_ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                                        ui.add_space(8.0);
                                        if dark_pill_button(ui, "Play", Some(Icon::Play)).clicked() {
                                            stream_controller.play_stream_track(track.clone());
                                        }
                                        ui.add_space(8.0);
                                        ui.label(RichText::new(track.formatted_duration()).size(11.5).color(theme.text_muted));
                                    });
                                });

                                if row_resp.hovered() {
                                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                                }
                                if row_resp.clicked() {
                                    stream_controller.play_stream_track(track.clone());
                                }
                            }
                        });
                    });
                }
            }

            ui.add_space(32.0);
        });
}

fn render_status_banner(ui: &mut Ui, message: &str, is_error: bool, _theme: &Theme) {
    let (bg, stroke, text_color, icon) = if is_error {
        (
            Color32::from_rgba_unmultiplied(239, 68, 68, 25),
            Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(239, 68, 68, 60)),
            Color32::from_rgb(252, 165, 165),
            Icon::Info,
        )
    } else {
        (
            Color32::from_rgba_unmultiplied(34, 197, 94, 25),
            Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(34, 197, 94, 60)),
            Color32::from_rgb(187, 247, 208),
            Icon::Sparkle,
        )
    };

    let frame = egui::Frame::none()
        .fill(bg)
        .stroke(stroke)
        .rounding(Rounding::same(8.0))
        .inner_margin(egui::Margin::symmetric(12.0, 8.0));

    frame.show(ui, |ui| {
        ui.horizontal(|ui| {
            render_icon(ui, icon, 14.0, text_color);
            ui.add_space(6.0);
            ui.label(RichText::new(message).size(12.0).color(text_color));
        });
    });
}

pub fn truncate_str(s: &str, max_chars: usize) -> String {
    if max_chars == 0 {
        return String::new();
    }
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

    #[test]
    fn test_truncate_str_boundary() {
        assert_eq!(truncate_str("Hello World", 5), "Hell…");
        assert_eq!(truncate_str("Hello", 10), "Hello");
        assert_eq!(truncate_str("", 5), "");
    }

    #[test]
    fn test_streams_view_state_defaults() {
        let state = StreamsViewState::default();
        assert!(state.spotify_url_input.is_empty());
        assert!(state.youtube_search_query.is_empty());
        assert!(state.status_msg.is_none());
        assert!(state.selected_track_id.is_none());
    }

    #[test]
    fn test_render_spotify_view_smoke() {
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let mut ctrl = StreamController::new(vec![]);
                let cfg = crate::config::AppConfig::default();
                let mut player = PlayerController::new(&cfg);
                let theme = Theme::from_name("Eclipse", None);
                let mut state = StreamsViewState::default();
                let mut art_manager = AlbumArtManager::new();
                let mut on_navigate = None;

                render_spotify_view(
                    ui,
                    &mut ctrl,
                    &mut player,
                    &theme,
                    &mut state,
                    &mut art_manager,
                    &mut on_navigate,
                );
            });
        });
    }

    #[test]
    fn test_render_youtube_view_smoke() {
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let mut ctrl = StreamController::new(vec![]);
                let cfg = crate::config::AppConfig::default();
                let mut player = PlayerController::new(&cfg);
                let theme = Theme::from_name("Eclipse", None);
                let mut state = StreamsViewState::default();
                let mut on_navigate = None;

                render_youtube_view(
                    ui,
                    &mut ctrl,
                    &mut player,
                    &theme,
                    &mut state,
                    &mut on_navigate,
                );
            });
        });
    }
}
