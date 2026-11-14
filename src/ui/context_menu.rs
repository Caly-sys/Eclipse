use crate::library::model::{Playlist, Track};
use crate::ui::icons::{paint_icon, Icon};
use crate::ui::theme::Theme;
use egui::{
    pos2, vec2, Align2, Color32, FontId, Pos2, Rect, RichText, Rounding, ScrollArea, Stroke, Ui,
    Vec2,
};

/// State holding the active track context menu.
#[derive(Clone, Debug)]
pub struct TrackContextMenuState {
    pub track: Track,
    pub anchor_pos: Pos2,
    pub from_playlist_id: Option<i64>,
    pub is_submenu_open: bool,
    pub playlist_search: String,
    pub is_creating_playlist: bool,
    pub new_playlist_name: String,
    pub main_menu_rect: Option<Rect>,
    pub submenu_rect: Option<Rect>,
}

impl TrackContextMenuState {
    pub fn new(track: Track, anchor_pos: Pos2, from_playlist_id: Option<i64>) -> Self {
        Self {
            track,
            anchor_pos,
            from_playlist_id,
            is_submenu_open: false,
            playlist_search: String::new(),
            is_creating_playlist: false,
            new_playlist_name: String::new(),
            main_menu_rect: None,
            submenu_rect: None,
        }
    }
}

/// Actions emitted when clicking any item in the context menu.
#[derive(Clone, Debug)]
pub enum TrackContextMenuAction {
    PlayNext(Track),
    AddToQueue(Track),
    AddToPlaylist { playlist_id: i64, track_id: i64 },
    CreatePlaylistAndAdd { name: String, track_id: i64 },
    RemoveFromPlaylist { playlist_id: i64, track_id: i64 },
    ToggleFavorite(i64),
    GoToArtist(String),
    GoToAlbum(String),
    ShowInExplorer(String),
}

/// Single interactive row inside the context menu.
fn context_menu_button(
    ui: &mut Ui,
    icon: Icon,
    label: &str,
    has_submenu: bool,
    theme: &Theme,
    danger: bool,
) -> egui::Response {
    let desired_size = vec2(ui.available_width(), 26.0);
    let (rect, resp) = ui.allocate_exact_size(desired_size, egui::Sense::click());

    let is_hovered = resp.hovered();
    if is_hovered {
        let hover_bg = if danger {
            Color32::from_rgba_premultiplied(239, 68, 68, 30)
        } else {
            theme.surface_hover
        };
        ui.painter().rect_filled(rect, Rounding::same(4.0), hover_bg);
    }

    let text_color = if danger {
        Color32::from_rgb(239, 68, 68)
    } else if is_hovered {
        theme.text_primary
    } else {
        theme.text_secondary
    };

    let icon_color = if danger {
        Color32::from_rgb(239, 68, 68)
    } else if is_hovered {
        theme.accent
    } else {
        theme.text_muted
    };

    // Vector Icon
    let icon_r = Rect::from_center_size(pos2(rect.min.x + 14.0, rect.center().y), vec2(12.0, 12.0));
    paint_icon(ui.painter(), icon, icon_r, icon_color);

    // Text Label
    ui.painter().text(
        pos2(rect.min.x + 32.0, rect.center().y),
        Align2::LEFT_CENTER,
        label,
        FontId::proportional(12.0),
        text_color,
    );

    // Submenu indicator (Chevron Right)
    if has_submenu {
        let chev_r = Rect::from_center_size(pos2(rect.max.x - 12.0, rect.center().y), vec2(10.0, 10.0));
        paint_icon(ui.painter(), Icon::ChevronRight, chev_r, theme.text_muted);
    }

    resp
}

/// Subtle hairline horizontal separator inside context menus.
fn context_menu_divider(ui: &mut Ui, theme: &Theme) {
    ui.add_space(2.0);
    let (line_rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 1.0), egui::Sense::hover());
    ui.painter().rect_filled(line_rect, Rounding::ZERO, theme.border);
    ui.add_space(2.0);
}

