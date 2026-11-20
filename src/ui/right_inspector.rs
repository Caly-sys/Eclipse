//! Persistent Right-Hand Inspector Panel for Eclipse.
//!
//! Provides a dedicated dark-glass inspector panel displaying the currently
//! playing track card, segmented tab switcher (Queue, Lyrics, Details), and
//! rich interactive bodies for upcoming queue tracks, synchronized lyrics,
//! and complete audio stream/file technical metadata.
//!
//! 100% pure vector graphics - ZERO Unicode emojis.
//! Zero-panic guarantee - NO unwrap() or expect() calls.

use crate::audio::PlayerController;
use crate::library::format_time_secs;
use crate::streams::controller::StreamController;
use crate::ui::art::AlbumArtManager;
use crate::ui::fullscreen_lyrics::FullscreenLyrics;
use crate::ui::icons::{paint_icon, Icon};
use crate::ui::playerbar::current_playback_time;
use crate::ui::theme::Theme;
use egui::{pos2, vec2, Color32, Rect, RichText, Rounding, ScrollArea, Stroke};

// ============================================================================
// Design tokens matching signature Bento theme & dark glass aesthetic
// ============================================================================

/// Bento surface fill (#101016)
pub const BG_PANEL: Color32 = Color32::from_rgb(16, 16, 22);

/// Elevated card sub-surface (#171720)
pub const BG_CARD: Color32 = Color32::from_rgb(23, 23, 32);

/// Subtle interactive hover highlight for table rows and buttons (#20202C)
pub const BG_CARD_HOVER: Color32 = Color32::from_rgb(32, 32, 44);

/// 1.0px hairline edge for bento frames and dividers
pub const BORDER_HAIRLINE: Color32 = Color32::from_rgba_premultiplied(71, 71, 71, 16);

/// Primary vibrant brand accent (#9C6CFF)
pub const ACCENT_PURPLE: Color32 = Color32::from_rgb(156, 108, 255);

/// Soft pill fill for active navigation items and active tabs
pub const ACCENT_PURPLE_SOFT: Color32 = Color32::from_rgba_premultiplied(59, 39, 102, 34);

/// High-contrast crisp text for track titles and active lyrics (#F2F2F8)
pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(242, 242, 248);

/// Muted secondary text for artists, album names, and keys (#9898A8)
pub const TEXT_MUTED: Color32 = Color32::from_rgb(152, 152, 168);

/// Faint timestamps and inactive lyrics (#585868)
pub const TEXT_FAINT: Color32 = Color32::from_rgb(88, 88, 104);

// Hi-Res Badge Tokens (Dark Amber-Gold Glass)
pub const BADGE_HIRES_BG: Color32 = Color32::from_rgba_premultiplied(101, 78, 21, 36);
pub const BADGE_HIRES_BORDER: Color32 = Color32::from_rgba_premultiplied(205, 162, 54, 170);
pub const BADGE_HIRES_TEXT: Color32 = Color32::from_rgb(255, 224, 130);

// Standard Lossless Badge Tokens (Purple Glass)
pub const BADGE_LOSSLESS_BG: Color32 = Color32::from_rgba_premultiplied(59, 39, 102, 34);
pub const BADGE_LOSSLESS_BORDER: Color32 = Color32::from_rgba_premultiplied(118, 81, 196, 140);
pub const BADGE_LOSSLESS_TEXT: Color32 = Color32::from_rgb(220, 200, 255);

// Spacing & Corner Radii
pub const BENTO_RADIUS: f32 = 12.0_f32;
pub const CARD_RADIUS: f32 = 8.0_f32;
pub const ART_RADIUS: f32 = 10.0_f32;
pub const RIGHT_INSPECTOR_WIDTH: f32 = 310.0_f32;

/// Active tab in the inspector panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InspectorTab {
    #[default]
    Queue,
    Lyrics,
    Details,
}

/// Action dispatched by interactive elements inside the right inspector.
#[derive(Debug, Clone, PartialEq)]
pub enum InspectorAction {
    None,
    SeekTo(f64),
    PlayUpcomingIndex(usize),
    ToggleFavorite(i64),
    OpenFullscreen,
}

/// Context borrowing necessary player, streaming, artwork, and theme subsystems.
pub struct InspectorContext<'a> {
    pub player: &'a mut PlayerController,
    pub stream_controller: Option<&'a StreamController>,
    pub art_manager: &'a mut AlbumArtManager,
    pub theme: &'a Theme,
    pub active_tab: InspectorTab,
}

/// Truncate string to `max_chars` with ASCII ellipsis if longer.
fn truncate_str(s: &str, max_chars: usize) -> String {
    if max_chars <= 3 {
        return s.chars().take(max_chars).collect();
    }
    if s.chars().count() > max_chars {
        let truncated: String = s.chars().take(max_chars.saturating_sub(3)).collect();
        format!("{truncated}...")
    } else {
        s.to_string()
    }
}

