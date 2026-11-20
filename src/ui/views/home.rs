use crate::audio::PlayerController;
use crate::library::model::Track;
use crate::ui::art::AlbumArtManager;
use crate::ui::icons::{paint_icon, Icon};
use crate::ui::sidebar::NavTab;
use crate::ui::theme::Theme;
use egui::{pos2, vec2, Color32, Layout, Rect, RichText, Rounding, ScrollArea, Sense, Stroke, Ui};
use std::collections::HashSet;

// Feishin Matte-Slate Palette Tokens
const BG_CENTER: Color32 = Color32::from_rgb(29, 33, 41);
const BG_CARD: Color32 = Color32::from_rgb(37, 42, 52);
const BG_CARD_HOVER: Color32 = Color32::from_rgb(47, 53, 66);
const SEAM_DARK: Color32 = Color32::from_rgb(38, 43, 53);
const ACCENT_PRIMARY: Color32 = Color32::from_rgb(156, 108, 255);
#[allow(dead_code)]
const ACCENT_SOFT: Color32 = Color32::from_rgba_premultiplied(59, 39, 102, 34);
const TEXT_PRIMARY: Color32 = Color32::from_rgb(240, 242, 246);
const TEXT_SECONDARY: Color32 = Color32::from_rgb(168, 174, 188);
const TEXT_MUTED: Color32 = Color32::from_rgb(118, 124, 138);
#[allow(dead_code)]
const TEXT_FAINT: Color32 = Color32::from_rgb(86, 92, 105);

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

/// Crisp high-contrast Feishin primary action pill (#F0F2F6 fill + #12151A bold dark text, Stroke::NONE).
fn feishin_primary_pill(ui: &mut Ui, label: &str, icon: Option<Icon>) -> egui::Response {
    let font_id = egui::FontId::proportional(12.5);
    let text_color = Color32::from_rgb(18, 21, 26); // #12151A
    let galley = ui.painter().layout_no_wrap(label.to_string(), font_id, text_color);
    let icon_w = if icon.is_some() { 18.0 } else { 0.0 };
    let padding_x = 14.0;
    let height = 30.0;
    let width = galley.rect.width() + icon_w + (padding_x * 2.0);

    let (rect, resp) = ui.allocate_exact_size(vec2(width, height), Sense::click());
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }

    let bg_color = if resp.is_pointer_button_down_on() {
        Color32::from_rgb(220, 222, 226)
    } else if resp.hovered() {
        Color32::WHITE
    } else {
        Color32::from_rgb(240, 242, 246) // #F0F2F6
    };

    ui.painter().rect_filled(rect, Rounding::same(15.0), bg_color);

    let mut cur_x = rect.min.x + padding_x;
    let center_y = rect.center().y;

    if let Some(ic) = icon {
        let icon_rect = Rect::from_center_size(pos2(cur_x + 7.0, center_y), vec2(14.0, 14.0));
        paint_icon(ui.painter(), ic, icon_rect, text_color);
        cur_x += icon_w;
    }

    let text_pos = pos2(cur_x, center_y - galley.rect.height() / 2.0);
    ui.painter().galley(text_pos, galley, text_color);

    resp
}

