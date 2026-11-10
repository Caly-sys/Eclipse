//! Stream Views for Eclipse (Spotify & YouTube Music).
//! Implements Spotify Connect browsing/playback and YouTube Music search/embedded playback
//! matching the signature Eclipse purple glass aesthetic and pure vector icons.

use crate::audio::PlayerController;
use crate::streams::controller::StreamController;
use crate::streams::credentials::CredentialStore;
use crate::streams::provider::StreamTrack;
use crate::streams::spotify::DEFAULT_REDIRECT_URI;
use crate::ui::components::{dark_pill_button, pill_badge, white_pill_button};
use crate::ui::icons::{paint_icon, render_icon, Icon};
use crate::ui::theme::Theme;
use egui::{
    pos2, vec2, Color32, Layout, Margin, Rect, RichText, Rounding, ScrollArea, Stroke, TextEdit, Ui,
};

#[derive(Debug, Clone, Default)]
pub struct StreamsViewState {
    pub spotify_client_id_input: String,
    pub spotify_search_query: String,
    pub spotify_status_msg: Option<(String, bool)>, // (message, is_error)
    pub spotify_selected_playlist_id: Option<String>,

    pub youtube_api_key_input: String,
    pub youtube_search_query: String,
    pub youtube_status_msg: Option<(String, bool)>,
    pub youtube_selected_playlist_id: Option<String>,
}

/// Render the official Spotify Connect & Web API view.
pub fn render_spotify_view(
    ui: &mut Ui,
    stream_controller: &mut StreamController,
    player: &mut PlayerController,
    theme: &Theme,
    state: &mut StreamsViewState,
) {
    let is_connected = stream_controller.is_connected("spotify");

    ScrollArea::vertical()
        .id_salt("spotify_main_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing = vec2(0.0, 14.0);

            if is_connected {
                render_spotify_connected(ui, stream_controller, player, theme, state);
            } else {
                render_spotify_setup(ui, stream_controller, theme, state);
            }
        });
}

/// Render the official YouTube Music Data API v3 & embedded playback view.
pub fn render_youtube_view(
    ui: &mut Ui,
    stream_controller: &mut StreamController,
    player: &mut PlayerController,
    theme: &Theme,
    state: &mut StreamsViewState,
) {
    let is_connected = stream_controller.is_connected("youtube");

    ScrollArea::vertical()
        .id_salt("youtube_main_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing = vec2(0.0, 14.0);

            if is_connected {
                render_youtube_connected(ui, stream_controller, player, theme, state);
            } else {
                render_youtube_setup(ui, stream_controller, theme, state);
            }
        });
}

// =============================================================================
// SPOTIFY SUB-VIEWS: Setup & Connected
// =============================================================================

