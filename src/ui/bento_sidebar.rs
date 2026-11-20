//! Bento Sidebar for Eclipse.
//! Provides a responsive, collapsible sidebar with Bento-grid aesthetic:
//! - 64px width when collapsed (icon-only mode with tooltips)
//! - 220px width when expanded (full labels, counts, and category headers)
//! - Crisp accent purple pill badges and vertical indicator bar
//! - Zero panics, 100% vector icons, zero Unicode emojis.

use crate::library::model::Playlist;
use crate::ui::icons::{paint_icon, Icon};
use crate::ui::sidebar::NavTab;
use crate::ui::theme::Theme;
use egui::{pos2, vec2, Color32, FontId, Layout, Rect, RichText, Rounding, ScrollArea, Stroke, Ui};

/// Feishin Matte-Slate Design Tokens
const BG_SIDEBAR: Color32 = Color32::from_rgb(22, 26, 32);
const BG_CARD: Color32 = Color32::from_rgb(37, 42, 52);
const BG_CARD_HOVER: Color32 = Color32::from_rgb(47, 53, 66);
const SEAM_DARK: Color32 = Color32::from_rgb(38, 43, 53);
const ACCENT_PRIMARY: Color32 = Color32::from_rgb(156, 108, 255);
const ACCENT_SOFT: Color32 = Color32::from_rgba_premultiplied(59, 39, 102, 34);
const TEXT_PRIMARY: Color32 = Color32::from_rgb(240, 242, 246);
const TEXT_MUTED: Color32 = Color32::from_rgb(118, 124, 138);
const TEXT_SECONDARY: Color32 = Color32::from_rgb(168, 174, 188);
const TEXT_HOVER: Color32 = Color32::from_rgb(240, 242, 246);
const DIVIDER_COLOR: Color32 = Color32::from_rgb(38, 43, 53);

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BentoSidebarAction {
    pub new_tab: Option<crate::ui::sidebar::NavTab>,
    pub toggle_collapse: bool,
    pub create_playlist: bool,
}