/// Borderless BG_CARD_HOVER pill with TEXT_PRIMARY text.
fn feishin_secondary_pill(ui: &mut Ui, label: &str, icon: Option<Icon>) -> egui::Response {
    let font_id = egui::FontId::proportional(12.5);
    let text_color = TEXT_PRIMARY;
    let galley = ui.painter().layout_no_wrap(label.to_string(), font_id, text_color);
    let icon_w = if icon.is_some() { 18.0 } else { 0.0 };
    let padding_x = 14.0;
    let height = 30.0;
    let width = galley.rect.width() + icon_w + (padding_x * 2.0);

    let (rect, resp) = ui.allocate_exact_size(vec2(width, height), Sense::click());
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }

    let bg_color = if resp.is_pointer_button_down_on() {
        Color32::from_rgb(55, 62, 78)
    } else if resp.hovered() {
        Color32::from_rgb(58, 66, 82)
    } else {
        BG_CARD_HOVER // Color32::from_rgb(47, 53, 66)
    };

    ui.painter().rect_filled(rect, Rounding::same(15.0), bg_color);

    let mut cur_x = rect.min.x + padding_x;
    let center_y = rect.center().y;

    if let Some(ic) = icon {
        let icon_rect = Rect::from_center_size(pos2(cur_x + 7.0, center_y), vec2(14.0, 14.0));
        paint_icon(ui.painter(), ic, icon_rect, text_color);
        cur_x += icon_w;
    }

    let text_pos = pos2(cur_x, center_y - galley.rect.height() / 2.0);
    ui.painter().galley(text_pos, galley, text_color);

    resp
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
        .id_salt("home_view_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            // Atmospheric Feishin Top Glow in Center Header:
            // Soft vertical gradient fading from faint moody purple/slate wash down to transparent BG_CENTER over top 220px
            let glow_rect = Rect::from_min_size(ui.cursor().min, vec2(ui.available_width(), 220.0));
            let mut glow_mesh = egui::Mesh::default();
            glow_mesh.add_triangle(0, 1, 2);
            glow_mesh.add_triangle(0, 2, 3);
            let top_glow = Color32::from_rgba_unmultiplied(110, 88, 170, 32);
            let bot_glow = Color32::from_rgba_unmultiplied(BG_CENTER.r(), BG_CENTER.g(), BG_CENTER.b(), 0);
            glow_mesh.vertices.push(egui::epaint::Vertex {
                pos: glow_rect.left_top(),
                uv: egui::epaint::WHITE_UV,
                color: top_glow,
            });
            glow_mesh.vertices.push(egui::epaint::Vertex {
                pos: glow_rect.right_top(),
                uv: egui::epaint::WHITE_UV,
                color: top_glow,
            });
            glow_mesh.vertices.push(egui::epaint::Vertex {
                pos: glow_rect.right_bottom(),
                uv: egui::epaint::WHITE_UV,
                color: bot_glow,
            });
            glow_mesh.vertices.push(egui::epaint::Vertex {
                pos: glow_rect.left_bottom(),
                uv: egui::epaint::WHITE_UV,
                color: bot_glow,
            });
            ui.painter().add(egui::Shape::mesh(glow_mesh));

            ui.spacing_mut().item_spacing = vec2(0.0, 16.0);

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
fn render_greeting_header(ui: &mut Ui, _theme: &Theme) {
    let (date_str, greeting) = get_local_date_and_greeting();
    let total_w = ui.available_width();
    let header_h = 52.0_f32;

    ui.allocate_ui_with_layout(
        vec2(total_w, header_h),
        Layout::left_to_right(egui::Align::Center),
        |ui| {
            // Left Group: Date in uppercase muted tracking stacked directly above greeting
            ui.allocate_ui_with_layout(
                vec2((total_w - 240.0).max(180.0), header_h),
                Layout::top_down(egui::Align::Min),
                |ui| {
                    ui.spacing_mut().item_spacing = vec2(0.0, 2.0);
                    ui.label(
                        RichText::new(&date_str)
                            .size(11.5)
                            .color(TEXT_MUTED)
                            .strong(),
                    );
                    ui.label(
                        RichText::new(&greeting)
                            .size(32.0)
                            .color(TEXT_PRIMARY)
                            .strong(),
                    );
                },
            );

            // Right Group: Right-aligned text column placed left of circular 44x44 equalizer badge
            ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                ui.spacing_mut().item_spacing = vec2(14.0, 0.0);

                // Circular waveform/equalizer badge on far right (borderless)
                let (badge_rect, badge_resp) = ui.allocate_exact_size(vec2(44.0, 44.0), Sense::hover());
                let badge_bg = if badge_resp.hovered() {
                    BG_CARD_HOVER
                } else {
                    BG_CARD
                };
                ui.painter().circle_filled(badge_rect.center(), 22.0, badge_bg);
                let icon_rect = Rect::from_center_size(badge_rect.center(), vec2(20.0, 20.0));
                paint_icon(ui.painter(), Icon::BarChart, icon_rect, ACCENT_PRIMARY);

                // Right-aligned text column placed left of the circular badge
                ui.allocate_ui_with_layout(
                    vec2(170.0, 44.0),
                    Layout::top_down(egui::Align::Max),
                    |ui| {
                        ui.spacing_mut().item_spacing = vec2(0.0, 2.0);
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing = vec2(6.0, 0.0);
                            ui.label(
                                RichText::new("E C L I P S E D")
                                    .size(13.5)
                                    .color(TEXT_PRIMARY)
                                    .strong(),
                            );
                            ui.label(
                                RichText::new(concat!("v", env!("CARGO_PKG_VERSION")))
                                    .size(10.5)
                                    .color(TEXT_MUTED),
                            );
                        });
                        ui.label(
                            RichText::new("music, in the dark.")
                                .size(11.0)
                                .color(TEXT_MUTED),
                        );
                    },
                );
            });
        },
    );

    ui.add_space(20.0);
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
    let gap = 16.0_f32;
    let is_wide = available_w >= 640.0;
    let row1_h = 184.0_f32;

    // Asymmetric 1.35fr : 1fr split
    let left_w = if is_wide {
        ((available_w - gap) * (1.35 / 2.35)).max(340.0)
    } else {
        available_w
    };
    let right_w = if is_wide {
        available_w - gap - left_w
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

            // Left Card (1.35fr width): PICK UP WHERE YOU LEFT OFF
            ui.allocate_ui_with_layout(
                vec2(left_w, row1_h),
                Layout::top_down(egui::Align::Min),
                |ui| {
                    render_pick_up_card(
                        ui,
                        left_w,
                        row1_h,
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

            // Right Nested 2x2 Bento Grid (1fr width)
            ui.allocate_ui_with_layout(
                vec2(right_w, row1_h),
                Layout::top_down(egui::Align::Min),
                |ui| {
                    render_stats_2x2_bento(ui, right_w, row1_h, tracks, theme, player, on_navigate);
                },
            );
        });
    } else {
        render_pick_up_card(
            ui,
            left_w,
            row1_h,
            active_track.as_ref(),
            player,
            theme,
            art_manager,
            on_play_track,
            on_navigate,
            on_trigger_scan,
        );
        ui.add_space(gap);
        render_stats_2x2_bento(ui, right_w, row1_h, tracks, theme, player, on_navigate);
    }
}

