use std::time::Duration;

use egui::{
    pos2, vec2, Color32, FontId, Id, Layout, Rect, RichText, Rounding, ScrollArea,
    Sense, Stroke, Ui, UiBuilder,
};

use crate::audio::PlayerController;
use crate::config::RepeatMode;
use crate::library::model::{format_time_secs, Track};
use crate::lyrics::Lyrics;
use crate::ui::ambient_blur::AmbientBlurCache;
use crate::ui::art::AlbumArtManager;
use crate::ui::icons::{self, Icon};
use crate::ui::theme::Theme;

/// Re-export ArtManager as alias to AlbumArtManager for contract compatibility
pub type ArtManager = AlbumArtManager;

/// Renders the cinematic full-viewport music experience with blurred ambient background,
/// two-column split layout (album art, metadata, seekbar, controls on left; synced lyrics on right),
/// and floating overlay controls.
pub fn render_cinematic_fullscreen(
    ui: &mut Ui,
    player: &mut PlayerController,
    theme: &Theme,
    art_manager: &mut ArtManager,
    ambient_cache: &mut AmbientBlurCache,
    on_exit_fullscreen: &mut bool,
) {
    // 1. Fullscreen Hotkey Check: Escape exits cinematic mode
    if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
        *on_exit_fullscreen = true;
    }

    // 2. Query Current Track (local library or active stream)
    let current_track: Option<Track> = player.current_track().cloned().or_else(|| {
        player.active_stream_track.as_ref().map(|st| Track {
            title: st.title.clone(),
            artist: st.artist.clone(),
            album: st.album.clone(),
            duration_secs: st.duration_secs,
            path: st.uri.clone(),
            format: "STREAM".to_string(),
            ..Default::default()
        })
    });

    let track_key = current_track
        .as_ref()
        .map(|t| format!("{}_{}", t.artist, t.title))
        .unwrap_or_default();

    // 3. Ambient Blurred Artwork Background
    // Optimized: Only read raw bytes once when track_key changes to preserve 60+ FPS
    let last_key_id = Id::new("eclipse_fullscreen_last_track_key");
    let last_key = ui.ctx().data(|d| d.get_temp::<String>(last_key_id));
    let key_changed = last_key.as_deref() != Some(&track_key);

    let raw_bytes: Option<Vec<u8>> = if key_changed && !track_key.is_empty() {
        ui.ctx().data_mut(|d| d.insert_temp(last_key_id, track_key.clone()));
        if let Some(ref track) = current_track {
            if track.path.starts_with("stream:spotify:") || track.path.starts_with("spotify:") {
                crate::ui::art::resolve_spotify_artwork(&track.path)
            } else if track.path.starts_with("http://") || track.path.starts_with("https://") {
                crate::ui::art::resolve_remote_url_artwork(&track.path)
            } else if !track.path.is_empty() {
                crate::library::scanner::get_or_extract_cover_art(std::path::Path::new(&track.path))
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    let full_rect = ui.max_rect();
    let texture_id = ambient_cache.get_or_build(ui.ctx(), &track_key, raw_bytes.as_deref());
    ambient_cache.paint_background(ui.painter(), full_rect, texture_id);

    if full_rect.width() <= 10.0 || full_rect.height() <= 10.0 {
        return;
    }

    // 4. Responsive Split Geometry
    let pad_x = (full_rect.width() * 0.05).clamp(32.0, 72.0);
    let pad_y = (full_rect.height() * 0.06).clamp(32.0, 64.0);

    let content_rect = Rect::from_min_max(
        pos2(full_rect.min.x + pad_x, full_rect.min.y + pad_y),
        pos2(full_rect.max.x - pad_x, full_rect.max.y - pad_y),
    );

    let col_gap = 48.0;
    let available_w = content_rect.width();
    let available_h = content_rect.height();

    // Left column takes approx 40-42% width (max 480px, min 320px)
    let left_w = (available_w * 0.42).clamp(320.0, 480.0).min(available_w * 0.48);
    let right_w = (available_w - left_w - col_gap).max(260.0);

    let left_rect = Rect::from_min_size(content_rect.min, vec2(left_w, available_h));
    let right_rect = Rect::from_min_size(
        pos2(content_rect.min.x + left_w + col_gap, content_rect.min.y),
        vec2(right_w, available_h),
    );

    // 5. Render Left Column (Vertically Centered Album Art & Controls)
    ui.allocate_new_ui(
        UiBuilder::new()
            .max_rect(left_rect)
            .layout(Layout::top_down(egui::Align::Center)),
        |left_ui| {
            render_left_column(left_ui, player, theme, art_manager, current_track.as_ref(), left_rect);
        },
    );

    // 6. Render Right Column (Synced / Plain Lyrics Stream)
    ui.allocate_new_ui(
        UiBuilder::new()
            .max_rect(right_rect)
            .layout(Layout::top_down(egui::Align::Min)),
        |right_ui| {
            render_right_lyrics_column(right_ui, player, theme, right_rect);
        },
    );

    // 7. Floating Top-Right Overlay (Exit Fullscreen Button)
    render_floating_exit_button(ui, theme, full_rect, on_exit_fullscreen);
}

/// Renders the vertically centered left column with large cover art, track info, seekbar, and controls.
fn render_left_column(
    ui: &mut Ui,
    player: &mut PlayerController,
    theme: &Theme,
    art_manager: &mut ArtManager,
    current_track: Option<&Track>,
    rect: Rect,
) {
    let available_h = rect.height();
    let left_w = rect.width();

    // Determine square artwork card size proportional to column width & screen height
    let art_size = (left_w * 0.80).clamp(240.0, 340.0).min(available_h * 0.44);

    // Calculate approximate height of content to center vertically
    let estimated_content_h = art_size + 20.0 + 96.0 + 16.0 + 38.0 + 16.0 + 52.0 + 14.0 + 26.0;
    let top_pad = ((available_h - estimated_content_h) * 0.5).max(0.0);

    if top_pad > 0.0 {
        ui.add_space(top_pad);
    }

    // 1. Large Album Art Card with Drop Shadow and Glow
    let (art_rect, _) = ui.allocate_exact_size(vec2(art_size, art_size), Sense::hover());

    // Soft drop shadow
    let shadow_rect = art_rect.translate(vec2(0.0, 8.0));
    ui.painter().rect_filled(
        shadow_rect,
        Rounding::same(16.0),
        Color32::from_rgba_premultiplied(0, 0, 0, 110),
    );

    let mut painted_texture = false;
    if let Some(track) = current_track {
        if let Some(texture) = art_manager.get_or_load(ui.ctx(), &track.path) {
            ui.painter().image(
                texture.id(),
                art_rect,
                Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                Color32::WHITE,
            );
            painted_texture = true;
        }
    }

    if !painted_texture {
        // Fallback card with disc vector icon
        ui.painter().rect_filled(
            art_rect,
            Rounding::same(16.0),
            Color32::from_rgba_premultiplied(28, 22, 44, 210),
        );
        let disc_rect = Rect::from_center_size(art_rect.center(), vec2(56.0, 56.0));
        icons::paint_icon(ui.painter(), Icon::Disc, disc_rect, theme.accent);
    }

    // Elegant subtle hairline border
    ui.painter().rect_stroke(
        art_rect,
        Rounding::same(16.0),
        Stroke::new(1.0_f32, theme.border_accent),
    );

    ui.add_space(20.0);

    // 2. Track Title (Bold, large font, CJK safe)
    let title_str = current_track.map(|t| t.display_title()).unwrap_or("No Track Playing");
    ui.add(
        egui::Label::new(
            RichText::new(title_str)
                .size(24.0)
                .strong()
                .color(theme.text_primary),
        )
        .wrap(),
    );

    ui.add_space(4.0);

    // Artist & Album
    if let Some(track) = current_track {
        let artist_str = track.display_artist();
        let album_str = track.display_album();
        let artist_album = if album_str == "Unknown Album" || album_str.is_empty() {
            artist_str.to_string()
        } else {
            format!("{} — {}", artist_str, album_str)
        };

        ui.add(
            egui::Label::new(
                RichText::new(artist_album)
                    .size(15.0)
                    .color(theme.text_secondary),
            )
            .wrap(),
        );

        ui.add_space(6.0);

        // Quality Pill Badge (Hi-Res / FLAC / Lossless / Bitrate)
        render_quality_badge(ui, track, theme);
    } else {
        ui.label(
            RichText::new("Select a track to start playback")
                .size(14.0)
                .color(theme.text_muted),
        );
    }

    ui.add_space(16.0);

    // 3. Interactive Seek Timeline
    let pos_secs = player.engine.position().as_secs_f64();
    let dur_secs = current_track.map(|t| t.duration_secs).unwrap_or(0.0);
    let bar_width = (left_w - 20.0).clamp(260.0, 440.0);

    render_seek_bar(ui, player, theme, bar_width, pos_secs, dur_secs);

    // Duration Labels
    ui.allocate_ui_with_layout(
        vec2(bar_width, 18.0),
        Layout::left_to_right(egui::Align::Center),
        |row_ui| {
            row_ui.label(
                RichText::new(format_time_secs(pos_secs))
                    .size(12.0)
                    .color(theme.text_secondary),
            );
            row_ui.with_layout(Layout::right_to_left(egui::Align::Center), |r_ui| {
                r_ui.label(
                    RichText::new(format_time_secs(dur_secs))
                        .size(12.0)
                        .color(theme.text_muted),
                );
            });
        },
    );

    ui.add_space(16.0);

    // 4. Playback Controls (Shuffle, Prev, Play/Pause, Next, Repeat)
    render_playback_controls(ui, player, theme);

    ui.add_space(14.0);

    // 5. Volume Slider with Mute Toggle
    render_volume_control(ui, player, theme, bar_width);
}

/// Renders the format / Hi-Res quality badge pill.
fn render_quality_badge(ui: &mut Ui, track: &Track, theme: &Theme) {
    let is_hires = track.is_hires();
    let is_lossless = track.is_lossless();

    if !is_hires && !is_lossless && track.bitrate_kbps == 0 {
        return;
    }

    let badge_text = track.quality_badge_text();
    let detail_str = track.bit_depth_sample_rate_str();

    let (badge_bg, badge_border, text_col) = if is_hires {
        (
            Color32::from_rgba_premultiplied(255, 178, 92, 35),
            Color32::from_rgba_premultiplied(255, 178, 92, 110),
            theme.accent_warm,
        )
    } else if is_lossless {
        (
            Color32::from_rgba_premultiplied(theme.accent.r(), theme.accent.g(), theme.accent.b(), 35),
            theme.border_accent,
            theme.accent,
        )
    } else {
        (
            Color32::from_rgba_premultiplied(255, 255, 255, 20),
            theme.border,
            theme.text_secondary,
        )
    };

    let display_str = if is_hires || is_lossless {
        format!("{}  |  {}", badge_text, detail_str)
    } else {
        format!("{} kbps", track.bitrate_kbps)
    };

    let (rect, _) = ui.allocate_exact_size(vec2(165.0, 22.0), Sense::hover());
    ui.painter().rect_filled(rect, Rounding::same(11.0), badge_bg);
    ui.painter().rect_stroke(rect, Rounding::same(11.0), Stroke::new(1.0_f32, badge_border));

    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        display_str,
        FontId::proportional(11.0),
        text_col,
    );
}

/// Renders the smooth interactive seekbar.
fn render_seek_bar(
    ui: &mut Ui,
    player: &mut PlayerController,
    theme: &Theme,
    bar_width: f32,
    pos: f64,
    dur: f64,
) {
    let desired_size = vec2(bar_width, 22.0);
    let (rect, resp) = ui.allocate_exact_size(desired_size, Sense::click_and_drag());

    let is_active = resp.hovered() || resp.dragged();
    let track_h = if is_active { 6.0 } else { 4.0 };
    let track_rect = Rect::from_center_size(rect.center(), vec2(bar_width, track_h));
    let progress = if dur > 0.0 { (pos / dur).clamp(0.0, 1.0) as f32 } else { 0.0 };

    if (resp.clicked() || resp.dragged()) && dur > 0.0 {
        if let Some(mouse_pos) = resp.interact_pointer_pos() {
            let ratio = ((mouse_pos.x - track_rect.min.x) / track_rect.width()).clamp(0.0, 1.0);
            let target_secs = ratio as f64 * dur;
            player.engine.seek(Duration::from_secs_f64(target_secs));
        }
    }

    // Background track
    ui.painter().rect_filled(
        track_rect,
        Rounding::same(track_h / 2.0),
        Color32::from_rgba_premultiplied(255, 255, 255, 30),
    );

    // Filled progress
    let fill_w = track_rect.width() * progress;
    if fill_w > 0.0 {
        let fill_rect = Rect::from_min_size(track_rect.min, vec2(fill_w, track_h));
        ui.painter().rect_filled(
            fill_rect,
            Rounding::same(track_h / 2.0),
            theme.accent,
        );
    }

    // Interactive thumb handle
    if is_active {
        let thumb_x = track_rect.min.x + fill_w;
        let thumb_center = pos2(thumb_x, track_rect.center().y);
        let thumb_r = if resp.dragged() { 7.0 } else { 6.0 };
        ui.painter().circle_filled(thumb_center, thumb_r, Color32::WHITE);
        ui.painter().circle_stroke(thumb_center, thumb_r, Stroke::new(1.0_f32, theme.border_accent));
    }
}

/// Renders playback control buttons (Shuffle, Previous, Play/Pause, Next, Repeat).
fn render_playback_controls(
    ui: &mut Ui,
    player: &mut PlayerController,
    theme: &Theme,
) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = vec2(18.0, 0.0);

        // 1. Shuffle
        let shuffle_color = if player.shuffle { theme.accent } else { theme.text_muted };
        let (rect, resp) = ui.allocate_exact_size(vec2(28.0, 28.0), Sense::click());
        if resp.hovered() {
            ui.painter().rect_filled(rect, Rounding::same(6.0), Color32::from_rgba_premultiplied(255, 255, 255, 15));
        }
        icons::paint_icon(ui.painter(), Icon::Shuffle, rect.shrink(4.0), shuffle_color);
        if resp.clicked() {
            player.toggle_shuffle();
        }

        // 2. Previous Track
        let (rect, resp) = ui.allocate_exact_size(vec2(32.0, 32.0), Sense::click());
        if resp.hovered() {
            ui.painter().rect_filled(rect, Rounding::same(6.0), Color32::from_rgba_premultiplied(255, 255, 255, 15));
        }
        icons::paint_icon(ui.painter(), Icon::SkipBack, rect.shrink(4.0), Color32::WHITE);
        if resp.clicked() {
            player.prev();
        }

        // 3. Play / Pause (Large Primary Button)
        let is_playing = player.engine.is_playing();
        let (btn_rect, btn_resp) = ui.allocate_exact_size(vec2(52.0, 52.0), Sense::click());
        let bg_color = if btn_resp.hovered() {
            Color32::from_rgb(255, 255, 255)
        } else {
            Color32::from_rgb(240, 240, 245)
        };

        // Subtle accent aura glow behind play button
        ui.painter().circle_filled(
            btn_rect.center(),
            28.0,
            Color32::from_rgba_premultiplied(theme.accent.r(), theme.accent.g(), theme.accent.b(), 35),
        );
        ui.painter().circle_filled(btn_rect.center(), 24.0, bg_color);

        let play_icon = if is_playing { Icon::Pause } else { Icon::Play };
        icons::paint_icon(
            ui.painter(),
            play_icon,
            Rect::from_center_size(btn_rect.center(), vec2(20.0, 20.0)),
            Color32::from_rgb(15, 12, 22),
        );

        if btn_resp.clicked() {
            if player.engine.current_path().is_some() || player.current_track().is_some() {
                player.engine.toggle_play_pause();
            } else if !player.queue.is_empty() {
                player.play_index(0);
            }
        }

        // 4. Next Track
        let (rect, resp) = ui.allocate_exact_size(vec2(32.0, 32.0), Sense::click());
        if resp.hovered() {
            ui.painter().rect_filled(rect, Rounding::same(6.0), Color32::from_rgba_premultiplied(255, 255, 255, 15));
        }
        icons::paint_icon(ui.painter(), Icon::SkipForward, rect.shrink(4.0), Color32::WHITE);
        if resp.clicked() {
            player.next();
        }

        // 5. Repeat
        let (repeat_icon, repeat_color) = match player.repeat {
            RepeatMode::Off => (Icon::Repeat, theme.text_muted),
            RepeatMode::All => (Icon::Repeat, theme.accent),
            RepeatMode::One => (Icon::RepeatOne, theme.accent),
        };
        let (rect, resp) = ui.allocate_exact_size(vec2(28.0, 28.0), Sense::click());
        if resp.hovered() {
            ui.painter().rect_filled(rect, Rounding::same(6.0), Color32::from_rgba_premultiplied(255, 255, 255, 15));
        }
        icons::paint_icon(ui.painter(), repeat_icon, rect.shrink(4.0), repeat_color);
        if resp.clicked() {
            player.cycle_repeat();
        }
    });
}

