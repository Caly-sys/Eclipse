//! Bento Player Dock for Eclipse.
//! Provides a docked bottom transport bar with Bento-grid aesthetic:
//! - 74px - 76px fixed dock height
//! - 3 strictly centered horizontal zones
//! - Level Transport Rule: Prev, Play, Next, Shuffle, Repeat stay dead-level
//! - Album Art Clipping Rule: 46x46 with Rounding::same(8.0)
//! - Interactive scrubbable seek rail with smooth dragging
//! - Zero panics, 100% vector icons, zero Unicode emojis.

use crate::audio::PlayerController;
use crate::config::RepeatMode;
use crate::library::model::format_time_secs;
use crate::streams::controller::StreamController;
use crate::ui::art::AlbumArtManager;
use crate::ui::bento_theme::{quality_badge_pill, BentoPalette};
use crate::ui::icons::{paint_icon, Icon};
use crate::ui::playerbar::{current_playback_time, seek_to, SeekbarDragState};
use crate::ui::theme::Theme;
use egui::{pos2, vec2, Color32, FontId, Layout, Rect, RichText, Rounding, Stroke, Ui};
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BentoPlayerDockAction {
    #[default]
    None,
    ToggleFavorite(i64),
    ToggleRightInspector,
    JumpToLyrics,
    OpenPopoutPip,
    OpenFullscreen,
}

/// Renders the fixed-height Bento player dock at the bottom of the Eclipse viewport.
pub fn render_bento_player_dock(
    ui: &mut Ui,
    player: &mut PlayerController,
    stream_controller: Option<&StreamController>,
    art_manager: &mut AlbumArtManager,
    theme: &Theme,
    is_inspector_open: bool,
) -> BentoPlayerDockAction {
    let palette = theme.bento_palette();
    let mut action = BentoPlayerDockAction::None;

    let dock_h = 75.0;
    let full_w = ui.available_width();
    ui.set_width(full_w);
    ui.set_min_width(full_w);

    let dock_frame = egui::Frame::none()
        .fill(palette.bg_bar)
        .rounding(Rounding::ZERO)
        .stroke(Stroke::NONE)
        .inner_margin(egui::Margin::symmetric(16.0, 5.0));

    let frame_resp = dock_frame.show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.set_height(dock_h - 10.0);

        let total_w = ui.available_width();

        // Right zone (right_w): ALWAYS reserve enough fixed width so all utility icons
        // (Queue/Lyrics, Mini-Player, Fullscreen, Inspector toggle) + Volume icon NEVER clip off the right edge of the window!
        // - If total_w >= 960.0: full volume slider (~80px) + all utility icons.
        // - If total_w < 960.0: compact volume slider (~44px) + all utility icons (or hide just the slider track below 820.0px, keeping the mute/volume icon + all utility icons 100% visible).
        let (right_w, slider_w) = if total_w >= 960.0 {
            (252.0_f32, Some(80.0_f32))
        } else if total_w >= 820.0 {
            (216.0_f32, Some(44.0_f32))
        } else {
            (168.0_f32, None)
        };

        // Left zone (left_w): clamp strictly to (total_w * 0.27).clamp(170.0, 290.0)
        let left_w = (total_w * 0.27).clamp(170.0, 290.0);

        // Center zone: center_w = (total_w - left_w - right_w - 24.0).max(200.0)
        let center_w = (total_w - left_w - right_w - 24.0).max(200.0);

        ui.spacing_mut().item_spacing = vec2(12.0, 0.0);

        ui.horizontal(|ui| {
            // Zone 1: Left - Album Art, Title, Artist, Badges, Heart (clipped child UI)
            ui.allocate_ui_with_layout(
                vec2(left_w, dock_h - 10.0),
                Layout::left_to_right(egui::Align::Center),
                |ui| {
                    ui.set_clip_rect(ui.max_rect());
                    render_zone1_metadata(ui, player, art_manager, &palette, left_w, total_w, &mut action);
                },
            );

            // Zone 2: Center - Transport controls & scrubbable seekbar
            ui.allocate_ui_with_layout(
                vec2(center_w, dock_h - 10.0),
                Layout::top_down(egui::Align::Center),
                |ui| {
                    ui.set_clip_rect(ui.max_rect());
                    render_zone2_transport(ui, player, stream_controller, &palette, center_w);
                },
            );

            // Zone 3: Right - Volume slider and view mode toggle buttons
            // Centered on the exact same vertical centerline as transport controls (controls_h = 36.0)
            ui.allocate_ui_with_layout(
                vec2(right_w, dock_h - 10.0),
                Layout::top_down(egui::Align::Min),
                |ui| {
                    ui.set_clip_rect(ui.max_rect());
                    ui.allocate_ui_with_layout(
                        vec2(right_w, 36.0),
                        Layout::right_to_left(egui::Align::Center),
                        |ui| {
                            render_zone3_controls(ui, player, stream_controller, &palette, is_inspector_open, slider_w, &mut action);
                        },
                    );
                },
            );
        });
    });

    // 1.0px seam top divider line across the top edge
    let top_line_rect = Rect::from_min_max(
        pos2(frame_resp.response.rect.min.x, frame_resp.response.rect.min.y),
        pos2(frame_resp.response.rect.max.x, frame_resp.response.rect.min.y + 1.0),
    );
    ui.painter().rect_filled(top_line_rect, Rounding::ZERO, palette.seam);

    action
}

// =============================================================================
// Zone 1: Metadata (46x46 Art, Title, Artist, Format Badge, Favorite Heart)
// =============================================================================

