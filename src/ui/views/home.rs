use crate::audio::PlayerController;
use crate::library::model::Track;
use crate::ui::art::AlbumArtManager;
use crate::ui::components::{dark_pill_button, white_pill_button};
use crate::ui::icons::{paint_icon, Icon};
use crate::ui::sidebar::NavTab;
use crate::ui::theme::Theme;
use egui::{pos2, vec2, Color32, Layout, Rect, RichText, Rounding, ScrollArea, Sense, Stroke, Ui};
use std::collections::HashSet;

#[cfg(windows)]
fn get_local_date_and_greeting() -> (String, String) {
    #[allow(clippy::upper_case_acronyms)]
    #[repr(C)]
    struct SYSTEMTIME {
        w_year: u16,
        w_month: u16,
        w_day_of_week: u16,
        w_day: u16,
        w_hour: u16,
        w_minute: u16,
        w_second: u16,
        w_milliseconds: u16,
    }
    extern "system" {
        fn GetLocalTime(lp_system_time: *mut SYSTEMTIME);
    }
    let mut st = SYSTEMTIME {
        w_year: 0,
        w_month: 0,
        w_day_of_week: 0,
        w_day: 0,
        w_hour: 0,
        w_minute: 0,
        w_second: 0,
        w_milliseconds: 0,
    };
    unsafe {
        GetLocalTime(&mut st);
    }
    let days = [
        "SUNDAY", "MONDAY", "TUESDAY", "WEDNESDAY", "THURSDAY", "FRIDAY", "SATURDAY",
    ];
    let months = [
        "JANUARY", "FEBRUARY", "MARCH", "APRIL", "MAY", "JUNE", "JULY", "AUGUST", "SEPTEMBER",
        "OCTOBER", "NOVEMBER", "DECEMBER",
    ];
    let day_name = days.get(st.w_day_of_week as usize).copied().unwrap_or("MONDAY");
    let month_idx = (st.w_month as usize).saturating_sub(1);
    let month_name = months.get(month_idx).copied().unwrap_or("NOVEMBER");
    let date_str = format!("{} · {:02} {}", day_name, st.w_day, month_name);

    let greeting = match st.w_hour {
        5..=11 => "Good morning.",
        12..=17 => "Good afternoon.",
        _ => "Good evening.",
    };
    (date_str, greeting.to_string())
}

#[cfg(not(windows))]
fn get_local_date_and_greeting() -> (String, String) {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let days_since_epoch = secs / 86400;
    let day_of_week_idx = ((days_since_epoch + 4) % 7) as usize;
    let days = [
        "SUNDAY", "MONDAY", "TUESDAY", "WEDNESDAY", "THURSDAY", "FRIDAY", "SATURDAY",
    ];
    let day_name = days.get(day_of_week_idx).copied().unwrap_or("MONDAY");
    let hour = ((secs % 86400) / 3600) as u32;
    let greeting = match hour {
        5..=11 => "Good morning.",
        12..=17 => "Good afternoon.",
        _ => "Good evening.",
    };
    (format!("{} · CATALOG", day_name), greeting.to_string())
}

#[allow(clippy::too_many_arguments)]
pub fn render_home(
    ui: &mut Ui,
    tracks: &[Track],
    recent_history: &[Track],
    player: &mut PlayerController,
    theme: &Theme,
    on_play_track: &mut Option<Track>,
    art_manager: &mut AlbumArtManager,
    on_trigger_scan: &mut bool,
    on_navigate: &mut Option<NavTab>,
) {
    ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing = vec2(0.0, 20.0);

            // =================================================================
            // 1. GREETING HEADER
            // =================================================================
            render_greeting_header(ui, theme);

            // =================================================================
            // 2. ROW 1 — MAIN BENTO GRID (2-COLUMN ASYMMETRIC SPLIT)
            // =================================================================
            render_row1_bento(
                ui,
                tracks,
                recent_history,
                player,
                theme,
                art_manager,
                on_play_track,
                on_navigate,
                on_trigger_scan,
            );

            // =================================================================
            // 3. ROW 2 — ON REPEAT & RECENTLY ADDED SPLIT GRID
            // =================================================================
            render_row2_split_grid(
                ui,
                tracks,
                theme,
                art_manager,
                on_play_track,
                on_navigate,
            );

            // =================================================================
            // 4. ROW 3 — JUMP BACK IN
            // =================================================================
            render_row3_jump_back_in(
                ui,
                tracks,
                recent_history,
                theme,
                art_manager,
                player,
                on_play_track,
                on_navigate,
            );
        });
}