fn render_spotify_setup(
    ui: &mut Ui,
    stream_controller: &mut StreamController,
    theme: &Theme,
    state: &mut StreamsViewState,
) {
    // If input is empty, attempt to prefill from Windows Credential Manager
    if state.spotify_client_id_input.is_empty() {
        if let Ok(Some(saved_id)) = CredentialStore::get_spotify_client_id() {
            state.spotify_client_id_input = saved_id;
        }
    }

    let card_frame = theme.glass_frame(14.0);
    card_frame.show(ui, |ui| {
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing = vec2(0.0, 10.0);

            // Hero Header
            ui.horizontal(|ui| {
                let (icon_rect, _) = ui.allocate_exact_size(vec2(28.0, 28.0), egui::Sense::hover());
                paint_icon(ui.painter(), Icon::Spotify, icon_rect, Color32::from_rgb(30, 215, 96));
                ui.add_space(8.0);
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new("Connect to Spotify")
                            .size(19.0)
                            .color(theme.text_primary)
                            .strong(),
                    );
                    ui.label(
                        RichText::new("Stream Spotify tracks and playlists directly via Spotify Connect.")
                            .size(12.0)
                            .color(theme.text_secondary),
                    );
                });
            });

            ui.add_space(6.0);
            ui.separator();
            ui.add_space(6.0);

            // Instructions Box
            let step_card = theme.card_frame(10.0);
            step_card.show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing = vec2(0.0, 8.0);
                    ui.label(
                        RichText::new("SETUP INSTRUCTIONS (RFC 7636 PKCE)")
                            .size(11.0)
                            .color(theme.text_muted)
                            .strong(),
                    );

                    step_item(
                        ui,
                        "1",
                        "Open the Spotify Developer Dashboard and log in or create an application.",
                        theme,
                    );
                    step_item(
                        ui,
                        "2",
                        &format!("In App Settings, set the Redirect URI to: {}", DEFAULT_REDIRECT_URI),
                        theme,
                    );
                    step_item(
                        ui,
                        "3",
                        "Copy your Spotify Client ID and paste it into the field below.",
                        theme,
                    );

                    ui.add_space(4.0);
                    if dark_pill_button(ui, "Open Spotify Developer Dashboard", Some(Icon::Expand)).clicked() {
                        stream_controller.open_browser_url("https://developer.spotify.com/dashboard");
                    }
                });
            });

            ui.add_space(8.0);

            // Client ID Input
            ui.label(
                RichText::new("SPOTIFY CLIENT ID")
                    .size(11.0)
                    .color(theme.text_muted)
                    .strong(),
            );

            let input_resp = ui.add(
                TextEdit::singleline(&mut state.spotify_client_id_input)
                    .hint_text("Paste your Spotify Client ID (e.g. 7d8f...)")
                    .desired_width(ui.available_width())
                    .margin(Margin::symmetric(10.0, 8.0)),
            );
            if input_resp.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::Text);
            }

            ui.add_space(4.0);

            // Action Buttons & Status
            ui.horizontal(|ui| {
                let connect_btn = white_pill_button(ui, "Connect with Spotify", Some(Icon::Spotify));
                if connect_btn.clicked() {
                    let client_id = state.spotify_client_id_input.trim();
                    if client_id.is_empty() {
                        state.spotify_status_msg = Some(("Please enter your Spotify Client ID".to_string(), true));
                    } else {
                        if let Err(e) = CredentialStore::save_spotify_client_id(client_id) {
                            state.spotify_status_msg = Some((format!("Failed to save Client ID: {e}"), true));
                        } else {
                            state.spotify_status_msg = Some((
                                "Opening Spotify authorization in your browser...".to_string(),
                                false,
                            ));
                            stream_controller.connect("spotify");
                        }
                    }
                }

                if dark_pill_button(ui, "Clear Credentials", Some(Icon::Trash)).clicked() {
                    let _ = CredentialStore::clear_spotify_credentials();
                    state.spotify_client_id_input.clear();
                    state.spotify_status_msg = Some(("Credentials cleared from Windows vault.".to_string(), false));
                }
            });

            // Loading state
            if stream_controller.is_busy {
                ui.add_space(6.0);
                render_loading_card(ui, "Connecting to Spotify & waiting for authorization...", theme);
            }

            // Error or status notification
            if let Some((ref msg, is_err)) = state.spotify_status_msg {
                ui.add_space(6.0);
                render_status_banner(ui, msg, is_err, theme);
            } else if let Some(ref err) = stream_controller.last_error {
                ui.add_space(6.0);
                render_status_banner(ui, err, true, theme);
            }
        });
    });
}

