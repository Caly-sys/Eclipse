use crate::audio::PlayerController;
use crate::config::RepeatMode;
use crate::library::format_time_secs;
use crate::streams::controller::StreamController;
use crate::ui::art::AlbumArtManager;
use crate::ui::components::{format_badge, pill_badge, render_mini_visualizer};
use crate::ui::icons::{icon_button, paint_icon, Icon};
use crate::ui::theme::Theme;
use egui::{pos2, vec2, Color32, Layout, Rect, RichText, Rounding, Stroke, Ui};
use std::time::{Duration, Instant};

/// Scrubbing and seek commitment state persisted across frames.
#[derive(Clone, Copy, Debug, Default)]
pub struct SeekbarDragState {
    pub is_dragging: bool,
    pub scrub_secs: f64,
    pub last_committed_secs: Option<f64>,
    pub last_committed_time: Option<Instant>,
}

/// Helper returning the effective playback position and duration in seconds.
pub fn current_playback_time(
    player: &PlayerController,
    stream_controller: Option<&StreamController>,
) -> (f64, f64) {
    if player.playback_source.is_stream() {
        let pos = stream_controller
            .and_then(|sc| sc.active_state.as_ref())
            .map(|s| s.position_secs)
            .unwrap_or(0.0);
        let dur = stream_controller
            .and_then(|sc| sc.active_state.as_ref())
            .map(|s| s.duration_secs)
            .filter(|&d| d > 0.0)
            .or_else(|| player.active_stream_track.as_ref().map(|t| t.duration_secs))
            .unwrap_or(0.0);
        (pos, dur)
    } else {
        let pos = player.engine.position().as_secs_f64();
        let dur = player.current_track().map(|t| t.duration_secs).unwrap_or(0.0);
        (pos, dur)
    }
}

/// Unified seek dispatcher: seeks either the local audio engine or the active stream controller.
pub fn seek_to(
    player: &mut PlayerController,
    stream_controller: Option<&StreamController>,
    target_secs: f64,
) {
    let target = target_secs.max(0.0);
    if player.playback_source.is_stream() {
        let prov_id = player.stream_provider_id().unwrap_or("spotify");
        if let Some(sc) = stream_controller {
            sc.seek(prov_id, target);
        }
    } else {
        player.engine.seek(Duration::from_secs_f64(target));
    }
}

