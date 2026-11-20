use crate::audio::PlayerController;
use crate::library::db::Database;
use crate::library::model::Track;
use crate::ui::icons::{paint_icon, Icon};
use crate::ui::theme::Theme;
use egui::{pos2, vec2, Color32, Layout, Rect, RichText, Rounding, ScrollArea, Stroke, Ui};
use std::collections::BTreeMap;
use std::sync::Arc;

#[derive(Default, Clone)]
pub struct GenresViewState {
    pub selected_genre: Option<String>,
    pub was_fetching: bool,
}

#[allow(clippy::too_many_arguments)]
pub fn render_genres(
    ui: &mut Ui,
    tracks: &[Track],
    search_query: &str,
    state: &mut GenresViewState,
    player: &mut PlayerController,
    theme: &Theme,
    db: &Database,
    on_refresh_library: &mut bool,
) {
    let progress = crate::library::genre_fetcher::get_genre_fetch_progress();
    let is_running = crate::library::genre_fetcher::is_genre_fetch_running() || progress.is_some();

    if is_running {
        ui.ctx().request_repaint();
        state.was_fetching = true;
    } else if state.was_fetching {
        state.was_fetching = false;
        *on_refresh_library = true;
    }

    // Detail view if a genre is selected
    let selected_genre_name = state.selected_genre.clone();
    if let Some(selected) = selected_genre_name {
        render_genre_detail(
            ui,
            &selected,
            tracks,
            search_query,
            state,
            player,
            theme,
            db,
        );
        return;
    }

    // Main Genres View Header & Action Bar
    render_header_and_action_bar(ui, theme, db, state, is_running, progress);

    ui.add_space(14.0);

    // Fetch genres with track counts from database
    let db_genres = db.get_all_genres_with_counts().unwrap_or_default();

    let mut genre_list: Vec<(String, usize)> = if !db_genres.is_empty() {
        let mut list = db_genres;
        // Count tracks with missing or unknown genre
        let unknown_count = tracks
            .iter()
            .filter(|t| crate::library::genre_fetcher::is_missing_or_unknown_genre(&t.genre))
            .count();
        if unknown_count > 0
            && !list.iter().any(|(g, _)| {
                g.eq_ignore_ascii_case("Unknown Genre") || g.eq_ignore_ascii_case("Unknown")
            })
        {
            list.push(("Unknown Genre".to_string(), unknown_count));
        }
        list
    } else {
        // Fallback to in-memory tracks grouping
        let mut map: BTreeMap<String, usize> = BTreeMap::new();
        for track in tracks {
            let g = if crate::library::genre_fetcher::is_missing_or_unknown_genre(&track.genre) {
                "Unknown Genre".to_string()
            } else {
                track.genre.trim().to_string()
            };
            *map.entry(g).or_default() += 1;
        }
        let mut list: Vec<(String, usize)> = map.into_iter().collect();
        list.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        list
    };

    let query = search_query.trim().to_lowercase();
    if !query.is_empty() {
        genre_list.retain(|(g, _)| g.to_lowercase().contains(&query));
    }

    if genre_list.is_empty() {
        render_empty_genres(ui, theme, !query.is_empty());
        return;
    }

    // Auto-wrapping responsive grid of Genres
    ScrollArea::vertical()
        .id_salt("genres_grid_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let spacing = 14.0_f32;
            let min_card_w = 170.0_f32;
            let available_w = ui.available_width();
            let cols = ((available_w + spacing) / (min_card_w + spacing))
                .floor()
                .max(1.0) as usize;
            let card_w = ((available_w - (cols.saturating_sub(1) as f32 * spacing)) / cols as f32)
                .max(min_card_w * 0.75);
            let card_h = 138.0_f32;

            ui.spacing_mut().item_spacing = vec2(spacing, spacing);

            for chunk in genre_list.chunks(cols) {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = vec2(spacing, 0.0);

                    for (genre_name, track_count) in chunk {
                        let is_unknown = genre_name.eq_ignore_ascii_case("Unknown Genre")
                            || genre_name.eq_ignore_ascii_case("Unknown");

                        let mut card = theme.card_frame(12.0);
                        card.inner_margin = egui::Margin::same(0.0);
                        let icon = genre_icon(genre_name);

                        card.show(ui, |ui| {
                            let (rect, resp) = ui.allocate_exact_size(
                                vec2(card_w, card_h),
                                egui::Sense::click(),
                            );
                            let is_hovered = resp.hovered();

                            if is_hovered {
                                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                                // Hover glow border
                                ui.painter().rect_stroke(
                                    rect,
                                    Rounding::same(12.0_f32),
                                    Stroke::new(1.2_f32, theme.accent),
                                );
                            }

                            // Upper icon section
                            let icon_h = 76.0_f32;
                            let icon_rect = Rect::from_min_size(rect.min, vec2(card_w, icon_h));
                            let icon_bg = if is_hovered {
                                theme.surface_hover
                            } else {
                                theme.surface
                            };
                            ui.painter().rect_filled(
                                icon_rect,
                                Rounding {
                                    nw: 11.0,
                                    ne: 11.0,
                                    sw: 4.0,
                                    se: 4.0,
                                },
                                icon_bg,
                            );

                            let icon_color = if is_unknown {
                                theme.text_muted
                            } else {
                                theme.accent
                            };
                            let icon_box =
                                Rect::from_center_size(icon_rect.center(), vec2(32.0, 32.0));
                            paint_icon(ui.painter(), icon, icon_box, icon_color);

                            // Text & Count badge area
                            let char_limit = ((card_w - 20.0) / 8.0).max(10.0) as usize;
                            let display_name = if genre_name.chars().count() > char_limit {
                                let s: String =
                                    genre_name.chars().take(char_limit.saturating_sub(1)).collect();
                                format!("{s}...")
                            } else {
                                genre_name.clone()
                            };

                            let title_pos = pos2(rect.min.x + 12.0, rect.min.y + 86.0);
                            ui.painter().text(
                                title_pos,
                                egui::Align2::LEFT_TOP,
                                &display_name,
                                egui::FontId::proportional(13.5),
                                theme.text_primary,
                            );

                            let count_text = if *track_count == 1 {
                                "1 track".to_string()
                            } else {
                                format!("{} tracks", track_count)
                            };

                            // Track count pill badge
                            let count_pos = pos2(rect.min.x + 12.0, rect.min.y + 110.0);
                            let font_id = egui::FontId::proportional(11.0);
                            let galley = ui.painter().layout_no_wrap(
                                count_text,
                                font_id,
                                theme.text_muted,
                            );
                            let badge_rect = Rect::from_min_size(
                                pos2(count_pos.x - 4.0, count_pos.y - 2.0),
                                vec2(galley.rect.width() + 8.0, 18.0),
                            );
                            ui.painter().rect_filled(
                                badge_rect,
                                Rounding::same(9.0_f32),
                                Color32::from_white_alpha(10),
                            );
                            ui.painter().galley(count_pos, galley, theme.text_muted);

                            if resp.clicked() {
                                state.selected_genre = Some(genre_name.clone());
                            }
                        });
                    }
                });
            }
        });
}