// =============================================================================
// 1. GREETING HEADER
// =============================================================================
fn render_greeting_header(ui: &mut Ui, theme: &Theme) {
    let (date_str, greeting) = get_local_date_and_greeting();

    ui.horizontal(|ui| {
        // Left Column: Date in uppercase muted tracking + bold greeting
        ui.vertical(|ui| {
            ui.label(
                RichText::new(&date_str)
                    .size(11.5)
                    .color(theme.text_muted)
                    .strong(),
            );
            ui.add_space(2.0);
            ui.label(
                RichText::new(&greeting)
                    .size(32.0)
                    .color(Color32::from_rgb(244, 244, 245))
                    .strong(),
            );
        });

        // Right Column: ECLIPSED brand block + circular waveform/equalizer badge
        ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
            // Circular waveform/equalizer badge
            let (badge_rect, badge_resp) = ui.allocate_exact_size(vec2(44.0, 44.0), Sense::hover());
            let badge_bg = if badge_resp.hovered() {
                Color32::from_rgb(28, 28, 32)
            } else {
                Color32::from_rgb(22, 22, 26)
            };
            ui.painter().circle_filled(badge_rect.center(), 22.0, badge_bg);
            ui.painter().circle_stroke(
                badge_rect.center(),
                22.0,
                Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(255, 255, 255, 22)),
            );
            let icon_rect = Rect::from_center_size(badge_rect.center(), vec2(20.0, 20.0));
            paint_icon(ui.painter(), Icon::BarChart, icon_rect, theme.accent);

            ui.add_space(14.0);

            // Brand Text Stack
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("E C L I P S E D")
                            .size(13.5)
                            .color(Color32::from_rgb(244, 244, 245))
                            .strong(),
                    );
                    ui.add_space(4.0);
                    // Version tag pill
                    let (tag_rect, _) = ui.allocate_exact_size(vec2(42.0, 16.0), Sense::hover());
                    ui.painter().rect_filled(tag_rect, Rounding::same(4.0), Color32::from_rgb(26, 26, 30));
                    ui.painter().rect_stroke(
                        tag_rect,
                        Rounding::same(4.0),
                        Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(255, 255, 255, 18)),
                    );
                    ui.painter().text(
                        tag_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "v1.2.0",
                        egui::FontId::proportional(10.0),
                        theme.text_muted,
                    );
                });
                ui.label(
                    RichText::new("music, in the dark.")
                        .size(11.0)
                        .color(theme.text_muted),
                );
            });
        });
    });
}

// =============================================================================
// 2. ROW 1 — MAIN BENTO GRID (2-COLUMN ASYMMETRIC SPLIT)
// =============================================================================
#[allow(clippy::too_many_arguments)]
fn render_row1_bento(
    ui: &mut Ui,
    tracks: &[Track],
    recent_history: &[Track],
    player: &mut PlayerController,
    theme: &Theme,
    art_manager: &mut AlbumArtManager,
    on_play_track: &mut Option<Track>,
    on_navigate: &mut Option<NavTab>,
    on_trigger_scan: &mut bool,
) {
    let available_w = ui.available_width();
    let gap = 14.0_f32;
    let is_wide = available_w >= 640.0;

    let left_w = if is_wide {
        ((available_w - gap) * 0.55).max(340.0)
    } else {
        available_w
    };
    let right_w = if is_wide {
        (available_w - gap - left_w).max(280.0)
    } else {
        available_w
    };

    let active_track: Option<Track> = player
        .current_track()
        .cloned()
        .or_else(|| recent_history.first().cloned())
        .or_else(|| tracks.first().cloned());

    if is_wide {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = vec2(gap, 0.0);

            // Left Card (~55% width): PICK UP WHERE YOU LEFT OFF
            ui.allocate_ui_with_layout(
                vec2(left_w, 0.0),
                Layout::top_down(egui::Align::Min),
                |ui| {
                    render_pick_up_card(
                        ui,
                        left_w,
                        active_track.as_ref(),
                        player,
                        theme,
                        art_manager,
                        on_play_track,
                        on_navigate,
                        on_trigger_scan,
                    );
                },
            );

            // Right Nested 2x2 Bento Grid (~45% width)
            ui.allocate_ui_with_layout(
                vec2(right_w, 0.0),
                Layout::top_down(egui::Align::Min),
                |ui| {
                    render_stats_2x2_bento(ui, right_w, tracks, theme, player, on_navigate);
                },
            );
        });
    } else {
        // Vertically stacked for narrow viewports
        render_pick_up_card(
            ui,
            left_w,
            active_track.as_ref(),
            player,
            theme,
            art_manager,
            on_play_track,
            on_navigate,
            on_trigger_scan,
        );
        ui.add_space(gap);
        render_stats_2x2_bento(ui, right_w, tracks, theme, player, on_navigate);
    }
}

