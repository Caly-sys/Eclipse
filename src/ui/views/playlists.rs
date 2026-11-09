use crate::audio::PlayerController;
use crate::library::model::{Playlist, Track};
use crate::ui::components::format_badge;
use crate::ui::icons::{icon_button, paint_icon, render_icon, Icon};
use crate::ui::theme::Theme;
use egui::{vec2, Color32, Layout, Rect, RichText, Rounding, ScrollArea, Stroke, Ui};

#[derive(Default)]
pub struct PlaylistsViewState {
    pub selected_playlist_id: Option<i64>,
    pub new_playlist_name: String,
    pub is_creating: bool,
}

#[allow(clippy::too_many_arguments)]
pub fn render_playlists(
    ui: &mut Ui,
    playlists: &[Playlist],
    playlist_tracks: &[Track],
    state: &mut PlaylistsViewState,
    player: &mut PlayerController,
    theme: &Theme,
    on_create_playlist: &mut Option<String>,
    on_delete_playlist: &mut Option<i64>,
) {
    ui.horizontal(|ui| {
        render_icon(ui, Icon::Playlist, 18.0, theme.accent);
        ui.add_space(4.0);
        ui.label(RichText::new("Playlists").size(18.0).color(theme.text_primary).strong());

        ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
            let (btn_rect, btn_resp) = ui.allocate_exact_size(vec2(116.0, 28.0), egui::Sense::click());
            let btn_bg = if btn_resp.hovered() { theme.surface_hover } else { theme.card };
            let btn_stroke = if btn_resp.hovered() { Stroke::new(1.0_f32, theme.border_accent) } else { Stroke::new(1.0_f32, theme.border) };
            ui.painter().rect_filled(btn_rect, Rounding::same(6.0), btn_bg);
            ui.painter().rect_stroke(btn_rect, Rounding::same(6.0), btn_stroke);

            let icon_r = Rect::from_center_size(egui::pos2(btn_rect.min.x + 16.0, btn_rect.center().y), vec2(12.0, 12.0));
            paint_icon(ui.painter(), Icon::Plus, icon_r, theme.accent);

            ui.painter().text(
                egui::pos2(btn_rect.min.x + 28.0, btn_rect.center().y),
                egui::Align2::LEFT_CENTER,
                "New Playlist",
                egui::FontId::proportional(12.0),
                theme.accent,
            );

            if btn_resp.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }
            if btn_resp.clicked() {
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

    ui.add_space(8.0);

    ui.columns(2, |cols| {
        // Left Column: List of Playlists
        cols[0].vertical(|ui| {
            ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing = vec2(0.0, 6.0);

                    for pl in playlists {
                        let is_selected = state.selected_playlist_id == Some(pl.id);

                        let frame = egui::Frame::none()
                            .fill(if is_selected {
                                theme.accent_glow
                            } else {
                                theme.card
                            })
                            .stroke(if is_selected {
                                Stroke::new(1.0_f32, theme.border_accent)
                            } else {
                                Stroke::new(1.0_f32, theme.border)
                            })
                            .rounding(Rounding::same(8.0))
                            .inner_margin(egui::Margin::symmetric(12.0, 8.0));

                        let resp = frame.show(ui, |ui| {
                            ui.horizontal(|ui| {
                                render_icon(ui, Icon::Playlist, 16.0, theme.accent);
                                ui.add_space(6.0);
                                ui.vertical(|ui| {
                                    ui.label(RichText::new(&pl.name).color(theme.text_primary).size(13.5).strong());
                                    ui.label(RichText::new(format!("{} Tracks", pl.track_count)).color(theme.text_muted).size(11.0));
                                });

                                ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                                    if icon_button(ui, Icon::Trash, 13.0, theme.text_muted).on_hover_text("Delete Playlist").clicked() {
                                        *on_delete_playlist = Some(pl.id);
                                    }
                                });
                            });
                        });

                        if resp.response.interact(egui::Sense::click()).clicked() {
                            state.selected_playlist_id = Some(pl.id);
                        }
                    }
                });
        });

        // Right Column: Tracks in Selected Playlist
        cols[1].vertical(|ui| {
            if let Some(pl_id) = state.selected_playlist_id {
                let pl_name = playlists
                    .iter()
                    .find(|p| p.id == pl_id)
                    .map(|p| p.name.as_str())
                    .unwrap_or("Playlist");

                ui.horizontal(|ui| {
                    ui.label(RichText::new(pl_name).size(16.0).color(theme.text_primary).strong());

                    ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                        let (btn_rect, btn_resp) = ui.allocate_exact_size(vec2(90.0, 26.0), egui::Sense::click());
                        let btn_bg = if btn_resp.hovered() { theme.accent_warm } else { theme.accent };
                        ui.painter().rect_filled(btn_rect, Rounding::same(8.0), btn_bg);

                        let icon_r = Rect::from_center_size(egui::pos2(btn_rect.min.x + 16.0, btn_rect.center().y), vec2(11.0, 11.0));
                        paint_icon(ui.painter(), Icon::Play, icon_r, Color32::from_rgb(11, 9, 20));

                        ui.painter().text(
                            egui::pos2(btn_rect.min.x + 28.0, btn_rect.center().y),
                            egui::Align2::LEFT_CENTER,
                            "Play All",
                            egui::FontId::proportional(11.5),
                            Color32::from_rgb(11, 9, 20),
                        );

                        if btn_resp.hovered() {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                        }
                        if btn_resp.clicked() && !playlist_tracks.is_empty() {
                            player.load_queue(playlist_tracks.to_vec(), 0);
                        }
                    });
                });

                ui.add_space(8.0);

                if playlist_tracks.is_empty() {
                    ui.label(RichText::new("No tracks in this playlist.").color(theme.text_muted));
                } else {
                    ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            for (i, t) in playlist_tracks.iter().enumerate() {
                                let row = egui::Frame::none()
                                    .fill(theme.surface)
                                    .rounding(Rounding::same(6.0))
                                    .inner_margin(egui::Margin::symmetric(10.0, 6.0));

                                let _resp = row.show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(RichText::new(format!("{}", i + 1)).size(11.0).color(theme.text_muted));
                                        ui.add_space(6.0);
                                        ui.label(RichText::new(t.display_title()).size(13.0).color(theme.text_primary));
                                        ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                                            ui.label(RichText::new(t.formatted_duration()).size(11.5).color(theme.text_secondary));
                                            ui.add_space(10.0);
                                            format_badge(ui, &t.format, t.is_lossless(), theme);
                                        });
                                    });
                                });
                            }
                        });
                }
            } else {
                ui.label(RichText::new("Select a playlist to view tracks.").color(theme.text_muted));
            }
        });
    });
}
