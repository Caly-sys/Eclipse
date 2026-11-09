use crate::library::model::Playlist;
use crate::ui::icons::{paint_icon, Icon};
use crate::ui::theme::Theme;
use egui::{pos2, vec2, Color32, Layout, Rect, RichText, Rounding, ScrollArea, Stroke, Ui};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavTab {
    // Library
    Home,
    NowPlaying,
    Songs,
    Artists,
    Albums,
    Genres,
    Folders,
    // Smart Playlists
    RecentlyPlayed,
    MostPlayed,
    RecentlyAdded,
    Favorites,
    NeverPlayed,
    RandomDiscovery,
    // Playlists
    Playlists,
    Playlist(i64),
    // Preferences & Info
    Settings,
    About,
}

#[allow(clippy::too_many_arguments)]
pub fn render_sidebar(
    ui: &mut Ui,
    current_tab: &mut NavTab,
    user_playlists: &[Playlist],
    theme: &Theme,
    total_tracks: usize,
    total_playlists: usize,
    on_create_playlist: &mut bool,
    show_spotify: bool,
    show_youtube: bool,
) {
    // Left sidebar fixed width 240px
    ui.set_min_width(240.0);
    ui.set_max_width(240.0);

    ui.vertical(|ui| {
        // Independent vertical scroll area with padding (12.0, 10.0) so top items never clip
        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing = vec2(0.0, 3.0);

                // Top padding so top items like Home never clip into top border
                ui.add_space(6.0);

                // Section 1: LIBRARY
                section_header(ui, "LIBRARY", Some(total_tracks), theme);
                nav_button(ui, current_tab, NavTab::Home, Icon::Home, "Home", theme);
                nav_button(ui, current_tab, NavTab::NowPlaying, Icon::Disc, "Now Playing", theme);
                nav_button(ui, current_tab, NavTab::Songs, Icon::Music, "Songs", theme);
                nav_button(ui, current_tab, NavTab::Artists, Icon::Mic, "Artists", theme);
                nav_button(ui, current_tab, NavTab::Albums, Icon::Disc, "Albums", theme);
                nav_button(ui, current_tab, NavTab::Genres, Icon::Tag, "Genres", theme);
                nav_button(ui, current_tab, NavTab::Folders, Icon::Folder, "Folders", theme);

                ui.add_space(8.0);

                // Section 2: SMART PLAYLISTS
                section_header(ui, "SMART", Some(6), theme);
                nav_button(ui, current_tab, NavTab::RecentlyPlayed, Icon::Clock, "Recently Played", theme);
                nav_button(ui, current_tab, NavTab::MostPlayed, Icon::Fire, "Most Played", theme);
                nav_button(ui, current_tab, NavTab::RecentlyAdded, Icon::Sparkle, "Recently Added", theme);
                nav_button(ui, current_tab, NavTab::Favorites, Icon::Heart, "Favorites", theme);
                nav_button(ui, current_tab, NavTab::NeverPlayed, Icon::Hourglass, "Never Played", theme);
                nav_button(ui, current_tab, NavTab::RandomDiscovery, Icon::Dices, "Random Discovery", theme);

                ui.add_space(8.0);

                // Section 3: STREAM
                if show_spotify || show_youtube {
                    section_header(ui, "STREAM", None, theme);
                    if show_spotify {
                        disabled_stream_item(ui, Icon::Spotify, "Spotify", theme);
                    }
                    if show_youtube {
                        disabled_stream_item(ui, Icon::YouTube, "YouTube Music", theme);
                    }
                    ui.add_space(8.0);
                }

                // Section 4: PLAYLISTS with count & '+' button
                ui.horizontal(|ui| {
                    ui.add_space(8.0);
                    ui.label(
                        RichText::new("PLAYLISTS")
                            .size(10.5)
                            .color(theme.text_muted)
                            .strong(),
                    );
                    ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                        let (plus_rect, plus_resp) = ui.allocate_exact_size(vec2(20.0, 20.0), egui::Sense::click());
                        let plus_bg = if plus_resp.hovered() {
                            theme.surface_hover
                        } else {
                            Color32::TRANSPARENT
                        };
                        let plus_stroke = if plus_resp.hovered() {
                            Stroke::new(1.0_f32, theme.border)
                        } else {
                            Stroke::NONE
                        };
                        ui.painter().rect_filled(plus_rect, Rounding::same(4.0), plus_bg);
                        if plus_stroke != Stroke::NONE {
                            ui.painter().rect_stroke(plus_rect, Rounding::same(4.0), plus_stroke);
                        }
                        let icon_rect = Rect::from_center_size(plus_rect.center(), vec2(11.0, 11.0));
                        let icon_color = if plus_resp.hovered() {
                            theme.text_primary
                        } else {
                            theme.text_muted
                        };
                        paint_icon(ui.painter(), Icon::Plus, icon_rect, icon_color);

                        if plus_resp.hovered() {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                        }
                        if plus_resp.clicked() {
                            *on_create_playlist = true;
                        }

                        ui.add_space(4.0);
                        ui.label(
                            RichText::new(format!("{}", total_playlists))
                                .size(10.5)
                                .color(theme.text_muted),
                        );
                    });
                });

                nav_button(ui, current_tab, NavTab::Playlists, Icon::Playlist, "All Playlists", theme);

                for pl in user_playlists {
                    playlist_nav_button(ui, current_tab, pl.id, &pl.name, pl.track_count, theme);
                }

                ui.add_space(8.0);

                // Section 5: PREFERENCES & INFO
                section_header(ui, "PREFERENCES", None, theme);
                nav_button(ui, current_tab, NavTab::Settings, Icon::Settings, "Settings", theme);
                nav_button(ui, current_tab, NavTab::About, Icon::Info, "About", theme);

                ui.add_space(14.0);

                // Library statistics card spanning full width of the sidebar footer with 10px gap
                let stats_frame = theme.card_frame(8.0);
                stats_frame.show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing = vec2(10.0, 0.0);
                        let (icon_rect, _) = ui.allocate_exact_size(vec2(20.0, 20.0), egui::Sense::hover());
                        paint_icon(ui.painter(), Icon::Library, icon_rect, theme.text_muted);

                        ui.vertical(|ui| {
                            ui.label(
                                RichText::new(format!("{} Tracks", total_tracks))
                                    .size(11.5)
                                    .color(theme.text_primary)
                                    .strong(),
                            );
                            ui.label(
                                RichText::new(format!("{} Playlists", total_playlists))
                                    .size(10.0)
                                    .color(theme.text_muted),
                            );
                        });
                    });
                });

                ui.add_space(10.0);
            });
    });
}