/// Format raw byte count into human-readable memory size.
fn format_file_size(bytes: u64) -> String {
    if bytes == 0 {
        return "-".to_string();
    }
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Render a compact audio quality badge (Amber-Gold for Hi-Res, Soft Purple for Lossless, dark for Lossy).
fn render_quality_badge(
    ui: &mut egui::Ui,
    label: &str,
    is_hires: bool,
    is_lossless: bool,
) {
    let (bg, border, text_color) = if is_hires {
        (BADGE_HIRES_BG, BADGE_HIRES_BORDER, BADGE_HIRES_TEXT)
    } else if is_lossless {
        (BADGE_LOSSLESS_BG, BADGE_LOSSLESS_BORDER, BADGE_LOSSLESS_TEXT)
    } else {
        (
            Color32::from_rgb(24, 24, 28),
            Color32::from_white_alpha(20),
            TEXT_MUTED,
        )
    };

    ui.scope(|ui| {
        let padding = vec2(6.0_f32, 2.0_f32);
        let font_id = egui::FontId::proportional(10.0_f32);
        let galley = ui.painter().layout_no_wrap(label.to_string(), font_id, text_color);
        let desired_size = galley.size() + padding * 2.0_f32;
        let (rect, _) = ui.allocate_exact_size(desired_size, egui::Sense::hover());

        ui.painter().rect_filled(rect, Rounding::same(5.0_f32), bg);
        ui.painter().rect_stroke(rect, Rounding::same(5.0_f32), Stroke::new(1.0_f32, border));
        ui.painter().galley(rect.min + padding, galley, text_color);
    });
}

/// Render a single scannable key-value row in the details view.
fn render_details_row(ui: &mut egui::Ui, key: &str, val: &str, bg_tint: bool) {
    let row_height = 24.0_f32;
    let avail_w = ui.available_width();
    let (rect, resp) = ui.allocate_exact_size(vec2(avail_w, row_height), egui::Sense::hover());

    if bg_tint {
        ui.painter().rect_filled(
            rect,
            Rounding::same(4.0_f32),
            Color32::from_rgba_premultiplied(255, 255, 255, 4),
        );
    }
    if resp.hovered() {
        ui.painter().rect_filled(
            rect,
            Rounding::same(4.0_f32),
            Color32::from_rgba_premultiplied(255, 255, 255, 8),
        );
    }

    let key_w = 90.0_f32;
    let text_pad_y = (row_height - 14.0_f32) * 0.5_f32;

    // Key label (TEXT_MUTED, 11.5px)
    let key_galley = ui.painter().layout_no_wrap(
        key.to_string(),
        egui::FontId::proportional(11.5_f32),
        TEXT_MUTED,
    );
    ui.painter().galley(pos2(rect.min.x + 6.0_f32, rect.min.y + text_pad_y), key_galley, TEXT_MUTED);

    // Value label (TEXT_PRIMARY, 12.0px)
    let val_x = rect.min.x + key_w + 4.0_f32;
    let val_max_w = (rect.max.x - val_x - 6.0_f32).max(10.0_f32);
    let val_truncated = truncate_str(val, (val_max_w / 7.0_f32) as usize + 3);
    let val_galley = ui.painter().layout_no_wrap(
        val_truncated,
        egui::FontId::proportional(12.0_f32),
        TEXT_PRIMARY,
    );
    ui.painter().galley(pos2(val_x, rect.min.y + text_pad_y), val_galley, TEXT_PRIMARY);

    if val.len() > 16 {
        resp.on_hover_text(val);
    }
}

/// Render the Top Section: Now Playing Card with clipped square artwork, titles, and quality badge.
fn render_now_playing_card(
    ui: &mut egui::Ui,
    ctx: &mut InspectorContext,
) -> InspectorAction {
    let mut action = InspectorAction::None;
    let avail_w = ui.available_width();

    let current_local = ctx.player.current_track().cloned();
    let current_stream = ctx.player.active_stream_track.clone();

    // 1. Cover Art (Large Square)
    let art_size = (avail_w - 16.0_f32).clamp(160.0_f32, 260.0_f32);
    let pad_x = ((avail_w - art_size) * 0.5_f32).max(0.0_f32);

    ui.horizontal(|ui| {
        ui.add_space(pad_x);
        let (art_rect, art_resp) = ui.allocate_exact_size(vec2(art_size, art_size), egui::Sense::click());

        // Artwork path: local track path or stream art_url/uri
        let art_path = current_local
            .as_ref()
            .map(|t| t.path.as_str())
            .or_else(|| {
                current_stream
                    .as_ref()
                    .and_then(|s| s.art_url.as_deref().or(Some(s.uri.as_str())))
            });

        let tex = art_path.and_then(|p| ctx.art_manager.get_or_load(ui.ctx(), p));

        // ALBUM ART CLIPPING RULE: Always render album art using
        // egui::Image::new(egui::load::SizedTexture::new(tex.id(), rect.size())).rounding(Rounding::same(ART_RADIUS))
        // painted at rect. Never paint raw unrounded images behind borders!
        let art_rounding = Rounding::same(ART_RADIUS);
        let mut painted_cover = false;
        if let Some(texture) = tex {
            let img = egui::Image::new(egui::load::SizedTexture::new(texture.id(), art_rect.size()))
                .rounding(art_rounding);
            img.paint_at(ui, art_rect);
            painted_cover = true;
        }

        if !painted_cover {
            ui.painter().rect_filled(art_rect, art_rounding, BG_CARD);
            let disc_size = (art_size * 0.4_f32).max(24.0_f32);
            let disc_rect = Rect::from_center_size(art_rect.center(), vec2(disc_size, disc_size));
            paint_icon(ui.painter(), Icon::Disc, disc_rect, Color32::from_white_alpha(35));
        }

        // Hairline border
        ui.painter().rect_stroke(art_rect, art_rounding, Stroke::new(1.0_f32, BORDER_HAIRLINE));

        // Hover overlay: darken slightly and show vector Expand icon
        if art_resp.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            ui.painter().rect_filled(
                art_rect,
                art_rounding,
                Color32::from_rgba_premultiplied(0, 0, 0, 80),
            );
            let exp_size = (art_size * 0.16_f32).clamp(20.0_f32, 32.0_f32);
            let exp_rect = Rect::from_center_size(art_rect.center(), vec2(exp_size, exp_size));
            paint_icon(
                ui.painter(),
                Icon::Expand,
                exp_rect,
                Color32::from_rgba_premultiplied(255, 255, 255, 220),
            );
        }

        if art_resp.clicked() {
            action = InspectorAction::OpenFullscreen;
        }
        art_resp.on_hover_text("Click to enter Fullscreen mode");
    });

    ui.add_space(8.0_f32);

    // 2. Track Title, Artist, Album, and Quality Details
    let (title, artist, album, is_hires, is_lossless, badge_label, fav_info) = if let Some(ref track) = current_local {
        (
            track.display_title().to_string(),
            track.display_artist().to_string(),
            track.display_album().to_string(),
            track.is_hires(),
            track.is_lossless(),
            track.quality_badge_text().to_string(),
            Some((track.id, track.favorite)),
        )
    } else if let Some(ref st) = current_stream {
        (
            st.title.clone(),
            st.artist.clone(),
            st.album.clone(),
            false,
            false,
            st.provider.to_uppercase(),
            st.local_track_id.map(|id| (id, false)),
        )
    } else {
        (
            "Nothing Playing".to_string(),
            "Select a track to play".to_string(),
            "-".to_string(),
            false,
            false,
            "IDLE".to_string(),
            None,
        )
    };

    // Title Row with Heart button
    ui.horizontal(|ui| {
        let max_title_chars = ((avail_w - 40.0_f32) / 8.0_f32) as usize + 2;
        let trunc_title = truncate_str(&title, max_title_chars);
        let title_label = ui.label(RichText::new(trunc_title).size(14.5_f32).strong().color(TEXT_PRIMARY));
        if title.len() > 18 {
            title_label.on_hover_text(&title);
        }

        if let Some((track_id, is_fav)) = fav_info {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let (heart_rect, heart_resp) = ui.allocate_exact_size(vec2(18.0_f32, 18.0_f32), egui::Sense::click());
                if heart_resp.hovered() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                }
                if is_fav {
                    paint_icon(ui.painter(), Icon::Heart, heart_rect, ACCENT_PURPLE);
                } else {
                    paint_icon(
                        ui.painter(),
                        Icon::HeartOutline,
                        heart_rect,
                        if heart_resp.hovered() { TEXT_PRIMARY } else { TEXT_MUTED },
                    );
                }
                if heart_resp.clicked() {
                    action = InspectorAction::ToggleFavorite(track_id);
                }
            });
        }
    });

    ui.add_space(2.0_f32);

    // Artist and Album subtitle
    let artist_album = if album == "-" || album.is_empty() {
        artist.clone()
    } else {
        format!("{} • {}", artist, album)
    };
    let max_sub_chars = ((avail_w - 20.0_f32) / 7.0_f32) as usize + 2;
    let trunc_sub = truncate_str(&artist_album, max_sub_chars);
    let sub_label = ui.label(RichText::new(trunc_sub).size(12.0_f32).color(TEXT_MUTED));
    if artist_album.len() > 24 {
        sub_label.on_hover_text(&artist_album);
    }

    ui.add_space(6.0_f32);

    // Audio Quality Badge
    render_quality_badge(ui, &badge_label, is_hires, is_lossless);

    action
}