#[allow(clippy::too_many_arguments)]
fn render_pick_up_card(
    ui: &mut Ui,
    card_w: f32,
    card_h: f32,
    active_track: Option<&Track>,
    player: &mut PlayerController,
    _theme: &Theme,
    art_manager: &mut AlbumArtManager,
    on_play_track: &mut Option<Track>,
    on_navigate: &mut Option<NavTab>,
    on_trigger_scan: &mut bool,
) {
    let mut frame = egui::Frame::none();
    frame.fill = BG_CARD;
    frame.rounding = Rounding::same(10.0);
    frame.stroke = Stroke::NONE;
    frame.inner_margin = egui::Margin::symmetric(16.0, 16.0);

    let frame_response = frame.show(ui, |ui| {
        ui.set_width(card_w - 32.0);
        ui.set_height(card_h - 32.0);

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = vec2(16.0, 0.0);

            // Large 136x136px rounded square Album Cover (border-radius: 8px, border: none)
            let (art_box, art_resp) = ui.allocate_exact_size(vec2(136.0, 136.0), Sense::click());
            if art_resp.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }

            let mut painted = false;
            if let Some(track) = active_track {
                if let Some(tex) = art_manager.get_or_load(ui.ctx(), &track.path) {
                    let img = egui::Image::new(tex).rounding(Rounding::same(8.0));
                    img.paint_at(ui, art_box);
                    painted = true;
                }
            }

            if !painted {
                ui.painter().rect_filled(art_box, Rounding::same(8.0), BG_CARD_HOVER);
                let icon_box = Rect::from_center_size(art_box.center(), vec2(48.0, 48.0));
                paint_icon(ui.painter(), Icon::Disc, icon_box, TEXT_MUTED);
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
                            .color(TEXT_MUTED)
                            .strong(),
                    );

                    if let Some(track) = active_track {
                        // Large bold track title: 18px bold TEXT_PRIMARY
                        let char_limit = (right_content_w / 10.5).max(10.0) as usize;
                        let title_str = truncate_str(track.display_title(), char_limit);
                        ui.label(
                            RichText::new(title_str)
                                .size(18.0)
                                .color(TEXT_PRIMARY)
                                .strong(),
                        );

                        // Muted artist name: 13px TEXT_SECONDARY
                        let artist_str = truncate_str(track.display_artist(), char_limit + 4);
                        ui.label(
                            RichText::new(artist_str)
                                .size(13.0)
                                .color(TEXT_SECONDARY),
                        );

                        // Single-line technical metadata: track.tech_summary_str() using 11.5px TEXT_MUTED
                        ui.label(
                            RichText::new(track.tech_summary_str())
                                .size(11.5)
                                .color(TEXT_MUTED),
                        );

                        ui.add_space(8.0);

                        // Action row: crisp high-contrast Feishin primary pill and borderless secondary pill
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing = vec2(8.0, 0.0);

                            let is_current_playing = player.engine.is_playing()
                                && player.current_track().is_some_and(|t| t.id == track.id);

                            let (btn_text, btn_icon) = if is_current_playing {
                                ("Pause", Icon::Pause)
                            } else {
                                ("Play", Icon::Play)
                            };

                            if feishin_primary_pill(ui, btn_text, Some(btn_icon)).clicked() {
                                if is_current_playing {
                                    player.engine.pause();
                                } else if player.current_track().is_some_and(|t| t.id == track.id) {
                                    player.engine.resume();
                                } else {
                                    *on_play_track = Some(track.clone());
                                }
                            }

                            if feishin_secondary_pill(ui, "Open", None).clicked() {
                                *on_navigate = Some(NavTab::NowPlaying);
                            }
                        });
                    } else {
                        ui.add_space(10.0);
                        ui.label(
                            RichText::new("Your library is empty")
                                .size(16.0)
                                .color(TEXT_PRIMARY)
                                .strong(),
                        );
                        ui.label(
                            RichText::new("Scan a local music folder to get started.")
                                .size(12.0)
                                .color(TEXT_MUTED),
                        );
                        ui.add_space(12.0);
                        if feishin_primary_pill(ui, "Scan Folder", Some(Icon::Refresh)).clicked() {
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
            SEAM_DARK,
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
                ui.painter().rect_filled(filled_rect, Rounding::same(1.5), ACCENT_PRIMARY);
            }
        }
    });

    let _ = frame_response;
}

