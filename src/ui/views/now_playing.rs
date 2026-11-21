use crate::audio::PlayerController;
use crate::config::VisualizerStyle;
use crate::library::format_time_secs;
use crate::lyrics::LyricLine;
use crate::streams::controller::StreamController;
use crate::streams::provider::StreamTrack;
use crate::ui::art::AlbumArtManager;
use crate::ui::bento_theme::{segmented_tab_button, BentoPalette};
use crate::ui::components::render_spectrum;
use crate::ui::icons::{icon_button as vector_icon_button, paint_icon, render_icon, Icon};
use crate::ui::playerbar::seek_to;
use crate::ui::theme::Theme;
use egui::{pos2, vec2, Color32, Layout, Rect, RichText, Rounding, ScrollArea, Stroke, Ui};

/// Extension trait providing `theme.bento_palette()` dynamically.
pub trait ThemeBentoExt {
    fn bento_palette(&self) -> BentoPalette;
}

impl ThemeBentoExt for Theme {
    fn bento_palette(&self) -> BentoPalette {
        BentoPalette::from_theme(self)
    }
}

pub fn render_now_playing(
    ui: &mut Ui,
    player: &mut PlayerController,
    theme: &Theme,
    art_manager: &mut AlbumArtManager,
    stream_controller: Option<&StreamController>,
) {
    let palette = theme.bento_palette();

    let outer_margin = egui::Margin {
        top: 20.0,
        bottom: 16.0,
        left: 24.0,
        right: 24.0,
    };

    egui::Frame::none()
        .inner_margin(outer_margin)
        .show(ui, |ui| {
            if player.playback_source.is_stream() {
                if let Some(stream_track) = player.active_stream_track.clone() {
                    render_stream_now_playing(ui, &stream_track, player, theme, &palette, stream_controller);
                    return;
                }
            }
            if let Some(track) = player.current_track().cloned() {
                let pos_secs = player.engine.position().as_secs_f64();
                let pos_ms = (pos_secs * 1000.0) as u64;

                let total_width = ui.available_width();

                // Responsive layout breakpoint: < 860px switches to 2-row layout
                if total_width < 860.0 {
                    ScrollArea::vertical()
                        .id_salt("now_playing_responsive_scroll")
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            ui.spacing_mut().item_spacing = vec2(0.0, 12.0);

                            let mut top_rects: [Option<Rect>; 2] = [None, None];

                            // Top row: Cover Art + Synced Lyrics side-by-side (50/50)
                            ui.columns(2, |cols| {
                                cols[0].set_clip_rect(cols[0].clip_rect().intersect(cols[0].max_rect()));
                                top_rects[0] = render_track_info_card(&mut cols[0], &track, &palette, art_manager);

                                cols[1].set_clip_rect(cols[1].clip_rect().intersect(cols[1].max_rect()));
                                top_rects[1] = render_synced_lyrics_card(&mut cols[1], player, &track, pos_ms, &palette);
                            });

                            // Seam divider between the top 2 columns
                            if let (Some(r0), Some(r1)) = (top_rects[0], top_rects[1]) {
                                let y_min = r0.min.y.min(r1.min.y);
                                let y_max = r0.max.y.max(r1.max.y);
                                let seam_x = (r0.max.x + r1.min.x) * 0.5;
                                let seam_rect = Rect::from_min_max(pos2(seam_x - 0.5, y_min), pos2(seam_x + 0.5, y_max));
                                ui.painter().rect_filled(seam_rect, Rounding::ZERO, palette.seam);
                            }

                            // Bottom row: full-width AUDIO ANALYZER
                            ui.vertical(|ui| {
                                ui.set_clip_rect(ui.clip_rect().intersect(ui.max_rect()));
                                render_audio_analyzer_card(ui, player, &track, theme, &palette);
                            });
                        });
                } else {
                    // Standard 3-column side-by-side layout (>= 860px)
                    let mut col_rects: [Option<Rect>; 3] = [None, None, None];

                    ui.columns(3, |cols| {
                        cols[0].set_clip_rect(cols[0].clip_rect().intersect(cols[0].max_rect()));
                        col_rects[0] = render_track_info_card(&mut cols[0], &track, &palette, art_manager);

                        cols[1].set_clip_rect(cols[1].clip_rect().intersect(cols[1].max_rect()));
                        col_rects[1] = render_synced_lyrics_card(&mut cols[1], player, &track, pos_ms, &palette);

                        cols[2].set_clip_rect(cols[2].clip_rect().intersect(cols[2].max_rect()));
                        col_rects[2] = render_audio_analyzer_card(&mut cols[2], player, &track, theme, &palette);
                    });

                    // Subtle 1px vertical dark seam dividers between the 3 panels
                    if let (Some(r0), Some(r1), Some(r2)) = (col_rects[0], col_rects[1], col_rects[2]) {
                        let y_min = r0.min.y.min(r1.min.y).min(r2.min.y);
                        let y_max = r0.max.y.max(r1.max.y).max(r2.max.y);

                        let seam1_x = (r0.max.x + r1.min.x) * 0.5;
                        let seam1_rect = Rect::from_min_max(pos2(seam1_x - 0.5, y_min), pos2(seam1_x + 0.5, y_max));
                        ui.painter().rect_filled(seam1_rect, Rounding::ZERO, palette.seam);

                        let seam2_x = (r1.max.x + r2.min.x) * 0.5;
                        let seam2_rect = Rect::from_min_max(pos2(seam2_x - 0.5, y_min), pos2(seam2_x + 0.5, y_max));
                        ui.painter().rect_filled(seam2_rect, Rounding::ZERO, palette.seam);
                    }
                }
            } else {
                ui.centered_and_justified(|ui| {
                    ui.vertical_centered(|ui| {
                        let (icon_rect, _) = ui.allocate_exact_size(vec2(40.0, 40.0), egui::Sense::hover());
                        paint_icon(ui.painter(), Icon::Disc, icon_rect, palette.text_muted);
                        ui.add_space(8.0);
                        ui.label(RichText::new("Nothing Playing").size(20.0).color(palette.text_muted));
                        ui.add_space(8.0);
                        ui.label(RichText::new("Select a song from Songs or Home to start listening.").color(palette.text_secondary));
                    });
                });
            }
        });
}

/// Column 1: Artwork, Track Info, Technical Badges
fn render_track_info_card(
    ui: &mut Ui,
    track: &crate::library::Track,
    palette: &BentoPalette,
    art_manager: &mut AlbumArtManager,
) -> Option<Rect> {
    ui.spacing_mut().item_spacing = vec2(0.0, 12.0);

    let panel_frame = egui::Frame::none()
        .fill(palette.bg_card)
        .rounding(Rounding::same(12.0))
        .stroke(Stroke::NONE)
        .inner_margin(egui::Margin::same(16.0));

    let resp = panel_frame.show(ui, |ui| {
        ui.set_clip_rect(ui.clip_rect().intersect(ui.max_rect()));

        ui.vertical_centered(|ui| {
            let avail_w = ui.available_width();
            let avail_h = ui.available_height();
            let max_h_card = (avail_h - 170.0).max(160.0);
            let card_width = (avail_w - 10.0).clamp(180.0, 260.0).min(max_h_card);
            let art_size = vec2(card_width, card_width);

            // Ambient glow behind album art card matching active accent
            let (rect, _) = ui.allocate_exact_size(art_size, egui::Sense::hover());

            // Soft dark ambient shadow translated downward
            let shadow_rect = rect.translate(vec2(0.0, 4.0));
            ui.painter().rect_filled(
                shadow_rect,
                Rounding::same(12.0),
                Color32::from_black_alpha(70),
            );

            // Album Art Image: rounded egui::Image with 12.0px rounding and subtle stroke
            let mut painted_cover = false;
            if let Some(texture) = art_manager.get_or_load(ui.ctx(), &track.path) {
                let img = egui::Image::new(egui::load::SizedTexture::new(texture.id(), rect.size()))
                    .rounding(Rounding::same(12.0));
                img.paint_at(ui, rect);
                ui.painter().rect_stroke(
                    rect,
                    Rounding::same(12.0),
                    Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(255, 255, 255, 28)),
                );
                painted_cover = true;
            }

            if !painted_cover {
                ui.painter().rect_filled(rect, Rounding::same(12.0), palette.bg_center);
                ui.painter().rect_stroke(
                    rect,
                    Rounding::same(12.0),
                    Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(255, 255, 255, 28)),
                );
                let icon_box = Rect::from_center_size(rect.center(), vec2(art_size.x * 0.36, art_size.x * 0.36));
                paint_icon(ui.painter(), Icon::Disc, icon_box, palette.accent);
            }

            ui.add_space(14.0);

            // Track Title
            ui.label(
                RichText::new(track.display_title())
                    .size(19.0)
                    .color(palette.text_primary)
                    .strong(),
            );
            ui.add_space(2.0);

            // Artist
            ui.label(
                RichText::new(track.display_artist())
                    .size(14.5)
                    .color(palette.text_secondary),
            );
            ui.add_space(2.0);

            // Album and Year
            let album_year = if track.year > 0 {
                format!("{} · {}", track.display_album(), track.year)
            } else {
                track.display_album().to_string()
            };
            ui.label(
                RichText::new(album_year)
                    .size(12.5)
                    .color(palette.text_muted),
            );

            ui.add_space(10.0);

            // Format Badges inside horizontal_wrapped so they wrap onto a second row
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = vec2(6.0, 6.0);

                // High-contrast format badge (FLAC, MP3, etc.)
                format_badge(ui, &track.format, track.is_lossless(), palette);

                // Bit depth & Sample rate badge (e.g. 24-BIT 48 KHZ)
                let bit_depth = if track.is_hires() { "24-BIT" } else { "16-BIT" };
                let sr_khz = if track.sample_rate > 0 {
                    format!("{} KHZ", track.sample_rate / 1000)
                } else {
                    "44.1 KHZ".to_string()
                };
                pill_badge(
                    ui,
                    &format!("{} {}", bit_depth, sr_khz),
                    palette.bg_center,
                    palette.text_secondary,
                );

                // Play count badge
                pill_badge(
                    ui,
                    &format!("{} plays", track.play_count),
                    palette.bg_center,
                    palette.text_secondary,
                );

                // Lossless or Hi-Res badge
                if track.is_hires() {
                    pill_badge(
                        ui,
                        "HI-RES LOSSLESS",
                        Color32::from_rgba_unmultiplied(255, 178, 92, 45),
                        Color32::from_rgb(255, 178, 92),
                    );
                } else if track.is_lossless() {
                    pill_badge(
                        ui,
                        "LOSSLESS",
                        palette.accent_soft,
                        palette.accent,
                    );
                }
            });
        });
    });

    Some(resp.response.rect)
}