/// Calculates smart viewport-constrained position for the main context menu.
pub fn calculate_menu_position(anchor: Pos2, menu_size: Vec2, screen_rect: Rect) -> Pos2 {
    let playerbar_height = 72.0;
    let titlebar_height = 36.0;
    let min_x = screen_rect.min.x + 8.0;
    let max_x = screen_rect.max.x - 8.0;
    let min_y = screen_rect.min.y + titlebar_height + 4.0;
    let max_y = screen_rect.max.y - playerbar_height - 8.0;

    let mut menu_x = anchor.x;
    if menu_x + menu_size.x > max_x {
        menu_x = (anchor.x - menu_size.x).max(min_x);
    }
    let mut menu_y = anchor.y;
    if menu_y + menu_size.y > max_y {
        menu_y = (anchor.y - menu_size.y).max(min_y);
    }
    pos2(menu_x, menu_y)
}

/// Calculates smart viewport-constrained position for the cascading submenu.
pub fn calculate_submenu_position(
    main_rect: Rect,
    anchor_y: f32,
    sub_size: Vec2,
    screen_rect: Rect,
) -> Pos2 {
    let playerbar_height = 72.0;
    let titlebar_height = 36.0;
    let min_x = screen_rect.min.x + 8.0;
    let max_x = screen_rect.max.x - 8.0;
    let min_y = screen_rect.min.y + titlebar_height + 4.0;
    let max_y = screen_rect.max.y - playerbar_height - 8.0;

    let mut sub_x = main_rect.max.x + 2.0;
    if sub_x + sub_size.x > max_x {
        sub_x = (main_rect.min.x - sub_size.x - 2.0).max(min_x);
    }

    let mut sub_y = anchor_y;
    if sub_y + sub_size.y > max_y {
        sub_y = (max_y - sub_size.y).max(min_y);
    }
    pos2(sub_x, sub_y)
}

