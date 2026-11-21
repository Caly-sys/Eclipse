use crate::config::{list_audio_devices, AppConfig};
use crate::streams::credentials::CredentialStore;
use crate::ui::components::{dark_pill_button, pill_badge, white_pill_button};
use crate::ui::icons::{icon_button, paint_icon, render_icon, Icon};
use crate::ui::theme::Theme;
use egui::{
    vec2, Color32, Frame, Layout, Margin, Rect, RichText, Rounding, ScrollArea, Stroke, TextEdit, Ui,
};

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SettingsTab {
    #[default]
    Appearance,
    Audio,
    Equalizer,
    Library,
    Lyrics,
    Streaming,
    Shortcuts,
}

impl SettingsTab {
    pub const ALL: [SettingsTab; 7] = [
        SettingsTab::Appearance,
        SettingsTab::Audio,
        SettingsTab::Equalizer,
        SettingsTab::Library,
        SettingsTab::Lyrics,
        SettingsTab::Streaming,
        SettingsTab::Shortcuts,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            SettingsTab::Appearance => "Appearance",
            SettingsTab::Audio => "Audio",
            SettingsTab::Equalizer => "Equalizer",
            SettingsTab::Library => "Library",
            SettingsTab::Lyrics => "Lyrics",
            SettingsTab::Streaming => "Streaming",
            SettingsTab::Shortcuts => "Shortcuts",
        }
    }
}

pub fn render_micro_label(ui: &mut Ui, text: &str, color: Color32) {
    let font_id = egui::FontId::proportional(11.0);
    let letter_spacing = 1.32_f32; // 0.12em on 11px
    let mut total_width = 0.0_f32;
    let mut galleys = Vec::new();
    for ch in text.chars() {
        let g = ui.painter().layout_no_wrap(ch.to_string(), font_id.clone(), color);
        let w = g.size().x;
        galleys.push((g, w));
        total_width += w + letter_spacing;
    }
    if !galleys.is_empty() {
        total_width -= letter_spacing;
    }
    let (rect, resp) = ui.allocate_exact_size(vec2(total_width, 14.0), egui::Sense::hover());
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Label, true, text));
    if ui.is_rect_visible(rect) {
        let mut x = rect.min.x;
        for (g, w) in galleys {
            let y = rect.center().y - g.size().y / 2.0;
            ui.painter().galley(egui::pos2(x, y), g, color);
            x += w + letter_spacing;
        }
    }
}

pub fn section_header(ui: &mut Ui, label: &str, theme: &Theme) {
    let palette = theme.bento_palette();
    ui.add_space(10.0);
    ui.label(
        RichText::new(label)
            .size(11.0)
            .color(palette.text_muted)
            .strong(),
    );
    ui.add_space(4.0);
}

pub fn pill_toggle_switch(ui: &mut Ui, value: &mut bool, theme: &Theme) -> egui::Response {
    let desired_size = vec2(40.0, 22.0);
    let (rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
    if response.clicked() {
        *value = !*value;
        response.mark_changed();
    }
    if ui.is_rect_visible(rect) {
        let palette = theme.bento_palette();
        let how_on = ui.ctx().animate_bool_responsive(response.id, *value);
        let radius = rect.height() / 2.0;

        // Smooth background interpolation between card surface and vibrant accent
        let bg_off = palette.bg_card;
        let bg_on = palette.accent;
        let r = egui::lerp(bg_off.r() as f32..=bg_on.r() as f32, how_on) as u8;
        let g = egui::lerp(bg_off.g() as f32..=bg_on.g() as f32, how_on) as u8;
        let b = egui::lerp(bg_off.b() as f32..=bg_on.b() as f32, how_on) as u8;
        let a = egui::lerp(bg_off.a() as f32..=bg_on.a() as f32, how_on) as u8;
        let bg = Color32::from_rgba_premultiplied(r, g, b, a);

        ui.painter().rect_filled(rect, Rounding::same(radius), bg);

        // Knob circle
        let knob_radius = radius - 3.0;
        let knob_x = egui::lerp((rect.min.x + radius)..=(rect.max.x - radius), how_on);
        let knob_center = egui::pos2(knob_x, rect.center().y);
        let knob_color = if *value {
            Color32::WHITE
        } else {
            palette.text_secondary
        };

        ui.painter().circle_filled(knob_center, knob_radius, knob_color);
    }

    if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    response
}

pub fn setting_row_card<R>(
    ui: &mut Ui,
    theme: &Theme,
    title: &str,
    subtitle: &str,
    content_fn: impl FnOnce(&mut Ui) -> R,
) -> R {
    let palette = theme.bento_palette();
    let frame = Frame::none()
        .fill(palette.bg_card)
        .stroke(Stroke::NONE)
        .rounding(Rounding::same(8.0))
        .inner_margin(Margin {
            left: 18.0,
            right: 18.0,
            top: 14.0,
            bottom: 14.0,
        });

    frame
        .show(ui, |ui| {
            let total_width = ui.available_width();
            ui.horizontal(|ui| {
                let right_reserved = 420.0_f32.min(total_width * 0.55);
                let text_width = (total_width - right_reserved).max(180.0);

                ui.allocate_ui_with_layout(
                    vec2(text_width, 0.0),
                    Layout::top_down(egui::Align::LEFT),
                    |ui| {
                        ui.label(
                            RichText::new(title)
                                .size(13.5)
                                .strong()
                                .color(palette.text_primary),
                        );
                        if !subtitle.is_empty() {
                            ui.add_space(2.0);
                            ui.add(
                                egui::Label::new(
                                    RichText::new(subtitle)
                                        .size(11.5)
                                        .color(palette.text_secondary),
                                )
                                .wrap(),
                            );
                        }
                    },
                );

                ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                    content_fn(ui)
                })
                .inner
            })
            .inner
        })
        .inner
}