/// Column 2: Synced Lyrics with Auto-Scrolling & Click-Seek
fn render_synced_lyrics_card(
    ui: &mut Ui,
    player: &mut PlayerController,
    track: &crate::library::Track,
    pos_ms: u64,
    palette: &BentoPalette,
) -> Option<Rect> {
    let search_open_id = egui::Id::new("now_playing_lyrics_search_open");
    let search_query_id = egui::Id::new("now_playing_lyrics_search_query");
    let track_id_key = egui::Id::new("now_playing_lyrics_current_track_id");
    let romaji_id = egui::Id::new("now_playing_lyrics_show_romaji");

    let mut is_search_open: bool = ui.data(|d| d.get_temp(search_open_id)).unwrap_or(false);
    let mut search_query: String = ui.data(|d| d.get_temp(search_query_id)).unwrap_or_default();
    let mut show_romaji: bool = ui.data(|d| d.get_temp(romaji_id)).unwrap_or(true);
    let last_track_id: Option<i64> = ui.data(|d| d.get_temp(track_id_key));

    if last_track_id != Some(track.id) {
        search_query = format!("{} {}", track.display_title(), track.display_artist()).trim().to_string();
        ui.data_mut(|d| d.insert_temp(track_id_key, track.id));
    }

    if search_query.trim().is_empty() {
        search_query = format!("{} {}", track.display_title(), track.display_artist()).trim().to_string();
    }

    // Zero per-frame conversions: Cache has_romaji per track/lyrics
    let has_romaji_id = egui::Id::new("now_playing_lyrics_has_romaji");
    let has_romaji_hash_id = egui::Id::new("now_playing_lyrics_has_romaji_hash");
    let lyrics_hash = {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        track.id.hash(&mut hasher);
        if let Some(ref lyrics) = player.lyrics {
            lyrics.lines.len().hash(&mut hasher);
            if let Some(first) = lyrics.lines.first() {
                first.timestamp_ms.hash(&mut hasher);
                first.text.hash(&mut hasher);
            }
        }
        hasher.finish()
    };
    let last_hash: Option<u64> = ui.data(|d| d.get_temp(has_romaji_hash_id));
    let has_romaji: bool = if last_hash == Some(lyrics_hash) {
        ui.data(|d| d.get_temp(has_romaji_id)).unwrap_or(false)
    } else {
        let hr = if let Some(ref lrc) = player.lyrics {
            lrc.lines.iter().any(|l| crate::ui::romaji::to_romaji_if_japanese(&l.text).is_some())
        } else {
            false
        };
        ui.data_mut(|d| {
            d.insert_temp(has_romaji_hash_id, lyrics_hash);
            d.insert_temp(has_romaji_id, hr);
        });
        hr
    };

    let panel_frame = egui::Frame::none()
        .fill(palette.bg_card)
        .rounding(Rounding::same(12.0))
        .stroke(Stroke::NONE)
        .inner_margin(egui::Margin::same(16.0));

    let resp = panel_frame.show(ui, |ui| {
        ui.set_clip_rect(ui.clip_rect().intersect(ui.max_rect()));

        let lyrics_width = ui.available_width();

        // Responsive Synced Lyrics Header: When < 310px, place badge + search button on second row
        if lyrics_width < 310.0 {
            ui.horizontal(|ui| {
                let (icon_box, _) = ui.allocate_exact_size(vec2(16.0, 16.0), egui::Sense::hover());
                paint_icon(ui.painter(), Icon::Mic, icon_box, palette.accent);
                ui.add_space(4.0);
                ui.label(
                    RichText::new("SYNCED LYRICS")
                        .size(13.0)
                        .color(palette.text_primary)
                        .strong(),
                );
            });

            ui.add_space(4.0);

            ui.horizontal(|ui| {
                let (badge_text, badge_fill, badge_color) = get_lyrics_badge(player, palette);
                pill_badge_with_stroke(ui, badge_text, badge_fill, badge_color, Some(Stroke::new(1.0_f32, palette.seam)));

                ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                    let btn_resp = search_online_header_button(ui, is_search_open, palette);
                    if btn_resp.clicked() {
                        is_search_open = !is_search_open;
                        if is_search_open && search_query.trim().is_empty() {
                            search_query = format!("{} {}", track.display_title(), track.display_artist()).trim().to_string();
                        }
                    }

                    if has_romaji {
                        ui.add_space(4.0);
                        let romaji_resp = romaji_toggle_button(ui, show_romaji, palette);
                        if romaji_resp.clicked() {
                            show_romaji = !show_romaji;
                        }
                    }
                });
            });
        } else {
            ui.horizontal(|ui| {
                let (icon_box, _) = ui.allocate_exact_size(vec2(16.0, 16.0), egui::Sense::hover());
                paint_icon(ui.painter(), Icon::Mic, icon_box, palette.accent);
                ui.add_space(4.0);
                ui.label(
                    RichText::new("SYNCED LYRICS")
                        .size(13.0)
                        .color(palette.text_primary)
                        .strong(),
                );

                let (badge_text, badge_fill, badge_color) = get_lyrics_badge(player, palette);
                ui.add_space(6.0);
                pill_badge_with_stroke(ui, badge_text, badge_fill, badge_color, Some(Stroke::new(1.0_f32, palette.seam)));

                ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                    let btn_resp = search_online_header_button(ui, is_search_open, palette);
                    if btn_resp.clicked() {
                        is_search_open = !is_search_open;
                        if is_search_open && search_query.trim().is_empty() {
                            search_query = format!("{} {}", track.display_title(), track.display_artist()).trim().to_string();
                        }
                    }

                    if has_romaji {
                        ui.add_space(4.0);
                        let romaji_resp = romaji_toggle_button(ui, show_romaji, palette);
                        if romaji_resp.clicked() {
                            show_romaji = !show_romaji;
                        }
                    }
                });
            });
        }

        seam_divider(ui, palette);

        if is_search_open {
            let mut picked_record = None;
            let mut trigger_search_query = None;

            // Inline LRCLIB Search Drawer inside Synced Lyrics card with Stroke::NONE
            let drawer_card = egui::Frame::none()
                .fill(palette.bg_center)
                .rounding(Rounding::same(10.0))
                .stroke(Stroke::NONE)
                .inner_margin(egui::Margin::same(12.0));

            drawer_card.show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing = vec2(0.0, 8.0);

                    // Header: "Search LRCLIB Online" and vector Close button
                    ui.horizontal(|ui| {
                        let (s_icon_box, _) = ui.allocate_exact_size(vec2(14.0, 14.0), egui::Sense::hover());
                        paint_icon(ui.painter(), Icon::Search, s_icon_box, palette.accent);
                        ui.add_space(4.0);
                        ui.label(
                            RichText::new("Search LRCLIB Online")
                                .size(13.0)
                                .color(palette.text_primary)
                                .strong(),
                        );

                        ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                            let close_resp = vector_icon_button(ui, Icon::Close, 14.0, palette.text_muted);
                            if close_resp.hovered() {
                                close_resp.clone().on_hover_text("Close Search Drawer");
                            }
                            if close_resp.clicked() {
                                is_search_open = false;
                            }
                        });
                    });

                    // Search input row: Text input + "Search" button
                    ui.horizontal(|ui| {
                        let btn_w = 72.0;
                        let input_w = (ui.available_width() - btn_w - 8.0).max(100.0);

                        let mut enter_pressed = false;
                        let input_frame = egui::Frame::none()
                            .fill(palette.bg_sidebar)
                            .rounding(Rounding::same(6.0))
                            .stroke(Stroke::NONE)
                            .inner_margin(egui::Margin::symmetric(8.0, 4.0));

                        input_frame.show(ui, |ui| {
                            let edit_resp = ui.add(
                                egui::TextEdit::singleline(&mut search_query)
                                    .hint_text("Title Artist...")
                                    .desired_width(input_w)
                                    .text_color(palette.text_primary)
                                    .frame(false),
                            );
                            if edit_resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                                enter_pressed = true;
                            }
                        });

                        let search_btn = egui::Button::new(
                            RichText::new("Search")
                                .color(Color32::from_rgb(11, 9, 20))
                                .size(12.5)
                                .strong(),
                        )
                        .fill(palette.accent)
                        .stroke(Stroke::NONE)
                        .rounding(Rounding::same(6.0))
                        .min_size(vec2(btn_w, 28.0));

                        let search_clicked = ui.add(search_btn).clicked();

                        if (search_clicked || enter_pressed) && !search_query.trim().is_empty() {
                            trigger_search_query = Some(search_query.trim().to_string());
                        }
                    });

                    seam_divider(ui, palette);

                    // Drawer Body: Searching, Error, Results, or Empty prompt
                    if player.is_searching_lyrics {
                        ui.add_space(24.0);
                        ui.vertical_centered(|ui| {
                            let (spk_box, _) = ui.allocate_exact_size(vec2(20.0, 20.0), egui::Sense::hover());
                            paint_icon(ui.painter(), Icon::Sparkle, spk_box, palette.accent);
                            ui.add_space(8.0);
                            ui.label(
                                RichText::new("Searching LRCLIB online...")
                                    .color(palette.text_secondary)
                                    .size(13.0),
                            );
                        });
                        ui.add_space(24.0);
                    } else if let Some(ref err) = player.manual_search_error {
                        ui.add_space(16.0);
                        ui.vertical_centered(|ui| {
                            ui.label(
                                RichText::new(format!("Search failed: {}", err))
                                    .color(Color32::from_rgb(248, 113, 113))
                                    .size(12.5),
                            );
                        });
                        ui.add_space(16.0);
                    } else if let Some(ref results) = player.manual_search_results {
                        if results.is_empty() {
                            ui.add_space(24.0);
                            ui.vertical_centered(|ui| {
                                ui.label(
                                    RichText::new("No matching lyrics found on LRCLIB.")
                                        .color(palette.text_muted)
                                        .size(13.0),
                                );
                                ui.add_space(4.0);
                                ui.label(
                                    RichText::new("Try adjusting the title or artist in your search query.")
                                        .color(palette.text_muted)
                                        .size(11.5),
                                );
                            });
                            ui.add_space(24.0);
                        } else {
                            ScrollArea::vertical()
                                .id_salt("manual_lyrics_search_results_scroll")
                                .auto_shrink([false, false])
                                .show(ui, |ui| {
                                    ui.spacing_mut().item_spacing = vec2(0.0, 6.0);

                                    for record in results {
                                        let dur_str = record
                                            .duration
                                            .map(format_time_secs)
                                            .unwrap_or_else(|| "--:--".to_string());

                                        let album_display = record
                                            .album_name
                                            .as_deref()
                                            .unwrap_or("Unknown Album");

                                        let title_display = if !record.track_name.is_empty() {
                                            &record.track_name
                                        } else if let Some(ref n) = record.name {
                                            n
                                        } else {
                                            "Unknown Title"
                                        };

                                        let item_frame = egui::Frame::none()
                                            .fill(palette.bg_card)
                                            .stroke(Stroke::NONE)
                                            .rounding(Rounding::same(8.0))
                                            .inner_margin(egui::Margin::symmetric(10.0, 8.0));

                                        let item_resp = item_frame.show(ui, |ui| {
                                            ui.horizontal(|ui| {
                                                ui.vertical(|ui| {
                                                    ui.spacing_mut().item_spacing = vec2(0.0, 2.0);
                                                    ui.label(
                                                        RichText::new(title_display)
                                                            .size(13.0)
                                                            .color(palette.text_primary)
                                                            .strong(),
                                                    );

                                                    let sub_str = format!(
                                                        "{} · {} · {}",
                                                        record.artist_name, album_display, dur_str
                                                    );
                                                    ui.label(
                                                        RichText::new(sub_str)
                                                            .size(11.0)
                                                            .color(palette.text_muted),
                                                    );
                                                });

                                                ui.with_layout(
                                                    Layout::right_to_left(egui::Align::Center),
                                                    |ui| {
                                                        if record
                                                            .synced_lyrics
                                                            .as_ref()
                                                            .is_some_and(|s| !s.trim().is_empty())
                                                        {
                                                            pill_badge(
                                                                ui,
                                                                "SYNCED",
                                                                palette.accent_soft,
                                                                palette.accent,
                                                            );
                                                        } else if record.instrumental {
                                                            pill_badge(
                                                                ui,
                                                                "INSTRUMENTAL",
                                                                Color32::from_rgba_unmultiplied(255, 178, 92, 45),
                                                                Color32::from_rgb(255, 178, 92),
                                                            );
                                                        } else {
                                                            pill_badge(
                                                                ui,
                                                                "PLAIN",
                                                                palette.bg_sidebar,
                                                                palette.text_muted,
                                                            );
                                                        }
                                                    },
                                                );
                                            });
                                        });

                                        let interact = item_resp.response.interact(egui::Sense::click());
                                        if interact.hovered() {
                                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                                            ui.painter().rect_filled(
                                                item_resp.response.rect,
                                                Rounding::same(8.0),
                                                palette.bg_card_hover,
                                            );
                                        }
                                        if interact.clicked() {
                                            picked_record = Some(record.clone());
                                        }
                                    }
                                });
                        }
                    } else {
                        // Initial prompt inside drawer
                        ui.add_space(16.0);
                        ui.vertical_centered(|ui| {
                            ui.label(
                                RichText::new("Enter title or artist to search LRCLIB online.")
                                    .color(palette.text_muted)
                                    .size(12.5),
                            );
                            ui.add_space(8.0);
                            let quick_search_btn = egui::Button::new(
                                RichText::new(format!("Search \"{}\"", search_query.trim()))
                                    .color(palette.text_primary)
                                    .size(12.0),
                            )
                            .fill(palette.bg_sidebar)
                            .stroke(Stroke::NONE)
                            .rounding(Rounding::same(6.0))
                            .min_size(vec2(140.0, 28.0));

                            if ui.add(quick_search_btn).clicked() && !search_query.trim().is_empty() {
                                trigger_search_query = Some(search_query.trim().to_string());
                            }
                        });
                        ui.add_space(16.0);
                    }
                });
            });

            if let Some(q) = trigger_search_query {
                player.search_lyrics_manual(
                    &q,
                    Some(&track.title),
                    Some(&track.artist),
                );
            }
            if let Some(rec) = picked_record {
                player.apply_custom_lyrics(&rec);
                is_search_open = false;
            }
        } else if player.is_loading_lyrics {
            ui.add_space(48.0);
            ui.vertical_centered(|ui| {
                let (icon_rect, _) = ui.allocate_exact_size(vec2(24.0, 24.0), egui::Sense::hover());
                paint_icon(ui.painter(), Icon::Sparkle, icon_rect, palette.accent);
                ui.add_space(10.0);
                ui.label(
                    RichText::new("Searching LRCLIB...")
                        .color(palette.text_secondary)
                        .size(14.0),
                );
                ui.add_space(4.0);
                ui.label(
                    RichText::new("Looking for synced lyrics online")
                        .color(palette.text_muted)
                        .size(12.0),
                );
            });
        } else if let Some(ref lyrics) = player.lyrics {
            if lyrics.lines.is_empty() {
                render_lyrics_empty_state(ui, palette, track, &mut is_search_open, &mut search_query);
            } else if lyrics.source.contains("Instrumental")
                || (lyrics.lines.len() == 1 && lyrics.lines[0].text.eq_ignore_ascii_case("instrumental"))
            {
                ui.add_space(40.0);
                ui.vertical_centered(|ui| {
                    let (icon_rect, _) = ui.allocate_exact_size(vec2(32.0, 32.0), egui::Sense::hover());
                    paint_icon(ui.painter(), Icon::Music, icon_rect, Color32::from_rgb(255, 178, 92));
                    ui.add_space(10.0);
                    ui.label(
                        RichText::new("Instrumental Track")
                            .size(15.0)
                            .color(palette.text_primary)
                            .strong(),
                    );
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new("This song does not contain vocal lyrics.")
                            .size(12.5)
                            .color(palette.text_muted),
                    );
                    ui.add_space(16.0);
                    let search_btn = egui::Button::new(
                        RichText::new("Search LRCLIB Anyway")
                            .color(palette.text_primary)
                            .size(12.5),
                    )
                    .fill(palette.bg_center)
                    .stroke(Stroke::NONE)
                    .rounding(Rounding::same(8.0))
                    .min_size(vec2(150.0, 30.0));

                    if ui.add(search_btn).clicked() {
                        is_search_open = true;
                        if search_query.trim().is_empty() {
                            search_query = format!("{} {}", track.display_title(), track.display_artist()).trim().to_string();
                        }
                    }
                });
            } else if lyrics.is_synced {
                if let Some(target) = render_synced_lyrics_list(
                    ui,
                    "synced_lyrics_scroll",
                    &lyrics.lines,
                    pos_ms,
                    palette,
                    show_romaji,
                ) {
                    seek_to(player, None, target);
                }
            } else {
                // Plain (unsynced) lyrics
                ScrollArea::vertical()
                    .id_salt("synced_lyrics_scroll")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.spacing_mut().item_spacing = vec2(0.0, 6.0);

                        for line in &lyrics.lines {
                            if line.text.trim().is_empty() {
                                ui.add_space(10.0);
                            } else {
                                ui.label(
                                    RichText::new(&line.text)
                                        .color(palette.text_primary)
                                        .size(14.5)
                                        .line_height(Some(22.0)),
                                );
                            }
                        }
                    });
            }
        } else {
            // Empty state: lyrics is None
            render_lyrics_empty_state(ui, palette, track, &mut is_search_open, &mut search_query);
        }
    });

    // Persist search drawer state in ui temp data
    ui.data_mut(|d| {
        d.insert_temp(search_open_id, is_search_open);
        d.insert_temp(search_query_id, search_query);
        d.insert_temp(romaji_id, show_romaji);
    });

    Some(resp.response.rect)
}