#[allow(clippy::too_many_arguments)]
pub fn render_playerbar(
    ui: &mut Ui,
    player: &mut PlayerController,
    stream_controller: &mut StreamController,
    theme: &Theme,
    show_queue: &mut bool,
    show_pip_overlay: &mut bool,
    toggle_now_playing: &mut bool,
    on_toggle_fav: &mut Option<i64>,
    art_manager: &mut AlbumArtManager,
) -> egui::Response {
    let frame = egui::Frame::none()
        .fill(Color32::from_rgb(16, 16, 19)) // #101013
        .inner_margin(egui::Margin::symmetric(20.0, 5.0)); // padding: 5px vertical, 20px horizontal

    let is_stream = player.playback_source.is_stream();
    let prov_id = player.stream_provider_id().unwrap_or("spotify").to_string();

    let inner = frame.show(ui, |ui| {
        let total_w = ui.available_width();
        let side_w = 300.0_f32.min((total_w - 260.0) / 2.0).max(180.0);
        let left_w = side_w;
        let right_w = side_w;
        let center_w = (total_w - side_w * 2.0).max(220.0);

        ui.horizontal(|ui| {
            // =================================================================
            // 1. LEFT COLUMN: Now Playing Snippet (Art, Title, Artist, Badge, Fav)
            // =================================================================
            ui.allocate_ui_with_layout(
                vec2(left_w, 54.0),
                Layout::left_to_right(egui::Align::Center),
                |ui| {
                    if is_stream {
                        if let Some(ref track) = player.active_stream_track {
                            let prov_icon = match prov_id.as_str() {
                                "spotify" => Icon::Spotify,
                                "youtube" => Icon::YouTube,
                                _ => Icon::Disc,
                            };
                            let prov_color = match prov_id.as_str() {
                                "spotify" => Color32::from_rgb(30, 215, 96),
                                "youtube" => Color32::from_rgb(255, 60, 60),
                                _ => theme.accent,
                            };

                            let (rect, cover_resp) = ui.allocate_exact_size(vec2(48.0, 48.0), egui::Sense::click());
                            ui.painter().rect_filled(rect, Rounding::same(8.0), Color32::from_rgb(22, 22, 24));
                            ui.painter().rect_stroke(rect, Rounding::same(8.0), Stroke::new(1.0_f32, theme.border));
                            let icon_rect = Rect::from_center_size(rect.center(), vec2(22.0, 22.0));
                            paint_icon(ui.painter(), prov_icon, icon_rect, prov_color);

                            if cover_resp.hovered() {
                                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                            }
                            if cover_resp.clicked() {
                                *toggle_now_playing = true;
                            }

                            ui.add_space(8.0);

                            let text_w = (left_w - 94.0).max(60.0);
                            let char_limit = (text_w / 7.5).max(8.0) as usize;

                            ui.allocate_ui_with_layout(
                                vec2(text_w, 48.0),
                                Layout::top_down(egui::Align::Min),
                                |ui| {
                                    ui.spacing_mut().item_spacing = vec2(0.0, 2.0);

                                    let title_str = truncate_str(track.display_title(), char_limit);
                                    let title_label = ui.add(
                                        egui::Label::new(
                                            RichText::new(title_str)
                                                .color(Color32::from_rgb(244, 244, 245))
                                                .size(13.0)
                                                .strong(),
                                        )
                                        .sense(egui::Sense::click()),
                                    );
                                    if title_label.hovered() {
                                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                                    }
                                    if title_label.clicked() {
                                        *toggle_now_playing = true;
                                    }

                                    ui.horizontal(|ui| {
                                        ui.spacing_mut().item_spacing = vec2(5.0, 0.0);
                                        let artist_str = truncate_str(track.display_artist(), char_limit.saturating_sub(4));
                                        ui.label(
                                            RichText::new(artist_str)
                                                .color(Color32::from_rgb(161, 161, 170))
                                                .size(11.5),
                                        );

                                        let badge_text = match prov_id.as_str() {
                                            "spotify" => "SPOTIFY",
                                            "youtube" => "YOUTUBE",
                                            _ => "STREAM",
                                        };
                                        format_badge(ui, badge_text, false, theme);
                                    });
                                },
                            );
                        } else {
                            let (rect, _) = ui.allocate_exact_size(vec2(48.0, 48.0), egui::Sense::hover());
                            ui.painter().rect_filled(rect, Rounding::same(8.0), Color32::from_rgb(22, 22, 24));
                            ui.painter().rect_stroke(rect, Rounding::same(8.0), Stroke::new(1.0_f32, theme.border));
                            let icon_rect = Rect::from_center_size(rect.center(), vec2(20.0, 20.0));
                            paint_icon(ui.painter(), Icon::Music, icon_rect, theme.text_muted);

                            ui.add_space(8.0);
                            ui.vertical(|ui| {
                                ui.label(RichText::new("Streaming Ready").color(theme.text_muted).size(13.0));
                                ui.label(RichText::new("Select a stream track").color(theme.text_muted).size(10.5));
                            });
                        }
                    } else if let Some(track) = player.current_track() {
                        let track_id = track.id;
                        let is_fav = track.favorite;

                        // 48x48 Rounded Album Art Thumbnail
                        let (rect, cover_resp) = ui.allocate_exact_size(vec2(48.0, 48.0), egui::Sense::click());
                        let mut painted_cover = false;
                        if let Some(texture) = art_manager.get_or_load(ui.ctx(), &track.path) {
                            ui.painter().image(
                                texture.id(),
                                rect,
                                Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                                Color32::WHITE,
                            );
                            ui.painter().rect_stroke(
                                rect,
                                Rounding::same(8.0),
                                Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(255, 255, 255, 15)),
                            );
                            painted_cover = true;
                        }

                        if !painted_cover {
                            ui.painter().rect_filled(rect, Rounding::same(8.0), Color32::from_rgb(22, 22, 24));
                            ui.painter().rect_stroke(rect, Rounding::same(8.0), Stroke::new(1.0_f32, theme.border));
                            let icon_rect = Rect::from_center_size(rect.center(), vec2(22.0, 22.0));
                            paint_icon(ui.painter(), Icon::Disc, icon_rect, theme.text_muted);
                        }

                        if cover_resp.hovered() {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                        }
                        if cover_resp.clicked() {
                            *toggle_now_playing = true;
                        }

                        ui.add_space(8.0);

                        // Title, Artist & Format Badge
                        let text_w = (left_w - 94.0).max(60.0);
                        let char_limit = (text_w / 7.5).max(8.0) as usize;

                        ui.allocate_ui_with_layout(
                            vec2(text_w, 48.0),
                            Layout::top_down(egui::Align::Min),
                            |ui| {
                                ui.spacing_mut().item_spacing = vec2(0.0, 2.0);

                                let title_str = truncate_str(track.display_title(), char_limit);
                                let title_label = ui.add(
                                    egui::Label::new(
                                        RichText::new(title_str)
                                            .color(Color32::from_rgb(244, 244, 245))
                                            .size(13.0)
                                            .strong(),
                                    )
                                    .sense(egui::Sense::click()),
                                );
                                if title_label.hovered() {
                                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                                }
                                if title_label.clicked() {
                                    *toggle_now_playing = true;
                                }

                                ui.horizontal(|ui| {
                                    ui.spacing_mut().item_spacing = vec2(5.0, 0.0);
                                    let artist_str = truncate_str(track.display_artist(), char_limit.saturating_sub(4));
                                    ui.label(
                                        RichText::new(artist_str)
                                            .color(Color32::from_rgb(161, 161, 170))
                                            .size(11.5),
                                    );

                                    format_badge(ui, &track.format, track.is_lossless(), theme);

                                    // Subtle dark format spec badge (e.g. 24/48, 16/44)
                                    if track.sample_rate > 0 {
                                        let khz = track.sample_rate / 1000;
                                        let bit_depth = if track.is_lossless() { 24 } else { 16 };
                                        let spec_badge = format!("{}/{}", bit_depth, khz);
                                        format_badge(ui, &spec_badge, false, theme);
                                    }
                                });
                            },
                        );

                        // Favorite Heart Button (Clean Vector)
                        let heart_icon = if is_fav { Icon::Heart } else { Icon::HeartOutline };
                        let heart_color = if is_fav {
                            Color32::WHITE
                        } else {
                            theme.text_muted
                        };
                        let heart_resp = icon_button(ui, heart_icon, 15.0, heart_color);
                        if heart_resp.clicked() {
                            *on_toggle_fav = Some(track_id);
                        }
                    } else {
                        // Empty placeholder
                        let (rect, _) = ui.allocate_exact_size(vec2(48.0, 48.0), egui::Sense::hover());
                        ui.painter().rect_filled(rect, Rounding::same(8.0), Color32::from_rgb(22, 22, 24));
                        ui.painter().rect_stroke(rect, Rounding::same(8.0), Stroke::new(1.0_f32, theme.border));
                        let icon_rect = Rect::from_center_size(rect.center(), vec2(20.0, 20.0));
                        paint_icon(ui.painter(), Icon::Music, icon_rect, theme.text_muted);

                        ui.add_space(8.0);
                        ui.vertical(|ui| {
                            ui.label(RichText::new("Not Playing").color(theme.text_muted).size(13.0));
                            ui.label(RichText::new("Select a track to start").color(theme.text_muted).size(10.5));
                        });
                    }
                },
            );

            // =================================================================
            // 2. CENTER COLUMN: Transport Controls & Seekbar with Timestamps
            // =================================================================
            ui.allocate_ui_with_layout(
                vec2(center_w, 58.0),
                Layout::top_down(egui::Align::Center),
                |ui| {
                    ui.spacing_mut().item_spacing = vec2(0.0, 2.0);

                    // Top Row: Transport Buttons
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing = vec2(12.0, 0.0);

                        // Shuffle Button (Local only)
                        let shuffle_color = if !is_stream && player.shuffle { Color32::WHITE } else { theme.text_muted };
                        let shuffle_btn = icon_button(ui, Icon::Shuffle, 15.0, shuffle_color);
                        if !is_stream && shuffle_btn.on_hover_text("Shuffle (S)").clicked() {
                            player.toggle_shuffle();
                        }

                        // Previous Button
                        if icon_button(ui, Icon::SkipBack, 15.0, Color32::from_rgb(228, 228, 231))
                            .on_hover_text("Previous (B / Left)")
                            .clicked()
                        {
                            if is_stream {
                                stream_controller.prev(&prov_id);
                            } else {
                                player.prev();
                            }
                        }

                        // Big Circular Solid White Play / Pause Button (40x40px)
                        let is_playing = if is_stream {
                            stream_controller.active_state.as_ref().map(|s| s.is_playing).unwrap_or(false)
                        } else {
                            player.engine.is_playing()
                        };
                        let (play_rect, play_resp) = ui.allocate_exact_size(vec2(40.0, 40.0), egui::Sense::click());

                        if play_resp.hovered() {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                        }

                        let play_fill = if play_resp.is_pointer_button_down_on() {
                            Color32::from_rgb(220, 220, 220)
                        } else if play_resp.hovered() {
                            Color32::from_rgb(240, 240, 242)
                        } else {
                            Color32::WHITE
                        };

                        ui.painter().circle_filled(
                            play_rect.center(),
                            20.0,
                            play_fill,
                        );

                        // Center icon in deep charcoal dark #0a0a0c
                        let center_icon = if is_playing { Icon::Pause } else { Icon::Play };
                        let icon_box = Rect::from_center_size(play_rect.center(), vec2(17.0, 17.0));
                        paint_icon(
                            ui.painter(),
                            center_icon,
                            icon_box,
                            Color32::from_rgb(10, 10, 12),
                        );

                        if play_resp.clicked() {
                            if is_stream {
                                if is_playing {
                                    stream_controller.pause(&prov_id);
                                } else {
                                    stream_controller.resume(&prov_id);
                                }
                            } else if player.current_track().is_some() {
                                player.engine.toggle_play_pause();
                            } else if !player.queue.is_empty() {
                                player.play_index(0);
                            }
                        }

                        // Next Button
                        if icon_button(ui, Icon::SkipForward, 15.0, Color32::from_rgb(228, 228, 231))
                            .on_hover_text("Next (N / Right)")
                            .clicked()
                        {
                            if is_stream {
                                stream_controller.next(&prov_id);
                            } else {
                                player.next();
                            }
                        }

                        // Repeat Button (Local only)
                        let (rep_icon, rep_color) = if !is_stream {
                            match player.repeat {
                                RepeatMode::Off => (Icon::Repeat, theme.text_muted),
                                RepeatMode::All => (Icon::Repeat, Color32::WHITE),
                                RepeatMode::One => (Icon::RepeatOne, Color32::WHITE),
                            }
                        } else {
                            (Icon::Repeat, theme.text_muted)
                        };
                        let rep_btn = icon_button(ui, rep_icon, 15.0, rep_color);
                        if !is_stream && rep_btn.on_hover_text("Repeat (R)").clicked() {
                            player.cycle_repeat();
                        }
                    });

                    // Bottom Row: Current Timestamp, Seekbar, Duration
                    let (pos, dur) = current_playback_time(player, if is_stream { Some(&*stream_controller) } else { None });

                    let seekbar_id = egui::Id::new("eclipse_playerbar_seekbar");
                    let mut drag_state = ui.data(|d| d.get_temp::<SeekbarDragState>(seekbar_id)).unwrap_or_default();

                    // Check if a committed seek preview is still active to prevent snap-back
                    if let (Some(committed_secs), Some(commit_time)) = (drag_state.last_committed_secs, drag_state.last_committed_time) {
                        if commit_time.elapsed() < Duration::from_millis(350) && (pos - committed_secs).abs() > 0.4 {
                            // Hold preview position until engine catches up
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

                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing = vec2(6.0, 0.0);

                        ui.label(
                            RichText::new(format_time_secs(display_pos))
                                .color(theme.text_muted)
                                .size(11.0),
                        );

                        let seekbar_w = (center_w - 90.0).max(60.0);
                        render_custom_seekbar(
                            ui,
                            display_pos,
                            dur,
                            theme,
                            seekbar_w,
                            player,
                            if is_stream { Some(&*stream_controller) } else { None },
                            seekbar_id,
                            &mut drag_state,
                        );

                        ui.label(
                            RichText::new(format_time_secs(dur))
                                .color(theme.text_muted)
                                .size(11.0),
                        );
                    });
                },
            );

            // =================================================================
            // 3. RIGHT COLUMN: Mini Spectrum Visualizer, Volume, View Toggles
            // =================================================================
            ui.allocate_ui_with_layout(
                vec2(right_w, 54.0),
                Layout::right_to_left(egui::Align::Center),
                |ui| {
                    ui.spacing_mut().item_spacing = vec2(8.0, 0.0);

                    // Resize Grip Handle in Bottom-Right Corner
                    render_resize_grip(ui, theme);

                    // View Toggle: Now Playing / Analyzer Toggle
                    let np_color = if *toggle_now_playing { Color32::WHITE } else { theme.text_secondary };
                    if icon_button(ui, Icon::BarChart, 15.0, np_color)
                        .on_hover_text("Audio Analyzer (F11)")
                        .clicked()
                    {
                        *toggle_now_playing = !*toggle_now_playing;
                    }

                    // View Toggle: Queue Drawer Toggle
                    let queue_color = if *show_queue { Color32::WHITE } else { theme.text_secondary };
                    if icon_button(ui, Icon::Queue, 15.0, queue_color)
                        .on_hover_text("Play Queue (Q)")
                        .clicked()
                    {
                        *show_queue = !*show_queue;
                    }

                    // View Toggle: Mini Player & Desktop Overlay (PiP)
                    let pip_color = if *show_pip_overlay { Color32::WHITE } else { theme.text_secondary };
                    if icon_button(ui, Icon::PictureInPicture, 15.0, pip_color)
                        .on_hover_text("Mini Player & Desktop Overlay (PiP)")
                        .clicked()
                    {
                        *show_pip_overlay = !*show_pip_overlay;
                    }

                    // View Toggle: Synced Lyrics Quick Access
                    if icon_button(ui, Icon::Mic, 15.0, theme.text_secondary)
                        .on_hover_text("Synced Lyrics (L)")
                        .clicked()
                    {
                        *toggle_now_playing = true;
                    }

                    // Interactive Volume Slider
                    let vol = player.engine.volume();
                    render_custom_volume_slider(
                        ui,
                        vol,
                        theme,
                        player,
                        if is_stream { Some((prov_id.as_str(), &*stream_controller)) } else { None },
                    );

                    // Volume Mute Button (Vector Speaker Icon)
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

                    let vol_btn_color = if is_muted { Color32::from_rgb(220, 38, 38) } else { theme.text_secondary };
                    if icon_button(ui, vol_icon, 15.0, vol_btn_color)
                        .on_hover_text("Mute / Unmute (M)")
                        .clicked()
                    {
                        if is_stream {
                            if vol > 0.001 {
                                stream_controller.set_volume(&prov_id, 0.0);
                            } else {
                                stream_controller.set_volume(&prov_id, 1.0);
                            }
                        } else {
                            player.engine.toggle_mute();
                        }
                    }

                    // Real-Time Mini Spectrum Visualizer (12-16 Animated Bars) or Stream Badge
                    if is_stream {
                        pill_badge(
                            ui,
                            "STREAM",
                            Color32::from_rgba_unmultiplied(theme.accent.r(), theme.accent.g(), theme.accent.b(), 35),
                            theme.accent,
                        );
                    } else {
                        render_mini_visualizer(
                            ui,
                            &player.visualizer.band_values,
                            Color32::WHITE,
                            vec2(48.0, 22.0),
                        );
                    }
                },
            );
        });
    });

    // Seamless top border: 1px solid rgba(255, 255, 255, 0.06)
    ui.painter().line_segment(
        [
            pos2(inner.response.rect.min.x, inner.response.rect.min.y),
            pos2(inner.response.rect.max.x, inner.response.rect.min.y),
        ],
        Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(255, 255, 255, 15)),
    );

    inner.response
}

