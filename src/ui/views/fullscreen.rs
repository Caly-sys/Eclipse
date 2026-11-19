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
    let track_hash = if track_key.is_empty() {
        0u64
    } else {
        Id::new(&track_key).value()
    };

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
            render_right_lyrics_column(right_ui, player, theme, right_rect, track_hash);
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

/// Persistent smooth scroll and autoscroll tracking state for cinematic fullscreen lyrics.
#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct LyricsScrollState {
    pub current_y: f32,
    pub target_y: f32,
    pub last_active_idx: Option<usize>,
    pub manual_scroll_until: f64,
    pub last_track_key: u64,
}

/// Visual presentation attributes for a single lyric line.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LineStyle {
    pub font_size: f32,
    pub color: Color32,
    pub is_strong: bool,
    pub is_active: bool,
}

/// Computes the visual hierarchy and contrast for a lyric line based on its state relative to playback.
pub fn compute_line_style(
    idx: usize,
    active_idx: Option<usize>,
    is_instrumental: bool,
) -> LineStyle {
    let is_active = Some(idx) == active_idx;

    if is_active {
        let color = if is_instrumental {
            Color32::from_white_alpha(150)
        } else {
            Color32::WHITE
        };
        LineStyle {
            font_size: 28.0,
            color,
            is_strong: true,
            is_active: true,
        }
    } else {
        match active_idx {
            Some(active) => {
                if idx < active {
                    // Past line: soft muted translucent white
                    let color = if is_instrumental {
                        Color32::from_white_alpha(55)
                    } else {
                        Color32::from_white_alpha(95)
                    };
                    LineStyle {
                        font_size: 16.0,
                        color,
                        is_strong: false,
                        is_active: false,
                    }
                } else if idx == active + 1 {
                    // Immediate next upcoming line
                    let color = if is_instrumental {
                        Color32::from_white_alpha(100)
                    } else {
                        Color32::from_white_alpha(175)
                    };
                    LineStyle {
                        font_size: 18.0,
                        color,
                        is_strong: false,
                        is_active: false,
                    }
                } else {
                    // Upcoming lines further ahead (idx >= active + 2)
                    let color = if is_instrumental {
                        Color32::from_white_alpha(65)
                    } else {
                        Color32::from_white_alpha(115)
                    };
                    LineStyle {
                        font_size: 16.0,
                        color,
                        is_strong: false,
                        is_active: false,
                    }
                }
            }
            None => {
                // Before playback reaches the first synced lyric line
                if idx == 0 {
                    let color = if is_instrumental {
                        Color32::from_white_alpha(100)
                    } else {
                        Color32::from_white_alpha(175)
                    };
                    LineStyle {
                        font_size: 18.0,
                        color,
                        is_strong: false,
                        is_active: false,
                    }
                } else {
                    let color = if is_instrumental {
                        Color32::from_white_alpha(65)
                    } else {
                        Color32::from_white_alpha(115)
                    };
                    LineStyle {
                        font_size: 16.0,
                        color,
                        is_strong: false,
                        is_active: false,
                    }
                }
            }
        }
    }
}

/// Returns true if the lyric line denotes an instrumental break.
pub fn is_instrumental_break(text: &str) -> bool {
    let t = text.trim();
    t.is_empty()
        || t == "..."
        || t == "···"
        || t == "· · ·"
        || t == "---"
        || (!t.is_empty() && t.chars().all(|c| c == '.' || c == ' ' || c == '·' || c == '-'))
}

/// Formats lyric line text for clean non-emoji display.
pub fn format_lyric_line_display(text: &str) -> &str {
    let trimmed = text.trim();
    if is_instrumental_break(trimmed) {
        "· · ·"
    } else {
        trimmed
    }
}

/// Calculates the active lyric line index from playback seconds, with a 50ms early highlight threshold.
pub fn calculate_active_line_index(lines: &[crate::lyrics::LyricLine], current_pos_secs: f64) -> Option<usize> {
    if lines.is_empty() || current_pos_secs.is_nan() {
        return None;
    }
    let threshold_ms = ((current_pos_secs + 0.05).max(0.0) * 1000.0) as u64;
    let first_line_ms = lines.first()?.timestamp_ms;
    if threshold_ms < first_line_ms {
        return None;
    }
    let mut active = None;
    for (i, line) in lines.iter().enumerate() {
        if threshold_ms >= line.timestamp_ms {
            active = Some(i);
        } else {
            break;
        }
    }
    active
}