fn section_header(ui: &mut Ui, label: &str, count: Option<usize>, theme: &Theme) {
    ui.add_space(6.0);
    let width = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(vec2(width, 16.0), egui::Sense::hover());
    let font = egui::FontId::proportional(10.5);

    ui.painter().text(
        egui::pos2(rect.min.x + 8.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        label,
        font.clone(),
        theme.text_muted,
    );

    if let Some(c) = count {
        ui.painter().text(
            egui::pos2(rect.max.x - 8.0, rect.center().y),
            egui::Align2::RIGHT_CENTER,
            c.to_string(),
            font,
            theme.text_muted,
        );
    }
    ui.add_space(2.0);
}

fn disabled_stream_item(
    ui: &mut Ui,
    icon: Icon,
    label: &str,
    theme: &Theme,
) {
    let width = ui.available_width();
    let height = 32.0;

    let (rect, resp) = ui.allocate_exact_size(vec2(width, height), egui::Sense::hover());

    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::NotAllowed);
    }

    let icon_rect = Rect::from_center_size(
        egui::pos2(rect.min.x + 18.0, rect.center().y),
        vec2(16.0, 16.0),
    );
    paint_icon(ui.painter(), icon, icon_rect, theme.text_muted);

    let text_left = rect.min.x + 36.0;
    ui.painter().text(
        egui::pos2(text_left, rect.center().y),
        egui::Align2::LEFT_CENTER,
        label,
        egui::FontId::proportional(13.0),
        theme.text_muted,
    );
}

