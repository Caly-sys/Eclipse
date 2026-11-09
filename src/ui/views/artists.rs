use crate::audio::PlayerController;
use crate::library::model::Track;
use crate::ui::icons::{paint_icon, Icon};
use crate::ui::theme::Theme;
use egui::{pos2, vec2, Color32, Layout, Rect, RichText, Rounding, ScrollArea, Stroke, Ui};
use std::collections::BTreeMap;

#[derive(Default)]
pub struct ArtistsViewState {
    pub selected_artist: Option<String>,
}

pub fn render_artists(
    ui: &mut Ui,
    tracks: &[Track],
    search_query: &str,
    state: &mut ArtistsViewState,
    player: &mut PlayerController,
    theme: &Theme,
) {
    let mut all_artists: BTreeMap<String, Vec<&Track>> = BTreeMap::new();
    for track in tracks {
        let artist_name = track.display_artist().to_string();
        all_artists.entry(artist_name).or_default().push(track);
    }

    let query = search_query.trim().to_lowercase();
    let artists: BTreeMap<String, Vec<&Track>> = all_artists
        .into_iter()
        .filter(|(artist_name, _)| {
            if query.is_empty() {
                true
            } else {
                artist_name.to_lowercase().contains(&query)
            }
        })
        .collect();

    let selected_artist_name = state.selected_artist.clone();
    if let Some(selected) = selected_artist_name {
        if let Some(artist_tracks) = artists.get(&selected) {
            let mut back_clicked = false;
            ui.horizontal(|ui| {
                if ui.button(RichText::new("Back to Artists").color(theme.accent).size(13.0)).clicked() {
                    back_clicked = true;
                }
            });

            if back_clicked {
                state.selected_artist = None;
                return;
            }

            ui.add_space(10.0);

            // Artist Header
            let mut header_frame = theme.glass_frame(12.0);
            header_frame.inner_margin = egui::Margin::same(16.0_f32);

            header_frame.show(ui, |ui| {
                ui.horizontal(|ui| {
                    let (rect, _) = ui.allocate_exact_size(vec2(80.0, 80.0), egui::Sense::hover());
                    ui.painter().rect_filled(rect, Rounding::same(40.0_f32), theme.card);
                    ui.painter().rect_stroke(rect, Rounding::same(40.0_f32), Stroke::new(1.0_f32, theme.border_accent));
                    let icon_box = Rect::from_center_size(rect.center(), vec2(36.0, 36.0));
                    paint_icon(ui.painter(), Icon::Mic, icon_box, theme.accent);

                    ui.add_space(14.0);

                    ui.vertical(|ui| {
                        ui.label(RichText::new(&selected).size(22.0).color(theme.text_primary).strong());
                        ui.label(RichText::new(format!("{} Tracks in Library", artist_tracks.len())).color(theme.text_muted).size(12.5));

                        ui.add_space(8.0);
                        let play_btn = egui::Button::new(RichText::new("Play All").color(Color32::from_rgb(11, 9, 20)).strong())
                            .fill(theme.accent)
                            .rounding(Rounding::same(8.0_f32));
                        if ui.add(play_btn).clicked() {
                            let list: Vec<Track> = artist_tracks.iter().map(|&t| t.clone()).collect();
                            player.load_queue(list, 0);
                        }
                    });
                });
            });

            ui.add_space(12.0);

            // Track list for this artist
            ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    for (i, t) in artist_tracks.iter().enumerate() {
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
                                ui.label(RichText::new(format!("({})", t.display_album())).size(11.5).color(theme.text_muted));
                                ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.label(RichText::new(t.formatted_duration()).size(12.0).color(theme.text_muted));
                                });
                            });
                        });

                        if resp.response.interact(egui::Sense::click()).clicked() {
                            let list: Vec<Track> = artist_tracks.iter().map(|&track| track.clone()).collect();
                            player.load_queue(list, i);
                        }
                    }
                });
            return;
        }
    }

    // Auto-wrapping responsive grid of Artists
    ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let spacing = 14.0_f32;
            let min_card_w = 165.0_f32;
            let available_w = ui.available_width();
            let cols = ((available_w + spacing) / (min_card_w + spacing)).floor().max(1.0) as usize;
            let card_w = ((available_w - (cols.saturating_sub(1) as f32 * spacing)) / cols as f32).max(min_card_w * 0.75);
            let card_h = 160.0_f32;

            let artist_list: Vec<(&String, &Vec<&Track>)> = artists.iter().collect();

            ui.spacing_mut().item_spacing = vec2(spacing, spacing);

            for chunk in artist_list.chunks(cols) {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = vec2(spacing, 0.0);

                    for (artist_name, artist_tracks) in chunk {
                        let mut card = theme.card_frame(10.0);
                        card.inner_margin = egui::Margin::same(12.0_f32);

                        card.show(ui, |ui| {
                            let (rect, resp) = ui.allocate_exact_size(vec2(card_w - 24.0, card_h - 24.0), egui::Sense::click());
                            if resp.hovered() {
                                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                                ui.painter().rect_stroke(rect.expand(12.0), Rounding::same(10.0_f32), Stroke::new(1.0_f32, theme.border_accent));
                            }

                            // Circular avatar area
                            let avatar_r = 34.0_f32;
                            let avatar_center = pos2(rect.center().x, rect.min.y + avatar_r + 6.0);
                            ui.painter().circle_filled(avatar_center, avatar_r, theme.surface);
                            ui.painter().circle_stroke(avatar_center, avatar_r, Stroke::new(1.0_f32, theme.border_accent));
                            let icon_box = Rect::from_center_size(avatar_center, vec2(28.0, 28.0));
                            paint_icon(ui.painter(), Icon::Mic, icon_box, theme.accent);

                            // Artist name & track count
                            let char_limit = ((card_w - 24.0) / 8.0).max(10.0) as usize;
                            let display_name = truncate_str(artist_name, char_limit);

                            ui.painter().text(
                                pos2(rect.center().x, avatar_center.y + avatar_r + 14.0),
                                egui::Align2::CENTER_TOP,
                                &display_name,
                                egui::FontId::proportional(13.5),
                                theme.text_primary,
                            );

                            ui.painter().text(
                                pos2(rect.center().x, avatar_center.y + avatar_r + 34.0),
                                egui::Align2::CENTER_TOP,
                                format!("{} Tracks", artist_tracks.len()),
                                egui::FontId::proportional(11.0),
                                theme.text_muted,
                            );

                            if resp.clicked() {
                                state.selected_artist = Some((*artist_name).clone());
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