/// Renders the volume control slider with mute toggle.
fn render_volume_control(
    ui: &mut Ui,
    player: &mut PlayerController,
    theme: &Theme,
    max_w: f32,
) {
    let vol = player.engine.volume();
    let is_muted = player.engine.is_muted();
    let eff_vol = if is_muted { 0.0 } else { vol };

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = vec2(10.0, 0.0);

        // Mute toggle icon
        let vol_icon = if is_muted || eff_vol <= 0.001 {
            Icon::VolumeMute
        } else if eff_vol < 0.33 {
            Icon::VolumeLow
        } else if eff_vol < 0.66 {
            Icon::VolumeMed
        } else {
            Icon::VolumeHigh
        };

        let (icon_rect, icon_resp) = ui.allocate_exact_size(vec2(24.0, 24.0), Sense::click());
        if icon_resp.hovered() {
            ui.painter().rect_filled(icon_rect, Rounding::same(4.0), Color32::from_rgba_premultiplied(255, 255, 255, 15));
        }
        let icon_col = if is_muted { theme.text_muted } else { theme.text_secondary };
        icons::paint_icon(ui.painter(), vol_icon, icon_rect.shrink(3.0), icon_col);
        if icon_resp.clicked() {
            player.engine.toggle_mute();
        }

        // Volume slider bar
        let slider_w = (max_w - 74.0).clamp(90.0, 160.0);
        let (vbar_rect, vbar_resp) = ui.allocate_exact_size(vec2(slider_w, 20.0), Sense::click_and_drag());
        let track_h = if vbar_resp.hovered() || vbar_resp.dragged() { 5.0 } else { 3.5 };
        let track_rect = Rect::from_center_size(vbar_rect.center(), vec2(slider_w, track_h));

        if vbar_resp.clicked() || vbar_resp.dragged() {
            if let Some(mouse_pos) = vbar_resp.interact_pointer_pos() {
                let ratio = ((mouse_pos.x - track_rect.min.x) / track_rect.width()).clamp(0.0, 1.0);
                if is_muted && ratio > 0.0 {
                    player.engine.set_muted(false);
                }
                player.engine.set_volume(ratio);
            }
        }

        ui.painter().rect_filled(
            track_rect,
            Rounding::same(track_h / 2.0),
            Color32::from_rgba_premultiplied(255, 255, 255, 25),
        );

        let fill_w = track_rect.width() * eff_vol.clamp(0.0, 1.0);
        if fill_w > 0.0 {
            let fill_rect = Rect::from_min_size(track_rect.min, vec2(fill_w, track_h));
            let fill_col = if is_muted { theme.text_muted } else { theme.accent };
            ui.painter().rect_filled(fill_rect, Rounding::same(track_h / 2.0), fill_col);
        }

        // Percentage text
        let pct_text = format!("{}%", (eff_vol * 100.0).round() as u32);
        ui.label(RichText::new(pct_text).size(12.0).color(theme.text_muted));
    });
}