fn playlist_nav_button(
    ui: &mut Ui,
    current: &mut NavTab,
    id: i64,
    name: &str,
    track_count: usize,
    theme: &Theme,
) {
    let target = NavTab::Playlist(id);
    let is_selected = *current == target;
    let width = ui.available_width();
    let height = 34.0;

    let (rect, resp) = ui.allocate_exact_size(vec2(width, height), egui::Sense::click());
    let is_hovered = resp.hovered();

    if is_hovered {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }

    // Active pill Umbra Obsidian tokens:
    // 1. Active: background: rgba(255, 255, 255, 0.07), color: #ffffff, border: none, border-radius: 8px
    // 2. Hover (inactive): background: rgba(255, 255, 255, 0.04), color: #e4e4e7, border: none
    // 3. Default (inactive): background: transparent, color: #a1a1aa, border: none
    let (fill, stroke, text_color, icon_color) = if is_selected {
        (
            Color32::from_rgba_premultiplied(255, 255, 255, 18), // rgba(255, 255, 255, 0.07)
            Stroke::NONE,
            Color32::WHITE,
            Color32::WHITE,
        )
    } else if is_hovered {
        (
            Color32::from_rgba_premultiplied(255, 255, 255, 10), // rgba(255, 255, 255, 0.04)
            Stroke::NONE,
            Color32::from_rgb(228, 228, 231), // #e4e4e7
            Color32::from_rgb(228, 228, 231), // #e4e4e7
        )
    } else {
        (
            Color32::TRANSPARENT,
            Stroke::NONE,
            Color32::from_rgb(161, 161, 170), // #a1a1aa
            Color32::from_rgb(161, 161, 170), // #a1a1aa
        )
    };

    ui.painter().rect_filled(rect, Rounding::same(8.0), fill);
    if stroke != Stroke::NONE {
        ui.painter().rect_stroke(rect, Rounding::same(8.0), stroke);
    }

    // Left Vertical White Indicator Bar (Umbra Style):
    // position: absolute; left: -6px; top: 50%; transform: translateY(-50%);
    // width: 3px; height: 16px; background-color: #ffffff; border-radius: 999px;
    if is_selected {
        let bar_rect = Rect::from_min_size(
            pos2(rect.min.x - 6.0, rect.center().y - 8.0),
            vec2(3.0, 16.0),
        );
        ui.painter().rect_filled(bar_rect, Rounding::same(999.0), Color32::WHITE);
    }

    // Vector playlist icon
    let icon_rect = Rect::from_center_size(
        egui::pos2(rect.min.x + 18.0, rect.center().y),
        vec2(15.0, 15.0),
    );
    paint_icon(ui.painter(), Icon::Playlist, icon_rect, icon_color);

    // Track count badge on right edge
    let count_str = format!("{}", track_count);
    let count_font = egui::FontId::proportional(10.5);
    let count_w = ui
        .painter()
        .layout_no_wrap(count_str.clone(), count_font.clone(), theme.text_muted)
        .rect
        .width();
    let count_pos = egui::pos2(rect.max.x - 10.0, rect.center().y);
    ui.painter().text(
        count_pos,
        egui::Align2::RIGHT_CENTER,
        count_str,
        count_font,
        theme.text_muted,
    );

    // Truncated playlist title
    let text_left = rect.min.x + 34.0;
    let max_text_w = (rect.max.x - count_w - 20.0 - text_left).max(20.0);
    let label_font = egui::FontId::proportional(12.5);
    let text_w = ui
        .painter()
        .layout_no_wrap(name.to_string(), label_font.clone(), text_color)
        .rect
        .width();

    let display_name = if text_w > max_text_w {
        let mut truncated = String::new();
        for c in name.chars() {
            let candidate = format!("{}…", truncated);
            let w = ui
                .painter()
                .layout_no_wrap(candidate, label_font.clone(), text_color)
                .rect
                .width();
            if w > max_text_w {
                break;
            }
            truncated.push(c);
        }
        format!("{}…", truncated)
    } else {
        name.to_string()
    };

    ui.painter().text(
        egui::pos2(text_left, rect.center().y),
        egui::Align2::LEFT_CENTER,
        display_name,
        label_font,
        text_color,
    );

    if resp.clicked() {
        *current = target;
    }
}