#[allow(clippy::too_many_arguments)]
fn render_pick_up_card(
    ui: &mut Ui,
    card_w: f32,
    active_track: Option<&Track>,
    player: &mut PlayerController,
    theme: &Theme,
    art_manager: &mut AlbumArtManager,
    on_play_track: &mut Option<Track>,
    on_navigate: &mut Option<NavTab>,
    on_trigger_scan: &mut bool,
) {
    let mut frame = theme.card_frame(14.0);
    frame.fill = Color32::from_rgb(22, 22, 24); // #161618
    frame.stroke = Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(255, 255, 255, 15)); // subtle rgba(255,255,255,0.06)
    frame.inner_margin = egui::Margin::symmetric(16.0, 16.0);

    let frame_response = frame.show(ui, |ui| {
        ui.set_width(card_w - 32.0);

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = vec2(16.0, 0.0);

            // Large 136x136px rounded square Album Cover with soft shadow / border
            let (art_box, art_resp) = ui.allocate_exact_size(vec2(136.0, 136.0), Sense::click());
            if art_resp.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }

            let mut painted = false;
            if let Some(track) = active_track {
                if let Some(tex) = art_manager.get_or_load(ui.ctx(), &track.path) {
                    ui.painter().image(
                        tex.id(),
                        art_box,
                        Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                        Color32::WHITE,
                    );
                    ui.painter().rect_stroke(
                        art_box,
                        Rounding::same(10.0),
                        Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(255, 255, 255, 25)),
                    );
                    painted = true;
                }
            }

            if !painted {
                ui.painter().rect_filled(art_box, Rounding::same(10.0), Color32::from_rgb(28, 28, 32));
                ui.painter().rect_stroke(
                    art_box,
                    Rounding::same(10.0),
                    Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(255, 255, 255, 18)),
                );
                let icon_box = Rect::from_center_size(art_box.center(), vec2(48.0, 48.0));
                paint_icon(ui.painter(), Icon::Disc, icon_box, theme.accent);
            }

            if art_resp.clicked() {
                if let Some(track) = active_track {
                    *on_play_track = Some(track.clone());
                }
            }

            // Right Stack
            let right_content_w = (card_w - 32.0 - 136.0 - 16.0).max(120.0);
            ui.allocate_ui_with_layout(
                vec2(right_content_w, 136.0),
                Layout::top_down(egui::Align::Min),
                |ui| {
                    ui.spacing_mut().item_spacing = vec2(0.0, 3.5);

                    // Uppercase muted label: PICK UP WHERE YOU LEFT OFF
                    ui.label(
                        RichText::new("PICK UP WHERE YOU LEFT OFF")
                            .size(11.0)
                            .color(theme.text_muted)
                            .strong(),
                    );

                    if let Some(track) = active_track {
                        // Large bold track title: 18px bold #f4f4f5
                        let char_limit = (right_content_w / 10.5).max(10.0) as usize;
                        let title_str = truncate_str(track.display_title(), char_limit);
                        ui.label(
                            RichText::new(title_str)
                                .size(18.0)
                                .color(Color32::from_rgb(244, 244, 245))
                                .strong(),
                        );

                        // Muted artist name: 13px #a1a1aa
                        let artist_str = truncate_str(track.display_artist(), char_limit + 4);
                        ui.label(
                            RichText::new(artist_str)
                                .size(13.0)
                                .color(Color32::from_rgb(161, 161, 170)),
                        );

                        // Single-line technical metadata: track.tech_summary_str() using 11.5px #71717a
                        ui.label(
                            RichText::new(track.tech_summary_str())
                                .size(11.5)
                                .color(Color32::from_rgb(113, 113, 122)),
                        );

                        ui.add_space(8.0);

                        // Action row: solid white Play pill button and dark Open pill button
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing = vec2(8.0, 0.0);

                            let is_current_playing = player.engine.is_playing()
                                && player.current_track().is_some_and(|t| t.id == track.id);

                            let (btn_text, btn_icon) = if is_current_playing {
                                ("Pause", Icon::Pause)
                            } else {
                                ("Play", Icon::Play)
                            };

                            if white_pill_button(ui, btn_text, Some(btn_icon)).clicked() {
                                if is_current_playing {
                                    player.engine.pause();
                                } else if player.current_track().is_some_and(|t| t.id == track.id) {
                                    player.engine.resume();
                                } else {
                                    *on_play_track = Some(track.clone());
                                }
                            }

                            if dark_pill_button(ui, "Open", None).clicked() {
                                *on_navigate = Some(NavTab::NowPlaying);
                            }
                        });
                    } else {
                        ui.add_space(10.0);
                        ui.label(
                            RichText::new("Your library is empty")
                                .size(16.0)
                                .color(Color32::from_rgb(244, 244, 245))
                                .strong(),
                        );
                        ui.label(
                            RichText::new("Scan a local music folder to get started.")
                                .size(12.0)
                                .color(theme.text_muted),
                        );
                        ui.add_space(12.0);
                        if white_pill_button(ui, "Scan Folder", Some(Icon::Refresh)).clicked() {
                            *on_trigger_scan = true;
                        }
                    }
                },
            );
        });

        // Subtle horizontal progress line along the bottom of the card
        ui.add_space(8.0);
        let (line_rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 3.0), Sense::hover());
        ui.painter().rect_filled(
            line_rect,
            Rounding::same(1.5),
            Color32::from_rgba_premultiplied(255, 255, 255, 14),
        );

        if let Some(track) = active_track {
            let progress = if player.current_track().is_some_and(|t| t.id == track.id)
                && track.duration_secs > 0.0
            {
                (player.engine.position().as_secs_f64() / track.duration_secs).clamp(0.0, 1.0) as f32
            } else {
                0.0
            };

            if progress > 0.0 {
                let fill_w = (line_rect.width() * progress).max(3.0);
                let filled_rect = Rect::from_min_size(line_rect.min, vec2(fill_w, 3.0));
                ui.painter().rect_filled(filled_rect, Rounding::same(1.5), Color32::WHITE);
            }
        }
    });

    let _ = frame_response;
}