/// Render the Middle Section: Segmented 3-Tab Switcher (Queue, Lyrics, Details).
fn render_tab_switcher(ui: &mut egui::Ui, active_tab: InspectorTab) -> InspectorTab {
    let mut selected = active_tab;
    let track_height = 32.0_f32;
    let track_width = ui.available_width();
    let (track_rect, _) = ui.allocate_exact_size(vec2(track_width, track_height), egui::Sense::hover());

    let track_rounding = Rounding::same(CARD_RADIUS);
    ui.painter().rect_filled(track_rect, track_rounding, BG_CARD);
    ui.painter().rect_stroke(track_rect, track_rounding, Stroke::new(1.0_f32, BORDER_HAIRLINE));

    let tabs = [
        (InspectorTab::Queue, "Queue"),
        (InspectorTab::Lyrics, "Lyrics"),
        (InspectorTab::Details, "Details"),
    ];

    let pad = 2.0_f32;
    let inner_rect = track_rect.shrink(pad);
    let tab_width = inner_rect.width() / 3.0_f32;

    for (i, (tab, label)) in tabs.iter().enumerate() {
        let seg_rect = Rect::from_min_size(
            pos2(inner_rect.min.x + i as f32 * tab_width, inner_rect.min.y),
            vec2(tab_width, inner_rect.height()),
        );
        let seg_resp = ui.allocate_rect(seg_rect, egui::Sense::click());
        let is_active = *tab == active_tab;
        let is_hovered = seg_resp.hovered();

        if seg_resp.clicked() {
            selected = *tab;
        }

        let seg_rounding = Rounding::same(6.0_f32);

        if is_active {
            ui.painter().rect_filled(seg_rect, seg_rounding, ACCENT_PURPLE_SOFT);
            ui.painter().rect_stroke(
                seg_rect,
                seg_rounding,
                Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(156, 108, 255, 110)),
            );
        } else if is_hovered {
            ui.painter().rect_filled(seg_rect, seg_rounding, BG_CARD_HOVER);
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }

        let text_color = if is_active || is_hovered {
            TEXT_PRIMARY
        } else {
            TEXT_MUTED
        };

        let font_id = if is_active {
            egui::FontId::proportional(12.0_f32)
        } else {
            egui::FontId::proportional(11.5_f32)
        };

        let galley = ui.painter().layout_no_wrap(label.to_string(), font_id, text_color);
        let text_pos = pos2(
            seg_rect.center().x - galley.size().x * 0.5_f32,
            seg_rect.center().y - galley.size().y * 0.5_f32,
        );
        ui.painter().galley(text_pos, galley, text_color);
    }

    selected
}