fn render_spotify_connected(
    ui: &mut Ui,
    stream_controller: &mut StreamController,
    player: &mut PlayerController,
    theme: &Theme,
    state: &mut StreamsViewState,
) {
    // 1. Top Status & Transport Header Card
    let header_frame = theme.glass_frame(12.0);
    header_frame.show(ui, |ui| {
        ui.horizontal(|ui| {
            let (icon_rect, _) = ui.allocate_exact_size(vec2(24.0, 24.0), egui::Sense::hover());
            paint_icon(ui.painter(), Icon::Spotify, icon_rect, Color32::from_rgb(30, 215, 96));
            ui.add_space(8.0);

            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("Spotify Connected")
                            .size(16.0)
                            .color(theme.text_primary)
                            .strong(),
                    );
                    pill_badge(
                        ui,
                        "OFFICIAL CONNECT",
                        Color32::from_rgba_unmultiplied(30, 215, 96, 35),
                        Color32::from_rgb(30, 215, 96),
                    );
                });

                let active_dev = stream_controller
                    .active_state
                    .as_ref()
                    .and_then(|s| s.device_name.as_ref())
                    .cloned()
                    .unwrap_or_else(|| "Connect Device".to_string());

                ui.label(
                    RichText::new(format!("Active Device: {}", active_dev))
                        .size(11.5)
                        .color(theme.text_secondary),
                );
            });

            ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                if dark_pill_button(ui, "Disconnect", Some(Icon::Close)).clicked() {
                    stream_controller.disconnect("spotify");
                    state.spotify_selected_playlist_id = None;
                    if player.playback_source.is_stream() {
                        player.switch_to_local();
                    }
                }

                if dark_pill_button(ui, "Refresh Playlists", Some(Icon::Refresh)).clicked() {
                    stream_controller.fetch_playlists("spotify");
                }
            });
        });
    });

    // 2. Search Card
    let search_card = theme.card_frame(10.0);
    search_card.show(ui, |ui| {
        ui.horizontal(|ui| {
            render_icon(ui, Icon::Search, 15.0, theme.accent);
            ui.add_space(4.0);

            let edit_resp = ui.add(
                TextEdit::singleline(&mut state.spotify_search_query)
                    .hint_text("Search songs, artists, or albums on Spotify...")
                    .desired_width(ui.available_width() - 170.0),
            );

            let do_search = edit_resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
            if white_pill_button(ui, "Search", Some(Icon::Search)).clicked() || do_search {
                let q = state.spotify_search_query.trim().to_string();
                if !q.is_empty() {
                    state.spotify_selected_playlist_id = None;
                    stream_controller.search("spotify", &q);
                }
            }

            if !state.spotify_search_query.is_empty()
                && dark_pill_button(ui, "Clear", Some(Icon::Close)).clicked()
            {
                state.spotify_search_query.clear();
            }
        });
    });

    // 3. User Playlists Shelf (Horizontal Scroll Area)
    render_playlists_shelf(
        ui,
        "spotify",
        stream_controller,
        &mut state.spotify_selected_playlist_id,
        theme,
    );

    // 4. Tracks Content View (Search Results or Selected Playlist Tracks)
    let tracks_to_show = if let Some(ref pl_id) = state.spotify_selected_playlist_id {
        stream_controller
            .playlist_tracks
            .get(&("spotify".to_string(), pl_id.clone()))
            .cloned()
    } else {
        stream_controller.search_results.get("spotify").cloned()
    };

    let title_prefix = if state.spotify_selected_playlist_id.is_some() {
        "PLAYLIST TRACKS"
    } else if !state.spotify_search_query.is_empty() {
        "SEARCH RESULTS"
    } else {
        "DISCOVER TRACKS"
    };

    ui.horizontal(|ui| {
        ui.label(
            RichText::new(title_prefix)
                .size(11.5)
                .color(theme.text_muted)
                .strong(),
        );

        if state.spotify_selected_playlist_id.is_some() {
            ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                if dark_pill_button(ui, "Show Search Results", Some(Icon::ChevronLeft)).clicked() {
                    state.spotify_selected_playlist_id = None;
                }
            });
        }
    });

    if stream_controller.is_busy {
        render_loading_card(ui, "Fetching tracks from Spotify...", theme);
    } else if let Some(ref tracks) = tracks_to_show {
        if tracks.is_empty() {
            render_empty_state_card(
                ui,
                "No tracks found",
                "Try searching for another song, artist, or album.",
                Icon::Search,
                theme,
            );
        } else {
            render_stream_tracks_table(ui, tracks, "spotify", stream_controller, player, theme);
        }
    } else {
        render_empty_state_card(
            ui,
            "Ready to play",
            "Type a song title above or choose one of your playlists to begin streaming.",
            Icon::Music,
            theme,
        );
    }
}

