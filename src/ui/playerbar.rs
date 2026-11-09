use crate::audio::PlayerController;
use crate::config::RepeatMode;
use crate::library::format_time_secs;
use crate::ui::art::AlbumArtManager;
use crate::ui::components::{format_badge, render_mini_visualizer};
use crate::ui::icons::{icon_button, paint_icon, Icon};
use crate::ui::theme::Theme;
use egui::{pos2, vec2, Color32, Layout, Rect, RichText, Rounding, Stroke, Ui};
use std::time::Duration;

#[allow(clippy::too_many_arguments)]
pub fn render_playerbar(
    ui: &mut Ui,
    player: &mut PlayerController,
    theme: &Theme,
    show_queue: &mut bool,
    show_pip_overlay: &mut bool,
    toggle_now_playing: &mut bool,
    on_toggle_fav: &mut Option<i64>,
    art_manager: &mut AlbumArtManager,
) -> egui::Response {
    let frame = egui::Frame::none()
        .fill(Color32::from_rgb(16, 16, 19)) // #101013
        .inner_margin(egui::Margin::symmetric(20.0, 8.0)); // padding: 0 20px

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
                    if let Some(track) = player.current_track() {
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
                vec2(center_w, 54.0),
                Layout::top_down(egui::Align::Center),
                |ui| {
                    ui.spacing_mut().item_spacing = vec2(0.0, 3.0);

                    // Top Row: Transport Buttons
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing = vec2(12.0, 0.0);

                        // Shuffle Button
                        let shuffle_color = if player.shuffle { Color32::WHITE } else { theme.text_muted };
                        if icon_button(ui, Icon::Shuffle, 15.0, shuffle_color)
                            .on_hover_text("Shuffle (S)")
                            .clicked()
                        {
                            player.toggle_shuffle();
                        }

                        // Previous Button
                        if icon_button(ui, Icon::SkipBack, 15.0, Color32::from_rgb(228, 228, 231))
                            .on_hover_text("Previous (B / Left)")
                            .clicked()
                        {
                            player.prev();
                        }

                        // Big Circular Solid White Play / Pause Button (40x40px)
                        let is_playing = player.engine.is_playing();
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
                            if player.current_track().is_some() {
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
                            player.next();
                        }

                        // Repeat Button
                        let (rep_icon, rep_color) = match player.repeat {
                            RepeatMode::Off => (Icon::Repeat, theme.text_muted),
                            RepeatMode::All => (Icon::Repeat, Color32::WHITE),
                            RepeatMode::One => (Icon::RepeatOne, Color32::WHITE),
                        };
                        if icon_button(ui, rep_icon, 15.0, rep_color)
                            .on_hover_text("Repeat (R)")
                            .clicked()
                        {
                            player.cycle_repeat();
                        }
                    });

                    // Bottom Row: Current Timestamp, Seekbar, Duration
                    let pos = player.engine.position().as_secs_f64();
                    let dur = player
                        .current_track()
                        .map(|t| t.duration_secs)
                        .unwrap_or(0.0);

                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing = vec2(6.0, 0.0);

                        ui.label(
                            RichText::new(format_time_secs(pos))
                                .color(theme.text_muted)
                                .size(11.0),
                        );

                        let seekbar_w = (center_w - 90.0).max(60.0);
                        render_custom_seekbar(ui, pos, dur, theme, seekbar_w, player);

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
                    render_custom_volume_slider(ui, vol, theme, player);

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
                        player.engine.toggle_mute();
                    }

                    // Real-Time Mini Spectrum Visualizer (12-16 Animated Bars)
                    render_mini_visualizer(
                        ui,
                        &player.visualizer.band_values,
                        Color32::WHITE,
                        vec2(48.0, 22.0),
                    );
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

fn render_custom_seekbar(
    ui: &mut Ui,
    pos: f64,
    dur: f64,
    theme: &Theme,
    desired_width: f32,
    player: &mut PlayerController,
) {
    let (rect, resp) = ui.allocate_exact_size(vec2(desired_width, 18.0), egui::Sense::click_and_drag());
    let is_hovered = resp.hovered();
    let is_dragged = resp.dragged();

    if is_hovered || is_dragged {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }

    let bar_height = if is_hovered || is_dragged { 4.5 } else { 3.0 };
    let bar_rect = Rect::from_center_size(rect.center(), vec2(rect.width(), bar_height));
    let rounding = Rounding::same(bar_height / 2.0);

    let max_dur = dur.max(0.001);

    // Continuous scrubbing and direct single-click seeking
    let display_pos = if is_dragged || resp.clicked() {
        if let Some(pointer_pos) = resp.interact_pointer_pos() {
            let ratio = ((pointer_pos.x - bar_rect.min.x) / bar_rect.width()).clamp(0.0, 1.0);
            let target_secs = ratio as f64 * max_dur;
            player.engine.seek(Duration::from_secs_f64(target_secs));
            target_secs
        } else {
            pos
        }
    } else {
        pos
    };

    let progress_ratio = (display_pos / max_dur).clamp(0.0, 1.0) as f32;

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

        // Time preview tooltip on hover
        if let Some(hover_pos) = resp.hover_pos() {
            let hover_ratio = ((hover_pos.x - bar_rect.min.x) / bar_rect.width()).clamp(0.0, 1.0);
            let hover_secs = hover_ratio as f64 * max_dur;
            resp.on_hover_ui(|ui| {
                ui.label(
                    RichText::new(format_time_secs(hover_secs))
                        .size(11.0)
                        .color(theme.text_primary),
                );
            });
        }
    }
}

fn render_custom_volume_slider(
    ui: &mut Ui,
    vol: f32,
    theme: &Theme,
    player: &mut PlayerController,
) {
    let desired_width = 72.0;
    let (rect, resp) = ui.allocate_exact_size(vec2(desired_width, 18.0), egui::Sense::click_and_drag());
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
            if player.engine.is_muted() && new_vol > 0.0 {
                player.engine.set_muted(false);
            }
            player.engine.set_volume(new_vol);
        }
    }

    let bar_height = if is_hovered || is_dragged { 4.5 } else { 3.0 };
    let bar_rect = Rect::from_center_size(rect.center(), vec2(rect.width(), bar_height));
    let rounding = Rounding::same(bar_height / 2.0);

    let effective_vol = if is_dragged || resp.clicked() {
        if let Some(pos) = resp.interact_pointer_pos() {
            let ratio = ((pos.x - bar_rect.min.x) / bar_rect.width()).clamp(0.0, 1.0);
            if player.engine.is_muted() && ratio > 0.0 {
                player.engine.set_muted(false);
            }
            player.engine.set_volume(ratio);
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

                // Render when not playing
                render_playerbar(
                    ui,
                    &mut player,
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
}