fn render_zone1_metadata(
    ui: &mut Ui,
    player: &PlayerController,
    art_manager: &mut AlbumArtManager,
    palette: &BentoPalette,
    left_w: f32,
    total_w: f32,
    action: &mut BentoPlayerDockAction,
) {
    let is_stream = player.playback_source.is_stream();
    let show_badge = total_w >= 900.0;

    if is_stream {
        if let Some(ref track) = player.active_stream_track {
            let prov_id = player.stream_provider_id().unwrap_or("spotify");
            let (prov_icon, prov_color) = match prov_id {
                "spotify" => (Icon::Spotify, Color32::from_rgb(30, 215, 96)),
                "youtube" => (Icon::YouTube, Color32::from_rgb(255, 60, 60)),
                _ => (Icon::Disc, palette.accent),
            };

            // 46x46 Album Art / Stream Badge with Rounding::same(8.0) (borderless)
            let (art_rect, _) = ui.allocate_exact_size(vec2(46.0, 46.0), egui::Sense::hover());
            ui.painter().rect_filled(art_rect, Rounding::same(8.0), palette.bg_card);
            let icon_rect = Rect::from_center_size(art_rect.center(), vec2(20.0, 20.0));
            paint_icon(ui.painter(), prov_icon, icon_rect, prov_color);

            ui.add_space(8.0);

            // Title and Artist column
            let text_w = (left_w - 46.0 - 16.0).max(60.0);
            render_title_artist_column(ui, track.display_title(), track.display_artist(), "STREAM", false, palette, text_w, show_badge);
        } else {
            render_empty_track_zone(ui, palette);
        }
    } else if let Some(track) = player.current_track() {
        let track_id = track.id;
        let is_fav = track.favorite;

        // 46x46 Rounded Album Art Thumbnail
        // Clean 8.0px rounded egui::Image with Stroke::NONE
        let (art_rect, _) = ui.allocate_exact_size(vec2(46.0, 46.0), egui::Sense::hover());
        let mut painted_cover = false;

        if let Some(tex) = art_manager.get_or_load(ui.ctx(), &track.path) {
            let img = egui::Image::new(tex).rounding(Rounding::same(8.0));
            img.paint_at(ui, art_rect);
            painted_cover = true;
        }

        if !painted_cover {
            ui.painter().rect_filled(art_rect, Rounding::same(8.0), palette.bg_card);
            let icon_rect = Rect::from_center_size(art_rect.center(), vec2(20.0, 20.0));
            paint_icon(ui.painter(), Icon::Disc, icon_rect, palette.text_muted);
        }

        ui.add_space(8.0);

        // Compute format badge text (e.g. FLAC 24/48, MP3 320k)
        let (format_badge_text, is_hires) = if track.is_lossless() {
            let is_hires = track.sample_rate > 48000 || track.bitrate_kbps > 2000;
            if track.sample_rate > 0 {
                let khz = track.sample_rate / 1000;
                let bit_depth = if is_hires { 24 } else { 16 };
                (format!("FLAC {}/{}", bit_depth, khz), is_hires)
            } else {
                ("FLAC 24/48".to_string(), true)
            }
        } else {
            let fmt = track.format.to_uppercase();
            if fmt.is_empty() {
                ("MP3 320k".to_string(), false)
            } else {
                (format!("{} 320k", fmt), false)
            }
        };

        // Text width accounts for Art (46px) + Spacing (8px) + Heart button (26px) + Spacing (4px) + margin
        let text_w = (left_w - 46.0 - 8.0 - 26.0 - 12.0).max(50.0);
        render_title_artist_column(ui, track.display_title(), track.display_artist(), &format_badge_text, is_hires, palette, text_w, show_badge);

        ui.add_space(4.0);

        // Heart favorite toggle button
        let (heart_rect, heart_resp) = ui.allocate_exact_size(vec2(26.0, 26.0), egui::Sense::click());
        let is_hovered = heart_resp.hovered();

        if is_hovered {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            ui.painter().rect_filled(heart_rect, Rounding::same(6.0), palette.bg_card_hover);
        }

        let heart_icon = if is_fav { Icon::Heart } else { Icon::HeartOutline };
        let heart_color = if is_fav { palette.accent } else { palette.text_muted };
        let icon_box = Rect::from_center_size(heart_rect.center(), vec2(15.0, 15.0));
        paint_icon(ui.painter(), heart_icon, icon_box, heart_color);

        let heart_resp = heart_resp.on_hover_text(if is_fav { "Remove from Favorites" } else { "Add to Favorites" });
        if heart_resp.clicked() {
            *action = BentoPlayerDockAction::ToggleFavorite(track_id);
        }
    } else {
        render_empty_track_zone(ui, palette);
    }
}

/// Helper for rendering title, artist, and format badge column.
#[allow(clippy::too_many_arguments)]
fn render_title_artist_column(
    ui: &mut Ui,
    title: &str,
    artist: &str,
    badge: &str,
    is_hires: bool,
    palette: &BentoPalette,
    max_w: f32,
    show_badge: bool,
) {
    ui.allocate_ui_with_layout(
        vec2(max_w, 46.0),
        Layout::top_down(egui::Align::Min),
        |ui| {
            ui.set_clip_rect(ui.max_rect());
            ui.spacing_mut().item_spacing = vec2(0.0, 2.0);

            // Title - single-line ellipsis truncation
            let title_font = FontId::proportional(12.5);
            let truncated_title = truncate_string(ui, title, title_font, palette.text_primary, max_w);
            ui.add(
                egui::Label::new(
                    RichText::new(truncated_title)
                        .size(12.5)
                        .strong()
                        .color(palette.text_primary),
                )
                .truncate(),
            );

            // Artist and Quality Badge row
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = vec2(6.0, 0.0);
                let artist_max_w = if show_badge {
                    (max_w - 60.0).max(30.0)
                } else {
                    max_w
                };
                let artist_font = FontId::proportional(11.0);
                let truncated_artist = truncate_string(ui, artist, artist_font, palette.text_secondary, artist_max_w);
                ui.add(
                    egui::Label::new(
                        RichText::new(truncated_artist)
                            .size(11.0)
                            .color(palette.text_secondary),
                    )
                    .truncate(),
                );

                // Quality format pill badge via quality_badge_pill (hidden when total_w < 900.0)
                if show_badge {
                    quality_badge_pill(ui, badge, is_hires, palette);
                }
            });
        },
    );
}