// =============================================================================
// YOUTUBE MUSIC SUB-VIEWS: Setup & Connected
// =============================================================================

fn render_youtube_setup(
    ui: &mut Ui,
    stream_controller: &mut StreamController,
    theme: &Theme,
    state: &mut StreamsViewState,
) {
    if state.youtube_api_key_input.is_empty() {
        if let Ok(Some(saved_key)) = CredentialStore::get_youtube_api_key() {
            state.youtube_api_key_input = saved_key;
        }
    }

    let card_frame = theme.glass_frame(14.0);
    card_frame.show(ui, |ui| {
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing = vec2(0.0, 10.0);

            // Hero Header
            ui.horizontal(|ui| {
                let (icon_rect, _) = ui.allocate_exact_size(vec2(28.0, 28.0), egui::Sense::hover());
                paint_icon(ui.painter(), Icon::YouTube, icon_rect, Color32::from_rgb(255, 60, 60));
                ui.add_space(8.0);
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new("Connect to YouTube Music")
                            .size(19.0)
                            .color(theme.text_primary)
                            .strong(),
                    );
                    ui.label(
                        RichText::new("Search music and play official embedded streams via YouTube Data API v3.")
                            .size(12.0)
                            .color(theme.text_secondary),
                    );
                });
            });

            ui.add_space(6.0);
            ui.separator();
            ui.add_space(6.0);

            // Instructions Box
            let step_card = theme.card_frame(10.0);
            step_card.show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing = vec2(0.0, 8.0);
                    ui.label(
                        RichText::new("SETUP INSTRUCTIONS (OFFICIAL DATA API V3)")
                            .size(11.0)
                            .color(theme.text_muted)
                            .strong(),
                    );

                    step_item(
                        ui,
                        "1",
                        "Open the Google Cloud Console and select or create a project.",
                        theme,
                    );
                    step_item(
                        ui,
                        "2",
                        "In the API Library, enable the 'YouTube Data API v3'.",
                        theme,
                    );
                    step_item(
                        ui,
                        "3",
                        "Create an API Key under Credentials and paste it below.",
                        theme,
                    );

                    ui.add_space(4.0);
                    if dark_pill_button(ui, "Open Google Cloud Console", Some(Icon::Expand)).clicked() {
                        stream_controller.open_browser_url(
                            "https://console.cloud.google.com/apis/library/youtube.googleapis.com",
                        );
                    }
                });
            });

            ui.add_space(8.0);

            // API Key Input (Masked for privacy)
            ui.label(
                RichText::new("GOOGLE CLOUD YOUTUBE API KEY")
                    .size(11.0)
                    .color(theme.text_muted)
                    .strong(),
            );

            let input_resp = ui.add(
                TextEdit::singleline(&mut state.youtube_api_key_input)
                    .password(true)
                    .hint_text("Paste your YouTube Data API v3 Key (e.g. AIzaSy...)")
                    .desired_width(ui.available_width())
                    .margin(Margin::symmetric(10.0, 8.0)),
            );
            if input_resp.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::Text);
            }

            ui.add_space(4.0);

            // Action Buttons & Status
            ui.horizontal(|ui| {
                let connect_btn = white_pill_button(ui, "Save API Key & Connect", Some(Icon::YouTube));
                if connect_btn.clicked() {
                    let key = state.youtube_api_key_input.trim();
                    if key.is_empty() {
                        state.youtube_status_msg = Some(("Please enter a YouTube API Key".to_string(), true));
                    } else {
                        if let Err(e) = CredentialStore::save_youtube_api_key(key) {
                            state.youtube_status_msg = Some((format!("Failed to save API Key: {e}"), true));
                        } else {
                            state.youtube_status_msg = Some((
                                "API Key saved. Connecting to YouTube Music...".to_string(),
                                false,
                            ));
                            stream_controller.connect("youtube");
                            stream_controller.fetch_playlists("youtube");
                        }
                    }
                }

                if dark_pill_button(ui, "Clear Key", Some(Icon::Trash)).clicked() {
                    let _ = CredentialStore::clear_youtube_credentials();
                    state.youtube_api_key_input.clear();
                    state.youtube_status_msg = Some(("API key cleared from Windows vault.".to_string(), false));
                }
            });

            // Loading state
            if stream_controller.is_busy {
                ui.add_space(6.0);
                render_loading_card(ui, "Validating YouTube API key...", theme);
            }

            // Error or status notification
            if let Some((ref msg, is_err)) = state.youtube_status_msg {
                ui.add_space(6.0);
                render_status_banner(ui, msg, is_err, theme);
            } else if let Some(ref err) = stream_controller.last_error {
                ui.add_space(6.0);
                render_status_banner(ui, err, true, theme);
            }
        });
    });
}