fn render_header_and_action_bar(
    ui: &mut Ui,
    theme: &Theme,
    db: &Database,
    state: &mut GenresViewState,
    is_running: bool,
    progress: Option<(usize, usize)>,
) {
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.label(
                RichText::new("Genres")
                    .size(22.0)
                    .color(theme.text_primary)
                    .strong(),
            );
            ui.label(
                RichText::new("Browse music by genre or automatically fetch missing tags")
                    .color(theme.text_muted)
                    .size(12.5),
            );
        });

        ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
            if is_running {
                // Fetching progress display & disabled button state
                let (processed, total) = progress.unwrap_or((0, 0));
                let status_label = if total > 0 {
                    format!("Fetching genres: {} / {}...", processed, total)
                } else {
                    "Fetching genres...".to_string()
                };

                // Active progress badge
                let font_id = egui::FontId::proportional(12.5);
                let galley = ui
                    .painter()
                    .layout_no_wrap(status_label, font_id, theme.accent);
                let icon_w = 18.0;
                let padding_x = 14.0;
                let height = 30.0;
                let width = galley.rect.width() + icon_w + (padding_x * 2.0);

                let (rect, _) = ui.allocate_exact_size(vec2(width, height), egui::Sense::hover());
                ui.painter().rect_filled(
                    rect,
                    Rounding::same(15.0_f32),
                    Color32::from_white_alpha(15),
                );
                ui.painter().rect_stroke(
                    rect,
                    Rounding::same(15.0_f32),
                    Stroke::new(1.0_f32, theme.border_accent),
                );

                let icon_rect = Rect::from_center_size(
                    pos2(rect.min.x + padding_x + 7.0, rect.center().y),
                    vec2(14.0, 14.0),
                );
                paint_icon(ui.painter(), Icon::Refresh, icon_rect, theme.accent);

                let text_pos = pos2(
                    rect.min.x + padding_x + icon_w,
                    rect.center().y - galley.rect.height() / 2.0,
                );
                ui.painter().galley(text_pos, galley, theme.accent);
            } else {
                // Interactive [ Fetch Missing Genres ] button with Refresh vector icon
                let label = "Fetch Missing Genres";
                let font_id = egui::FontId::proportional(12.5);
                let text_color = theme.text_primary;
                let galley = ui
                    .painter()
                    .layout_no_wrap(label.to_string(), font_id, text_color);
                let icon_w = 18.0;
                let padding_x = 14.0;
                let height = 30.0;
                let width = galley.rect.width() + icon_w + (padding_x * 2.0);

                let (rect, resp) =
                    ui.allocate_exact_size(vec2(width, height), egui::Sense::click());
                if resp.hovered() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                }

                let bg_color = if resp.is_pointer_button_down_on() {
                    Color32::from_white_alpha(30)
                } else if resp.hovered() {
                    Color32::from_white_alpha(20)
                } else {
                    theme.card
                };

                let stroke_color = if resp.hovered() {
                    theme.border_accent
                } else {
                    theme.border
                };

                ui.painter().rect_filled(rect, Rounding::same(15.0_f32), bg_color);
                ui.painter().rect_stroke(
                    rect,
                    Rounding::same(15.0_f32),
                    Stroke::new(1.0_f32, stroke_color),
                );

                let mut cur_x = rect.min.x + padding_x;
                let center_y = rect.center().y;

                let icon_rect =
                    Rect::from_center_size(pos2(cur_x + 7.0, center_y), vec2(14.0, 14.0));
                let icon_color = if resp.hovered() {
                    theme.accent
                } else {
                    theme.text_secondary
                };
                paint_icon(ui.painter(), Icon::Refresh, icon_rect, icon_color);
                cur_x += icon_w;

                let text_pos = pos2(cur_x, center_y - galley.rect.height() / 2.0);
                ui.painter().galley(text_pos, galley, text_color);

                if resp.clicked() {
                    crate::library::genre_fetcher::start_genre_backfill(Arc::new(db.clone()));
                    state.was_fetching = true;
                    ui.ctx().request_repaint();
                }
            }
        });
    });

    // If running, draw a sleek progress line
    if is_running {
        if let Some((processed, total)) = progress {
            if total > 0 {
                ui.add_space(8.0);
                let (p_rect, _) = ui.allocate_exact_size(
                    vec2(ui.available_width(), 4.0),
                    egui::Sense::hover(),
                );
                ui.painter().rect_filled(
                    p_rect,
                    Rounding::same(2.0_f32),
                    Color32::from_white_alpha(15),
                );
                let pct = (processed as f32 / total as f32).clamp(0.0, 1.0);
                let filled_w = p_rect.width() * pct;
                if filled_w > 0.0 {
                    let fill_rect = Rect::from_min_size(p_rect.min, vec2(filled_w, p_rect.height()));
                    ui.painter().rect_filled(fill_rect, Rounding::same(2.0_f32), theme.accent);
                }
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn render_genre_detail(
    ui: &mut Ui,
    selected: &str,
    tracks: &[Track],
    search_query: &str,
    state: &mut GenresViewState,
    player: &mut PlayerController,
    theme: &Theme,
    db: &Database,
) {
    let mut back_clicked = false;
    ui.horizontal(|ui| {
        let (rect, resp) = ui.allocate_exact_size(vec2(130.0, 30.0), egui::Sense::click());
        if resp.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
        let stroke_color = if resp.hovered() {
            theme.border_accent
        } else {
            theme.border
        };
        let bg_color = if resp.hovered() {
            theme.surface_hover
        } else {
            theme.card
        };
        ui.painter().rect_filled(rect, Rounding::same(8.0_f32), bg_color);
        ui.painter().rect_stroke(rect, Rounding::same(8.0_f32), Stroke::new(1.0_f32, stroke_color));

        let icon_box = Rect::from_center_size(pos2(rect.min.x + 16.0, rect.center().y), vec2(14.0, 14.0));
        paint_icon(ui.painter(), Icon::ChevronLeft, icon_box, theme.accent);

        ui.painter().text(
            pos2(rect.min.x + 32.0, rect.center().y),
            egui::Align2::LEFT_CENTER,
            "Back to Genres",
            egui::FontId::proportional(12.5),
            theme.accent,
        );

        if resp.clicked() {
            back_clicked = true;
        }
    });

    if back_clicked {
        state.selected_genre = None;
        return;
    }

    ui.add_space(10.0);

    // Fetch tracks for this genre
    let is_unknown = selected.eq_ignore_ascii_case("Unknown Genre") || selected.eq_ignore_ascii_case("Unknown");
    let genre_tracks: Vec<Track> = if is_unknown {
        tracks
            .iter()
            .filter(|t| crate::library::genre_fetcher::is_missing_or_unknown_genre(&t.genre))
            .cloned()
            .collect()
    } else {
        match db.get_tracks_by_genre(selected) {
            Ok(list) if !list.is_empty() => list,
            _ => tracks
                .iter()
                .filter(|t| t.genre.trim().eq_ignore_ascii_case(selected))
                .cloned()
                .collect(),
        }
    };

    let query = search_query.trim().to_lowercase();
    let display_tracks: Vec<Track> = if query.is_empty() {
        genre_tracks.clone()
    } else {
        genre_tracks
            .iter()
            .filter(|t| {
                t.display_title().to_lowercase().contains(&query)
                    || t.display_artist().to_lowercase().contains(&query)
                    || t.album.to_lowercase().contains(&query)
            })
            .cloned()
            .collect()
    };

    // Genre Header Card
    let mut header_frame = theme.glass_frame(12.0);
    header_frame.inner_margin = egui::Margin::same(16.0_f32);
    let icon = genre_icon(selected);

    header_frame.show(ui, |ui| {
        ui.horizontal(|ui| {
            let (rect, _) = ui.allocate_exact_size(vec2(80.0, 80.0), egui::Sense::hover());
            ui.painter().rect_filled(rect, Rounding::same(12.0_f32), theme.card);
            ui.painter().rect_stroke(
                rect,
                Rounding::same(12.0_f32),
                Stroke::new(1.0_f32, theme.border_accent),
            );
            let icon_box = Rect::from_center_size(rect.center(), vec2(36.0, 36.0));
            paint_icon(ui.painter(), icon, icon_box, theme.accent);

            ui.add_space(14.0);

            ui.vertical(|ui| {
                ui.label(
                    RichText::new(selected)
                        .size(22.0)
                        .color(theme.text_primary)
                        .strong(),
                );
                ui.label(
                    RichText::new(format!("{} Tracks in Library", genre_tracks.len()))
                        .color(theme.text_muted)
                        .size(12.5),
                );

                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    let play_btn = egui::Button::new(
                        RichText::new("Play All")
                            .color(Color32::from_rgb(11, 9, 20))
                            .strong(),
                    )
                    .fill(theme.accent)
                    .rounding(Rounding::same(8.0_f32));
                    if ui.add(play_btn).clicked() && !genre_tracks.is_empty() {
                        player.load_queue(genre_tracks.clone(), 0);
                    }

                    ui.add_space(8.0);
                    let queue_btn = egui::Button::new(
                        RichText::new("Add All to Queue")
                            .color(theme.text_primary)
                            .size(12.5),
                    )
                    .fill(theme.card)
                    .stroke(Stroke::new(1.0_f32, theme.border))
                    .rounding(Rounding::same(8.0_f32));
                    if ui.add(queue_btn).clicked() {
                        for t in &genre_tracks {
                            player.queue.push(t.clone());
                        }
                    }
                });
            });
        });
    });

    ui.add_space(12.0);

    if display_tracks.is_empty() {
        ui.vertical_centered(|ui| {
            ui.add_space(30.0);
            ui.label(
                RichText::new("No tracks found matching your query in this genre.")
                    .color(theme.text_muted)
                    .size(14.0),
            );
        });
        return;
    }

    // Track list for this genre
    ScrollArea::vertical()
        .id_salt("genre_detail_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for (i, t) in display_tracks.iter().enumerate() {
                let row = egui::Frame::none()
                    .fill(theme.card)
                    .stroke(Stroke::new(1.0_f32, theme.border))
                    .rounding(Rounding::same(6.0_f32))
                    .inner_margin(egui::Margin::symmetric(12.0_f32, 8.0_f32));

                let resp = row.show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(format!("{}", i + 1))
                                .size(12.0)
                                .color(theme.text_muted),
                        );
                        ui.add_space(8.0);
                        ui.label(
                            RichText::new(t.display_title())
                                .size(13.0)
                                .color(theme.text_primary)
                                .strong(),
                        );
                        ui.label(
                            RichText::new(format!("- {}", t.display_artist()))
                                .size(12.0)
                                .color(theme.text_secondary),
                        );

                        if !t.album.trim().is_empty() {
                            ui.label(
                                RichText::new(format!("({})", t.album))
                                    .size(11.5)
                                    .color(theme.text_muted),
                            );
                        }

                        ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(
                                RichText::new(t.formatted_duration())
                                    .size(12.0)
                                    .color(theme.text_muted),
                            );
                        });
                    });
                });

                let interact = resp.response.interact(egui::Sense::click());
                if interact.hovered() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                }
                if interact.clicked() || interact.double_clicked() {
                    player.load_queue(display_tracks.clone(), i);
                }
            }
        });
}

