//! Picture-in-Picture (PiP) Floating Mini-Player & Desktop Overlay for Eclipse.
//!
//! Provides a unified dark-glass floating mini-player card with integrated synced desktop lyrics.
//! 100% pure vector graphics - ZERO Unicode emojis.

use crate::audio::PlayerController;
use crate::streams::controller::StreamController;
use crate::ui::art::AlbumArtManager;
use crate::ui::icons::{paint_icon, Icon};
use crate::ui::playerbar::{current_playback_time, seek_to, SeekbarDragState};
use crate::ui::theme::Theme;
use egui::{pos2, vec2, Color32, Rect, Rounding, Stroke, Ui};
use std::time::{Duration, Instant};

#[cfg(target_os = "windows")]
#[allow(clippy::upper_case_acronyms)]
mod win32_rounding {
    use std::sync::Mutex;

    type Hwnd = *mut std::ffi::c_void;
    type Hrgn = *mut std::ffi::c_void;
    type Hresult = i32;
    type Bool = i32;

    const DWMWA_WINDOW_CORNER_PREFERENCE: u32 = 33;
    const DWMWCP_ROUND: u32 = 2;
    const DWMWA_BORDER_COLOR: u32 = 34;
    const DWMWA_COLOR_NONE: u32 = 0xFFFFFFFE;

    #[link(name = "user32")]
    extern "system" {
        fn FindWindowW(lpClassName: *const u16, lpWindowName: *const u16) -> Hwnd;
        fn SetWindowRgn(hWnd: Hwnd, hRgn: Hrgn, bRedraw: Bool) -> i32;
    }

    #[link(name = "dwmapi")]
    extern "system" {
        fn DwmSetWindowAttribute(
            hwnd: Hwnd,
            dwAttribute: u32,
            pvAttribute: *const std::ffi::c_void,
            cbAttribute: u32,
        ) -> Hresult;
    }

    #[link(name = "gdi32")]
    extern "system" {
        fn CreateRoundRectRgn(
            x1: i32,
            y1: i32,
            x2: i32,
            y2: i32,
            w: i32,
            h: i32,
        ) -> Hrgn;
        fn DeleteObject(ho: *mut std::ffi::c_void) -> Bool;
    }

    static LAST_APPLIED: Mutex<Option<(usize, i32, i32)>> = Mutex::new(None);

    /// Safely apply rounded window corners to the native Win32 window matching `title`.
    ///
    /// Win 11: Uses `DwmSetWindowAttribute(DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_ROUND)`
    /// and `DWMWA_BORDER_COLOR = DWMWA_COLOR_NONE`.
    /// Win 10: Falls back to `CreateRoundRectRgn` + `SetWindowRgn` if DWM corner preference fails.
    pub fn apply_rounded_window_corners(
        title: &str,
        phys_w: i32,
        phys_h: i32,
        corner_radius_px: i32,
    ) {
        if phys_w <= 0 || phys_h <= 0 {
            return;
        }

        let wide_title: Vec<u16> = title.encode_utf16().chain(std::iter::once(0)).collect();
        let hwnd = unsafe { FindWindowW(std::ptr::null(), wide_title.as_ptr()) };
        if hwnd.is_null() {
            return;
        }

        let key = (hwnd as usize, phys_w, phys_h);
        if let Ok(mut last) = LAST_APPLIED.lock() {
            if *last == Some(key) {
                return;
            }

            // 1. Windows 11 DWM Corner Preference
            let corner_pref: u32 = DWMWCP_ROUND;
            let hr_corner = unsafe {
                DwmSetWindowAttribute(
                    hwnd,
                    DWMWA_WINDOW_CORNER_PREFERENCE,
                    &corner_pref as *const u32 as *const std::ffi::c_void,
                    std::mem::size_of::<u32>() as u32,
                )
            };

            let border_color: u32 = DWMWA_COLOR_NONE;
            let _ = unsafe {
                DwmSetWindowAttribute(
                    hwnd,
                    DWMWA_BORDER_COLOR,
                    &border_color as *const u32 as *const std::ffi::c_void,
                    std::mem::size_of::<u32>() as u32,
                )
            };

            // 2. Windows 10 Fallback: GDI SetWindowRgn if DWM preference returned non-zero error
            if hr_corner != 0 {
                let rgn = unsafe {
                    CreateRoundRectRgn(
                        0,
                        0,
                        phys_w + 1,
                        phys_h + 1,
                        corner_radius_px * 2,
                        corner_radius_px * 2,
                    )
                };
                if !rgn.is_null() {
                    let set_res = unsafe { SetWindowRgn(hwnd, rgn, 1) };
                    if set_res == 0 {
                        unsafe {
                            DeleteObject(rgn);
                        }
                    }
                }
            }

            *last = Some(key);
        }
    }
}

#[cfg(target_os = "windows")]
pub use win32_rounding::apply_rounded_window_corners;