/// Fallback placeholder when no track is currently playing.
fn render_empty_track_zone(ui: &mut Ui, palette: &BentoPalette) {
    let (art_rect, _) = ui.allocate_exact_size(vec2(46.0, 46.0), egui::Sense::hover());
    ui.painter().rect_filled(art_rect, Rounding::same(8.0), palette.bg_card);
    let icon_rect = Rect::from_center_size(art_rect.center(), vec2(20.0, 20.0));
    paint_icon(ui.painter(), Icon::Music, icon_rect, palette.text_muted);

    ui.add_space(8.0);
    ui.vertical(|ui| {
        ui.label(RichText::new("Not Playing").color(palette.text_muted).size(12.5));
        ui.label(RichText::new("Select a track to start").color(palette.text_muted).size(10.5));
    });
}

/// Renders a crisp format pill badge (e.g. FLAC 24/48, MP3 320k, STREAM).
#[allow(dead_code)]
fn render_dock_format_badge(ui: &mut Ui, text: &str, palette: &BentoPalette) {
    let is_hires = text.contains("24/") || text.contains("96") || text.contains("192");
    quality_badge_pill(ui, text, is_hires, palette);
}

// =============================================================================
// Zone 2: Transport Controls & Scrubbable Seekbar
// =============================================================================