#[allow(clippy::too_many_arguments)]
pub fn render_bento_sidebar(
    ui: &mut Ui,
    current_tab: NavTab,
    is_collapsed: bool,
    user_playlists: &[Playlist],
    theme: &Theme,
    total_tracks: usize,
    total_playlists: usize,
    show_spotify: bool,
    show_youtube: bool,
) -> BentoSidebarAction {
    let width = if is_collapsed { 64.0 } else { 220.0 };
    ui.set_min_width(width);
    ui.set_max_width(width);

    let mut action = BentoSidebarAction::default();

    let sidebar_frame = egui::Frame::none()
        .fill(BG_SIDEBAR)
        .rounding(Rounding::ZERO)
        .stroke(Stroke::NONE)
        .inner_margin(egui::Margin {
            left: 0.0,
            right: 0.0,
            top: 0.0,
            bottom: 0.0,
        });

    let frame_resp = sidebar_frame.show(ui, |ui| {
        ui.vertical(|ui| {
            // Top Header Row
            render_sidebar_header(ui, is_collapsed, theme, &mut action);

            ui.add_space(4.0);

            // Scrollable content area
            ScrollArea::vertical()
                .id_salt("bento_sidebar_scroll")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing = vec2(0.0, 3.0);

                    // Section 1: LIBRARY
                    if !is_collapsed {
                        section_header(ui, "LIBRARY", Some(total_tracks));
                    } else {
                        collapsed_divider(ui);
                    }

                    render_nav_item(ui, current_tab, NavTab::Home, Icon::Home, "Home", is_collapsed, None, &mut action);
                    render_nav_item(ui, current_tab, NavTab::Songs, Icon::Music, "Songs", is_collapsed, None, &mut action);
                    render_nav_item(ui, current_tab, NavTab::Artists, Icon::Mic, "Artists", is_collapsed, None, &mut action);
                    render_nav_item(ui, current_tab, NavTab::Albums, Icon::Disc, "Albums", is_collapsed, None, &mut action);
                    render_nav_item(ui, current_tab, NavTab::Genres, Icon::Tag, "Genres", is_collapsed, None, &mut action);
                    render_nav_item(ui, current_tab, NavTab::Folders, Icon::Folder, "Folders", is_collapsed, None, &mut action);

                    ui.add_space(6.0);

                    // Section 2: SMART PLAYLISTS
                    if !is_collapsed {
                        section_header(ui, "SMART", None);
                    } else {
                        collapsed_divider(ui);
                    }

                    render_nav_item(ui, current_tab, NavTab::RecentlyPlayed, Icon::Clock, "Recently Played", is_collapsed, None, &mut action);
                    render_nav_item(ui, current_tab, NavTab::MostPlayed, Icon::Fire, "Most Played", is_collapsed, None, &mut action);
                    render_nav_item(ui, current_tab, NavTab::RecentlyAdded, Icon::Sparkle, "Recently Added", is_collapsed, None, &mut action);
                    render_nav_item(ui, current_tab, NavTab::Favorites, Icon::Heart, "Favorites", is_collapsed, None, &mut action);
                    render_nav_item(ui, current_tab, NavTab::NeverPlayed, Icon::Hourglass, "Never Played", is_collapsed, None, &mut action);
                    render_nav_item(ui, current_tab, NavTab::RandomDiscovery, Icon::Dices, "Random Discovery", is_collapsed, None, &mut action);

                    // Section 3: STREAM
                    if show_spotify || show_youtube {
                        ui.add_space(6.0);
                        if !is_collapsed {
                            section_header(ui, "STREAM", None);
                        } else {
                            collapsed_divider(ui);
                        }

                        if show_spotify {
                            render_nav_item(ui, current_tab, NavTab::Spotify, Icon::Spotify, "Spotify", is_collapsed, None, &mut action);
                        }
                        if show_youtube {
                            render_nav_item(ui, current_tab, NavTab::YouTubeMusic, Icon::YouTube, "YouTube Music", is_collapsed, None, &mut action);
                        }
                    }

                    ui.add_space(6.0);

                    // Section 4: PLAYLISTS
                    if !is_collapsed {
                        render_playlists_header(ui, total_playlists, &mut action);
                    } else {
                        collapsed_divider(ui);
                        render_collapsed_create_playlist(ui, &mut action);
                    }

                    render_nav_item(ui, current_tab, NavTab::Playlists, Icon::Playlist, "All Playlists", is_collapsed, None, &mut action);

                    for pl in user_playlists {
                        let target = NavTab::Playlist(pl.id);
                        render_nav_item(
                            ui,
                            current_tab,
                            target,
                            Icon::Playlist,
                            &pl.name,
                            is_collapsed,
                            Some(pl.track_count),
                            &mut action,
                        );
                    }

                    ui.add_space(6.0);

                    // Section 5: PREFERENCES & INFO
                    if !is_collapsed {
                        section_header(ui, "PREFERENCES", None);
                    } else {
                        collapsed_divider(ui);
                    }

                    render_nav_item(ui, current_tab, NavTab::Settings, Icon::Settings, "Settings", is_collapsed, None, &mut action);
                    render_nav_item(ui, current_tab, NavTab::About, Icon::Info, "About", is_collapsed, None, &mut action);

                    // Expanded stats card at bottom
                    if !is_collapsed {
                        ui.add_space(10.0);
                        render_stats_card(ui, theme, total_tracks, total_playlists);
                        ui.add_space(8.0);
                    }
                });
        });
    });

    // Single 1.0px SEAM_DARK right border seam
    let r = frame_resp.response.rect;
    let right_seam = Rect::from_min_max(
        pos2(r.max.x - 1.0, r.min.y),
        pos2(r.max.x, r.max.y),
    );
    ui.painter().rect_filled(right_seam, Rounding::ZERO, SEAM_DARK);

    action
}