#[cfg(not(target_os = "windows"))]
pub fn apply_rounded_window_corners(
    _title: &str,
    _phys_w: i32,
    _phys_h: i32,
    _corner_radius_px: i32,
) {
}

/// Truncate string to `max_chars` with an ellipsis if longer.
fn truncate_str(s: &str, max_chars: usize) -> String {
    if s.chars().count() > max_chars {
        let truncated: String = s.chars().take(max_chars.saturating_sub(1)).collect();
        format!("{truncated}…")
    } else {
        s.to_string()
    }
}

/// Resolve the currently active single-line lyric text for the mini-player header strip.
pub fn active_lyric_text(player: &PlayerController) -> Option<String> {
    player.current_track()?;
    let lyrics = player.lyrics.as_ref()?;
    if lyrics.is_empty() {
        return None;
    }
    if lyrics.source.contains("Instrumental")
        || (lyrics.lines.len() == 1 && lyrics.lines[0].text.eq_ignore_ascii_case("instrumental"))
    {
        return Some("Instrumental".to_string());
    }
    let pos_ms = player.engine.position().as_millis() as u64;
    if lyrics.is_synced {
        if let Some(idx) = lyrics.current_index(pos_ms) {
            let line = lyrics.lines[idx].text.trim();
            if line.is_empty() {
                Some("...".to_string())
            } else {
                Some(line.to_string())
            }
        } else {
            Some("...".to_string())
        }
    } else {
        // Plain unsynced lyrics: show first non-empty line
        lyrics
            .lines
            .iter()
            .find(|l| !l.text.trim().is_empty())
            .map(|l| l.text.trim().to_string())
    }
}

/// Standalone helper for rendering high-contrast synced desktop lyrics overlay.
pub fn render_desktop_lyrics(
    ui: &mut Ui,
    player: &PlayerController,
    _theme: &Theme,
) -> egui::Response {
    let width = ui.available_width().max(320.0);
    let height = 44.0;
    let (rect, resp) = ui.allocate_exact_size(vec2(width, height), egui::Sense::click_and_drag());

    if resp.drag_started_by(egui::PointerButton::Primary) || resp.dragged() {
        ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
    }

    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        let pill_rect = Rect::from_center_size(rect.center(), vec2(rect.width(), height));
        painter.rect_filled(
            pill_rect,
            Rounding::same(12.0),
            Color32::from_rgba_unmultiplied(16, 16, 22, 245),
        );
        painter.rect_stroke(
            pill_rect,
            Rounding::same(12.0),
            Stroke::new(1.0_f32, Color32::from_white_alpha(28)),
        );

        let active_text = active_lyric_text(player).unwrap_or_else(|| "Eclipse · Lossless Audio".to_string());
        let truncated = truncate_str(&active_text, 40);
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            &truncated,
            egui::FontId::proportional(14.0),
            Color32::from_white_alpha(235),
        );
    }

    resp
}