/// Render the Queue tab content: "Up Next" header and compact upcoming track rows.
fn render_queue_tab(ui: &mut egui::Ui, ctx: &mut InspectorContext) -> InspectorAction {
    let mut action = InspectorAction::None;
    let upcoming = ctx.player.peek_upcoming_tracks(20);

    // Header
    ui.horizontal(|ui| {
        ui.label(RichText::new("Up Next").size(13.5_f32).strong().color(TEXT_PRIMARY));
        ui.add_space(4.0_f32);

        let count_str = format!("{}", upcoming.len());
        let font_id = egui::FontId::proportional(10.5_f32);
        let galley = ui.painter().layout_no_wrap(count_str, font_id, ACCENT_PURPLE);
        let badge_size = galley.size() + vec2(10.0_f32, 3.0_f32);
        let (b_rect, _) = ui.allocate_exact_size(badge_size, egui::Sense::hover());
        ui.painter().rect_filled(b_rect, Rounding::same(10.0_f32), ACCENT_PURPLE_SOFT);
        ui.painter().rect_stroke(
            b_rect,
            Rounding::same(10.0_f32),
            Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(156, 108, 255, 90)),
        );
        ui.painter().galley(b_rect.min + vec2(5.0_f32, 1.5_f32), galley, ACCENT_PURPLE);
    });

    ui.add_space(6.0_f32);

    if upcoming.is_empty() {
        ui.add_space(30.0_f32);
        ui.vertical_centered(|ui| {
            let (icon_rect, _) = ui.allocate_exact_size(vec2(36.0_f32, 36.0_f32), egui::Sense::hover());
            paint_icon(ui.painter(), Icon::Disc, icon_rect, Color32::from_white_alpha(35));
            ui.add_space(10.0_f32);
            ui.label(
                RichText::new("Queue is empty. Add songs from your library.")
                    .size(12.5_f32)
                    .color(TEXT_MUTED),
            );
        });
        return action;
    }

    ui.spacing_mut().item_spacing = vec2(0.0_f32, 4.0_f32);

    for (idx, track) in upcoming.iter().enumerate() {
        let row_height = 42.0_f32;
        let row_w = ui.available_width();
        let (row_rect, row_resp) = ui.allocate_exact_size(
            vec2(row_w, row_height),
            egui::Sense::click(),
        );

        let is_hovered = row_resp.hovered();
        if is_hovered {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            ui.painter().rect_filled(row_rect, Rounding::same(6.0_f32), BG_CARD_HOVER);
            ui.painter().rect_stroke(
                row_rect,
                Rounding::same(6.0_f32),
                Stroke::new(1.0_f32, BORDER_HAIRLINE),
            );
        }

        if row_resp.clicked() {
            action = InspectorAction::PlayUpcomingIndex(idx);
        }

        // 1. Index # (1..N)
        let idx_str = format!("{}", idx + 1);
        let idx_galley = ui.painter().layout_no_wrap(
            idx_str,
            egui::FontId::proportional(11.0_f32),
            TEXT_MUTED,
        );
        let idx_pos = pos2(
            row_rect.min.x + 6.0_f32,
            row_rect.min.y + (row_height - idx_galley.size().y) * 0.5_f32,
        );
        ui.painter().galley(idx_pos, idx_galley, TEXT_MUTED);

        // 2. Mini Album Art (28x28)
        let mini_x = row_rect.min.x + 24.0_f32;
        let mini_art_rect = Rect::from_min_size(
            pos2(mini_x, row_rect.min.y + (row_height - 28.0_f32) * 0.5_f32),
            vec2(28.0_f32, 28.0_f32),
        );
        let mini_rounding = Rounding::same(5.0_f32);
        let mut painted_mini = false;
        if let Some(tex) = ctx.art_manager.get_or_load(ui.ctx(), &track.path) {
            let img = egui::Image::new(egui::load::SizedTexture::new(tex.id(), mini_art_rect.size()))
                .rounding(mini_rounding);
            img.paint_at(ui, mini_art_rect);
            painted_mini = true;
        }
        if !painted_mini {
            ui.painter().rect_filled(mini_art_rect, mini_rounding, BG_CARD);
            let disc_r = mini_art_rect.shrink(6.0_f32);
            paint_icon(ui.painter(), Icon::Music, disc_r, Color32::from_white_alpha(40));
        }
        ui.painter().rect_stroke(mini_art_rect, mini_rounding, Stroke::new(1.0_f32, BORDER_HAIRLINE));

        // 3. Duration on right
        let dur_str = format_time_secs(track.duration_secs);
        let dur_galley = ui.painter().layout_no_wrap(
            dur_str,
            egui::FontId::proportional(11.0_f32),
            TEXT_MUTED,
        );
        let dur_w = dur_galley.size().x;
        let dur_pos = pos2(
            row_rect.max.x - dur_w - 8.0_f32,
            row_rect.min.y + (row_height - dur_galley.size().y) * 0.5_f32,
        );
        ui.painter().galley(dur_pos, dur_galley, TEXT_MUTED);

        // 4. Title and Artist column
        let text_start_x = mini_x + 34.0_f32;
        let text_max_w = (row_rect.max.x - dur_w - text_start_x - 14.0_f32).max(20.0_f32);

        let title_trunc = truncate_str(track.display_title(), (text_max_w / 7.2_f32) as usize + 2);
        let artist_trunc = truncate_str(track.display_artist(), (text_max_w / 6.8_f32) as usize + 2);

        let title_galley = ui.painter().layout_no_wrap(
            title_trunc,
            egui::FontId::proportional(12.0_f32),
            TEXT_PRIMARY,
        );
        let artist_galley = ui.painter().layout_no_wrap(
            artist_trunc,
            egui::FontId::proportional(10.5_f32),
            TEXT_MUTED,
        );

        let title_y = row_rect.min.y + 6.0_f32;
        let artist_y = row_rect.min.y + 22.0_f32;

        ui.painter().galley(pos2(text_start_x, title_y), title_galley, TEXT_PRIMARY);
        ui.painter().galley(pos2(text_start_x, artist_y), artist_galley, TEXT_MUTED);
    }

    action
}