fn render_empty_genres(ui: &mut Ui, theme: &Theme, is_filtered: bool) {
    ui.vertical_centered(|ui| {
        ui.add_space(60.0);
        let (rect, _) = ui.allocate_exact_size(vec2(72.0, 72.0), egui::Sense::hover());
        ui.painter().rect_filled(rect, Rounding::same(16.0_f32), theme.card);
        ui.painter().rect_stroke(rect, Rounding::same(16.0_f32), Stroke::new(1.0_f32, theme.border));
        let icon_box = Rect::from_center_size(rect.center(), vec2(32.0, 32.0));
        paint_icon(ui.painter(), Icon::Tag, icon_box, theme.text_muted);

        ui.add_space(16.0);
        if is_filtered {
            ui.label(
                RichText::new("No matching genres found")
                    .size(16.0)
                    .color(theme.text_primary)
                    .strong(),
            );
            ui.label(
                RichText::new("Try a different search term")
                    .size(12.5)
                    .color(theme.text_muted),
            );
        } else {
            ui.label(
                RichText::new("No genres in library")
                    .size(16.0)
                    .color(theme.text_primary)
                    .strong(),
            );
            ui.label(
                RichText::new("Click 'Fetch Missing Genres' above to identify and tag tracks")
                    .size(12.5)
                    .color(theme.text_muted),
            );
        }
    });
}