fn render_zone2_transport(
    ui: &mut Ui,
    player: &mut PlayerController,
    stream_controller: Option<&StreamController>,
    palette: &BentoPalette,
    center_w: f32,
) {
    ui.spacing_mut().item_spacing = vec2(0.0, 1.0);

    let is_stream = player.playback_source.is_stream();
    let is_playing = if is_stream {
        if player.engine.is_playing() {
            true
        } else if player.engine.current_path().is_some() && player.engine.is_paused() {
            false
        } else {
            stream_controller.and_then(|sc| sc.active_state.as_ref()).map(|s| s.is_playing).unwrap_or(false)
        }
    } else {
        player.engine.is_playing()
    };

    // -------------------------------------------------------------------------
    // Row 1: Transport Buttons (Shuffle, Prev, Play/Pause, Next, Repeat)
    // LEVEL TRANSPORT RULE: Every transport button must share the exact same center().y!
    // -------------------------------------------------------------------------
    let controls_h = 36.0;
    let (controls_rect, _) = ui.allocate_exact_size(vec2(center_w, controls_h), egui::Sense::hover());
    let center_y = controls_rect.center().y;
    let cx = controls_rect.center().x;

    // Explicit shared center_y positions for 100% dead-level alignment
    let shuffle_rect = Rect::from_center_size(pos2(cx - 88.0, center_y), vec2(28.0, 28.0));
    let prev_rect = Rect::from_center_size(pos2(cx - 44.0, center_y), vec2(28.0, 28.0));
    let play_rect = Rect::from_center_size(pos2(cx, center_y), vec2(32.0, 32.0));
    let next_rect = Rect::from_center_size(pos2(cx + 44.0, center_y), vec2(28.0, 28.0));
    let repeat_rect = Rect::from_center_size(pos2(cx + 88.0, center_y), vec2(28.0, 28.0));

    // 1. Shuffle Button
    let shuffle_resp = ui.interact(shuffle_rect, ui.id().with("bento_dock_shuffle"), egui::Sense::click());
    if shuffle_resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        ui.painter().rect_filled(shuffle_rect, Rounding::same(6.0), palette.bg_card_hover);
    }
    let shuffle_color = if !is_stream && player.shuffle { palette.accent } else { palette.text_muted };
    paint_icon(ui.painter(), Icon::Shuffle, Rect::from_center_size(shuffle_rect.center(), vec2(14.0, 14.0)), shuffle_color);
    let shuffle_resp = shuffle_resp.on_hover_text("Shuffle (S)");
    if shuffle_resp.clicked() && !is_stream {
        player.toggle_shuffle();
    }

    // 2. Previous Button
    let prev_resp = ui.interact(prev_rect, ui.id().with("bento_dock_prev"), egui::Sense::click());
    if prev_resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        ui.painter().rect_filled(prev_rect, Rounding::same(6.0), palette.bg_card_hover);
    }
    paint_icon(ui.painter(), Icon::SkipBack, Rect::from_center_size(prev_rect.center(), vec2(15.0, 15.0)), palette.text_primary);
    let prev_resp = prev_resp.on_hover_text("Previous (B)");
    if prev_resp.clicked() {
        player.prev();
    }

    // 3. Play / Pause Button (32px diameter circular accent badge with white vector icon)
    let play_resp = ui.interact(play_rect, ui.id().with("bento_dock_play"), egui::Sense::click());
    if play_resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    ui.painter().circle_filled(play_rect.center(), 16.0, palette.accent);

    let play_icon = if is_playing { Icon::Pause } else { Icon::Play };
    let play_icon_rect = Rect::from_center_size(play_rect.center(), vec2(14.0, 14.0));
    paint_icon(ui.painter(), play_icon, play_icon_rect, Color32::WHITE);

    let play_resp = play_resp.on_hover_text(if is_playing { "Pause (Space)" } else { "Play (Space)" });
    if play_resp.clicked() {
        if is_stream {
            let prov_id = player.stream_provider_id().unwrap_or("spotify");
            if player.engine.current_path().is_some() {
                player.engine.toggle_play_pause();
            } else if let Some(sc) = stream_controller {
                if is_playing {
                    sc.pause(prov_id);
                } else {
                    sc.resume(prov_id);
                }
            }
        } else if player.current_track().is_some() {
            player.engine.toggle_play_pause();
        } else if !player.queue.is_empty() {
            player.play_index(0);
        }
    }

    // 4. Next Button
    let next_resp = ui.interact(next_rect, ui.id().with("bento_dock_next"), egui::Sense::click());
    if next_resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        ui.painter().rect_filled(next_rect, Rounding::same(6.0), palette.bg_card_hover);
    }
    paint_icon(ui.painter(), Icon::SkipForward, Rect::from_center_size(next_rect.center(), vec2(15.0, 15.0)), palette.text_primary);
    let next_resp = next_resp.on_hover_text("Next (N)");
    if next_resp.clicked() {
        player.next();
    }

    // 5. Repeat Button
    let repeat_resp = ui.interact(repeat_rect, ui.id().with("bento_dock_repeat"), egui::Sense::click());
    if repeat_resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        ui.painter().rect_filled(repeat_rect, Rounding::same(6.0), palette.bg_card_hover);
    }
    let (rep_icon, rep_color) = if !is_stream {
        match player.repeat {
            RepeatMode::Off => (Icon::Repeat, palette.text_muted),
            RepeatMode::All => (Icon::Repeat, palette.accent),
            RepeatMode::One => (Icon::RepeatOne, palette.accent),
        }
    } else {
        (Icon::Repeat, palette.text_muted)
    };
    paint_icon(ui.painter(), rep_icon, Rect::from_center_size(repeat_rect.center(), vec2(14.0, 14.0)), rep_color);
    let repeat_resp = repeat_resp.on_hover_text("Repeat (R)");
    if repeat_resp.clicked() && !is_stream {
        player.cycle_repeat();
    }

    // -------------------------------------------------------------------------
    // Row 2: Seekbar & Timestamps directly beneath transport controls
    // -------------------------------------------------------------------------
    let (pos, dur) = current_playback_time(player, stream_controller);

    let seekbar_id = egui::Id::new("eclipse_bento_seekbar");
    let mut drag_state = ui.data(|d| d.get_temp::<SeekbarDragState>(seekbar_id)).unwrap_or_default();

    // Check if committed preview is still holding to prevent snap-back
    if let (Some(committed_secs), Some(commit_time)) = (drag_state.last_committed_secs, drag_state.last_committed_time) {
        if commit_time.elapsed() < Duration::from_millis(350) && (pos - committed_secs).abs() > 0.4 {
            // Hold preview position
        } else {
            drag_state.last_committed_secs = None;
            drag_state.last_committed_time = None;
        }
    }

    let display_pos = if drag_state.is_dragging {
        drag_state.scrub_secs
    } else if let Some(committed_secs) = drag_state.last_committed_secs {
        committed_secs
    } else {
        pos
    };

    let orig_interact_y = ui.spacing().interact_size.y;
    ui.spacing_mut().interact_size.y = 16.0;

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = vec2(6.0, 0.0);

        // Elapsed timestamp
        ui.label(
            RichText::new(format_time_secs(display_pos))
                .color(palette.text_muted)
                .size(11.0),
        );

        // Scrubbable seek rail
        let seekbar_w = (center_w - 90.0).max(60.0);
        render_dock_seekbar(
            ui,
            display_pos,
            dur,
            seekbar_w,
            player,
            stream_controller,
            palette,
            seekbar_id,
            &mut drag_state,
        );

        // Total duration timestamp
        ui.label(
            RichText::new(format_time_secs(dur))
                .color(palette.text_muted)
                .size(11.0),
        );
    });

    ui.spacing_mut().interact_size.y = orig_interact_y;
}