/// Column 3: Audio Analyzer Card (Stats, Metrics, Meters, Spectrum Visualizer)
fn render_audio_analyzer_card(
    ui: &mut Ui,
    player: &mut PlayerController,
    track: &crate::library::Track,
    theme: &Theme,
    palette: &BentoPalette,
) -> Option<Rect> {
    ui.spacing_mut().item_spacing = vec2(0.0, 10.0);

    let panel_frame = egui::Frame::none()
        .fill(palette.bg_card)
        .rounding(Rounding::same(12.0))
        .stroke(Stroke::NONE)
        .inner_margin(egui::Margin::same(14.0));

    let resp = panel_frame.show(ui, |ui| {
        ui.set_clip_rect(ui.clip_rect().intersect(ui.max_rect()));

        // 1. Format Header: When width < 300px, place badge cleanly below AUDIO ANALYZER heading
        let card_w = ui.available_width();
        if card_w < 300.0 {
            ui.horizontal(|ui| {
                render_icon(ui, Icon::BarChart, 14.0, palette.accent);
                ui.add_space(4.0);
                ui.label(
                    RichText::new("AUDIO ANALYZER")
                        .size(12.5)
                        .color(palette.text_secondary)
                        .strong(),
                );
            });
            ui.add_space(4.0);
            render_audio_analyzer_badge(ui, track, palette);
        } else {
            ui.horizontal(|ui| {
                render_icon(ui, Icon::BarChart, 14.0, palette.accent);
                ui.add_space(4.0);
                ui.label(
                    RichText::new("AUDIO ANALYZER")
                        .size(12.5)
                        .color(palette.text_secondary)
                        .strong(),
                );

                ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                    render_audio_analyzer_badge(ui, track, palette);
                });
            });
        }

        ui.add_space(8.0);

        // 2. Technical Stats Sub-Card with Stroke::NONE
        let specs_card = egui::Frame::none()
            .fill(palette.bg_center)
            .rounding(Rounding::same(8.0))
            .stroke(Stroke::NONE)
            .inner_margin(egui::Margin::same(8.0));

        specs_card.show(ui, |ui| {
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing = vec2(0.0, 4.0);

                stat_row(ui, "Format / Codec", &track.format.to_uppercase(), palette);

                let sr_val = if track.sample_rate > 0 {
                    format!("{} Hz", track.sample_rate)
                } else {
                    "44,100 Hz".to_string()
                };
                stat_row(ui, "Sample Rate", &sr_val, palette);

                let bit_depth = if track.is_hires() { "24-bit" } else { "16-bit" };
                let br_val = if track.bitrate_kbps > 0 {
                    format!("{} · {} kbps", bit_depth, track.bitrate_kbps)
                } else {
                    format!("{} · Lossless", bit_depth)
                };
                stat_row(ui, "Bit Depth / Rate", &br_val, palette);

                let ch_val = if track.channels == 1 {
                    "Mono (1.0 ch)"
                } else {
                    "Stereo (2.0 ch)"
                };
                stat_row(ui, "Channels", ch_val, palette);
            });
        });

        ui.add_space(8.0);

        // 3. Real-Time Audio Metrics Sub-Card (Peak dBFS, LUFS, Dynamic Range)
        let metrics_card = egui::Frame::none()
            .fill(palette.bg_center)
            .rounding(Rounding::same(8.0))
            .stroke(Stroke::NONE)
            .inner_margin(egui::Margin::same(8.0));

        metrics_card.show(ui, |ui| {
            ui.horizontal(|ui| {
                let col_w = (ui.available_width() - 8.0) / 3.0;

                // Metric 1: Peak Level (dBFS)
                ui.allocate_ui_with_layout(vec2(col_w, 42.0), Layout::top_down(egui::Align::Center), |ui| {
                    ui.label(RichText::new("PEAK").size(9.5).color(palette.text_muted).strong());
                    let peak_str = if player.visualizer.peak_dbfs <= -55.0 {
                        "-∞ dBFS".to_string()
                    } else {
                        format!("{:.1} dBFS", player.visualizer.peak_dbfs)
                    };
                    let peak_col = if player.visualizer.peak_dbfs > -1.0 {
                        Color32::from_rgb(255, 178, 92)
                    } else {
                        palette.text_primary
                    };
                    ui.label(RichText::new(peak_str).size(12.5).color(peak_col).strong());
                });

                // Metric 2: LUFS
                ui.allocate_ui_with_layout(vec2(col_w, 42.0), Layout::top_down(egui::Align::Center), |ui| {
                    ui.label(RichText::new("LUFS").size(9.5).color(palette.text_muted).strong());
                    let lufs_str = if player.visualizer.lufs <= -55.0 {
                        "-∞".to_string()
                    } else {
                        format!("{:.1}", player.visualizer.lufs)
                    };
                    ui.label(RichText::new(lufs_str).size(12.5).color(palette.text_primary).strong());
                });

                // Metric 3: Dynamic Range
                ui.allocate_ui_with_layout(vec2(col_w, 42.0), Layout::top_down(egui::Align::Center), |ui| {
                    ui.label(RichText::new("DYN RANGE").size(9.5).color(palette.text_muted).strong());
                    let dr_str = if player.visualizer.dynamic_range < 1.0 {
                        "--".to_string()
                    } else {
                        format!("DR {:.0}", player.visualizer.dynamic_range)
                    };
                    ui.label(RichText::new(dr_str).size(12.5).color(palette.accent).strong());
                });
            });
        });

        ui.add_space(8.0);

        // 4. Live L/R Stereo Volume Level Meters with palette.accent
        let meter_card = egui::Frame::none()
            .fill(palette.bg_center)
            .rounding(Rounding::same(8.0))
            .stroke(Stroke::NONE)
            .inner_margin(egui::Margin::same(8.0));

        meter_card.show(ui, |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("STEREO METERS").size(10.0).color(palette.text_muted).strong());
                    ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(RichText::new("0 dB").size(9.0).color(palette.text_muted));
                    });
                });

                ui.add_space(4.0);
                render_channel_meter(ui, "L", player.visualizer.left_level, player.visualizer.left_peak, palette);
                ui.add_space(3.0);
                render_channel_meter(ui, "R", player.visualizer.right_level, player.visualizer.right_peak, palette);
            });
        });

        ui.add_space(8.0);

        // 5. 20Hz - 20kHz Responsive Audio Spectrum Visualizer
        let spec_card = egui::Frame::none()
            .fill(palette.bg_center)
            .rounding(Rounding::same(8.0))
            .stroke(Stroke::NONE)
            .inner_margin(egui::Margin::same(8.0));

        spec_card.show(ui, |ui| {
            ui.vertical(|ui| {
                let spec_w = ui.available_width();
                // Responsive switcher: When < 290px, stack switcher cleanly below SPECTRUM heading
                if spec_w < 290.0 {
                    ui.label(RichText::new("SPECTRUM").size(10.0).color(palette.text_muted).strong());
                    ui.add_space(3.0);
                    ui.horizontal(|ui| {
                        if segmented_tab_button(ui, "Bars", player.visualizer.style == VisualizerStyle::Bars, palette).clicked() {
                            player.visualizer.style = VisualizerStyle::Bars;
                        }
                        if segmented_tab_button(ui, "Mirror", player.visualizer.style == VisualizerStyle::Mirror, palette).clicked() {
                            player.visualizer.style = VisualizerStyle::Mirror;
                        }
                        if segmented_tab_button(ui, "Scope", player.visualizer.style == VisualizerStyle::Scope, palette).clicked() {
                            player.visualizer.style = VisualizerStyle::Scope;
                        }
                    });
                } else {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("SPECTRUM").size(10.0).color(palette.text_muted).strong());
                        ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                            if segmented_tab_button(ui, "Bars", player.visualizer.style == VisualizerStyle::Bars, palette).clicked() {
                                player.visualizer.style = VisualizerStyle::Bars;
                            }
                            if segmented_tab_button(ui, "Mirror", player.visualizer.style == VisualizerStyle::Mirror, palette).clicked() {
                                player.visualizer.style = VisualizerStyle::Mirror;
                            }
                            if segmented_tab_button(ui, "Scope", player.visualizer.style == VisualizerStyle::Scope, palette).clicked() {
                                player.visualizer.style = VisualizerStyle::Scope;
                            }
                        });
                    });
                }

                ui.add_space(4.0);

                // Responsive Spectrum Visualizer Canvas using palette.accent
                let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 76.0), egui::Sense::hover());
                render_spectrum(
                    ui.painter(),
                    rect,
                    &player.visualizer.band_values,
                    &player.visualizer.peak_values,
                    &player.visualizer.wave_buffer,
                    player.visualizer.style,
                    palette.accent,
                    theme,
                );

                ui.add_space(4.0);

                // Frequency Axis Markers (20Hz - 20kHz) dynamically thinned out based on width
                let (axis_rect, _) = ui.allocate_exact_size(vec2(rect.width(), 16.0), egui::Sense::hover());
                let painter = ui.painter();
                let freq_markers = frequency_axis_markers(axis_rect.width());

                for &(lbl, fraction) in freq_markers {
                    let x = axis_rect.min.x + fraction * axis_rect.width();

                    // Subtle seam tick mark connecting spectrum to frequency label
                    painter.line_segment(
                        [pos2(x, axis_rect.min.y), pos2(x, axis_rect.min.y + 3.0)],
                        Stroke::new(1.0_f32, palette.seam),
                    );

                    let align = if fraction <= 0.01 {
                        egui::Align2::LEFT_TOP
                    } else if fraction >= 0.99 {
                        egui::Align2::RIGHT_TOP
                    } else {
                        egui::Align2::CENTER_TOP
                    };

                    painter.text(
                        pos2(x, axis_rect.min.y + 4.0),
                        align,
                        lbl,
                        egui::FontId::proportional(9.0),
                        palette.text_muted,
                    );
                }
            });
        });
    });

    Some(resp.response.rect)
}

/// Dynamically thins out spectrum frequency markers based on available width:
/// Wide (>= 280px): all 7 markers (20Hz, 100Hz, 500Hz, 1kHz, 5kHz, 10kHz, 20kHz).
/// Narrow (< 280px): spaced anchors (20Hz, 500Hz, 2kHz, 20kHz).
pub fn frequency_axis_markers(width: f32) -> &'static [(&'static str, f32)] {
    if width >= 280.0 {
        &[
            ("20Hz", 0.00),
            ("100Hz", 0.17),
            ("500Hz", 0.36),
            ("1kHz", 0.52),
            ("5kHz", 0.70),
            ("10kHz", 0.86),
            ("20kHz", 1.00),
        ]
    } else {
        &[
            ("20Hz", 0.00),
            ("500Hz", 0.36),
            ("2kHz", 0.60),
            ("20kHz", 1.00),
        ]
    }
}

fn render_audio_analyzer_badge(ui: &mut Ui, track: &crate::library::Track, palette: &BentoPalette) {
    if track.is_hires() {
        pill_badge(
            ui,
            "HI-RES LOSSLESS",
            Color32::from_rgba_unmultiplied(255, 178, 92, 45),
            Color32::from_rgb(255, 178, 92),
        );
    } else if track.is_lossless() {
        pill_badge(
            ui,
            "LOSSLESS",
            palette.accent_soft,
            palette.accent,
        );
    } else {
        pill_badge(
            ui,
            &track.format.to_uppercase(),
            palette.bg_center,
            palette.text_secondary,
        );
    }
}

/// Subtle 1.0px dark seam divider line
fn seam_divider(ui: &mut Ui, palette: &BentoPalette) {
    ui.add_space(3.0);
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 1.0), egui::Sense::hover());
    ui.painter().rect_filled(rect, Rounding::ZERO, palette.seam);
    ui.add_space(3.0);
}

/// Borderless pill badge helper matching matte-slate design
fn pill_badge(ui: &mut Ui, text: &str, fill: Color32, text_color: Color32) {
    pill_badge_with_stroke(ui, text, fill, text_color, None);
}

/// Pill badge helper with optional stroke
fn pill_badge_with_stroke(ui: &mut Ui, text: &str, fill: Color32, text_color: Color32, stroke: Option<Stroke>) {
    let padding = vec2(7.5, 2.5);
    let galley = ui.painter().layout_no_wrap(
        text.to_string(),
        egui::FontId::proportional(11.0),
        text_color,
    );
    let desired_size = galley.size() + padding * 2.0;
    let (rect, _) = ui.allocate_exact_size(desired_size, egui::Sense::hover());

    ui.painter().rect_filled(rect, Rounding::same(6.0), fill);
    if let Some(s) = stroke {
        ui.painter().rect_stroke(rect, Rounding::same(6.0), s);
    }
    ui.painter().galley(rect.min + padding, galley, text_color);
}