fn render_youtube_connected(
    ui: &mut Ui,
    stream_controller: &mut StreamController,
    player: &mut PlayerController,
    theme: &Theme,
    state: &mut StreamsViewState,
) {
    // 1. Top Status & Quota Notice Card
    let header_frame = theme.glass_frame(12.0);
    header_frame.show(ui, |ui| {
        ui.horizontal(|ui| {
            let (icon_rect, _) = ui.allocate_exact_size(vec2(24.0, 24.0), egui::Sense::hover());
            paint_icon(ui.painter(), Icon::YouTube, icon_rect, Color32::from_rgb(255, 60, 60));
            ui.add_space(8.0);

            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("YouTube Music Connected")
                            .size(16.0)
                            .color(theme.text_primary)
                            .strong(),
                    );
                    pill_badge(
                        ui,
                        "OFFICIAL EMBED (ADR-003)",
                        Color32::from_rgba_unmultiplied(255, 60, 60, 35),
                        Color32::from_rgb(255, 100, 100),
                    );
                });

                ui.label(
                    RichText::new("Daily Free Quota: 10,000 units/day · Zero scrapers")
                        .size(11.5)
                        .color(theme.text_secondary),
                );
            });

            ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                if dark_pill_button(ui, "Change Key / Disconnect", Some(Icon::Close)).clicked() {
                    stream_controller.disconnect("youtube");
                    state.youtube_selected_playlist_id = None;
                    if player.playback_source.is_stream() {
                        player.switch_to_local();
                    }
                }

                if dark_pill_button(ui, "Refresh Playlists", Some(Icon::Refresh)).clicked() {
                    stream_controller.fetch_playlists("youtube");
                }
            });
        });
    });

    // 2. Search Card
    let search_card = theme.card_frame(10.0);
    search_card.show(ui, |ui| {
        ui.horizontal(|ui| {
            render_icon(ui, Icon::Search, 15.0, theme.accent);
            ui.add_space(4.0);

            let edit_resp = ui.add(
                TextEdit::singleline(&mut state.youtube_search_query)
                    .hint_text("Search music videos, songs, or artists on YouTube...")
                    .desired_width(ui.available_width() - 170.0),
            );

            let do_search = edit_resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
            if white_pill_button(ui, "Search", Some(Icon::Search)).clicked() || do_search {
                let q = state.youtube_search_query.trim().to_string();
                if !q.is_empty() {
                    state.youtube_selected_playlist_id = None;
                    stream_controller.search("youtube", &q);
                }
            }

            if !state.youtube_search_query.is_empty()
                && dark_pill_button(ui, "Clear", Some(Icon::Close)).clicked()
            {
                state.youtube_search_query.clear();
            }
        });
    });

    // 3. Playlists Shelf (Curated Playlists)
    render_playlists_shelf(
        ui,
        "youtube",
        stream_controller,
        &mut state.youtube_selected_playlist_id,
        theme,
    );

    // 4. Tracks Content View (Search Results or Selected Playlist Tracks)
    let tracks_to_show = if let Some(ref pl_id) = state.youtube_selected_playlist_id {
        stream_controller
            .playlist_tracks
            .get(&("youtube".to_string(), pl_id.clone()))
            .cloned()
    } else {
        stream_controller.search_results.get("youtube").cloned()
    };

    let title_prefix = if state.youtube_selected_playlist_id.is_some() {
        "PLAYLIST TRACKS"
    } else if !state.youtube_search_query.is_empty() {
        "SEARCH RESULTS"
    } else {
        "DISCOVER MUSIC"
    };

    ui.horizontal(|ui| {
        ui.label(
            RichText::new(title_prefix)
                .size(11.5)
                .color(theme.text_muted)
                .strong(),
        );

        if state.youtube_selected_playlist_id.is_some() {
            ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                if dark_pill_button(ui, "Show Search Results", Some(Icon::ChevronLeft)).clicked() {
                    state.youtube_selected_playlist_id = None;
                }
            });
        }
    });

    if stream_controller.is_busy {
        render_loading_card(ui, "Fetching tracks from YouTube Data API...", theme);
    } else if let Some(ref tracks) = tracks_to_show {
        if tracks.is_empty() {
            render_empty_state_card(
                ui,
                "No music tracks found",
                "Try searching for another artist, video title, or music genre.",
                Icon::Search,
                theme,
            );
        } else {
            render_stream_tracks_table(ui, tracks, "youtube", stream_controller, player, theme);
        }
    } else {
        render_empty_state_card(
            ui,
            "Ready to play",
            "Type a song or video title above to search, or choose a curated playlist.",
            Icon::Music,
            theme,
        );
    }
}