/// Interactive scrubbable seek rail with smooth dragging, preview knob, and zero panics.
#[allow(clippy::too_many_arguments)]
fn render_dock_seekbar(
    ui: &mut Ui,
    display_pos: f64,
    dur: f64,
    width: f32,
    player: &mut PlayerController,
    stream_controller: Option<&StreamController>,
    palette: &BentoPalette,
    seekbar_id: egui::Id,
    drag_state: &mut SeekbarDragState,
) {
    let (rect, resp) = ui.allocate_exact_size(vec2(width, 16.0), egui::Sense::click_and_drag());
    let is_hovered = resp.hovered();
    let is_dragged = resp.dragged();

    if is_hovered || is_dragged {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }

    let max_dur = if dur > 0.0 { dur } else { 1.0 };
    let bar_h = if is_hovered || is_dragged { 4.5 } else { 3.0 };
    let bar_rect = Rect::from_center_size(rect.center(), vec2(rect.width(), bar_h));
    let rounding = Rounding::same(bar_h / 2.0);

    // Scrub handling
    if resp.drag_started() {
        drag_state.is_dragging = true;
        drag_state.scrub_secs = display_pos;
    }

    if is_dragged {
        if let Some(pointer) = resp.interact_pointer_pos() {
            let ratio = ((pointer.x - bar_rect.min.x) / bar_rect.width()).clamp(0.0, 1.0);
            drag_state.scrub_secs = ratio as f64 * max_dur;
        }
    }

    if resp.drag_stopped() {
        if drag_state.is_dragging {
            seek_to(player, stream_controller, drag_state.scrub_secs);
            drag_state.last_committed_secs = Some(drag_state.scrub_secs);
            drag_state.last_committed_time = Some(std::time::Instant::now());
        }
        drag_state.is_dragging = false;
    } else if resp.clicked() {
        if let Some(pointer) = resp.interact_pointer_pos() {
            let ratio = ((pointer.x - bar_rect.min.x) / bar_rect.width()).clamp(0.0, 1.0);
            let target = ratio as f64 * max_dur;
            seek_to(player, stream_controller, target);
            drag_state.last_committed_secs = Some(target);
            drag_state.last_committed_time = Some(std::time::Instant::now());
            drag_state.is_dragging = false;
        }
    }

    ui.data_mut(|d| d.insert_temp(seekbar_id, *drag_state));

    let active_pos = if drag_state.is_dragging {
        drag_state.scrub_secs
    } else {
        display_pos
    };

    let progress = (active_pos / max_dur).clamp(0.0, 1.0) as f32;

    // Background track rail
    ui.painter().rect_filled(bar_rect, rounding, palette.bg_card);

    // Buffer / hover preview fill: palette.accent_soft
    if is_hovered || is_dragged {
        if let Some(pointer) = resp.interact_pointer_pos() {
            let hover_ratio = ((pointer.x - bar_rect.min.x) / bar_rect.width()).clamp(0.0, 1.0);
            let hover_w = bar_rect.width() * hover_ratio;
            if hover_w > 0.0 {
                let hover_rect = Rect::from_min_size(bar_rect.min, vec2(hover_w, bar_h));
                ui.painter().rect_filled(hover_rect, rounding, palette.accent_soft);
            }
        }
    }

    // Active progress fill: palette.accent
    let fill_w = bar_rect.width() * progress;
    if fill_w > 0.0 {
        let fill_rect = Rect::from_min_size(bar_rect.min, vec2(fill_w, bar_h));
        ui.painter().rect_filled(fill_rect, rounding, palette.accent);
    }

    // Scrubber knob on hover/drag
    if is_hovered || is_dragged {
        let knob_x = bar_rect.min.x + fill_w;
        let knob_pos = pos2(knob_x, bar_rect.center().y);
        let knob_radius = if is_dragged { 5.0 } else { 4.0 };
        ui.painter().circle_filled(knob_pos, knob_radius, Color32::WHITE);
        ui.painter().circle_stroke(knob_pos, knob_radius, Stroke::new(1.0_f32, Color32::from_rgb(10, 10, 12)));

        resp.on_hover_ui(|ui| {
            ui.label(
                RichText::new(format_time_secs(active_pos))
                    .size(11.0)
                    .color(palette.text_primary),
            );
        });
    }
}

// =============================================================================
// Zone 3: Right (Inspector, Fullscreen, Pop-out, Lyrics, Slim Volume Slider)
// =============================================================================