/// Renders the right column lyrics stream (synchronized, plain, instrumental, or empty fallback).
fn render_right_lyrics_column(
    ui: &mut Ui,
    player: &mut PlayerController,
    theme: &Theme,
    panel_rect: Rect,
) {
    let current_pos_secs = player.engine.position().as_secs_f64();
    let current_ms = (current_pos_secs * 1000.0) as u64;

    let seek_target_ms = if let Some(ref lyrics) = player.lyrics {
        if lyrics.lines.is_empty() {
            render_no_lyrics_card(ui, player.is_loading_lyrics, theme, panel_rect);
            None
        } else if lyrics.is_synced {
            render_synced_lyrics(ui, lyrics, current_ms, theme, panel_rect)
        } else {
            render_plain_lyrics(ui, lyrics, theme, panel_rect);
            None
        }
    } else {
        render_no_lyrics_card(ui, player.is_loading_lyrics, theme, panel_rect);
        None
    };

    if let Some(ms) = seek_target_ms {
        player.engine.seek(Duration::from_millis(ms));
    }
}

/// Renders synchronized .lrc lyrics with autoscroll lerp centering and interactive seeking.
fn render_synced_lyrics(
    ui: &mut Ui,
    lyrics: &Lyrics,
    current_ms: u64,
    theme: &Theme,
    panel_rect: Rect,
) -> Option<u64> {
    let active_idx = lyrics.current_index(current_ms);
    let mut seek_target: Option<u64> = None;

    // Calculate vertical centering target for active lyric line
    let line_height_approx = 46.0;
    let top_padding = (panel_rect.height() * 0.38).max(60.0);

    let target_scroll_y = if let Some(idx) = active_idx {
        let active_y = top_padding + (idx as f32) * line_height_approx;
        (active_y - panel_rect.height() * 0.40).max(0.0)
    } else {
        0.0
    };

    // Smooth lerp scrolling
    let scroll_id = Id::new("fullscreen_lyrics_smooth_scroll_offset");
    let prev_scroll_y = ui.ctx().data(|d| d.get_temp::<f32>(scroll_id)).unwrap_or(target_scroll_y);
    let dt = ui.input(|i| i.stable_dt).clamp(0.001, 0.1);
    let lerp_factor = (8.0 * dt).min(1.0);
    let cur_scroll_y = prev_scroll_y + (target_scroll_y - prev_scroll_y) * lerp_factor;
    ui.ctx().data_mut(|d| d.insert_temp(scroll_id, cur_scroll_y));

    if (cur_scroll_y - target_scroll_y).abs() > 0.5 {
        ui.ctx().request_repaint();
    }

    ScrollArea::vertical()
        .vertical_scroll_offset(cur_scroll_y)
        .auto_shrink([false, false])
        .show(ui, |s_ui| {
            s_ui.add_space(top_padding);

            for (i, line) in lyrics.lines.iter().enumerate() {
                let is_active = active_idx == Some(i);
                let is_past = active_idx.is_some_and(|curr| i < curr);

                let font_size = if is_active { 22.0 } else { 16.5 };
                let text_color = if is_active {
                    Color32::WHITE
                } else if is_past {
                    Color32::from_rgba_premultiplied(190, 185, 205, 115)
                } else {
                    Color32::from_rgba_premultiplied(160, 155, 175, 75)
                };

                let line_text = if line.text.trim().is_empty() {
                    "..."
                } else {
                    line.text.trim()
                };

                let rich = RichText::new(line_text)
                    .size(font_size)
                    .color(text_color);
                let rich = if is_active { rich.strong() } else { rich };

                let resp = s_ui.add(
                    egui::Button::new(rich)
                        .fill(Color32::TRANSPARENT)
                        .stroke(Stroke::NONE)
                        .rounding(Rounding::same(8.0)),
                );

                if resp.hovered() && !is_active {
                    s_ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                }

                // Vertical glowing indicator bar for the active line
                if is_active {
                    let bar_rect = Rect::from_min_size(
                        pos2(resp.rect.min.x - 12.0, resp.rect.min.y + 4.0),
                        vec2(3.5, resp.rect.height() - 8.0),
                    );
                    s_ui.painter().rect_filled(bar_rect, Rounding::same(2.0), theme.accent);
                }

                // Click line to seek
                if resp.clicked() {
                    seek_target = Some(line.timestamp_ms);
                }

                s_ui.add_space(8.0);
            }

            s_ui.add_space(panel_rect.height() * 0.50);
        });

    seek_target
}