/// Renders the floating Track Context Menu and cascading "Add to playlist" submenu.
/// Uses `egui::Area` on `Order::Foreground` to guarantee zero clipping inside scroll containers.
pub fn render_track_context_menu(
    ctx: &egui::Context,
    state: &mut Option<TrackContextMenuState>,
    playlists: &[Playlist],
    theme: &Theme,
) -> Option<TrackContextMenuAction> {
    let mut current_state = state.take()?;

    // 1. Dismissal checks: Escape key
    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        return None;
    }

    // 2. Dismissal checks: Click outside (only evaluated once menu has rendered at least one frame)
    if let Some(main_rect) = current_state.main_menu_rect {
        if ctx.input(|i| i.pointer.any_click()) {
            if let Some(click_pos) = ctx.input(|i| i.pointer.interact_pos()) {
                let in_main = main_rect.expand(4.0).contains(click_pos);
                let in_sub = current_state.is_submenu_open
                    && current_state
                        .submenu_rect
                        .is_some_and(|r| r.expand(4.0).contains(click_pos));
                if !in_main && !in_sub {
                    return None;
                }
            }
        }
    }

    // 3. Dismissal checks: Scroll outside menu (only evaluated once menu has rendered at least one frame)
    if let Some(main_rect) = current_state.main_menu_rect {
        let scrolled = ctx.input(|i| i.smooth_scroll_delta.y.abs() > 0.0 || i.raw_scroll_delta.y.abs() > 0.0);
        if scrolled {
            if let Some(hover_pos) = ctx.input(|i| i.pointer.hover_pos()) {
                let in_main = main_rect.contains(hover_pos);
                let in_sub = current_state.is_submenu_open
                    && current_state
                        .submenu_rect
                        .is_some_and(|r| r.contains(hover_pos));
                if !in_main && !in_sub {
                    return None;
                }
            }
        }
    }

    // Viewport geometry & boundaries
    let screen_rect = ctx.screen_rect();

    let main_w = 205.0;
    let main_estimated_h = if current_state.from_playlist_id.is_some() {
        275.0
    } else {
        245.0
    };

    // Calculate smart position for Main Menu
    let menu_pos = calculate_menu_position(
        current_state.anchor_pos,
        vec2(main_w, main_estimated_h),
        screen_rect,
    );
    let menu_x = menu_pos.x;
    let menu_y = menu_pos.y;

    let mut action_to_emit: Option<TrackContextMenuAction> = None;
    let mut add_to_playlist_rect = Rect::ZERO;
    let mut main_rect_out = Rect::ZERO;

    // Render Primary Context Menu
    egui::Area::new(egui::Id::new("track_context_menu_overlay"))
        .order(egui::Order::Foreground)
        .fixed_pos(pos2(menu_x, menu_y))
        .show(ctx, |ui| {
            let mut frame = theme.card_frame(8.0);
            frame.fill = Color32::from_rgb(18, 18, 22);
            frame.stroke = Stroke::new(1.0_f32, theme.border_accent);
            frame.inner_margin = egui::Margin::symmetric(6.0, 6.0);

            frame.show(ui, |ui| {
                ui.set_width(main_w - 12.0);
                ui.spacing_mut().item_spacing = vec2(0.0, 2.0);

                // Section 1: Playlists
                let add_pl_resp = context_menu_button(ui, Icon::Playlist, "Add to playlist", true, theme, false);
                add_to_playlist_rect = add_pl_resp.rect;
                if add_pl_resp.hovered() || add_pl_resp.clicked() {
                    current_state.is_submenu_open = true;
                }

                if let Some(current_pl_id) = current_state.from_playlist_id {
                    let remove_pl_resp = context_menu_button(
                        ui,
                        Icon::Trash,
                        "Remove from playlist",
                        false,
                        theme,
                        true,
                    );
                    if remove_pl_resp.clicked() {
                        action_to_emit = Some(TrackContextMenuAction::RemoveFromPlaylist {
                            playlist_id: current_pl_id,
                            track_id: current_state.track.id,
                        });
                    }
                    if remove_pl_resp.hovered() {
                        current_state.is_submenu_open = false;
                    }
                }

                // Section 2: Favorite / Liked
                let (fav_icon, fav_label) = if current_state.track.favorite {
                    (Icon::Heart, "Remove from Liked Songs")
                } else {
                    (Icon::HeartOutline, "Add to Liked Songs")
                };
                let fav_resp = context_menu_button(ui, fav_icon, fav_label, false, theme, false);
                if fav_resp.clicked() {
                    action_to_emit = Some(TrackContextMenuAction::ToggleFavorite(current_state.track.id));
                }
                if fav_resp.hovered() {
                    current_state.is_submenu_open = false;
                }

                context_menu_divider(ui, theme);

                // Section 3: Queue Management
                let play_next_resp = context_menu_button(ui, Icon::Play, "Play next", false, theme, false);
                if play_next_resp.clicked() {
                    action_to_emit = Some(TrackContextMenuAction::PlayNext(current_state.track.clone()));
                }
                if play_next_resp.hovered() {
                    current_state.is_submenu_open = false;
                }

                let add_queue_resp = context_menu_button(ui, Icon::Queue, "Add to queue", false, theme, false);
                if add_queue_resp.clicked() {
                    action_to_emit = Some(TrackContextMenuAction::AddToQueue(current_state.track.clone()));
                }
                if add_queue_resp.hovered() {
                    current_state.is_submenu_open = false;
                }

                context_menu_divider(ui, theme);

                // Section 4: Navigation & File Explorer
                if !current_state.track.artist.trim().is_empty() {
                    let artist_resp = context_menu_button(ui, Icon::Mic, "Go to artist", false, theme, false);
                    if artist_resp.clicked() {
                        action_to_emit = Some(TrackContextMenuAction::GoToArtist(current_state.track.artist.clone()));
                    }
                    if artist_resp.hovered() {
                        current_state.is_submenu_open = false;
                    }
                }

                if !current_state.track.album.trim().is_empty() {
                    let album_resp = context_menu_button(ui, Icon::Disc, "Go to album", false, theme, false);
                    if album_resp.clicked() {
                        action_to_emit = Some(TrackContextMenuAction::GoToAlbum(current_state.track.album.clone()));
                    }
                    if album_resp.hovered() {
                        current_state.is_submenu_open = false;
                    }
                }

                let explorer_resp = context_menu_button(ui, Icon::Folder, "Show in file explorer", false, theme, false);
                if explorer_resp.clicked() {
                    action_to_emit = Some(TrackContextMenuAction::ShowInExplorer(current_state.track.path.clone()));
                }
                if explorer_resp.hovered() {
                    current_state.is_submenu_open = false;
                }

                main_rect_out = ui.min_rect();
            });
        });

    current_state.main_menu_rect = Some(main_rect_out);

    // Render Cascading Submenu (Add to playlist)
    if current_state.is_submenu_open {
        let sub_w = 215.0;
        let sub_estimated_h = 240.0;

        // Smart Submenu Positioning:
        let sub_pos = calculate_submenu_position(
            main_rect_out,
            add_to_playlist_rect.min.y,
            vec2(sub_w, sub_estimated_h),
            screen_rect,
        );
        let sub_x = sub_pos.x;
        let sub_y = sub_pos.y;

        let mut sub_rect_out = Rect::ZERO;

        egui::Area::new(egui::Id::new("track_context_submenu_overlay"))
            .order(egui::Order::Foreground)
            .fixed_pos(pos2(sub_x, sub_y))
            .show(ctx, |ui| {
                let mut frame = theme.card_frame(8.0);
                frame.fill = Color32::from_rgb(18, 18, 22);
                frame.stroke = Stroke::new(1.0_f32, theme.border_accent);
                frame.inner_margin = egui::Margin::symmetric(6.0, 6.0);

                frame.show(ui, |ui| {
                    ui.set_width(sub_w - 12.0);
                    ui.spacing_mut().item_spacing = vec2(0.0, 2.0);

                    // 1. Search Filter Input: "Find a playlist"
                    ui.horizontal(|ui| {
                        let search_icon_r = Rect::from_center_size(
                            pos2(ui.cursor().min.x + 10.0, ui.cursor().min.y + 11.0),
                            vec2(11.0, 11.0),
                        );
                        paint_icon(ui.painter(), Icon::Search, search_icon_r, theme.text_muted);

                        ui.add_space(16.0);
                        let text_edit = egui::TextEdit::singleline(&mut current_state.playlist_search)
                            .hint_text("Find a playlist")
                            .desired_width(sub_w - 44.0);
                        ui.add(text_edit);
                    });

                    context_menu_divider(ui, theme);

                    // 2. "+ New playlist" creation row
                    if current_state.is_creating_playlist {
                        ui.horizontal(|ui| {
                            let text_resp = ui.add(
                                egui::TextEdit::singleline(&mut current_state.new_playlist_name)
                                    .hint_text("Playlist name")
                                    .desired_width(120.0),
                            );
                            if text_resp.lost_focus() && ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
                                let name = current_state.new_playlist_name.trim();
                                if !name.is_empty() {
                                    action_to_emit = Some(TrackContextMenuAction::CreatePlaylistAndAdd {
                                        name: name.to_string(),
                                        track_id: current_state.track.id,
                                    });
                                }
                            }

                            let create_btn = egui::Button::new(
                                RichText::new("Save").size(11.0).color(Color32::from_rgb(11, 9, 20)).strong(),
                            )
                            .fill(theme.accent)
                            .rounding(Rounding::same(4.0));
                            if ui.add(create_btn).clicked() {
                                let name = current_state.new_playlist_name.trim();
                                if !name.is_empty() {
                                    action_to_emit = Some(TrackContextMenuAction::CreatePlaylistAndAdd {
                                        name: name.to_string(),
                                        track_id: current_state.track.id,
                                    });
                                }
                            }
                        });
                    } else {
                        let new_pl_resp = context_menu_button(ui, Icon::Plus, "New playlist", false, theme, false);
                        if new_pl_resp.clicked() {
                            current_state.is_creating_playlist = true;
                            if current_state.new_playlist_name.is_empty() {
                                current_state.new_playlist_name = format!("My Playlist #{}", playlists.len() + 1);
                            }
                        }
                    }

                    context_menu_divider(ui, theme);

                    // 3. Scrollable Playlist List
                    let query = current_state.playlist_search.trim().to_lowercase();
                    let matching_playlists: Vec<&Playlist> = playlists
                        .iter()
                        .filter(|p| {
                            if query.is_empty() {
                                true
                            } else {
                                p.name.to_lowercase().contains(&query)
                            }
                        })
                        .collect();

                    if matching_playlists.is_empty() {
                        ui.add_space(4.0);
                        ui.vertical_centered(|ui| {
                            ui.label(RichText::new("No playlists found").size(11.0).color(theme.text_muted));
                        });
                        ui.add_space(4.0);
                    } else {
                        ScrollArea::vertical()
                            .id_salt("context_menu_playlists_scroll")
                            .max_height(160.0)
                            .auto_shrink([false, true])
                            .show(ui, |ui| {
                                ui.spacing_mut().item_spacing = vec2(0.0, 2.0);
                                for pl in matching_playlists {
                                    let label = if pl.name.len() > 18 {
                                        format!("{}...", &pl.name[..15])
                                    } else {
                                        pl.name.clone()
                                    };
                                    let pl_resp = context_menu_button(
                                        ui,
                                        Icon::Playlist,
                                        &label,
                                        false,
                                        theme,
                                        false,
                                    );
                                    if pl_resp.clicked() {
                                        action_to_emit = Some(TrackContextMenuAction::AddToPlaylist {
                                            playlist_id: pl.id,
                                            track_id: current_state.track.id,
                                        });
                                    }
                                }
                            });
                    }

                    sub_rect_out = ui.min_rect();
                });
            });

        current_state.submenu_rect = Some(sub_rect_out);
    } else {
        current_state.submenu_rect = None;
    }

    if action_to_emit.is_some() {
        *state = None; // Close menu after selecting an action
    } else {
        *state = Some(current_state); // Persist updated menu state for next frame
    }

    action_to_emit
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_track() -> Track {
        Track {
            id: 42,
            path: "C:/music/song.flac".into(),
            title: "Test Song".into(),
            artist: "Test Artist".into(),
            album_artist: "Test Artist".into(),
            album: "Test Album".into(),
            genre: "Rock".into(),
            year: 2024,
            track_number: 1,
            duration_secs: 200.0,
            bitrate_kbps: 1411,
            sample_rate: 44100,
            channels: 2,
            format: "FLAC".into(),
            file_size: 20_000_000,
            play_count: 5,
            favorite: false,
            has_art: false,
            folder: "C:/music".into(),
        }
    }

    #[test]
    fn test_menu_survives_opening_click_frame() {
        let ctx = egui::Context::default();
        let track = dummy_track();

        // Simulate frame where user clicked to open context menu
        let mut state = Some(TrackContextMenuState::new(track.clone(), pos2(100.0, 100.0), None));
        let playlists: [Playlist; 0] = [];
        let theme = Theme::from_name("dark", None);

        let _ = ctx.run(
            egui::RawInput {
                events: vec![
                    egui::Event::PointerMoved(pos2(100.0, 100.0)),
                    egui::Event::PointerButton {
                        pos: pos2(100.0, 100.0),
                        button: egui::PointerButton::Primary,
                        pressed: true,
                        modifiers: Default::default(),
                    },
                    egui::Event::PointerButton {
                        pos: pos2(100.0, 100.0),
                        button: egui::PointerButton::Primary,
                        pressed: false,
                        modifiers: Default::default(),
                    },
                ],
                ..Default::default()
            },
            |ctx| {
                // In the opening frame, render_track_context_menu MUST NOT dismiss itself!
                let _ = render_track_context_menu(ctx, &mut state, &playlists, &theme);
            },
        );

        assert!(state.is_some(), "Context menu must remain open after the frame it was clicked open!");
    }

    #[test]
    fn test_menu_dismisses_on_subsequent_outside_click() {
        let ctx = egui::Context::default();
        let track = dummy_track();
        let playlists: [Playlist; 0] = [];
        let theme = Theme::from_name("dark", None);

        // Frame 1: Initial opening frame (menu renders and records main_menu_rect)
        let mut state = Some(TrackContextMenuState::new(track.clone(), pos2(100.0, 100.0), None));
        let _ = ctx.run(Default::default(), |ctx| {
            let _ = render_track_context_menu(ctx, &mut state, &playlists, &theme);
        });
        assert!(state.is_some(), "Menu must be rendered and populated on frame 1");
        assert!(state.as_ref().unwrap().main_menu_rect.is_some(), "main_menu_rect must be recorded");

        // Frame 2: User clicks outside the menu (at pos 500, 500)
        let _ = ctx.run(
            egui::RawInput {
                events: vec![
                    egui::Event::PointerMoved(pos2(500.0, 500.0)),
                    egui::Event::PointerButton {
                        pos: pos2(500.0, 500.0),
                        button: egui::PointerButton::Primary,
                        pressed: true,
                        modifiers: Default::default(),
                    },
                    egui::Event::PointerButton {
                        pos: pos2(500.0, 500.0),
                        button: egui::PointerButton::Primary,
                        pressed: false,
                        modifiers: Default::default(),
                    },
                ],
                ..Default::default()
            },
            |ctx| {
                let _ = render_track_context_menu(ctx, &mut state, &playlists, &theme);
            },
        );
        assert!(state.is_none(), "Menu must be dismissed when user clicks outside on subsequent frame");
    }

    #[test]
    fn test_viewport_bounds_flip_calculation() {
        let screen = Rect::from_min_size(pos2(0.0, 0.0), vec2(1000.0, 800.0));
        let menu_size = vec2(205.0, 245.0);

        // Case 1: Standard interior click (no flipping needed)
        let p1 = calculate_menu_position(pos2(200.0, 200.0), menu_size, screen);
        assert_eq!(p1, pos2(200.0, 200.0));

        // Case 2: Near bottom playerbar (anchor y = 700.0 in 800px screen)
        // Max Y allowed is 800 - 72 (playerbar) - 8 = 720.0
        // Menu height 245 -> 700 + 245 = 945 > 720, so flips upward
        let p2 = calculate_menu_position(pos2(200.0, 700.0), menu_size, screen);
        assert_eq!(p2.y, 700.0 - 245.0);
        assert!(p2.y + menu_size.y <= 720.0, "Menu must remain strictly above playerbar");

        // Case 3: Near right edge (anchor x = 900.0 in 1000px screen)
        // Max X allowed is 1000 - 8 = 992.0
        // Menu width 205 -> 900 + 205 = 1105 > 992, so flips to the left
        let p3 = calculate_menu_position(pos2(900.0, 200.0), menu_size, screen);
        assert_eq!(p3.x, 900.0 - 205.0);

        // Case 4: Submenu right edge flip
        // Main menu at x=750..955
        let main_rect = Rect::from_min_size(pos2(750.0, 200.0), vec2(205.0, 245.0));
        let sub_size = vec2(215.0, 240.0);
        // Default right placement: 955 + 2 = 957 + 215 = 1172 > 992 -> Flips to left!
        let sub_pos = calculate_submenu_position(main_rect, 200.0, sub_size, screen);
        assert_eq!(sub_pos.x, 750.0 - 215.0 - 2.0);
    }

    #[test]
    fn test_submenu_playlist_filtering() {
        let playlists = [
            Playlist {
                id: 1,
                name: "Late Night Chill".into(),
                description: "".into(),
                created: 0.0,
                track_count: 5,
            },
            Playlist {
                id: 2,
                name: "Workout Jams".into(),
                description: "".into(),
                created: 0.0,
                track_count: 10,
            },
            Playlist {
                id: 3,
                name: "Chillhop Beats".into(),
                description: "".into(),
                created: 0.0,
                track_count: 8,
            },
        ];

        // Empty filter matches all
        let q_empty = "".to_lowercase();
        let matches: Vec<_> = playlists.iter().filter(|p| q_empty.is_empty() || p.name.to_lowercase().contains(&q_empty)).collect();
        assert_eq!(matches.len(), 3);

        // Substring case-insensitive filter
        let q_chill = "chill".to_lowercase();
        let matches_chill: Vec<_> = playlists.iter().filter(|p| p.name.to_lowercase().contains(&q_chill)).collect();
        assert_eq!(matches_chill.len(), 2);
        assert_eq!(matches_chill[0].name, "Late Night Chill");
        assert_eq!(matches_chill[1].name, "Chillhop Beats");

        // No match
        let q_none = "classical".to_lowercase();
        let matches_none: Vec<_> = playlists.iter().filter(|p| p.name.to_lowercase().contains(&q_none)).collect();
        assert!(matches_none.is_empty());
    }
}
