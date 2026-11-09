use crate::audio::PlayerController;
use crate::library::model::Track;
use crate::ui::components::format_badge;
use crate::ui::icons::{icon_button, paint_icon, Icon};
use crate::ui::theme::Theme;
use egui::{vec2, Color32, Layout, Rect, RichText, Rounding, ScrollArea, Stroke, Ui};
use rand::seq::SliceRandom;

#[derive(Default, Clone)]
pub struct SongsViewState {
    pub selected_track_id: Option<i64>,
}

pub fn render_songs(
    ui: &mut Ui,
    tracks: &[Track],
    search_query: &str,
    state: &mut SongsViewState,
    player: &mut PlayerController,
    theme: &Theme,
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
    on_toggle_fav: &mut Option<i64>,
) {
    let query = search_query.trim().to_lowercase();
    let filtered_tracks: Vec<&Track> = tracks
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
        .collect();

    ui.vertical(|ui| {
        // Optional smart playlist / section header banner
        let mut header_banner = theme.glass_frame(10.0);
        header_banner.inner_margin = egui::Margin::symmetric(14.0, 10.0);

        header_banner.show(ui, |ui| {
            ui.horizontal(|ui| {
                // Icon bubble with subtle glow
                let (icon_rect, _) = ui.allocate_exact_size(vec2(38.0, 38.0), egui::Sense::hover());
                ui.painter().rect_filled(icon_rect, Rounding::same(10.0), theme.card);
                ui.painter().rect_stroke(icon_rect, Rounding::same(10.0), Stroke::new(1.0_f32, theme.border));
                let vector_rect = Rect::from_center_size(icon_rect.center(), vec2(20.0, 20.0));
                paint_icon(ui.painter(), icon, vector_rect, theme.accent);

                ui.add_space(10.0);

                ui.vertical(|ui| {
                    ui.label(RichText::new(title).size(17.0).color(theme.text_primary).strong());
                    let desc_text = if !query.is_empty() {
                        format!("{} · {} of {} tracks match \"{}\"", description, filtered_tracks.len(), tracks.len(), query)
                    } else {
                        format!("{} · {} tracks", description, tracks.len())
                    };
                    ui.label(RichText::new(desc_text).size(11.5).color(theme.text_muted));
                });

                if !filtered_tracks.is_empty() {
                    ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                        // Shuffle button
                        let (shuf_rect, shuf_resp) = ui.allocate_exact_size(vec2(86.0, 28.0), egui::Sense::click());
                        let shuf_bg = if shuf_resp.hovered() { theme.surface_hover } else { theme.card };
                        let shuf_border = if shuf_resp.hovered() { Stroke::new(1.0_f32, theme.border_accent) } else { Stroke::new(1.0_f32, theme.border) };
                        ui.painter().rect_filled(shuf_rect, Rounding::same(8.0), shuf_bg);
                        ui.painter().rect_stroke(shuf_rect, Rounding::same(8.0), shuf_border);

                        let shuf_icon_r = Rect::from_center_size(egui::pos2(shuf_rect.min.x + 14.0, shuf_rect.center().y), vec2(12.0, 12.0));
                        paint_icon(ui.painter(), Icon::Shuffle, shuf_icon_r, theme.text_primary);
                        ui.painter().text(
                            egui::pos2(shuf_rect.min.x + 26.0, shuf_rect.center().y),
                            egui::Align2::LEFT_CENTER,
                            "Shuffle",
                            egui::FontId::proportional(12.0),
                            theme.text_primary,
                        );

                        if shuf_resp.hovered() {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                        }
                        if shuf_resp.clicked() {
                            let mut list: Vec<Track> = filtered_tracks.iter().map(|&t| t.clone()).collect();
                            let mut rng = rand::thread_rng();
                            list.shuffle(&mut rng);
                            player.load_queue(list, 0);
                        }

                        ui.add_space(6.0);

                        // Play All button
                        let (play_rect, play_resp) = ui.allocate_exact_size(vec2(90.0, 28.0), egui::Sense::click());
                        let play_bg = if play_resp.hovered() { theme.accent_warm } else { theme.accent };
                        ui.painter().rect_filled(play_rect, Rounding::same(8.0), play_bg);

                        let play_icon_r = Rect::from_center_size(egui::pos2(play_rect.min.x + 14.0, play_rect.center().y), vec2(11.0, 11.0));
                        paint_icon(ui.painter(), Icon::Play, play_icon_r, Color32::from_rgb(11, 9, 20));

                        ui.painter().text(
                            egui::pos2(play_rect.min.x + 26.0, play_rect.center().y),
                            egui::Align2::LEFT_CENTER,
                            "Play All",
                            egui::FontId::proportional(12.0),
                            Color32::from_rgb(11, 9, 20),
                        );

                        if play_resp.hovered() {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                        }
                        if play_resp.clicked() {
                            let list: Vec<Track> = filtered_tracks.iter().map(|&t| t.clone()).collect();
                            player.load_queue(list, 0);
                        }
                    });
                }
            });
        });

        ui.add_space(6.0);

        // Table Header
        let header_frame = egui::Frame::none()
            .fill(theme.surface)
            .stroke(Stroke::new(1.0_f32, theme.border))
            .rounding(Rounding::same(6.0))
            .inner_margin(egui::Margin::symmetric(12.0, 8.0));

        header_frame.show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("#").size(11.0).color(theme.text_muted).strong());
                ui.add_space(8.0);
                ui.label(RichText::new("FAV").size(11.0).color(theme.text_muted).strong());
                ui.add_space(8.0);
                ui.allocate_ui_with_layout(vec2(210.0, 16.0), Layout::left_to_right(egui::Align::Center), |ui| {
                    ui.label(RichText::new("TITLE").size(11.0).color(theme.text_muted).strong());
                });
                ui.allocate_ui_with_layout(vec2(150.0, 16.0), Layout::left_to_right(egui::Align::Center), |ui| {
                    ui.label(RichText::new("ARTIST").size(11.0).color(theme.text_muted).strong());
                });
                ui.allocate_ui_with_layout(vec2(150.0, 16.0), Layout::left_to_right(egui::Align::Center), |ui| {
                    ui.label(RichText::new("ALBUM").size(11.0).color(theme.text_muted).strong());
                });
                ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(RichText::new("DURATION").size(11.0).color(theme.text_muted).strong());
                    ui.add_space(14.0);
                    ui.label(RichText::new("FORMAT").size(11.0).color(theme.text_muted).strong());
                });
            });
        });

        ui.add_space(4.0);

        // Songs list
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
            ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing = vec2(0.0, 2.0);

                    for (idx, track) in filtered_tracks.iter().enumerate() {
                        let is_current = player.current_track().map(|t| t.id) == Some(track.id);
                        let is_selected = state.selected_track_id == Some(track.id);

                        let (bg_color, stroke) = if is_current {
                            (theme.accent_glow, Stroke::new(1.0_f32, theme.border_accent))
                        } else if is_selected {
                            (Color32::from_rgba_premultiplied(180, 140, 255, 18), Stroke::new(1.0_f32, theme.border))
                        } else {
                            (Color32::TRANSPARENT, Stroke::NONE)
                        };

                        let row_frame = egui::Frame::none()
                            .fill(bg_color)
                            .stroke(stroke)
                            .rounding(Rounding::same(6.0))
                            .inner_margin(egui::Margin::symmetric(12.0, 8.0));

                        let mut fav_clicked = false;

                        let resp = row_frame.show(ui, |ui| {
                            ui.horizontal(|ui| {
                                // Track number or playing vector icon
                                ui.allocate_ui_with_layout(vec2(22.0, 18.0), Layout::left_to_right(egui::Align::Center), |ui| {
                                    if is_current {
                                        let play_state_icon = if player.engine.is_playing() { Icon::Play } else { Icon::Pause };
                                        let icon_r = Rect::from_center_size(ui.available_rect_before_wrap().center(), vec2(12.0, 12.0));
                                        paint_icon(ui.painter(), play_state_icon, icon_r, theme.accent);
                                    } else {
                                        ui.label(RichText::new(format!("{}", idx + 1)).size(12.0).color(theme.text_muted));
                                    }
                                });

                                ui.add_space(4.0);

                                // Favorite toggle vector button
                                let (fav_icon, fav_color) = if track.favorite {
                                    (Icon::Heart, Color32::from_rgb(239, 68, 68))
                                } else {
                                    (Icon::HeartOutline, theme.text_muted)
                                };
                                if icon_button(ui, fav_icon, 13.0, fav_color).clicked() {
                                    *on_toggle_fav = Some(track.id);
                                    fav_clicked = true;
                                }

                                ui.add_space(4.0);

                                // Title
                                ui.allocate_ui_with_layout(vec2(210.0, 18.0), Layout::left_to_right(egui::Align::Center), |ui| {
                                    let c = if is_current { theme.accent } else { theme.text_primary };
                                    ui.add(egui::Label::new(RichText::new(track.display_title()).size(13.0).color(c).strong()).truncate());
                                });

                                // Artist
                                ui.allocate_ui_with_layout(vec2(150.0, 18.0), Layout::left_to_right(egui::Align::Center), |ui| {
                                    ui.add(egui::Label::new(RichText::new(track.display_artist()).size(12.0).color(theme.text_secondary)).truncate());
                                });

                                // Album
                                ui.allocate_ui_with_layout(vec2(150.0, 18.0), Layout::left_to_right(egui::Align::Center), |ui| {
                                    ui.add(egui::Label::new(RichText::new(track.display_album()).size(12.0).color(theme.text_muted)).truncate());
                                });

                                // Duration & Format
                                ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.label(RichText::new(track.formatted_duration()).size(12.0).color(theme.text_secondary));
                                    ui.add_space(14.0);
                                    format_badge(ui, &track.format, track.is_lossless(), theme);
                                });
                            });
                        });

                        let r = resp.response.interact(egui::Sense::click());
                        if r.hovered() && !fav_clicked {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                        }

                        if r.double_clicked() && !fav_clicked {
                            let list: Vec<Track> = filtered_tracks.iter().map(|&t| t.clone()).collect();
                            player.load_queue(list, idx);
                        } else if r.clicked() && !fav_clicked {
                            state.selected_track_id = Some(track.id);
                        }
                    }
                });
        }
    });
}
