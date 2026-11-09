use crate::audio::PlayerController;
use crate::library::model::Track;
use crate::ui::icons::{paint_icon, Icon};
use crate::ui::theme::Theme;
use egui::{pos2, vec2, Color32, Layout, Rect, RichText, Rounding, ScrollArea, Stroke, Ui};
use std::collections::BTreeMap;

#[derive(Default, Clone)]
pub struct GenresViewState {
    pub selected_genre: Option<String>,
}

pub fn render_genres(
    ui: &mut Ui,
    tracks: &[Track],
    search_query: &str,
    state: &mut GenresViewState,
    player: &mut PlayerController,
    theme: &Theme,
) {
    let mut all_genres: BTreeMap<String, Vec<&Track>> = BTreeMap::new();
    for track in tracks {
        let genre_name = if track.genre.trim().is_empty() {
            "Unknown Genre".to_string()
        } else {
            track.genre.trim().to_string()
        };
        all_genres.entry(genre_name).or_default().push(track);
    }

    let query = search_query.trim().to_lowercase();
    let genres: BTreeMap<String, Vec<&Track>> = all_genres
        .into_iter()
        .filter(|(g, _)| {
            if query.is_empty() {
                true
            } else {
                g.to_lowercase().contains(&query)
            }
        })
        .collect();

    // Detail view if a genre is selected
    let selected_genre_name = state.selected_genre.clone();
    if let Some(selected) = selected_genre_name {
        if let Some(genre_tracks) = genres.get(&selected) {
            let mut back_clicked = false;
            ui.horizontal(|ui| {
                if ui.button(RichText::new("Back to Genres").color(theme.accent).size(13.0)).clicked() {
                    back_clicked = true;
                }
            });

            if back_clicked {
                state.selected_genre = None;
                return;
            }

            ui.add_space(10.0);

            // Genre Header
            let mut header_frame = theme.glass_frame(12.0);
            header_frame.inner_margin = egui::Margin::same(16.0_f32);

            let icon = genre_icon(&selected);

            header_frame.show(ui, |ui| {
                ui.horizontal(|ui| {
                    let (rect, _) = ui.allocate_exact_size(vec2(80.0, 80.0), egui::Sense::hover());
                    ui.painter().rect_filled(rect, Rounding::same(12.0_f32), theme.card);
                    ui.painter().rect_stroke(rect, Rounding::same(12.0_f32), Stroke::new(1.0_f32, theme.border_accent));
                    let icon_box = Rect::from_center_size(rect.center(), vec2(36.0, 36.0));
                    paint_icon(ui.painter(), icon, icon_box, theme.accent);

                    ui.add_space(14.0);

                    ui.vertical(|ui| {
                        ui.label(RichText::new(&selected).size(22.0).color(theme.text_primary).strong());
                        ui.label(RichText::new(format!("{} Tracks in Library", genre_tracks.len())).color(theme.text_muted).size(12.5));

                        ui.add_space(8.0);
                        let play_btn = egui::Button::new(RichText::new("Play All").color(Color32::from_rgb(11, 9, 20)).strong())
                            .fill(theme.accent)
                            .rounding(Rounding::same(8.0_f32));
                        if ui.add(play_btn).clicked() {
                            let list: Vec<Track> = genre_tracks.iter().map(|&t| t.clone()).collect();
                            player.load_queue(list, 0);
                        }
                    });
                });
            });

            ui.add_space(12.0);

            // Track list for this genre
            ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    for (i, t) in genre_tracks.iter().enumerate() {
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
                                ui.label(RichText::new(format!("· {}", t.display_artist())).size(12.0).color(theme.text_secondary));
                                ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.label(RichText::new(t.formatted_duration()).size(12.0).color(theme.text_muted));
                                });
                            });
                        });

                        if resp.response.interact(egui::Sense::click()).clicked() {
                            let list: Vec<Track> = genre_tracks.iter().map(|&track| track.clone()).collect();
                            player.load_queue(list, i);
                        }
                    }
                });
            return;
        }
    }

    // Auto-wrapping responsive grid of Genres
    ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let spacing = 14.0_f32;
            let min_card_w = 165.0_f32;
            let available_w = ui.available_width();
            let cols = ((available_w + spacing) / (min_card_w + spacing)).floor().max(1.0) as usize;
            let card_w = ((available_w - (cols.saturating_sub(1) as f32 * spacing)) / cols as f32).max(min_card_w * 0.75);
            let card_h = 140.0_f32;

            let genre_list: Vec<(&String, &Vec<&Track>)> = genres.iter().collect();

            ui.spacing_mut().item_spacing = vec2(spacing, spacing);

            for chunk in genre_list.chunks(cols) {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = vec2(spacing, 0.0);

                    for (genre_name, genre_tracks) in chunk {
                        let mut card = theme.card_frame(10.0);
                        card.inner_margin = egui::Margin::same(0.0);
                        let icon = genre_icon(genre_name);

                        card.show(ui, |ui| {
                            let (rect, resp) = ui.allocate_exact_size(vec2(card_w, card_h), egui::Sense::click());
                            if resp.hovered() {
                                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                                ui.painter().rect_stroke(rect, Rounding::same(10.0_f32), Stroke::new(1.0_f32, theme.border_accent));
                            }

                            let icon_rect = Rect::from_min_size(rect.min, vec2(card_w, 80.0));
                            ui.painter().rect_filled(icon_rect, Rounding::same(8.0_f32), theme.surface);
                            ui.painter().rect_stroke(icon_rect, Rounding::same(8.0_f32), Stroke::new(1.0_f32, theme.border));
                            let icon_box = Rect::from_center_size(icon_rect.center(), vec2(36.0, 36.0));
                            paint_icon(ui.painter(), icon, icon_box, theme.accent);

                            let char_limit = ((card_w - 16.0) / 8.0).max(10.0) as usize;
                            let display_name = if genre_name.chars().count() > char_limit {
                                let s: String = genre_name.chars().take(char_limit.saturating_sub(1)).collect();
                                format!("{s}…")
                            } else {
                                (*genre_name).clone()
                            };

                            let title_pos = pos2(rect.min.x + 10.0, rect.min.y + 90.0);
                            let count_pos = pos2(rect.min.x + 10.0, rect.min.y + 112.0);

                            ui.painter().text(
                                title_pos,
                                egui::Align2::LEFT_TOP,
                                &display_name,
                                egui::FontId::proportional(13.0),
                                theme.text_primary,
                            );

                            ui.painter().text(
                                count_pos,
                                egui::Align2::LEFT_TOP,
                                format!("{} Tracks", genre_tracks.len()),
                                egui::FontId::proportional(11.0),
                                theme.text_muted,
                            );

                            if resp.clicked() {
                                state.selected_genre = Some((*genre_name).clone());
                            }
                        });
                    }
                });
            }
        });
}

fn genre_icon(genre: &str) -> Icon {
    let lower = genre.to_lowercase();
    if lower.contains("rock") || lower.contains("metal") || lower.contains("punk") {
        Icon::Music
    } else if lower.contains("jazz") || lower.contains("blues") || lower.contains("soul") {
        Icon::Headphones
    } else if lower.contains("classical") || lower.contains("piano") || lower.contains("orchestra") {
        Icon::Disc
    } else if lower.contains("electro") || lower.contains("dance") || lower.contains("edm") || lower.contains("techno") {
        Icon::BarChart
    } else if lower.contains("pop") || lower.contains("vocal") {
        Icon::Mic
    } else {
        Icon::Tag
    }
}