fn nav_button(
    ui: &mut Ui,
    current: &mut NavTab,
    target: NavTab,
    icon: Icon,
    label: &str,
    _theme: &Theme,
) {
    let is_selected = *current == target;
    let width = ui.available_width();
    let height = 34.0;

    let (rect, resp) = ui.allocate_exact_size(vec2(width, height), egui::Sense::click());
    let is_hovered = resp.hovered();

    if is_hovered {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }

    // Active pill Umbra Obsidian tokens:
    // 1. Active: background: rgba(255, 255, 255, 0.07), color: #ffffff, border: none, border-radius: 8px
    // 2. Hover (inactive): background: rgba(255, 255, 255, 0.04), color: #e4e4e7, border: none
    // 3. Default (inactive): background: transparent, color: #a1a1aa, border: none
    let (fill, stroke, text_color, icon_color) = if is_selected {
        (
            Color32::from_rgba_premultiplied(255, 255, 255, 18), // rgba(255, 255, 255, 0.07)
            Stroke::NONE,
            Color32::WHITE,
            Color32::WHITE,
        )
    } else if is_hovered {
        (
            Color32::from_rgba_premultiplied(255, 255, 255, 10), // rgba(255, 255, 255, 0.04)
            Stroke::NONE,
            Color32::from_rgb(228, 228, 231), // #e4e4e7
            Color32::from_rgb(228, 228, 231), // #e4e4e7
        )
    } else {
        (
            Color32::TRANSPARENT,
            Stroke::NONE,
            Color32::from_rgb(161, 161, 170), // #a1a1aa
            Color32::from_rgb(161, 161, 170), // #a1a1aa
        )
    };

    ui.painter().rect_filled(rect, Rounding::same(8.0), fill);
    if stroke != Stroke::NONE {
        ui.painter().rect_stroke(rect, Rounding::same(8.0), stroke);
    }

    // Left Vertical White Indicator Bar (Umbra Style):
    // position: absolute; left: -6px; top: 50%; transform: translateY(-50%);
    // width: 3px; height: 16px; background-color: #ffffff; border-radius: 999px;
    if is_selected {
        let bar_rect = Rect::from_min_size(
            pos2(rect.min.x - 6.0, rect.center().y - 8.0),
            vec2(3.0, 16.0),
        );
        ui.painter().rect_filled(bar_rect, Rounding::same(999.0), Color32::WHITE);
    }

    // Clean vector icon (16px)
    let icon_rect = Rect::from_center_size(
        egui::pos2(rect.min.x + 18.0, rect.center().y),
        vec2(16.0, 16.0),
    );
    paint_icon(ui.painter(), icon, icon_rect, icon_color);

    // Text label
    ui.painter().text(
        egui::pos2(rect.min.x + 36.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        label,
        egui::FontId::proportional(13.0),
        text_color,
    );

    if resp.clicked() {
        *current = target;
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_sidebar_dimensions_and_tabs() {
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let mut current_tab = NavTab::Home;
                let theme = Theme::from_name("Eclipse", None);
                let playlists = vec![];
                let mut on_create = false;

                render_sidebar(
                    ui,
                    &mut current_tab,
                    &playlists,
                    &theme,
                    150,
                    3,
                    &mut on_create,
                    true,
                    true,
                );

                assert_eq!(current_tab, NavTab::Home);
            });
        });
    }
}