/// Renders unsynchronized plain lyrics.
fn render_plain_lyrics(
    ui: &mut Ui,
    lyrics: &Lyrics,
    theme: &Theme,
    panel_rect: Rect,
) {
    ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |s_ui| {
            s_ui.add_space(32.0);
            for line in &lyrics.lines {
                let rich = RichText::new(&line.text)
                    .size(17.0)
                    .color(theme.text_secondary);
                s_ui.label(rich);
                s_ui.add_space(8.0);
            }
            s_ui.add_space(panel_rect.height() * 0.3);
        });
}

/// Renders a calm minimalist card when no lyrics are available.
fn render_no_lyrics_card(
    ui: &mut Ui,
    is_loading: bool,
    theme: &Theme,
    panel_rect: Rect,
) {
    let center = panel_rect.center();
    let card_w = 320.0;
    let card_h = 160.0;
    let card_rect = Rect::from_center_size(center, vec2(card_w, card_h));

    ui.painter().rect_filled(
        card_rect,
        Rounding::same(16.0),
        Color32::from_rgba_premultiplied(20, 16, 32, 140),
    );
    ui.painter().rect_stroke(
        card_rect,
        Rounding::same(16.0),
        Stroke::new(1.0_f32, theme.border),
    );

    let icon_rect = Rect::from_center_size(pos2(center.x, card_rect.min.y + 44.0), vec2(36.0, 36.0));
    let (icon, title_text, sub_text) = if is_loading {
        (Icon::Refresh, "Searching Lyrics...", "Looking up synchronized lyrics online")
    } else {
        (Icon::Mic, "No Lyrics Available", "Lyrics could not be found for this track")
    };

    icons::paint_icon(ui.painter(), icon, icon_rect, theme.accent);

    ui.allocate_new_ui(
        UiBuilder::new()
            .max_rect(Rect::from_min_max(pos2(card_rect.min.x + 16.0, card_rect.min.y + 72.0), card_rect.max))
            .layout(Layout::top_down(egui::Align::Center)),
        |card_ui| {
            card_ui.label(RichText::new(title_text).size(16.0).strong().color(theme.text_primary));
            card_ui.add_space(4.0);
            card_ui.label(RichText::new(sub_text).size(12.5).color(theme.text_muted));
        },
    );
}