fn render_stats_2x2_bento(
    ui: &mut Ui,
    right_w: f32,
    tracks: &[Track],
    theme: &Theme,
    player: &mut PlayerController,
    on_navigate: &mut Option<NavTab>,
) {
    let total_tracks = tracks.len();
    let total_albums = tracks
        .iter()
        .map(|t| {
            if t.album.trim().is_empty() {
                "Unknown Album"
            } else {
                t.album.as_str()
            }
        })
        .collect::<HashSet<_>>()
        .len();

    let total_artists = tracks
        .iter()
        .map(|t| t.display_artist())
        .collect::<HashSet<_>>()
        .len();

    let lossless_count = tracks.iter().filter(|t| t.is_lossless()).count();
    let total_secs: f64 = tracks.iter().map(|t| t.duration_secs).sum();
    let total_hours = total_secs / 3600.0;
    let total_plays: u32 = tracks.iter().map(|t| t.play_count).sum();
    let lossless_pct = if total_tracks > 0 {
        (lossless_count as f32 / total_tracks as f32) * 100.0
    } else {
        100.0
    };

    let fav_count = tracks.iter().filter(|t| t.favorite).count();

    let card_gap = 10.0_f32;
    let sub_w = ((right_w - card_gap) / 2.0).max(120.0);
    let sub_h = 85.0_f32;

    // Row 1: LIBRARY & LISTENING TIME
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = vec2(card_gap, 0.0);

        // Card 1: LIBRARY
        render_bento_subcard(
            ui,
            sub_w,
            sub_h,
            theme,
            |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("LIBRARY")
                            .size(10.0)
                            .color(theme.text_muted)
                            .strong(),
                    );
                    ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                        let (icon_rect, _) = ui.allocate_exact_size(vec2(14.0, 14.0), Sense::hover());
                        paint_icon(ui.painter(), Icon::Music, icon_rect, theme.text_muted);
                    });
                });
                ui.add_space(4.0);
                ui.label(
                    RichText::new(format!("{} tracks", total_tracks))
                        .size(16.0)
                        .color(Color32::from_rgb(244, 244, 245))
                        .strong(),
                );
                ui.label(
                    RichText::new(format!("{} albums · {} artists", total_albums, total_artists))
                        .size(11.0)
                        .color(theme.text_muted),
                );
            },
            || {
                *on_navigate = Some(NavTab::Songs);
            },
        );

        // Card 2: LISTENING TIME
        render_bento_subcard(
            ui,
            sub_w,
            sub_h,
            theme,
            |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("LISTENING TIME")
                            .size(10.0)
                            .color(theme.text_muted)
                            .strong(),
                    );
                    ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                        let (icon_rect, _) = ui.allocate_exact_size(vec2(14.0, 14.0), Sense::hover());
                        paint_icon(ui.painter(), Icon::Clock, icon_rect, theme.text_muted);
                    });
                });
                ui.add_space(4.0);
                ui.label(
                    RichText::new(format!("{:.1} h", total_hours))
                        .size(16.0)
                        .color(Color32::from_rgb(244, 244, 245))
                        .strong(),
                );
                ui.label(
                    RichText::new(format!("{:.0}% lossless · {} plays", lossless_pct, total_plays))
                        .size(11.0)
                        .color(theme.text_muted),
                );
            },
            || {
                *on_navigate = Some(NavTab::MostPlayed);
            },
        );
    });

    ui.add_space(card_gap);

    // Row 2: Favorites & Random Discovery
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = vec2(card_gap, 0.0);

        // Card 3: Favorites
        render_action_subcard(
            ui,
            sub_w,
            sub_h,
            Icon::Heart,
            Color32::from_rgb(239, 68, 68),
            "Favorites",
            &format!("{} TRACKS", fav_count),
            theme,
            || {
                let favs: Vec<Track> = tracks.iter().filter(|t| t.favorite).cloned().collect();
                if !favs.is_empty() {
                    player.load_queue(favs, 0);
                }
            },
            || {
                *on_navigate = Some(NavTab::Favorites);
            },
        );

        // Card 4: Random Discovery
        render_action_subcard(
            ui,
            sub_w,
            sub_h,
            Icon::Dices,
            theme.accent,
            "Random Discovery",
            &format!("{} TRACKS", tracks.len().min(50)),
            theme,
            || {
                use rand::seq::SliceRandom;
                let mut rng_tracks = tracks.to_vec();
                let mut rng = rand::thread_rng();
                rng_tracks.shuffle(&mut rng);
                rng_tracks.truncate(50);
                if !rng_tracks.is_empty() {
                    player.shuffle = true;
                    player.load_queue(rng_tracks, 0);
                }
            },
            || {
                *on_navigate = Some(NavTab::RandomDiscovery);
            },
        );
    });
}

fn render_bento_subcard<F, C>(
    ui: &mut Ui,
    w: f32,
    h: f32,
    theme: &Theme,
    content: F,
    on_click: C,
) where
    F: FnOnce(&mut Ui),
    C: FnOnce(),
{
    let mut frame = theme.card_frame(12.0);
    frame.fill = Color32::from_rgb(22, 22, 24); // #161618
    frame.stroke = Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(255, 255, 255, 15));
    frame.inner_margin = egui::Margin::symmetric(14.0, 10.0);

    let resp = frame.show(ui, |ui| {
        ui.allocate_ui_with_layout(
            vec2(w - 28.0, h - 20.0),
            Layout::top_down(egui::Align::Min),
            content,
        );
    });

    let interact = resp.response.interact(Sense::click());
    if interact.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        ui.painter().rect_stroke(
            resp.response.rect,
            Rounding::same(12.0),
            Stroke::new(1.0_f32, theme.border_accent),
        );
    }
    if interact.clicked() {
        on_click();
    }
}

