use egui::{Context, CursorIcon, Pos2, Rect, ResizeDirection, ViewportCommand};

/// Margin in pixels along each edge of the frameless window used for resize hit-testing.
pub const RESIZE_MARGIN: f32 = 7.0;

/// Detects if a pointer position is within the window resize margin,
/// returning the appropriate ResizeDirection and CursorIcon.
/// Corners take priority over edges to allow diagonal resizing.
pub fn detect_resize_direction(
    screen_rect: Rect,
    pos: Pos2,
    margin: f32,
) -> Option<(ResizeDirection, CursorIcon)> {
    if !screen_rect.expand(2.0).contains(pos) {
        return None;
    }

    // Protect window titlebar control buttons (Minimize, Maximize, Close in top-right)
    if pos.x >= screen_rect.max.x - 120.0 && pos.y > 2.0 && pos.y <= 36.0 {
        return None;
    }

    let on_left = pos.x <= screen_rect.min.x + margin;
    let on_right = pos.x >= screen_rect.max.x - margin;
    let on_top = pos.y <= screen_rect.min.y + margin;
    let on_bottom = pos.y >= screen_rect.max.y - margin;

    if on_top && on_left {
        Some((ResizeDirection::NorthWest, CursorIcon::ResizeNorthWest))
    } else if on_top && on_right {
        Some((ResizeDirection::NorthEast, CursorIcon::ResizeNorthEast))
    } else if on_bottom && on_left {
        Some((ResizeDirection::SouthWest, CursorIcon::ResizeSouthWest))
    } else if on_bottom && on_right {
        Some((ResizeDirection::SouthEast, CursorIcon::ResizeSouthEast))
    } else if on_top {
        Some((ResizeDirection::North, CursorIcon::ResizeNorth))
    } else if on_bottom {
        Some((ResizeDirection::South, CursorIcon::ResizeSouth))
    } else if on_left {
        Some((ResizeDirection::West, CursorIcon::ResizeWest))
    } else if on_right {
        Some((ResizeDirection::East, CursorIcon::ResizeEast))
    } else {
        None
    }
}