/// Borderless format badge (e.g. FLAC, MP3, AAC)
fn format_badge(ui: &mut Ui, format: &str, is_lossless: bool, palette: &BentoPalette) {
    let fmt_str = if format.trim().is_empty() {
        "AUDIO"
    } else {
        format.trim()
    }
    .to_uppercase();

    let text_color = if is_lossless {
        palette.text_primary
    } else {
        palette.text_muted
    };

    let padding = vec2(6.0, 2.0);
    let galley = ui.painter().layout_no_wrap(
        fmt_str,
        egui::FontId::proportional(10.5),
        text_color,
    );
    let desired_size = galley.size() + padding * 2.0;
    let (rect, _) = ui.allocate_exact_size(desired_size, egui::Sense::hover());

    ui.painter().rect_filled(rect, Rounding::same(4.0), palette.bg_center);
    ui.painter().galley(rect.min + padding, galley, text_color);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisplayLyricLine {
    pub timestamp_ms: u64,
    pub text: String,
    pub is_instrumental: bool,
    pub romaji: Option<String>,
}

pub fn is_line_instrumental(text: &str) -> bool {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return true;
    }
    if trimmed.eq_ignore_ascii_case("instrumental")
        || trimmed.eq_ignore_ascii_case("(instrumental)")
        || trimmed.eq_ignore_ascii_case("[instrumental]")
        || trimmed.chars().all(|c| {
            c == '.' || c == '\u{2022}' || c == '-' || c == '~' || c == ' '
                || c == '\u{266a}'
                || c == '\u{266b}'
                || c == '\u{2669}'
                || c == '\u{266c}'
        })
    {
        return true;
    }
    false
}

pub fn build_display_lyrics(lines: &[LyricLine]) -> Vec<DisplayLyricLine> {
    if lines.is_empty() {
        return Vec::new();
    }

    let mut result = Vec::with_capacity(lines.len() + 4);

    // 1. Check for massive intro gap (> 10s) before first lyric line
    if let Some(first) = lines.first() {
        if first.timestamp_ms > 10_000 && !is_line_instrumental(&first.text) {
            result.push(DisplayLyricLine {
                timestamp_ms: 0,
                text: String::new(),
                is_instrumental: true,
                romaji: None,
            });
        }
    }

    // 2. Iterate through lines and inject instrumental lines for empty lines or gaps > 10s
    for i in 0..lines.len() {
        let current = &lines[i];
        let is_inst = is_line_instrumental(&current.text);
        let romaji = if is_inst {
            None
        } else {
            crate::ui::romaji::to_romaji_if_japanese(&current.text)
        };

        result.push(DisplayLyricLine {
            timestamp_ms: current.timestamp_ms,
            text: current.text.clone(),
            is_instrumental: is_inst,
            romaji,
        });

        if i + 1 < lines.len() {
            let next = &lines[i + 1];
            let gap = next.timestamp_ms.saturating_sub(current.timestamp_ms);
            // If there is a massive gap (> 10s) and neither line is already an instrumental break
            if gap > 10_000 && !is_inst && !is_line_instrumental(&next.text) {
                let mid_ts = current.timestamp_ms + gap / 2;
                result.push(DisplayLyricLine {
                    timestamp_ms: mid_ts,
                    text: String::new(),
                    is_instrumental: true,
                    romaji: None,
                });
            }
        }
    }

    result
}

pub fn current_display_index(lines: &[DisplayLyricLine], pos_ms: u64) -> Option<usize> {
    if lines.is_empty() {
        return None;
    }
    let mut idx = 0;
    let mut found = false;
    for (i, line) in lines.iter().enumerate() {
        if pos_ms >= line.timestamp_ms {
            idx = i;
            found = true;
        } else {
            break;
        }
    }
    if found {
        Some(idx)
    } else {
        None
    }
}

#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct NowPlayingLyricsScrollState {
    pub current_y: f32,
    pub target_y: f32,
    pub last_active_idx: Option<usize>,
    pub manual_scroll_until: f64,
    pub last_track_key: u64,
}

/// Linearly interpolates opaque RGB channels between background and foreground colors.
#[inline]
pub fn blend_rgb(bg: Color32, fg: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    let r = (bg.r() as f32 + (fg.r() as f32 - bg.r() as f32) * t).round() as u8;
    let g = (bg.g() as f32 + (fg.g() as f32 - bg.g() as f32) * t).round() as u8;
    let b = (bg.b() as f32 + (fg.b() as f32 - bg.b() as f32) * t).round() as u8;
    Color32::from_rgb(r, g, b)
}

/// Multiplies the alpha channel of a color by an edge fade factor (0.0..=1.0).
#[inline]
pub fn apply_edge_alpha(c: Color32, edge_alpha: f32) -> Color32 {
    let factor = edge_alpha.clamp(0.0, 1.0);
    let new_a = (c.a() as f32 * factor).round() as u8;
    Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), new_a)
}

pub fn render_synced_lyrics_list(
    ui: &mut Ui,
    scroll_id: &str,
    lines: &[LyricLine],
    pos_ms: u64,
    palette: &BentoPalette,
    show_romaji: bool,
) -> Option<f64> {
    // Zero per-frame conversions: memoize display lines in ui.data
    let cache_id = egui::Id::new("now_playing_cached_display_lyrics");
    let cache_hash_id = egui::Id::new("now_playing_cached_display_lyrics_hash");
    let hash = {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        lines.len().hash(&mut hasher);
        if let Some(first) = lines.first() {
            first.timestamp_ms.hash(&mut hasher);
            first.text.hash(&mut hasher);
        }
        hasher.finish()
    };
    let last_hash: Option<u64> = ui.data(|d| d.get_temp(cache_hash_id));
    let display_lines: std::sync::Arc<Vec<DisplayLyricLine>> = if last_hash == Some(hash) {
        if let Some(cached) = ui.data(|d| d.get_temp::<std::sync::Arc<Vec<DisplayLyricLine>>>(cache_id)) {
            cached
        } else {
            let built = std::sync::Arc::new(build_display_lyrics(lines));
            ui.data_mut(|d| {
                d.insert_temp(cache_id, built.clone());
                d.insert_temp(cache_hash_id, hash);
            });
            built
        }
    } else {
        let built = std::sync::Arc::new(build_display_lyrics(lines));
        ui.data_mut(|d| {
            d.insert_temp(cache_id, built.clone());
            d.insert_temp(cache_hash_id, hash);
        });
        built
    };

    let current_idx = current_display_index(&display_lines, pos_ms);
    let mut clicked_seek_secs = None;
    let mut detected_target_y: Option<f32> = None;

    let viewport_h = ui.available_height().max(1.0);
    let scroll_state_id = egui::Id::new(scroll_id).with("now_playing_scroll_state");
    let mut scroll_state: NowPlayingLyricsScrollState = ui
        .data(|d| d.get_temp(scroll_state_id))
        .unwrap_or_default();

    let now_secs = ui.input(|i| i.time);
    let track_changed = scroll_state.last_track_key != hash;
    if track_changed {
        scroll_state.last_track_key = hash;
        scroll_state.manual_scroll_until = 0.0;
        scroll_state.last_active_idx = None;
    }

    let scroll_output = ScrollArea::vertical()
        .id_salt(scroll_id)
        .vertical_scroll_offset(scroll_state.current_y)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let content_origin_y = ui.cursor().top();
            let vp = ui.clip_rect();
            ui.add_space(48.0);
            ui.spacing_mut().item_spacing = vec2(0.0, 14.0);

            for (i, line) in display_lines.iter().enumerate() {
                let is_current = current_idx == Some(i);
                let is_past = current_idx.is_some_and(|act| i < act);
                let dist = match current_idx {
                    Some(act) => (i as isize - act as isize).unsigned_abs(),
                    None => i,
                };

                let pad_v = 10.0_f32;
                let pad_h = 14.0_f32;
                let avail_w = ui.available_width();
                let wrap_width = (avail_w - pad_h * 2.0 - 12.0).max(100.0);

                if line.is_instrumental {
                    let icon_size = if is_current { 16.0 } else { 12.0 };
                    let desired_size = vec2(avail_w, icon_size + pad_v * 2.0);
                    let (rect, resp) = ui.allocate_exact_size(desired_size, egui::Sense::click());

                    let line_top = rect.min.y;
                    let line_bottom = rect.max.y;
                    let top_fade = ((line_top - (vp.top() + 14.0)) / 62.0).clamp(0.0, 1.0);
                    let bot_fade = (((vp.bottom() - 14.0) - line_bottom) / 62.0).clamp(0.0, 1.0);
                    let edge_alpha = top_fade.min(bot_fade).powf(1.3);

                    if is_current {
                        let local_center_y = rect.center().y - content_origin_y;
                        detected_target_y = Some((local_center_y - viewport_h * 0.38).max(0.0));
                    }

                    if edge_alpha > 0.01 {
                        if resp.hovered() {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                        }

                        let (icon_color, fill) = if is_current {
                            (
                                Color32::WHITE,
                                blend_rgb(palette.bg_center, palette.accent, 0.11),
                            )
                        } else if resp.hovered() {
                            (
                                Color32::from_white_alpha(225),
                                Color32::from_white_alpha(10),
                            )
                        } else if is_past {
                            (
                                Color32::from_white_alpha(85),
                                Color32::TRANSPARENT,
                            )
                        } else if dist <= 2 {
                            (
                                Color32::from_white_alpha(155),
                                Color32::TRANSPARENT,
                            )
                        } else if dist == 3 {
                            (
                                Color32::from_white_alpha(90),
                                Color32::TRANSPARENT,
                            )
                        } else {
                            (
                                Color32::from_white_alpha(70),
                                Color32::TRANSPARENT,
                            )
                        };

                        let icon_color = apply_edge_alpha(icon_color, edge_alpha);
                        let fill = apply_edge_alpha(fill, edge_alpha);

                        if fill != Color32::TRANSPARENT {
                            ui.painter().rect_filled(rect, Rounding::same(10.0), fill);
                        }

                        if is_current {
                            let bar_h = (rect.height() - 12.0).max(14.0);
                            let bar_rect = Rect::from_min_size(
                                pos2(rect.min.x + 4.0, rect.center().y - bar_h * 0.5),
                                vec2(3.0, bar_h),
                            );
                            ui.painter().rect_filled(
                                bar_rect,
                                Rounding::same(1.5),
                                apply_edge_alpha(palette.accent, edge_alpha),
                            );
                        }

                        let icon_rect = Rect::from_center_size(
                            pos2(rect.min.x + pad_h + icon_size * 0.5, rect.center().y),
                            vec2(icon_size, icon_size),
                        );
                        paint_icon(ui.painter(), Icon::Music, icon_rect, icon_color);

                        if resp.clicked() {
                            clicked_seek_secs = Some(line.timestamp_ms as f64 / 1000.0);
                        }
                    }
                } else {
                    let has_romaji_line = show_romaji && line.romaji.is_some();
                    let (main_size, base_main_color, romaji_size, base_romaji_color) = if is_current {
                        (
                            20.5_f32,
                            Color32::WHITE,
                            12.5_f32,
                            Color32::from_white_alpha(190),
                        )
                    } else if is_past {
                        (
                            15.0_f32,
                            Color32::from_white_alpha(85),
                            11.5_f32,
                            Color32::from_white_alpha(55),
                        )
                    } else if dist <= 2 {
                        (
                            15.0_f32,
                            Color32::from_white_alpha(155),
                            11.5_f32,
                            Color32::from_white_alpha(95),
                        )
                    } else if dist == 3 {
                        (
                            14.5_f32,
                            Color32::from_white_alpha(90),
                            11.0_f32,
                            Color32::from_white_alpha(60),
                        )
                    } else {
                        (
                            14.0_f32,
                            Color32::from_white_alpha(70),
                            10.5_f32,
                            Color32::from_white_alpha(45),
                        )
                    };

                    let mut main_rich = RichText::new(&line.text)
                        .size(main_size)
                        .color(base_main_color);
                    if is_current {
                        main_rich = main_rich.strong();
                    }
                    let main_widget: egui::WidgetText = main_rich.into();
                    let main_galley = main_widget.into_galley(
                        ui,
                        Some(egui::TextWrapMode::Wrap),
                        wrap_width,
                        egui::TextStyle::Body,
                    );

                    let romaji_galley = if has_romaji_line {
                        line.romaji.as_deref().map(|romaji_str| {
                            let r_rich = RichText::new(romaji_str)
                                .size(romaji_size)
                                .color(base_romaji_color);
                            let r_widget: egui::WidgetText = r_rich.into();
                            r_widget.into_galley(
                                ui,
                                Some(egui::TextWrapMode::Wrap),
                                wrap_width,
                                egui::TextStyle::Body,
                            )
                        })
                    } else {
                        None
                    };

                    let text_h = if let Some(ref rg) = romaji_galley {
                        main_galley.size().y + 4.0 + rg.size().y
                    } else {
                        main_galley.size().y
                    };

                    let desired_size = vec2(avail_w, text_h + pad_v * 2.0);
                    let (rect, resp) = ui.allocate_exact_size(desired_size, egui::Sense::click());

                    let line_top = rect.min.y;
                    let line_bottom = rect.max.y;
                    let top_fade = ((line_top - (vp.top() + 14.0)) / 62.0).clamp(0.0, 1.0);
                    let bot_fade = (((vp.bottom() - 14.0) - line_bottom) / 62.0).clamp(0.0, 1.0);
                    let edge_alpha = top_fade.min(bot_fade).powf(1.3);

                    if is_current {
                        let local_center_y = rect.center().y - content_origin_y;
                        detected_target_y = Some((local_center_y - viewport_h * 0.38).max(0.0));
                    }

                    if edge_alpha > 0.01 {
                        if resp.hovered() {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                        }

                        let (main_color, romaji_color, fill) = if is_current {
                            (
                                Color32::WHITE,
                                Color32::from_white_alpha(190),
                                blend_rgb(palette.bg_center, palette.accent, 0.11),
                            )
                        } else if resp.hovered() {
                            (
                                Color32::from_white_alpha(225),
                                Color32::from_white_alpha(155),
                                Color32::from_white_alpha(10),
                            )
                        } else {
                            (
                                base_main_color,
                                base_romaji_color,
                                Color32::TRANSPARENT,
                            )
                        };

                        let main_color = apply_edge_alpha(main_color, edge_alpha);
                        let romaji_color = apply_edge_alpha(romaji_color, edge_alpha);
                        let fill = apply_edge_alpha(fill, edge_alpha);

                        if fill != Color32::TRANSPARENT {
                            ui.painter().rect_filled(rect, Rounding::same(10.0), fill);
                        }

                        if is_current {
                            let bar_h = (rect.height() - 16.0).max(16.0);
                            let bar_rect = Rect::from_min_size(
                                pos2(rect.min.x + 4.0, rect.center().y - bar_h * 0.5),
                                vec2(3.0, bar_h),
                            );
                            ui.painter().rect_filled(
                                bar_rect,
                                Rounding::same(1.5),
                                apply_edge_alpha(palette.accent, edge_alpha),
                            );
                        }

                        let main_pos = pos2(rect.min.x + pad_h, rect.min.y + pad_v);
                        ui.painter().galley(main_pos, main_galley, main_color);

                        if let Some(ref rg) = romaji_galley {
                            let romaji_pos = pos2(
                                rect.min.x + pad_h,
                                rect.min.y + pad_v + text_h - rg.size().y,
                            );
                            ui.painter().galley(romaji_pos, rg.clone(), romaji_color);
                        }

                        if resp.clicked() {
                            clicked_seek_secs = Some(line.timestamp_ms as f64 / 1000.0);
                        }
                    }
                }
            }

            // 48.0px bottom spacer padding inside scroll content so last line can scroll completely clear of fade zone
            ui.add_space(48.0);
        });

    if let Some(target) = detected_target_y {
        scroll_state.target_y = target;
    } else if current_idx.is_none() {
        scroll_state.target_y = 0.0;
    }

    if clicked_seek_secs.is_some() {
        scroll_state.manual_scroll_until = 0.0;
    }

    let actual_offset_y = scroll_output.state.offset.y;
    let wheel_delta = ui.input(|i| i.raw_scroll_delta.y.abs() + i.smooth_scroll_delta.y.abs());
    let offset_diff = (actual_offset_y - scroll_state.current_y).abs();
    let is_dragging = ui.input(|i| i.pointer.is_decidedly_dragging());

    if (offset_diff > 0.5 && (wheel_delta > 0.0 || is_dragging)) || wheel_delta > 0.0 {
        scroll_state.current_y = actual_offset_y;
        scroll_state.manual_scroll_until = now_secs + 3.0;
    }

    if now_secs >= scroll_state.manual_scroll_until {
        if track_changed || (scroll_state.target_y - scroll_state.current_y).abs() > viewport_h * 1.5 {
            scroll_state.current_y = scroll_state.target_y;
        } else {
            let dt = ui.input(|i| i.stable_dt).clamp(0.001, 0.1);
            let t = (dt * 8.0).clamp(0.0, 1.0);
            scroll_state.current_y += (scroll_state.target_y - scroll_state.current_y) * t;
            if (scroll_state.target_y - scroll_state.current_y).abs() > 0.5 {
                ui.ctx().request_repaint();
            }
        }
    } else {
        let remaining_secs = (scroll_state.manual_scroll_until - now_secs).max(0.0);
        ui.ctx().request_repaint_after(std::time::Duration::from_secs_f64(remaining_secs));
    }

    scroll_state.last_active_idx = current_idx;
    ui.data_mut(|d| d.insert_temp(scroll_state_id, scroll_state));

    clicked_seek_secs
}