/// Top header containing Eclipse wordmark and collapse/expand toggle button.
fn render_sidebar_header(
    ui: &mut Ui,
    is_collapsed: bool,
    _theme: &Theme,
    action: &mut BentoSidebarAction,
) {
    let header_h = 38.0;
    let full_w = ui.available_width();
    let (header_rect, _) = ui.allocate_exact_size(vec2(full_w, header_h), egui::Sense::hover());

    let mut header_ui = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(header_rect)
            .layout(Layout::left_to_right(egui::Align::Center)),
    );
    header_ui.set_width(full_w);

    if is_collapsed {
        // Centered expand button
        let btn_size = vec2(34.0, 34.0);
        let center_x = header_rect.center().x;
        let btn_rect = Rect::from_center_size(pos2(center_x, header_rect.center().y), btn_size);
        let resp = header_ui.interact(btn_rect, header_ui.id().with("bento_sidebar_toggle"), egui::Sense::click());

        let is_hovered = resp.hovered();
        if is_hovered {
            header_ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            header_ui.painter().rect_filled(btn_rect, Rounding::same(8.0), BG_CARD_HOVER);
        }

        let icon_color = if is_hovered { TEXT_HOVER } else { TEXT_SECONDARY };
        let icon_rect = Rect::from_center_size(btn_rect.center(), vec2(16.0, 16.0));
        paint_icon(header_ui.painter(), Icon::ChevronRight, icon_rect, icon_color);

        let resp = resp.on_hover_text("Expand Sidebar");
        if resp.clicked() {
            action.toggle_collapse = true;
        }
    } else {
        // Left: Minimalist wordmark
        header_ui.add_space(10.0);
        header_ui.label(
            RichText::new("ECLIPSE")
                .size(13.0)
                .strong()
                .color(TEXT_PRIMARY),
        );

        // Right: Collapse toggle icon
        header_ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(6.0);
            let (btn_rect, resp) = ui.allocate_exact_size(vec2(28.0, 28.0), egui::Sense::click());
            let is_hovered = resp.hovered();

            if is_hovered {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                ui.painter().rect_filled(btn_rect, Rounding::same(6.0), BG_CARD_HOVER);
            }

            let icon_color = if is_hovered { TEXT_HOVER } else { TEXT_MUTED };
            let icon_rect = Rect::from_center_size(btn_rect.center(), vec2(15.0, 15.0));
            paint_icon(ui.painter(), Icon::ChevronLeft, icon_rect, icon_color);

            let resp = resp.on_hover_text("Collapse Sidebar");
            if resp.clicked() {
                action.toggle_collapse = true;
            }
        });
    }
}