/// Checks the pointer position against the window perimeter and handles cursor
/// updates and native window resize initiation for the frameless window.
pub fn handle_window_resize(ctx: &Context) {
    let is_maximized = ctx.input(|i| i.viewport().maximized.unwrap_or(false));
    let is_fullscreen = ctx.input(|i| i.viewport().fullscreen.unwrap_or(false));
    if is_maximized || is_fullscreen {
        return;
    }

    // Skip resize handling if an active egui widget has pointer focus or is being hovered/dragged
    if ctx.wants_pointer_input() {
        return;
    }

    let screen_rect = ctx.screen_rect();
    if let Some(pos) = ctx.input(|i| i.pointer.latest_pos()) {
        // Playerbar height docked at the bottom of the window
        let playerbar_h = 72.0;
        let is_over_playerbar = pos.y >= screen_rect.max.y - playerbar_h;
        // Protect the playerbar center controls (seekbar and transport buttons) from South edge resize
        let center_margin = (screen_rect.width() * 0.2).clamp(120.0, 240.0);
        let is_playerbar_center = pos.x > screen_rect.min.x + center_margin && pos.x < screen_rect.max.x - center_margin;

        // Skip resize handling entirely if pointer is over the playerbar center
        if is_over_playerbar && is_playerbar_center {
            return;
        }

        if let Some((direction, cursor)) = detect_resize_direction(screen_rect, pos, RESIZE_MARGIN) {
            // South edge resize is skipped if within the center zone
            if direction == ResizeDirection::South && is_playerbar_center {
                return;
            }

            ctx.set_cursor_icon(cursor);

            if ctx.input(|i| i.pointer.primary_down() && (i.pointer.is_decidedly_dragging() || i.pointer.primary_pressed())) {
                ctx.send_viewport_cmd(ViewportCommand::BeginResize(direction));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{pos2, vec2, Rect};

    #[test]
    fn test_resize_corner_detection() {
        let screen = Rect::from_min_size(pos2(0.0, 0.0), vec2(1000.0, 700.0));
        let margin = RESIZE_MARGIN;

        // Top-left
        let (dir, cur) = detect_resize_direction(screen, pos2(2.0, 2.0), margin).unwrap();
        assert_eq!(dir, ResizeDirection::NorthWest);
        assert_eq!(cur, CursorIcon::ResizeNorthWest);

        // Top-right
        let (dir, cur) = detect_resize_direction(screen, pos2(998.0, 2.0), margin).unwrap();
        assert_eq!(dir, ResizeDirection::NorthEast);
        assert_eq!(cur, CursorIcon::ResizeNorthEast);

        // Bottom-left
        let (dir, cur) = detect_resize_direction(screen, pos2(2.0, 698.0), margin).unwrap();
        assert_eq!(dir, ResizeDirection::SouthWest);
        assert_eq!(cur, CursorIcon::ResizeSouthWest);

        // Bottom-right
        let (dir, cur) = detect_resize_direction(screen, pos2(998.0, 698.0), margin).unwrap();
        assert_eq!(dir, ResizeDirection::SouthEast);
        assert_eq!(cur, CursorIcon::ResizeSouthEast);
    }

    #[test]
    fn test_resize_edge_detection() {
        let screen = Rect::from_min_size(pos2(0.0, 0.0), vec2(1000.0, 700.0));
        let margin = RESIZE_MARGIN;

        // Top edge (centered horizontally)
        let (dir, cur) = detect_resize_direction(screen, pos2(500.0, 3.0), margin).unwrap();
        assert_eq!(dir, ResizeDirection::North);
        assert_eq!(cur, CursorIcon::ResizeNorth);

        // Bottom edge
        let (dir, cur) = detect_resize_direction(screen, pos2(500.0, 697.0), margin).unwrap();
        assert_eq!(dir, ResizeDirection::South);
        assert_eq!(cur, CursorIcon::ResizeSouth);

        // Left edge
        let (dir, cur) = detect_resize_direction(screen, pos2(3.0, 350.0), margin).unwrap();
        assert_eq!(dir, ResizeDirection::West);
        assert_eq!(cur, CursorIcon::ResizeWest);

        // Right edge
        let (dir, cur) = detect_resize_direction(screen, pos2(997.0, 350.0), margin).unwrap();
        assert_eq!(dir, ResizeDirection::East);
        assert_eq!(cur, CursorIcon::ResizeEast);
    }

    #[test]
    fn test_interior_points_return_none() {
        let screen = Rect::from_min_size(pos2(0.0, 0.0), vec2(1000.0, 700.0));
        let margin = RESIZE_MARGIN;

        assert!(detect_resize_direction(screen, pos2(50.0, 50.0), margin).is_none());
        assert!(detect_resize_direction(screen, pos2(500.0, 350.0), margin).is_none());
        assert!(detect_resize_direction(screen, pos2(950.0, 650.0), margin).is_none());
    }

    #[test]
    fn test_outside_window_returns_none() {
        let screen = Rect::from_min_size(pos2(0.0, 0.0), vec2(1000.0, 700.0));
        let margin = RESIZE_MARGIN;

        assert!(detect_resize_direction(screen, pos2(-10.0, 300.0), margin).is_none());
        assert!(detect_resize_direction(screen, pos2(1020.0, 300.0), margin).is_none());
        assert!(detect_resize_direction(screen, pos2(500.0, -15.0), margin).is_none());
        assert!(detect_resize_direction(screen, pos2(500.0, 720.0), margin).is_none());
    }

    #[test]
    fn test_handle_window_resize_sets_cursor_and_command() {
        let ctx = Context::default();
        let mut raw_input = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(1000.0, 700.0))),
            ..Default::default()
        };
        raw_input.events.push(egui::Event::PointerMoved(pos2(3.0, 350.0))); // West edge
        raw_input.events.push(egui::Event::PointerButton {
            pos: pos2(3.0, 350.0),
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: Default::default(),
        });

        let full_output = ctx.run(raw_input, |ctx| {
            handle_window_resize(ctx);
        });

        assert_eq!(full_output.platform_output.cursor_icon, CursorIcon::ResizeWest);

        let has_resize_cmd = full_output.viewport_output.values().any(|vp| {
            vp.commands.iter().any(|cmd| matches!(cmd, ViewportCommand::BeginResize(ResizeDirection::West)))
        });
        assert!(has_resize_cmd, "Must send BeginResize(West) on left border click");
    }

    #[test]
    fn test_handle_window_resize_disabled_when_maximized() {
        let ctx = Context::default();
        let mut raw_input = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(1000.0, 700.0))),
            ..Default::default()
        };
        raw_input.viewports.entry(egui::ViewportId::ROOT).or_default().maximized = Some(true);
        raw_input.events.push(egui::Event::PointerMoved(pos2(3.0, 350.0)));
        raw_input.events.push(egui::Event::PointerButton {
            pos: pos2(3.0, 350.0),
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: Default::default(),
        });

        let full_output = ctx.run(raw_input, |ctx| {
            handle_window_resize(ctx);
        });

        assert_ne!(full_output.platform_output.cursor_icon, CursorIcon::ResizeWest);

        let has_resize_cmd = full_output.viewport_output.values().any(|vp| {
            vp.commands.iter().any(|cmd| matches!(cmd, ViewportCommand::BeginResize(_)))
        });
        assert!(!has_resize_cmd, "Must NOT send BeginResize when window is maximized");
    }

    #[test]
    fn test_handle_window_resize_disabled_when_fullscreen() {
        let ctx = Context::default();
        let mut raw_input = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(1000.0, 700.0))),
            ..Default::default()
        };
        raw_input.viewports.entry(egui::ViewportId::ROOT).or_default().fullscreen = Some(true);
        raw_input.events.push(egui::Event::PointerMoved(pos2(3.0, 350.0)));
        raw_input.events.push(egui::Event::PointerButton {
            pos: pos2(3.0, 350.0),
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: Default::default(),
        });

        let full_output = ctx.run(raw_input, |ctx| {
            handle_window_resize(ctx);
        });

        assert_ne!(full_output.platform_output.cursor_icon, CursorIcon::ResizeWest);

        let has_resize_cmd = full_output.viewport_output.values().any(|vp| {
            vp.commands.iter().any(|cmd| matches!(cmd, ViewportCommand::BeginResize(_)))
        });
        assert!(!has_resize_cmd, "Must NOT send BeginResize when window is fullscreen");
    }

    #[test]
    fn test_handle_window_resize_playerbar_center_exempt() {
        let ctx = Context::default();
        let mut raw_input = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(1000.0, 700.0))),
            ..Default::default()
        };
        // Pointer right in the center of the bottom border where the seekbar is located
        raw_input.events.push(egui::Event::PointerMoved(pos2(500.0, 698.0)));
        raw_input.events.push(egui::Event::PointerButton {
            pos: pos2(500.0, 698.0),
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: Default::default(),
        });

        let full_output = ctx.run(raw_input, |ctx| {
            handle_window_resize(ctx);
        });

        assert_ne!(
            full_output.platform_output.cursor_icon,
            CursorIcon::ResizeSouth,
            "Must NOT set ResizeSouth cursor over playerbar center"
        );

        let has_resize_cmd = full_output.viewport_output.values().any(|vp| {
            vp.commands.iter().any(|cmd| matches!(cmd, ViewportCommand::BeginResize(_)))
        });
        assert!(!has_resize_cmd, "Must NOT send BeginResize when clicking in playerbar center");
    }

    #[test]
    fn test_handle_window_resize_bottom_outside_center_resizes() {
        let ctx = Context::default();
        let mut raw_input = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(1000.0, 700.0))),
            ..Default::default()
        };
        // Pointer at bottom border, but outside the playerbar center (e.g. x = 50.0, margin is 7.0)
        // 50.0 is outside corner (x <= 7.0) and outside center (x > 200.0)
        raw_input.events.push(egui::Event::PointerMoved(pos2(50.0, 698.0)));
        raw_input.events.push(egui::Event::PointerButton {
            pos: pos2(50.0, 698.0),
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: Default::default(),
        });

        let full_output = ctx.run(raw_input, |ctx| {
            handle_window_resize(ctx);
        });

        assert_eq!(
            full_output.platform_output.cursor_icon,
            CursorIcon::ResizeSouth,
            "Must set ResizeSouth cursor on bottom border outside playerbar center"
        );

        let has_resize_cmd = full_output.viewport_output.values().any(|vp| {
            vp.commands.iter().any(|cmd| matches!(cmd, ViewportCommand::BeginResize(ResizeDirection::South)))
        });
        assert!(has_resize_cmd, "Must send BeginResize(South) on bottom border outside playerbar center");
    }

    #[test]
    fn test_handle_window_resize_corners_work() {
        let ctx = Context::default();
        let mut raw_input = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(1000.0, 700.0))),
            ..Default::default()
        };
        // Bottom-left corner (x = 2.0, y = 698.0)
        raw_input.events.push(egui::Event::PointerMoved(pos2(2.0, 698.0)));
        raw_input.events.push(egui::Event::PointerButton {
            pos: pos2(2.0, 698.0),
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: Default::default(),
        });

        let full_output = ctx.run(raw_input, |ctx| {
            handle_window_resize(ctx);
        });

        assert_eq!(full_output.platform_output.cursor_icon, CursorIcon::ResizeSouthWest);
        let has_resize_cmd = full_output.viewport_output.values().any(|vp| {
            vp.commands.iter().any(|cmd| matches!(cmd, ViewportCommand::BeginResize(ResizeDirection::SouthWest)))
        });
        assert!(has_resize_cmd, "Must send BeginResize(SouthWest) on bottom-left corner");
    }

    #[test]
    fn test_handle_window_resize_suppressed_when_wants_pointer_input() {
        let ctx = Context::default();
        let mut raw_input1 = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(1000.0, 700.0))),
            ..Default::default()
        };
        raw_input1.events.push(egui::Event::PointerMoved(pos2(3.0, 350.0)));

        // Frame 1: register the widget under the pointer
        let _ = ctx.run(raw_input1, |ctx| {
            egui::Area::new(egui::Id::new("overlay_btn"))
                .fixed_pos(pos2(0.0, 300.0))
                .show(ctx, |ui| {
                    let _ = ui.allocate_exact_size(vec2(50.0, 100.0), egui::Sense::click_and_drag());
                });
        });

        // Frame 2: mouse click on the widget
        let mut raw_input2 = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(1000.0, 700.0))),
            ..Default::default()
        };
        raw_input2.events.push(egui::Event::PointerButton {
            pos: pos2(3.0, 350.0),
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: Default::default(),
        });

        let full_output = ctx.run(raw_input2, |ctx| {
            egui::Area::new(egui::Id::new("overlay_btn"))
                .fixed_pos(pos2(0.0, 300.0))
                .show(ctx, |ui| {
                    let _ = ui.allocate_exact_size(vec2(50.0, 100.0), egui::Sense::click_and_drag());
                });
            handle_window_resize(ctx);
        });

        // When a widget occupies the pointer, resize must be skipped
        let has_resize_cmd = full_output.viewport_output.values().any(|vp| {
            vp.commands.iter().any(|cmd| matches!(cmd, ViewportCommand::BeginResize(_)))
        });
        assert!(!has_resize_cmd, "Must NOT send BeginResize when ctx wants pointer input");
    }

    #[test]
    fn test_handle_window_resize_all_corners() {
        let ctx = Context::default();
        let screen = Rect::from_min_size(pos2(0.0, 0.0), vec2(1000.0, 700.0));

        // 1. NorthWest (2.0, 2.0)
        let mut raw_input = egui::RawInput {
            screen_rect: Some(screen),
            ..Default::default()
        };
        raw_input.events.push(egui::Event::PointerMoved(pos2(2.0, 2.0)));
        raw_input.events.push(egui::Event::PointerButton {
            pos: pos2(2.0, 2.0),
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: Default::default(),
        });
        let out = ctx.run(raw_input, handle_window_resize);
        assert_eq!(out.platform_output.cursor_icon, CursorIcon::ResizeNorthWest);
        assert!(out.viewport_output.values().any(|vp| vp.commands.iter().any(|cmd| matches!(cmd, ViewportCommand::BeginResize(ResizeDirection::NorthWest)))));

        // 2. NorthEast top-edge corner (998.0, 1.0)
        let mut raw_input = egui::RawInput {
            screen_rect: Some(screen),
            ..Default::default()
        };
        raw_input.events.push(egui::Event::PointerMoved(pos2(998.0, 1.0)));
        raw_input.events.push(egui::Event::PointerButton {
            pos: pos2(998.0, 1.0),
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: Default::default(),
        });
        let out = ctx.run(raw_input, handle_window_resize);
        assert_eq!(out.platform_output.cursor_icon, CursorIcon::ResizeNorthEast);
        assert!(out.viewport_output.values().any(|vp| vp.commands.iter().any(|cmd| matches!(cmd, ViewportCommand::BeginResize(ResizeDirection::NorthEast)))));

        // 3. SouthEast corner (998.0, 698.0)
        let mut raw_input = egui::RawInput {
            screen_rect: Some(screen),
            ..Default::default()
        };
        raw_input.events.push(egui::Event::PointerMoved(pos2(998.0, 698.0)));
        raw_input.events.push(egui::Event::PointerButton {
            pos: pos2(998.0, 698.0),
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: Default::default(),
        });
        let out = ctx.run(raw_input, handle_window_resize);
        assert_eq!(out.platform_output.cursor_icon, CursorIcon::ResizeSouthEast);
        assert!(out.viewport_output.values().any(|vp| vp.commands.iter().any(|cmd| matches!(cmd, ViewportCommand::BeginResize(ResizeDirection::SouthEast)))));
    }

    #[test]
    fn test_handle_window_resize_all_edges() {
        let ctx = Context::default();
        let screen = Rect::from_min_size(pos2(0.0, 0.0), vec2(1000.0, 700.0));

        // 1. North Edge (500.0, 2.0)
        let mut raw_input = egui::RawInput {
            screen_rect: Some(screen),
            ..Default::default()
        };
        raw_input.events.push(egui::Event::PointerMoved(pos2(500.0, 2.0)));
        raw_input.events.push(egui::Event::PointerButton {
            pos: pos2(500.0, 2.0),
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: Default::default(),
        });
        let out = ctx.run(raw_input, handle_window_resize);
        assert_eq!(out.platform_output.cursor_icon, CursorIcon::ResizeNorth);
        assert!(out.viewport_output.values().any(|vp| vp.commands.iter().any(|cmd| matches!(cmd, ViewportCommand::BeginResize(ResizeDirection::North)))));

        // 2. East Edge (998.0, 350.0)
        let mut raw_input = egui::RawInput {
            screen_rect: Some(screen),
            ..Default::default()
        };
        raw_input.events.push(egui::Event::PointerMoved(pos2(998.0, 350.0)));
        raw_input.events.push(egui::Event::PointerButton {
            pos: pos2(998.0, 350.0),
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: Default::default(),
        });
        let out = ctx.run(raw_input, handle_window_resize);
        assert_eq!(out.platform_output.cursor_icon, CursorIcon::ResizeEast);
        assert!(out.viewport_output.values().any(|vp| vp.commands.iter().any(|cmd| matches!(cmd, ViewportCommand::BeginResize(ResizeDirection::East)))));

        // 3. South Edge Right of Center (950.0, 698.0)
        let mut raw_input = egui::RawInput {
            screen_rect: Some(screen),
            ..Default::default()
        };
        raw_input.events.push(egui::Event::PointerMoved(pos2(950.0, 698.0)));
        raw_input.events.push(egui::Event::PointerButton {
            pos: pos2(950.0, 698.0),
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: Default::default(),
        });
        let out = ctx.run(raw_input, handle_window_resize);
        assert_eq!(out.platform_output.cursor_icon, CursorIcon::ResizeSouth);
        assert!(out.viewport_output.values().any(|vp| vp.commands.iter().any(|cmd| matches!(cmd, ViewportCommand::BeginResize(ResizeDirection::South)))));
    }

    #[test]
    fn test_handle_window_resize_playerbar_center_deep_check() {
        let ctx = Context::default();
        let screen = Rect::from_min_size(pos2(0.0, 0.0), vec2(1000.0, 700.0));

        // Sample points across playerbar center (x from 250.0 to 750.0, y from 630.0 to 699.0)
        let test_points = [
            pos2(300.0, 650.0),
            pos2(500.0, 660.0), // seekbar area
            pos2(500.0, 695.0), // bottom border
            pos2(500.0, 698.0), // bottom-most border margin
            pos2(700.0, 680.0),
        ];

        for &pt in &test_points {
            let mut raw_input = egui::RawInput {
                screen_rect: Some(screen),
                ..Default::default()
            };
            raw_input.events.push(egui::Event::PointerMoved(pt));
            raw_input.events.push(egui::Event::PointerButton {
                pos: pt,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: Default::default(),
            });
            let out = ctx.run(raw_input, handle_window_resize);
            assert_ne!(out.platform_output.cursor_icon, CursorIcon::ResizeSouth, "Point {:?} must NOT have ResizeSouth cursor", pt);
            let has_resize = out.viewport_output.values().any(|vp| vp.commands.iter().any(|cmd| matches!(cmd, ViewportCommand::BeginResize(_))));
            assert!(!has_resize, "Point {:?} must NOT emit BeginResize command", pt);
        }
    }
}