/// Render the unified dark-glass floating mini-player card.
///
/// Styling:
/// - Window-edge flush layout (no outer black gap)
/// - Background: solid card color `Color32::from_rgb(16, 16, 22)`
/// - Border: `Stroke::new(1.0, Color32::from_white_alpha(32))` hugging native rounded edge
/// - Rounding: `Rounding::same(8.0)` matching native Win32 window rounding
/// - Top lyrics strip: centered `13.5px` white text separated by `Color32::from_white_alpha(18)` hairline divider
/// - Left: 54px x 54px album cover art clipped with `Rounding::same(8.0)`, fallback to vector `Icon::Disc`
/// - Top-Right Pinned: Close button `[×]` (`Icon::Close`, size 10.0) with subtle circular hover effect
/// - Track Metadata: Row 1 = Title (bold white, 14.0px), Row 2 = Artist (muted, 12.0px)
/// - Bottom Controls Row (all items locked to the exact same `center().y`):
///   SkipBack (14.0px), Play/Pause (25px purple solid circle), SkipForward (14.0px),
///   interactive seekbar (dark rail, purple active fill, white thumb), Favorite Heart button (14.0px)
#[allow(clippy::too_many_arguments)]
pub fn render_unified_pip_card(
    ui: &mut Ui,
    player: &mut PlayerController,
    stream_controller: Option<&StreamController>,
    theme: &Theme,
    art_manager: &mut AlbumArtManager,
    show_pip: &mut bool,
    on_toggle_fav: &mut Option<i64>,
    show_lyrics_setting: bool,
) -> egui::Response {
    let active_lyric = if show_lyrics_setting {
        active_lyric_text(player)
    } else {
        None
    };
    let has_lyrics = active_lyric.is_some();
    let card_height = if has_lyrics { 122.0 } else { 86.0 };
    let card_width = 386.0;

    let (card_rect, resp) = ui.allocate_exact_size(vec2(card_width, card_height), egui::Sense::click_and_drag());

    let painter = ui.painter();

    // 1. Draw unified card background
    painter.rect_filled(
        card_rect,
        Rounding::same(8.0),
        Color32::from_rgb(16, 16, 22),
    );
    // 2. Subtle glass border hugging native rounded edge
    painter.rect_stroke(
        card_rect.shrink(0.5),
        Rounding::same(8.0),
        Stroke::new(1.0_f32, Color32::from_white_alpha(32)),
    );

    // 3. Lyrics strip if present
    let body_top = if let Some(ref lyric_str) = active_lyric {
        let lyrics_h = 32.0;
        let lyrics_rect = Rect::from_min_max(
            pos2(card_rect.min.x, card_rect.min.y),
            pos2(card_rect.max.x, card_rect.min.y + lyrics_h),
        );
        let divider_y = lyrics_rect.max.y;

        // Truncate lyric text cleanly
        let trunc_lyric = truncate_str(lyric_str, 42);
        let font = egui::FontId::proportional(13.5);
        let color = Color32::from_white_alpha(235);
        painter.text(
            lyrics_rect.center(),
            egui::Align2::CENTER_CENTER,
            &trunc_lyric,
            font,
            color,
        );

        // Hairline divider
        painter.line_segment(
            [pos2(card_rect.min.x, divider_y), pos2(card_rect.max.x, divider_y)],
            Stroke::new(1.0_f32, Color32::from_white_alpha(18)),
        );

        divider_y
    } else {
        card_rect.min.y
    };

    let body_rect = Rect::from_min_max(pos2(card_rect.min.x, body_top), card_rect.max);
    let current_track = player.current_track().cloned();

    // 4. Album cover art (Left: 54x54)
    let art_x = card_rect.min.x + 12.0;
    let art_y = body_rect.min.y + ((body_rect.height() - 54.0) / 2.0).round();
    let art_rect = Rect::from_min_size(pos2(art_x, art_y), vec2(54.0, 54.0));

    let mut painted_cover = false;
    if let Some(ref track) = current_track {
        if let Some(texture) = art_manager.get_or_load(ui.ctx(), &track.path) {
            let img = egui::Image::new(egui::load::SizedTexture::new(texture.id(), art_rect.size()))
                .rounding(Rounding::same(8.0));
            img.paint_at(ui, art_rect);
            painted_cover = true;
        }
    }
    if !painted_cover {
        painter.rect_filled(
            art_rect,
            Rounding::same(8.0),
            Color32::from_rgba_unmultiplied(28, 24, 38, 220),
        );
        let disc_box = Rect::from_center_size(art_rect.center(), vec2(26.0, 26.0));
        paint_icon(painter, Icon::Disc, disc_box, theme.accent);
    }
    painter.rect_stroke(
        art_rect,
        Rounding::same(8.0),
        Stroke::new(1.0_f32, Color32::from_white_alpha(30)),
    );

    // 5. Close Button (Pinned Top-Right)
    let close_rect = Rect::from_center_size(
        pos2(card_rect.max.x - 18.0, body_rect.min.y + 16.0),
        vec2(20.0, 20.0),
    );
    let close_resp = ui.interact(close_rect, ui.id().with("pip_close_btn"), egui::Sense::click());
    if close_resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        painter.circle_filled(close_rect.center(), 10.0, Color32::from_white_alpha(24));
    }
    let close_icon_color = if close_resp.hovered() {
        Color32::WHITE
    } else {
        Color32::from_white_alpha(155)
    };
    paint_icon(
        painter,
        Icon::Close,
        Rect::from_center_size(close_rect.center(), vec2(10.0, 10.0)),
        close_icon_color,
    );
    if close_resp.clicked() {
        *show_pip = false;
    }
    let _ = close_resp.on_hover_text("Close Mini Player");

    // 6. Track Metadata (Title & Artist)
    let meta_x = art_rect.max.x + 12.0;
    let meta_max_w = (close_rect.min.x - meta_x - 6.0).max(80.0);
    let title_str = current_track
        .as_ref()
        .map(|t| t.display_title())
        .unwrap_or("No playback");
    let artist_str = current_track
        .as_ref()
        .map(|t| t.display_artist())
        .unwrap_or("Eclipse");

    // Estimate max chars from meta_max_w: ~7.5px per char at 14px font
    let title_max_chars = ((meta_max_w / 7.8) as usize).clamp(10, 40);
    let artist_max_chars = ((meta_max_w / 6.8) as usize).clamp(12, 45);

    let trunc_title = truncate_str(title_str, title_max_chars);
    let trunc_artist = truncate_str(artist_str, artist_max_chars);

    let title_y = body_rect.min.y + 13.0;
    let artist_y = body_rect.min.y + 32.0;

    painter.text(
        pos2(meta_x, title_y),
        egui::Align2::LEFT_TOP,
        &trunc_title,
        egui::FontId::proportional(14.0),
        Color32::WHITE,
    );
    painter.text(
        pos2(meta_x, artist_y),
        egui::Align2::LEFT_TOP,
        &trunc_artist,
        egui::FontId::proportional(12.0),
        Color32::from_white_alpha(155),
    );

    // 7. Bottom Controls Row (ALL items locked to exact same center().y)
    let controls_y = body_rect.max.y - 18.0;

    // A. Previous Button (SkipBack, 14px)
    let prev_rect = Rect::from_center_size(pos2(meta_x + 9.0, controls_y), vec2(20.0, 20.0));
    let prev_resp = ui.interact(prev_rect, ui.id().with("pip_prev_btn"), egui::Sense::click());
    if prev_resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    let prev_color = if prev_resp.hovered() {
        Color32::WHITE
    } else {
        Color32::from_white_alpha(175)
    };
    paint_icon(
        painter,
        Icon::SkipBack,
        Rect::from_center_size(prev_rect.center(), vec2(14.0, 14.0)),
        prev_color,
    );
    if prev_resp.clicked() {
        player.prev();
    }
    let _ = prev_resp.on_hover_text("Previous Track");

    // B. Play/Pause Button (Purple circular, 25px diameter, dark icon)
    let play_center_x = prev_rect.max.x + 6.0 + 12.5;
    let play_rect = Rect::from_center_size(pos2(play_center_x, controls_y), vec2(25.0, 25.0));
    let play_resp = ui.interact(play_rect, ui.id().with("pip_play_btn"), egui::Sense::click());
    if play_resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    let play_fill = if play_resp.hovered() {
        Color32::from_rgb(175, 132, 255)
    } else {
        Color32::from_rgb(156, 108, 255)
    };
    painter.circle_filled(play_rect.center(), 12.5, play_fill);

    let is_playing = player.engine.is_playing();
    let play_icon = if is_playing { Icon::Pause } else { Icon::Play };
    let icon_offset = if is_playing { vec2(0.0, 0.0) } else { vec2(0.8, 0.0) };
    paint_icon(
        painter,
        play_icon,
        Rect::from_center_size(play_rect.center() + icon_offset, vec2(12.0, 12.0)),
        Color32::from_rgb(18, 16, 24),
    );
    if play_resp.clicked() {
        if player.current_track().is_some() {
            player.engine.toggle_play_pause();
        } else if !player.queue.is_empty() {
            player.play_index(0);
        }
    }
    let _ = play_resp.on_hover_text(if is_playing { "Pause" } else { "Play" });

    // C. Next Button (SkipForward, 14px)
    let next_center_x = play_rect.max.x + 6.0 + 10.0;
    let next_rect = Rect::from_center_size(pos2(next_center_x, controls_y), vec2(20.0, 20.0));
    let next_resp = ui.interact(next_rect, ui.id().with("pip_next_btn"), egui::Sense::click());
    if next_resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    let next_color = if next_resp.hovered() {
        Color32::WHITE
    } else {
        Color32::from_white_alpha(175)
    };
    paint_icon(
        painter,
        Icon::SkipForward,
        Rect::from_center_size(next_rect.center(), vec2(14.0, 14.0)),
        next_color,
    );
    if next_resp.clicked() {
        player.next();
    }
    let _ = next_resp.on_hover_text("Next Track");

    // D. Favorite Button (Far Right, 14px)
    let fav_center_x = card_rect.max.x - 12.0 - 10.0;
    let fav_rect = Rect::from_center_size(pos2(fav_center_x, controls_y), vec2(20.0, 20.0));
    let fav_resp = ui.interact(fav_rect, ui.id().with("pip_fav_btn"), egui::Sense::click());
    if fav_resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    let is_fav = current_track.as_ref().map(|t| t.favorite).unwrap_or(false);
    let fav_icon = if is_fav { Icon::Heart } else { Icon::HeartOutline };
    let fav_color = if is_fav {
        Color32::from_rgb(156, 108, 255)
    } else if fav_resp.hovered() {
        Color32::WHITE
    } else {
        Color32::from_white_alpha(150)
    };
    paint_icon(
        painter,
        fav_icon,
        Rect::from_center_size(fav_rect.center(), vec2(14.0, 14.0)),
        fav_color,
    );
    if fav_resp.clicked() {
        if let Some(ref track) = current_track {
            *on_toggle_fav = Some(track.id);
        }
    }
    let _ = fav_resp.on_hover_text(if is_fav { "Favorited" } else { "Favorite" });

    // E. Seekbar (Slim interactive seekbar between Next and Favorite)
    let seek_min_x = next_rect.max.x + 8.0;
    let seek_max_x = fav_rect.min.x - 8.0;
    let seek_w = (seek_max_x - seek_min_x).max(60.0);
    let seek_rect = Rect::from_center_size(
        pos2(seek_min_x + seek_w / 2.0, controls_y),
        vec2(seek_w, 16.0),
    );
    let seek_resp = ui.interact(seek_rect, ui.id().with("pip_mini_seekbar"), egui::Sense::click_and_drag());
    render_mini_seekbar_rect(ui, player, stream_controller, theme, seek_rect, &seek_resp);

    // 8. Native window dragging on non-interactive card surface
    if resp.drag_started_by(egui::PointerButton::Primary) || resp.dragged() {
        if let Some(ptr_pos) = ui.input(|i| i.pointer.interact_pos().or(i.pointer.latest_pos())) {
            let on_interactive = close_rect.contains(ptr_pos)
                || prev_rect.contains(ptr_pos)
                || play_rect.contains(ptr_pos)
                || next_rect.contains(ptr_pos)
                || seek_rect.expand(2.0).contains(ptr_pos)
                || fav_rect.contains(ptr_pos);
            if !on_interactive {
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
            }
        }
    }

    resp
}