/// Upper-case muted section title with optional right-aligned count.
fn section_header(ui: &mut Ui, label: &str, count: Option<usize>) {
    ui.add_space(4.0);
    let width = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(vec2(width, 16.0), egui::Sense::hover());
    let font = FontId::proportional(10.5);

    ui.painter().text(
        pos2(rect.min.x + 8.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        label,
        font.clone(),
        TEXT_MUTED,
    );

    if let Some(c) = count {
        ui.painter().text(
            pos2(rect.max.x - 8.0, rect.center().y),
            egui::Align2::RIGHT_CENTER,
            c.to_string(),
            font,
            TEXT_MUTED,
        );
    }
    ui.add_space(2.0);
}

/// Subtle horizontal separator in collapsed rail.
fn collapsed_divider(ui: &mut Ui) {
    ui.add_space(3.0);
    let width = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(vec2(width, 1.0), egui::Sense::hover());
    let line_rect = Rect::from_center_size(rect.center(), vec2(28.0, 1.0));
    ui.painter().rect_filled(line_rect, Rounding::ZERO, DIVIDER_COLOR);
    ui.add_space(3.0);
}

/// Single navigation item handling both expanded and collapsed modes.
#[allow(clippy::too_many_arguments)]
fn render_nav_item(
    ui: &mut Ui,
    current_tab: NavTab,
    target: NavTab,
    icon: Icon,
    label: &str,
    is_collapsed: bool,
    count: Option<usize>,
    action: &mut BentoSidebarAction,
) {
    let is_selected = current_tab == target;

    if is_collapsed {
        render_nav_item_collapsed(ui, is_selected, target, icon, label, count, action);
    } else {
        render_nav_item_expanded(ui, is_selected, target, icon, label, count, action);
    }
}

/// Expanded navigation row with purple pill, indicator bar, vector icon, label, and count badge.
fn render_nav_item_expanded(
    ui: &mut Ui,
    is_selected: bool,
    target: NavTab,
    icon: Icon,
    label: &str,
    count: Option<usize>,
    action: &mut BentoSidebarAction,
) {
    let width = ui.available_width();
    let height = 34.0;
    let (rect, resp) = ui.allocate_exact_size(vec2(width, height), egui::Sense::click());
    let is_hovered = resp.hovered();

    if is_hovered {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }

    // Active / Hover styling
    let (bg_fill, text_color, icon_color) = if is_selected {
        (ACCENT_SOFT, TEXT_PRIMARY, ACCENT_PRIMARY)
    } else if is_hovered {
        (BG_CARD_HOVER, TEXT_HOVER, TEXT_HOVER)
    } else {
        (Color32::TRANSPARENT, TEXT_SECONDARY, TEXT_SECONDARY)
    };

    ui.painter().rect_filled(rect, Rounding::same(8.0), bg_fill);

    // Left vertical thin 3.0px ACCENT_PRIMARY indicator bar when active
    if is_selected {
        let bar_rect = Rect::from_min_size(
            pos2(rect.min.x + 2.0, rect.center().y - 8.0),
            vec2(3.0, 16.0),
        );
        ui.painter().rect_filled(bar_rect, Rounding::same(1.5), ACCENT_PRIMARY);
    }

    // Icon (16x16)
    let icon_rect = Rect::from_center_size(
        pos2(rect.min.x + 18.0, rect.center().y),
        vec2(16.0, 16.0),
    );
    paint_icon(ui.painter(), icon, icon_rect, icon_color);

    // Optional count text on right
    let count_w = if let Some(c) = count {
        let c_str = c.to_string();
        let c_font = FontId::proportional(10.5);
        let w = ui.painter().layout_no_wrap(c_str.clone(), c_font.clone(), TEXT_MUTED).rect.width();
        ui.painter().text(
            pos2(rect.max.x - 10.0, rect.center().y),
            egui::Align2::RIGHT_CENTER,
            c_str,
            c_font,
            TEXT_MUTED,
        );
        w + 14.0
    } else {
        0.0
    };

    // Label text with overflow truncation
    let text_left = rect.min.x + 36.0;
    let max_text_w = (rect.max.x - count_w - text_left - 4.0).max(20.0);
    let label_font = FontId::proportional(12.5);

    let display_label = truncate_label(ui, label, label_font.clone(), text_color, max_text_w);
    let label_rect = Rect::from_min_max(
        pos2(text_left, rect.min.y),
        pos2(rect.max.x - count_w - 4.0, rect.max.y),
    );
    let mut label_ui = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(label_rect)
            .layout(Layout::left_to_right(egui::Align::Center)),
    );
    if is_selected {
        label_ui.label(
            RichText::new(display_label)
                .size(12.5)
                .strong()
                .color(TEXT_PRIMARY),
        );
    } else {
        label_ui.label(
            RichText::new(display_label)
                .size(12.5)
                .color(text_color),
        );
    }

    if resp.clicked() {
        action.new_tab = Some(target);
    }
}