fn render_stats_2x2_bento(
    ui: &mut Ui,
    right_w: f32,
    row1_h: f32,
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

    let card_gap = 12.0_f32;
    let sub_w = ((right_w - card_gap) / 2.0).max(100.0);
    let sub_h = (row1_h - card_gap) / 2.0;

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
                            .color(TEXT_MUTED)
                            .strong(),
                    );
                    ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                        let (icon_rect, _) = ui.allocate_exact_size(vec2(14.0, 14.0), Sense::hover());
                        paint_icon(ui.painter(), Icon::Music, icon_rect, TEXT_MUTED);
                    });
                });
                ui.add_space(4.0);
                ui.label(
                    RichText::new(format!("{} tracks", total_tracks))
                        .size(16.0)
                        .color(TEXT_PRIMARY)
                        .strong(),
                );
                ui.label(
                    RichText::new(format!("{} albums · {} artists", total_albums, total_artists))
                        .size(11.0)
                        .color(TEXT_MUTED),
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
                            .color(TEXT_MUTED)
                            .strong(),
                    );
                    ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                        let (icon_rect, _) = ui.allocate_exact_size(vec2(14.0, 14.0), Sense::hover());
                        paint_icon(ui.painter(), Icon::Clock, icon_rect, TEXT_MUTED);
                    });
                });
                ui.add_space(4.0);
                ui.label(
                    RichText::new(format!("{:.1} h", total_hours))
                        .size(16.0)
                        .color(TEXT_PRIMARY)
                        .strong(),
                );
                ui.label(
                    RichText::new(format!("{:.0}% lossless · {} plays", lossless_pct, total_plays))
                        .size(11.0)
                        .color(TEXT_MUTED),
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
            ACCENT_PRIMARY,
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
    _theme: &Theme,
    content: F,
    on_click: C,
) where
    F: FnOnce(&mut Ui),
    C: FnOnce(),
{
    let (rect, resp) = ui.allocate_exact_size(vec2(w, h), Sense::click());
    let is_hovered = resp.hovered();
    if is_hovered {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }

    let bg_fill = if is_hovered { BG_CARD_HOVER } else { BG_CARD };
    ui.painter().rect_filled(rect, Rounding::same(10.0), bg_fill);

    let inner_rect = rect.shrink2(vec2(14.0, 10.0));
    let mut child_ui = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(inner_rect)
            .layout(Layout::top_down(egui::Align::Min)),
    );
    content(&mut child_ui);

    if resp.clicked() {
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
    _theme: &Theme,
    on_play: P,
    on_click_card: C,
) where
    P: FnOnce(),
    C: FnOnce(),
{
    let (rect, resp) = ui.allocate_exact_size(vec2(w, h), Sense::click());
    let is_hovered = resp.hovered();
    if is_hovered {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }

    let bg_fill = if is_hovered { BG_CARD_HOVER } else { BG_CARD };
    ui.painter().rect_filled(rect, Rounding::same(10.0), bg_fill);

    let inner_rect = rect.shrink2(vec2(14.0, 10.0));
    let mut child_ui = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(inner_rect)
            .layout(Layout::left_to_right(egui::Align::Center)),
    );

    let mut play_clicked = false;

    // Left text stack
    let left_w = (inner_rect.width() - 32.0).max(60.0);
    child_ui.allocate_ui_with_layout(
        vec2(left_w, inner_rect.height()),
        Layout::top_down(egui::Align::Min),
        |ui| {
            let (icon_box, _) = ui.allocate_exact_size(vec2(14.0, 14.0), Sense::hover());
            paint_icon(ui.painter(), icon, icon_box, icon_color);

            ui.add_space(3.0);
            ui.label(
                RichText::new(title)
                    .size(13.5)
                    .color(TEXT_PRIMARY)
                    .strong(),
            );
            ui.label(
                RichText::new(subtitle)
                    .size(10.0)
                    .color(TEXT_MUTED)
                    .strong(),
            );
        },
    );

    // Bottom-right circular button with Icon::Play (Stroke::NONE)
    child_ui.with_layout(Layout::right_to_left(egui::Align::BOTTOM), |ui| {
        let btn_size = 28.0_f32;
        let (btn_rect, btn_resp) = ui.allocate_exact_size(vec2(btn_size, btn_size), Sense::click());
        if btn_resp.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
        let bg_color = if btn_resp.is_pointer_button_down_on() {
            Color32::from_rgb(30, 34, 43)
        } else if btn_resp.hovered() {
            Color32::from_rgb(55, 62, 78)
        } else {
            Color32::from_rgb(42, 48, 60)
        };
        ui.painter().circle_filled(btn_rect.center(), btn_size / 2.0, bg_color);
        let play_icon_rect = Rect::from_center_size(
            pos2(btn_rect.center().x + 1.0, btn_rect.center().y),
            vec2(12.0, 12.0),
        );
        paint_icon(ui.painter(), Icon::Play, play_icon_rect, TEXT_PRIMARY);

        if btn_resp.clicked() {
            play_clicked = true;
        }
    });

    if play_clicked {
        on_play();
    } else if resp.clicked() {
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
    let gap = 16.0_f32;
    let is_wide = available_w >= 640.0;
    let row2_h = 212.0_f32;
    let col_w = if is_wide {
        ((available_w - gap) / 2.0).max(280.0)
    } else {
        available_w
    };

    if is_wide {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = vec2(gap, 0.0);

            // Left Card: ON REPEAT (equal 50/50 width, shared row2_h)
            ui.allocate_ui_with_layout(
                vec2(col_w, row2_h),
                Layout::top_down(egui::Align::Min),
                |ui| {
                    render_on_repeat_card(ui, col_w, row2_h, tracks, theme, art_manager, on_play_track);
                },
            );

            // Right Card: RECENTLY ADDED (equal 50/50 width, shared row2_h)
            ui.allocate_ui_with_layout(
                vec2(col_w, row2_h),
                Layout::top_down(egui::Align::Min),
                |ui| {
                    render_recently_added_card(ui, col_w, row2_h, tracks, theme, art_manager, on_play_track, on_navigate);
                },
            );
        });
    } else {
        render_on_repeat_card(ui, col_w, row2_h, tracks, theme, art_manager, on_play_track);
        ui.add_space(gap);
        render_recently_added_card(ui, col_w, row2_h, tracks, theme, art_manager, on_play_track, on_navigate);
    }
}