// =============================================================================
// SHARED STREAM UI HELPERS: Shelf, Tables, Cards, Banners
// =============================================================================

fn render_playlists_shelf(
    ui: &mut Ui,
    provider_id: &str,
    stream_controller: &mut StreamController,
    selected_playlist_id: &mut Option<String>,
    theme: &Theme,
) {
    let playlists = stream_controller.playlists.get(provider_id).cloned();

    if let Some(ref list) = playlists {
        if !list.is_empty() {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("PLAYLISTS")
                            .size(11.5)
                            .color(theme.text_muted)
                            .strong(),
                    );
                    ui.label(
                        RichText::new(format!("({})", list.len()))
                            .size(11.0)
                            .color(theme.text_muted),
                    );
                });

                ScrollArea::horizontal()
                    .id_salt(format!("{provider_id}_playlists_shelf"))
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing = vec2(10.0, 0.0);

                            for pl in list {
                                let is_active = selected_playlist_id.as_deref() == Some(&pl.id);
                                let card_w = 170.0;
                                let card_h = 76.0;

                                let (rect, resp) =
                                    ui.allocate_exact_size(vec2(card_w, card_h), egui::Sense::click());
                                if resp.hovered() {
                                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                                }

                                let bg_color = if is_active {
                                    Color32::from_rgba_premultiplied(
                                        theme.accent.r(),
                                        theme.accent.g(),
                                        theme.accent.b(),
                                        26,
                                    )
                                } else if resp.hovered() {
                                    theme.surface_hover
                                } else {
                                    theme.card
                                };

                                let stroke = if is_active {
                                    Stroke::new(1.0_f32, theme.border_accent)
                                } else {
                                    Stroke::new(1.0_f32, theme.border)
                                };

                                ui.painter().rect_filled(rect, Rounding::same(10.0), bg_color);
                                ui.painter().rect_stroke(rect, Rounding::same(10.0), stroke);

                                // Inner card layout
                                let inner_rect = rect.shrink(10.0);
                                let icon_box = Rect::from_min_size(inner_rect.min, vec2(16.0, 16.0));
                                let icon_color = if is_active { theme.accent } else { theme.text_muted };
                                paint_icon(ui.painter(), Icon::Playlist, icon_box, icon_color);

                                let title_pos = pos2(inner_rect.min.x + 22.0, inner_rect.min.y + 1.0);
                                let truncated_name = truncate_str(&pl.name, 18);
                                ui.painter().text(
                                    title_pos,
                                    egui::Align2::LEFT_TOP,
                                    truncated_name,
                                    egui::FontId::proportional(12.5),
                                    if is_active { theme.text_primary } else { theme.text_secondary },
                                );

                                let count_str = if pl.track_count > 0 {
                                    format!("{} tracks", pl.track_count)
                                } else {
                                    "Playlist".to_string()
                                };
                                ui.painter().text(
                                    pos2(inner_rect.min.x + 22.0, inner_rect.min.y + 20.0),
                                    egui::Align2::LEFT_TOP,
                                    count_str,
                                    egui::FontId::proportional(11.0),
                                    theme.text_muted,
                                );

                                if resp.clicked() {
                                    *selected_playlist_id = Some(pl.id.clone());
                                    stream_controller.fetch_playlist_tracks(provider_id, &pl.id);
                                }
                            }
                        });
                    });
            });
        }
    }
}