pub fn get_lyrics_badge(player: &PlayerController, palette: &BentoPalette) -> (&'static str, Color32, Color32) {
    if player.is_loading_lyrics {
        return (
            "Searching...",
            palette.bg_card,
            palette.accent,
        );
    }

    let lyrics = match player.lyrics.as_ref() {
        Some(l) if !l.lines.is_empty() => l,
        _ => {
            return (
                "No Lyrics",
                palette.bg_card,
                palette.text_muted,
            );
        }
    };

    if lyrics.source.contains("Instrumental")
        || (lyrics.lines.len() == 1 && lyrics.lines[0].text.eq_ignore_ascii_case("instrumental"))
    {
        return (
            "Instrumental",
            Color32::from_rgba_unmultiplied(245, 158, 11, 45),
            Color32::from_rgb(251, 191, 36),
        );
    }

    if lyrics.source == "Local .lrc" {
        return (
            "Local .lrc",
            palette.bg_card,
            palette.text_secondary,
        );
    }

    if lyrics.source == "Embedded Tag" {
        return (
            "Embedded Tag",
            palette.bg_card,
            palette.text_secondary,
        );
    }

    if lyrics.is_synced {
        (
            "LRCLIB (Synced)",
            palette.bg_card,
            palette.text_secondary,
        )
    } else {
        (
            "Plain Lyrics (LRCLIB)",
            palette.bg_card,
            palette.text_muted,
        )
    }
}

fn search_online_header_button(ui: &mut Ui, is_active: bool, palette: &BentoPalette) -> egui::Response {
    let height = 24.0;
    let icon_size = 12.0;
    let font_size = 11.0;
    let text = if is_active {
        "Close Search"
    } else {
        "Search Online (LRCLIB)"
    };
    let text_color = if is_active {
        palette.accent
    } else {
        palette.text_secondary
    };

    let galley = ui.painter().layout_no_wrap(
        text.to_string(),
        egui::FontId::proportional(font_size),
        text_color,
    );

    let pad_x = 7.0;
    let desired_width = pad_x + icon_size + 5.0 + galley.size().x + pad_x;
    let (rect, resp) = ui.allocate_exact_size(vec2(desired_width, height), egui::Sense::click());

    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }

    let fill = if is_active {
        palette.accent_soft
    } else if resp.hovered() {
        palette.bg_card_hover
    } else {
        palette.bg_card
    };

    // Borderless button with Stroke::NONE
    ui.painter().rect_filled(rect, Rounding::same(6.0), fill);

    let icon = if is_active { Icon::Close } else { Icon::Search };
    let icon_rect = Rect::from_center_size(
        pos2(rect.min.x + pad_x + icon_size * 0.5, rect.center().y),
        vec2(icon_size, icon_size),
    );
    paint_icon(ui.painter(), icon, icon_rect, text_color);

    let text_pos = pos2(
        rect.min.x + pad_x + icon_size + 5.0,
        rect.center().y - galley.size().y * 0.5,
    );
    ui.painter().galley(text_pos, galley, text_color);

    resp
}

fn romaji_toggle_button(ui: &mut Ui, is_active: bool, palette: &BentoPalette) -> egui::Response {
    let height = 24.0;
    let font_size = 11.0;
    let text = "Romaji";
    let text_color = if is_active {
        Color32::WHITE
    } else {
        palette.text_muted
    };

    let galley = ui.painter().layout_no_wrap(
        text.to_string(),
        egui::FontId::proportional(font_size),
        text_color,
    );

    let pad_x = 8.0;
    let desired_width = pad_x * 2.0 + galley.size().x;
    let (rect, resp) = ui.allocate_exact_size(vec2(desired_width, height), egui::Sense::click());

    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }

    let is_hovered = resp.hovered();
    let bg = palette.bg_card;
    let (fill, stroke) = if is_active {
        let active_fill = if is_hovered {
            blend_rgb(bg, palette.accent, 0.24)
        } else {
            blend_rgb(bg, palette.accent, 0.18)
        };
        (
            active_fill,
            Stroke::new(1.0_f32, blend_rgb(bg, palette.accent, 0.45)),
        )
    } else {
        (
            if is_hovered { palette.bg_card_hover } else { palette.bg_card },
            Stroke::new(1.0_f32, palette.seam),
        )
    };

    ui.painter().rect_filled(rect, Rounding::same(6.0), fill);
    ui.painter().rect_stroke(rect, Rounding::same(6.0), stroke);

    let text_pos = pos2(
        rect.center().x - galley.size().x * 0.5,
        rect.center().y - galley.size().y * 0.5,
    );
    ui.painter().galley(text_pos, galley, text_color);

    resp.on_hover_text(if is_active {
        "Romaji sub-lines: ON (Click to hide)"
    } else {
        "Romaji sub-lines: OFF (Click to show)"
    })
}

fn render_lyrics_empty_state(
    ui: &mut Ui,
    palette: &BentoPalette,
    track: &crate::library::Track,
    is_search_open: &mut bool,
    search_query: &mut String,
) {
    ui.add_space(48.0);
    ui.vertical_centered(|ui| {
        let (icon_rect, _) = ui.allocate_exact_size(vec2(36.0, 36.0), egui::Sense::hover());
        paint_icon(ui.painter(), Icon::Mic, icon_rect, palette.text_muted);
        ui.add_space(12.0);
        ui.label(
            RichText::new("No lyrics found for this track")
                .size(15.0)
                .color(palette.text_primary)
                .strong(),
        );
        ui.add_space(4.0);
        ui.label(
            RichText::new("Search the LRCLIB online database for synchronized or plain lyrics.")
                .size(12.5)
                .color(palette.text_muted),
        );
        ui.add_space(18.0);

        let btn = egui::Button::new(
            RichText::new("Search LRCLIB")
                .color(Color32::from_rgb(11, 9, 20))
                .size(13.5)
                .strong(),
        )
        .fill(palette.accent)
        .stroke(Stroke::NONE)
        .rounding(Rounding::same(8.0))
        .min_size(vec2(140.0, 34.0));

        if ui.add(btn).clicked() {
            *is_search_open = true;
            if search_query.trim().is_empty() {
                *search_query = format!("{} {}", track.display_title(), track.display_artist()).trim().to_string();
            }
        }
    });
}

fn stat_row(ui: &mut Ui, label: &str, val: &str, palette: &BentoPalette) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(label).size(11.0).color(palette.text_muted));
        ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(RichText::new(val).size(11.5).color(palette.text_primary).strong());
        });
    });
}

fn render_channel_meter(
    ui: &mut Ui,
    channel_lbl: &str,
    level: f32,
    peak: f32,
    palette: &BentoPalette,
) {
    ui.horizontal(|ui| {
        ui.allocate_ui_with_layout(vec2(14.0, 14.0), Layout::left_to_right(egui::Align::Center), |ui| {
            ui.label(RichText::new(channel_lbl).size(10.0).color(palette.text_secondary).strong());
        });

        let meter_width = ui.available_width() - 52.0;
        let (rect, _) = ui.allocate_exact_size(vec2(meter_width, 8.0), egui::Sense::hover());

        // Background track with Stroke::NONE
        ui.painter().rect_filled(
            rect,
            Rounding::same(2.0),
            palette.bg_sidebar,
        );

        // Filled level bar using palette.accent
        let clamped_level = level.clamp(0.0, 1.0);
        if clamped_level > 0.01 {
            let fill_w = (rect.width() * clamped_level).max(2.0);
            let fill_rect = Rect::from_min_size(rect.min, vec2(fill_w, rect.height()));

            let bar_color = if clamped_level > 0.88 {
                Color32::from_rgb(255, 178, 92)
            } else {
                palette.accent
            };

            ui.painter().rect_filled(fill_rect, Rounding::same(2.0), bar_color);
        }

        // Peak tick
        let clamped_peak = peak.clamp(0.0, 1.0);
        if clamped_peak > 0.02 {
            let peak_x = rect.min.x + rect.width() * clamped_peak;
            let peak_rect = Rect::from_min_size(pos2((peak_x - 1.5).max(rect.min.x), rect.min.y), vec2(2.0, rect.height()));
            let peak_color = if clamped_peak > 0.95 {
                Color32::from_rgb(239, 68, 68) // Red clip warning
            } else {
                palette.accent
            };
            ui.painter().rect_filled(peak_rect, Rounding::same(1.0), peak_color);
        }

        // dB value display
        let db_val = if level > 0.001 {
            format!("{:.1} dB", (20.0 * level.log10()).clamp(-60.0, 0.0))
        } else {
            "-∞ dB".to_string()
        };
        ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(RichText::new(db_val).size(9.5).color(palette.text_muted));
        });
    });
}