#[allow(clippy::too_many_arguments)]
fn render_action_subcard<P, C>(
    ui: &mut Ui,
    w: f32,
    h: f32,
    icon: Icon,
    icon_color: Color32,
    title: &str,
    subtitle: &str,
    theme: &Theme,
    on_play: P,
    on_click_card: C,
) where
    P: FnOnce(),
    C: FnOnce(),
{
    let mut frame = theme.card_frame(12.0);
    frame.fill = Color32::from_rgb(22, 22, 24); // #161618
    frame.stroke = Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(255, 255, 255, 15));
    frame.inner_margin = egui::Margin::symmetric(14.0, 10.0);

    let mut play_clicked = false;

    let resp = frame.show(ui, |ui| {
        ui.allocate_ui_with_layout(
            vec2(w - 28.0, h - 20.0),
            Layout::left_to_right(egui::Align::Center),
            |ui| {
                // Left text stack
                let left_w = (w - 28.0 - 32.0).max(60.0);
                ui.allocate_ui_with_layout(
                    vec2(left_w, h - 20.0),
                    Layout::top_down(egui::Align::Min),
                    |ui| {
                        // Top icon
                        let (icon_box, _) = ui.allocate_exact_size(vec2(14.0, 14.0), Sense::hover());
                        paint_icon(ui.painter(), icon, icon_box, icon_color);

                        ui.add_space(3.0);
                        ui.label(
                            RichText::new(title)
                                .size(13.5)
                                .color(Color32::from_rgb(244, 244, 245))
                                .strong(),
                        );
                        ui.label(
                            RichText::new(subtitle)
                                .size(10.0)
                                .color(theme.text_muted)
                                .strong(),
                        );
                    },
                );

                // Bottom-right dark circular button with Icon::Play
                ui.with_layout(Layout::right_to_left(egui::Align::BOTTOM), |ui| {
                    let btn_size = 28.0_f32;
                    let (btn_rect, btn_resp) = ui.allocate_exact_size(vec2(btn_size, btn_size), Sense::click());
                    if btn_resp.hovered() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    }
                    let bg_color = if btn_resp.is_pointer_button_down_on() {
                        Color32::from_rgb(22, 22, 26)
                    } else if btn_resp.hovered() {
                        Color32::from_rgb(44, 44, 50)
                    } else {
                        Color32::from_rgb(34, 34, 38)
                    };
                    ui.painter().circle_filled(btn_rect.center(), btn_size / 2.0, bg_color);
                    ui.painter().circle_stroke(
                        btn_rect.center(),
                        btn_size / 2.0,
                        Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(255, 255, 255, 20)),
                    );
                    let play_icon_rect = Rect::from_center_size(
                        pos2(btn_rect.center().x + 1.0, btn_rect.center().y),
                        vec2(12.0, 12.0),
                    );
                    paint_icon(ui.painter(), Icon::Play, play_icon_rect, Color32::WHITE);

                    if btn_resp.clicked() {
                        play_clicked = true;
                    }
                });
            },
        );
    });

    if play_clicked {
        on_play();
        return;
    }

    let interact = resp.response.interact(Sense::click());
    if interact.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        ui.painter().rect_stroke(
            resp.response.rect,
            Rounding::same(12.0),
            Stroke::new(1.0_f32, theme.border_accent),
        );
    }
    if interact.clicked() {
        on_click_card();
    }
}

// =============================================================================
// 3. ROW 2 — ON REPEAT & RECENTLY ADDED SPLIT GRID
// =============================================================================
fn render_row2_split_grid(
    ui: &mut Ui,
    tracks: &[Track],
    theme: &Theme,
    art_manager: &mut AlbumArtManager,
    on_play_track: &mut Option<Track>,
    on_navigate: &mut Option<NavTab>,
) {
    let available_w = ui.available_width();
    let gap = 14.0_f32;
    let is_wide = available_w >= 640.0;
    let col_w = if is_wide {
        ((available_w - gap) / 2.0).max(280.0)
    } else {
        available_w
    };

    if is_wide {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = vec2(gap, 0.0);

            // Left Card: ON REPEAT
            ui.allocate_ui_with_layout(
                vec2(col_w, 0.0),
                Layout::top_down(egui::Align::Min),
                |ui| {
                    render_on_repeat_card(ui, col_w, tracks, theme, art_manager, on_play_track);
                },
            );

            // Right Card: RECENTLY ADDED
            ui.allocate_ui_with_layout(
                vec2(col_w, 0.0),
                Layout::top_down(egui::Align::Min),
                |ui| {
                    render_recently_added_card(ui, col_w, tracks, theme, art_manager, on_play_track, on_navigate);
                },
            );
        });
    } else {
        render_on_repeat_card(ui, col_w, tracks, theme, art_manager, on_play_track);
        ui.add_space(gap);
        render_recently_added_card(ui, col_w, tracks, theme, art_manager, on_play_track, on_navigate);
    }
}