pub fn segmented_pill_group(
    ui: &mut Ui,
    options: &[(&str, &str)],
    current: &mut String,
    theme: &Theme,
) -> bool {
    let mut changed = false;
    let palette = theme.bento_palette();

    Frame::none()
        .fill(palette.bg_sidebar)
        .stroke(Stroke::NONE)
        .rounding(Rounding::same(999.0))
        .inner_margin(Margin::same(2.0))
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing = vec2(2.0, 0.0);
            ui.horizontal(|ui| {
                for (val, label) in options {
                    let is_selected = current.as_str() == *val;
                    let (bg, text_color) = if is_selected {
                        (
                            palette.accent,
                            Color32::from_rgb(16, 18, 22),
                        )
                    } else {
                        (Color32::TRANSPARENT, palette.text_secondary)
                    };

                    let btn_padding = vec2(11.0, 5.0);
                    let font_id = egui::FontId::proportional(11.5);
                    let galley = ui.painter().layout_no_wrap(label.to_string(), font_id, text_color);
                    let desired_size = galley.size() + btn_padding * 2.0;
                    let (rect, mut resp) = ui.allocate_exact_size(desired_size, egui::Sense::click());

                    if resp.clicked() && !is_selected {
                        *current = val.to_string();
                        resp.mark_changed();
                        changed = true;
                    }

                    if ui.is_rect_visible(rect) {
                        let fill = if is_selected {
                            bg
                        } else if resp.hovered() {
                            palette.bg_card_hover
                        } else {
                            Color32::TRANSPARENT
                        };
                        if fill != Color32::TRANSPARENT {
                            ui.painter().rect_filled(rect, Rounding::same(999.0), fill);
                        }
                        let text_pos = egui::pos2(
                            rect.min.x + btn_padding.x,
                            rect.center().y - galley.size().y / 2.0,
                        );
                        ui.painter().galley(text_pos, galley, text_color);
                    }

                    if resp.hovered() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    }
                }
            });
        });

    changed
}

pub fn kbd_badge(ui: &mut Ui, key: &str, theme: &Theme) {
    let palette = theme.bento_palette();
    let parts: Vec<&str> = key.split('+').map(|s| s.trim()).collect();
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = vec2(4.0, 0.0);
        for (i, part) in parts.iter().enumerate() {
            if i > 0 {
                ui.label(RichText::new("+").size(11.0).color(palette.text_muted));
            }
            let font_id = egui::FontId::monospace(11.5);
            let text_color = palette.text_primary;
            let galley = ui.painter().layout_no_wrap(part.to_string(), font_id, text_color);
            let pad = vec2(8.0, 4.0);
            let size = vec2((galley.size().x + pad.x * 2.0).max(24.0), 22.0);
            let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
            if ui.is_rect_visible(rect) {
                ui.painter().rect_filled(rect, Rounding::same(5.0), palette.bg_sidebar);
                let text_pos = egui::pos2(
                    rect.center().x - galley.size().x / 2.0,
                    rect.center().y - galley.size().y / 2.0,
                );
                ui.painter().galley(text_pos, galley, text_color);
            }
        }
    });
}

#[allow(clippy::too_many_arguments)]
pub fn render_settings(
    ui: &mut Ui,
    cfg: &mut AppConfig,
    theme: &Theme,
    on_rescan: &mut bool,
    on_add_folder: &mut bool,
    on_theme_changed: &mut bool,
    db: Option<&crate::library::db::Database>,
    on_refresh_library: Option<&mut bool>,
) {
    let old_cfg = cfg.clone();
    let palette = theme.bento_palette();

    ScrollArea::vertical()
        .id_salt("settings_view_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing = vec2(0.0, 10.0);

            // 1. Header Layout
            // Top micro-label PREFERENCES (11px, uppercase, letter-spacing 0.12em, palette.text_muted)
            render_micro_label(ui, "PREFERENCES", palette.text_muted);
            ui.add_space(1.0);

            // Large Settings heading (28px, bold, palette.text_primary)
            ui.label(
                RichText::new("Settings")
                    .size(28.0)
                    .color(palette.text_primary)
                    .strong(),
            );
            ui.add_space(12.0);

            // 2. Horizontal pill tab bar (7 tabs)
            let tab_id = egui::Id::new("eclipse_settings_active_tab");
            let mut current_tab = ui
                .ctx()
                .data_mut(|d| *d.get_temp_mut_or_insert_with(tab_id, SettingsTab::default));

            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = vec2(6.0, 0.0);
                for tab in SettingsTab::ALL {
                    let is_active = current_tab == tab;
                    let text_color = if is_active {
                        palette.text_primary
                    } else {
                        palette.text_secondary
                    };

                    let font_id = egui::FontId::proportional(12.5);
                    let galley = ui.painter().layout_no_wrap(tab.label().to_string(), font_id, text_color);
                    let pad = vec2(16.0, 7.0);
                    let desired_size = vec2(galley.size().x + pad.x * 2.0, 32.0);
                    let (rect, resp) = ui.allocate_exact_size(desired_size, egui::Sense::click());

                    if resp.clicked() {
                        current_tab = tab;
                        ui.ctx().data_mut(|d| d.insert_temp(tab_id, tab));
                    }

                    if ui.is_rect_visible(rect) {
                        if is_active {
                            ui.painter().rect_filled(rect, Rounding::same(999.0), palette.accent_soft);
                        } else if resp.hovered() {
                            ui.painter().rect_filled(rect, Rounding::same(999.0), palette.bg_card_hover);
                        }

                        let text_pos = egui::pos2(
                            rect.center().x - galley.size().x / 2.0,
                            rect.center().y - galley.size().y / 2.0,
                        );
                        ui.painter().galley(text_pos, galley, text_color);
                    }

                    if resp.hovered() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    }
                }
            });

            ui.add_space(8.0);

            // 3. Tab Contents
            match current_tab {
                SettingsTab::Appearance => {
                    render_appearance_tab(ui, cfg, theme, on_theme_changed);
                }
                SettingsTab::Audio => {
                    render_audio_tab(ui, cfg, theme);
                }
                SettingsTab::Equalizer => {
                    render_equalizer_tab(ui, cfg, theme);
                }
                SettingsTab::Library => {
                    render_library_tab(ui, cfg, theme, on_rescan, on_add_folder);
                }
                SettingsTab::Lyrics => {
                    render_lyrics_tab(ui, cfg, theme);
                }
                SettingsTab::Streaming => {
                    render_streaming_tab(ui, cfg, theme, db, on_refresh_library);
                }
                SettingsTab::Shortcuts => {
                    render_shortcuts_tab(ui, theme);
                }
            }
        });

    // Auto-save and sync top-level fields immediately upon modification
    if *cfg != old_cfg {
        cfg.scan_on_startup = cfg.library.scan_on_startup;
        cfg.music_folders = cfg.library.music_folders.clone();
        cfg.normalize();
        let _ = cfg.save();
    }
}