fn genre_icon(genre: &str) -> Icon {
    let lower = genre.to_lowercase();
    if lower.contains("rock") || lower.contains("metal") || lower.contains("punk") || lower.contains("grunge") {
        Icon::Music
    } else if lower.contains("jazz") || lower.contains("blues") || lower.contains("soul") || lower.contains("funk") || lower.contains("r&b") {
        Icon::Headphones
    } else if lower.contains("classical") || lower.contains("piano") || lower.contains("orchestra") || lower.contains("soundtrack") || lower.contains("score") || lower.contains("instrumental") {
        Icon::Disc
    } else if lower.contains("electro") || lower.contains("dance") || lower.contains("edm") || lower.contains("techno") || lower.contains("house") || lower.contains("synth") || lower.contains("ambient") || lower.contains("trance") {
        Icon::BarChart
    } else if lower.contains("pop") || lower.contains("vocal") || lower.contains("j-pop") || lower.contains("k-pop") || lower.contains("idol") {
        Icon::Mic
    } else if lower.contains("hip hop") || lower.contains("hip-hop") || lower.contains("rap") || lower.contains("trap") {
        Icon::Fire
    } else if lower.contains("folk") || lower.contains("acoustic") || lower.contains("country") || lower.contains("indie") || lower.contains("alternative") {
        Icon::Sparkle
    } else if lower.contains("podcast") || lower.contains("speech") || lower.contains("audio") || lower.contains("spoken") {
        Icon::Library
    } else {
        Icon::Tag
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppConfig;

    fn sample_track(id: i64, title: &str, artist: &str, album: &str, genre: &str) -> Track {
        Track {
            id,
            path: format!("C:/music/{title}.flac"),
            title: title.to_string(),
            artist: artist.to_string(),
            album_artist: "".to_string(),
            album: album.to_string(),
            genre: genre.to_string(),
            year: 2024,
            track_number: 1,
            duration_secs: 200.0,
            bitrate_kbps: 1000,
            sample_rate: 44100,
            channels: 2,
            format: "FLAC".to_string(),
            file_size: 1024,
            play_count: 0,
            favorite: false,
            has_art: false,
            folder: "C:/music".to_string(),
        }
    }

    #[test]
    fn test_genres_view_state_defaults() {
        let state = GenresViewState::default();
        assert_eq!(state.selected_genre, None);
        assert!(!state.was_fetching);
    }

    #[test]
    fn test_genre_icon_matching() {
        assert!(matches!(genre_icon("Alternative Rock"), Icon::Music));
        assert!(matches!(genre_icon("Heavy Metal"), Icon::Music));
        assert!(matches!(genre_icon("J-Pop"), Icon::Mic));
        assert!(matches!(genre_icon("K-Pop"), Icon::Mic));
        assert!(matches!(genre_icon("Vocal Pop"), Icon::Mic));
        assert!(matches!(genre_icon("EDM / Dance"), Icon::BarChart));
        assert!(matches!(genre_icon("Synthwave"), Icon::BarChart));
        assert!(matches!(genre_icon("Classical Symphony"), Icon::Disc));
        assert!(matches!(genre_icon("Piano Soundtrack"), Icon::Disc));
        assert!(matches!(genre_icon("Smooth Jazz"), Icon::Headphones));
        assert!(matches!(genre_icon("R&B / Soul"), Icon::Headphones));
        assert!(matches!(genre_icon("Hip-Hop"), Icon::Fire));
        assert!(matches!(genre_icon("Acoustic Folk"), Icon::Sparkle));
        assert!(matches!(genre_icon("Unknown Genre"), Icon::Tag));
        assert!(matches!(genre_icon(""), Icon::Tag));
    }

    #[test]
    fn test_render_genres_empty_library() {
        let ctx = egui::Context::default();
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let theme = Theme::from_name("Eclipse", None);
                let cfg = AppConfig::default();
                let mut player = PlayerController::new(&cfg);
                let mut state = GenresViewState::default();
                let db = Database::open_in_memory().unwrap();
                let mut on_refresh = false;

                render_genres(
                    ui,
                    &[],
                    "",
                    &mut state,
                    &mut player,
                    &theme,
                    &db,
                    &mut on_refresh,
                );

                assert_eq!(state.selected_genre, None);
                assert!(!on_refresh);
            });
        });
    }

    #[test]
    fn test_render_genres_grid_with_tracks() {
        let ctx = egui::Context::default();
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let theme = Theme::from_name("Eclipse", None);
                let cfg = AppConfig::default();
                let mut player = PlayerController::new(&cfg);
                let mut state = GenresViewState::default();
                let db = Database::open_in_memory().unwrap();
                let mut on_refresh = false;

                let t1 = sample_track(1, "Song 1", "Artist 1", "Album 1", "Rock");
                let t2 = sample_track(2, "Song 2", "Artist 2", "Album 2", "Rock");
                let t3 = sample_track(3, "Song 3", "Artist 3", "Album 3", "Electronic");
                let t4 = sample_track(4, "Song 4", "Artist 4", "Album 4", "");

                let _ = db.upsert_track(&t1);
                let _ = db.upsert_track(&t2);
                let _ = db.upsert_track(&t3);
                let _ = db.upsert_track(&t4);

                let tracks = vec![t1, t2, t3, t4];

                render_genres(
                    ui,
                    &tracks,
                    "",
                    &mut state,
                    &mut player,
                    &theme,
                    &db,
                    &mut on_refresh,
                );

                assert_eq!(state.selected_genre, None);
            });
        });
    }

    #[test]
    fn test_render_genres_search_filter() {
        let ctx = egui::Context::default();
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let theme = Theme::from_name("Eclipse", None);
                let cfg = AppConfig::default();
                let mut player = PlayerController::new(&cfg);
                let mut state = GenresViewState::default();
                let db = Database::open_in_memory().unwrap();
                let mut on_refresh = false;

                let t1 = sample_track(1, "Song 1", "Artist 1", "Album 1", "Rock");
                let t2 = sample_track(2, "Song 2", "Artist 2", "Album 2", "Electronic");
                let tracks = vec![t1, t2];

                render_genres(
                    ui,
                    &tracks,
                    "Electro",
                    &mut state,
                    &mut player,
                    &theme,
                    &db,
                    &mut on_refresh,
                );

                assert_eq!(state.selected_genre, None);
            });
        });
    }

    #[test]
    fn test_render_genre_detail_drilldown() {
        let ctx = egui::Context::default();
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let theme = Theme::from_name("Eclipse", None);
                let cfg = AppConfig::default();
                let mut player = PlayerController::new(&cfg);
                let mut state = GenresViewState {
                    selected_genre: Some("Rock".to_string()),
                    was_fetching: false,
                };
                let db = Database::open_in_memory().unwrap();
                let mut on_refresh = false;

                let t1 = sample_track(1, "Song 1", "Artist 1", "Album 1", "Rock");
                let _ = db.upsert_track(&t1);
                let tracks = vec![t1];

                render_genres(
                    ui,
                    &tracks,
                    "",
                    &mut state,
                    &mut player,
                    &theme,
                    &db,
                    &mut on_refresh,
                );

                assert_eq!(state.selected_genre, Some("Rock".to_string()));
            });
        });
    }
}