fn render_on_repeat_card(
    ui: &mut Ui,
    col_w: f32,
    tracks: &[Track],
    theme: &Theme,
    art_manager: &mut AlbumArtManager,
    on_play_track: &mut Option<Track>,
) {
    let mut frame = theme.card_frame(14.0);
    frame.fill = Color32::from_rgb(22, 22, 24); // #161618
    frame.stroke = Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(255, 255, 255, 15));
    frame.inner_margin = egui::Margin::symmetric(16.0, 14.0);

    frame.show(ui, |ui| {
        ui.set_width(col_w - 32.0);

        // Card header
        ui.label(
            RichText::new("ON REPEAT")
                .size(11.0)
                .color(theme.text_muted)
                .strong(),
        );
        ui.add_space(8.0);

        let mut on_repeat_list = tracks.to_vec();
        on_repeat_list.sort_by_key(|t| std::cmp::Reverse(t.play_count));

        let top_4: Vec<&Track> = on_repeat_list.iter().take(4).collect();

        if top_4.is_empty() {
            ui.add_space(10.0);
            ui.label(
                RichText::new("No played tracks yet.")
                    .size(12.0)
                    .color(theme.text_muted),
            );
        } else {
            for (idx, track) in top_4.iter().enumerate() {
                let row_h = 38.0_f32;
                let (row_rect, row_resp) = ui.allocate_exact_size(vec2(ui.available_width(), row_h), Sense::click());

                if row_resp.hovered() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    ui.painter().rect_filled(row_rect, Rounding::same(6.0), Color32::from_rgb(28, 28, 32));
                }

                if row_resp.clicked() {
                    *on_play_track = Some((*track).clone());
                }

                // 2-digit index (01-04)
                let idx_str = format!("{:02}", idx + 1);
                ui.painter().text(
                    pos2(row_rect.min.x + 8.0, row_rect.center().y),
                    egui::Align2::LEFT_CENTER,
                    idx_str,
                    egui::FontId::proportional(12.0),
                    theme.text_muted,
                );

                // 36x36px rounded Album Cover
                let art_rect = Rect::from_min_size(
                    pos2(row_rect.min.x + 32.0, row_rect.center().y - 18.0),
                    vec2(36.0, 36.0),
                );

                let mut painted = false;
                if let Some(tex) = art_manager.get_or_load(ui.ctx(), &track.path) {
                    ui.painter().image(
                        tex.id(),
                        art_rect,
                        Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                        Color32::WHITE,
                    );
                    ui.painter().rect_stroke(
                        art_rect,
                        Rounding::same(6.0),
                        Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(255, 255, 255, 20)),
                    );
                    painted = true;
                }

                if !painted {
                    ui.painter().rect_filled(art_rect, Rounding::same(6.0), Color32::from_rgb(32, 32, 36));
                    let icon_box = Rect::from_center_size(art_rect.center(), vec2(16.0, 16.0));
                    paint_icon(ui.painter(), Icon::Disc, icon_box, theme.accent);
                }

                // Right-aligned Artist · Nx plays
                let play_badge = if track.play_count > 0 {
                    format!("{} · {}x", track.display_artist(), track.play_count)
                } else {
                    format!("{} · 1x", track.display_artist())
                };
                let right_galley = ui.painter().layout_no_wrap(
                    play_badge,
                    egui::FontId::proportional(11.5),
                    Color32::from_rgb(113, 113, 122),
                );
                let right_pos = pos2(
                    row_rect.max.x - 8.0 - right_galley.rect.width(),
                    row_rect.center().y - right_galley.rect.height() / 2.0,
                );
                ui.painter().galley(right_pos, right_galley, Color32::from_rgb(113, 113, 122));

                // Bold track title
                let title_start_x = art_rect.max.x + 10.0;
                let max_title_w = (right_pos.x - 10.0 - title_start_x).max(20.0);
                let char_limit = (max_title_w / 7.5).max(6.0) as usize;
                let title_str = truncate_str(track.display_title(), char_limit);
                let title_galley = ui.painter().layout_no_wrap(
                    title_str,
                    egui::FontId::proportional(12.5),
                    Color32::from_rgb(244, 244, 245),
                );
                ui.painter().galley(
                    pos2(title_start_x, row_rect.center().y - title_galley.rect.height() / 2.0),
                    title_galley,
                    Color32::from_rgb(244, 244, 245),
                );

                ui.add_space(2.0);
            }
        }
    });
}