fn render_appearance_tab(
    ui: &mut Ui,
    cfg: &mut AppConfig,
    theme: &Theme,
    on_theme_changed: &mut bool,
) {
    section_header(ui, "PALETTE & GLASS", theme);

    // 1. Theme selector
    setting_row_card(
        ui,
        theme,
        "Theme",
        "Select base visual palette and glass tone.",
        |ui| {
            let theme_options = [
                ("Eclipse", "Eclipse"),
                ("Obsidian", "Obsidian"),
                ("Glacier", "Glacier"),
                ("Nebula", "Nebula"),
                ("Verdant", "Verdant"),
                ("Ember", "Ember"),
            ];
            let mut cur_theme = cfg.theme.clone();
            if segmented_pill_group(ui, &theme_options, &mut cur_theme, theme) {
                cfg.set_theme(cur_theme);
                *on_theme_changed = true;
            }
        },
    );

    // 2. Accent color swatches
    setting_row_card(
        ui,
        theme,
        "Accent color",
        "Signature glow accent color throughout the UI.",
        |ui| {
            let presets = [
                ("Violet (Signature)", [179, 136, 255]),
                ("Amber (Warm)", [255, 178, 92]),
                ("Cyan", [100, 181, 246]),
                ("Emerald", [105, 240, 174]),
                ("Coral", [255, 82, 82]),
            ];
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = vec2(8.0, 0.0);
                for (name, rgb) in &presets {
                    let c = Color32::from_rgb(rgb[0], rgb[1], rgb[2]);
                    let (rect, resp) = ui.allocate_exact_size(vec2(24.0, 24.0), egui::Sense::click());
                    if resp.clicked() {
                        cfg.set_accent_color(*rgb);
                        *on_theme_changed = true;
                    }
                    if ui.is_rect_visible(rect) {
                        let palette = theme.bento_palette();
                        ui.painter().rect_filled(rect, Rounding::same(12.0), c);
                        if cfg.accent_color == *rgb {
                            ui.painter().rect_stroke(
                                rect.expand(2.0),
                                Rounding::same(14.0),
                                Stroke::new(2.0_f32, palette.text_primary),
                            );
                        } else if resp.hovered() {
                            ui.painter().rect_stroke(
                                rect.expand(1.0),
                                Rounding::same(13.0),
                                Stroke::new(1.0_f32, palette.accent_soft),
                            );
                        }
                    }
                    if resp.hovered() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    }
                    let _ = resp.on_hover_text(*name);
                }
            });
        },
    );

    // 3. Glass opacity
    setting_row_card(
        ui,
        theme,
        "Glass opacity",
        "Frosted background translucency and depth.",
        |ui| {
            ui.horizontal(|ui| {
                let mut opacity = cfg.appearance.glass_opacity;
                if ui
                    .add(
                        egui::Slider::new(&mut opacity, 0.50..=1.00)
                            .step_by(0.05)
                            .show_value(false),
                    )
                    .changed()
                {
                    cfg.set_glass_opacity(opacity);
                }
                ui.add_space(8.0);
                ui.label(
                    RichText::new(format!("{:.0}%", cfg.appearance.glass_opacity * 100.0))
                        .monospace()
                        .size(12.5)
                        .color(theme.text_primary),
                );
            });
        },
    );

    section_header(ui, "LAYOUT & NAVIGATION", theme);

    // Keep playing when closed (System Tray background playback)
    setting_row_card(
        ui,
        theme,
        "Keep playing when closed",
        "Minimize to system tray so music continues playing in the background when the window is closed.",
        |ui| {
            pill_toggle_switch(ui, &mut cfg.appearance.close_to_tray, theme);
        },
    );

    // 4. Compact sidebar
    setting_row_card(
        ui,
        theme,
        "Compact sidebar",
        "Icon-only navigation mode for maximum music canvas space.",
        |ui| {
            pill_toggle_switch(ui, &mut cfg.appearance.compact_sidebar, theme);
        },
    );

    // 5. Show spectrum by default
    setting_row_card(
        ui,
        theme,
        "Show spectrum by default",
        "Always render real-time audio FFT bars on playerbar and cards.",
        |ui| {
            pill_toggle_switch(ui, &mut cfg.appearance.show_spectrum_default, theme);
        },
    );

    // 6. Quit Application
    setting_row_card(
        ui,
        theme,
        "Quit Application",
        "Completely close Eclipse and stop background playback (or press Ctrl+Q).",
        |ui| {
            let palette = theme.bento_palette();
            let btn = egui::Button::new(
                RichText::new("Exit Eclipse")
                    .color(Color32::from_rgb(239, 68, 68))
                    .size(12.0),
            )
            .fill(palette.bg_sidebar)
            .stroke(Stroke::NONE)
            .rounding(Rounding::same(6.0));
            if ui.add(btn).clicked() {
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
            }
        },
    );
}