/// Helper for backwards-compatible card rendering without lyrics header.
pub fn render_pip_card(
    ui: &mut Ui,
    player: &mut PlayerController,
    stream_controller: Option<&StreamController>,
    theme: &Theme,
    art_manager: &mut AlbumArtManager,
    show_pip: &mut bool,
    on_toggle_fav: &mut Option<i64>,
) -> egui::Response {
    render_unified_pip_card(
        ui,
        player,
        stream_controller,
        theme,
        art_manager,
        show_pip,
        on_toggle_fav,
        false,
    )
}

/// Helper to render the compact seek line with knob and click/drag-to-seek functionality at a given rect.
fn render_mini_seekbar_rect(
    ui: &mut Ui,
    player: &mut PlayerController,
    stream_controller: Option<&StreamController>,
    _theme: &Theme,
    rect: Rect,
    resp: &egui::Response,
) {
    if resp.hovered() || resp.dragged() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }

    let (pos_secs, dur_secs) = current_playback_time(player, stream_controller);
    let max_dur = dur_secs.max(0.001);

    let pip_seek_id = egui::Id::new("pip_mini_seekbar");
    let mut drag_state = ui.data(|d| d.get_temp::<SeekbarDragState>(pip_seek_id)).unwrap_or_default();

    if let (Some(committed_secs), Some(commit_time)) = (drag_state.last_committed_secs, drag_state.last_committed_time) {
        if commit_time.elapsed() < Duration::from_millis(350) && (pos_secs - committed_secs).abs() > 0.4 {
            // Hold preview position until engine catches up
        } else {
            drag_state.last_committed_secs = None;
            drag_state.last_committed_time = None;
        }
    }

    let rail_h = 3.5_f32;
    let rail_rect = Rect::from_center_size(rect.center(), vec2(rect.width(), rail_h));
    let rail_rounding = Rounding::same(rail_h / 2.0);

    // Track background rail: dark track Color32::from_white_alpha(25)
    ui.painter().rect_filled(
        rail_rect,
        rail_rounding,
        Color32::from_white_alpha(25),
    );

    // Continuous visual scrubbing during drag
    if resp.drag_started() || resp.dragged() {
        drag_state.is_dragging = true;
        if let Some(pointer_pos) = resp.interact_pointer_pos() {
            let frac = ((pointer_pos.x - rail_rect.min.x) / rail_rect.width()).clamp(0.0, 1.0);
            let target = (frac as f64 * max_dur).clamp(0.0, max_dur);
            drag_state.scrub_secs = target;
        }
        ui.ctx().request_repaint();
    }

    // Commit seek on release or click
    if resp.drag_stopped() {
        let target = drag_state.scrub_secs;
        seek_to(player, stream_controller, target);
        drag_state.is_dragging = false;
        drag_state.last_committed_secs = Some(target);
        drag_state.last_committed_time = Some(Instant::now());
        ui.ctx().request_repaint();
    } else if resp.clicked() {
        if let Some(pointer_pos) = resp.interact_pointer_pos() {
            let frac = ((pointer_pos.x - rail_rect.min.x) / rail_rect.width()).clamp(0.0, 1.0);
            let target = (frac as f64 * max_dur).clamp(0.0, max_dur);
            seek_to(player, stream_controller, target);
            drag_state.is_dragging = false;
            drag_state.scrub_secs = target;
            drag_state.last_committed_secs = Some(target);
            drag_state.last_committed_time = Some(Instant::now());
            ui.ctx().request_repaint();
        }
    } else if !resp.dragged() && drag_state.is_dragging {
        drag_state.is_dragging = false;
    }

    ui.data_mut(|d| d.insert_temp(pip_seek_id, drag_state));

    let display_pos = if drag_state.is_dragging {
        drag_state.scrub_secs
    } else if let Some(committed_secs) = drag_state.last_committed_secs {
        committed_secs
    } else {
        pos_secs
    };

    // Filled progress rail: purple Color32::from_rgb(156, 108, 255)
    let progress = (display_pos / max_dur).clamp(0.0, 1.0) as f32;
    let fill_w = rail_rect.width() * progress;
    if fill_w > 0.0 {
        let filled_rect = Rect::from_min_size(rail_rect.min, vec2(fill_w, rail_h));
        ui.painter().rect_filled(
            filled_rect,
            rail_rounding,
            Color32::from_rgb(156, 108, 255),
        );
    }

    // Interactive scrubber knob: small white thumb
    let knob_pos = pos2(rail_rect.min.x + fill_w, rail_rect.center().y);
    let knob_r = if resp.hovered() || resp.dragged() { 4.5 } else { 3.2 };
    ui.painter().circle_filled(knob_pos, knob_r, Color32::WHITE);
}