fn render_recently_added_card(
    ui: &mut Ui,
    col_w: f32,
    tracks: &[Track],
    theme: &Theme,
    art_manager: &mut AlbumArtManager,
    on_play_track: &mut Option<Track>,
    on_navigate: &mut Option<NavTab>,
) {
    let mut frame = theme.card_frame(14.0);
    frame.fill = Color32::from_rgb(22, 22, 24); // #161618
    frame.stroke = Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(255, 255, 255, 15));
    frame.inner_margin = egui::Margin::symmetric(16.0, 14.0);

    frame.show(ui, |ui| {
        ui.set_width(col_w - 32.0);

        // Card header
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("RECENTLY ADDED")
                    .size(11.0)
                    .color(theme.text_muted)
                    .strong(),
            );
            ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                let see_all = ui.add(
                    egui::Label::new(
                        RichText::new("See all")
                            .size(11.0)
                            .color(theme.text_muted),
                    )
                    .sense(Sense::click()),
                );
                if see_all.hovered() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                }
                if see_all.clicked() {
                    *on_navigate = Some(NavTab::RecentlyAdded);
                }
            });
        });
        ui.add_space(14.0);

        let mut sorted_recent = tracks.to_vec();
        sorted_recent.sort_by_key(|t| std::cmp::Reverse(t.id));
        let recent_5: Vec<&Track> = sorted_recent.iter().take(5).collect();

        if recent_5.is_empty() {
            ui.add_space(10.0);
            ui.label(
                RichText::new("No tracks added yet.")
                    .size(12.0)
                    .color(theme.text_muted),
            );
        } else {
            let inner_w = col_w - 32.0;
            let item_gap = 10.0_f32;
            let col_item_w = ((inner_w - 4.0 * item_gap) / 5.0).clamp(52.0, 72.0);
            let cover_size = col_item_w;

            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = vec2(item_gap, 0.0);

                for track in recent_5 {
                    ui.allocate_ui_with_layout(
                        vec2(col_item_w, cover_size + 42.0),
                        Layout::top_down(egui::Align::Center),
                        |ui| {
                            // Square Album Cover with border-radius 8px
                            let (art_box, art_resp) = ui.allocate_exact_size(vec2(cover_size, cover_size), Sense::click());
                            if art_resp.hovered() {
                                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                            }

                            let mut painted = false;
                            if let Some(tex) = art_manager.get_or_load(ui.ctx(), &track.path) {
                                ui.painter().image(
                                    tex.id(),
                                    art_box,
                                    Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                                    Color32::WHITE,
                                );
                                let stroke_color = if art_resp.hovered() {
                                    theme.border_accent
                                } else {
                                    Color32::from_rgba_premultiplied(255, 255, 255, 20)
                                };
                                ui.painter().rect_stroke(art_box, Rounding::same(8.0), Stroke::new(1.0_f32, stroke_color));
                                painted = true;
                            }

                            if !painted {
                                ui.painter().rect_filled(art_box, Rounding::same(8.0), Color32::from_rgb(28, 28, 32));
                                let stroke_color = if art_resp.hovered() {
                                    theme.border_accent
                                } else {
                                    Color32::from_rgba_premultiplied(255, 255, 255, 18)
                                };
                                ui.painter().rect_stroke(art_box, Rounding::same(8.0), Stroke::new(1.0_f32, stroke_color));
                                let icon_box = Rect::from_center_size(art_box.center(), vec2(22.0, 22.0));
                                paint_icon(ui.painter(), Icon::Disc, icon_box, theme.accent);
                            }

                            ui.add_space(5.0);

                            // Bold truncated track title below
                            let title_chars = (col_item_w / 7.0).max(5.0) as usize;
                            let title_str = truncate_str(track.display_title(), title_chars);
                            let title_lbl = ui.add(
                                egui::Label::new(
                                    RichText::new(title_str)
                                        .size(11.0)
                                        .color(if art_resp.hovered() {
                                            Color32::WHITE
                                        } else {
                                            Color32::from_rgb(244, 244, 245)
                                        })
                                        .strong(),
                                )
                                .sense(Sense::click()),
                            );

                            // Artist name
                            let artist_chars = (col_item_w / 6.5).max(5.0) as usize;
                            let artist_str = truncate_str(track.display_artist(), artist_chars);
                            ui.label(
                                RichText::new(artist_str)
                                    .size(9.5)
                                    .color(Color32::from_rgb(113, 113, 122)),
                            );

                            if art_resp.clicked() || title_lbl.clicked() {
                                *on_play_track = Some((*track).clone());
                            }
                        },
                    );
                }
            });
        }
    });
}