fn render_audio_tab(ui: &mut Ui, cfg: &mut AppConfig, theme: &Theme) {
    let devices_id = egui::Id::new("eclipse_cached_audio_devices");
    let devices = ui.ctx().data_mut(|d| {
        d.get_temp_mut_or_insert_with(devices_id, list_audio_devices)
            .clone()
    });

    // SECTION 1: OUTPUT
    section_header(ui, "OUTPUT", theme);

    // 1. Output device
    setting_row_card(
        ui,
        theme,
        "Output device",
        "Choose WASAPI devices for the cleanest path on Windows.",
        |ui| {
            let combo_width = (ui.available_width() - 20.0).clamp(240.0, 380.0);
            egui::ComboBox::from_id_salt("settings_audio_output_device")
                .width(combo_width)
                .selected_text(&cfg.audio.output_device)
                .show_ui(ui, |ui| {
                    for dev in &devices {
                        let is_sel = cfg.audio.output_device == *dev;
                        if ui.selectable_label(is_sel, dev).clicked() {
                            cfg.audio.output_device = dev.clone();
                        }
                    }
                });
        },
    );

    // 2. Exclusive mode
    setting_row_card(
        ui,
        theme,
        "Exclusive mode",
        "WASAPI exclusive — bypasses the Windows mixer for bit-perfect output.",
        |ui| {
            pill_toggle_switch(ui, &mut cfg.audio.exclusive_mode, theme);
        },
    );

    // 3. Buffer
    setting_row_card(
        ui,
        theme,
        "Buffer",
        "Lower = snappier, higher = safer on busy systems.",
        |ui| {
            ui.horizontal(|ui| {
                ui.add(
                    egui::Slider::new(&mut cfg.audio.buffer_ms, 20..=500)
                        .step_by(10.0)
                        .show_value(false),
                );
                ui.add_space(8.0);
                ui.label(
                    RichText::new(format!("{:>3} ms", cfg.audio.buffer_ms))
                        .monospace()
                        .size(12.5)
                        .color(theme.text_primary),
                );
            });
        },
    );

    // SECTION 2: TRANSITIONS
    section_header(ui, "TRANSITIONS", theme);

    // 4. Gapless playback
    setting_row_card(
        ui,
        theme,
        "Gapless playback",
        "Seamless joins between consecutive tracks (live albums, DJ mixes).",
        |ui| {
            pill_toggle_switch(ui, &mut cfg.audio.gapless_playback, theme);
        },
    );

    // 5. Crossfade
    setting_row_card(
        ui,
        theme,
        "Crossfade",
        "Equal-power crossfade between tracks. 0 = off.",
        |ui| {
            ui.horizontal(|ui| {
                ui.add(
                    egui::Slider::new(&mut cfg.audio.crossfade_sec, 0.0..=12.0)
                        .step_by(0.5)
                        .show_value(false),
                );
                ui.add_space(8.0);
                ui.label(
                    RichText::new(format!("{:>4.1} s", cfg.audio.crossfade_sec))
                        .monospace()
                        .size(12.5)
                        .color(theme.text_primary),
                );
            });
        },
    );

    // 6. Fade on pause / resume
    setting_row_card(
        ui,
        theme,
        "Fade on pause / resume",
        "Click-free soft fades.",
        |ui| {
            ui.horizontal(|ui| {
                ui.add(
                    egui::Slider::new(&mut cfg.audio.fade_pause_resume_ms, 0..=500)
                        .step_by(10.0)
                        .show_value(false),
                );
                ui.add_space(8.0);
                ui.label(
                    RichText::new(format!("{:>3} ms", cfg.audio.fade_pause_resume_ms))
                        .monospace()
                        .size(12.5)
                        .color(theme.text_primary),
                );
            });
        },
    );

    // SECTION 3: LOUDNESS
    section_header(ui, "LOUDNESS", theme);

    // 7. Volume leveling
    setting_row_card(
        ui,
        theme,
        "Volume leveling",
        "ReplayGain from tags, or Smart: Eclipse measures each track (EBU R128) as you listen.",
        |ui| {
            let options = [
                ("off", "Off"),
                ("rg_track", "RG Track"),
                ("rg_album", "RG Album"),
                ("smart_lufs", "Smart LUFS"),
            ];
            segmented_pill_group(ui, &options, &mut cfg.audio.volume_leveling, theme);
        },
    );

    // 8. Smart target
    setting_row_card(
        ui,
        theme,
        "Smart target",
        "Target loudness level for Smart LUFS normalization.",
        |ui| {
            ui.horizontal(|ui| {
                ui.add(
                    egui::Slider::new(&mut cfg.audio.smart_target_lufs, -24.0..=-9.0)
                        .step_by(1.0)
                        .show_value(false),
                );
                ui.add_space(8.0);
                ui.label(
                    RichText::new(format!("{:>3.0} LUFS", cfg.audio.smart_target_lufs))
                        .monospace()
                        .size(12.5)
                        .color(theme.text_primary),
                );
            });
        },
    );
}

