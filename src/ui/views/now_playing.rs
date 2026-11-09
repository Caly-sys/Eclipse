use crate::audio::PlayerController;
use crate::config::VisualizerStyle;
use crate::library::format_time_secs;
use crate::ui::art::AlbumArtManager;
use crate::ui::components::{format_badge, pill_badge, render_spectrum};
use crate::ui::icons::{icon_button as vector_icon_button, paint_icon, render_icon, Icon};
use crate::ui::theme::Theme;
use egui::{pos2, vec2, Color32, Layout, Rect, RichText, Rounding, ScrollArea, Stroke, Ui};
use std::time::Duration;

pub fn render_now_playing(
    ui: &mut Ui,
    player: &mut PlayerController,
    theme: &Theme,
    art_manager: &mut AlbumArtManager,
) {
    if let Some(track) = player.current_track().cloned() {
        let pos_secs = player.engine.position().as_secs_f64();
        let pos_ms = (pos_secs * 1000.0) as u64;

        ui.columns(3, |cols| {
            // =================================================================
            // COLUMN 1 (LEFT): Artwork, Track Info, Technical Badges
            // =================================================================
            cols[0].vertical(|ui| {
                ui.spacing_mut().item_spacing = vec2(0.0, 12.0);

                let mut art_frame = theme.glass_frame(16.0);
                art_frame.inner_margin = egui::Margin::same(16.0);

                art_frame.show(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        let card_width = (ui.available_width() - 10.0).clamp(180.0, 260.0);
                        let art_size = vec2(card_width, card_width);

                        // Ambient glow & drop shadow behind album art card
                        let (rect, _) = ui.allocate_exact_size(art_size, egui::Sense::hover());

                        // Multi-layered specular violet halo
                        ui.painter().rect_filled(
                            rect.expand(10.0),
                            Rounding::same(18.0),
                            Color32::from_rgba_premultiplied(
                                theme.accent.r(),
                                theme.accent.g(),
                                theme.accent.b(),
                                18,
                            ),
                        );
                        ui.painter().rect_filled(
                            rect.expand(4.0),
                            Rounding::same(16.0),
                            Color32::from_rgba_premultiplied(
                                theme.accent.r(),
                                theme.accent.g(),
                                theme.accent.b(),
                                32,
                            ),
                        );

                        // Album Art Image or fallback placeholder
                        let mut painted_cover = false;
                        if let Some(texture) = art_manager.get_or_load(ui.ctx(), &track.path) {
                            ui.painter().image(
                                texture.id(),
                                rect,
                                Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                                Color32::WHITE,
                            );
                            ui.painter().rect_stroke(rect, Rounding::same(14.0), Stroke::new(1.0_f32, theme.border_accent));
                            painted_cover = true;
                        }

                        if !painted_cover {
                            ui.painter().rect_filled(rect, Rounding::same(14.0), theme.card);
                            ui.painter().rect_stroke(rect, Rounding::same(14.0), Stroke::new(1.0_f32, theme.border_accent));
                            let icon_box = Rect::from_center_size(rect.center(), vec2(art_size.x * 0.36, art_size.x * 0.36));
                            paint_icon(ui.painter(), Icon::Disc, icon_box, theme.accent);
                        }

                        ui.add_space(14.0);

                        // Track Title
                        ui.label(
                            RichText::new(track.display_title())
                                .size(19.0)
                                .color(theme.text_primary)
                                .strong(),
                        );
                        ui.add_space(2.0);

                        // Artist
                        ui.label(
                            RichText::new(track.display_artist())
                                .size(14.5)
                                .color(theme.text_secondary),
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
                                .color(theme.text_muted),
                        );

                        ui.add_space(10.0);

                        // Format Badges matching design spec
                        ui.horizontal_wrapped(|ui| {
                            ui.spacing_mut().item_spacing = vec2(6.0, 6.0);

                            // High-contrast format badge (FLAC, MP3, etc.)
                            format_badge(ui, &track.format, track.is_lossless(), theme);

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
                                Color32::from_rgba_unmultiplied(38, 30, 62, 215),
                                Color32::from_rgb(220, 210, 245),
                            );

                            // Play count badge
                            pill_badge(
                                ui,
                                &format!("{} plays", track.play_count),
                                Color32::from_rgba_unmultiplied(38, 30, 62, 190),
                                theme.text_secondary,
                            );

                            // Lossless or Hi-Res badge
                            if track.is_hires() {
                                pill_badge(
                                    ui,
                                    "HI-RES LOSSLESS",
                                    Color32::from_rgba_unmultiplied(theme.accent_warm.r(), theme.accent_warm.g(), theme.accent_warm.b(), 56),
                                    theme.accent_warm,
                                );
                            } else if track.is_lossless() {
                                pill_badge(
                                    ui,
                                    "LOSSLESS",
                                    Color32::from_rgba_unmultiplied(theme.accent.r(), theme.accent.g(), theme.accent.b(), 56),
                                    theme.accent,
                                );
                            }
                        });
                    });
                });
            });

            // =================================================================
            // COLUMN 2 (CENTER): Synced Lyrics with Auto-Scrolling & Click-Seek
            // =================================================================
            cols[1].vertical(|ui| {
                let search_open_id = egui::Id::new("now_playing_lyrics_search_open");
                let search_query_id = egui::Id::new("now_playing_lyrics_search_query");
                let track_id_key = egui::Id::new("now_playing_lyrics_current_track_id");

                let mut is_search_open: bool = ui.data(|d| d.get_temp(search_open_id)).unwrap_or(false);
                let mut search_query: String = ui.data(|d| d.get_temp(search_query_id)).unwrap_or_default();
                let last_track_id: Option<i64> = ui.data(|d| d.get_temp(track_id_key));

                if last_track_id != Some(track.id) {
                    search_query = format!("{} {}", track.display_title(), track.display_artist()).trim().to_string();
                    ui.data_mut(|d| d.insert_temp(track_id_key, track.id));
                }

                if search_query.trim().is_empty() {
                    search_query = format!("{} {}", track.display_title(), track.display_artist()).trim().to_string();
                }

                let mut lyrics_frame = theme.glass_frame(16.0);
                lyrics_frame.inner_margin = egui::Margin::same(16.0);

                lyrics_frame.show(ui, |ui| {
                    // Header layout: Title `SYNCED LYRICS`, source badge, compact "Search Online (LRCLIB)" button
                    ui.horizontal(|ui| {
                        let (icon_box, _) = ui.allocate_exact_size(vec2(16.0, 16.0), egui::Sense::hover());
                        paint_icon(ui.painter(), Icon::Mic, icon_box, theme.accent);
                        ui.add_space(4.0);
                        ui.label(
                            RichText::new("SYNCED LYRICS")
                                .size(13.0)
                                .color(theme.text_primary)
                                .strong(),
                        );

                        let (badge_text, badge_fill, badge_color) = get_lyrics_badge(player);
                        ui.add_space(6.0);
                        pill_badge(ui, badge_text, badge_fill, badge_color);

                        ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                            let btn_resp = search_online_header_button(ui, is_search_open, theme);
                            if btn_resp.clicked() {
                                is_search_open = !is_search_open;
                                if is_search_open && search_query.trim().is_empty() {
                                    search_query = format!("{} {}", track.display_title(), track.display_artist()).trim().to_string();
                                }
                            }
                        });
                    });

                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(8.0);

                    if is_search_open {
                        let mut picked_record = None;
                        let mut trigger_search_query = None;

                        // Inline LRCLIB Search Drawer / Modal inside Synced Lyrics card
                        let drawer_card = theme.card_frame(10.0);
                        drawer_card.show(ui, |ui| {
                            ui.vertical(|ui| {
                                ui.spacing_mut().item_spacing = vec2(0.0, 8.0);

                                // Header: "Search LRCLIB" and vector Close button
                                ui.horizontal(|ui| {
                                    let (s_icon_box, _) = ui.allocate_exact_size(vec2(14.0, 14.0), egui::Sense::hover());
                                    paint_icon(ui.painter(), Icon::Search, s_icon_box, theme.accent);
                                    ui.add_space(4.0);
                                    ui.label(
                                        RichText::new("Search LRCLIB Online")
                                            .size(13.0)
                                            .color(theme.text_primary)
                                            .strong(),
                                    );

                                    ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                                        let close_resp = vector_icon_button(ui, Icon::Close, 14.0, theme.text_muted);
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

                                    let edit_resp = ui.add(
                                        egui::TextEdit::singleline(&mut search_query)
                                            .hint_text("Title Artist...")
                                            .desired_width(input_w)
                                            .text_color(theme.text_primary)
                                            .margin(egui::Margin::symmetric(8.0, 6.0)),
                                    );

                                    let search_btn = egui::Button::new(
                                        RichText::new("Search")
                                            .color(Color32::from_rgb(11, 9, 20))
                                            .size(12.5)
                                            .strong(),
                                    )
                                    .fill(theme.accent)
                                    .rounding(Rounding::same(6.0))
                                    .min_size(vec2(btn_w, 28.0));

                                    let search_clicked = ui.add(search_btn).clicked();
                                    let enter_pressed = edit_resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));

                                    if (search_clicked || enter_pressed) && !search_query.trim().is_empty() {
                                        trigger_search_query = Some(search_query.trim().to_string());
                                    }
                                });

                                ui.add_space(2.0);
                                ui.separator();
                                ui.add_space(2.0);

                                // Drawer Body: Searching, Error, Results, or Empty prompt
                                if player.is_searching_lyrics {
                                    ui.add_space(24.0);
                                    ui.vertical_centered(|ui| {
                                        let (spk_box, _) = ui.allocate_exact_size(vec2(20.0, 20.0), egui::Sense::hover());
                                        paint_icon(ui.painter(), Icon::Sparkle, spk_box, theme.accent);
                                        ui.add_space(8.0);
                                        ui.label(
                                            RichText::new("Searching LRCLIB online...")
                                                .color(theme.text_secondary)
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
                                                    .color(theme.text_muted)
                                                    .size(13.0),
                                            );
                                            ui.add_space(4.0);
                                            ui.label(
                                                RichText::new("Try adjusting the title or artist in your search query.")
                                                    .color(theme.text_muted)
                                                    .size(11.5),
                                            );
                                        });
                                        ui.add_space(24.0);
                                    } else {
                                        ScrollArea::vertical()
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
                                                        .fill(Color32::from_rgba_unmultiplied(22, 16, 38, 190))
                                                        .stroke(Stroke::new(1.0_f32, theme.border))
                                                        .rounding(Rounding::same(8.0))
                                                        .inner_margin(egui::Margin::symmetric(10.0, 8.0));

                                                    let item_resp = item_frame.show(ui, |ui| {
                                                        ui.horizontal(|ui| {
                                                            ui.vertical(|ui| {
                                                                ui.spacing_mut().item_spacing = vec2(0.0, 2.0);
                                                                ui.label(
                                                                    RichText::new(title_display)
                                                                        .size(13.0)
                                                                        .color(theme.text_primary)
                                                                        .strong(),
                                                                );

                                                                let sub_str = format!(
                                                                    "{} · {} · {}",
                                                                    record.artist_name, album_display, dur_str
                                                                );
                                                                ui.label(
                                                                    RichText::new(sub_str)
                                                                        .size(11.0)
                                                                        .color(theme.text_muted),
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
                                                                            Color32::from_rgba_unmultiplied(
                                                                                theme.accent.r(),
                                                                                theme.accent.g(),
                                                                                theme.accent.b(),
                                                                                45,
                                                                            ),
                                                                            theme.accent,
                                                                        );
                                                                    } else if record.instrumental {
                                                                        pill_badge(
                                                                            ui,
                                                                            "INSTRUMENTAL",
                                                                            Color32::from_rgba_unmultiplied(
                                                                                theme.accent_warm.r(),
                                                                                theme.accent_warm.g(),
                                                                                theme.accent_warm.b(),
                                                                                45,
                                                                            ),
                                                                            theme.accent_warm,
                                                                        );
                                                                    } else {
                                                                        pill_badge(
                                                                            ui,
                                                                            "PLAIN",
                                                                            theme.card,
                                                                            theme.text_muted,
                                                                        );
                                                                    }
                                                                },
                                                            );
                                                        });
                                                    });

                                                    let interact = item_resp.response.interact(egui::Sense::click());
                                                    if interact.hovered() {
                                                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                                                        ui.painter().rect_stroke(
                                                            item_resp.response.rect,
                                                            Rounding::same(8.0),
                                                            Stroke::new(1.0_f32, theme.border_accent),
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
                                                .color(theme.text_muted)
                                                .size(12.5),
                                        );
                                        ui.add_space(8.0);
                                        let quick_search_btn = egui::Button::new(
                                            RichText::new(format!("Search \"{}\"", search_query.trim()))
                                                .color(theme.text_primary)
                                                .size(12.0),
                                        )
                                        .fill(theme.card)
                                        .stroke(Stroke::new(1.0_f32, theme.border_accent))
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
                            paint_icon(ui.painter(), Icon::Sparkle, icon_rect, theme.accent);
                            ui.add_space(10.0);
                            ui.label(
                                RichText::new("Searching LRCLIB...")
                                    .color(theme.text_secondary)
                                    .size(14.0),
                            );
                            ui.add_space(4.0);
                            ui.label(
                                RichText::new("Looking for synced lyrics online")
                                    .color(theme.text_muted)
                                    .size(12.0),
                            );
                        });
                    } else if let Some(ref lyrics) = player.lyrics {
                        if lyrics.lines.is_empty() {
                            render_lyrics_empty_state(ui, theme, &track, &mut is_search_open, &mut search_query);
                        } else if lyrics.source.contains("Instrumental")
                            || (lyrics.lines.len() == 1 && lyrics.lines[0].text.eq_ignore_ascii_case("instrumental"))
                        {
                            ui.add_space(40.0);
                            ui.vertical_centered(|ui| {
                                let (icon_rect, _) = ui.allocate_exact_size(vec2(32.0, 32.0), egui::Sense::hover());
                                paint_icon(ui.painter(), Icon::Music, icon_rect, theme.accent_warm);
                                ui.add_space(10.0);
                                ui.label(
                                    RichText::new("Instrumental Track")
                                        .size(15.0)
                                        .color(theme.text_primary)
                                        .strong(),
                                );
                                ui.add_space(4.0);
                                ui.label(
                                    RichText::new("This song does not contain vocal lyrics.")
                                        .size(12.5)
                                        .color(theme.text_muted),
                                );
                                ui.add_space(16.0);
                                let search_btn = egui::Button::new(
                                    RichText::new("Search LRCLIB Anyway")
                                        .color(theme.text_primary)
                                        .size(12.5),
                                )
                                .fill(theme.card)
                                .stroke(Stroke::new(1.0_f32, theme.border))
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
                            let current_idx = lyrics.current_index(pos_ms);

                            ScrollArea::vertical()
                                .auto_shrink([false, false])
                                .show(ui, |ui| {
                                    ui.spacing_mut().item_spacing = vec2(0.0, 8.0);

                                    for (i, line) in lyrics.lines.iter().enumerate() {
                                        let is_current = current_idx == Some(i);

                                        let text_color = if is_current {
                                            theme.text_primary
                                        } else {
                                            theme.text_muted
                                        };

                                        let size = if is_current { 18.0 } else { 14.5 };
                                        let mut rich = RichText::new(&line.text)
                                            .color(text_color)
                                            .size(size);

                                        if is_current {
                                            rich = rich.strong();
                                        }

                                        let (fill, stroke) = if is_current {
                                            (
                                                Color32::from_rgba_premultiplied(
                                                    theme.accent.r(),
                                                    theme.accent.g(),
                                                    theme.accent.b(),
                                                    30,
                                                ),
                                                Stroke::new(1.0_f32, theme.border_accent),
                                            )
                                        } else {
                                            (Color32::TRANSPARENT, Stroke::NONE)
                                        };

                                        let btn = ui.add(
                                            egui::Button::new(rich)
                                                .fill(fill)
                                                .stroke(stroke)
                                                .rounding(Rounding::same(8.0))
                                                .wrap(),
                                        );

                                        if is_current {
                                            btn.scroll_to_me(Some(egui::Align::Center));
                                        }

                                        if btn.clicked() {
                                            player.engine.seek(Duration::from_millis(line.timestamp_ms));
                                        }
                                    }
                                });
                        } else {
                            // Plain (unsynced) lyrics
                            ScrollArea::vertical()
                                .auto_shrink([false, false])
                                .show(ui, |ui| {
                                    ui.spacing_mut().item_spacing = vec2(0.0, 6.0);

                                    for line in &lyrics.lines {
                                        if line.text.trim().is_empty() {
                                            ui.add_space(10.0);
                                        } else {
                                            ui.label(
                                                RichText::new(&line.text)
                                                    .color(theme.text_primary)
                                                    .size(14.5)
                                                    .line_height(Some(22.0)),
                                            );
                                        }
                                    }
                                });
                        }
                    } else {
                        // Empty state: lyrics is None
                        render_lyrics_empty_state(ui, theme, &track, &mut is_search_open, &mut search_query);
                    }
                });

                // Persist search drawer state in ui temp data
                ui.data_mut(|d| {
                    d.insert_temp(search_open_id, is_search_open);
                    d.insert_temp(search_query_id, search_query);
                });
            });

            // =================================================================
            // COLUMN 3 (RIGHT): Audio Analyzer Card
            // =================================================================
            cols[2].vertical(|ui| {
                ui.spacing_mut().item_spacing = vec2(0.0, 10.0);

                let mut analyzer_frame = theme.glass_frame(16.0);
                analyzer_frame.inner_margin = egui::Margin::same(14.0);

                analyzer_frame.show(ui, |ui| {
                    // 1. Format Header with glowing HI-RES LOSSLESS / LOSSLESS badge
                    ui.horizontal(|ui| {
                        render_icon(ui, Icon::BarChart, 14.0, theme.accent);
                        ui.add_space(4.0);
                        ui.label(
                            RichText::new("AUDIO ANALYZER")
                                .size(12.5)
                                .color(theme.text_secondary)
                                .strong(),
                        );

                        ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                            if track.is_hires() {
                                pill_badge(
                                    ui,
                                    "HI-RES LOSSLESS",
                                    Color32::from_rgba_premultiplied(
                                        theme.accent_warm.r(),
                                        theme.accent_warm.g(),
                                        theme.accent_warm.b(),
                                        45,
                                    ),
                                    theme.accent_warm,
                                );
                            } else if track.is_lossless() {
                                pill_badge(
                                    ui,
                                    "LOSSLESS",
                                    Color32::from_rgba_premultiplied(
                                        theme.accent_warm.r(),
                                        theme.accent_warm.g(),
                                        theme.accent_warm.b(),
                                        35,
                                    ),
                                    theme.accent_warm,
                                );
                            } else {
                                pill_badge(
                                    ui,
                                    &track.format.to_uppercase(),
                                    theme.card,
                                    theme.text_secondary,
                                );
                            }
                        });
                    });

                    ui.add_space(8.0);

                    // 2. Technical Stats Sub-Card
                    let specs_card = theme.card_frame(8.0);
                    specs_card.show(ui, |ui| {
                        ui.vertical(|ui| {
                            ui.spacing_mut().item_spacing = vec2(0.0, 4.0);

                            stat_row(ui, "Format / Codec", &track.format.to_uppercase(), theme);

                            let sr_val = if track.sample_rate > 0 {
                                format!("{} Hz", track.sample_rate)
                            } else {
                                "44,100 Hz".to_string()
                            };
                            stat_row(ui, "Sample Rate", &sr_val, theme);

                            let bit_depth = if track.is_hires() { "24-bit" } else { "16-bit" };
                            let br_val = if track.bitrate_kbps > 0 {
                                format!("{} · {} kbps", bit_depth, track.bitrate_kbps)
                            } else {
                                format!("{} · Lossless", bit_depth)
                            };
                            stat_row(ui, "Bit Depth / Rate", &br_val, theme);

                            let ch_val = if track.channels == 1 {
                                "Mono (1.0 ch)"
                            } else {
                                "Stereo (2.0 ch)"
                            };
                            stat_row(ui, "Channels", ch_val, theme);
                        });
                    });

                    ui.add_space(8.0);

                    // 3. Real-Time Audio Metrics Sub-Card (Peak dBFS, LUFS, Dynamic Range)
                    let metrics_card = theme.card_frame(8.0);
                    metrics_card.show(ui, |ui| {
                        ui.horizontal(|ui| {
                            let col_w = (ui.available_width() - 8.0) / 3.0;

                            // Metric 1: Peak Level (dBFS)
                            ui.allocate_ui_with_layout(vec2(col_w, 42.0), Layout::top_down(egui::Align::Center), |ui| {
                                ui.label(RichText::new("PEAK").size(9.5).color(theme.text_muted).strong());
                                let peak_str = if player.visualizer.peak_dbfs <= -55.0 {
                                    "-∞ dBFS".to_string()
                                } else {
                                    format!("{:.1} dBFS", player.visualizer.peak_dbfs)
                                };
                                let peak_col = if player.visualizer.peak_dbfs > -1.0 {
                                    theme.accent_warm
                                } else {
                                    theme.text_primary
                                };
                                ui.label(RichText::new(peak_str).size(12.5).color(peak_col).strong());
                            });

                            // Metric 2: LUFS
                            ui.allocate_ui_with_layout(vec2(col_w, 42.0), Layout::top_down(egui::Align::Center), |ui| {
                                ui.label(RichText::new("LUFS").size(9.5).color(theme.text_muted).strong());
                                let lufs_str = if player.visualizer.lufs <= -55.0 {
                                    "-∞".to_string()
                                } else {
                                    format!("{:.1}", player.visualizer.lufs)
                                };
                                ui.label(RichText::new(lufs_str).size(12.5).color(theme.text_primary).strong());
                            });

                            // Metric 3: Dynamic Range
                            ui.allocate_ui_with_layout(vec2(col_w, 42.0), Layout::top_down(egui::Align::Center), |ui| {
                                ui.label(RichText::new("DYN RANGE").size(9.5).color(theme.text_muted).strong());
                                let dr_str = if player.visualizer.dynamic_range < 1.0 {
                                    "--".to_string()
                                } else {
                                    format!("DR {:.0}", player.visualizer.dynamic_range)
                                };
                                ui.label(RichText::new(dr_str).size(12.5).color(theme.accent).strong());
                            });
                        });
                    });

                    ui.add_space(8.0);

                    // 4. Live L/R Stereo Volume Level Meters
                    let meter_card = theme.card_frame(8.0);
                    meter_card.show(ui, |ui| {
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("STEREO METERS").size(10.0).color(theme.text_muted).strong());
                                ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.label(RichText::new("0 dB").size(9.0).color(theme.text_muted));
                                });
                            });

                            ui.add_space(4.0);
                            render_channel_meter(ui, "L", player.visualizer.left_level, player.visualizer.left_peak, theme);
                            ui.add_space(3.0);
                            render_channel_meter(ui, "R", player.visualizer.right_level, player.visualizer.right_peak, theme);
                        });
                    });

                    ui.add_space(8.0);

                    // 5. 20Hz - 20kHz Responsive Audio Spectrum Visualizer
                    let spec_card = theme.card_frame(8.0);
                    spec_card.show(ui, |ui| {
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("SPECTRUM").size(10.0).color(theme.text_muted).strong());
                                ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.selectable_label(player.visualizer.style == VisualizerStyle::Scope, "Scope").clicked() {
                                        player.visualizer.style = VisualizerStyle::Scope;
                                    }
                                    if ui.selectable_label(player.visualizer.style == VisualizerStyle::Mirror, "Mirror").clicked() {
                                        player.visualizer.style = VisualizerStyle::Mirror;
                                    }
                                    if ui.selectable_label(player.visualizer.style == VisualizerStyle::Bars, "Bars").clicked() {
                                        player.visualizer.style = VisualizerStyle::Bars;
                                    }
                                });
                            });

                            ui.add_space(4.0);

                            // Responsive Spectrum Visualizer Canvas
                            let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 76.0), egui::Sense::hover());
                            render_spectrum(
                                ui.painter(),
                                rect,
                                &player.visualizer.band_values,
                                &player.visualizer.peak_values,
                                &player.visualizer.wave_buffer,
                                player.visualizer.style,
                                theme.accent,
                                theme,
                            );

                            ui.add_space(4.0);

                            // Frequency Axis Markers (20Hz - 20kHz) proportionally spaced across spectrum width
                            let (axis_rect, _) = ui.allocate_exact_size(vec2(rect.width(), 16.0), egui::Sense::hover());
                            let painter = ui.painter();

                            let freq_markers: [(&str, f32); 7] = [
                                ("20Hz", 0.00),
                                ("100Hz", 0.17),
                                ("500Hz", 0.36),
                                ("1kHz", 0.52),
                                ("5kHz", 0.70),
                                ("10kHz", 0.86),
                                ("20kHz", 1.00),
                            ];

                            for &(lbl, fraction) in &freq_markers {
                                let x = axis_rect.min.x + fraction * axis_rect.width();

                                // Subtle tick mark connecting spectrum to frequency label
                                painter.line_segment(
                                    [pos2(x, axis_rect.min.y), pos2(x, axis_rect.min.y + 3.0)],
                                    Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(180, 140, 255, 60)),
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
                                    theme.text_muted,
                                );
                            }
                        });
                    });
                });
            });
        });
    } else {
        ui.centered_and_justified(|ui| {
            ui.vertical_centered(|ui| {
                let (icon_rect, _) = ui.allocate_exact_size(vec2(40.0, 40.0), egui::Sense::hover());
                paint_icon(ui.painter(), Icon::Disc, icon_rect, theme.text_muted);
                ui.add_space(8.0);
                ui.label(RichText::new("Nothing Playing").size(20.0).color(theme.text_muted));
                ui.add_space(8.0);
                ui.label(RichText::new("Select a song from Songs or Home to start listening.").color(theme.text_secondary));
            });
        });
    }
}

