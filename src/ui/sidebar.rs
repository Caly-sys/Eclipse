use crate::library::model::Playlist;
use crate::ui::icons::{paint_icon, Icon};
use crate::ui::theme::Theme;
use egui::{vec2, Color32, Layout, Rect, RichText, Rounding, ScrollArea, Stroke, Ui};

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
                section_header(ui, "LIBRARY", theme);
                nav_button(ui, current_tab, NavTab::Home, Icon::Home, "Home", theme);
                nav_button(ui, current_tab, NavTab::NowPlaying, Icon::Disc, "Now Playing", theme);
                nav_button(ui, current_tab, NavTab::Songs, Icon::Music, "Songs", theme);
                nav_button(ui, current_tab, NavTab::Artists, Icon::Mic, "Artists", theme);
                nav_button(ui, current_tab, NavTab::Albums, Icon::Disc, "Albums", theme);
                nav_button(ui, current_tab, NavTab::Genres, Icon::Tag, "Genres", theme);
                nav_button(ui, current_tab, NavTab::Folders, Icon::Folder, "Folders", theme);

                ui.add_space(8.0);

                // Section 2: SMART PLAYLISTS
                section_header(ui, "SMART", theme);
                nav_button(ui, current_tab, NavTab::RecentlyPlayed, Icon::Clock, "Recently Played", theme);
                nav_button(ui, current_tab, NavTab::MostPlayed, Icon::Fire, "Most Played", theme);
                nav_button(ui, current_tab, NavTab::RecentlyAdded, Icon::Sparkle, "Recently Added", theme);
                nav_button(ui, current_tab, NavTab::Favorites, Icon::Heart, "Favorites", theme);
                nav_button(ui, current_tab, NavTab::NeverPlayed, Icon::Hourglass, "Never Played", theme);
                nav_button(ui, current_tab, NavTab::RandomDiscovery, Icon::Dices, "Random Discovery", theme);

                ui.add_space(8.0);

                // Section 3: STREAM
                if show_spotify || show_youtube {
                    section_header(ui, "STREAM", theme);
                    if show_spotify {
                        disabled_stream_item(ui, Icon::Spotify, Color32::from_rgb(30, 215, 96), "Spotify", "Disabled", theme);
                    }
                    if show_youtube {
                        disabled_stream_item(ui, Icon::YouTube, Color32::from_rgb(255, 68, 68), "YouTube Music", "Disabled", theme);
                    }
                    ui.add_space(8.0);
                }

                // Section 4: PLAYLISTS with '+' button
                ui.horizontal(|ui| {
                    section_header_label(ui, "PLAYLISTS", theme);
                    ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                        let (plus_rect, plus_resp) = ui.allocate_exact_size(vec2(22.0, 20.0), egui::Sense::click());
                        let plus_bg = if plus_resp.hovered() {
                            theme.surface_hover
                        } else {
                            theme.card
                        };
                        let plus_stroke = if plus_resp.hovered() {
                            Stroke::new(1.0_f32, theme.border_accent)
                        } else {
                            Stroke::new(1.0_f32, theme.border)
                        };
                        ui.painter().rect_filled(plus_rect, Rounding::same(5.0), plus_bg);
                        ui.painter().rect_stroke(plus_rect, Rounding::same(5.0), plus_stroke);
                        let icon_rect = Rect::from_center_size(plus_rect.center(), vec2(12.0, 12.0));
                        let icon_color = if plus_resp.hovered() {
                            theme.accent
                        } else {
                            theme.text_secondary
                        };
                        paint_icon(ui.painter(), Icon::Plus, icon_rect, icon_color);

                        if plus_resp.hovered() {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                        }
                        if plus_resp.clicked() {
                            *on_create_playlist = true;
                        }
                    });
                });

                nav_button(ui, current_tab, NavTab::Playlists, Icon::Playlist, "All Playlists", theme);

                for pl in user_playlists {
                    playlist_nav_button(ui, current_tab, pl.id, &pl.name, pl.track_count, theme);
                }

                ui.add_space(8.0);

                // Section 5: PREFERENCES & INFO
                section_header(ui, "PREFERENCES", theme);
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
                        paint_icon(ui.painter(), Icon::Library, icon_rect, theme.accent);

                        ui.vertical(|ui| {
                            ui.label(
                                RichText::new(format!("{} Tracks", total_tracks))
                                    .size(11.5)
                                    .color(theme.text_secondary)
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

fn section_header(ui: &mut Ui, label: &str, theme: &Theme) {
    ui.add_space(4.0);
    section_header_label(ui, label, theme);
    ui.add_space(2.0);
}

fn section_header_label(ui: &mut Ui, label: &str, theme: &Theme) {
    ui.label(
        RichText::new(label)
            .size(10.0)
            .color(theme.text_muted)
            .strong(),
    );
}

fn disabled_stream_item(
    ui: &mut Ui,
    icon: Icon,
    icon_color: Color32,
    label: &str,
    badge: &str,
    theme: &Theme,
) {
    let width = ui.available_width();
    let height = 32.0;

    let (rect, resp) = ui.allocate_exact_size(vec2(width, height), egui::Sense::hover());

    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::NotAllowed);
    }

    // 1. Right compact pill badge: font-size 10px, padding 2px 6px, border-radius 4px
    let badge_font = egui::FontId::proportional(10.0);
    let badge_text_w = ui
        .painter()
        .layout_no_wrap(badge.to_string(), badge_font.clone(), theme.text_muted)
        .rect
        .width();
    let badge_padding_x = 6.0;
    let badge_w = badge_text_w + badge_padding_x * 2.0;
    let badge_h = 18.0;
    let badge_rect = Rect::from_min_size(
        egui::pos2(rect.max.x - badge_w - 6.0, rect.center().y - badge_h / 2.0),
        vec2(badge_w, badge_h),
    );

    let badge_bg = Color32::from_rgba_premultiplied(180, 140, 255, 18);
    let badge_stroke = Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(180, 140, 255, 35));
    ui.painter().rect_filled(badge_rect, Rounding::same(4.0), badge_bg);
    ui.painter().rect_stroke(badge_rect, Rounding::same(4.0), badge_stroke);
    ui.painter().text(
        badge_rect.center(),
        egui::Align2::CENTER_CENTER,
        badge,
        badge_font,
        Color32::from_rgba_premultiplied(170, 160, 195, 200),
    );

    // 2. Clean vector stream icon on left
    let icon_rect = Rect::from_center_size(
        egui::pos2(rect.min.x + 16.0, rect.center().y),
        vec2(16.0, 16.0),
    );
    // Dim icon slightly to indicate disabled state
    let muted_icon_color = Color32::from_rgba_premultiplied(
        icon_color.r() / 2,
        icon_color.g() / 2,
        icon_color.b() / 2,
        180,
    );
    paint_icon(ui.painter(), icon, icon_rect, muted_icon_color);

    // 3. Label between icon and badge with clean truncation
    let text_left = rect.min.x + 32.0;
    let max_text_width = (badge_rect.min.x - text_left - 8.0).max(20.0);
    let label_font = egui::FontId::proportional(12.5);

    let full_text_w = ui
        .painter()
        .layout_no_wrap(label.to_string(), label_font.clone(), theme.text_muted)
        .rect
        .width();

    let display_label = if full_text_w > max_text_width {
        let mut truncated = String::new();
        for c in label.chars() {
            let candidate = format!("{}…", truncated);
            let w = ui
                .painter()
                .layout_no_wrap(candidate, label_font.clone(), theme.text_muted)
                .rect
                .width();
            if w > max_text_width {
                break;
            }
            truncated.push(c);
        }
        format!("{}…", truncated)
    } else {
        label.to_string()
    };

    ui.painter().text(
        egui::pos2(text_left, rect.center().y),
        egui::Align2::LEFT_CENTER,
        display_label,
        label_font,
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

    // Active pill contrast:
    // Glowing accent background: rgba(168, 85, 247, 0.18) -> alpha ~46
    // Purple border: rgba(192, 132, 252, 0.40) -> alpha ~102
    // Text: #f3e8ff crisp light lavender
    // Indicator bar: prominent 3px left vertical accent bar indicator #c084fc
    let (fill, stroke, text_color, icon_color) = if is_selected {
        (
            Color32::from_rgba_premultiplied(168, 85, 247, 46),
            Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(192, 132, 252, 102)),
            Color32::from_rgb(243, 232, 255),
            Color32::from_rgb(192, 132, 252),
        )
    } else if is_hovered {
        (
            Color32::from_rgba_premultiplied(180, 140, 255, 20),
            Stroke::new(1.0_f32, theme.border),
            theme.text_primary,
            theme.text_primary,
        )
    } else {
        (
            Color32::TRANSPARENT,
            Stroke::NONE,
            theme.text_secondary,
            theme.text_muted,
        )
    };

    ui.painter().rect_filled(rect, Rounding::same(8.0), fill);
    if stroke != Stroke::NONE {
        ui.painter().rect_stroke(rect, Rounding::same(8.0), stroke);
    }

    // Prominent 3px left vertical accent bar indicator (#c084fc)
    if is_selected {
        let bar_rect = Rect::from_min_size(
            rect.min + vec2(2.5, 6.0),
            vec2(3.0, height - 12.0),
        );
        ui.painter().rect_filled(bar_rect, Rounding::same(1.5), Color32::from_rgb(192, 132, 252));
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
    theme: &Theme,
) {
    let is_selected = *current == target;
    let width = ui.available_width();
    let height = 34.0;

    let (rect, resp) = ui.allocate_exact_size(vec2(width, height), egui::Sense::click());
    let is_hovered = resp.hovered();

    if is_hovered {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }

    // Active pill contrast:
    // Glowing accent background: rgba(168, 85, 247, 0.18) -> alpha ~46
    // Purple border: rgba(192, 132, 252, 0.40) -> alpha ~102
    // Text: #f3e8ff crisp light lavender
    // Indicator bar: prominent 3px left vertical accent bar indicator #c084fc
    let (fill, stroke, text_color, icon_color) = if is_selected {
        (
            Color32::from_rgba_premultiplied(168, 85, 247, 46),
            Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(192, 132, 252, 102)),
            Color32::from_rgb(243, 232, 255),
            Color32::from_rgb(192, 132, 252),
        )
    } else if is_hovered {
        (
            Color32::from_rgba_premultiplied(180, 140, 255, 20),
            Stroke::new(1.0_f32, theme.border),
            theme.text_primary,
            theme.text_primary,
        )
    } else {
        (
            Color32::TRANSPARENT,
            Stroke::NONE,
            theme.text_secondary,
            theme.text_muted,
        )
    };

    ui.painter().rect_filled(rect, Rounding::same(8.0), fill);
    if stroke != Stroke::NONE {
        ui.painter().rect_stroke(rect, Rounding::same(8.0), stroke);
    }

    // Prominent 3px left vertical accent bar indicator (#c084fc)
    if is_selected {
        let bar_rect = Rect::from_min_size(
            rect.min + vec2(2.5, 6.0),
            vec2(3.0, height - 12.0),
        );
        ui.painter().rect_filled(bar_rect, Rounding::same(1.5), Color32::from_rgb(192, 132, 252));
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
