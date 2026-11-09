//! Picture-in-Picture (PiP) Floating Mini-Player & Desktop Overlay for Eclipse.
//!
//! Provides a glassmorphic floating mini-player card with high-contrast synced desktop lyrics.
//! 100% pure vector graphics - ZERO Unicode emojis.

use crate::audio::PlayerController;
use crate::ui::art::AlbumArtManager;
use crate::ui::icons::{icon_button, paint_icon, Icon};
use crate::ui::theme::Theme;
use egui::{pos2, vec2, Color32, Layout, Rect, RichText, Rounding, Stroke, Ui};
use std::time::Duration;

/// Truncate string to `max_chars` with an ellipsis if longer.
fn truncate_str(s: &str, max_chars: usize) -> String {
    if s.chars().count() > max_chars {
        let truncated: String = s.chars().take(max_chars.saturating_sub(1)).collect();
        format!("{truncated}…")
    } else {
        s.to_string()
    }
}

/// Render the high-contrast desktop synced lyrics overlay positioned above the mini-player card.
/// Uses background halo text shadows and a translucent frosted pill for maximum legibility
/// over any wallpaper or desktop environment.
pub fn render_desktop_lyrics(
    ui: &mut Ui,
    player: &PlayerController,
    theme: &Theme,
) -> egui::Response {
    let width = ui.available_width().max(320.0);
    let height = 44.0;
    let (rect, resp) = ui.allocate_exact_size(vec2(width, height), egui::Sense::click_and_drag());

    // Native window drag interaction on the lyrics overlay surface
    if resp.drag_started_by(egui::PointerButton::Primary) || resp.dragged() {
        ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
    }

    if ui.is_rect_visible(rect) {
        let painter = ui.painter();

        // Translucent frosted glass backdrop pill for wallpaper independence
        let pill_rect = Rect::from_center_size(rect.center(), vec2(rect.width(), height));
        painter.rect_filled(
            pill_rect,
            Rounding::same(12.0),
            Color32::from_rgba_premultiplied(12, 10, 20, 160),
        );
        painter.rect_stroke(
            pill_rect,
            Rounding::same(12.0),
            Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(255, 255, 255, 22)),
        );

        // Resolve active lyric line and next lyric line
        let pos_ms = player.engine.position().as_millis() as u64;
        let (active_text, next_text) = if let Some(ref lyrics) = player.lyrics {
            if lyrics.is_empty() {
                ("Eclipse · Lossless Audio".to_string(), None)
            } else if lyrics.source.contains("Instrumental")
                || (lyrics.lines.len() == 1 && lyrics.lines[0].text.eq_ignore_ascii_case("instrumental"))
            {
                ("Instrumental".to_string(), None)
            } else if lyrics.is_synced {
                if let Some(idx) = lyrics.current_index(pos_ms) {
                    let active = lyrics.lines[idx].text.trim().to_string();
                    let next = if idx + 1 < lyrics.lines.len() {
                        Some(lyrics.lines[idx + 1].text.trim().to_string())
                    } else {
                        None
                    };
                    (active, next)
                } else {
                    let next = lyrics.lines.first().map(|l| l.text.trim().to_string());
                    ("...".to_string(), next)
                }
            } else {
                // Plain lyrics fallback: show current or first line
                (
                    lyrics.lines[0].text.trim().to_string(),
                    lyrics.lines.get(1).map(|l| l.text.trim().to_string()),
                )
            }
        } else {
            ("Eclipse · Lossless Audio".to_string(), None)
        };

        let center_x = rect.center().x;
        let shadow_color = Color32::from_rgba_premultiplied(0, 0, 0, 230);

        if let Some(ref next_str) = next_text {
            // Two-line layout: active lyric line (16.0px bold) and next line (11.5px muted)
            let active_y = rect.min.y + 15.0;
            let next_y = rect.min.y + 31.0;

            let active_font = egui::FontId::proportional(16.0);
            let active_color = Color32::WHITE;

            let truncated_active = truncate_str(&active_text, 36);

            // Multi-direction text shadow halo for wallpaper contrast
            for (dx, dy) in &[(-1.0, 0.0), (1.0, 0.0), (0.0, -1.0), (0.0, 1.0), (1.0, 1.0)] {
                painter.text(
                    pos2(center_x + dx, active_y + dy),
                    egui::Align2::CENTER_CENTER,
                    &truncated_active,
                    active_font.clone(),
                    shadow_color,
                );
            }
            painter.text(
                pos2(center_x, active_y),
                egui::Align2::CENTER_CENTER,
                &truncated_active,
                active_font,
                active_color,
            );

            // Next lyric line
            let next_font = egui::FontId::proportional(11.5);
            let next_color = Color32::from_gray(180);
            let truncated_next = truncate_str(next_str, 40);

            painter.text(
                pos2(center_x + 1.0, next_y + 1.0),
                egui::Align2::CENTER_CENTER,
                &truncated_next,
                next_font.clone(),
                Color32::from_rgba_premultiplied(0, 0, 0, 180),
            );
            painter.text(
                pos2(center_x, next_y),
                egui::Align2::CENTER_CENTER,
                &truncated_next,
                next_font,
                next_color,
            );
        } else {
            // Single-line centered lyric or calm placeholder
            let active_y = rect.center().y;
            let active_font = egui::FontId::proportional(16.5);
            let active_color = if active_text == "Eclipse · Lossless Audio" {
                theme.text_secondary
            } else {
                Color32::WHITE
            };
            let truncated_active = truncate_str(&active_text, 36);

            for (dx, dy) in &[(-1.0, 0.0), (1.0, 0.0), (0.0, -1.0), (0.0, 1.0), (1.0, 1.0)] {
                painter.text(
                    pos2(center_x + dx, active_y + dy),
                    egui::Align2::CENTER_CENTER,
                    &truncated_active,
                    active_font.clone(),
                    shadow_color,
                );
            }
            painter.text(
                pos2(center_x, active_y),
                egui::Align2::CENTER_CENTER,
                &truncated_active,
                active_font,
                active_color,
            );
        }
    }

    resp
}