/// Collapsed rail navigation item with centered 16px icon and tooltip.
fn render_nav_item_collapsed(
    ui: &mut Ui,
    is_selected: bool,
    target: NavTab,
    icon: Icon,
    label: &str,
    count: Option<usize>,
    action: &mut BentoSidebarAction,
) {
    let width = ui.available_width();
    let height = 36.0;
    let (rect, resp) = ui.allocate_exact_size(vec2(width, height), egui::Sense::click());
    let is_hovered = resp.hovered();

    if is_hovered {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }

    let pill_rect = rect.shrink2(vec2(8.0, 2.0));

    let (bg_fill, icon_color) = if is_selected {
        (ACCENT_SOFT, ACCENT_PRIMARY)
    } else if is_hovered {
        (BG_CARD_HOVER, TEXT_HOVER)
    } else {
        (Color32::TRANSPARENT, TEXT_SECONDARY)
    };

    ui.painter().rect_filled(pill_rect, Rounding::same(8.0), bg_fill);

    // Left indicator bar when selected
    if is_selected {
        let bar_rect = Rect::from_min_size(
            pos2(pill_rect.min.x + 2.0, pill_rect.center().y - 8.0),
            vec2(3.0, 16.0),
        );
        ui.painter().rect_filled(bar_rect, Rounding::same(1.5), ACCENT_PRIMARY);
    }

    // Centered 16px icon
    let icon_rect = Rect::from_center_size(pill_rect.center(), vec2(16.0, 16.0));
    paint_icon(ui.painter(), icon, icon_rect, icon_color);

    // Tooltip showing label and count if any
    let tooltip_text = if let Some(c) = count {
        format!("{} ({} tracks)", label, c)
    } else {
        label.to_string()
    };
    let resp = resp.on_hover_text(tooltip_text);

    if resp.clicked() {
        action.new_tab = Some(target);
    }
}

/// Header row for playlists with label, count, and '+' create button.
fn render_playlists_header(
    ui: &mut Ui,
    total_playlists: usize,
    action: &mut BentoSidebarAction,
) {
    ui.horizontal(|ui| {
        ui.add_space(8.0);
        ui.label(
            RichText::new("PLAYLISTS")
                .size(10.5)
                .color(TEXT_MUTED)
                .strong(),
        );

        ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(4.0);
            let (plus_rect, plus_resp) = ui.allocate_exact_size(vec2(20.0, 20.0), egui::Sense::click());
            let is_hovered = plus_resp.hovered();

            let plus_bg = if is_hovered { BG_CARD_HOVER } else { Color32::TRANSPARENT };
            ui.painter().rect_filled(plus_rect, Rounding::same(4.0), plus_bg);

            let icon_color = if is_hovered { TEXT_PRIMARY } else { TEXT_MUTED };
            let icon_rect = Rect::from_center_size(plus_rect.center(), vec2(11.0, 11.0));
            paint_icon(ui.painter(), Icon::Plus, icon_rect, icon_color);

            if is_hovered {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }
            let plus_resp = plus_resp.on_hover_text("Create Playlist");
            if plus_resp.clicked() {
                action.create_playlist = true;
            }

            ui.add_space(4.0);
            ui.label(
                RichText::new(total_playlists.to_string())
                    .size(10.5)
                    .color(TEXT_MUTED),
            );
        });
    });
}