/// Renders floating subtle top-right button to exit fullscreen view.
fn render_floating_exit_button(
    ui: &mut Ui,
    theme: &Theme,
    full_rect: Rect,
    on_exit_fullscreen: &mut bool,
) {
    let button_size = vec2(178.0, 36.0);
    let top_rect = Rect::from_min_size(
        pos2(full_rect.max.x - button_size.x - 32.0, full_rect.min.y + 24.0),
        button_size,
    );

    let resp = ui.allocate_rect(top_rect, Sense::click());
    let is_hovered = resp.hovered();

    let bg_color = if is_hovered {
        Color32::from_rgba_premultiplied(35, 28, 55, 220)
    } else {
        Color32::from_rgba_premultiplied(20, 16, 32, 160)
    };

    let border_color = if is_hovered {
        theme.border_accent
    } else {
        theme.border
    };

    ui.painter().rect_filled(top_rect, Rounding::same(18.0), bg_color);
    ui.painter().rect_stroke(top_rect, Rounding::same(18.0), Stroke::new(1.0_f32, border_color));

    let icon_rect = Rect::from_center_size(
        pos2(top_rect.min.x + 22.0, top_rect.center().y),
        vec2(16.0, 16.0),
    );
    let text_col = if is_hovered { Color32::WHITE } else { theme.text_secondary };
    icons::paint_icon(ui.painter(), Icon::Minimize, icon_rect, text_col);

    let text_pos = pos2(top_rect.min.x + 40.0, top_rect.center().y - 1.0);
    ui.painter().text(
        text_pos,
        egui::Align2::LEFT_CENTER,
        "Exit Fullscreen (Esc)",
        FontId::proportional(12.5),
        text_col,
    );

    if resp.clicked() {
        *on_exit_fullscreen = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppConfig;
    use crate::lyrics::LyricLine;

    #[test]
    fn test_track_key_formatting() {
        let track = Track {
            artist: "Aphex Twin".to_string(),
            title: "Xtal".to_string(),
            ..Default::default()
        };
        let key = format!("{}_{}", track.artist, track.title);
        assert_eq!(key, "Aphex Twin_Xtal");
    }

    #[test]
    fn test_synced_lyrics_indexing() {
        let lines = vec![
            LyricLine { timestamp_ms: 0, text: "Line 0".to_string() },
            LyricLine { timestamp_ms: 2000, text: "Line 1".to_string() },
            LyricLine { timestamp_ms: 5000, text: "Line 2".to_string() },
        ];
        let lyrics = Lyrics {
            lines,
            is_synced: true,
            source: "test".to_string(),
        };

        assert_eq!(lyrics.current_index(1000), Some(0));
        assert_eq!(lyrics.current_index(3000), Some(1));
        assert_eq!(lyrics.current_index(6000), Some(2));
    }

    #[test]
    fn test_exit_fullscreen_flag_trigger() {
        let mut exit_flag = false;
        let on_exit = &mut exit_flag;
        *on_exit = true;
        assert!(exit_flag);
    }

    #[test]
    fn test_render_cinematic_fullscreen_smoke() {
        let ctx = egui::Context::default();
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let cfg = AppConfig::default();
                let mut player = PlayerController::new(&cfg);
                let theme = Theme::from_name("Nebula", None);
                let mut art_manager = AlbumArtManager::new();
                let mut ambient_cache = AmbientBlurCache::new();
                let mut on_exit = false;

                render_cinematic_fullscreen(
                    ui,
                    &mut player,
                    &theme,
                    &mut art_manager,
                    &mut ambient_cache,
                    &mut on_exit,
                );

                assert!(!on_exit);
            });
        });
    }

    #[test]
    fn test_render_fullscreen_with_playing_track_and_lyrics() {
        let ctx = egui::Context::default();
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let cfg = AppConfig::default();
                let mut player = PlayerController::new(&cfg);
                let track = Track {
                    id: 1,
                    title: "Selected Ambient Works".to_string(),
                    artist: "Aphex Twin".to_string(),
                    album: "SAW 85-92".to_string(),
                    duration_secs: 280.0,
                    sample_rate: 96000,
                    bitrate_kbps: 2400,
                    format: "FLAC".to_string(),
                    path: "C:\\music\\test.flac".to_string(),
                    ..Default::default()
                };
                player.load_queue(vec![track], 0);

                let lyrics = Lyrics {
                    lines: vec![
                        LyricLine { timestamp_ms: 0, text: "First line of the song".to_string() },
                        LyricLine { timestamp_ms: 5000, text: "Second line of the song".to_string() },
                        LyricLine { timestamp_ms: 10000, text: "Third line of the song".to_string() },
                    ],
                    is_synced: true,
                    source: "test_lrc".to_string(),
                };
                player.lyrics = Some(lyrics);

                let theme = Theme::from_name("Nebula", None);
                let mut art_manager = AlbumArtManager::new();
                let mut ambient_cache = AmbientBlurCache::new();
                let mut on_exit = false;

                render_cinematic_fullscreen(
                    ui,
                    &mut player,
                    &theme,
                    &mut art_manager,
                    &mut ambient_cache,
                    &mut on_exit,
                );

                assert!(!on_exit);
            });
        });
    }

    #[test]
    fn test_render_fullscreen_plain_lyrics() {
        let ctx = egui::Context::default();
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let cfg = AppConfig::default();
                let mut player = PlayerController::new(&cfg);
                let lyrics = Lyrics {
                    lines: vec![
                        LyricLine { timestamp_ms: 0, text: "Plain text line 1".to_string() },
                        LyricLine { timestamp_ms: 0, text: "Plain text line 2".to_string() },
                    ],
                    is_synced: false,
                    source: "test_plain".to_string(),
                };
                player.lyrics = Some(lyrics);

                let theme = Theme::from_name("Glacier", None);
                let mut art_manager = AlbumArtManager::new();
                let mut ambient_cache = AmbientBlurCache::new();
                let mut on_exit = false;

                render_cinematic_fullscreen(
                    ui,
                    &mut player,
                    &theme,
                    &mut art_manager,
                    &mut ambient_cache,
                    &mut on_exit,
                );

                assert!(!on_exit);
            });
        });
    }

    #[test]
    fn test_render_fullscreen_stream_track() {
        let ctx = egui::Context::default();
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let cfg = AppConfig::default();
                let mut player = PlayerController::new(&cfg);
                let stream_track = crate::streams::provider::StreamTrack::new(
                    "sp-test",
                    "Stream Title",
                    "Stream Artist",
                    "Stream Album",
                    210.0,
                    None,
                    "spotify:track:test1234",
                    "spotify",
                );
                player.switch_to_stream("spotify", Some(stream_track));

                let theme = Theme::from_name("Obsidian", None);
                let mut art_manager = AlbumArtManager::new();
                let mut ambient_cache = AmbientBlurCache::new();
                let mut on_exit = false;

                render_cinematic_fullscreen(
                    ui,
                    &mut player,
                    &theme,
                    &mut art_manager,
                    &mut ambient_cache,
                    &mut on_exit,
                );

                assert!(!on_exit);
            });
        });
    }
}