fn render_stream_now_playing(
    ui: &mut Ui,
    track: &StreamTrack,
    player: &mut PlayerController,
    _theme: &Theme,
    palette: &BentoPalette,
    stream_controller: Option<&StreamController>,
) {
    let pos_secs = stream_controller
        .and_then(|sc| sc.active_state.as_ref())
        .map(|s| s.position_secs)
        .unwrap_or(0.0);
    let pos_ms = (pos_secs * 1000.0) as u64;

    let total_width = ui.available_width();

    if total_width < 860.0 {
        ScrollArea::vertical()
            .id_salt("stream_now_playing_responsive_scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing = vec2(0.0, 12.0);

                let mut top_rects: [Option<Rect>; 2] = [None, None];

                ui.columns(2, |cols| {
                    cols[0].set_clip_rect(cols[0].clip_rect().intersect(cols[0].max_rect()));
                    top_rects[0] = render_stream_info_card(&mut cols[0], track, palette);

                    cols[1].set_clip_rect(cols[1].clip_rect().intersect(cols[1].max_rect()));
                    top_rects[1] = render_stream_lyrics_card(&mut cols[1], player, track, pos_ms, palette, stream_controller);
                });

                if let (Some(r0), Some(r1)) = (top_rects[0], top_rects[1]) {
                    let y_min = r0.min.y.min(r1.min.y);
                    let y_max = r0.max.y.max(r1.max.y);
                    let seam_x = (r0.max.x + r1.min.x) * 0.5;
                    let seam_rect = Rect::from_min_max(pos2(seam_x - 0.5, y_min), pos2(seam_x + 0.5, y_max));
                    ui.painter().rect_filled(seam_rect, Rounding::ZERO, palette.seam);
                }

                ui.vertical(|ui| {
                    ui.set_clip_rect(ui.clip_rect().intersect(ui.max_rect()));
                    render_stream_analyzer_notice_card(ui, track, palette, stream_controller);
                });
            });
    } else {
        let mut col_rects: [Option<Rect>; 3] = [None, None, None];

        ui.columns(3, |cols| {
            cols[0].set_clip_rect(cols[0].clip_rect().intersect(cols[0].max_rect()));
            col_rects[0] = render_stream_info_card(&mut cols[0], track, palette);

            cols[1].set_clip_rect(cols[1].clip_rect().intersect(cols[1].max_rect()));
            col_rects[1] = render_stream_lyrics_card(&mut cols[1], player, track, pos_ms, palette, stream_controller);

            cols[2].set_clip_rect(cols[2].clip_rect().intersect(cols[2].max_rect()));
            col_rects[2] = render_stream_analyzer_notice_card(&mut cols[2], track, palette, stream_controller);
        });

        if let (Some(r0), Some(r1), Some(r2)) = (col_rects[0], col_rects[1], col_rects[2]) {
            let y_min = r0.min.y.min(r1.min.y).min(r2.min.y);
            let y_max = r0.max.y.max(r1.max.y).max(r2.max.y);

            let seam1_x = (r0.max.x + r1.min.x) * 0.5;
            let seam1_rect = Rect::from_min_max(pos2(seam1_x - 0.5, y_min), pos2(seam1_x + 0.5, y_max));
            ui.painter().rect_filled(seam1_rect, Rounding::ZERO, palette.seam);

            let seam2_x = (r1.max.x + r2.min.x) * 0.5;
            let seam2_rect = Rect::from_min_max(pos2(seam2_x - 0.5, y_min), pos2(seam2_x + 0.5, y_max));
            ui.painter().rect_filled(seam2_rect, Rounding::ZERO, palette.seam);
        }
    }
}

fn render_stream_info_card(
    ui: &mut Ui,
    track: &StreamTrack,
    palette: &BentoPalette,
) -> Option<Rect> {
    ui.spacing_mut().item_spacing = vec2(0.0, 12.0);

    let panel_frame = egui::Frame::none()
        .fill(palette.bg_card)
        .rounding(Rounding::same(12.0))
        .stroke(Stroke::NONE)
        .inner_margin(egui::Margin::same(16.0));

    let resp = panel_frame.show(ui, |ui| {
        ui.set_clip_rect(ui.clip_rect().intersect(ui.max_rect()));

        ui.vertical_centered(|ui| {
            let avail_w = ui.available_width();
            let avail_h = ui.available_height();
            let max_h_card = (avail_h - 170.0).max(160.0);
            let card_width = (avail_w - 10.0).clamp(180.0, 260.0).min(max_h_card);
            let art_size = vec2(card_width, card_width);

            let (rect, _) = ui.allocate_exact_size(art_size, egui::Sense::hover());

            let prov_color = match track.provider.to_lowercase().as_str() {
                "spotify" => Color32::from_rgb(30, 215, 96),
                "youtube" | "youtube music" => Color32::from_rgb(255, 60, 60),
                _ => palette.accent,
            };

            let prov_icon = match track.provider.to_lowercase().as_str() {
                "spotify" => Icon::Spotify,
                "youtube" | "youtube music" => Icon::YouTube,
                _ => Icon::Disc,
            };

            // Soft dark ambient shadow translated downward
            let shadow_rect = rect.translate(vec2(0.0, 4.0));
            ui.painter().rect_filled(
                shadow_rect,
                Rounding::same(12.0),
                Color32::from_black_alpha(70),
            );

            // Provider icon inside card with 12.0px rounding and subtle stroke
            ui.painter().rect_filled(rect, Rounding::same(12.0), palette.bg_center);
            ui.painter().rect_stroke(
                rect,
                Rounding::same(12.0),
                Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(255, 255, 255, 28)),
            );
            let icon_box = Rect::from_center_size(rect.center(), vec2(art_size.x * 0.38, art_size.x * 0.38));
            paint_icon(ui.painter(), prov_icon, icon_box, prov_color);

            ui.add_space(14.0);

            // Track Title
            ui.label(
                RichText::new(track.display_title())
                    .size(19.0)
                    .color(palette.text_primary)
                    .strong(),
            );
            ui.add_space(2.0);

            // Artist
            ui.label(
                RichText::new(track.display_artist())
                    .size(14.5)
                    .color(palette.text_secondary),
            );
            ui.add_space(2.0);

            // Album
            ui.label(
                RichText::new(track.display_album())
                    .size(12.5)
                    .color(palette.text_muted),
            );

            ui.add_space(10.0);

            // Stream Badges with Stroke::NONE
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = vec2(6.0, 6.0);

                let badge_label = match track.provider.to_lowercase().as_str() {
                    "spotify" => "SPOTIFY CONNECT",
                    "youtube" | "youtube music" => "YOUTUBE MUSIC",
                    _ => "STREAMING",
                };
                pill_badge(
                    ui,
                    badge_label,
                    Color32::from_rgba_unmultiplied(prov_color.r(), prov_color.g(), prov_color.b(), 45),
                    prov_color,
                );

                pill_badge(
                    ui,
                    "REMOTE STREAM",
                    palette.accent_soft,
                    palette.accent,
                );

                pill_badge(
                    ui,
                    &track.formatted_duration(),
                    palette.bg_center,
                    palette.text_secondary,
                );
            });
        });
    });

    Some(resp.response.rect)
}

fn render_stream_lyrics_card(
    ui: &mut Ui,
    player: &mut PlayerController,
    track: &StreamTrack,
    pos_ms: u64,
    palette: &BentoPalette,
    stream_controller: Option<&StreamController>,
) -> Option<Rect> {
    let search_open_id = egui::Id::new("now_playing_stream_lyrics_search_open");
    let search_query_id = egui::Id::new("now_playing_stream_lyrics_search_query");
    let track_id_key = egui::Id::new("now_playing_stream_lyrics_track_id");
    let romaji_id = egui::Id::new("now_playing_lyrics_show_romaji");

    let mut is_search_open: bool = ui.data(|d| d.get_temp(search_open_id)).unwrap_or(false);
    let mut search_query: String = ui.data(|d| d.get_temp(search_query_id)).unwrap_or_default();
    let mut show_romaji: bool = ui.data(|d| d.get_temp(romaji_id)).unwrap_or(true);
    let last_track_id: Option<String> = ui.data(|d| d.get_temp(track_id_key));

    if last_track_id.as_deref() != Some(&track.id) {
        search_query = format!("{} {}", track.display_title(), track.display_artist()).trim().to_string();
        ui.data_mut(|d| d.insert_temp(track_id_key, track.id.clone()));
    }

    if search_query.trim().is_empty() {
        search_query = format!("{} {}", track.display_title(), track.display_artist()).trim().to_string();
    }

    let panel_frame = egui::Frame::none()
        .fill(palette.bg_card)
        .rounding(Rounding::same(12.0))
        .stroke(Stroke::NONE)
        .inner_margin(egui::Margin::same(16.0));

    let resp = panel_frame.show(ui, |ui| {
        ui.set_clip_rect(ui.clip_rect().intersect(ui.max_rect()));

        let lyrics_width = ui.available_width();
        if lyrics_width < 310.0 {
            ui.horizontal(|ui| {
                let (icon_box, _) = ui.allocate_exact_size(vec2(16.0, 16.0), egui::Sense::hover());
                paint_icon(ui.painter(), Icon::Mic, icon_box, palette.accent);
                ui.add_space(4.0);
                ui.label(
                    RichText::new("SYNCED LYRICS (LRCLIB)")
                        .size(13.0)
                        .color(palette.text_primary)
                        .strong(),
                );
            });

            ui.add_space(4.0);

            ui.horizontal(|ui| {
                let (badge_text, badge_fill, badge_color) = get_lyrics_badge(player, palette);
                pill_badge(ui, badge_text, badge_fill, badge_color);

                ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                    let btn_resp = search_online_header_button(ui, is_search_open, palette);
                    if btn_resp.clicked() {
                        is_search_open = !is_search_open;
                    }

                    ui.add_space(4.0);
                    let romaji_resp = romaji_toggle_button(ui, show_romaji, palette);
                    if romaji_resp.clicked() {
                        show_romaji = !show_romaji;
                    }
                });
            });
        } else {
            ui.horizontal(|ui| {
                let (icon_box, _) = ui.allocate_exact_size(vec2(16.0, 16.0), egui::Sense::hover());
                paint_icon(ui.painter(), Icon::Mic, icon_box, palette.accent);
                ui.add_space(4.0);
                ui.label(
                    RichText::new("SYNCED LYRICS (LRCLIB)")
                        .size(13.0)
                        .color(palette.text_primary)
                        .strong(),
                );

                let (badge_text, badge_fill, badge_color) = get_lyrics_badge(player, palette);
                ui.add_space(6.0);
                pill_badge(ui, badge_text, badge_fill, badge_color);

                ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                    let btn_resp = search_online_header_button(ui, is_search_open, palette);
                    if btn_resp.clicked() {
                        is_search_open = !is_search_open;
                    }

                    ui.add_space(4.0);
                    let romaji_resp = romaji_toggle_button(ui, show_romaji, palette);
                    if romaji_resp.clicked() {
                        show_romaji = !show_romaji;
                    }
                });
            });
        }

        seam_divider(ui, palette);

        if let Some(ref lyrics) = player.lyrics {
            if lyrics.lines.is_empty() {
                render_stream_lyrics_empty_state(ui, palette, &mut is_search_open);
            } else if lyrics.is_synced {
                if let Some(target) = render_synced_lyrics_list(
                    ui,
                    "stream_synced_lyrics_scroll",
                    &lyrics.lines,
                    pos_ms,
                    palette,
                    show_romaji,
                ) {
                    seek_to(player, stream_controller, target);
                }
            } else {
                // Plain lyrics
                ScrollArea::vertical()
                    .id_salt("stream_plain_lyrics_scroll")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.spacing_mut().item_spacing = vec2(0.0, 6.0);

                        for line in &lyrics.lines {
                            if line.text.trim().is_empty() {
                                ui.add_space(10.0);
                            } else {
                                ui.label(
                                    RichText::new(&line.text)
                                        .color(palette.text_primary)
                                        .size(14.5)
                                        .line_height(Some(22.0)),
                                );
                            }
                        }
                    });
            }
        } else {
            render_stream_lyrics_empty_state(ui, palette, &mut is_search_open);
        }
    });

    ui.data_mut(|d| {
        d.insert_temp(search_open_id, is_search_open);
        d.insert_temp(search_query_id, search_query);
        d.insert_temp(romaji_id, show_romaji);
    });

    Some(resp.response.rect)
}

fn render_stream_analyzer_notice_card(
    ui: &mut Ui,
    track: &StreamTrack,
    palette: &BentoPalette,
    stream_controller: Option<&StreamController>,
) -> Option<Rect> {
    ui.spacing_mut().item_spacing = vec2(0.0, 10.0);

    let panel_frame = egui::Frame::none()
        .fill(palette.bg_card)
        .rounding(Rounding::same(12.0))
        .stroke(Stroke::NONE)
        .inner_margin(egui::Margin::same(14.0));

    let resp = panel_frame.show(ui, |ui| {
        ui.set_clip_rect(ui.clip_rect().intersect(ui.max_rect()));

        // Header with Notice Badge
        let card_w = ui.available_width();
        if card_w < 300.0 {
            ui.horizontal(|ui| {
                render_icon(ui, Icon::Info, 14.0, palette.text_muted);
                ui.add_space(4.0);
                ui.label(
                    RichText::new("AUDIO ANALYZER")
                        .size(12.5)
                        .color(palette.text_secondary)
                        .strong(),
                );
            });
            ui.add_space(4.0);
            pill_badge(
                ui,
                "NOT AVAILABLE FOR STREAMS",
                Color32::from_rgba_unmultiplied(234, 179, 8, 35),
                Color32::from_rgb(250, 204, 21),
            );
        } else {
            ui.horizontal(|ui| {
                render_icon(ui, Icon::Info, 14.0, palette.text_muted);
                ui.add_space(4.0);
                ui.label(
                    RichText::new("AUDIO ANALYZER")
                        .size(12.5)
                        .color(palette.text_secondary)
                        .strong(),
                );

                ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                    pill_badge(
                        ui,
                        "NOT AVAILABLE FOR STREAMS",
                        Color32::from_rgba_unmultiplied(234, 179, 8, 35),
                        Color32::from_rgb(250, 204, 21),
                    );
                });
            });
        }

        ui.add_space(8.0);

        // Informative explanation card with Stroke::NONE
        let notice_card = egui::Frame::none()
            .fill(palette.bg_center)
            .rounding(Rounding::same(10.0))
            .stroke(Stroke::NONE)
            .inner_margin(egui::Margin::same(10.0));

        notice_card.show(ui, |ui| {
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing = vec2(0.0, 6.0);

                ui.label(
                    RichText::new("Audio Analyzer not available for streams")
                        .size(13.0)
                        .color(palette.text_primary)
                        .strong(),
                );

                ui.label(
                    RichText::new(
                        "Real-time 20Hz - 20kHz FFT spectrum visualizers, peak dBFS meters, \
                         LUFS loudness analysis, and stereo meters require local raw audio frames \
                         decoded by the Eclipse Rodio engine (Local lossless PCM decoding only).",
                    )
                    .size(11.5)
                    .color(palette.text_secondary),
                );
            });
        });

        ui.add_space(8.0);

        // Technical Stream Info Sub-Card with Stroke::NONE
        let specs_card = egui::Frame::none()
            .fill(palette.bg_center)
            .rounding(Rounding::same(8.0))
            .stroke(Stroke::NONE)
            .inner_margin(egui::Margin::same(8.0));

        specs_card.show(ui, |ui| {
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing = vec2(0.0, 4.0);

                stat_row(ui, "Source Provider", &track.provider.to_uppercase(), palette);
                stat_row(ui, "Playback Engine", "Remote Session (Official API)", palette);

                let dev_name = stream_controller
                    .and_then(|sc| sc.active_state.as_ref())
                    .and_then(|s| s.device_name.as_deref())
                    .unwrap_or("Active Device");
                stat_row(ui, "Connect Device", dev_name, palette);

                stat_row(ui, "Spectrum Tap", "Local Decoders Only (FLAC/WAV/MP3)", palette);
                stat_row(ui, "Lyrics Engine", "LRCLIB Synced (Title/Artist Match)", palette);
            });
        });
    });

    Some(resp.response.rect)
}