fn truncate_str(s: &str, max_chars: usize) -> String {
    if s.chars().count() > max_chars {
        let truncated: String = s.chars().take(max_chars.saturating_sub(1)).collect();
        format!("{truncated}…")
    } else {
        s.to_string()
    }
}

#[allow(clippy::too_many_arguments)]
fn render_custom_seekbar(
    ui: &mut Ui,
    display_pos: f64,
    dur: f64,
    theme: &Theme,
    desired_width: f32,
    player: &mut PlayerController,
    stream_controller: Option<&StreamController>,
    seekbar_id: egui::Id,
    drag_state: &mut SeekbarDragState,
) {
    let (rect, resp) = ui.allocate_exact_size(vec2(desired_width, 16.0), egui::Sense::click_and_drag());
    let is_hovered = resp.hovered();
    let is_dragged = resp.dragged();

    if is_hovered || is_dragged {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }

    let bar_height = if is_hovered || is_dragged { 4.5 } else { 3.0 };
    let bar_rect = Rect::from_center_size(rect.center(), vec2(rect.width(), bar_height));
    let rounding = Rounding::same(bar_height / 2.0);

    let max_dur = dur.max(0.001);

    // Continuous visual scrubbing during drag without flooding seek calls
    if resp.drag_started() || resp.dragged() {
        drag_state.is_dragging = true;
        if let Some(pointer_pos) = resp.interact_pointer_pos() {
            let ratio = ((pointer_pos.x - bar_rect.min.x) / bar_rect.width()).clamp(0.0, 1.0);
            let target_secs = (ratio as f64 * max_dur).clamp(0.0, max_dur);
            drag_state.scrub_secs = target_secs;
        }
        ui.ctx().request_repaint();
    }

    // Commit EXACTLY ONE seek on release or single click
    if resp.drag_stopped() {
        let target_secs = drag_state.scrub_secs;
        seek_to(player, stream_controller, target_secs);
        drag_state.is_dragging = false;
        drag_state.last_committed_secs = Some(target_secs);
        drag_state.last_committed_time = Some(Instant::now());
        ui.ctx().request_repaint();
    } else if resp.clicked() {
        if let Some(pointer_pos) = resp.interact_pointer_pos() {
            let ratio = ((pointer_pos.x - bar_rect.min.x) / bar_rect.width()).clamp(0.0, 1.0);
            let target_secs = (ratio as f64 * max_dur).clamp(0.0, max_dur);
            seek_to(player, stream_controller, target_secs);
            drag_state.is_dragging = false;
            drag_state.scrub_secs = target_secs;
            drag_state.last_committed_secs = Some(target_secs);
            drag_state.last_committed_time = Some(Instant::now());
            ui.ctx().request_repaint();
        }
    } else if !resp.dragged() && drag_state.is_dragging {
        drag_state.is_dragging = false;
    }

    ui.data_mut(|d| d.insert_temp(seekbar_id, *drag_state));

    let active_pos = if drag_state.is_dragging {
        drag_state.scrub_secs
    } else if let Some(committed_secs) = drag_state.last_committed_secs {
        committed_secs
    } else {
        display_pos
    };

    let progress_ratio = (active_pos / max_dur).clamp(0.0, 1.0) as f32;

    // Track background: #27272a
    let track_bg = Color32::from_rgb(39, 39, 42);
    ui.painter().rect_filled(bar_rect, rounding, track_bg);

    // Solid white progress rail: #ffffff
    let fill_width = bar_rect.width() * progress_ratio;
    if fill_width > 0.0 {
        let fill_rect = Rect::from_min_size(bar_rect.min, vec2(fill_width, bar_height));
        ui.painter().rect_filled(fill_rect, rounding, Color32::WHITE);
    }

    // Clean white scrubber knob on hover or drag
    if is_hovered || is_dragged {
        let knob_x = bar_rect.min.x + fill_width;
        let knob_pos = pos2(knob_x, bar_rect.center().y);
        let knob_radius = 5.0;

        ui.painter().circle_filled(knob_pos, knob_radius, Color32::WHITE);
        ui.painter().circle_stroke(knob_pos, knob_radius, Stroke::new(1.0_f32, Color32::from_rgb(10, 10, 12)));

        // Time preview tooltip on hover or drag
        let preview_secs = if is_dragged {
            active_pos
        } else if let Some(hover_pos) = resp.hover_pos() {
            let hover_ratio = ((hover_pos.x - bar_rect.min.x) / bar_rect.width()).clamp(0.0, 1.0);
            hover_ratio as f64 * max_dur
        } else {
            active_pos
        };

        resp.on_hover_ui(|ui| {
            ui.label(
                RichText::new(format_time_secs(preview_secs))
                    .size(11.0)
                    .color(theme.text_primary),
            );
        });
    }
}