/// Centered '+' create playlist icon button in collapsed rail.
fn render_collapsed_create_playlist(ui: &mut Ui, action: &mut BentoSidebarAction) {
    let width = ui.available_width();
    let (rect, resp) = ui.allocate_exact_size(vec2(width, 28.0), egui::Sense::click());
    let is_hovered = resp.hovered();

    let pill_rect = Rect::from_center_size(rect.center(), vec2(28.0, 24.0));
    let bg = if is_hovered { BG_CARD_HOVER } else { Color32::TRANSPARENT };
    ui.painter().rect_filled(pill_rect, Rounding::same(6.0), bg);

    let icon_color = if is_hovered { TEXT_PRIMARY } else { TEXT_MUTED };
    let icon_rect = Rect::from_center_size(pill_rect.center(), vec2(12.0, 12.0));
    paint_icon(ui.painter(), Icon::Plus, icon_rect, icon_color);

    if is_hovered {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    let resp = resp.on_hover_text("Create Playlist");
    if resp.clicked() {
        action.create_playlist = true;
    }
}

/// Library overview stats card at the bottom of the expanded sidebar.
fn render_stats_card(
    ui: &mut Ui,
    _theme: &Theme,
    total_tracks: usize,
    total_playlists: usize,
) {
    let mut stats_frame = egui::Frame::none();
    stats_frame.fill = BG_CARD;
    stats_frame.rounding = Rounding::same(8.0);
    stats_frame.stroke = Stroke::NONE;
    stats_frame.inner_margin = egui::Margin::same(8.0);
    stats_frame.show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = vec2(10.0, 0.0);
            let (icon_rect, _) = ui.allocate_exact_size(vec2(20.0, 20.0), egui::Sense::hover());
            paint_icon(ui.painter(), Icon::Library, icon_rect, TEXT_MUTED);

            ui.vertical(|ui| {
                ui.label(
                    RichText::new(format!("{} Tracks", total_tracks))
                        .size(11.5)
                        .color(TEXT_PRIMARY)
                        .strong(),
                );
                ui.label(
                    RichText::new(format!("{} Playlists", total_playlists))
                        .size(10.0)
                        .color(TEXT_MUTED),
                );
            });
        });
    });
}

/// Truncate label safely with trailing ellipsis if it exceeds maximum width.
fn truncate_label(
    ui: &Ui,
    text: &str,
    font: FontId,
    color: Color32,
    max_w: f32,
) -> String {
    let clean = if text.trim().is_empty() { "Untitled" } else { text };
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
    fn test_bento_sidebar_action_default() {
        let action = BentoSidebarAction::default();
        assert_eq!(action.new_tab, None);
        assert!(!action.toggle_collapse);
        assert!(!action.create_playlist);
    }

    #[test]
    fn test_bento_sidebar_action_variants() {
        let action = BentoSidebarAction {
            new_tab: Some(NavTab::Albums),
            toggle_collapse: true,
            create_playlist: false,
        };
        assert_eq!(action.new_tab, Some(NavTab::Albums));
        assert!(action.toggle_collapse);
        assert!(!action.create_playlist);
    }

    #[test]
    fn test_bento_sidebar_render_expanded_smoke() {
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let theme = Theme::from_name("Obsidian", None);
                let playlists = vec![
                    Playlist {
                        id: 1,
                        name: "Synthwave Chill".to_string(),
                        created: 0.0,
                        description: String::new(),
                        track_count: 42,
                    },
                ];

                let action = render_bento_sidebar(
                    ui,
                    NavTab::Home,
                    false, // expanded
                    &playlists,
                    &theme,
                    320,
                    1,
                    true,
                    true,
                );

                assert_eq!(action, BentoSidebarAction::default());
            });
        });
    }

    #[test]
    fn test_bento_sidebar_render_collapsed_smoke() {
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let theme = Theme::from_name("Obsidian", None);
                let playlists = vec![];

                let action = render_bento_sidebar(
                    ui,
                    NavTab::Songs,
                    true, // collapsed
                    &playlists,
                    &theme,
                    150,
                    0,
                    false,
                    false,
                );

                assert_eq!(action, BentoSidebarAction::default());
            });
        });
    }

    #[test]
    fn test_truncate_label_logic() {
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let font = FontId::proportional(12.0);
                let short = truncate_label(ui, "Test", font.clone(), Color32::WHITE, 200.0);
                assert_eq!(short, "Test");

                let long_text = "A very long playlist title that should definitely be truncated";
                let truncated = truncate_label(ui, long_text, font, Color32::WHITE, 40.0);
                assert!(truncated.ends_with('…'));
            });
        });
    }
}