fn render_equalizer_tab(ui: &mut Ui, cfg: &mut AppConfig, theme: &Theme) {
    section_header(ui, "EQUALIZER CONFIGURATION", theme);

    // 1. Master EQ toggle
    setting_row_card(
        ui,
        theme,
        "Master Equalizer",
        "Enable or disable real-time 10-band parametric audio shaping.",
        |ui| {
            pill_toggle_switch(ui, &mut cfg.equalizer.enabled, theme);
        },
    );

    // 2. Preset dropdown
    setting_row_card(
        ui,
        theme,
        "Equalizer preset",
        "Factory tuning presets or custom frequency curves.",
        |ui| {
            egui::ComboBox::from_id_salt("settings_eq_preset")
                .width(180.0)
                .selected_text(&cfg.equalizer.preset)
                .show_ui(ui, |ui| {
                    for preset_name in &["Flat", "Bass Boost", "Vocal", "Harman", "V-Shape", "Custom"] {
                        let is_sel = cfg.equalizer.preset == *preset_name;
                        if ui.selectable_label(is_sel, *preset_name).clicked() {
                            cfg.equalizer.apply_preset(preset_name);
                        }
                    }
                });
        },
    );

    // 3. Preamp slider
    setting_row_card(
        ui,
        theme,
        "Preamp gain",
        "Input gain before frequency processing.",
        |ui| {
            ui.horizontal(|ui| {
                ui.add(
                    egui::Slider::new(&mut cfg.equalizer.preamp_db, -12.0..=12.0)
                        .step_by(0.5)
                        .show_value(false),
                );
                ui.add_space(8.0);
                let sign = if cfg.equalizer.preamp_db > 0.0 { "+" } else { "" };
                ui.label(
                    RichText::new(format!("{}{:.1} dB", sign, cfg.equalizer.preamp_db))
                        .monospace()
                        .size(12.5)
                        .color(theme.text_primary),
                );
            });
        },
    );

    section_header(ui, "FREQUENCY BANDS", theme);

    // 4. 10-Band Graphic Rack Card
    let palette = theme.bento_palette();
    Frame::none()
        .fill(palette.bg_card)
        .stroke(Stroke::NONE)
        .rounding(Rounding::same(8.0))
        .inner_margin(Margin {
            left: 18.0,
            right: 18.0,
            top: 16.0,
            bottom: 16.0,
        })
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("10-Band Graphic Equalizer")
                        .size(13.5)
                        .strong()
                        .color(palette.text_primary),
                );
                ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Reset to Flat").clicked() {
                        cfg.equalizer.apply_preset("Flat");
                    }
                });
            });
            ui.add_space(2.0);
            ui.label(
                RichText::new("Adjust discrete frequency bands from -12.0 dB to +12.0 dB.")
                    .size(11.5)
                    .color(palette.text_secondary),
            );
            ui.add_space(14.0);

            let freqs = [
                "31Hz", "62Hz", "125Hz", "250Hz", "500Hz", "1kHz", "2kHz", "4kHz", "8kHz", "16kHz",
            ];

            ui.horizontal(|ui| {
                let avail_w = ui.available_width();
                let col_w = (avail_w / 10.0).max(36.0);
                for (i, freq) in freqs.iter().enumerate() {
                    ui.allocate_ui_with_layout(
                        vec2(col_w, 160.0),
                        Layout::top_down(egui::Align::Center),
                        |ui| {
                            let val = cfg.equalizer.bands[i];
                            let sign = if val > 0.0 { "+" } else { "" };
                            ui.label(
                                RichText::new(format!("{}{:.1}", sign, val))
                                    .monospace()
                                    .size(10.5)
                                    .color(if val.abs() > 0.01 {
                                        theme.accent
                                    } else {
                                        theme.text_muted
                                    }),
                            );
                            ui.add_space(2.0);
                            let slider_resp = ui.add(
                                egui::Slider::new(&mut cfg.equalizer.bands[i], -12.0..=12.0)
                                    .vertical()
                                    .step_by(0.5)
                                    .show_value(false),
                            );
                            if slider_resp.changed() {
                                cfg.equalizer.preset = "Custom".to_string();
                            }
                            ui.add_space(4.0);
                            ui.label(
                                RichText::new(*freq)
                                    .size(11.0)
                                    .color(theme.text_secondary),
                            );
                        },
                    );
                }
            });
        });
}

fn render_library_tab(
    ui: &mut Ui,
    cfg: &mut AppConfig,
    theme: &Theme,
    on_rescan: &mut bool,
    on_add_folder: &mut bool,
) {
    section_header(ui, "MONITORED FOLDERS", theme);

    // 1. Monitored Folders Card
    let palette = theme.bento_palette();
    Frame::none()
        .fill(palette.bg_card)
        .stroke(Stroke::NONE)
        .rounding(Rounding::same(8.0))
        .inner_margin(Margin {
            left: 18.0,
            right: 18.0,
            top: 14.0,
            bottom: 14.0,
        })
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new("Music folders")
                            .size(13.5)
                            .strong()
                            .color(palette.text_primary),
                    );
                    ui.add_space(2.0);
                    ui.label(
                        RichText::new(
                            "Eclipse scans these directories for lossless and compressed audio.",
                        )
                        .size(11.5)
                        .color(palette.text_secondary),
                    );
                });

                ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                    let (add_rect, add_resp) =
                        ui.allocate_exact_size(vec2(110.0, 28.0), egui::Sense::click());
                    let add_bg = if add_resp.hovered() {
                        palette.bg_card_hover
                    } else {
                        palette.bg_sidebar
                    };
                    ui.painter().rect_filled(add_rect, Rounding::same(6.0), add_bg);
                    let icon_r = Rect::from_center_size(
                        egui::pos2(add_rect.min.x + 14.0, add_rect.center().y),
                        vec2(11.0, 11.0),
                    );
                    paint_icon(ui.painter(), Icon::Plus, icon_r, palette.accent);
                    ui.painter().text(
                        egui::pos2(add_rect.min.x + 26.0, add_rect.center().y),
                        egui::Align2::LEFT_CENTER,
                        "Add Folder",
                        egui::FontId::proportional(11.5),
                        palette.accent,
                    );
                    if add_resp.hovered() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    }
                    if add_resp.clicked() {
                        *on_add_folder = true;
                    }
                });
            });

            ui.add_space(10.0);

            let mut to_remove = None;
            if cfg.music_folders.is_empty() {
                ui.label(
                    RichText::new("No monitored folders configured.")
                        .size(12.0)
                        .color(palette.text_muted)
                        .italics(),
                );
            } else {
                for (i, folder) in cfg.music_folders.iter().enumerate() {
                    ui.horizontal(|ui| {
                        render_icon(ui, Icon::Folder, 14.0, palette.accent);
                        ui.add_space(4.0);
                        ui.label(RichText::new(folder).size(12.5).color(palette.text_primary));

                        ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                            if icon_button(ui, Icon::Close, 11.0, palette.text_muted)
                                .on_hover_text("Remove Folder")
                                .clicked()
                            {
                                to_remove = Some(i);
                            }
                        });
                    });
                    if i + 1 < cfg.music_folders.len() {
                        ui.add_space(4.0);
                    }
                }
            }

            if let Some(idx) = to_remove {
                cfg.music_folders.remove(idx);
                cfg.library.music_folders = cfg.music_folders.clone();
            }

            ui.add_space(12.0);

            let (scan_rect, scan_resp) =
                ui.allocate_exact_size(vec2(190.0, 30.0), egui::Sense::click());
            let scan_bg = if scan_resp.hovered() {
                palette.bg_card_hover
            } else {
                palette.bg_sidebar
            };
            ui.painter()
                .rect_filled(scan_rect, Rounding::same(6.0), scan_bg);
            let scan_icon_r = Rect::from_center_size(
                egui::pos2(scan_rect.min.x + 16.0, scan_rect.center().y),
                vec2(12.0, 12.0),
            );
            paint_icon(ui.painter(), Icon::Refresh, scan_icon_r, palette.accent);
            ui.painter().text(
                egui::pos2(scan_rect.min.x + 30.0, scan_rect.center().y),
                egui::Align2::LEFT_CENTER,
                "Rescan Library Now",
                egui::FontId::proportional(12.0),
                palette.accent,
            );
            if scan_resp.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }
            if scan_resp.clicked() {
                *on_rescan = true;
            }
        });

    section_header(ui, "SCANNING PREFERENCES", theme);

    // 2. Scan on startup
    setting_row_card(
        ui,
        theme,
        "Scan on startup",
        "Automatically index newly added tracks when Eclipse launches.",
        |ui| {
            if pill_toggle_switch(ui, &mut cfg.library.scan_on_startup, theme).changed() {
                cfg.scan_on_startup = cfg.library.scan_on_startup;
            }
        },
    );

    // 3. Ignore short audio clips (< 15s)
    setting_row_card(
        ui,
        theme,
        "Ignore short clips (< 15s)",
        "Filter out ringtones, sound effects, and samples under 15 seconds.",
        |ui| {
            pill_toggle_switch(ui, &mut cfg.library.ignore_short_clips, theme);
        },
    );
}