fn get_lyrics_badge(player: &PlayerController) -> (&'static str, Color32, Color32) {
    if player.is_loading_lyrics {
        return (
            "Searching...",
            Color32::from_rgba_unmultiplied(168, 85, 247, 40),
            Color32::from_rgb(216, 180, 254),
        );
    }

    let lyrics = match player.lyrics.as_ref() {
        Some(l) if !l.lines.is_empty() => l,
        _ => {
            return (
                "No Lyrics",
                Color32::from_rgba_unmultiplied(40, 32, 60, 180),
                Color32::from_rgb(148, 140, 175),
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
            Color32::from_rgba_unmultiplied(59, 130, 246, 50),
            Color32::from_rgb(147, 197, 253),
        );
    }

    if lyrics.source == "Embedded Tag" {
        return (
            "Embedded Tag",
            Color32::from_rgba_unmultiplied(16, 185, 129, 45),
            Color32::from_rgb(110, 231, 183),
        );
    }

    if lyrics.is_synced {
        (
            "LRCLIB (Synced)",
            Color32::from_rgba_unmultiplied(168, 85, 247, 45),
            Color32::from_rgb(216, 180, 254),
        )
    } else {
        (
            "Plain Lyrics (LRCLIB)",
            Color32::from_rgba_unmultiplied(75, 85, 99, 50),
            Color32::from_rgb(209, 213, 219),
        )
    }
}

fn search_online_header_button(ui: &mut Ui, is_active: bool, theme: &Theme) -> egui::Response {
    let height = 24.0;
    let icon_size = 12.0;
    let font_size = 11.0;
    let text = if is_active {
        "Close Search"
    } else {
        "Search Online (LRCLIB)"
    };
    let text_color = if is_active {
        theme.accent
    } else {
        theme.text_secondary
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
        theme.accent_glow
    } else if resp.hovered() {
        Color32::from_rgba_premultiplied(
            theme.accent.r(),
            theme.accent.g(),
            theme.accent.b(),
            35,
        )
    } else {
        theme.card
    };

    let stroke = if is_active {
        Stroke::new(1.0_f32, theme.border_accent)
    } else if resp.hovered() {
        Stroke::new(
            1.0_f32,
            Color32::from_rgba_premultiplied(
                theme.accent.r(),
                theme.accent.g(),
                theme.accent.b(),
                120,
            ),
        )
    } else {
        Stroke::new(1.0_f32, theme.border)
    };

    ui.painter().rect_filled(rect, Rounding::same(6.0), fill);
    ui.painter().rect_stroke(rect, Rounding::same(6.0), stroke);

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

fn render_lyrics_empty_state(
    ui: &mut Ui,
    theme: &Theme,
    track: &crate::library::Track,
    is_search_open: &mut bool,
    search_query: &mut String,
) {
    ui.add_space(48.0);
    ui.vertical_centered(|ui| {
        let (icon_rect, _) = ui.allocate_exact_size(vec2(36.0, 36.0), egui::Sense::hover());
        paint_icon(ui.painter(), Icon::Mic, icon_rect, theme.text_muted);
        ui.add_space(12.0);
        ui.label(
            RichText::new("No lyrics found for this track")
                .size(15.0)
                .color(theme.text_primary)
                .strong(),
        );
        ui.add_space(4.0);
        ui.label(
            RichText::new("Search the LRCLIB online database for synchronized or plain lyrics.")
                .size(12.5)
                .color(theme.text_muted),
        );
        ui.add_space(18.0);

        let btn = egui::Button::new(
            RichText::new("Search LRCLIB")
                .color(Color32::from_rgb(11, 9, 20))
                .size(13.5)
                .strong(),
        )
        .fill(theme.accent)
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

fn stat_row(ui: &mut Ui, label: &str, val: &str, theme: &Theme) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(label).size(11.0).color(theme.text_muted));
        ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(RichText::new(val).size(11.5).color(theme.text_primary).strong());
        });
    });
}