fn render_stream_lyrics_empty_state(ui: &mut Ui, palette: &BentoPalette, is_search_open: &mut bool) {
    ui.add_space(48.0);
    ui.vertical_centered(|ui| {
        let (icon_rect, _) = ui.allocate_exact_size(vec2(36.0, 36.0), egui::Sense::hover());
        paint_icon(ui.painter(), Icon::Mic, icon_rect, palette.text_muted);
        ui.add_space(12.0);
        ui.label(
            RichText::new("No lyrics found for this stream track")
                .size(15.0)
                .color(palette.text_primary)
                .strong(),
        );
        ui.add_space(4.0);
        ui.label(
            RichText::new("Click 'Search Online' above to look up lyrics on LRCLIB.")
                .size(12.5)
                .color(palette.text_muted),
        );
        ui.add_space(18.0);

        let btn = egui::Button::new(
            RichText::new("Search LRCLIB Online")
                .color(Color32::from_rgb(11, 9, 20))
                .size(13.5)
                .strong(),
        )
        .fill(palette.accent)
        .stroke(Stroke::NONE)
        .rounding(Rounding::same(8.0));

        if ui.add(btn).clicked() {
            *is_search_open = true;
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppConfig;
    use crate::library::model::Track;
    use crate::lyrics::{LyricLine, Lyrics, LrclibRecord};

    #[test]
    fn test_frequency_axis_markers_adaptive_thinning() {
        let wide_markers = frequency_axis_markers(300.0);
        assert_eq!(wide_markers.len(), 7);
        assert_eq!(wide_markers[0].0, "20Hz");
        assert_eq!(wide_markers[1].0, "100Hz");
        assert_eq!(wide_markers[2].0, "500Hz");
        assert_eq!(wide_markers[3].0, "1kHz");
        assert_eq!(wide_markers[4].0, "5kHz");
        assert_eq!(wide_markers[5].0, "10kHz");
        assert_eq!(wide_markers[6].0, "20kHz");

        let narrow_markers = frequency_axis_markers(250.0);
        assert_eq!(narrow_markers.len(), 4);
        assert_eq!(narrow_markers[0].0, "20Hz");
        assert_eq!(narrow_markers[1].0, "500Hz");
        assert_eq!(narrow_markers[2].0, "2kHz");
        assert_eq!(narrow_markers[3].0, "20kHz");
    }

    #[test]
    fn test_lyrics_source_badge_mapping() {
        let cfg = AppConfig::default();
        let mut player = PlayerController::new(&cfg);
        let palette = BentoPalette::default();

        // 1. None -> "No Lyrics"
        player.lyrics = None;
        let (label, _, _) = get_lyrics_badge(&player, &palette);
        assert_eq!(label, "No Lyrics");

        // 2. Loading -> "Searching..."
        player.is_loading_lyrics = true;
        let (label, _, _) = get_lyrics_badge(&player, &palette);
        assert_eq!(label, "Searching...");
        player.is_loading_lyrics = false;

        // 3. Instrumental
        player.lyrics = Some(Lyrics {
            lines: vec![LyricLine { timestamp_ms: 0, text: "Instrumental".into() }],
            is_synced: false,
            source: "LRCLIB (Instrumental)".into(),
        });
        let (label, _, _) = get_lyrics_badge(&player, &palette);
        assert_eq!(label, "Instrumental");

        // 4. Local .lrc
        player.lyrics = Some(Lyrics {
            lines: vec![LyricLine { timestamp_ms: 1000, text: "Line 1".into() }],
            is_synced: true,
            source: "Local .lrc".into(),
        });
        let (label, _, _) = get_lyrics_badge(&player, &palette);
        assert_eq!(label, "Local .lrc");

        // 5. Embedded Tag
        player.lyrics = Some(Lyrics {
            lines: vec![LyricLine { timestamp_ms: 1000, text: "Line 1".into() }],
            is_synced: false,
            source: "Embedded Tag".into(),
        });
        let (label, _, _) = get_lyrics_badge(&player, &palette);
        assert_eq!(label, "Embedded Tag");

        // 6. LRCLIB Synced
        player.lyrics = Some(Lyrics {
            lines: vec![LyricLine { timestamp_ms: 1000, text: "Line 1".into() }],
            is_synced: true,
            source: "LRCLIB (Synced)".into(),
        });
        let (label, _, _) = get_lyrics_badge(&player, &palette);
        assert_eq!(label, "LRCLIB (Synced)");

        // 7. Plain Lyrics
        player.lyrics = Some(Lyrics {
            lines: vec![LyricLine { timestamp_ms: 0, text: "Plain text line".into() }],
            is_synced: false,
            source: "LRCLIB (Plain)".into(),
        });
        let (label, _, _) = get_lyrics_badge(&player, &palette);
        assert_eq!(label, "Plain Lyrics (LRCLIB)");
    }

    #[test]
    fn test_now_playing_rendering_with_frequency_axis() {
        let ctx = egui::Context::default();
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let theme = Theme::from_name("Eclipse", None);
                let cfg = AppConfig::default();
                let mut player = PlayerController::new(&cfg);
                let mut art_manager = AlbumArtManager::new();

                // Empty state test
                render_now_playing(ui, &mut player, &theme, &mut art_manager, None);

                // Playing track test
                let track = Track {
                    id: 1,
                    path: "hires.flac".into(),
                    title: "Hi-Res Master".into(),
                    artist: "Audiophile Artist".into(),
                    album: "Master Recording".into(),
                    genre: "Classical".into(),
                    year: 2024,
                    track_number: 1,
                    duration_secs: 320.0,
                    format: "FLAC".into(),
                    bitrate_kbps: 4608,
                    sample_rate: 96000,
                    channels: 2,
                    favorite: true,
                    has_art: false,
                    folder: "".into(),
                    album_artist: "".into(),
                    play_count: 7,
                    file_size: 8192,
                };
                player.load_queue(vec![track], 0);

                // Update visualizer values
                player.visualizer.band_values = vec![0.5; 32];
                player.visualizer.peak_values = vec![0.8; 32];
                player.visualizer.wave_buffer = vec![0.0; 128];
                player.visualizer.peak_dbfs = -3.2;
                player.visualizer.lufs = -14.1;
                player.visualizer.dynamic_range = 12.0;

                render_now_playing(ui, &mut player, &theme, &mut art_manager, None);
            });
        });
    }

    #[test]
    fn test_now_playing_narrow_width_responsive_layout() {
        let ctx = egui::Context::default();
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.allocate_ui(vec2(600.0, 800.0), |ui| {
                    let theme = Theme::from_name("Eclipse", None);
                    let cfg = AppConfig::default();
                    let mut player = PlayerController::new(&cfg);
                    let mut art_manager = AlbumArtManager::new();

                    let track = Track {
                        id: 1,
                        path: "test.flac".into(),
                        title: "Narrow Song Title That Is Long".into(),
                        artist: "Narrow Artist That Is Long".into(),
                        album: "Narrow Album".into(),
                        genre: "Rock".into(),
                        year: 2024,
                        track_number: 1,
                        duration_secs: 200.0,
                        format: "FLAC".into(),
                        bitrate_kbps: 1411,
                        sample_rate: 48000,
                        channels: 2,
                        favorite: false,
                        has_art: false,
                        folder: "".into(),
                        album_artist: "".into(),
                        play_count: 5,
                        file_size: 1024,
                    };
                    player.load_queue(vec![track], 0);

                    render_now_playing(ui, &mut player, &theme, &mut art_manager, None);
                });
            });
        });
    }

    #[test]
    fn test_search_drawer_rendering_and_interaction() {
        let ctx = egui::Context::default();
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let theme = Theme::from_name("Eclipse", None);
                let cfg = AppConfig::default();
                let mut player = PlayerController::new(&cfg);
                let mut art_manager = AlbumArtManager::new();

                let track = Track {
                    id: 42,
                    path: "song.flac".into(),
                    title: "Bohemian Rhapsody".into(),
                    artist: "Queen".into(),
                    album: "A Night at the Opera".into(),
                    genre: "Rock".into(),
                    year: 1975,
                    track_number: 11,
                    duration_secs: 354.0,
                    format: "FLAC".into(),
                    bitrate_kbps: 1411,
                    sample_rate: 44100,
                    channels: 2,
                    favorite: false,
                    has_art: false,
                    folder: "".into(),
                    album_artist: "".into(),
                    play_count: 3,
                    file_size: 40960,
                };
                player.load_queue(vec![track], 0);

                // Set search drawer open
                let search_open_id = egui::Id::new("now_playing_lyrics_search_open");
                ui.data_mut(|d| d.insert_temp(search_open_id, true));

                // Provide mock search results
                player.manual_search_results = Some(vec![
                    LrclibRecord {
                        id: 101,
                        name: Some("Bohemian Rhapsody".into()),
                        track_name: "Bohemian Rhapsody".into(),
                        artist_name: "Queen".into(),
                        album_name: Some("A Night at the Opera".into()),
                        duration: Some(354.0),
                        instrumental: false,
                        plain_lyrics: Some("Is this the real life?".into()),
                        synced_lyrics: Some("[00:01.00] Is this the real life?".into()),
                    },
                    LrclibRecord {
                        id: 102,
                        name: None,
                        track_name: "Bohemian Rhapsody (Instrumental)".into(),
                        artist_name: "Queen".into(),
                        album_name: None,
                        duration: Some(354.0),
                        instrumental: true,
                        plain_lyrics: None,
                        synced_lyrics: None,
                    },
                    LrclibRecord {
                        id: 103,
                        name: None,
                        track_name: "Bohemian Rhapsody (Plain)".into(),
                        artist_name: "Queen".into(),
                        album_name: None,
                        duration: Some(354.0),
                        instrumental: false,
                        plain_lyrics: Some("Mama, just killed a man".into()),
                        synced_lyrics: None,
                    },
                ]);

                // Render Now Playing view with Search Drawer and Results
                render_now_playing(ui, &mut player, &theme, &mut art_manager, None);

                // Apply custom lyrics test without unwrap or expect
                if let Some(ref results) = player.manual_search_results {
                    if let Some(first) = results.first() {
                        let selected = first.clone();
                        player.apply_custom_lyrics(&selected);
                    }
                }
                assert!(player.lyrics.is_some());
                if let Some(ref lyrics) = player.lyrics {
                    assert!(lyrics.is_synced);
                    assert_eq!(lyrics.lines.len(), 1);
                }
            });
        });
    }

    #[test]
    fn test_plain_lyrics_and_loading_states() {
        let ctx = egui::Context::default();
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let theme = Theme::from_name("Eclipse", None);
                let cfg = AppConfig::default();
                let mut player = PlayerController::new(&cfg);
                let mut art_manager = AlbumArtManager::new();

                let track = Track {
                    id: 99,
                    path: "instrumental.mp3".into(),
                    title: "Orion".into(),
                    artist: "Metallica".into(),
                    album: "Master of Puppets".into(),
                    genre: "Metal".into(),
                    year: 1986,
                    track_number: 7,
                    duration_secs: 507.0,
                    format: "MP3".into(),
                    bitrate_kbps: 320,
                    sample_rate: 44100,
                    channels: 2,
                    favorite: false,
                    has_art: false,
                    folder: "".into(),
                    album_artist: "".into(),
                    play_count: 5,
                    file_size: 20480,
                };
                player.load_queue(vec![track], 0);

                // 1. Loading state
                player.is_loading_lyrics = true;
                player.lyrics = None;
                render_now_playing(ui, &mut player, &theme, &mut art_manager, None);

                // 2. Plain lyrics state
                player.is_loading_lyrics = false;
                player.lyrics = Some(Lyrics {
                    lines: vec![
                        LyricLine { timestamp_ms: 0, text: "Some verse line 1".into() },
                        LyricLine { timestamp_ms: 0, text: "Some verse line 2".into() },
                    ],
                    is_synced: false,
                    source: "LRCLIB (Plain)".into(),
                });
                render_now_playing(ui, &mut player, &theme, &mut art_manager, None);

                // 3. Instrumental state
                player.lyrics = Some(Lyrics {
                    lines: vec![
                        LyricLine { timestamp_ms: 0, text: "Instrumental".into() },
                    ],
                    is_synced: false,
                    source: "LRCLIB (Instrumental)".into(),
                });
                render_now_playing(ui, &mut player, &theme, &mut art_manager, None);
            });
        });
    }

    #[test]
    fn test_render_now_playing_stream_track() {
        let ctx = egui::Context::default();
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let theme = Theme::from_name("Eclipse", None);
                let cfg = AppConfig::default();
                let mut player = PlayerController::new(&cfg);
                let mut art_manager = AlbumArtManager::new();
                let ctrl = StreamController::new(vec![]);

                let stream_track = StreamTrack::new(
                    "spotify-456",
                    "Starboy",
                    "The Weeknd",
                    "Starboy",
                    230.0,
                    None,
                    "spotify:track:456",
                    "spotify",
                );
                player.switch_to_stream("spotify", Some(stream_track));

                render_now_playing(ui, &mut player, &theme, &mut art_manager, Some(&ctrl));
            });
        });
    }

    #[test]
    fn test_synced_lyrics_click_to_seek_dispatch() {
        let cfg = AppConfig::default();
        let mut player = PlayerController::new(&cfg);
        let stream_ctrl = StreamController::new(vec![]);

        // 1. Local track synced lyrics seek
        let line_timestamp_ms = 45200;
        let target_secs = line_timestamp_ms as f64 / 1000.0;
        seek_to(&mut player, None, target_secs);

        // 2. Stream track synced lyrics seek
        let stream_track = StreamTrack::new(
            "spotify-lyrics",
            "Song",
            "Artist",
            "Album",
            200.0,
            None,
            "spotify:track:lyrics",
            "spotify",
        );
        player.switch_to_stream("spotify", Some(stream_track));

        let stream_line_ms = 78500;
        let stream_target_secs = stream_line_ms as f64 / 1000.0;
        seek_to(&mut player, Some(&stream_ctrl), stream_target_secs);

        // 3. Smoke render now_playing with synced lyrics
        let ctx = egui::Context::default();
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let theme = Theme::from_name("Eclipse", None);
                let mut art_manager = AlbumArtManager::new();
                player.lyrics = Some(crate::lyrics::Lyrics {
                    is_synced: true,
                    lines: vec![
                        crate::lyrics::LyricLine {
                            timestamp_ms: 10000,
                            text: "Line 1".into(),
                        },
                        crate::lyrics::LyricLine {
                            timestamp_ms: 45200,
                            text: "Line 2".into(),
                        },
                    ],
                    source: "LRCLIB".into(),
                });

                render_now_playing(ui, &mut player, &theme, &mut art_manager, Some(&stream_ctrl));
            });
        });
    }

    #[test]
    fn test_is_line_instrumental_detection() {
        assert!(super::is_line_instrumental(""));
        assert!(super::is_line_instrumental("   "));
        assert!(super::is_line_instrumental("instrumental"));
        assert!(super::is_line_instrumental("(Instrumental)"));
        assert!(super::is_line_instrumental("[INSTRUMENTAL]"));
        assert!(super::is_line_instrumental("\u{266a}"));
        assert!(super::is_line_instrumental("\u{266b} \u{266a}"));
        assert!(super::is_line_instrumental("..."));
        assert!(super::is_line_instrumental("\u{2022} \u{2022} \u{2022}"));
        assert!(super::is_line_instrumental("---"));
        assert!(super::is_line_instrumental("~ ~"));

        assert!(!super::is_line_instrumental("Hello darkness my old friend"));
        assert!(!super::is_line_instrumental("Just a regular lyric line"));
        assert!(!super::is_line_instrumental("Guitar solo coming up"));
    }

    #[test]
    fn test_build_display_lyrics_empty_and_gap_injection() {
        use crate::lyrics::LyricLine;

        // 1. Empty lines input
        let empty_res = super::build_display_lyrics(&[]);
        assert!(empty_res.is_empty());

        // 2. Explicit empty lines are preserved with is_instrumental = true
        let lines_with_empty = vec![
            LyricLine {
                timestamp_ms: 5000,
                text: "Intro vocal".into(),
            },
            LyricLine {
                timestamp_ms: 10000,
                text: "".into(),
            },
            LyricLine {
                timestamp_ms: 15000,
                text: "Next vocal".into(),
            },
        ];
        let disp1 = super::build_display_lyrics(&lines_with_empty);
        assert_eq!(disp1.len(), 3);
        assert!(!disp1[0].is_instrumental);
        assert!(disp1[1].is_instrumental);
        assert_eq!(disp1[1].timestamp_ms, 10000);
        assert!(!disp1[2].is_instrumental);

        // 3. Massive intro gap (> 10s) injects dummy line at 0ms
        let lines_late_intro = vec![
            LyricLine {
                timestamp_ms: 25000,
                text: "Verse 1 starts late".into(),
            },
        ];
        let disp2 = super::build_display_lyrics(&lines_late_intro);
        assert_eq!(disp2.len(), 2);
        assert!(disp2[0].is_instrumental);
        assert_eq!(disp2[0].timestamp_ms, 0);
        assert_eq!(disp2[1].timestamp_ms, 25000);

        // 4. Massive gap (> 10s) between consecutive lines injects halfway instrumental line
        let lines_with_solo = vec![
            LyricLine {
                timestamp_ms: 10000,
                text: "Solo begins".into(),
            },
            LyricLine {
                timestamp_ms: 40000,
                text: "Solo ends".into(),
            },
        ];
        let disp3 = super::build_display_lyrics(&lines_with_solo);
        assert_eq!(disp3.len(), 3);
        assert_eq!(disp3[0].text, "Solo begins");
        assert!(!disp3[0].is_instrumental);
        assert!(disp3[1].is_instrumental);
        assert_eq!(disp3[1].timestamp_ms, 25000); // 10000 + (40000 - 10000) / 2
        assert_eq!(disp3[2].text, "Solo ends");
        assert!(!disp3[2].is_instrumental);

        // 5. Short gap (<= 10s) does not inject dummy lines
        let lines_short_gaps = vec![
            LyricLine {
                timestamp_ms: 2000,
                text: "Line A".into(),
            },
            LyricLine {
                timestamp_ms: 6000,
                text: "Line B".into(),
            },
            LyricLine {
                timestamp_ms: 10000,
                text: "Line C".into(),
            },
        ];
        let disp4 = super::build_display_lyrics(&lines_short_gaps);
        assert_eq!(disp4.len(), 3);

        // 6. Test current_display_index
        assert_eq!(super::current_display_index(&disp3, 5000), None);
        assert_eq!(super::current_display_index(&disp3, 10000), Some(0));
        assert_eq!(super::current_display_index(&disp3, 20000), Some(0));
        assert_eq!(super::current_display_index(&disp3, 25000), Some(1));
        assert_eq!(super::current_display_index(&disp3, 35000), Some(1));
        assert_eq!(super::current_display_index(&disp3, 40000), Some(2));
    }

    #[test]
    fn test_synced_lyrics_empty_and_instrumental_indicator_rendering() {
        use crate::lyrics::LyricLine;

        let ctx = egui::Context::default();
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let theme = Theme::from_name("Eclipse", None);
                let palette = BentoPalette::from_theme(&theme);
                let lines = vec![
                    LyricLine {
                        timestamp_ms: 5000,
                        text: "First sung lyric".into(),
                    },
                    LyricLine {
                        timestamp_ms: 10000,
                        text: "".into(), // Explicit empty line
                    },
                    LyricLine {
                        timestamp_ms: 35000, // Gap of 25s to next line
                        text: "Next sung lyric".into(),
                    },
                ];

                // Render at pos_ms = 12000 (empty line active with music note indicator)
                let clicked = super::render_synced_lyrics_list(
                    ui,
                    "test_lyrics_scroll",
                    &lines,
                    12000,
                    &palette,
                    true,
                );
                assert!(clicked.is_none());
            });
        });
    }

    #[test]
    fn test_build_display_lyrics_romaji_caching() {
        use crate::lyrics::LyricLine;

        let lines = vec![
            LyricLine {
                timestamp_ms: 1000,
                text: "君が好きだと叫びたい".into(),
            },
            LyricLine {
                timestamp_ms: 5000,
                text: "I want to shout I love you".into(),
            },
            LyricLine {
                timestamp_ms: 9000,
                text: "instrumental".into(),
            },
        ];

        let display = super::build_display_lyrics(&lines);
        assert_eq!(display.len(), 3);
        if crate::ui::romaji::to_romaji_if_japanese(&lines[0].text).is_some() {
            assert!(display[0].romaji.is_some());
        }
        assert_eq!(display[1].romaji, None);
        assert_eq!(display[2].romaji, None);
    }

    #[test]
    fn test_feishin_matte_lyrics_badge_and_romaji_pill() {
        use crate::config::AppConfig;
        let theme = Theme::from_name("Eclipse", None);
        let palette = BentoPalette::from_theme(&theme);
        let cfg = AppConfig::default();
        let mut player = PlayerController::new(&cfg);

        // 1. Loading lyrics badge
        player.is_loading_lyrics = true;
        let (txt, fill, _col) = super::get_lyrics_badge(&player, &palette);
        assert_eq!(txt, "Searching...");
        assert_eq!(fill, palette.bg_card);

        // 2. Synced LRCLIB badge
        player.is_loading_lyrics = false;
        player.lyrics = Some(crate::lyrics::Lyrics {
            lines: vec![crate::lyrics::LyricLine { timestamp_ms: 1000, text: "Line".into() }],
            is_synced: true,
            source: "LRCLIB (Synced)".into(),
        });
        let (txt2, fill2, col2) = super::get_lyrics_badge(&player, &palette);
        assert_eq!(txt2, "LRCLIB (Synced)");
        assert_eq!(fill2, palette.bg_card);
        assert_eq!(col2, palette.text_secondary);

        // 3. Local .lrc badge
        player.lyrics = Some(crate::lyrics::Lyrics {
            lines: vec![crate::lyrics::LyricLine { timestamp_ms: 1000, text: "Line".into() }],
            is_synced: true,
            source: "Local .lrc".into(),
        });
        let (txt3, fill3, col3) = super::get_lyrics_badge(&player, &palette);
        assert_eq!(txt3, "Local .lrc");
        assert_eq!(fill3, palette.bg_card);
        assert_eq!(col3, palette.text_secondary);
    }

    #[test]
    fn test_romaji_hiding_on_non_japanese_tracks() {
        // English text has no romaji
        let en_text = "Never gonna give you up";
        assert_eq!(crate::ui::romaji::to_romaji_if_japanese(en_text), None);

        // Korean text has no romaji
        let kr_text = "안녕하세요";
        assert_eq!(crate::ui::romaji::to_romaji_if_japanese(kr_text), None);

        // Check FullscreenLyrics has_romaji
        let en_lrc = crate::lyrics::Lyrics {
            lines: vec![
                crate::lyrics::LyricLine { timestamp_ms: 1000, text: en_text.into() },
                crate::lyrics::LyricLine { timestamp_ms: 3000, text: "Together forever".into() },
            ],
            is_synced: true,
            source: "test_en".into(),
        };
        let fs_en = crate::ui::fullscreen_lyrics::FullscreenLyrics::from_lyrics(&en_lrc);
        assert!(!fs_en.has_romaji(), "English track must have has_romaji == false");

        // When Japanese text is converted to romaji, has_romaji returns true
        let jp_text = "君が好きだと叫びたい";
        if crate::ui::romaji::to_romaji_if_japanese(jp_text).is_some() {
            let jp_lrc = crate::lyrics::Lyrics {
                lines: vec![
                    crate::lyrics::LyricLine { timestamp_ms: 1000, text: jp_text.into() },
                ],
                is_synced: true,
                source: "test_jp".into(),
            };
            let fs_jp = crate::ui::fullscreen_lyrics::FullscreenLyrics::from_lyrics(&jp_lrc);
            assert!(fs_jp.has_romaji(), "Japanese track must have has_romaji == true");
        }
    }

    #[test]
    fn test_blend_rgb() {
        let bg = Color32::from_rgb(29, 33, 41);
        let fg = Color32::from_rgb(156, 108, 255);
        let blended_0 = blend_rgb(bg, fg, 0.0);
        assert_eq!(blended_0, bg);
        let blended_1 = blend_rgb(bg, fg, 1.0);
        assert_eq!(blended_1, fg);
        let blended_mid = blend_rgb(bg, fg, 0.11);
        assert_eq!(blended_mid, Color32::from_rgb(43, 41, 65));
    }

    #[test]
    fn test_apply_edge_alpha() {
        let white = Color32::from_rgba_unmultiplied(255, 255, 255, 255);
        let half_alpha = apply_edge_alpha(white, 0.5);
        assert_eq!(half_alpha.a(), 128);

        let zero_alpha = apply_edge_alpha(white, 0.0);
        assert_eq!(zero_alpha.a(), 0);

        let full_alpha = apply_edge_alpha(white, 1.0);
        assert_eq!(full_alpha.a(), 255);

        // Clamp below 0 and above 1
        assert_eq!(apply_edge_alpha(white, -0.5).a(), 0);
        assert_eq!(apply_edge_alpha(white, 1.5).a(), 255);

        // Scaling non-255 alpha
        let semi = Color32::from_rgba_unmultiplied(100, 150, 200, 100);
        assert_eq!(apply_edge_alpha(semi, 0.5).a(), 50);
    }
}
