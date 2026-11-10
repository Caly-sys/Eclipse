//! Stream Views for Eclipse (Spotify & YouTube Music).
//! Clean minimal placeholder views adhering to the Eclipse purple glass aesthetic and pure vector icons.
//! Full public Spotify link importer UI and YouTube search are implemented in subsequent tasks (TASK-R2..R5).

use crate::audio::PlayerController;
use crate::streams::controller::StreamController;
use crate::streams::credentials::CredentialStore;
use crate::ui::components::{dark_pill_button, pill_badge, white_pill_button};
use crate::ui::icons::{paint_icon, render_icon, Icon};
use crate::ui::theme::Theme;
use egui::{
    vec2, Color32, Margin, RichText, ScrollArea, TextEdit, Ui,
};

#[derive(Debug, Clone, Default)]
pub struct StreamsViewState {
    pub spotify_url_input: String,
    pub youtube_search_query: String,
    pub status_msg: Option<(String, bool)>, // (message, is_error)
}

/// Render the Spotify Public Link Importer view placeholder (zero login required).
pub fn render_spotify_view(
    ui: &mut Ui,
    _stream_controller: &mut StreamController,
    _player: &mut PlayerController,
    theme: &Theme,
    state: &mut StreamsViewState,
) {
    ScrollArea::vertical()
        .id_salt("spotify_main_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing = vec2(0.0, 14.0);

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
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new("Spotify Link Importer")
                                        .size(19.0)
                                        .color(theme.text_primary)
                                        .strong(),
                                );
                                pill_badge(
                                    ui,
                                    "PUBLIC IMPORT",
                                    Color32::from_rgba_unmultiplied(30, 215, 96, 35),
                                    Color32::from_rgb(30, 215, 96),
                                );
                            });
                            ui.label(
                                RichText::new(
                                    "Paste a public Spotify track, album or playlist link. Save it as a playlist, stream it (matched from YouTube — Spotify doesn't allow third-party streaming).",
                                )
                                .size(12.0)
                                .color(theme.text_secondary),
                            );
                        });
                    });

                    ui.add_space(6.0);
                    ui.separator();
                    ui.add_space(6.0);

                    // Paste / Input Bar
                    ui.label(
                        RichText::new("PASTE SPOTIFY URL OR URI")
                            .size(11.0)
                            .color(theme.text_muted)
                            .strong(),
                    );

                    ui.horizontal(|ui| {
                        let input_resp = ui.add(
                            TextEdit::singleline(&mut state.spotify_url_input)
                                .hint_text("https://open.spotify.com/playlist/... or spotify:track:...")
                                .desired_width(ui.available_width() - 120.0)
                                .margin(Margin::symmetric(10.0, 8.0)),
                        );
                        if input_resp.hovered() {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::Text);
                        }

                        let import_btn = white_pill_button(ui, "Import", Some(Icon::Search));
                        if import_btn.clicked() && !state.spotify_url_input.trim().is_empty() {
                            state.status_msg = Some((
                                "Spotify public metadata importer ready for TASK-R2.".to_string(),
                                false,
                            ));
                        }
                    });

                    if let Some((ref msg, is_err)) = state.status_msg {
                        ui.add_space(4.0);
                        render_status_banner(ui, msg, is_err, theme);
                    }
                });
            });
        });
}

/// Render the YouTube Music search and playback view placeholder.
pub fn render_youtube_view(
    ui: &mut Ui,
    stream_controller: &mut StreamController,
    _player: &mut PlayerController,
    theme: &Theme,
    state: &mut StreamsViewState,
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

            let header_frame = theme.glass_frame(14.0);
            header_frame.show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing = vec2(0.0, 10.0);

                    // Hero Header
                    ui.horizontal(|ui| {
                        let (icon_rect, _) = ui.allocate_exact_size(vec2(28.0, 28.0), egui::Sense::hover());
                        paint_icon(ui.painter(), Icon::YouTube, icon_rect, Color32::from_rgb(255, 60, 60));
                        ui.add_space(8.0);
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new("YouTube Music")
                                        .size(19.0)
                                        .color(theme.text_primary)
                                        .strong(),
                                );
                                pill_badge(
                                    ui,
                                    "OFFICIAL DATA API V3",
                                    Color32::from_rgba_unmultiplied(255, 60, 60, 35),
                                    Color32::from_rgb(255, 100, 100),
                                );
                            });
                            ui.label(
                                RichText::new(
                                    "Search music and play official embedded streams via YouTube Data API v3.",
                                )
                                .size(12.0)
                                .color(theme.text_secondary),
                            );
                        });
                    });

                    ui.add_space(6.0);
                    ui.separator();
                    ui.add_space(6.0);

                    if !has_api_key {
                        let notice_frame = theme.card_frame(10.0);
                        notice_frame.show(ui, |ui| {
                            ui.horizontal(|ui| {
                                render_icon(ui, Icon::Info, 16.0, theme.accent);
                                ui.add_space(6.0);
                                ui.vertical(|ui| {
                                    ui.label(
                                        RichText::new("YouTube Data API v3 key not configured")
                                            .size(13.0)
                                            .color(theme.text_primary)
                                            .strong(),
                                    );
                                    ui.label(
                                        RichText::new(
                                            "Add your free YouTube Data API v3 key in Settings > Streaming to enable search and track matching.",
                                        )
                                        .size(12.0)
                                        .color(theme.text_secondary),
                                    );
                                });
                            });
                        });
                    } else {
                        // Search bar
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
                                    stream_controller.search("youtube", &q);
                                }
                            }

                            if !state.youtube_search_query.is_empty()
                                && dark_pill_button(ui, "Clear", Some(Icon::Close)).clicked()
                            {
                                state.youtube_search_query.clear();
                            }
                        });
                    }
                });
            });
        });
}

fn render_status_banner(ui: &mut Ui, message: &str, is_error: bool, _theme: &Theme) {
    let (bg, stroke, text_color, icon) = if is_error {
        (
            Color32::from_rgba_unmultiplied(239, 68, 68, 25),
            egui::Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(239, 68, 68, 60)),
            Color32::from_rgb(252, 165, 165),
            Icon::Info,
        )
    } else {
        (
            Color32::from_rgba_unmultiplied(34, 197, 94, 25),
            egui::Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(34, 197, 94, 60)),
            Color32::from_rgb(187, 247, 208),
            Icon::Sparkle,
        )
    };

    let frame = egui::Frame::none()
        .fill(bg)
        .stroke(stroke)
        .rounding(egui::Rounding::same(8.0))
        .inner_margin(Margin::symmetric(12.0, 8.0));

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