/// Render the Lyrics tab content: Synchronized lyrics highlighting and interactive seek-to.
fn render_lyrics_tab(ui: &mut egui::Ui, ctx: &mut InspectorContext) -> InspectorAction {
    let mut action = InspectorAction::None;

    let Some(ref lrc) = ctx.player.lyrics else {
        // Friendly empty state
        ui.add_space(30.0_f32);
        ui.vertical_centered(|ui| {
            let (icon_rect, _) = ui.allocate_exact_size(vec2(36.0_f32, 36.0_f32), egui::Sense::hover());
            paint_icon(ui.painter(), Icon::Mic, icon_rect, Color32::from_white_alpha(35));
            ui.add_space(10.0_f32);
            ui.label(RichText::new("No synced lyrics available").size(13.0_f32).strong().color(TEXT_MUTED));
            ui.add_space(4.0_f32);
            ui.label(RichText::new("Play a song with synchronized LRC lyrics").size(11.0_f32).color(TEXT_FAINT));
        });
        return action;
    };

    let fs_lyrics = FullscreenLyrics::from_lyrics(lrc);
    if fs_lyrics.lines.is_empty() {
        ui.add_space(30.0_f32);
        ui.vertical_centered(|ui| {
            let (icon_rect, _) = ui.allocate_exact_size(vec2(36.0_f32, 36.0_f32), egui::Sense::hover());
            paint_icon(ui.painter(), Icon::Mic, icon_rect, Color32::from_white_alpha(35));
            ui.add_space(10.0_f32);
            ui.label(RichText::new("No synced lyrics available").size(13.0_f32).strong().color(TEXT_MUTED));
        });
        return action;
    }

    let (pos_secs, _) = current_playback_time(ctx.player, ctx.stream_controller);
    let active_idx = fs_lyrics.active_line_index(pos_secs);

    let auto_scroll_id = ui.id().with("inspector_last_lyric_line");
    let last_active: Option<usize> = ui.data(|d| d.get_temp(auto_scroll_id));
    let should_scroll = active_idx.is_some() && active_idx != last_active;
    if should_scroll {
        ui.data_mut(|d| d.insert_temp(auto_scroll_id, active_idx));
    }

    ui.add_space(4.0_f32);
    ui.spacing_mut().item_spacing = vec2(0.0_f32, 6.0_f32);

    for (i, line) in fs_lyrics.lines.iter().enumerate() {
        let is_active = Some(i) == active_idx;
        let is_instrumental = line.is_instrumental || line.text.trim().is_empty() || line.text == "...";
        let display_text = if is_instrumental {
            "..."
        } else {
            line.text.trim()
        };

        let font_size = if is_active { 14.5_f32 } else { 12.5_f32 };
        let text_color = if is_active {
            ACCENT_PURPLE
        } else {
            TEXT_FAINT
        };

        let galley = ui.painter().layout(
            display_text.to_string(),
            egui::FontId::proportional(font_size),
            text_color,
            ui.available_width() - 16.0_f32,
        );

        let row_height = (galley.size().y + 10.0_f32).max(28.0_f32);
        let (row_rect, row_resp) = ui.allocate_exact_size(
            vec2(ui.available_width(), row_height),
            egui::Sense::click(),
        );

        let is_hovered = row_resp.hovered();
        if is_hovered {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }

        let rounding = Rounding::same(6.0_f32);
        if is_active {
            ui.painter().rect_filled(
                row_rect,
                rounding,
                Color32::from_rgba_premultiplied(156, 108, 255, 25),
            );
            ui.painter().rect_stroke(
                row_rect,
                rounding,
                Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(156, 108, 255, 65)),
            );
        } else if is_hovered {
            ui.painter().rect_filled(row_rect, rounding, BG_CARD_HOVER);
        }

        let final_text_color = if is_active || is_hovered {
            TEXT_PRIMARY
        } else {
            TEXT_FAINT
        };

        let text_x = row_rect.min.x + 8.0_f32;
        let text_y = row_rect.min.y + (row_height - galley.size().y) * 0.5_f32;
        ui.painter().galley(pos2(text_x, text_y), galley, final_text_color);

        if row_resp.clicked() {
            action = InspectorAction::SeekTo(line.time_secs);
        }

        if is_active && should_scroll {
            row_resp.scroll_to_me(Some(egui::Align::Center));
        }
    }

    action
}