/// Helper to render the compact seek line with knob and click/drag-to-seek functionality.
pub fn render_mini_seekbar(
    ui: &mut Ui,
    player: &mut PlayerController,
    stream_controller: Option<&StreamController>,
    theme: &Theme,
    desired_width: f32,
) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(vec2(desired_width, 16.0), egui::Sense::click_and_drag());
    render_mini_seekbar_rect(ui, player, stream_controller, theme, rect, &resp);
    resp
}

/// Render the combined overlay content: unified dark-glass floating card.
#[allow(clippy::too_many_arguments)]
pub fn render_pip_overlay_content(
    ui: &mut Ui,
    player: &mut PlayerController,
    stream_controller: Option<&StreamController>,
    theme: &Theme,
    art_manager: &mut AlbumArtManager,
    show_pip: &mut bool,
    on_toggle_fav: &mut Option<i64>,
    show_lyrics: bool,
) -> egui::Response {
    render_unified_pip_card(
        ui,
        player,
        stream_controller,
        theme,
        art_manager,
        show_pip,
        on_toggle_fav,
        show_lyrics,
    )
}

/// Display the floating native Picture-in-Picture viewport window.
#[allow(clippy::too_many_arguments)]
pub fn show_pip_viewport(
    ctx: &egui::Context,
    player: &mut PlayerController,
    stream_controller: Option<&StreamController>,
    theme: &Theme,
    art_manager: &mut AlbumArtManager,
    show_pip: &mut bool,
    on_toggle_fav: &mut Option<i64>,
    pip_settings: &crate::config::PipSettings,
) {
    if !*show_pip {
        return;
    }

    let viewport_id = egui::ViewportId::from_hash_of("eclipse_pip_overlay");
    let initial_pos = match (pip_settings.pos_x, pip_settings.pos_y) {
        (Some(x), Some(y)) => Some(pos2(x, y)),
        _ => None,
    };

    let has_lyrics = pip_settings.show_desktop_lyrics && active_lyric_text(player).is_some();
    let target_height = if has_lyrics { 122.0 } else { 86.0 };
    let target_width = 386.0;

    let mut builder = egui::ViewportBuilder::default()
        .with_title("Eclipse Mini Player")
        .with_inner_size(vec2(target_width, target_height))
        .with_decorations(false)
        .with_always_on_top()
        .with_taskbar(false)
        .with_resizable(false);

    if pip_settings.click_through {
        builder = builder.with_mouse_passthrough(true);
    }

    if let Some(pos) = initial_pos {
        builder = builder.with_position(pos);
    } else {
        let screen = ctx.input(|i| i.screen_rect());
        if screen.width() > 500.0 && screen.height() > 400.0 {
            builder = builder.with_position(pos2(
                (screen.max.x - target_width - 20.0).max(20.0),
                (screen.max.y - target_height - 70.0).max(20.0),
            ));
        }
    }

    // Dynamically adjust inner size when tracks gain or lose lyrics
    ctx.send_viewport_cmd_to(viewport_id, egui::ViewportCommand::InnerSize(vec2(target_width, target_height)));

    ctx.show_viewport_immediate(
        viewport_id,
        builder,
        |ctx, _class| {
            if ctx.input(|i| i.viewport().close_requested()) {
                *show_pip = false;
            }

            // Apply Win32 native rounded corners (cached, only re-applied if size/HWND changes)
            let ppp = ctx.pixels_per_point();
            let phys_w = (target_width * ppp).round() as i32;
            let phys_h = (target_height * ppp).round() as i32;
            let corner_radius_px = (8.0 * ppp).round() as i32;
            apply_rounded_window_corners("Eclipse Mini Player", phys_w, phys_h, corner_radius_px);

            egui::CentralPanel::default()
                .frame(egui::Frame::none().fill(Color32::from_rgb(16, 16, 22)))
                .show(ctx, |ui| {
                    render_pip_overlay_content(
                        ui,
                        player,
                        stream_controller,
                        theme,
                        art_manager,
                        show_pip,
                        on_toggle_fav,
                        pip_settings.show_desktop_lyrics,
                    );
                });
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppConfig;
    use crate::library::model::Track;
    use crate::lyrics::{LyricLine, Lyrics};

    #[test]
    fn test_render_desktop_lyrics_empty_and_synced() {
        let ctx = egui::Context::default();
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let theme = Theme::from_name("Eclipse", None);
                let cfg = AppConfig::default();
                let mut player = PlayerController::new(&cfg);

                // 1. Without lyrics (calm placeholder)
                let resp1 = render_desktop_lyrics(ui, &player, &theme);
                assert!(resp1.rect.width() >= 320.0);

                // 2. With playing track & synced lyrics
                let track = Track {
                    id: 1,
                    path: "song.flac".into(),
                    title: "Song".into(),
                    artist: "Artist".into(),
                    album: "Album".into(),
                    genre: "Genre".into(),
                    year: 2020,
                    track_number: 1,
                    duration_secs: 180.0,
                    format: "FLAC".into(),
                    bitrate_kbps: 800,
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

                player.lyrics = Some(Lyrics {
                    lines: vec![
                        LyricLine {
                            timestamp_ms: 0,
                            text: "First active line".into(),
                        },
                        LyricLine {
                            timestamp_ms: 5000,
                            text: "Next upcoming line".into(),
                        },
                    ],
                    is_synced: true,
                    source: "LRCLIB".into(),
                });

                let resp2 = render_desktop_lyrics(ui, &player, &theme);
                assert!(resp2.rect.height() >= 40.0);
            });
        });
    }

    #[test]
    fn test_render_pip_card_interactions() {
        let ctx = egui::Context::default();
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let theme = Theme::from_name("Eclipse", None);
                let cfg = AppConfig::default();
                let mut player = PlayerController::new(&cfg);
                let mut art_manager = AlbumArtManager::new();
                let mut show_pip = true;
                let mut on_toggle_fav = None;

                // Empty playback state
                let resp = render_pip_card(
                    ui,
                    &mut player,
                    None,
                    &theme,
                    &mut art_manager,
                    &mut show_pip,
                    &mut on_toggle_fav,
                );
                assert!(resp.rect.width() > 0.0);

                // With active playing track
                let track = Track {
                    id: 42,
                    path: "test.flac".into(),
                    title: "Atmosphere".into(),
                    artist: "Joy Division".into(),
                    album: "Substance".into(),
                    genre: "Post-Punk".into(),
                    year: 1988,
                    track_number: 1,
                    duration_secs: 245.0,
                    format: "FLAC".into(),
                    bitrate_kbps: 900,
                    sample_rate: 44100,
                    channels: 2,
                    favorite: true,
                    has_art: false,
                    folder: "".into(),
                    album_artist: "".into(),
                    play_count: 5,
                    file_size: 2048,
                };
                player.load_queue(vec![track], 0);

                render_pip_card(
                    ui,
                    &mut player,
                    None,
                    &theme,
                    &mut art_manager,
                    &mut show_pip,
                    &mut on_toggle_fav,
                );
                assert!(show_pip);
            });
        });
    }

    #[test]
    fn test_pip_overlay_content_layout() {
        let ctx = egui::Context::default();
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let theme = Theme::from_name("Eclipse", None);
                let cfg = AppConfig::default();
                let mut player = PlayerController::new(&cfg);
                let mut art_manager = AlbumArtManager::new();
                let mut show_pip = true;
                let mut on_toggle_fav = None;

                // Render full content with lyrics setting on (no track playing -> 86.0 fallback)
                let resp_lyrics = render_pip_overlay_content(
                    ui,
                    &mut player,
                    None,
                    &theme,
                    &mut art_manager,
                    &mut show_pip,
                    &mut on_toggle_fav,
                    true,
                );
                assert_eq!(resp_lyrics.rect.width(), 386.0);
                assert_eq!(resp_lyrics.rect.height(), 86.0);

                // Render full content without lyrics
                let resp_no_lyrics = render_pip_overlay_content(
                    ui,
                    &mut player,
                    None,
                    &theme,
                    &mut art_manager,
                    &mut show_pip,
                    &mut on_toggle_fav,
                    false,
                );
                assert_eq!(resp_no_lyrics.rect.width(), 386.0);
                assert_eq!(resp_no_lyrics.rect.height(), 86.0);
            });
        });
    }

    #[test]
    fn test_unified_pip_card_sizing_and_lyrics() {
        let ctx = egui::Context::default();
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let theme = Theme::from_name("Eclipse", None);
                let cfg = AppConfig::default();
                let mut player = PlayerController::new(&cfg);
                let mut art_manager = AlbumArtManager::new();
                let mut show_pip = true;
                let mut on_toggle_fav = None;

                let track = Track {
                    id: 99,
                    path: "sample.flac".into(),
                    title: "Test Track".into(),
                    artist: "Test Artist".into(),
                    album: "Test Album".into(),
                    genre: "Ambient".into(),
                    year: 2024,
                    track_number: 1,
                    duration_secs: 300.0,
                    format: "FLAC".into(),
                    bitrate_kbps: 1000,
                    sample_rate: 44100,
                    channels: 2,
                    favorite: false,
                    has_art: false,
                    folder: "".into(),
                    album_artist: "".into(),
                    play_count: 0,
                    file_size: 2048,
                };
                player.load_queue(vec![track], 0);

                // Add lyrics
                player.lyrics = Some(Lyrics {
                    lines: vec![LyricLine {
                        timestamp_ms: 0,
                        text: "Floating in dark glass".into(),
                    }],
                    is_synced: true,
                    source: "LRCLIB".into(),
                });

                assert_eq!(active_lyric_text(&player), Some("Floating in dark glass".into()));

                let resp_with = render_unified_pip_card(
                    ui,
                    &mut player,
                    None,
                    &theme,
                    &mut art_manager,
                    &mut show_pip,
                    &mut on_toggle_fav,
                    true,
                );
                assert_eq!(resp_with.rect.width(), 386.0);
                assert_eq!(resp_with.rect.height(), 122.0);

                let resp_without = render_unified_pip_card(
                    ui,
                    &mut player,
                    None,
                    &theme,
                    &mut art_manager,
                    &mut show_pip,
                    &mut on_toggle_fav,
                    false,
                );
                assert_eq!(resp_without.rect.width(), 386.0);
                assert_eq!(resp_without.rect.height(), 86.0);
            });
        });
    }

    #[test]
    fn test_apply_rounded_window_corners_safety() {
        // Safe no-op on non-existent window handle
        apply_rounded_window_corners("NonExistentWindow_EclipseTest", 386, 122, 8);
        // Safe no-op on zero or negative dimensions
        apply_rounded_window_corners("Eclipse Mini Player", 0, 0, 8);
        apply_rounded_window_corners("Eclipse Mini Player", -1, 122, 8);
    }

    #[test]
    fn test_pip_mini_seekbar_stream() {
        let ctx = egui::Context::default();
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let theme = Theme::from_name("Eclipse", None);
                let cfg = AppConfig::default();
                let mut player = PlayerController::new(&cfg);
                let stream_ctrl = StreamController::new(vec![]);

                let stream_track = crate::streams::provider::StreamTrack::new(
                    "sp-pip",
                    "PiP Stream",
                    "Artist",
                    "Album",
                    210.0,
                    None,
                    "spotify:track:pip",
                    "spotify",
                );
                player.switch_to_stream("spotify", Some(stream_track));

                let resp = render_mini_seekbar(ui, &mut player, Some(&stream_ctrl), &theme, 90.0);
                assert_eq!(resp.rect.width(), 90.0);
            });
        });
    }

    #[test]
    fn test_pip_mini_seekbar_click_and_drag() {
        let ctx = egui::Context::default();
        let theme = Theme::from_name("Eclipse", None);
        let cfg = AppConfig::default();
        let mut player = PlayerController::new(&cfg);
        let stream_ctrl = StreamController::new(vec![]);

        let stream_track = crate::streams::provider::StreamTrack::new(
            "sp-pip-click",
            "Track",
            "Artist",
            "Album",
            200.0,
            None,
            "spotify:track:click",
            "spotify",
        );
        player.switch_to_stream("spotify", Some(stream_track));

        let pt = pos2(45.0, 8.0); // 50% on 90.0px seekbar

        // Frame 0: Warmup
        let mut input0 = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(500.0, 500.0))),
            time: Some(0.0),
            ..Default::default()
        };
        input0.events.push(egui::Event::PointerMoved(pt));
        let _ = ctx.run(input0, |ctx| {
            egui::CentralPanel::default().frame(egui::Frame::none()).show(ctx, |ui| {
                render_mini_seekbar(ui, &mut player, Some(&stream_ctrl), &theme, 90.0);
            });
        });

        // Frame 1: Pointer pressed
        let mut input1 = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(500.0, 500.0))),
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
            egui::CentralPanel::default().frame(egui::Frame::none()).show(ctx, |ui| {
                render_mini_seekbar(ui, &mut player, Some(&stream_ctrl), &theme, 90.0);
            });
        });

        // Frame 2: Pointer released -> clicked()
        let mut input2 = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(500.0, 500.0))),
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
            egui::CentralPanel::default().frame(egui::Frame::none()).show(ctx, |ui| {
                render_mini_seekbar(ui, &mut player, Some(&stream_ctrl), &theme, 90.0);
            });
        });

        let pip_seek_id = egui::Id::new("pip_mini_seekbar");
        let drag_state = ctx.data(|d| d.get_temp::<SeekbarDragState>(pip_seek_id)).unwrap_or_default();
        assert!(drag_state.last_committed_secs.is_some(), "PiP mini seekbar click must commit seek");
        let committed = drag_state.last_committed_secs.unwrap();
        assert!((committed - 100.0).abs() < 2.0, "Expected seek ~100.0s on 200s track, got {}", committed);
        assert!(!drag_state.is_dragging);
    }
}