fn render_lyrics_tab(ui: &mut Ui, cfg: &mut AppConfig, theme: &Theme) {
    section_header(ui, "SYNCHRONIZATION & SOURCES", theme);

    // 1. Auto-fetch online lyrics from LRCLIB
    setting_row_card(
        ui,
        theme,
        "Auto-fetch from LRCLIB",
        "Query the LRCLIB open-source database for synchronized and plain lyrics.",
        |ui| {
            pill_toggle_switch(ui, &mut cfg.lyrics.auto_fetch_lrclib, theme);
        },
    );

    // 2. Save fetched .lrc files locally
    setting_row_card(
        ui,
        theme,
        "Save .lrc locally",
        "Store downloaded synchronized lyrics next to audio files for offline playback.",
        |ui| {
            pill_toggle_switch(ui, &mut cfg.lyrics.save_lrc_locally, theme);
        },
    );

    // 3. Synced lyrics auto-scroll
    setting_row_card(
        ui,
        theme,
        "Auto-scroll lyrics",
        "Automatically center and track the active singing line in Now Playing view.",
        |ui| {
            pill_toggle_switch(ui, &mut cfg.lyrics.synced_autoscroll, theme);
        },
    );

    section_header(ui, "TYPOGRAPHY", theme);

    // 4. Lyrics font size
    setting_row_card(
        ui,
        theme,
        "Lyrics font size",
        "Typographic scale of synchronized lines in the lyric reader.",
        |ui| {
            ui.horizontal(|ui| {
                ui.add(
                    egui::Slider::new(&mut cfg.lyrics.lyrics_font_size, 14.0..=28.0)
                        .step_by(1.0)
                        .show_value(false),
                );
                ui.add_space(8.0);
                ui.label(
                    RichText::new(format!("{:.0} pt", cfg.lyrics.lyrics_font_size))
                        .monospace()
                        .size(12.5)
                        .color(theme.text_primary),
                );
            });
        },
    );

    // 5. Desktop overlay lyrics in PiP mode
    setting_row_card(
        ui,
        theme,
        "Desktop overlay lyrics",
        "Display floating synchronized lyrics above the desktop in Picture-in-Picture mode.",
        |ui| {
            pill_toggle_switch(ui, &mut cfg.pip.show_desktop_lyrics, theme);
        },
    );
}