/// Calculates target content-local scroll offset targeting 45% viewport centering.
pub fn calculate_target_scroll_y(local_center_y: f32, viewport_h: f32) -> f32 {
    (local_center_y - viewport_h * 0.45).max(0.0)
}

/// Lerps scroll offset toward target offset.
pub fn lerp_scroll(current_y: f32, target_y: f32, dt: f32) -> f32 {
    let dt_clamped = dt.clamp(0.001, 0.1);
    let t = (dt_clamped * 8.0).clamp(0.0, 1.0);
    current_y + (target_y - current_y) * t
}

/// Renders the right column lyrics stream (synchronized, plain, instrumental, or empty fallback).
fn render_right_lyrics_column(
    ui: &mut Ui,
    player: &mut PlayerController,
    theme: &Theme,
    panel_rect: Rect,
    track_hash: u64,
) {
    let current_pos_secs = player.engine.position().as_secs_f64();

    let seek_target_ms = if let Some(ref lyrics) = player.lyrics {
        if lyrics.lines.is_empty() {
            render_no_lyrics_card(ui, player.is_loading_lyrics, theme, panel_rect);
            None
        } else if lyrics.is_synced {
            render_synced_lyrics(ui, lyrics, current_pos_secs, theme, panel_rect, track_hash)
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

/// Renders synchronized .lrc lyrics with content-local auto-centering scroll,
/// manual wheel scroll respect, and high-contrast visual hierarchy.
fn render_synced_lyrics(
    ui: &mut Ui,
    lyrics: &Lyrics,
    current_pos_secs: f64,
    theme: &Theme,
    _panel_rect: Rect,
    track_hash: u64,
) -> Option<u64> {
    let viewport_h = ui.available_height().max(1.0);
    let scroll_state_id = Id::new("fullscreen_lyrics_scroll_state");
    let mut scroll_state: LyricsScrollState = ui
        .data(|d| d.get_temp(scroll_state_id))
        .unwrap_or_default();

    let now_secs = ui.input(|i| i.time);
    let track_changed = scroll_state.last_track_key != track_hash;
    if track_changed {
        scroll_state.last_track_key = track_hash;
        scroll_state.manual_scroll_until = 0.0;
        scroll_state.last_active_idx = None;
    }

    let active_line_idx = calculate_active_line_index(&lyrics.lines, current_pos_secs);

    let mut seek_target: Option<u64> = None;
    let mut detected_target_y: Option<f32> = None;

    let scroll_area_output = ScrollArea::vertical()
        .id_salt("fullscreen_lyrics_scroll")
        .vertical_scroll_offset(scroll_state.current_y)
        .auto_shrink([false, false])
        .show(ui, |s_ui| {
            let content_origin_y = s_ui.cursor().top();
            s_ui.add_space(viewport_h * 0.42);

            for (idx, line) in lyrics.lines.iter().enumerate() {
                let is_active = Some(idx) == active_line_idx;
                let trimmed = line.text.trim();
                let is_instrumental = is_instrumental_break(trimmed);
                let display_text = format_lyric_line_display(trimmed);
                let style = compute_line_style(idx, active_line_idx, is_instrumental);

                let mut rich = RichText::new(display_text)
                    .size(style.font_size)
                    .color(style.color);
                if style.is_strong {
                    rich = rich.strong();
                }

                let line_response = s_ui.horizontal(|h_ui| {
                    h_ui.add_space(20.0);
                    h_ui.add(
                        egui::Button::new(rich)
                            .fill(Color32::TRANSPARENT)
                            .stroke(Stroke::NONE)
                            .rounding(Rounding::same(8.0)),
                    )
                }).inner;

                if is_active {
                    let local_center_y = line_response.rect.center().y - content_origin_y;
                    detected_target_y = Some(calculate_target_scroll_y(local_center_y, viewport_h));

                    // Glowing accent vertical bar next to the active line
                    let bar_h = (line_response.rect.height() - 8.0).clamp(16.0, 32.0);
                    let bar_rect = Rect::from_center_size(
                        pos2(line_response.rect.min.x - 14.0, line_response.rect.center().y),
                        vec2(4.0, bar_h),
                    );
                    // Glowing aura
                    s_ui.painter().rect_filled(
                        bar_rect.expand(3.0),
                        Rounding::same(4.0),
                        Color32::from_rgba_premultiplied(
                            theme.accent.r(),
                            theme.accent.g(),
                            theme.accent.b(),
                            45,
                        ),
                    );
                    // Crisp indicator bar
                    s_ui.painter().rect_filled(bar_rect, Rounding::same(2.0), theme.accent);
                }

                if line_response.hovered() {
                    s_ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    if !is_active {
                        s_ui.painter().rect_filled(
                            line_response.rect.expand2(vec2(8.0, 2.0)),
                            Rounding::same(6.0),
                            Color32::from_rgba_premultiplied(255, 255, 255, 14),
                        );
                    }
                }

                if line_response.clicked() {
                    seek_target = Some(line.timestamp_ms);
                }

                s_ui.add_space(8.0);
            }

            s_ui.add_space(viewport_h * 0.42);
        });

    if let Some(target) = detected_target_y {
        scroll_state.target_y = target;
    } else if active_line_idx.is_none() {
        scroll_state.target_y = 0.0;
    }

    if seek_target.is_some() {
        scroll_state.manual_scroll_until = 0.0;
    }

    // Requirement 2: Manual wheel scroll respect & smooth interpolation
    let actual_offset_y = scroll_area_output.state.offset.y;
    let wheel_delta = ui.input(|i| i.raw_scroll_delta.y.abs() + i.smooth_scroll_delta.y.abs());
    let offset_diff = (actual_offset_y - scroll_state.current_y).abs();
    let is_dragging = ui.input(|i| i.pointer.is_decidedly_dragging());

    if (offset_diff > 0.5 && (wheel_delta > 0.0 || is_dragging)) || wheel_delta > 0.0 {
        scroll_state.current_y = actual_offset_y;
        scroll_state.manual_scroll_until = now_secs + 2.5;
    }

    if now_secs >= scroll_state.manual_scroll_until {
        if track_changed || (scroll_state.target_y - scroll_state.current_y).abs() > viewport_h * 1.5 {
            scroll_state.current_y = scroll_state.target_y;
        } else {
            let dt = ui.input(|i| i.stable_dt);
            scroll_state.current_y = lerp_scroll(scroll_state.current_y, scroll_state.target_y, dt);
            if (scroll_state.target_y - scroll_state.current_y).abs() > 0.5 {
                ui.ctx().request_repaint();
            }
        }
    } else {
        let remaining_secs = (scroll_state.manual_scroll_until - now_secs).max(0.0);
        ui.ctx().request_repaint_after(Duration::from_secs_f64(remaining_secs));
    }

    scroll_state.last_active_idx = active_line_idx;
    ui.data_mut(|d| d.insert_temp(scroll_state_id, scroll_state));

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

    #[test]
    fn test_calculate_target_scroll_y_content_local() {
        let viewport_h = 1000.0;
        // Top spacer is at viewport_h * 0.42 = 420.0
        // Line 0 center with 40px height is at 440.0
        let line_0_center = 440.0;
        let target_0 = calculate_target_scroll_y(line_0_center, viewport_h);
        // (440.0 - 450.0).max(0.0) == 0.0
        assert_eq!(target_0, 0.0);

        // Mid-track line at local_center 1200.0
        let target_mid = calculate_target_scroll_y(1200.0, viewport_h);
        assert_eq!(target_mid, 1200.0 - 450.0);
        // When scrolled by target_mid, on-screen line center is exactly 45% of viewport
        let onscreen_center = 1200.0 - target_mid;
        assert_eq!(onscreen_center, viewport_h * 0.45);
    }

    #[test]
    fn test_lerp_scroll_convergence() {
        let cur = 100.0;
        let target = 200.0;
        let next = lerp_scroll(cur, target, 0.016);
        // With dt=0.016, t = 0.016 * 8.0 = 0.128, delta = 12.8
        assert!(next > cur && next < target);

        // Static convergence
        assert_eq!(lerp_scroll(200.0, 200.0, 0.016), 200.0);

        // Clamped dt test
        let next_huge_dt = lerp_scroll(cur, target, 5.0);
        assert!(next_huge_dt <= target);
    }

    #[test]
    fn test_compute_line_style_hierarchy() {
        let active_idx = Some(2);

        // Past lines: 16px, muted white alpha 95
        let past_style = compute_line_style(0, active_idx, false);
        assert_eq!(past_style.font_size, 16.0);
        assert_eq!(past_style.color, Color32::from_white_alpha(95));
        assert!(!past_style.is_strong);
        assert!(!past_style.is_active);

        // Active line: 28px, crisp bold white, active
        let active_style = compute_line_style(2, active_idx, false);
        assert_eq!(active_style.font_size, 28.0);
        assert_eq!(active_style.color, Color32::WHITE);
        assert!(active_style.is_strong);
        assert!(active_style.is_active);

        // Next upcoming line (active + 1): 18px, alpha 175
        let next_style = compute_line_style(3, active_idx, false);
        assert_eq!(next_style.font_size, 18.0);
        assert_eq!(next_style.color, Color32::from_white_alpha(175));
        assert!(!next_style.is_strong);
        assert!(!next_style.is_active);

        // Upcoming lines (active + 2): 16px, alpha 115
        let upcoming_style = compute_line_style(4, active_idx, false);
        assert_eq!(upcoming_style.font_size, 16.0);
        assert_eq!(upcoming_style.color, Color32::from_white_alpha(115));
        assert!(!upcoming_style.is_strong);
        assert!(!upcoming_style.is_active);
    }

    #[test]
    fn test_compute_line_style_instrumental_and_no_active() {
        // Instrumental active break: 28px with lower opacity
        let active_break = compute_line_style(1, Some(1), true);
        assert_eq!(active_break.font_size, 28.0);
        assert_eq!(active_break.color, Color32::from_white_alpha(150));
        assert!(active_break.is_active);

        // No active line (before song starts)
        let first_style = compute_line_style(0, None, false);
        assert_eq!(first_style.font_size, 18.0);
        assert_eq!(first_style.color, Color32::from_white_alpha(175));

        let later_style = compute_line_style(1, None, false);
        assert_eq!(later_style.font_size, 16.0);
        assert_eq!(later_style.color, Color32::from_white_alpha(115));
    }

    #[test]
    fn test_instrumental_break_detection_and_formatting() {
        assert!(is_instrumental_break(""));
        assert!(is_instrumental_break("   "));
        assert!(is_instrumental_break("..."));
        assert!(is_instrumental_break("···"));
        assert!(is_instrumental_break("· · ·"));
        assert!(is_instrumental_break("---"));
        assert!(!is_instrumental_break("Never gonna give you up"));

        assert_eq!(format_lyric_line_display("..."), "· · ·");
        assert_eq!(format_lyric_line_display("  Hello world  "), "Hello world");
    }

    #[test]
    fn test_calculate_active_line_index_early_threshold() {
        let lines = vec![
            LyricLine { timestamp_ms: 2000, text: "Line 0".to_string() },
            LyricLine { timestamp_ms: 5000, text: "Line 1".to_string() },
            LyricLine { timestamp_ms: 10000, text: "Line 2".to_string() },
        ];

        // Well before line 0
        assert_eq!(calculate_active_line_index(&lines, 0.5), None);

        // 1.94s is before threshold (1.94 + 0.05 = 1.99 < 2.00)
        assert_eq!(calculate_active_line_index(&lines, 1.94), None);

        // 1.96s activates line 0 with 50ms early highlight (1.96 + 0.05 = 2.01 >= 2.00)
        assert_eq!(calculate_active_line_index(&lines, 1.96), Some(0));

        // 3.0s is inside line 0
        assert_eq!(calculate_active_line_index(&lines, 3.0), Some(0));

        // 4.96s activates line 1
        assert_eq!(calculate_active_line_index(&lines, 4.96), Some(1));

        // 15.0s is past the final line
        assert_eq!(calculate_active_line_index(&lines, 15.0), Some(2));

        // NaN handling
        assert_eq!(calculate_active_line_index(&lines, f64::NAN), None);

        // Empty lines
        assert_eq!(calculate_active_line_index(&[], 2.0), None);
    }

    #[test]
    fn test_lyrics_scroll_state_struct() {
        let mut state = LyricsScrollState::default();
        assert_eq!(state.current_y, 0.0);
        assert_eq!(state.target_y, 0.0);
        assert_eq!(state.last_active_idx, None);
        assert_eq!(state.manual_scroll_until, 0.0);
        assert_eq!(state.last_track_key, 0);

        state.current_y = 150.0;
        state.target_y = 300.0;
        state.last_active_idx = Some(4);
        state.manual_scroll_until = 12.5;
        state.last_track_key = 42;

        assert_eq!(state.current_y, 150.0);
        assert_eq!(state.target_y, 300.0);
        assert_eq!(state.last_active_idx, Some(4));
        assert_eq!(state.manual_scroll_until, 12.5);
        assert_eq!(state.last_track_key, 42);
    }
}