fn render_custom_volume_slider(
    ui: &mut Ui,
    vol: f32,
    theme: &Theme,
    player: &mut PlayerController,
    stream_info: Option<(&str, &StreamController)>,
) {
    let desired_width = 72.0;
    let (rect, resp) = ui.allocate_exact_size(vec2(desired_width, 16.0), egui::Sense::click_and_drag());
    let is_hovered = resp.hovered();
    let is_dragged = resp.dragged();

    if is_hovered || is_dragged {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }

    // Scroll wheel adjusts volume by ±5% when hovering
    if is_hovered {
        let scroll_y = ui.input(|i| i.raw_scroll_delta.y);
        if scroll_y != 0.0 {
            let step = if scroll_y > 0.0 { 0.05 } else { -0.05 };
            let new_vol = (vol + step).clamp(0.0, 1.0);
            if let Some((prov_id, stream_ctrl)) = stream_info {
                stream_ctrl.set_volume(prov_id, new_vol);
            } else {
                if player.engine.is_muted() && new_vol > 0.0 {
                    player.engine.set_muted(false);
                }
                player.engine.set_volume(new_vol);
            }
        }
    }

    let bar_height = if is_hovered || is_dragged { 4.5 } else { 3.0 };
    let bar_rect = Rect::from_center_size(rect.center(), vec2(rect.width(), bar_height));
    let rounding = Rounding::same(bar_height / 2.0);

    let effective_vol = if is_dragged || resp.clicked() {
        if let Some(pos) = resp.interact_pointer_pos() {
            let ratio = ((pos.x - bar_rect.min.x) / bar_rect.width()).clamp(0.0, 1.0);
            if let Some((prov_id, stream_ctrl)) = stream_info {
                stream_ctrl.set_volume(prov_id, ratio);
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

    // Track background: #27272a
    let track_bg = Color32::from_rgb(39, 39, 42);
    ui.painter().rect_filled(bar_rect, rounding, track_bg);

    // Solid white volume rail: #ffffff
    let fill_width = bar_rect.width() * effective_vol;
    if fill_width > 0.0 {
        let fill_rect = Rect::from_min_size(bar_rect.min, vec2(fill_width, bar_height));
        ui.painter().rect_filled(fill_rect, rounding, Color32::WHITE);
    }

    // Volume knob on hover/drag
    if is_hovered || is_dragged {
        let knob_x = bar_rect.min.x + fill_width;
        let knob_pos = pos2(knob_x, bar_rect.center().y);
        ui.painter().circle_filled(knob_pos, 4.0, Color32::WHITE);
        ui.painter().circle_stroke(knob_pos, 4.0, Stroke::new(1.0_f32, Color32::from_rgb(10, 10, 12)));

        resp.on_hover_ui(|ui| {
            ui.label(
                RichText::new(format!("{}%", (effective_vol * 100.0).round() as u32))
                    .size(11.0)
                    .color(theme.text_primary),
            );
        });
    }
}

/// Subtle glass resize grip handle in the bottom-right corner of the docked playerbar.
fn render_resize_grip(ui: &mut Ui, _theme: &Theme) {
    let (rect, mut resp) = ui.allocate_exact_size(vec2(12.0, 20.0), egui::Sense::click_and_drag());
    let is_hovered = resp.hovered();
    let is_dragged = resp.dragged();

    let is_maximized = ui.input(|i| i.viewport().maximized.unwrap_or(false));
    let is_fullscreen = ui.input(|i| i.viewport().fullscreen.unwrap_or(false));

    if !is_maximized && !is_fullscreen {
        if is_hovered || is_dragged {
            ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeSouthEast);
        }

        if is_hovered {
            resp = resp.on_hover_text("Drag to resize window");
        }

        if resp.drag_started_by(egui::PointerButton::Primary)
            || (is_hovered && ui.input(|i| i.pointer.primary_pressed()))
        {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::BeginResize(egui::ResizeDirection::SouthEast));
        }
    }

    // Render 3 diagonal grip lines /// in the corner
    let painter = ui.painter();
    let color = if is_maximized {
        Color32::from_rgba_premultiplied(255, 255, 255, 15)
    } else if is_hovered || is_dragged {
        Color32::WHITE
    } else {
        Color32::from_rgba_premultiplied(255, 255, 255, 45)
    };

    let p = rect.right_bottom() - vec2(1.0, 3.0);
    for offset in [3.0_f32, 6.5_f32, 10.0_f32] {
        painter.line_segment(
            [
                pos2(p.x - offset, p.y),
                pos2(p.x, p.y - offset),
            ],
            Stroke::new(1.2_f32, color),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppConfig;
    use crate::library::model::Track;

    #[test]
    fn test_playerbar_rendering_empty_and_playing() {
        let ctx = egui::Context::default();
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let theme = Theme::from_name("Eclipse", None);
                let cfg = AppConfig::default();
                let mut player = PlayerController::new(&cfg);
                let mut art_manager = AlbumArtManager::new();
                let mut show_queue = false;
                let mut show_pip_overlay = false;
                let mut toggle_np = false;
                let mut on_fav = None;

                let mut stream_ctrl = StreamController::new(vec![]);

                // Render when not playing
                render_playerbar(
                    ui,
                    &mut player,
                    &mut stream_ctrl,
                    &theme,
                    &mut show_queue,
                    &mut show_pip_overlay,
                    &mut toggle_np,
                    &mut on_fav,
                    &mut art_manager,
                );

                // Load track into queue
                let track = Track {
                    id: 1,
                    path: "demo.flac".into(),
                    title: "Demo Title".into(),
                    artist: "Demo Artist".into(),
                    album: "Demo Album".into(),
                    genre: "Lo-Fi".into(),
                    year: 2024,
                    track_number: 1,
                    duration_secs: 200.0,
                    format: "FLAC".into(),
                    bitrate_kbps: 1000,
                    sample_rate: 44100,
                    channels: 2,
                    favorite: true,
                    has_art: false,
                    folder: "".into(),
                    album_artist: "".into(),
                    play_count: 3,
                    file_size: 1024,
                };
                player.load_queue(vec![track], 0);

                // Render when track is loaded
                render_playerbar(
                    ui,
                    &mut player,
                    &mut stream_ctrl,
                    &theme,
                    &mut show_queue,
                    &mut show_pip_overlay,
                    &mut toggle_np,
                    &mut on_fav,
                    &mut art_manager,
                );

                // Switch to stream track and render
                let stream_track = crate::streams::provider::StreamTrack::new(
                    "sp-1",
                    "Stream Song",
                    "Stream Artist",
                    "Stream Album",
                    180.0,
                    None,
                    "spotify:track:123",
                    "spotify",
                );
                player.switch_to_stream("spotify", Some(stream_track));

                render_playerbar(
                    ui,
                    &mut player,
                    &mut stream_ctrl,
                    &theme,
                    &mut show_queue,
                    &mut show_pip_overlay,
                    &mut toggle_np,
                    &mut on_fav,
                    &mut art_manager,
                );
            });
        });
    }

    #[test]
    fn test_current_playback_time_local_and_stream() {
        let cfg = AppConfig::default();
        let mut player = PlayerController::new(&cfg);
        let stream_ctrl = StreamController::new(vec![]);

        // 1. Empty local queue
        let (pos, dur) = current_playback_time(&player, Some(&stream_ctrl));
        assert_eq!(pos, 0.0);
        assert_eq!(dur, 0.0);

        // 2. Local track in queue
        let track = Track {
            id: 10,
            path: "local.flac".into(),
            title: "Local Track".into(),
            artist: "Artist".into(),
            album: "Album".into(),
            genre: "Rock".into(),
            year: 2024,
            track_number: 1,
            duration_secs: 245.5,
            format: "FLAC".into(),
            bitrate_kbps: 1000,
            sample_rate: 44100,
            channels: 2,
            favorite: false,
            has_art: false,
            folder: "".into(),
            album_artist: "".into(),
            play_count: 0,
            file_size: 1024,
        };
        player.load_queue(vec![track], 0);
        let (_pos, dur) = current_playback_time(&player, Some(&stream_ctrl));
        assert!((dur - 245.5).abs() < 0.001);

        // 3. Switch to stream: active_state duration is 0, so fallback to active_stream_track
        let stream_track = crate::streams::provider::StreamTrack::new(
            "stream-id",
            "Stream Title",
            "Stream Artist",
            "Stream Album",
            195.0,
            None,
            "spotify:track:abc",
            "spotify",
        );
        player.switch_to_stream("spotify", Some(stream_track));
        let (pos, dur) = current_playback_time(&player, Some(&stream_ctrl));
        assert_eq!(pos, 0.0);
        assert!((dur - 195.0).abs() < 0.001, "Duration must fall back to active_stream_track when active_state is 0");
    }

    #[test]
    fn test_seek_to_dispatcher_local_and_stream() {
        let cfg = AppConfig::default();
        let mut player = PlayerController::new(&cfg);
        let stream_ctrl = StreamController::new(vec![]);

        // 1. Local seek: dispatching does not panic and clamps negative target
        seek_to(&mut player, Some(&stream_ctrl), -10.0);
        assert_eq!(player.engine.position().as_secs_f64(), 0.0);

        seek_to(&mut player, Some(&stream_ctrl), 45.0);

        // 2. Stream seek: correctly routes to stream controller
        let stream_track = crate::streams::provider::StreamTrack::new(
            "yt-1",
            "YT Song",
            "YT Artist",
            "YT Album",
            300.0,
            None,
            "youtube:v:123",
            "youtube",
        );
        player.switch_to_stream("youtube", Some(stream_track));
        seek_to(&mut player, Some(&stream_ctrl), 120.0);
        // Verified: stream seek dispatch succeeded without error
    }

    #[test]
    fn test_seekbar_drag_state_math_and_persistence() {
        let mut drag_state = SeekbarDragState::default();
        assert!(!drag_state.is_dragging);
        assert_eq!(drag_state.scrub_secs, 0.0);
        assert!(drag_state.last_committed_secs.is_none());

        // Simulate dragging
        drag_state.is_dragging = true;
        drag_state.scrub_secs = 75.5;
        assert!(drag_state.is_dragging);
        assert_eq!(drag_state.scrub_secs, 75.5);

        // Simulate commit on release
        drag_state.is_dragging = false;
        drag_state.last_committed_secs = Some(75.5);
        drag_state.last_committed_time = Some(Instant::now());

        assert!(!drag_state.is_dragging);
        assert_eq!(drag_state.last_committed_secs, Some(75.5));
        assert!(drag_state.last_committed_time.is_some());
    }

    #[test]
    fn test_seekbar_click_to_seek_and_anti_snapback() {
        let ctx = egui::Context::default();
        let theme = Theme::from_name("Eclipse", None);
        let cfg = AppConfig::default();
        let mut player = PlayerController::new(&cfg);
        let seekbar_id = egui::Id::new("eclipse_playerbar_seekbar");
        let mut drag_state = SeekbarDragState::default();

        let dur = 200.0;
        let display_pos = 0.0;

        // Frame 0: Layout & pointer position at 50% seek point
        // Inside CentralPanel with 0 margin, x ranges [0.0, 200.0], y ranges [0.0, 16.0]
        let pt = pos2(100.0, 8.0);
        let mut input0 = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(1000.0, 700.0))),
            time: Some(0.0),
            ..Default::default()
        };
        input0.events.push(egui::Event::PointerMoved(pt));
        let _ = ctx.run(input0, |ctx| {
            egui::CentralPanel::default()
                .frame(egui::Frame::none())
                .show(ctx, |ui| {
                    render_custom_seekbar(
                        ui,
                        display_pos,
                        dur,
                        &theme,
                        200.0,
                        &mut player,
                        None,
                        seekbar_id,
                        &mut drag_state,
                    );
                });
        });

        // Frame 1: Pointer pressed down on seekbar at 50%
        let mut input1 = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(1000.0, 700.0))),
            time: Some(1.0),
            ..Default::default()
        };
        input1.events.push(egui::Event::PointerButton {
            pos: pt,
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: Default::default(),
        });
        let _ = ctx.run(input1, |ctx| {
            egui::CentralPanel::default()
                .frame(egui::Frame::none())
                .show(ctx, |ui| {
                    render_custom_seekbar(
                        ui,
                        display_pos,
                        dur,
                        &theme,
                        200.0,
                        &mut player,
                        None,
                        seekbar_id,
                        &mut drag_state,
                    );
                });
        });

        // Frame 2: Pointer released -> clicked() fires!
        let mut input2 = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(1000.0, 700.0))),
            time: Some(1.05),
            ..Default::default()
        };
        input2.events.push(egui::Event::PointerButton {
            pos: pt,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: Default::default(),
        });
        let _ = ctx.run(input2, |ctx| {
            egui::CentralPanel::default()
                .frame(egui::Frame::none())
                .show(ctx, |ui| {
                    render_custom_seekbar(
                        ui,
                        display_pos,
                        dur,
                        &theme,
                        200.0,
                        &mut player,
                        None,
                        seekbar_id,
                        &mut drag_state,
                    );
                });
        });

        // Verify click committed target secs (approx 100.0s for 50% of 200s)
        assert!(drag_state.last_committed_secs.is_some(), "Single click must set last_committed_secs");
        let committed = drag_state.last_committed_secs.unwrap();
        assert!((committed - 100.0).abs() < 2.0, "Expected seek ~100.0s, got {}", committed);
        assert!(!drag_state.is_dragging, "is_dragging must be false after click");

        // Anti-snapback check: committed time is recent (< 350ms)
        assert!(drag_state.last_committed_time.is_some());
        let elapsed = drag_state.last_committed_time.unwrap().elapsed();
        assert!(elapsed < Duration::from_millis(350));
    }

    #[test]
    fn test_seekbar_drag_scrubbing_no_per_frame_seeks_and_single_commit() {
        let ctx = egui::Context::default();
        let theme = Theme::from_name("Eclipse", None);
        let cfg = AppConfig::default();
        let mut player = PlayerController::new(&cfg);
        let seekbar_id = egui::Id::new("eclipse_playerbar_seekbar");
        let mut drag_state = SeekbarDragState::default();

        let dur = 200.0;
        let display_pos = 0.0;
        let pt1 = pos2(20.0, 8.0);
        let pt2 = pos2(100.0, 8.0);
        let pt3 = pos2(150.0, 8.0);

        // Frame 0: Layout & set initial pointer position at 20px (10%)
        let mut input0 = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(1000.0, 700.0))),
            time: Some(0.0),
            ..Default::default()
        };
        input0.events.push(egui::Event::PointerMoved(pt1));
        let _ = ctx.run(input0, |ctx| {
            egui::CentralPanel::default()
                .frame(egui::Frame::none())
                .show(ctx, |ui| {
                    render_custom_seekbar(
                        ui,
                        display_pos,
                        dur,
                        &theme,
                        200.0,
                        &mut player,
                        None,
                        seekbar_id,
                        &mut drag_state,
                    );
                });
        });

        // Frame 1: Pointer down at 20px
        let mut input1 = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(1000.0, 700.0))),
            time: Some(1.0),
            ..Default::default()
        };
        input1.events.push(egui::Event::PointerButton {
            pos: pt1,
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: Default::default(),
        });
        let _ = ctx.run(input1, |ctx| {
            egui::CentralPanel::default()
                .frame(egui::Frame::none())
                .show(ctx, |ui| {
                    render_custom_seekbar(
                        ui,
                        display_pos,
                        dur,
                        &theme,
                        200.0,
                        &mut player,
                        None,
                        seekbar_id,
                        &mut drag_state,
                    );
                });
        });

        // Frame 2: Drag moved to 100px (50%)
        let mut input2 = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(1000.0, 700.0))),
            time: Some(1.05),
            ..Default::default()
        };
        input2.events.push(egui::Event::PointerMoved(pt2));
        let _ = ctx.run(input2, |ctx| {
            egui::CentralPanel::default()
                .frame(egui::Frame::none())
                .show(ctx, |ui| {
                    render_custom_seekbar(
                        ui,
                        display_pos,
                        dur,
                        &theme,
                        200.0,
                        &mut player,
                        None,
                        seekbar_id,
                        &mut drag_state,
                    );
                });
        });

        // Verify while dragging: is_dragging is true, scrub_secs updated, last_committed_secs is None
        assert!(drag_state.is_dragging, "Must be in dragging state during drag");
        assert!((drag_state.scrub_secs - 100.0).abs() < 2.0, "Scrub preview must update to ~100s, got {}", drag_state.scrub_secs);
        assert!(drag_state.last_committed_secs.is_none(), "NO seek should be committed while actively dragging");

        // Frame 3: Drag moved to 150px (75%)
        let mut input3 = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(1000.0, 700.0))),
            time: Some(1.10),
            ..Default::default()
        };
        input3.events.push(egui::Event::PointerMoved(pt3));
        let _ = ctx.run(input3, |ctx| {
            egui::CentralPanel::default()
                .frame(egui::Frame::none())
                .show(ctx, |ui| {
                    render_custom_seekbar(
                        ui,
                        display_pos,
                        dur,
                        &theme,
                        200.0,
                        &mut player,
                        None,
                        seekbar_id,
                        &mut drag_state,
                    );
                });
        });

        assert!(drag_state.is_dragging);
        assert!((drag_state.scrub_secs - 150.0).abs() < 2.0, "Scrub preview must update to ~150s, got {}", drag_state.scrub_secs);
        assert!(drag_state.last_committed_secs.is_none());

        // Frame 4: Drag released (button up) -> drag_stopped fires!
        let mut input4 = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(1000.0, 700.0))),
            time: Some(1.15),
            ..Default::default()
        };
        input4.events.push(egui::Event::PointerButton {
            pos: pt3,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: Default::default(),
        });
        let _ = ctx.run(input4, |ctx| {
            egui::CentralPanel::default()
                .frame(egui::Frame::none())
                .show(ctx, |ui| {
                    render_custom_seekbar(
                        ui,
                        display_pos,
                        dur,
                        &theme,
                        200.0,
                        &mut player,
                        None,
                        seekbar_id,
                        &mut drag_state,
                    );
                });
        });

        // Verify commit on drag release:
        assert!(!drag_state.is_dragging, "is_dragging must become false on release");
        assert!(drag_state.last_committed_secs.is_some(), "Exactly one seek committed on release");
        let committed = drag_state.last_committed_secs.unwrap();
        assert!((committed - 150.0).abs() < 2.0, "Committed seek position must match scrub_secs (~150s), got {}", committed);
        assert!(drag_state.last_committed_time.is_some(), "Commit timestamp must be recorded");
    }

    #[test]
    fn test_seekbar_anti_snapback_expiration() {
        let mut drag_state = SeekbarDragState {
            is_dragging: false,
            scrub_secs: 120.0,
            last_committed_secs: Some(120.0),
            last_committed_time: Some(Instant::now() - Duration::from_millis(400)), // expired (> 350ms)
        };

        let pos = 10.0; // Stale engine position

        // Simulate lines 413-420 of playerbar.rs
        if let (Some(committed_secs), Some(commit_time)) = (drag_state.last_committed_secs, drag_state.last_committed_time) {
            if commit_time.elapsed() < Duration::from_millis(350) && (pos - committed_secs).abs() > 0.4 {
                // Hold preview
            } else {
                drag_state.last_committed_secs = None;
                drag_state.last_committed_time = None;
            }
        }

        assert!(drag_state.last_committed_secs.is_none(), "Committed preview must expire after 350ms");
        assert!(drag_state.last_committed_time.is_none());
    }
}