// =============================================================================
// 4. ROW 3 — JUMP BACK IN
// =============================================================================
#[allow(clippy::too_many_arguments)]
fn render_row3_jump_back_in(
    ui: &mut Ui,
    tracks: &[Track],
    recent_history: &[Track],
    theme: &Theme,
    art_manager: &mut AlbumArtManager,
    player: &mut PlayerController,
    on_play_track: &mut Option<Track>,
    on_navigate: &mut Option<NavTab>,
) {
    ui.horizontal(|ui| {
        ui.label(
            RichText::new("JUMP BACK IN")
                .size(13.0)
                .color(theme.text_secondary)
                .strong(),
        );

        ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
            if dark_pill_button(ui, "Show all", None).clicked() {
                *on_navigate = Some(NavTab::Albums);
            }
        });
    });

    ui.add_space(10.0);

    struct JumpAlbum {
        album: String,
        artist: String,
        sample_track: Track,
        tracks: Vec<Track>,
    }

    let mut seen_albums = HashSet::new();
    let mut jump_albums: Vec<JumpAlbum> = Vec::new();

    // Prioritize albums from recent listening history, then fallback to library
    for t in recent_history.iter().chain(tracks.iter()) {
        let a_name = if t.album.trim().is_empty() {
            "Unknown Album".to_string()
        } else {
            t.album.trim().to_string()
        };
        let key = (a_name.clone(), t.display_artist().to_string());
        if !seen_albums.contains(&key) {
            seen_albums.insert(key.clone());
            let album_tracks: Vec<Track> = tracks
                .iter()
                .filter(|tr| {
                    let tr_album = if tr.album.trim().is_empty() {
                        "Unknown Album"
                    } else {
                        tr.album.trim()
                    };
                    tr_album == key.0 && tr.display_artist() == key.1
                })
                .cloned()
                .collect();

            jump_albums.push(JumpAlbum {
                album: a_name,
                artist: key.1,
                sample_track: t.clone(),
                tracks: if album_tracks.is_empty() {
                    vec![t.clone()]
                } else {
                    album_tracks
                },
            });

            if jump_albums.len() >= 10 {
                break;
            }
        }
    }

    if jump_albums.is_empty() {
        let mut empty_frame = theme.card_frame(10.0);
        empty_frame.fill = Color32::from_rgb(22, 22, 24);
        empty_frame.inner_margin = egui::Margin::symmetric(20.0, 16.0);
        empty_frame.show(ui, |ui| {
            ui.horizontal(|ui| {
                let (icon_box, _) = ui.allocate_exact_size(vec2(22.0, 22.0), Sense::hover());
                paint_icon(ui.painter(), Icon::Disc, icon_box, theme.text_muted);
                ui.add_space(8.0);
                ui.label(
                    RichText::new("No albums to resume yet. Add songs to your library to populate Jump Back In.")
                        .size(12.5)
                        .color(theme.text_muted),
                );
            });
        });
    } else {
        ScrollArea::horizontal()
            .id_salt("jump_back_in_horizontal_shelf")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing = vec2(14.0, 0.0);

                ui.horizontal(|ui| {
                    for album in &jump_albums {
                        let card_w = 140.0_f32;
                        ui.allocate_ui_with_layout(
                            vec2(card_w, card_w + 50.0),
                            Layout::top_down(egui::Align::Min),
                            |ui| {
                                // 140x140px square Album Cover
                                let (art_box, art_resp) =
                                    ui.allocate_exact_size(vec2(card_w, card_w), Sense::click());
                                if art_resp.hovered() {
                                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                                }

                                let mut painted = false;
                                if let Some(tex) = art_manager.get_or_load(ui.ctx(), &album.sample_track.path) {
                                    ui.painter().image(
                                        tex.id(),
                                        art_box,
                                        Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                                        Color32::WHITE,
                                    );
                                    let border_color = if art_resp.hovered() {
                                        theme.border_accent
                                    } else {
                                        Color32::from_rgba_premultiplied(255, 255, 255, 22)
                                    };
                                    ui.painter().rect_stroke(
                                        art_box,
                                        Rounding::same(10.0),
                                        Stroke::new(1.0_f32, border_color),
                                    );
                                    painted = true;
                                }

                                if !painted {
                                    ui.painter().rect_filled(
                                        art_box,
                                        Rounding::same(10.0),
                                        Color32::from_rgb(26, 26, 30),
                                    );
                                    let border_color = if art_resp.hovered() {
                                        theme.border_accent
                                    } else {
                                        Color32::from_rgba_premultiplied(255, 255, 255, 18)
                                    };
                                    ui.painter().rect_stroke(
                                        art_box,
                                        Rounding::same(10.0),
                                        Stroke::new(1.0_f32, border_color),
                                    );
                                    let icon_box =
                                        Rect::from_center_size(art_box.center(), vec2(40.0, 40.0));
                                    paint_icon(ui.painter(), Icon::Disc, icon_box, theme.accent);
                                }

                                // Subtle play button overlay on hover
                                if art_resp.hovered() {
                                    let overlay_center =
                                        pos2(art_box.max.x - 24.0, art_box.max.y - 24.0);
                                    ui.painter().circle_filled(
                                        overlay_center,
                                        16.0,
                                        Color32::from_rgba_premultiplied(14, 14, 18, 220),
                                    );
                                    ui.painter().circle_stroke(
                                        overlay_center,
                                        16.0,
                                        Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(255, 255, 255, 40)),
                                    );
                                    let play_icon_rect = Rect::from_center_size(
                                        pos2(overlay_center.x + 1.0, overlay_center.y),
                                        vec2(12.0, 12.0),
                                    );
                                    paint_icon(ui.painter(), Icon::Play, play_icon_rect, Color32::WHITE);
                                }

                                ui.add_space(6.0);

                                // Title below
                                let title_str = truncate_str(&album.album, 18);
                                let title_lbl = ui.add(
                                    egui::Label::new(
                                        RichText::new(title_str)
                                            .size(13.0)
                                            .color(if art_resp.hovered() {
                                                Color32::WHITE
                                            } else {
                                                Color32::from_rgb(244, 244, 245)
                                            })
                                            .strong(),
                                    )
                                    .sense(Sense::click()),
                                );

                                // Subtitle below
                                let sub_str = truncate_str(
                                    &format!("{} · {} songs", album.artist, album.tracks.len()),
                                    22,
                                );
                                ui.label(
                                    RichText::new(sub_str)
                                        .size(11.5)
                                        .color(Color32::from_rgb(113, 113, 122)),
                                );

                                if art_resp.clicked() || title_lbl.clicked() {
                                    player.load_queue(album.tracks.clone(), 0);
                                    *on_play_track = Some(album.sample_track.clone());
                                }
                            },
                        );
                    }
                });
            });
    }
}

fn truncate_str(s: &str, max_chars: usize) -> String {
    if s.chars().count() > max_chars {
        let truncated: String = s.chars().take(max_chars.saturating_sub(1)).collect();
        format!("{truncated}...")
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppConfig;

    #[test]
    fn test_home_view_rendering() {
        let ctx = egui::Context::default();
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let theme = Theme::from_name("Eclipse", None);
                let cfg = AppConfig::default();
                let mut player = PlayerController::new(&cfg);
                let mut art_manager = AlbumArtManager::new();
                let mut on_play = None;
                let mut on_scan = false;
                let mut on_navigate = None;

                let tracks = vec![Track {
                    id: 1,
                    path: "song.flac".into(),
                    title: "Home Track".into(),
                    artist: "Home Artist".into(),
                    album: "Home Album".into(),
                    genre: "Ambient".into(),
                    year: 2024,
                    track_number: 1,
                    duration_secs: 240.0,
                    format: "FLAC".into(),
                    bitrate_kbps: 1411,
                    sample_rate: 48000,
                    channels: 2,
                    favorite: true,
                    has_art: false,
                    folder: "".into(),
                    album_artist: "".into(),
                    play_count: 12,
                    file_size: 4096,
                }];

                render_home(
                    ui,
                    &tracks,
                    &tracks,
                    &mut player,
                    &theme,
                    &mut on_play,
                    &mut art_manager,
                    &mut on_scan,
                    &mut on_navigate,
                );
            });
        });
    }

    #[test]
    fn test_date_and_greeting_computation() {
        let (date_str, greeting) = get_local_date_and_greeting();
        assert!(!date_str.is_empty());
        assert!(!greeting.is_empty());
        assert!(
            greeting == "Good morning."
                || greeting == "Good afternoon."
                || greeting == "Good evening."
        );
    }
}