fn render_stream_tracks_table(
    ui: &mut Ui,
    tracks: &[StreamTrack],
    provider_id: &str,
    stream_controller: &mut StreamController,
    player: &mut PlayerController,
    theme: &Theme,
) {
    let mut to_play: Option<StreamTrack> = None;

    ScrollArea::vertical()
        .id_salt(format!("{provider_id}_tracks_table_scroll"))
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing = vec2(0.0, 4.0);

            for (idx, track) in tracks.iter().enumerate() {
                let is_current = player
                    .active_stream_track
                    .as_ref()
                    .map(|t| t.id == track.id)
                    .unwrap_or(false)
                    && player.playback_source.is_stream();

                let row_h = 44.0;
                let avail_w = ui.available_width();
                let (rect, resp) = ui.allocate_exact_size(vec2(avail_w, row_h), egui::Sense::click());
                if resp.hovered() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                }

                let bg = if is_current {
                    Color32::from_rgba_premultiplied(
                        theme.accent.r(),
                        theme.accent.g(),
                        theme.accent.b(),
                        28,
                    )
                } else if resp.hovered() {
                    theme.surface_hover
                } else {
                    Color32::from_rgba_unmultiplied(255, 255, 255, 4)
                };

                let stroke = if is_current {
                    Stroke::new(1.0_f32, theme.border_accent)
                } else if resp.hovered() {
                    Stroke::new(1.0_f32, theme.border)
                } else {
                    Stroke::NONE
                };

                ui.painter().rect_filled(rect, Rounding::same(8.0), bg);
                if stroke != Stroke::NONE {
                    ui.painter().rect_stroke(rect, Rounding::same(8.0), stroke);
                }

                // Col 1: Play Icon or Index
                let play_rect = Rect::from_min_size(
                    pos2(rect.min.x + 12.0, rect.center().y - 8.0),
                    vec2(16.0, 16.0),
                );

                if resp.hovered() || is_current {
                    let play_icon = if is_current { Icon::Pause } else { Icon::Play };
                    let play_color = if is_current { theme.accent } else { theme.text_primary };
                    paint_icon(ui.painter(), play_icon, play_rect, play_color);
                } else {
                    ui.painter().text(
                        pos2(rect.min.x + 18.0, rect.center().y),
                        egui::Align2::CENTER_CENTER,
                        format!("{}", idx + 1),
                        egui::FontId::proportional(11.5),
                        theme.text_muted,
                    );
                }

                // Col 2: Title & Artist
                let title_x = rect.min.x + 46.0;
                let title_w = (avail_w * 0.44).max(120.0);
                let title_str = truncate_str(track.display_title(), (title_w / 7.5) as usize);

                let title_color = if is_current { theme.accent } else { theme.text_primary };
                ui.painter().text(
                    pos2(title_x, rect.center().y - 7.0),
                    egui::Align2::LEFT_CENTER,
                    title_str,
                    egui::FontId::proportional(13.0),
                    title_color,
                );

                let artist_str = truncate_str(track.display_artist(), (title_w / 8.0) as usize);
                ui.painter().text(
                    pos2(title_x, rect.center().y + 8.0),
                    egui::Align2::LEFT_CENTER,
                    artist_str,
                    egui::FontId::proportional(11.0),
                    theme.text_muted,
                );

                // Col 3: Album (Hidden on narrow widths)
                if avail_w > 540.0 {
                    let album_x = rect.min.x + title_w + 60.0;
                    let album_w = (avail_w * 0.28).max(80.0);
                    let album_str = truncate_str(track.display_album(), (album_w / 7.5) as usize);
                    ui.painter().text(
                        pos2(album_x, rect.center().y),
                        egui::Align2::LEFT_CENTER,
                        album_str,
                        egui::FontId::proportional(11.5),
                        theme.text_secondary,
                    );
                }

                // Col 4: Duration
                let dur_str = track.formatted_duration();
                ui.painter().text(
                    pos2(rect.max.x - 16.0, rect.center().y),
                    egui::Align2::RIGHT_CENTER,
                    dur_str,
                    egui::FontId::proportional(11.5),
                    theme.text_muted,
                );

                if resp.clicked() {
                    to_play = Some(track.clone());
                }
            }
        });

    if let Some(track) = to_play {
        stream_controller.play_track(provider_id, track.clone());
        player.switch_to_stream(provider_id, Some(track));
    }
}