/// Render the Details tab content: Scannable key-value properties table.
fn render_details_tab(ui: &mut egui::Ui, ctx: &mut InspectorContext) {
    let current_local = ctx.player.current_track();
    let current_stream = ctx.player.active_stream_track.as_ref();

    if current_local.is_none() && current_stream.is_none() {
        ui.add_space(30.0_f32);
        ui.vertical_centered(|ui| {
            let (icon_rect, _) = ui.allocate_exact_size(vec2(36.0_f32, 36.0_f32), egui::Sense::hover());
            paint_icon(ui.painter(), Icon::Info, icon_rect, Color32::from_white_alpha(35));
            ui.add_space(10.0_f32);
            ui.label(RichText::new("No track currently playing").size(13.0_f32).strong().color(TEXT_MUTED));
            ui.add_space(4.0_f32);
            ui.label(RichText::new("Track details and technical specs will appear here").size(11.0_f32).color(TEXT_FAINT));
        });
        return;
    }

    let items: Vec<(&str, String)> = if let Some(track) = current_local {
        vec![
            ("Title", track.display_title().to_string()),
            ("Artist", track.display_artist().to_string()),
            ("Album", track.display_album().to_string()),
            ("Genre", if track.genre.trim().is_empty() { "-".to_string() } else { track.genre.clone() }),
            ("Year", if track.year == 0 { "-".to_string() } else { format!("{}", track.year) }),
            ("Track #", if track.track_number == 0 { "-".to_string() } else { format!("{}", track.track_number) }),
            ("File Format", if track.format.trim().is_empty() { "AUDIO".to_string() } else { track.format.to_uppercase() }),
            ("Codec", if track.format.trim().is_empty() { "Unknown".to_string() } else { track.format.to_uppercase() }),
            ("Bitrate", if track.bitrate_kbps > 0 { format!("{} kbps", track.bitrate_kbps) } else { "-".to_string() }),
            ("Sample Rate", if track.sample_rate > 0 { format!("{} Hz ({:.1} kHz)", track.sample_rate, track.sample_rate as f64 / 1000.0) } else { "-".to_string() }),
            ("Channels", match track.channels {
                1 => "Mono (1 ch)".to_string(),
                2 => "Stereo (2 ch)".to_string(),
                ch if ch > 2 => format!("{} channels", ch),
                _ => "-".to_string(),
            }),
            ("Play Count", format!("{} plays", track.play_count)),
            ("File Size", format_file_size(track.file_size)),
        ]
    } else if let Some(st) = current_stream {
        vec![
            ("Title", st.title.clone()),
            ("Artist", st.artist.clone()),
            ("Album", st.album.clone()),
            ("Genre", "Streaming".to_string()),
            ("Year", "-".to_string()),
            ("Track #", "-".to_string()),
            ("File Format", "Remote Stream".to_string()),
            ("Codec", st.provider.to_uppercase()),
            ("Bitrate", "Variable (High)".to_string()),
            ("Sample Rate", "44100 Hz (44.1 kHz)".to_string()),
            ("Channels", "Stereo (2 ch)".to_string()),
            ("Play Count", "-".to_string()),
            ("File Size", "-".to_string()),
        ]
    } else {
        Vec::new()
    };

    ui.add_space(4.0_f32);
    for (i, (key, val)) in items.iter().enumerate() {
        render_details_row(ui, key, val, i % 2 == 1);
    }
}