fn render_zone3_controls(
    ui: &mut Ui,
    player: &mut PlayerController,
    stream_controller: Option<&StreamController>,
    palette: &BentoPalette,
    is_inspector_open: bool,
    slider_w: Option<f32>,
    action: &mut BentoPlayerDockAction,
) {
    ui.spacing_mut().item_spacing = vec2(6.0, 0.0);

    // 1. Right Inspector toggle icon button (far right)
    let (insp_rect, insp_resp) = ui.allocate_exact_size(vec2(28.0, 28.0), egui::Sense::click());
    let is_insp_hovered = insp_resp.hovered();

    if is_insp_hovered {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        ui.painter().rect_filled(insp_rect, Rounding::same(6.0), palette.bg_card_hover);
    }

    let insp_color = if is_inspector_open {
        palette.accent
    } else if is_insp_hovered {
        palette.text_primary
    } else {
        palette.text_secondary
    };
    paint_panel_right(ui.painter(), Rect::from_center_size(insp_rect.center(), vec2(16.0, 16.0)), insp_color);
    let insp_resp = insp_resp.on_hover_text("Toggle Inspector");
    if insp_resp.clicked() {
        *action = BentoPlayerDockAction::ToggleRightInspector;
    }

    // 2. Cinematic Fullscreen icon button
    let (fs_rect, fs_resp) = ui.allocate_exact_size(vec2(28.0, 28.0), egui::Sense::click());
    let is_fs_hovered = fs_resp.hovered();

    if is_fs_hovered {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        ui.painter().rect_filled(fs_rect, Rounding::same(6.0), palette.bg_card_hover);
    }
    let fs_color = if is_fs_hovered { palette.text_primary } else { palette.text_secondary };
    paint_icon(ui.painter(), Icon::Maximize, Rect::from_center_size(fs_rect.center(), vec2(15.0, 15.0)), fs_color);
    let fs_resp = fs_resp.on_hover_text("Cinematic Fullscreen (F11)");
    if fs_resp.clicked() {
        *action = BentoPlayerDockAction::OpenFullscreen;
    }

    // 3. Pop-out Mini-Player icon button
    let (pip_rect, pip_resp) = ui.allocate_exact_size(vec2(28.0, 28.0), egui::Sense::click());
    let is_pip_hovered = pip_resp.hovered();

    if is_pip_hovered {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        ui.painter().rect_filled(pip_rect, Rounding::same(6.0), palette.bg_card_hover);
    }
    let pip_color = if is_pip_hovered { palette.text_primary } else { palette.text_secondary };
    paint_icon(ui.painter(), Icon::PictureInPicture, Rect::from_center_size(pip_rect.center(), vec2(15.0, 15.0)), pip_color);
    let pip_resp = pip_resp.on_hover_text("Mini Player & Pop-out (P)");
    if pip_resp.clicked() {
        *action = BentoPlayerDockAction::OpenPopoutPip;
    }

    // 4. Jump to Lyrics icon button (Feather file-text vector)
    let (lyr_rect, lyr_resp) = ui.allocate_exact_size(vec2(28.0, 28.0), egui::Sense::click());
    let is_lyr_hovered = lyr_resp.hovered();

    if is_lyr_hovered {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        ui.painter().rect_filled(lyr_rect, Rounding::same(6.0), palette.bg_card_hover);
    }
    let lyr_color = if is_lyr_hovered { palette.text_primary } else { palette.text_secondary };
    paint_file_text(ui.painter(), Rect::from_center_size(lyr_rect.center(), vec2(16.0, 16.0)), lyr_color);
    let lyr_resp = lyr_resp.on_hover_text("Synced Lyrics (L)");
    if lyr_resp.clicked() {
        *action = BentoPlayerDockAction::JumpToLyrics;
    }

    // 5. Slim Volume Slider (rendered if slider_w is Some, hidden below 820.0px)
    let vol = player.engine.volume();
    if let Some(desired_w) = slider_w {
        render_dock_volume_slider(ui, vol, desired_w, player, stream_controller, palette);
    }

    // 6. Volume Mute Icon Button (always 100% visible)
    let is_muted = player.engine.is_muted();
    let vol_icon = if is_muted || vol <= 0.001 {
        Icon::VolumeMute
    } else if vol < 0.4 {
        Icon::VolumeLow
    } else if vol < 0.75 {
        Icon::VolumeMed
    } else {
        Icon::VolumeHigh
    };

    let (mute_rect, mute_resp) = ui.allocate_exact_size(vec2(28.0, 28.0), egui::Sense::click());
    let is_mute_hovered = mute_resp.hovered();

    if is_mute_hovered {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        ui.painter().rect_filled(mute_rect, Rounding::same(6.0), palette.bg_card_hover);

        // Scroll wheel adjusts volume even when hovering mute button
        let scroll_y = ui.input(|i| i.raw_scroll_delta.y);
        if scroll_y != 0.0 {
            let step = if scroll_y > 0.0 { 0.05 } else { -0.05 };
            let new_vol = (vol + step).clamp(0.0, 1.0);
            if player.playback_source.is_stream() {
                if let Some(sc) = stream_controller {
                    let prov_id = player.stream_provider_id().unwrap_or("spotify");
                    sc.set_volume(prov_id, new_vol);
                }
            } else {
                if player.engine.is_muted() && new_vol > 0.0 {
                    player.engine.set_muted(false);
                }
                player.engine.set_volume(new_vol);
            }
        }
    }
    let vol_btn_color = if is_muted {
        Color32::from_rgb(220, 38, 38)
    } else if is_mute_hovered {
        palette.text_primary
    } else {
        palette.text_secondary
    };
    paint_icon(ui.painter(), vol_icon, Rect::from_center_size(mute_rect.center(), vec2(15.0, 15.0)), vol_btn_color);
    let mute_resp = mute_resp.on_hover_text("Mute / Unmute (M)");
    if mute_resp.clicked() {
        if player.playback_source.is_stream() {
            let prov_id = player.stream_provider_id().unwrap_or("spotify");
            if let Some(sc) = stream_controller {
                if vol > 0.001 {
                    sc.set_volume(prov_id, 0.0);
                } else {
                    sc.set_volume(prov_id, 1.0);
                }
            }
        } else {
            player.engine.toggle_mute();
        }
    }
}

/// Clean slim volume slider with hover/drag scrub and scroll wheel adjustment.
fn render_dock_volume_slider(
    ui: &mut Ui,
    vol: f32,
    desired_w: f32,
    player: &mut PlayerController,
    stream_controller: Option<&StreamController>,
    palette: &BentoPalette,
) {
    let (rect, resp) = ui.allocate_exact_size(vec2(desired_w, 16.0), egui::Sense::click_and_drag());
    let is_hovered = resp.hovered();
    let is_dragged = resp.dragged();

    if is_hovered || is_dragged {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }

    // Scroll wheel adjusts volume by ±5%
    if is_hovered {
        let scroll_y = ui.input(|i| i.raw_scroll_delta.y);
        if scroll_y != 0.0 {
            let step = if scroll_y > 0.0 { 0.05 } else { -0.05 };
            let new_vol = (vol + step).clamp(0.0, 1.0);
            if player.playback_source.is_stream() {
                if let Some(sc) = stream_controller {
                    let prov_id = player.stream_provider_id().unwrap_or("spotify");
                    sc.set_volume(prov_id, new_vol);
                }
            } else {
                if player.engine.is_muted() && new_vol > 0.0 {
                    player.engine.set_muted(false);
                }
                player.engine.set_volume(new_vol);
            }
        }
    }

    let bar_h = if is_hovered || is_dragged { 4.5 } else { 3.0 };
    let bar_rect = Rect::from_center_size(rect.center(), vec2(rect.width(), bar_h));
    let rounding = Rounding::same(bar_h / 2.0);

    let effective_vol = if is_dragged || resp.clicked() {
        if let Some(pos) = resp.interact_pointer_pos() {
            let ratio = ((pos.x - bar_rect.min.x) / bar_rect.width()).clamp(0.0, 1.0);
            if player.playback_source.is_stream() {
                if let Some(sc) = stream_controller {
                    let prov_id = player.stream_provider_id().unwrap_or("spotify");
                    sc.set_volume(prov_id, ratio);
                }
            } else {
                if player.engine.is_muted() && ratio > 0.0 {
                    player.engine.set_muted(false);
                }
                player.engine.set_volume(ratio);
            }
            ratio
        } else {
            vol
        }
    } else {
        vol
    };

    // Dark track rail
    ui.painter().rect_filled(bar_rect, rounding, palette.bg_card);

    // Active volume fill in palette.accent
    let fill_w = bar_rect.width() * effective_vol;
    if fill_w > 0.0 {
        let fill_rect = Rect::from_min_size(bar_rect.min, vec2(fill_w, bar_h));
        ui.painter().rect_filled(fill_rect, rounding, palette.accent);
    }

    // Scrubber knob on hover/drag
    if is_hovered || is_dragged {
        let knob_x = bar_rect.min.x + fill_w;
        let knob_pos = pos2(knob_x, bar_rect.center().y);
        ui.painter().circle_filled(knob_pos, 4.0, Color32::WHITE);
        ui.painter().circle_stroke(knob_pos, 4.0, Stroke::new(1.0_f32, Color32::from_rgb(10, 10, 12)));

        resp.on_hover_ui(|ui| {
            ui.label(
                RichText::new(format!("{}%", (effective_vol * 100.0).round() as u32))
                    .size(11.0)
                    .color(palette.text_primary),
            );
        });
    }
}