/// Render the glassmorphic mini-player card matching the reference design.
///
/// Styling:
/// - Background: `Color32::from_rgba_premultiplied(18, 16, 26, 230)`
/// - Border: `Stroke::new(1.0, Color32::from_rgba_premultiplied(255, 255, 255, 28))`
/// - Rounding: `Rounding::same(16.0)`
/// - Inner padding: `12.0`
///
/// Features:
/// - Smooth native drag interaction
/// - Close button `[×]` (`Icon::Close`, size 11.0)
/// - Left: 52px x 52px album cover art with `Rounding::same(8.0)`, fallback to vector `Icon::Disc`
/// - Right: Title (13.5px bold white) and Artist (11.0px muted)
/// - Transport controls: SkipBack (14.0px), solid circular Play/Pause (24px diameter), SkipForward (14.0px)
/// - Mini seekbar (compact seek line with knob and drag-to-seek)
/// - Heart button (`Icon::Heart` / `Icon::HeartOutline`, size 14.0px)
pub fn render_pip_card(
    ui: &mut Ui,
    player: &mut PlayerController,
    theme: &Theme,
    art_manager: &mut AlbumArtManager,
    show_pip: &mut bool,
    on_toggle_fav: &mut Option<i64>,
) -> egui::Response {
    let card_width = 360.0_f32;

    let frame = egui::Frame {
        inner_margin: egui::Margin::same(12.0),
        rounding: Rounding::same(16.0),
        fill: Color32::from_rgba_premultiplied(18, 16, 26, 230),
        stroke: Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(255, 255, 255, 28)),
        ..Default::default()
    };

    let current_track = player.current_track().cloned();

    let inner = frame.show(ui, |ui| {
        ui.set_width(card_width - 24.0);
        ui.horizontal(|ui| {
            // 1. LEFT: 52px x 52px album cover art with Rounding::same(8.0)
            let (art_rect, art_resp) = ui.allocate_exact_size(vec2(52.0, 52.0), egui::Sense::click_and_drag());
            if art_resp.drag_started_by(egui::PointerButton::Primary) || art_resp.dragged() {
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
            }

            let mut painted_cover = false;
            if let Some(ref track) = current_track {
                if let Some(texture) = art_manager.get_or_load(ui.ctx(), &track.path) {
                    ui.painter().image(
                        texture.id(),
                        art_rect,
                        Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                        Color32::WHITE,
                    );
                    ui.painter().rect_stroke(
                        art_rect,
                        Rounding::same(8.0),
                        Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(255, 255, 255, 30)),
                    );
                    painted_cover = true;
                }
            }

            if !painted_cover {
                ui.painter().rect_filled(
                    art_rect,
                    Rounding::same(8.0),
                    Color32::from_rgba_premultiplied(32, 28, 46, 220),
                );
                ui.painter().rect_stroke(
                    art_rect,
                    Rounding::same(8.0),
                    Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(255, 255, 255, 20)),
                );
                let disc_box = Rect::from_center_size(art_rect.center(), vec2(26.0, 26.0));
                paint_icon(ui.painter(), Icon::Disc, disc_box, theme.accent);
            }

            ui.add_space(8.0);

            // 2. RIGHT: Title/Artist row & Controls row
            let right_width = (card_width - 24.0 - 52.0 - 8.0).max(180.0);
            ui.allocate_ui_with_layout(
                vec2(right_width, 52.0),
                Layout::top_down(egui::Align::Min),
                |ui| {
                    ui.spacing_mut().item_spacing = vec2(0.0, 4.0);

                    // Row A: Title & Artist + Close Button
                    ui.horizontal(|ui| {
                        let text_w = (right_width - 24.0).max(120.0);
                        let title_str = current_track
                            .as_ref()
                            .map(|t| t.display_title())
                            .unwrap_or("No playback");
                        let artist_str = current_track
                            .as_ref()
                            .map(|t| t.display_artist())
                            .unwrap_or("Eclipse");

                        ui.allocate_ui_with_layout(
                            vec2(text_w, 22.0),
                            Layout::left_to_right(egui::Align::Center),
                            |ui| {
                                ui.spacing_mut().item_spacing = vec2(6.0, 0.0);
                                let trunc_title = truncate_str(title_str, 16);
                                ui.label(
                                    RichText::new(trunc_title)
                                        .color(Color32::WHITE)
                                        .size(13.5)
                                        .strong(),
                                );
                                let trunc_artist = truncate_str(artist_str, 14);
                                ui.label(
                                    RichText::new(format!("• {trunc_artist}"))
                                        .color(theme.text_secondary)
                                        .size(11.0),
                                );
                            },
                        );

                        // Close button [×] (Icon::Close, size 11.0) in top-right corner
                        if icon_button(ui, Icon::Close, 11.0, theme.text_muted)
                            .on_hover_text("Close Mini Player")
                            .clicked()
                        {
                            *show_pip = false;
                        }
                    });

                    // Row B: Controls row
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing = vec2(7.0, 0.0);

                        // SkipBack (14.0px)
                        if icon_button(ui, Icon::SkipBack, 14.0, theme.text_primary)
                            .on_hover_text("Previous Track")
                            .clicked()
                        {
                            player.prev();
                        }

                        // Play/Pause circular solid button: 24px diameter circle filled with theme.accent
                        let (play_rect, play_resp) = ui.allocate_exact_size(vec2(24.0, 24.0), egui::Sense::click());
                        if play_resp.hovered() {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                        }
                        let play_center = play_rect.center();
                        let play_fill = if play_resp.hovered() {
                            Color32::from_rgba_premultiplied(
                                (theme.accent.r() as u16 + 25).min(255) as u8,
                                (theme.accent.g() as u16 + 25).min(255) as u8,
                                (theme.accent.b() as u16 + 25).min(255) as u8,
                                255,
                            )
                        } else {
                            theme.accent
                        };
                        ui.painter().circle_filled(play_center, 12.0, play_fill);

                        let is_playing = player.engine.is_playing();
                        let play_icon = if is_playing { Icon::Pause } else { Icon::Play };
                        let icon_box = Rect::from_center_size(play_center, vec2(12.0, 12.0));
                        paint_icon(ui.painter(), play_icon, icon_box, Color32::from_rgb(18, 16, 26));

                        if play_resp.clicked() {
                            if player.current_track().is_some() {
                                player.engine.toggle_play_pause();
                            } else if !player.queue.is_empty() {
                                player.play_index(0);
                            }
                        }

                        // SkipForward (14.0px)
                        if icon_button(ui, Icon::SkipForward, 14.0, theme.text_primary)
                            .on_hover_text("Next Track")
                            .clicked()
                        {
                            player.next();
                        }

                        // Mini Seekbar: compact seek line (80-120px wide, height 3.5px, filled rail with knob)
                        let seekbar_w = 90.0_f32;
                        render_mini_seekbar(ui, player, theme, seekbar_w);

                        // Heart button (Icon::Heart / Icon::HeartOutline, size 14.0px, toggles favorite)
                        let is_fav = current_track.as_ref().map(|t| t.favorite).unwrap_or(false);
                        let heart_icon = if is_fav { Icon::Heart } else { Icon::HeartOutline };
                        let heart_color = if is_fav { theme.accent } else { theme.text_muted };
                        if icon_button(ui, heart_icon, 14.0, heart_color)
                            .on_hover_text(if is_fav { "Favorited" } else { "Favorite" })
                            .clicked()
                        {
                            if let Some(ref track) = current_track {
                                *on_toggle_fav = Some(track.id);
                            }
                        }
                    });
                },
            );
        });
    });

    // Native window drag interaction on empty surface of the card
    let card_resp = ui.interact(
        inner.response.rect,
        ui.id().with("pip_card_surface_drag"),
        egui::Sense::click_and_drag(),
    );
    if card_resp.drag_started_by(egui::PointerButton::Primary) || card_resp.dragged() {
        ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
    }

    inner.response
}