fn render_streaming_tab(
    ui: &mut Ui,
    cfg: &mut AppConfig,
    theme: &Theme,
    db: Option<&crate::library::db::Database>,
    mut on_refresh_library: Option<&mut bool>,
) {
    let youtube_input_id = egui::Id::new("settings_streaming_youtube_api_key");
    let mut youtube_input: String = ui.data(|d| d.get_temp(youtube_input_id)).unwrap_or_else(|| {
        CredentialStore::get_youtube_api_key().ok().flatten().unwrap_or_default()
    });

    let youtube_msg_id = egui::Id::new("settings_streaming_youtube_status_msg");
    let mut youtube_msg: Option<String> = ui.data(|d| d.get_temp(youtube_msg_id));

    let cache_msg_id = egui::Id::new("settings_streaming_cache_status_msg");
    let mut cache_msg: Option<String> = ui.data(|d| d.get_temp(cache_msg_id));

    let delete_msg_id = egui::Id::new("settings_streaming_delete_status_msg");
    let mut delete_msg: Option<String> = ui.data(|d| d.get_temp(delete_msg_id));

    let fallback_db = if db.is_none() {
        crate::library::db::Database::open().ok()
    } else {
        None
    };
    let active_db = db.or(fallback_db.as_ref());
    let stream_tracks_count = active_db
        .and_then(|d| d.get_stream_tracks_count().ok())
        .unwrap_or(0);

    let has_youtube = CredentialStore::get_youtube_api_key()
        .ok()
        .flatten()
        .map(|s| !s.trim().is_empty())
        .unwrap_or(false);

    section_header(ui, "SIDEBAR INTEGRATIONS", theme);

    // 1. Show Spotify in sidebar
    setting_row_card(
        ui,
        theme,
        "Spotify in sidebar",
        "Display Spotify integration shortcut in the left navigation sidebar.",
        |ui| {
            pill_toggle_switch(ui, &mut cfg.streaming.show_spotify, theme);
        },
    );

    // 2. Show YouTube Music in sidebar
    setting_row_card(
        ui,
        theme,
        "YouTube Music in sidebar",
        "Display YouTube Music integration shortcut in the left navigation sidebar.",
        |ui| {
            pill_toggle_switch(ui, &mut cfg.streaming.show_youtube_music, theme);
        },
    );

    section_header(ui, "YOUTUBE DATA API CONFIGURATION", theme);

    // 3. YouTube API Key
    setting_row_card(
        ui,
        theme,
        "YouTube API Key",
        "Google Cloud YouTube Data API v3 Key for music and playlist queries.",
        |ui| {
            ui.horizontal(|ui| {
                if has_youtube {
                    pill_badge(
                        ui,
                        "CONFIGURED",
                        Color32::from_rgba_unmultiplied(30, 215, 96, 35),
                        Color32::from_rgb(30, 215, 96),
                    );
                } else {
                    pill_badge(
                        ui,
                        "NOT CONFIGURED",
                        Color32::from_rgba_unmultiplied(239, 68, 68, 35),
                        Color32::from_rgb(239, 68, 68),
                    );
                }

                ui.add_space(6.0);
                ui.add(
                    TextEdit::singleline(&mut youtube_input)
                        .password(true)
                        .hint_text("Enter API Key...")
                        .desired_width(170.0),
                );

                if white_pill_button(ui, "Save", None).clicked() {
                    let trimmed = youtube_input.trim();
                    if !trimmed.is_empty() {
                        let _ = CredentialStore::save_youtube_api_key(trimmed);
                        youtube_msg = Some("YouTube API key saved to Windows vault.".to_string());
                    }
                }

                if dark_pill_button(ui, "Clear", None).clicked() {
                    let _ = CredentialStore::clear_youtube_credentials();
                    youtube_input.clear();
                    youtube_msg = Some("YouTube API key cleared.".to_string());
                }
            });
        },
    );

    if let Some(ref msg) = youtube_msg {
        ui.add_space(2.0);
        ui.label(RichText::new(msg).size(11.0).color(theme.accent));
    }

    section_header(ui, "STORAGE & STREAM CACHE", theme);

    let cache_size = crate::streams::downloader::get_stream_cache_size();
    let mut cleared_freed = None;

    setting_row_card(
        ui,
        theme,
        "Audio stream cache",
        "Temporary audio stream cache for instant zero-latency playback. Automatically pruned to 500 MB.",
        |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(format!(
                        "{:.1} MB / 500 MB limit",
                        cache_size as f64 / (1024.0 * 1024.0)
                    ))
                    .size(12.5)
                    .color(theme.text_secondary),
                );
                ui.add_space(8.0);

                if dark_pill_button(ui, "Clear Stream Cache", None).clicked() {
                    cleared_freed = Some(crate::streams::downloader::clear_stream_cache());
                }
            });
        },
    );

    if let Some(res) = cleared_freed {
        match res {
            Ok(freed) => {
                let freed_mb = freed as f64 / (1024.0 * 1024.0);
                cache_msg = Some(format!("Stream cache cleared ({freed_mb:.1} MB freed)"));
                ui.ctx().request_repaint();
            }
            Err(e) => {
                cache_msg = Some(format!("Failed to clear stream cache: {e}"));
            }
        }
    }

    if let Some(ref msg) = cache_msg {
        ui.add_space(2.0);
        ui.label(RichText::new(msg).size(11.0).color(theme.accent));
    }

    setting_row_card(
        ui,
        theme,
        "Remove stream songs",
        "Delete all streaming/imported tracks from the library so only local audio files remain.",
        |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(format!("{stream_tracks_count} stream tracks in library"))
                        .size(12.5)
                        .color(theme.text_secondary),
                );
                ui.add_space(8.0);

                if dark_pill_button(ui, "Delete All Stream Songs", Some(Icon::Trash)).clicked() {
                    if let Some(d) = active_db {
                        match d.delete_all_stream_tracks() {
                            Ok(deleted) => {
                                delete_msg = Some(format!("Removed {deleted} stream tracks from library."));
                                if let Some(ref mut refresh) = on_refresh_library {
                                    **refresh = true;
                                }
                                ui.ctx().request_repaint();
                            }
                            Err(e) => {
                                delete_msg = Some(format!("Failed to delete stream tracks: {e}"));
                            }
                        }
                    }
                }
            });
        },
    );

    if let Some(ref msg) = delete_msg {
        ui.add_space(2.0);
        ui.label(RichText::new(msg).size(11.0).color(theme.accent));
    }

    section_header(ui, "EXPERIMENTAL", theme);

    // 4. Enable experimental streaming
    setting_row_card(
        ui,
        theme,
        "Experimental streaming",
        "Enable early preview streaming endpoints and remote playlist sync.",
        |ui| {
            pill_toggle_switch(ui, &mut cfg.streaming.enable_streaming, theme);
        },
    );

    ui.data_mut(|d| {
        d.insert_temp(youtube_input_id, youtube_input);
        d.insert_temp(youtube_msg_id, youtube_msg);
        d.insert_temp(cache_msg_id, cache_msg);
        d.insert_temp(delete_msg_id, delete_msg);
    });
}