fn render_channel_meter(
    ui: &mut Ui,
    channel_lbl: &str,
    level: f32,
    peak: f32,
    theme: &Theme,
) {
    ui.horizontal(|ui| {
        ui.allocate_ui_with_layout(vec2(14.0, 14.0), Layout::left_to_right(egui::Align::Center), |ui| {
            ui.label(RichText::new(channel_lbl).size(10.0).color(theme.text_secondary).strong());
        });

        let meter_width = ui.available_width() - 52.0;
        let (rect, _) = ui.allocate_exact_size(vec2(meter_width, 8.0), egui::Sense::hover());

        // Background track
        ui.painter().rect_filled(
            rect,
            Rounding::same(2.0),
            Color32::from_rgba_premultiplied(180, 140, 255, 20),
        );

        // Filled level bar
        let clamped_level = level.clamp(0.0, 1.0);
        if clamped_level > 0.01 {
            let fill_w = (rect.width() * clamped_level).max(2.0);
            let fill_rect = Rect::from_min_size(rect.min, vec2(fill_w, rect.height()));

            let bar_color = if clamped_level > 0.88 {
                theme.accent_warm
            } else {
                theme.accent
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
                theme.accent_warm
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
            ui.label(RichText::new(db_val).size(9.5).color(theme.text_muted));
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppConfig;
    use crate::library::model::Track;
    use crate::lyrics::{LyricLine, Lyrics, LrclibRecord};

    #[test]
    fn test_lyrics_source_badge_mapping() {
        let cfg = AppConfig::default();
        let mut player = PlayerController::new(&cfg);

        // 1. None -> "No Lyrics"
        player.lyrics = None;
        let (label, _, _) = get_lyrics_badge(&player);
        assert_eq!(label, "No Lyrics");

        // 2. Loading -> "Searching..."
        player.is_loading_lyrics = true;
        let (label, _, _) = get_lyrics_badge(&player);
        assert_eq!(label, "Searching...");
        player.is_loading_lyrics = false;

        // 3. Instrumental
        player.lyrics = Some(Lyrics {
            lines: vec![LyricLine { timestamp_ms: 0, text: "Instrumental".into() }],
            is_synced: false,
            source: "LRCLIB (Instrumental)".into(),
        });
        let (label, _, _) = get_lyrics_badge(&player);
        assert_eq!(label, "Instrumental");

        // 4. Local .lrc
        player.lyrics = Some(Lyrics {
            lines: vec![LyricLine { timestamp_ms: 1000, text: "Line 1".into() }],
            is_synced: true,
            source: "Local .lrc".into(),
        });
        let (label, _, _) = get_lyrics_badge(&player);
        assert_eq!(label, "Local .lrc");

        // 5. Embedded Tag
        player.lyrics = Some(Lyrics {
            lines: vec![LyricLine { timestamp_ms: 1000, text: "Line 1".into() }],
            is_synced: false,
            source: "Embedded Tag".into(),
        });
        let (label, _, _) = get_lyrics_badge(&player);
        assert_eq!(label, "Embedded Tag");

        // 6. LRCLIB Synced
        player.lyrics = Some(Lyrics {
            lines: vec![LyricLine { timestamp_ms: 1000, text: "Line 1".into() }],
            is_synced: true,
            source: "LRCLIB (Synced)".into(),
        });
        let (label, _, _) = get_lyrics_badge(&player);
        assert_eq!(label, "LRCLIB (Synced)");

        // 7. Plain Lyrics
        player.lyrics = Some(Lyrics {
            lines: vec![LyricLine { timestamp_ms: 0, text: "Plain text line".into() }],
            is_synced: false,
            source: "LRCLIB (Plain)".into(),
        });
        let (label, _, _) = get_lyrics_badge(&player);
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
                render_now_playing(ui, &mut player, &theme, &mut art_manager);

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

                render_now_playing(ui, &mut player, &theme, &mut art_manager);
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
                render_now_playing(ui, &mut player, &theme, &mut art_manager);

                // Apply custom lyrics test
                let selected = player.manual_search_results.as_ref().unwrap()[0].clone();
                player.apply_custom_lyrics(&selected);
                assert!(player.lyrics.is_some());
                let lyrics = player.lyrics.as_ref().unwrap();
                assert!(lyrics.is_synced);
                assert_eq!(lyrics.lines.len(), 1);
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
                render_now_playing(ui, &mut player, &theme, &mut art_manager);

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
                render_now_playing(ui, &mut player, &theme, &mut art_manager);

                // 3. Instrumental state
                player.lyrics = Some(Lyrics {
                    lines: vec![
                        LyricLine { timestamp_ms: 0, text: "Instrumental".into() },
                    ],
                    is_synced: false,
                    source: "LRCLIB (Instrumental)".into(),
                });
                render_now_playing(ui, &mut player, &theme, &mut art_manager);
            });
        });
    }
}