// =============================================================================
// Pure Vector Drawing Helpers: Feather FileText & Lucide PanelRight
// =============================================================================

/// Pure vector Feather/Lucide `file-text` icon for synced lyrics.
fn paint_file_text(painter: &egui::Painter, rect: Rect, color: Color32) {
    let side = rect.width().min(rect.height());
    let s = side / 24.0;
    let center = rect.center();
    let origin = pos2(center.x - 12.0 * s, center.y - 12.0 * s);
    let p = |x: f32, y: f32| pos2(origin.x + x * s, origin.y + y * s);
    let stroke = Stroke::new((1.75 * s).max(1.1), color);

    // Page outline with dog-ear fold at top-right
    painter.line_segment([p(14.0, 2.0), p(6.0, 2.0)], stroke);
    painter.line_segment([p(6.0, 2.0), p(6.0, 22.0)], stroke);
    painter.line_segment([p(6.0, 22.0), p(18.0, 22.0)], stroke);
    painter.line_segment([p(18.0, 22.0), p(18.0, 6.0)], stroke);
    painter.line_segment([p(18.0, 6.0), p(14.0, 2.0)], stroke);

    // Dog-ear fold
    painter.line_segment([p(14.0, 2.0), p(14.0, 6.0)], stroke);
    painter.line_segment([p(14.0, 6.0), p(18.0, 6.0)], stroke);

    // 3 Text lines
    painter.line_segment([p(9.0, 10.0), p(15.0, 10.0)], stroke);
    painter.line_segment([p(9.0, 14.0), p(15.0, 14.0)], stroke);
    painter.line_segment([p(9.0, 18.0), p(13.0, 18.0)], stroke);
}

/// Pure vector Lucide `panel-right` icon for right inspector drawer.
fn paint_panel_right(painter: &egui::Painter, rect: Rect, color: Color32) {
    let side = rect.width().min(rect.height());
    let s = side / 24.0;
    let center = rect.center();
    let origin = pos2(center.x - 12.0 * s, center.y - 12.0 * s);
    let p = |x: f32, y: f32| pos2(origin.x + x * s, origin.y + y * s);
    let stroke = Stroke::new((1.75 * s).max(1.1), color);

    // Outer frame with slight corner rounding
    let box_rect = Rect::from_min_max(p(3.0, 3.0), p(21.0, 21.0));
    painter.rect_stroke(box_rect, Rounding::same(2.0 * s), stroke);

    // Vertical dividing line representing right panel
    painter.line_segment([p(15.0, 3.0), p(15.0, 21.0)], stroke);
}