/// Helper to render the compact seek line with knob and click/drag-to-seek functionality.
fn render_mini_seekbar(
    ui: &mut Ui,
    player: &mut PlayerController,
    theme: &Theme,
    desired_width: f32,
) {
    let (rect, resp) = ui.allocate_exact_size(vec2(desired_width, 16.0), egui::Sense::click_and_drag());
    if resp.hovered() || resp.dragged() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }

    let pos_secs = player.engine.position().as_secs_f64();
    let dur_secs = player
        .current_track()
        .map(|t| t.duration_secs)
        .unwrap_or(0.0)
        .max(0.001);

    let rail_h = 3.5;
    let rail_rect = Rect::from_center_size(rect.center(), vec2(rect.width(), rail_h));
    let rail_rounding = Rounding::same(rail_h / 2.0);

    // Track background rail
    ui.painter().rect_filled(
        rail_rect,
        rail_rounding,
        Color32::from_rgba_premultiplied(255, 255, 255, 38),
    );

    let mut current_pos = pos_secs;
    if resp.clicked() || resp.dragged() {
        if let Some(pointer_pos) = resp.interact_pointer_pos() {
            let frac = ((pointer_pos.x - rail_rect.min.x) / rail_rect.width()).clamp(0.0, 1.0);
            let seek_target = frac as f64 * dur_secs;
            current_pos = seek_target;
            player.engine.seek(Duration::from_secs_f64(seek_target));
        }
    }

    // Filled progress rail
    let progress = (current_pos / dur_secs).clamp(0.0, 1.0) as f32;
    let fill_w = rail_rect.width() * progress;
    if fill_w > 0.0 {
        let filled_rect = Rect::from_min_size(rail_rect.min, vec2(fill_w, rail_h));
        ui.painter().rect_filled(filled_rect, rail_rounding, theme.accent);
    }

    // Interactive scrubber knob
    let knob_pos = pos2(rail_rect.min.x + fill_w, rail_rect.center().y);
    let knob_r = if resp.hovered() || resp.dragged() { 4.5 } else { 3.2 };
    ui.painter().circle_filled(knob_pos, knob_r, Color32::WHITE);
}