fn render_on_repeat_card(
    ui: &mut Ui,
    col_w: f32,
    card_h: f32,
    tracks: &[Track],
    _theme: &Theme,
    art_manager: &mut AlbumArtManager,
    on_play_track: &mut Option<Track>,
) {
    let mut frame = egui::Frame::none();
    frame.fill = BG_CARD;
    frame.rounding = Rounding::same(10.0);
    frame.stroke = Stroke::NONE;
    frame.inner_margin = egui::Margin::symmetric(16.0, 14.0);

    frame.show(ui, |ui| {
        ui.set_width(col_w - 32.0);
        ui.set_height(card_h - 28.0);

        // Card header
        ui.label(
            RichText::new("ON REPEAT")
                .size(11.0)
                .color(TEXT_MUTED)
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
                    .color(TEXT_MUTED),
            );
        } else {
            for (idx, track) in top_4.iter().enumerate() {
                let row_h = 36.0_f32;
                let (row_rect, row_resp) = ui.allocate_exact_size(vec2(ui.available_width(), row_h), Sense::click());

                if row_resp.hovered() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    ui.painter().rect_filled(row_rect, Rounding::same(6.0), BG_CARD_HOVER);
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
                    TEXT_MUTED,
                );

                // 34x34px square Album Cover (border: none)
                let art_rect = Rect::from_min_size(
                    pos2(row_rect.min.x + 32.0, row_rect.center().y - 17.0),
                    vec2(34.0, 34.0),
                );

                let mut painted = false;
                if let Some(tex) = art_manager.get_or_load(ui.ctx(), &track.path) {
                    let img = egui::Image::new(tex).rounding(Rounding::same(6.0));
                    img.paint_at(ui, art_rect);
                    painted = true;
                }

                if !painted {
                    ui.painter().rect_filled(art_rect, Rounding::same(6.0), BG_CARD_HOVER);
                    let icon_box = Rect::from_center_size(art_rect.center(), vec2(16.0, 16.0));
                    paint_icon(ui.painter(), Icon::Disc, icon_box, TEXT_MUTED);
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
                    TEXT_MUTED,
                );
                let right_pos = pos2(
                    row_rect.max.x - 8.0 - right_galley.rect.width(),
                    row_rect.center().y - right_galley.rect.height() / 2.0,
                );
                ui.painter().galley(right_pos, right_galley, TEXT_MUTED);

                // Bold track title
                let title_start_x = art_rect.max.x + 10.0;
                let max_title_w = (right_pos.x - 10.0 - title_start_x).max(20.0);
                let char_limit = (max_title_w / 7.5).max(6.0) as usize;
                let title_str = truncate_str(track.display_title(), char_limit);
                let title_galley = ui.painter().layout_no_wrap(
                    title_str,
                    egui::FontId::proportional(12.5),
                    TEXT_PRIMARY,
                );
                ui.painter().galley(
                    pos2(title_start_x, row_rect.center().y - title_galley.rect.height() / 2.0),
                    title_galley,
                    TEXT_PRIMARY,
                );

                ui.add_space(2.0);
            }
        }
    });
}