fn step_item(ui: &mut Ui, number: &str, text: &str, theme: &Theme) {
    ui.horizontal(|ui| {
        let (num_rect, _) = ui.allocate_exact_size(vec2(18.0, 18.0), egui::Sense::hover());
        ui.painter().circle_filled(
            num_rect.center(),
            9.0,
            Color32::from_rgba_premultiplied(theme.accent.r(), theme.accent.g(), theme.accent.b(), 35),
        );
        ui.painter().text(
            num_rect.center(),
            egui::Align2::CENTER_CENTER,
            number,
            egui::FontId::proportional(10.5),
            theme.accent,
        );

        ui.add_space(6.0);
        ui.label(RichText::new(text).size(12.0).color(theme.text_secondary));
    });
}

fn render_loading_card(ui: &mut Ui, message: &str, theme: &Theme) {
    let frame = theme.card_frame(10.0);
    frame.show(ui, |ui| {
        ui.horizontal(|ui| {
            render_icon(ui, Icon::Hourglass, 14.0, theme.accent);
            ui.add_space(6.0);
            ui.label(RichText::new(message).size(12.5).color(theme.text_primary));
        });
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
        .inner_margin(Margin::symmetric(12.0, 8.0));

    frame.show(ui, |ui| {
        ui.horizontal(|ui| {
            render_icon(ui, icon, 14.0, text_color);
            ui.add_space(6.0);
            ui.label(RichText::new(message).size(12.0).color(text_color));
        });
    });
}

fn render_empty_state_card(
    ui: &mut Ui,
    title: &str,
    description: &str,
    icon: Icon,
    theme: &Theme,
) {
    let frame = theme.card_frame(12.0);
    frame.show(ui, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(20.0);
            let (icon_rect, _) = ui.allocate_exact_size(vec2(32.0, 32.0), egui::Sense::hover());
            paint_icon(ui.painter(), icon, icon_rect, theme.text_muted);
            ui.add_space(8.0);
            ui.label(RichText::new(title).size(15.0).color(theme.text_primary).strong());
            ui.add_space(4.0);
            ui.label(RichText::new(description).size(12.0).color(theme.text_muted));
            ui.add_space(20.0);
        });
    });
}

fn truncate_str(s: &str, max_chars: usize) -> String {
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
        assert!(state.spotify_client_id_input.is_empty());
        assert!(state.spotify_search_query.is_empty());
        assert!(state.spotify_selected_playlist_id.is_none());
        assert!(state.youtube_api_key_input.is_empty());
        assert!(state.youtube_search_query.is_empty());
        assert!(state.youtube_selected_playlist_id.is_none());
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

                render_spotify_view(ui, &mut ctrl, &mut player, &theme, &mut state);
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

                render_youtube_view(ui, &mut ctrl, &mut player, &theme, &mut state);
            });
        });
    }
}