fn render_shortcuts_tab(ui: &mut Ui, theme: &Theme) {
    section_header(ui, "GLOBAL KEYBOARD SHORTCUTS", theme);

    let shortcuts = [
        ("Play / Pause", "Toggle track playback or resume paused track", "Space"),
        ("Next Track", "Skip forward to next audio track in play queue", "Ctrl + Right"),
        ("Previous Track", "Return to previous track or track start", "Ctrl + Left"),
        ("Volume Up / Down", "Increase or decrease audio output master volume", "Ctrl + Up / Down"),
        ("Mute Audio", "Instant master audio output mute toggle", "M"),
        ("Toggle Shuffle", "Randomize queue playback order", "S"),
        ("Cycle Repeat", "Switch between Off, Repeat All, and Repeat One", "R"),
        ("Focus Search", "Instantly highlight top global search bar", "Ctrl + K"),
        ("Toggle Sidebar", "Collapse or expand left navigation sidebar", "Ctrl + B"),
        ("Toggle Fullscreen", "Enter or exit frameless borderless fullscreen", "F11"),
        ("Toggle Lyrics", "Open or close synchronized lyrics drawer", "L"),
    ];

    for (action, desc, keys) in shortcuts {
        setting_row_card(ui, theme, action, desc, |ui| {
            kbd_badge(ui, keys, theme);
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_tab_all_count_and_labels() {
        assert_eq!(SettingsTab::ALL.len(), 7);
        assert_eq!(SettingsTab::ALL[0].label(), "Appearance");
        assert_eq!(SettingsTab::ALL[1].label(), "Audio");
        assert_eq!(SettingsTab::ALL[2].label(), "Equalizer");
        assert_eq!(SettingsTab::ALL[3].label(), "Library");
        assert_eq!(SettingsTab::ALL[4].label(), "Lyrics");
        assert_eq!(SettingsTab::ALL[5].label(), "Streaming");
        assert_eq!(SettingsTab::ALL[6].label(), "Shortcuts");
    }

    #[test]
    fn test_pill_toggle_switch_logic() {
        let ctx = egui::Context::default();
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let theme = Theme::from_name("Eclipse", None);
                let mut val = false;
                let _resp = pill_toggle_switch(ui, &mut val, &theme);
                assert!(!val);
            });
        });
    }

    #[test]
    fn test_segmented_pill_group_selection() {
        let ctx = egui::Context::default();
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let theme = Theme::from_name("Eclipse", None);
                let options = [
                    ("off", "Off"),
                    ("rg_track", "RG Track"),
                    ("rg_album", "RG Album"),
                    ("smart_lufs", "Smart LUFS"),
                ];
                let mut current = "off".to_string();
                let _changed = segmented_pill_group(ui, &options, &mut current, &theme);
                assert_eq!(current, "off");
            });
        });
    }

    #[test]
    fn test_render_settings_smoke_render() {
        let ctx = egui::Context::default();
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let mut cfg = AppConfig::default();
                let theme = Theme::from_name("Eclipse", None);
                let mut on_rescan = false;
                let mut on_add_folder = false;
                let mut on_theme_changed = false;

                render_settings(
                    ui,
                    &mut cfg,
                    &theme,
                    &mut on_rescan,
                    &mut on_add_folder,
                    &mut on_theme_changed,
                    None,
                    None,
                );
            });
        });
    }

    #[test]
    fn test_render_all_individual_tabs() {
        let ctx = egui::Context::default();
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let theme = Theme::from_name("Eclipse", None);
                let tab_id = egui::Id::new("eclipse_settings_active_tab");

                for tab in SettingsTab::ALL {
                    ui.ctx().data_mut(|d| d.insert_temp(tab_id, tab));
                    let mut cfg = AppConfig::default();
                    let mut on_rescan = false;
                    let mut on_add_folder = false;
                    let mut on_theme_changed = false;

                    render_settings(
                        ui,
                        &mut cfg,
                        &theme,
                        &mut on_rescan,
                        &mut on_add_folder,
                        &mut on_theme_changed,
                        None,
                        None,
                    );
                }
            });
        });
    }

    #[test]
    fn test_render_micro_label_and_section_headers() {
        let ctx = egui::Context::default();
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let theme = Theme::from_name("Eclipse", None);
                render_micro_label(ui, "PREFERENCES", theme.text_muted);
                section_header(ui, "OUTPUT", &theme);
                section_header(ui, "TRANSITIONS", &theme);
                section_header(ui, "LOUDNESS", &theme);
            });
        });
    }

    #[test]
    fn test_shortcuts_list_valid() {
        let ctx = egui::Context::default();
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let theme = Theme::from_name("Eclipse", None);
                render_shortcuts_tab(ui, &theme);
            });
        });
    }

    #[test]
    fn test_render_streaming_tab_with_stream_cache_section() {
        let ctx = egui::Context::default();
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let mut cfg = AppConfig::default();
                let theme = Theme::from_name("Eclipse", None);
                render_streaming_tab(ui, &mut cfg, &theme, None, None);
            });
        });
    }

    #[test]
    fn test_render_streaming_tab_delete_stream_songs_flow() {
        let db = crate::library::db::Database::open_in_memory().expect("open memory db");
        let stream_track = crate::library::model::Track {
            id: 0,
            path: "stream:spotify:flow_test_1".to_string(),
            title: "Test Stream".to_string(),
            artist: "Streamer".to_string(),
            album_artist: "Streamer".to_string(),
            album: "Single".to_string(),
            genre: "Electronic".to_string(),
            year: 2024,
            track_number: 1,
            duration_secs: 180.0,
            bitrate_kbps: 320,
            sample_rate: 44100,
            channels: 2,
            format: "STREAM".to_string(),
            file_size: 0,
            play_count: 0,
            favorite: false,
            has_art: false,
            folder: String::new(),
        };
        db.upsert_track(&stream_track).expect("insert stream track");
        assert_eq!(db.get_stream_tracks_count().unwrap(), 1);

        let ctx = egui::Context::default();
        let mut refreshed = false;
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let mut cfg = AppConfig::default();
                let theme = Theme::from_name("Eclipse", None);
                render_streaming_tab(ui, &mut cfg, &theme, Some(&db), Some(&mut refreshed));
            });
        });
    }

    #[test]
    fn test_setting_row_card_and_segmented_pill_group_styling() {
        let ctx = egui::Context::default();
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let theme = Theme::from_name("Eclipse", None);
                let palette = theme.bento_palette();

                let val = setting_row_card(ui, &theme, "Title", "Subtitle", |_ui| 42);
                assert_eq!(val, 42);

                let options = [("a", "Alpha"), ("b", "Beta")];
                let mut current = "a".to_string();
                let changed = segmented_pill_group(ui, &options, &mut current, &theme);
                assert!(!changed);
                assert_eq!(current, "a");
                assert_eq!(palette.bg_card, Color32::from_rgb(37, 42, 52));
            });
        });
    }
}