#[allow(clippy::too_many_arguments)]
fn render_recently_added_card(
    ui: &mut Ui,
    col_w: f32,
    card_h: f32,
    tracks: &[Track],
    _theme: &Theme,
    art_manager: &mut AlbumArtManager,
    on_play_track: &mut Option<Track>,
    on_navigate: &mut Option<NavTab>,
) {
    let mut frame = egui::Frame::none();
    frame.fill = BG_CARD;
    frame.rounding = Rounding::same(10.0);
    frame.stroke = Stroke::NONE;
    frame.inner_margin = egui::Margin::symmetric(16.0, 14.0);

    frame.show(ui, |ui| {
        ui.set_width(col_w - 32.0);
        ui.set_height(card_h - 28.0);

        // Card header
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("RECENTLY ADDED")
                    .size(11.0)
                    .color(TEXT_MUTED)
                    .strong(),
            );
            ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                let see_all = ui.add(
                    egui::Label::new(
                        RichText::new("See all")
                            .size(11.0)
                            .color(TEXT_MUTED),
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
        ui.add_space(12.0);

        let mut sorted_recent = tracks.to_vec();
        sorted_recent.sort_by_key(|t| std::cmp::Reverse(t.id));
        let recent_5: Vec<&Track> = sorted_recent.iter().take(5).collect();

        if recent_5.is_empty() {
            ui.add_space(10.0);
            ui.label(
                RichText::new("No tracks added yet.")
                    .size(12.0)
                    .color(TEXT_MUTED),
            );
        } else {
            let inner_w = col_w - 32.0;
            let item_gap = 10.0_f32;
            let col_item_w = ((inner_w - 4.0 * item_gap) / 5.0).clamp(52.0, 76.0);
            let cover_size = col_item_w;

            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = vec2(item_gap, 0.0);

                for track in recent_5 {
                    ui.allocate_ui_with_layout(
                        vec2(col_item_w, cover_size + 44.0),
                        Layout::top_down(egui::Align::Center),
                        |ui| {
                            // Square Album Cover with border-radius 8px and NO border
                            let (art_box, art_resp) = ui.allocate_exact_size(vec2(cover_size, cover_size), Sense::click());
                            if art_resp.hovered() {
                                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                            }

                            let mut painted = false;
                            if let Some(tex) = art_manager.get_or_load(ui.ctx(), &track.path) {
                                let img = egui::Image::new(tex).rounding(Rounding::same(8.0));
                                img.paint_at(ui, art_box);
                                painted = true;
                            }

                            if !painted {
                                ui.painter().rect_filled(art_box, Rounding::same(8.0), BG_CARD_HOVER);
                                let icon_box = Rect::from_center_size(art_box.center(), vec2(22.0, 22.0));
                                paint_icon(ui.painter(), Icon::Disc, icon_box, TEXT_MUTED);
                            }

                            ui.add_space(6.0);

                            // Bold truncated track title directly underneath: 12px TEXT_PRIMARY
                            let title_chars = (col_item_w / 7.0).max(5.0) as usize;
                            let title_str = truncate_str(track.display_title(), title_chars);
                            let title_lbl = ui.add(
                                egui::Label::new(
                                    RichText::new(title_str)
                                        .size(12.0)
                                        .color(if art_resp.hovered() {
                                            Color32::WHITE
                                        } else {
                                            TEXT_PRIMARY
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
                                    .size(10.0)
                                    .color(TEXT_MUTED),
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
    _theme: &Theme,
    art_manager: &mut AlbumArtManager,
    player: &mut PlayerController,
    on_play_track: &mut Option<Track>,
    on_navigate: &mut Option<NavTab>,
) {
    ui.horizontal(|ui| {
        ui.label(
            RichText::new("JUMP BACK IN")
                .size(13.0)
                .color(TEXT_SECONDARY)
                .strong(),
        );

        ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
            if feishin_secondary_pill(ui, "Show all", None).clicked() {
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
        let mut empty_frame = egui::Frame::none();
        empty_frame.fill = BG_CARD;
        empty_frame.rounding = Rounding::same(10.0);
        empty_frame.stroke = Stroke::NONE;
        empty_frame.inner_margin = egui::Margin::symmetric(20.0, 16.0);
        empty_frame.show(ui, |ui| {
            ui.horizontal(|ui| {
                let (icon_box, _) = ui.allocate_exact_size(vec2(22.0, 22.0), Sense::hover());
                paint_icon(ui.painter(), Icon::Disc, icon_box, TEXT_MUTED);
                ui.add_space(8.0);
                ui.label(
                    RichText::new("No albums to resume yet. Add songs to your library to populate Jump Back In.")
                        .size(12.5)
                        .color(TEXT_MUTED),
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
                                // 140x140px square Album Cover (border-radius: 8px, Stroke::NONE)
                                let (art_box, art_resp) =
                                    ui.allocate_exact_size(vec2(card_w, card_w), Sense::click());
                                if art_resp.hovered() {
                                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                                }

                                let mut painted = false;
                                if let Some(tex) = art_manager.get_or_load(ui.ctx(), &album.sample_track.path) {
                                    let img = egui::Image::new(tex).rounding(Rounding::same(8.0));
                                    img.paint_at(ui, art_box);
                                    painted = true;
                                }

                                if !painted {
                                    ui.painter().rect_filled(
                                        art_box,
                                        Rounding::same(8.0),
                                        BG_CARD,
                                    );
                                    let icon_box =
                                        Rect::from_center_size(art_box.center(), vec2(40.0, 40.0));
                                    paint_icon(ui.painter(), Icon::Disc, icon_box, TEXT_MUTED);
                                }

                                // Subtle play button overlay on hover (Stroke::NONE)
                                if art_resp.hovered() {
                                    let overlay_center =
                                        pos2(art_box.max.x - 24.0, art_box.max.y - 24.0);
                                    ui.painter().circle_filled(
                                        overlay_center,
                                        16.0,
                                        Color32::from_rgba_premultiplied(14, 14, 18, 220),
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
                                                TEXT_PRIMARY
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
                                        .color(TEXT_MUTED),
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