/// Helper for safe string truncation with ellipsis.
fn truncate_string(
    ui: &Ui,
    text: &str,
    font: FontId,
    color: Color32,
    max_w: f32,
) -> String {
    let clean = if text.trim().is_empty() { "Unknown" } else { text };
    let full_w = ui.painter().layout_no_wrap(clean.to_string(), font.clone(), color).rect.width();
    if full_w <= max_w {
        return clean.to_string();
    }

    let mut truncated = String::new();
    for c in clean.chars() {
        let candidate = format!("{}…", truncated);
        let w = ui.painter().layout_no_wrap(candidate, font.clone(), color).rect.width();
        if w > max_w {
            break;
        }
        truncated.push(c);
    }
    format!("{}…", truncated)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dock_action_default() {
        let action = BentoPlayerDockAction::default();
        assert_eq!(action, BentoPlayerDockAction::None);
    }

    #[test]
    fn test_dock_action_variants() {
        let actions = [
            BentoPlayerDockAction::None,
            BentoPlayerDockAction::ToggleFavorite(101),
            BentoPlayerDockAction::ToggleRightInspector,
            BentoPlayerDockAction::JumpToLyrics,
            BentoPlayerDockAction::OpenPopoutPip,
            BentoPlayerDockAction::OpenFullscreen,
        ];
        assert_eq!(actions[1], BentoPlayerDockAction::ToggleFavorite(101));
        assert_eq!(actions[2], BentoPlayerDockAction::ToggleRightInspector);
        assert_eq!(actions[3], BentoPlayerDockAction::JumpToLyrics);
        assert_eq!(actions[4], BentoPlayerDockAction::OpenPopoutPip);
        assert_eq!(actions[5], BentoPlayerDockAction::OpenFullscreen);
    }

    #[test]
    fn test_dock_transport_level_alignment() {
        let center_y = 37.5_f32;
        let cx = 250.0_f32;

        let shuffle_rect = Rect::from_center_size(pos2(cx - 88.0, center_y), vec2(28.0, 28.0));
        let prev_rect = Rect::from_center_size(pos2(cx - 44.0, center_y), vec2(28.0, 28.0));
        let play_rect = Rect::from_center_size(pos2(cx, center_y), vec2(32.0, 32.0));
        let next_rect = Rect::from_center_size(pos2(cx + 44.0, center_y), vec2(28.0, 28.0));
        let repeat_rect = Rect::from_center_size(pos2(cx + 88.0, center_y), vec2(28.0, 28.0));

        assert_eq!(shuffle_rect.center().y, center_y);
        assert_eq!(prev_rect.center().y, center_y);
        assert_eq!(play_rect.center().y, center_y);
        assert_eq!(next_rect.center().y, center_y);
        assert_eq!(repeat_rect.center().y, center_y);
    }

    #[test]
    fn test_vector_icons_paint_without_panic() {
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let rect = Rect::from_min_size(pos2(10.0, 10.0), vec2(24.0, 24.0));
                paint_file_text(ui.painter(), rect, Color32::WHITE);
                paint_panel_right(ui.painter(), rect, Color32::WHITE);
            });
        });
    }

    #[test]
    fn test_render_bento_player_dock_smoke() {
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let theme = Theme::from_name("Obsidian", None);
                let cfg = crate::config::AppConfig::default();
                let mut player = PlayerController::new(&cfg);
                let mut art_manager = AlbumArtManager::new();

                let action = render_bento_player_dock(
                    ui,
                    &mut player,
                    None,
                    &mut art_manager,
                    &theme,
                    false,
                );

                assert_eq!(action, BentoPlayerDockAction::None);
            });
        });
    }

    #[test]
    fn test_dock_width_budgets_and_adaptive_slider() {
        // 1. Wide viewport (>= 960.0): full slider (~80px)
        let total_w_wide = 1200.0_f32;
        let (right_w_wide, slider_w_wide) = if total_w_wide >= 960.0 {
            (252.0_f32, Some(80.0_f32))
        } else if total_w_wide >= 820.0 {
            (216.0_f32, Some(44.0_f32))
        } else {
            (168.0_f32, None)
        };
        let left_w_wide = (total_w_wide * 0.27).clamp(170.0, 290.0);
        let center_w_wide = (total_w_wide - left_w_wide - right_w_wide - 24.0).max(200.0);

        assert_eq!(right_w_wide, 252.0);
        assert_eq!(slider_w_wide, Some(80.0));
        assert_eq!(left_w_wide, 290.0);
        assert_eq!(center_w_wide, 1200.0 - 290.0 - 252.0 - 24.0);

        // 2. Compact viewport (820.0 <= total_w < 960.0): compact slider (~44px)
        let total_w_compact = 880.0_f32;
        let (right_w_compact, slider_w_compact) = if total_w_compact >= 960.0 {
            (252.0_f32, Some(80.0_f32))
        } else if total_w_compact >= 820.0 {
            (216.0_f32, Some(44.0_f32))
        } else {
            (168.0_f32, None)
        };
        let left_w_compact = (total_w_compact * 0.27).clamp(170.0, 290.0);
        let center_w_compact = (total_w_compact - left_w_compact - right_w_compact - 24.0).max(200.0);

        assert_eq!(right_w_compact, 216.0);
        assert_eq!(slider_w_compact, Some(44.0));
        assert!((left_w_compact - 880.0 * 0.27).abs() < 1e-4);
        assert!(center_w_compact >= 200.0);

        // 3. Narrow viewport (< 820.0): slider track hidden (None), all utility icons + mute visible
        let total_w_narrow = 760.0_f32;
        let (right_w_narrow, slider_w_narrow) = if total_w_narrow >= 960.0 {
            (252.0_f32, Some(80.0_f32))
        } else if total_w_narrow >= 820.0 {
            (216.0_f32, Some(44.0_f32))
        } else {
            (168.0_f32, None)
        };
        let left_w_narrow = (total_w_narrow * 0.27).clamp(170.0, 290.0);
        let center_w_narrow = (total_w_narrow - left_w_narrow - right_w_narrow - 24.0).max(200.0);

        assert_eq!(right_w_narrow, 168.0);
        assert_eq!(slider_w_narrow, None);
        assert!((left_w_narrow - 760.0 * 0.27).abs() < 1e-4);
        assert!(center_w_narrow >= 200.0);
    }

    #[test]
    fn test_render_bento_player_dock_different_widths() {
        for width in [760.0, 810.0, 880.0, 960.0, 1280.0] {
            let ctx = egui::Context::default();
            let _ = ctx.run(egui::RawInput::default(), |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.set_width(width);
                    let theme = Theme::from_name("Obsidian", None);
                    let cfg = crate::config::AppConfig::default();
                    let mut player = PlayerController::new(&cfg);
                    let mut art_manager = AlbumArtManager::new();

                    let action = render_bento_player_dock(
                        ui,
                        &mut player,
                        None,
                        &mut art_manager,
                        &theme,
                        false,
                    );

                    assert_eq!(action, BentoPlayerDockAction::None);
                });
            });
        }
    }

    #[test]
    fn test_truncate_string_helper() {
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let font = FontId::proportional(12.0);
                let short = truncate_string(ui, "Track Title", font.clone(), Color32::WHITE, 200.0);
                assert_eq!(short, "Track Title");

                let long_title = "Extremely Long Track Title That Exceeds Any Standard Bento Player Bar Column Width";
                let truncated = truncate_string(ui, long_title, font, Color32::WHITE, 50.0);
                assert!(truncated.ends_with('…'));
            });
        });
    }
}
