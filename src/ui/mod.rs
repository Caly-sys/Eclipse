pub mod art;
pub mod components;
pub mod fonts;
pub mod icons;
pub mod pip;
pub mod playerbar;
pub mod resize;
pub mod shortcuts;
pub mod sidebar;
pub mod theme;
pub mod titlebar;
pub mod views;

use crate::audio::PlayerController;
use crate::config::AppConfig;
use crate::library::db::Database;
use crate::library::model::{Playlist, Track};
use crate::library::scanner::scan_directories;
use crate::ui::icons::{icon_button, paint_icon, render_icon, Icon};
use crate::ui::sidebar::NavTab;
use crate::ui::theme::Theme;
use crate::ui::views::*;
use eframe::App;
use egui::{vec2, Color32, Layout, RichText, Rounding, ScrollArea, Stroke};
use rand::seq::SliceRandom;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::time::Instant;

pub struct EclipseApp {
    pub cfg: AppConfig,
    pub theme: Theme,
    pub db: Database,
    pub player: PlayerController,
    pub tracks: Vec<Track>,
    pub playlists: Vec<Playlist>,
    pub recent_history: Vec<Track>,
    pub current_tab: NavTab,
    pub nav_history: Vec<NavTab>,
    pub nav_history_idx: usize,
    pub search_query: String,
    pub focus_search: bool,
    pub show_queue: bool,
    pub songs_state: SongsViewState,
    pub albums_state: AlbumsViewState,
    pub artists_state: ArtistsViewState,
    pub genres_state: GenresViewState,
    pub folders_state: FoldersViewState,
    pub playlists_state: PlaylistsViewState,
    pub art_manager: art::AlbumArtManager,
    scan_rx: Receiver<(usize, usize)>,
    scan_tx: Sender<(usize, usize)>,
    is_scanning: bool,
    status_msg: Option<(String, Instant)>,
}

pub type UmbraApp = EclipseApp;

impl EclipseApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let fonts = fonts::setup_custom_fonts();
        cc.egui_ctx.set_fonts(fonts);

        let cfg = AppConfig::load();
        let theme = Theme::from_name(&cfg.theme, Some(cfg.accent_color));
        let db = Database::open().expect("Failed to open Eclipse database");
        let mut player = PlayerController::new(&cfg);
        player.apply_audio_settings(&cfg.audio);

        let (scan_tx, scan_rx) = channel();

        let mut app = Self {
            cfg,
            theme,
            db,
            player,
            tracks: Vec::new(),
            playlists: Vec::new(),
            recent_history: Vec::new(),
            current_tab: NavTab::Home,
            nav_history: vec![NavTab::Home],
            nav_history_idx: 0,
            search_query: String::new(),
            focus_search: false,
            show_queue: false,
            songs_state: SongsViewState::default(),
            albums_state: AlbumsViewState::default(),
            artists_state: ArtistsViewState::default(),
            genres_state: GenresViewState::default(),
            folders_state: FoldersViewState::default(),
            playlists_state: PlaylistsViewState::default(),
            art_manager: art::AlbumArtManager::new(),
            scan_rx,
            scan_tx,
            is_scanning: false,
            status_msg: None,
        };

        app.reload_data();

        // If library is empty on startup, scan watched folders
        if app.tracks.is_empty() {
            app.trigger_scan();
        }

        app
    }

    pub fn navigate_to(&mut self, tab: NavTab) {
        if self.current_tab == tab {
            return;
        }
        if self.nav_history_idx + 1 < self.nav_history.len() {
            self.nav_history.truncate(self.nav_history_idx + 1);
        }
        self.nav_history.push(tab);
        self.nav_history_idx = self.nav_history.len() - 1;
        self.current_tab = tab;
    }

    pub fn can_go_back(&self) -> bool {
        self.nav_history_idx > 0
    }

    pub fn can_go_forward(&self) -> bool {
        self.nav_history_idx + 1 < self.nav_history.len()
    }

    pub fn go_back(&mut self) {
        if self.can_go_back() {
            self.nav_history_idx -= 1;
            self.current_tab = self.nav_history[self.nav_history_idx];
        }
    }

    pub fn go_forward(&mut self) {
        if self.can_go_forward() {
            self.nav_history_idx += 1;
            self.current_tab = self.nav_history[self.nav_history_idx];
        }
    }

    pub fn reload_data(&mut self) {
        if let Ok(tracks) = self.db.get_all_tracks() {
            self.tracks = tracks;
        }
        if let Ok(playlists) = self.db.get_playlists() {
            self.playlists = playlists;
        }
        if let Ok(history) = self.db.get_recent_history(10) {
            self.recent_history = history;
        }
    }

    pub fn trigger_scan(&mut self) {
        if self.is_scanning {
            return;
        }
        self.is_scanning = true;
        self.set_status("Scanning music folders...");

        let tx = self.scan_tx.clone();
        let folders = self.cfg.music_folders.clone();
        let db = self.db.clone();

        thread::spawn(move || {
            let res = scan_directories(&folders, &db);
            let _ = tx.send(res);
        });
    }

    pub fn set_status(&mut self, msg: &str) {
        self.status_msg = Some((msg.to_string(), Instant::now()));
    }
}