/// Main entry point for the Right-Hand Inspector component.
///
/// Returns the updated `InspectorTab` and any dispatched `InspectorAction`.
pub fn render_right_inspector(
    ui: &mut egui::Ui,
    ctx: &mut InspectorContext,
) -> (InspectorTab, InspectorAction) {
    let panel_w = ui.available_width().clamp(300.0_f32, 320.0_f32);
    ui.set_width(panel_w);

    let mut action = InspectorAction::None;

    let outer_frame = egui::Frame::none()
        .fill(BG_PANEL)
        .rounding(Rounding::same(BENTO_RADIUS))
        .inner_margin(egui::Margin::same(10.0_f32))
        .stroke(Stroke::new(1.0_f32, BORDER_HAIRLINE));

    outer_frame.show(ui, |ui| {
        ui.vertical(|ui| {
            // 1. Top Section - Now Playing Card
            let np_action = render_now_playing_card(ui, ctx);
            if np_action != InspectorAction::None {
                action = np_action;
            }

            ui.add_space(10.0_f32);

            // 2. Middle Section - Segmented 3-Tab Switcher
            let new_tab = render_tab_switcher(ui, ctx.active_tab);
            ctx.active_tab = new_tab;

            ui.add_space(10.0_f32);

            // 3. Body Section - Tab Contents in ScrollArea::vertical()
            ScrollArea::vertical()
                .id_salt("eclipse_right_inspector_scroll")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    match ctx.active_tab {
                        InspectorTab::Queue => {
                            let q_action = render_queue_tab(ui, ctx);
                            if q_action != InspectorAction::None {
                                action = q_action;
                            }
                        }
                        InspectorTab::Lyrics => {
                            let l_action = render_lyrics_tab(ui, ctx);
                            if l_action != InspectorAction::None {
                                action = l_action;
                            }
                        }
                        InspectorTab::Details => {
                            render_details_tab(ui, ctx);
                        }
                    }
                });
        });
    });

    (ctx.active_tab, action)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppConfig;
    use crate::library::model::Track;

    fn sample_test_track(id: i64, title: &str, artist: &str, format: &str) -> Track {
        Track {
            id,
            path: format!("test_track_{}.flac", id),
            title: title.to_string(),
            artist: artist.to_string(),
            album: "Test Album".to_string(),
            album_artist: "".to_string(),
            genre: "Electronic".to_string(),
            year: 2024,
            track_number: 1,
            duration_secs: 215.0,
            bitrate_kbps: 1411,
            sample_rate: 44100,
            channels: 2,
            format: format.to_string(),
            file_size: 28_400_000,
            play_count: 7,
            favorite: false,
            has_art: false,
            folder: "".to_string(),
        }
    }

    #[test]
    fn test_default_inspector_tab_and_action_equality() {
        assert_eq!(InspectorTab::default(), InspectorTab::Queue);
        assert_ne!(InspectorTab::Queue, InspectorTab::Lyrics);
        assert_ne!(InspectorTab::Lyrics, InspectorTab::Details);
        assert_ne!(InspectorTab::Details, InspectorTab::Queue);

        assert_eq!(InspectorAction::None, InspectorAction::None);
        assert_eq!(InspectorAction::SeekTo(15.5), InspectorAction::SeekTo(15.5));
        assert_ne!(InspectorAction::SeekTo(10.0), InspectorAction::SeekTo(20.0));
        assert_eq!(InspectorAction::PlayUpcomingIndex(3), InspectorAction::PlayUpcomingIndex(3));
        assert_ne!(InspectorAction::PlayUpcomingIndex(1), InspectorAction::PlayUpcomingIndex(2));
        assert_eq!(InspectorAction::ToggleFavorite(42), InspectorAction::ToggleFavorite(42));
        assert_ne!(InspectorAction::ToggleFavorite(10), InspectorAction::ToggleFavorite(20));
        assert_eq!(InspectorAction::OpenFullscreen, InspectorAction::OpenFullscreen);
        assert_ne!(InspectorAction::OpenFullscreen, InspectorAction::None);
    }

    #[test]
    fn test_empty_state_rendering_no_panic() {
        let ctx = egui::Context::default();
        let cfg = AppConfig::default();
        let mut player = PlayerController::new(&cfg);
        let mut art_manager = AlbumArtManager::new();
        let theme = Theme::from_name("Eclipse", None);

        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                // Test Queue tab on empty player
                let mut insp_ctx_queue = InspectorContext {
                    player: &mut player,
                    stream_controller: None,
                    art_manager: &mut art_manager,
                    theme: &theme,
                    active_tab: InspectorTab::Queue,
                };
                let (tab_q, act_q) = render_right_inspector(ui, &mut insp_ctx_queue);
                assert_eq!(tab_q, InspectorTab::Queue);
                assert_eq!(act_q, InspectorAction::None);

                // Test Lyrics tab on empty player
                let mut insp_ctx_lyrics = InspectorContext {
                    player: &mut player,
                    stream_controller: None,
                    art_manager: &mut art_manager,
                    theme: &theme,
                    active_tab: InspectorTab::Lyrics,
                };
                let (tab_l, act_l) = render_right_inspector(ui, &mut insp_ctx_lyrics);
                assert_eq!(tab_l, InspectorTab::Lyrics);
                assert_eq!(act_l, InspectorAction::None);

                // Test Details tab on empty player
                let mut insp_ctx_details = InspectorContext {
                    player: &mut player,
                    stream_controller: None,
                    art_manager: &mut art_manager,
                    theme: &theme,
                    active_tab: InspectorTab::Details,
                };
                let (tab_d, act_d) = render_right_inspector(ui, &mut insp_ctx_details);
                assert_eq!(tab_d, InspectorTab::Details);
                assert_eq!(act_d, InspectorAction::None);
            });
        });
    }

    #[test]
    fn test_queue_rendering_and_interaction() {
        let ctx = egui::Context::default();
        let cfg = AppConfig::default();
        let mut player = PlayerController::new(&cfg);
        let mut art_manager = AlbumArtManager::new();
        let theme = Theme::from_name("Eclipse", None);

        let tracks = vec![
            sample_test_track(1, "Track One", "Artist Alpha", "FLAC"),
            sample_test_track(2, "Track Two", "Artist Beta", "FLAC"),
            sample_test_track(3, "Track Three", "Artist Gamma", "MP3"),
        ];
        player.load_queue(tracks, 0);

        let upcoming = player.peek_upcoming_tracks(20);
        assert_eq!(upcoming.len(), 2);
        assert_eq!(upcoming[0].title, "Track Two");
        assert_eq!(upcoming[1].title, "Track Three");

        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let mut insp_ctx = InspectorContext {
                    player: &mut player,
                    stream_controller: None,
                    art_manager: &mut art_manager,
                    theme: &theme,
                    active_tab: InspectorTab::Queue,
                };
                let (tab, action) = render_right_inspector(ui, &mut insp_ctx);
                assert_eq!(tab, InspectorTab::Queue);
                assert_eq!(action, InspectorAction::None);
            });
        });
    }

    #[test]
    fn test_synced_lyrics_rendering_no_panic() {
        let ctx = egui::Context::default();
        let cfg = AppConfig::default();
        let mut player = PlayerController::new(&cfg);
        let mut art_manager = AlbumArtManager::new();
        let theme = Theme::from_name("Eclipse", None);

        let lrc_content = "[00:01.00] First test lyric line\n[00:05.00] Second test lyric line\n[00:10.00] ...\n[00:15.00] Final chorus line";
        player.lyrics = Some(crate::lyrics::lrc::Lyrics::parse(lrc_content, "test"));

        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let mut insp_ctx = InspectorContext {
                    player: &mut player,
                    stream_controller: None,
                    art_manager: &mut art_manager,
                    theme: &theme,
                    active_tab: InspectorTab::Lyrics,
                };
                let (tab, action) = render_right_inspector(ui, &mut insp_ctx);
                assert_eq!(tab, InspectorTab::Lyrics);
                assert_eq!(action, InspectorAction::None);
            });
        });
    }

    #[test]
    fn test_details_tab_rendering_local_and_stream() {
        let ctx = egui::Context::default();
        let cfg = AppConfig::default();
        let mut player = PlayerController::new(&cfg);
        let mut art_manager = AlbumArtManager::new();
        let theme = Theme::from_name("Eclipse", None);

        // 1. Local track loaded
        let local_track = sample_test_track(99, "Lossless Demo", "Audiophile Artist", "FLAC");
        player.load_queue(vec![local_track], 0);

        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let mut insp_ctx = InspectorContext {
                    player: &mut player,
                    stream_controller: None,
                    art_manager: &mut art_manager,
                    theme: &theme,
                    active_tab: InspectorTab::Details,
                };
                let (tab, _) = render_right_inspector(ui, &mut insp_ctx);
                assert_eq!(tab, InspectorTab::Details);
            });
        });

        // 2. Stream track loaded
        let stream_track = crate::streams::provider::StreamTrack::new(
            "stream-999",
            "Streaming Masterpiece",
            "Streamer Artist",
            "Stream Album",
            240.0,
            None,
            "spotify:track:stream-999",
            "spotify",
        );
        player.switch_to_stream("spotify", Some(stream_track));

        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let mut insp_ctx = InspectorContext {
                    player: &mut player,
                    stream_controller: None,
                    art_manager: &mut art_manager,
                    theme: &theme,
                    active_tab: InspectorTab::Details,
                };
                let (tab, _) = render_right_inspector(ui, &mut insp_ctx);
                assert_eq!(tab, InspectorTab::Details);
            });
        });
    }

    #[test]
    fn test_quality_badge_and_format_file_size() {
        assert_eq!(format_file_size(0), "-");
        assert_eq!(format_file_size(500), "500 B");
        assert_eq!(format_file_size(2048), "2.0 KB");
        assert_eq!(format_file_size(15 * 1024 * 1024), "15.0 MB");
        assert_eq!(format_file_size(2 * 1024 * 1024 * 1024), "2.00 GB");

        assert_eq!(truncate_str("Hello", 10), "Hello");
        assert_eq!(truncate_str("Long Track Name Exceeding", 12), "Long Trac...");
    }
}
