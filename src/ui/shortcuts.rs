use crate::audio::PlayerController;
use crate::config::RepeatMode;
use crate::library::format_time_secs;
use egui::{Context, Key};
use std::time::Duration;

pub struct ShortcutAction {
    pub status_message: Option<String>,
    pub focus_search: bool,
}

pub fn handle_shortcuts(ctx: &Context, player: &mut PlayerController) -> ShortcutAction {
    // 1. Global shortcuts that work even when an input has focus
    let ctrl_k = ctx.input(|i| (i.modifiers.command || i.modifiers.ctrl) && i.key_pressed(Key::K));

    if ctx.input(|i| i.key_pressed(Key::F11)) {
        let is_full = ctx.input(|i| i.viewport().fullscreen.unwrap_or(false));
        ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(!is_full));
    }

    // If a text input has focus, do NOT trigger playback or navigation shortcuts
    if ctx.wants_keyboard_input() {
        return ShortcutAction {
            status_message: None,
            focus_search: ctrl_k,
        };
    }

    // 2. Playback & navigation shortcuts (only when not typing in a text field)
    if ctx.input(|i| i.key_pressed(Key::Space)) {
        if player.current_track().is_some() {
            player.engine.toggle_play_pause();
            let msg = if player.engine.is_playing() { "Playing" } else { "Paused" };
            return ShortcutAction {
                status_message: Some(msg.to_string()),
                focus_search: ctrl_k,
            };
        } else if !player.queue.is_empty() {
            player.play_index(0);
            return ShortcutAction {
                status_message: Some("Playing queue".to_string()),
                focus_search: ctrl_k,
            };
        }
    }

    if ctx.input(|i| i.key_pressed(Key::ArrowLeft)) {
        let cur = player.engine.position().as_secs_f64();
        let new_pos = (cur - 5.0).max(0.0);
        player.engine.seek(Duration::from_secs_f64(new_pos));
        return ShortcutAction {
            status_message: Some(format!("Seek -5s  [{}]", format_time_secs(new_pos))),
            focus_search: ctrl_k,
        };
    }

    if ctx.input(|i| i.key_pressed(Key::ArrowRight)) {
        let cur = player.engine.position().as_secs_f64();
        let dur = player.current_track().map(|t| t.duration_secs).unwrap_or(0.0);
        let new_pos = (cur + 5.0).min(dur);
        player.engine.seek(Duration::from_secs_f64(new_pos));
        return ShortcutAction {
            status_message: Some(format!("Seek +5s  [{}]", format_time_secs(new_pos))),
            focus_search: ctrl_k,
        };
    }

    if ctx.input(|i| i.key_pressed(Key::ArrowUp)) {
        let new_vol = (player.engine.volume() + 0.05).min(1.0);
        if player.engine.is_muted() {
            player.engine.set_muted(false);
        }
        player.engine.set_volume(new_vol);
        return ShortcutAction {
            status_message: Some(format!("Volume: {}%", (new_vol * 100.0).round() as u32)),
            focus_search: ctrl_k,
        };
    }

    if ctx.input(|i| i.key_pressed(Key::ArrowDown)) {
        let new_vol = (player.engine.volume() - 0.05).max(0.0);
        player.engine.set_volume(new_vol);
        return ShortcutAction {
            status_message: Some(format!("Volume: {}%", (new_vol * 100.0).round() as u32)),
            focus_search: ctrl_k,
        };
    }

    if ctx.input(|i| i.key_pressed(Key::M)) {
        player.engine.toggle_mute();
        let is_muted = player.engine.is_muted();
        let msg = if is_muted {
            "Muted".to_string()
        } else {
            format!("Unmuted ({}%)", (player.engine.volume() * 100.0).round() as u32)
        };
        return ShortcutAction {
            status_message: Some(msg),
            focus_search: ctrl_k,
        };
    }

    if ctx.input(|i| i.key_pressed(Key::S)) {
        player.toggle_shuffle();
        let msg = if player.shuffle { "Shuffle: ON" } else { "Shuffle: OFF" };
        return ShortcutAction {
            status_message: Some(msg.to_string()),
            focus_search: ctrl_k,
        };
    }

    if ctx.input(|i| i.key_pressed(Key::R)) {
        player.cycle_repeat();
        let msg = match player.repeat {
            RepeatMode::Off => "Repeat: OFF",
            RepeatMode::All => "Repeat: ALL",
            RepeatMode::One => "Repeat: ONE",
        };
        return ShortcutAction {
            status_message: Some(msg.to_string()),
            focus_search: ctrl_k,
        };
    }

    ShortcutAction {
        status_message: None,
        focus_search: ctrl_k,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppConfig;
    use egui::{Event, Modifiers, RawInput};

    #[test]
    fn test_shortcuts_toggle_shuffle_and_repeat() {
        let ctx = Context::default();
        let cfg = AppConfig::default();
        let mut player = PlayerController::new(&cfg);

        // Press 'S' to toggle shuffle
        let mut input = RawInput::default();
        input.events.push(Event::Key {
            key: Key::S,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::default(),
        });
        let _ = ctx.run(input, |_| {});

        let action = handle_shortcuts(&ctx, &mut player);
        assert!(player.shuffle, "Shuffle should be toggled ON");
        assert_eq!(action.status_message.as_deref(), Some("Shuffle: ON"));

        // Press 'R' to cycle repeat
        let mut input = RawInput::default();
        input.events.push(Event::Key {
            key: Key::R,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::default(),
        });
        let _ = ctx.run(input, |_| {});

        let action = handle_shortcuts(&ctx, &mut player);
        assert_eq!(player.repeat, RepeatMode::All, "Repeat should cycle to All");
        assert_eq!(action.status_message.as_deref(), Some("Repeat: ALL"));
    }

    #[test]
    fn test_shortcuts_volume_and_mute() {
        let ctx = Context::default();
        let cfg = AppConfig::default();
        let mut player = PlayerController::new(&cfg);
        player.engine.set_volume(0.5);

        // Press Up Arrow
        let mut input = RawInput::default();
        input.events.push(Event::Key {
            key: Key::ArrowUp,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::default(),
        });
        let _ = ctx.run(input, |_| {});

        let action = handle_shortcuts(&ctx, &mut player);
        assert!((player.engine.volume() - 0.55).abs() < 0.001);
        assert_eq!(action.status_message.as_deref(), Some("Volume: 55%"));

        // Press 'M' to mute
        let mut input = RawInput::default();
        input.events.push(Event::Key {
            key: Key::M,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::default(),
        });
        let _ = ctx.run(input, |_| {});

        let action = handle_shortcuts(&ctx, &mut player);
        assert!(player.engine.is_muted());
        assert_eq!(action.status_message.as_deref(), Some("Muted"));
    }

    #[test]
    fn test_ctrl_k_focus_search() {
        let ctx = Context::default();
        let cfg = AppConfig::default();
        let mut player = PlayerController::new(&cfg);

        let mut input = RawInput::default();
        input.modifiers.ctrl = true;
        input.events.push(Event::Key {
            key: Key::K,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers {
                ctrl: true,
                ..Default::default()
            },
        });
        let _ = ctx.run(input, |_| {});

        let action = handle_shortcuts(&ctx, &mut player);
        assert!(action.focus_search, "Ctrl+K must trigger search focus");
    }

    #[test]
    fn test_shortcuts_suppressed_when_typing() {
        let ctx = Context::default();
        let cfg = AppConfig::default();
        let mut player = PlayerController::new(&cfg);

        let mut text = String::new();
        let mut input = RawInput::default();
        input.events.push(Event::Key {
            key: Key::Space,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::default(),
        });

        let _ = ctx.run(input, |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let edit = ui.add(egui::TextEdit::singleline(&mut text));
                edit.request_focus();
            });
        });

        let action = handle_shortcuts(&ctx, &mut player);
        assert!(action.status_message.is_none(), "Space should NOT toggle playback when typing");
    }
}