/// Render the combined overlay content: desktop synced lyrics placed above the mini-player card.
pub fn render_pip_overlay_content(
    ui: &mut Ui,
    player: &mut PlayerController,
    theme: &Theme,
    art_manager: &mut AlbumArtManager,
    show_pip: &mut bool,
    on_toggle_fav: &mut Option<i64>,
    show_lyrics: bool,
) {
    ui.vertical_centered(|ui| {
        if show_lyrics {
            render_desktop_lyrics(ui, player, theme);
            ui.add_space(8.0);
        }
        render_pip_card(ui, player, theme, art_manager, show_pip, on_toggle_fav);
    });
}

/// Display the floating native Picture-in-Picture viewport window.
pub fn show_pip_viewport(
    ctx: &egui::Context,
    player: &mut PlayerController,
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

    let target_height = if pip_settings.show_desktop_lyrics { 136.0 } else { 82.0 };
    let mut builder = egui::ViewportBuilder::default()
        .with_title("Eclipse Mini Player")
        .with_inner_size(vec2(364.0, target_height))
        .with_transparent(true)
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
                (screen.max.x - 384.0).max(20.0),
                (screen.max.y - target_height - 70.0).max(20.0),
            ));
        }
    }

    ctx.show_viewport_immediate(
        viewport_id,
        builder,
        |ctx, _class| {
            if ctx.input(|i| i.viewport().close_requested()) {
                *show_pip = false;
            }

            egui::CentralPanel::default()
                .frame(egui::Frame::none().fill(Color32::TRANSPARENT))
                .show(ctx, |ui| {
                    render_pip_overlay_content(
                        ui,
                        player,
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

                // 2. With synced lyrics
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

                // Render full content with lyrics
                render_pip_overlay_content(
                    ui,
                    &mut player,
                    &theme,
                    &mut art_manager,
                    &mut show_pip,
                    &mut on_toggle_fav,
                    true,
                );

                // Render full content without lyrics
                render_pip_overlay_content(
                    ui,
                    &mut player,
                    &theme,
                    &mut art_manager,
                    &mut show_pip,
                    &mut on_toggle_fav,
                    false,
                );
            });
        });
    }
}