impl App for EclipseApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // High frame rate during playback for smooth spectrum visualizer and PiP overlay
        if self.player.engine.is_playing() || self.is_scanning || self.cfg.pip.enabled {
            ctx.request_repaint();
        }

        self.player.update();
        self.art_manager.update(ctx);

        // Keyboard shortcuts listener
        let shortcut = shortcuts::handle_shortcuts(ctx, &mut self.player);
        if let Some(msg) = shortcut.status_message {
            self.set_status(&msg);
        }
        if shortcut.focus_search {
            self.focus_search = true;
        }

        // Window border edge & corner resize detection
        resize::handle_window_resize(ctx);

        // Check if background scanning finished
        if let Ok((total, updated)) = self.scan_rx.try_recv() {
            self.is_scanning = false;
            self.reload_data();
            self.set_status(&format!("Scan finished: {} files found, {} updated", total, updated));
        }

        // Auto-navigate to Songs view if user presses Enter on a global search from other views
        if !self.search_query.trim().is_empty()
            && ctx.input(|i| i.key_pressed(egui::Key::Enter))
            && self.current_tab != NavTab::Songs
            && self.current_tab != NavTab::Albums
            && self.current_tab != NavTab::Artists
            && self.current_tab != NavTab::Genres
        {
            self.navigate_to(NavTab::Songs);
        }

        let mut on_scan = false;
        let mut on_rescan = false;
        let mut on_add_folder = false;
        let mut on_theme_changed = false;
        let mut on_toggle_fav = None;
        let mut on_play_track = None;
        let mut on_create_playlist = None;
        let mut on_delete_playlist = None;
        let mut toggle_now_playing = false;

        let can_back = self.can_go_back();
        let can_fwd = self.can_go_forward();
        let mut on_back = false;
        let mut on_forward = false;

        // Apply dark custom background with rounded corners when not fullscreen / not maximized
        let is_maximized = ctx.input(|i| i.viewport().maximized.unwrap_or(false));
        let is_fullscreen = ctx.input(|i| i.viewport().fullscreen.unwrap_or(false));
        let is_floating = !is_maximized && !is_fullscreen;

        let central_frame = self.theme.panel_frame_with_state(is_floating);

        egui::CentralPanel::default().frame(central_frame).show(ctx, |ui| {
            ui.spacing_mut().item_spacing = vec2(0.0, 8.0);

            // 1. TOP TITLEBAR
            titlebar::render_titlebar(
                ui,
                &self.theme,
                &mut self.search_query,
                &mut on_scan,
                &mut self.focus_search,
                can_back,
                can_fwd,
                &mut on_back,
                &mut on_forward,
            );

            // Toast / Status Message bar
            if let Some((ref msg, time)) = self.status_msg {
                if time.elapsed().as_secs() < 4 {
                    ui.add_space(2.0);
                    let toast_frame = egui::Frame::none()
                        .fill(self.theme.accent_glow)
                        .stroke(Stroke::new(1.0_f32, self.theme.border_accent))
                        .rounding(Rounding::same(8.0_f32))
                        .inner_margin(egui::Margin::symmetric(12.0_f32, 5.0_f32));

                    toast_frame.show(ui, |ui| {
                        ui.horizontal(|ui| {
                            render_icon(ui, Icon::Sparkle, 13.0, self.theme.accent);
                            ui.label(RichText::new(msg).color(self.theme.text_primary).size(12.0));
                        });
                    });
                }
            }

            // 2. MAIN WORKSPACE (Sidebar + Viewport + Queue drawer)
            // Dynamically calculate workspace height leaving exact space for docked bottom playerbar
            let playerbar_id = egui::Id::new("eclipse_docked_playerbar_height");
            let playerbar_h: f32 = ctx.data(|d| d.get_temp(playerbar_id)).unwrap_or(68.0);
            let workspace_height = (ui.available_height() - playerbar_h - ui.spacing().item_spacing.y).max(100.0);

            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = vec2(10.0, 0.0);

                // Sidebar Left
                let mut sidebar_frame = self.theme.glass_frame(12.0);
                sidebar_frame.inner_margin = egui::Margin::symmetric(12.0_f32, 10.0_f32);

                let sidebar_width = 240.0;
                let mut next_tab = self.current_tab;
                let mut on_create_sidebar_playlist = false;

                ui.allocate_ui_with_layout(
                    vec2(sidebar_width, workspace_height),
                    Layout::top_down(egui::Align::Min),
                    |ui| {
                        sidebar_frame.show(ui, |ui| {
                            sidebar::render_sidebar(
                                ui,
                                &mut next_tab,
                                &self.playlists,
                                &self.theme,
                                self.tracks.len(),
                                self.playlists.len(),
                                &mut on_create_sidebar_playlist,
                                self.cfg.streaming.show_spotify,
                                self.cfg.streaming.show_youtube_music,
                            );
                        });
                    },
                );

                if next_tab != self.current_tab {
                    self.navigate_to(next_tab);
                }

                if on_create_sidebar_playlist {
                    let default_name = format!("New Playlist {}", self.playlists.len() + 1);
                    if let Ok(id) = self.db.create_playlist(&default_name) {
                        self.reload_data();
                        self.playlists_state.selected_playlist_id = Some(id);
                        self.navigate_to(NavTab::Playlist(id));
                        self.set_status(&format!("Playlist '{}' created", default_name));
                    }
                }

                // Center Content Area
                let content_width = if self.show_queue {
                    ui.available_width() - 250.0
                } else {
                    ui.available_width()
                };

                let mut on_trigger_scan = false;
                let mut content_frame = self.theme.card_frame(12.0);
                content_frame.inner_margin = egui::Margin::same(14.0_f32);

                ui.allocate_ui_with_layout(
                    vec2(content_width, workspace_height),
                    Layout::top_down(egui::Align::Min),
                    |ui| {
                        content_frame.show(ui, |ui| {
                            match self.current_tab {
                                NavTab::Home => {
                                    render_home(
                                        ui,
                                        &self.tracks,
                                        &self.recent_history,
                                        &mut self.player,
                                        &self.theme,
                                        &mut on_play_track,
                                        &mut self.art_manager,
                                        &mut on_trigger_scan,
                                    );
                                }
                                NavTab::NowPlaying => {
                                    render_now_playing(ui, &mut self.player, &self.theme, &mut self.art_manager);
                                }
                                NavTab::Songs => {
                                    render_songs(
                                        ui,
                                        &self.tracks,
                                        &self.search_query,
                                        &mut self.songs_state,
                                        &mut self.player,
                                        &self.theme,
                                        &mut on_toggle_fav,
                                    );
                                }
                                NavTab::Albums => {
                                    render_albums(
                                        ui,
                                        &self.tracks,
                                        &self.search_query,
                                        &mut self.albums_state,
                                        &mut self.player,
                                        &self.theme,
                                        &mut self.art_manager,
                                    );
                                }
                                NavTab::Artists => {
                                    render_artists(
                                        ui,
                                        &self.tracks,
                                        &self.search_query,
                                        &mut self.artists_state,
                                        &mut self.player,
                                        &self.theme,
                                    );
                                }
                                NavTab::Genres => {
                                    render_genres(
                                        ui,
                                        &self.tracks,
                                        &self.search_query,
                                        &mut self.genres_state,
                                        &mut self.player,
                                        &self.theme,
                                    );
                                }
                                NavTab::Folders => {
                                    render_folders(
                                        ui,
                                        &self.cfg.music_folders,
                                        &self.tracks,
                                        &mut self.folders_state,
                                        &mut self.player,
                                        &self.theme,
                                        &mut on_add_folder,
                                    );
                                }
                                NavTab::RecentlyPlayed => {
                                    let list = self.db.get_smart_recently_played(50).unwrap_or_else(|_| self.recent_history.clone());
                                    render_songs_view(
                                        ui,
                                        "Recently Played",
                                        "Tracks you've listened to recently",
                                        Icon::Clock,
                                        &list,
                                        &self.search_query,
                                        &mut self.songs_state,
                                        &mut self.player,
                                        &self.theme,
                                        &mut on_toggle_fav,
                                    );
                                }
                                NavTab::MostPlayed => {
                                    let list = self.db.get_smart_most_played(50).unwrap_or_else(|_| {
                                        let mut s = self.tracks.clone();
                                        s.retain(|t| t.play_count > 0);
                                        s.sort_by_key(|a| std::cmp::Reverse(a.play_count));
                                        s
                                    });
                                    render_songs_view(
                                        ui,
                                        "Most Played",
                                        "Your top listened tracks of all time",
                                        Icon::Fire,
                                        &list,
                                        &self.search_query,
                                        &mut self.songs_state,
                                        &mut self.player,
                                        &self.theme,
                                        &mut on_toggle_fav,
                                    );
                                }
                                NavTab::RecentlyAdded => {
                                    let list = self.db.get_smart_recently_added(50).unwrap_or_else(|_| {
                                        let mut s = self.tracks.clone();
                                        s.sort_by_key(|a| std::cmp::Reverse(a.id));
                                        s
                                    });
                                    render_songs_view(
                                        ui,
                                        "Recently Added",
                                        "Fresh additions to your music library",
                                        Icon::Sparkle,
                                        &list,
                                        &self.search_query,
                                        &mut self.songs_state,
                                        &mut self.player,
                                        &self.theme,
                                        &mut on_toggle_fav,
                                    );
                                }
                                NavTab::Favorites => {
                                    let list: Vec<Track> = self.tracks.iter().filter(|t| t.favorite).cloned().collect();
                                    render_songs_view(
                                        ui,
                                        "Favorites",
                                        "Songs you've loved and favorited",
                                        Icon::Heart,
                                        &list,
                                        &self.search_query,
                                        &mut self.songs_state,
                                        &mut self.player,
                                        &self.theme,
                                        &mut on_toggle_fav,
                                    );
                                }
                                NavTab::NeverPlayed => {
                                    let list = self.db.get_smart_never_played(100).unwrap_or_else(|_| {
                                        self.tracks.iter().filter(|t| t.play_count == 0).cloned().collect()
                                    });
                                    render_songs_view(
                                        ui,
                                        "Never Played",
                                        "Unplayed gems waiting to be discovered",
                                        Icon::Hourglass,
                                        &list,
                                        &self.search_query,
                                        &mut self.songs_state,
                                        &mut self.player,
                                        &self.theme,
                                        &mut on_toggle_fav,
                                    );
                                }
                                NavTab::RandomDiscovery => {
                                    let list = self.db.get_smart_random_discovery(50).unwrap_or_else(|_| {
                                        let mut s = self.tracks.clone();
                                        let mut rng = rand::thread_rng();
                                        s.shuffle(&mut rng);
                                        s.truncate(50);
                                        s
                                    });
                                    render_songs_view(
                                        ui,
                                        "Random Discovery",
                                        "A serendipitous mix from across your library",
                                        Icon::Dices,
                                        &list,
                                        &self.search_query,
                                        &mut self.songs_state,
                                        &mut self.player,
                                        &self.theme,
                                        &mut on_toggle_fav,
                                    );
                                }
                                NavTab::Playlists => {
                                    let playlist_tracks = if let Some(pl_id) = self.playlists_state.selected_playlist_id {
                                        self.db.get_playlist_tracks(pl_id).unwrap_or_default()
                                    } else {
                                        Vec::new()
                                    };

                                    render_playlists(
                                        ui,
                                        &self.playlists,
                                        &playlist_tracks,
                                        &mut self.playlists_state,
                                        &mut self.player,
                                        &self.theme,
                                        &mut on_create_playlist,
                                        &mut on_delete_playlist,
                                    );
                                }
                                NavTab::Playlist(id) => {
                                    let playlist_tracks = self.db.get_playlist_tracks(id).unwrap_or_default();
                                    let pl_name = self
                                        .playlists
                                        .iter()
                                        .find(|p| p.id == id)
                                        .map(|p| p.name.clone())
                                        .unwrap_or_else(|| format!("Playlist {}", id));
                                    let pl_desc = format!("{} tracks in this custom playlist", playlist_tracks.len());

                                    render_songs_view(
                                        ui,
                                        &pl_name,
                                        &pl_desc,
                                        Icon::Playlist,
                                        &playlist_tracks,
                                        &self.search_query,
                                        &mut self.songs_state,
                                        &mut self.player,
                                        &self.theme,
                                        &mut on_toggle_fav,
                                    );
                                }
                                NavTab::Settings => {
                                    render_settings(
                                        ui,
                                        &mut self.cfg,
                                        &self.theme,
                                        &mut on_rescan,
                                        &mut on_add_folder,
                                        &mut on_theme_changed,
                                    );
                                    self.player.apply_audio_settings(&self.cfg.audio);
                                }
                                NavTab::About => {
                                    render_about(ui, &self.theme);
                                }
                            }
                        });
                    },
                );

                if on_trigger_scan {
                    self.trigger_scan();
                }

                // Queue Side Drawer (if toggled open)
                if self.show_queue {
                    let queue_frame = self.theme.glass_frame(12.0);

                    let mut play_queue_idx = None;

                    ui.allocate_ui_with_layout(
                        vec2(240.0, workspace_height),
                        Layout::top_down(egui::Align::Min),
                        |ui| {
                            queue_frame.show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    render_icon(ui, Icon::Queue, 15.0, self.theme.accent);
                                    ui.add_space(4.0);
                                    ui.label(RichText::new("Play Queue").size(14.0).color(self.theme.text_primary).strong());
                                    ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                                        if icon_button(ui, Icon::Close, 11.0, self.theme.text_muted).clicked() {
                                            self.show_queue = false;
                                        }
                                    });
                                });

                                ui.add_space(6.0);
                                ui.separator();
                                ui.add_space(6.0);

                                if self.player.queue.is_empty() {
                                    ui.label(RichText::new("Queue is empty").color(self.theme.text_muted).size(12.0));
                                } else {
                                    ScrollArea::vertical()
                                        .auto_shrink([false, false])
                                        .show(ui, |ui| {
                                            ui.spacing_mut().item_spacing = vec2(0.0, 4.0);

                                            for (i, t) in self.player.queue.iter().enumerate() {
                                                let is_current = self.player.current_index == Some(i);
                                                let color = if is_current { self.theme.accent } else { self.theme.text_primary };

                                                ui.horizontal(|ui| {
                                                    let (icon_r, _) = ui.allocate_exact_size(vec2(12.0, 12.0), egui::Sense::hover());
                                                    if is_current {
                                                        let state_icon = if self.player.engine.is_playing() { Icon::Play } else { Icon::Pause };
                                                        paint_icon(ui.painter(), state_icon, icon_r, color);
                                                    } else {
                                                        ui.painter().circle_filled(icon_r.center(), 1.5, self.theme.text_muted);
                                                    }
                                                    let btn = ui.add(
                                                        egui::Button::new(
                                                            RichText::new(t.display_title()).color(color).size(12.0),
                                                        )
                                                        .fill(Color32::TRANSPARENT)
                                                        .stroke(Stroke::NONE),
                                                    );
                                                    if btn.clicked() {
                                                        play_queue_idx = Some(i);
                                                    }
                                                });
                                            }
                                        });
                                }
                            });
                        },
                    );

                    if let Some(i) = play_queue_idx {
                        self.player.play_index(i);
                    }
                }
            });

            // 3. BOTTOM PLAYERBAR DOCKED
            let pb_response = playerbar::render_playerbar(
                ui,
                &mut self.player,
                &self.theme,
                &mut self.show_queue,
                &mut self.cfg.pip.enabled,
                &mut toggle_now_playing,
                &mut on_toggle_fav,
                &mut self.art_manager,
            );
            ctx.data_mut(|d| d.insert_temp(playerbar_id, pb_response.rect.height()));
        });

        // Handle navigation history buttons
        if on_back {
            self.go_back();
        }
        if on_forward {
            self.go_forward();
        }

        // Handle dispatched events
        if on_scan || on_rescan {
            self.trigger_scan();
        }

        if on_add_folder {
            if let Some(path) = rfd::FileDialog::new().pick_folder() {
                let s = path.to_string_lossy().to_string();
                if !self.cfg.music_folders.contains(&s) {
                    self.cfg.music_folders.push(s);
                    let _ = self.cfg.save();
                    self.trigger_scan();
                }
            }
        }

        if on_theme_changed {
            self.theme = Theme::from_name(&self.cfg.theme, Some(self.cfg.accent_color));
            let _ = self.cfg.save();
        }

        if let Some(track_id) = on_toggle_fav {
            if let Ok(new_fav) = self.db.toggle_favorite(track_id) {
                // Update in memory tracks
                if let Some(t) = self.tracks.iter_mut().find(|t| t.id == track_id) {
                    t.favorite = new_fav;
                }
                if self.player.current_track().is_some() {
                    for q in &mut self.player.queue {
                        if q.id == track_id {
                            q.favorite = new_fav;
                        }
                    }
                }
            }
        }

        if let Some(track) = on_play_track {
            if let Some(pos) = self.player.queue.iter().position(|t| t.id == track.id) {
                self.player.play_index(pos);
            } else if let Some(pos) = self.tracks.iter().position(|t| t.id == track.id) {
                self.player.load_queue(self.tracks.clone(), pos);
            } else {
                self.player.load_queue(vec![track], 0);
            }
        }

        if let Some(name) = on_create_playlist {
            if let Ok(id) = self.db.create_playlist(&name) {
                self.reload_data();
                self.playlists_state.selected_playlist_id = Some(id);
                self.navigate_to(NavTab::Playlist(id));
                self.set_status(&format!("Playlist '{}' created", name));
            }
        }

        if let Some(id) = on_delete_playlist {
            let _ = self.db.delete_playlist(id);
            self.reload_data();
            if self.playlists_state.selected_playlist_id == Some(id) {
                self.playlists_state.selected_playlist_id = None;
            }
            if self.current_tab == NavTab::Playlist(id) {
                self.navigate_to(NavTab::Playlists);
            }
        }

        if toggle_now_playing {
            let target = if self.current_tab == NavTab::NowPlaying {
                NavTab::Songs
            } else {
                NavTab::NowPlaying
            };
            self.navigate_to(target);
        }

        // Render Desktop PiP Overlay Viewport Window
        let prev_pip_enabled = self.cfg.pip.enabled;
        if self.cfg.pip.enabled {
            let mut pip_fav = None;
            let pip_settings = self.cfg.pip.clone();
            pip::show_pip_viewport(
                ctx,
                &mut self.player,
                &self.theme,
                &mut self.art_manager,
                &mut self.cfg.pip.enabled,
                &mut pip_fav,
                &pip_settings,
            );
            if let Some(track_id) = pip_fav {
                if let Ok(new_fav) = self.db.toggle_favorite(track_id) {
                    if let Some(t) = self.tracks.iter_mut().find(|t| t.id == track_id) {
                        t.favorite = new_fav;
                    }
                    if self.player.current_track().is_some() {
                        for q in &mut self.player.queue {
                            if q.id == track_id {
                                q.favorite = new_fav;
                            }
                        }
                    }
                }
            }
        }
        if prev_pip_enabled != self.cfg.pip.enabled {
            let _ = self.cfg.save();
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        let _ = self.cfg.save();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_navigation_history_flow() {
        let mut history = vec![NavTab::Home];
        let mut idx = 0;

        let navigate = |history: &mut Vec<NavTab>, idx: &mut usize, tab: NavTab| {
            if *idx + 1 < history.len() {
                history.truncate(*idx + 1);
            }
            history.push(tab);
            *idx = history.len() - 1;
        };

        navigate(&mut history, &mut idx, NavTab::Songs);
        assert_eq!(idx, 1);
        assert_eq!(history, vec![NavTab::Home, NavTab::Songs]);

        navigate(&mut history, &mut idx, NavTab::Artists);
        assert_eq!(idx, 2);
        assert_eq!(history, vec![NavTab::Home, NavTab::Songs, NavTab::Artists]);

        // Go back
        assert!(idx > 0);
        idx -= 1;
        assert_eq!(history[idx], NavTab::Songs);

        // Go back again
        assert!(idx > 0);
        idx -= 1;
        assert_eq!(history[idx], NavTab::Home);

        // Can't go back further
        assert_eq!(idx, 0);

        // Go forward
        assert!(idx + 1 < history.len());
        idx += 1;
        assert_eq!(history[idx], NavTab::Songs);

        // Navigate to new branch from Songs: Albums
        navigate(&mut history, &mut idx, NavTab::Albums);
        assert_eq!(idx, 2);
        assert_eq!(history, vec![NavTab::Home, NavTab::Songs, NavTab::Albums]);
    }

    #[test]
    fn test_smart_navtabs_equality() {
        assert_ne!(NavTab::RecentlyPlayed, NavTab::MostPlayed);
        assert_ne!(NavTab::Favorites, NavTab::NeverPlayed);
        assert_eq!(NavTab::Playlist(1), NavTab::Playlist(1));
        assert_ne!(NavTab::Playlist(1), NavTab::Playlist(2));
    }

    #[test]
    fn test_viewport_api() {
        let _builder = egui::ViewportBuilder::default()
            .with_always_on_top()
            .with_decorations(false)
            .with_transparent(true)
            .with_inner_size(egui::vec2(360.0, 110.0))
            .with_resizable(false);
    }
}
