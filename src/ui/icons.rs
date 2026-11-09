//! Pure vector icon rendering system for Eclipse using `egui::Painter`.
//! Lucide / Feather aesthetic: 16px-18px native size, stroke width ~1.75, crisp at any DPI.
//! 100% vector - ZERO Unicode emojis or font-dependent text glyphs.

use egui::{
    pos2, vec2, Color32, Painter, Pos2, Rect, Response, Rounding, Sense, Shape, Stroke, Ui,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Icon {
    Play,
    Pause,
    SkipBack,
    SkipForward,
    Shuffle,
    Repeat,
    RepeatOne,
    Heart,
    HeartOutline,
    VolumeMute,
    VolumeLow,
    VolumeMed,
    VolumeHigh,
    Search,
    Home,
    Music,
    Disc,
    Mic,
    Tag,
    Folder,
    Clock,
    Fire,
    Sparkle,
    Hourglass,
    Dices,
    Playlist,
    Settings,
    Library,
    Close,
    Maximize,
    Restore,
    Minimize,
    Trash,
    Plus,
    ChevronLeft,
    ChevronRight,
    ChevronDown,
    ChevronUp,
    Info,
    Refresh,
    Queue,
    Expand,
    Spotify,
    YouTube,
    Palette,
    BarChart,
    Headphones,
    PictureInPicture,
    MoreHorizontal,
}

/// Paint an icon onto any egui `Painter` within the target `Rect`.
/// The icon is automatically centered and scaled to fit the destination square.
pub fn paint_icon(painter: &Painter, icon: Icon, rect: Rect, color: Color32) {
    paint_icon_with_stroke(painter, icon, rect, color, None);
}

/// Paint an icon with custom stroke width override.
pub fn paint_icon_with_stroke(
    painter: &Painter,
    icon: Icon,
    rect: Rect,
    color: Color32,
    custom_stroke_width: Option<f32>,
) {
    if rect.width() <= 0.0 || rect.height() <= 0.0 {
        return;
    }

    let side = rect.width().min(rect.height());
    let s = side / 24.0;
    let center = rect.center();
    let origin = pos2(center.x - 12.0 * s, center.y - 12.0 * s);

    let p = |x: f32, y: f32| -> Pos2 { pos2(origin.x + x * s, origin.y + y * s) };

    let stroke_w = custom_stroke_width.unwrap_or_else(|| (1.75 * s).max(1.1));
    let stroke = Stroke::new(stroke_w, color);

    match icon {
        Icon::Play => {
            let pts = vec![p(6.0, 4.0), p(20.0, 12.0), p(6.0, 20.0)];
            painter.add(Shape::convex_polygon(pts, color, Stroke::NONE));
        }

        Icon::Pause => {
            let r1 = Rect::from_min_max(p(6.0, 4.5), p(6.0 + 3.5, 19.5));
            let r2 = Rect::from_min_max(p(14.5, 4.5), p(14.5 + 3.5, 19.5));
            painter.rect_filled(r1, Rounding::same(1.0 * s), color);
            painter.rect_filled(r2, Rounding::same(1.0 * s), color);
        }

        Icon::SkipBack => {
            painter.line_segment([p(5.0, 4.5), p(5.0, 19.5)], stroke);
            let pts = vec![p(19.0, 4.5), p(8.0, 12.0), p(19.0, 19.5)];
            painter.add(Shape::convex_polygon(pts, color, Stroke::NONE));
        }

        Icon::SkipForward => {
            let pts = vec![p(5.0, 4.5), p(16.0, 12.0), p(5.0, 19.5)];
            painter.add(Shape::convex_polygon(pts, color, Stroke::NONE));
            painter.line_segment([p(19.0, 4.5), p(19.0, 19.5)], stroke);
        }

        Icon::Shuffle => {
            // Crossed arrows
            // Arrow 1 top right
            painter.line_segment([p(16.0, 3.0), p(21.0, 3.0)], stroke);
            painter.line_segment([p(21.0, 3.0), p(21.0, 8.0)], stroke);
            painter.add(Shape::line(
                vec![p(4.0, 20.0), p(9.0, 15.0), p(15.0, 9.0), p(21.0, 3.0)],
                stroke,
            ));
            // Arrow 2 bottom right
            painter.line_segment([p(16.0, 21.0), p(21.0, 21.0)], stroke);
            painter.line_segment([p(21.0, 21.0), p(21.0, 16.0)], stroke);
            painter.add(Shape::line(vec![p(4.0, 4.0), p(9.0, 9.0)], stroke));
            painter.add(Shape::line(vec![p(15.0, 15.0), p(21.0, 21.0)], stroke));
        }

        Icon::Repeat => {
            // Upper loop arrow
            painter.line_segment([p(17.0, 1.5), p(21.0, 5.0)], stroke);
            painter.line_segment([p(21.0, 5.0), p(17.0, 8.5)], stroke);
            painter.add(Shape::line(
                vec![p(21.0, 5.0), p(10.0, 5.0), p(5.0, 10.0), p(5.0, 12.0)],
                stroke,
            ));
            // Lower loop arrow
            painter.line_segment([p(7.0, 22.5), p(3.0, 19.0)], stroke);
            painter.line_segment([p(3.0, 19.0), p(7.0, 15.5)], stroke);
            painter.add(Shape::line(
                vec![p(3.0, 19.0), p(14.0, 19.0), p(19.0, 14.0), p(19.0, 12.0)],
                stroke,
            ));
        }

        Icon::RepeatOne => {
            // Repeat loop + numeral 1
            painter.line_segment([p(17.0, 1.5), p(21.0, 5.0)], stroke);
            painter.line_segment([p(21.0, 5.0), p(17.0, 8.5)], stroke);
            painter.add(Shape::line(
                vec![p(21.0, 5.0), p(10.0, 5.0), p(5.0, 10.0), p(5.0, 12.0)],
                stroke,
            ));
            painter.line_segment([p(7.0, 22.5), p(3.0, 19.0)], stroke);
            painter.line_segment([p(3.0, 19.0), p(7.0, 15.5)], stroke);
            painter.add(Shape::line(
                vec![p(3.0, 19.0), p(14.0, 19.0), p(19.0, 14.0), p(19.0, 12.0)],
                stroke,
            ));
            // Numeral 1
            painter.line_segment([p(10.5, 10.0), p(12.0, 8.5)], stroke);
            painter.line_segment([p(12.0, 8.5), p(12.0, 15.5)], stroke);
        }

        Icon::Heart => {
            // Filled heart
            let pts = heart_points(&p);
            painter.add(Shape::convex_polygon(pts.clone(), color, Stroke::NONE));
            painter.add(Shape::closed_line(pts, stroke));
        }

        Icon::HeartOutline => {
            // Outlined heart
            let pts = heart_points(&p);
            painter.add(Shape::closed_line(pts, stroke));
        }

        Icon::VolumeMute => {
            let pts = vec![
                p(11.0, 5.0),
                p(6.0, 9.0),
                p(2.0, 9.0),
                p(2.0, 15.0),
                p(6.0, 15.0),
                p(11.0, 19.0),
            ];
            painter.add(Shape::convex_polygon(pts, color, stroke));
            // Mute cross
            painter.line_segment([p(22.0, 9.0), p(16.0, 15.0)], stroke);
            painter.line_segment([p(16.0, 9.0), p(22.0, 15.0)], stroke);
        }

        Icon::VolumeLow => {
            let pts = vec![
                p(11.0, 5.0),
                p(6.0, 9.0),
                p(2.0, 9.0),
                p(2.0, 15.0),
                p(6.0, 15.0),
                p(11.0, 19.0),
            ];
            painter.add(Shape::convex_polygon(pts, color, stroke));
            // 1 sound wave arc
            painter.add(Shape::line(
                vec![p(15.5, 8.5), p(17.5, 12.0), p(15.5, 15.5)],
                stroke,
            ));
        }

        Icon::VolumeMed => {
            let pts = vec![
                p(11.0, 5.0),
                p(6.0, 9.0),
                p(2.0, 9.0),
                p(2.0, 15.0),
                p(6.0, 15.0),
                p(11.0, 19.0),
            ];
            painter.add(Shape::convex_polygon(pts, color, stroke));
            painter.add(Shape::line(
                vec![p(15.0, 9.0), p(17.0, 12.0), p(15.0, 15.0)],
                stroke,
            ));
            painter.add(Shape::line(
                vec![p(18.5, 6.5), p(21.0, 12.0), p(18.5, 17.5)],
                stroke,
            ));
        }

        Icon::VolumeHigh => {
            let pts = vec![
                p(11.0, 5.0),
                p(6.0, 9.0),
                p(2.0, 9.0),
                p(2.0, 15.0),
                p(6.0, 15.0),
                p(11.0, 19.0),
            ];
            painter.add(Shape::convex_polygon(pts, color, stroke));
            painter.add(Shape::line(
                vec![p(15.0, 9.0), p(17.0, 12.0), p(15.0, 15.0)],
                stroke,
            ));
            painter.add(Shape::line(
                vec![p(18.5, 6.0), p(21.5, 12.0), p(18.5, 18.0)],
                stroke,
            ));
        }

        Icon::Search => {
            painter.circle_stroke(p(11.0, 11.0), 6.5 * s, stroke);
            painter.line_segment([p(16.0, 16.0), p(21.0, 21.0)], stroke);
        }

        Icon::Home => {
            // Roof
            painter.add(Shape::line(
                vec![p(3.0, 9.5), p(12.0, 2.5), p(21.0, 9.5)],
                stroke,
            ));
            // Body
            painter.add(Shape::line(
                vec![
                    p(4.5, 9.5),
                    p(4.5, 21.0),
                    p(9.5, 21.0),
                    p(9.5, 13.0),
                    p(14.5, 13.0),
                    p(14.5, 21.0),
                    p(19.5, 21.0),
                    p(19.5, 9.5),
                ],
                stroke,
            ));
        }

        Icon::Music => {
            // Double musical note with beam
            painter.circle_filled(p(6.0, 18.0), 2.8 * s, color);
            painter.circle_filled(p(18.0, 15.0), 2.8 * s, color);
            painter.line_segment([p(8.8, 18.0), p(8.8, 6.0)], stroke);
            painter.line_segment([p(20.8, 15.0), p(20.8, 3.0)], stroke);
            painter.add(Shape::line(
                vec![p(8.5, 6.0), p(21.0, 3.0)],
                Stroke::new(stroke_w * 1.5, color),
            ));
        }

        Icon::Disc => {
            // Vinyl disc
            painter.circle_stroke(p(12.0, 12.0), 9.5 * s, stroke);
            painter.circle_stroke(p(12.0, 12.0), 3.5 * s, stroke);
            painter.circle_filled(p(12.0, 12.0), 1.2 * s, color);
        }

        Icon::Mic => {
            // Microphone capsule
            let r = Rect::from_min_max(p(9.0, 2.5), p(15.0, 13.0));
            painter.rect_stroke(r, Rounding::same(3.0 * s), stroke);
            // Cradle
            painter.add(Shape::line(
                vec![p(5.0, 10.0), p(5.0, 13.0), p(12.0, 18.5), p(19.0, 13.0), p(19.0, 10.0)],
                stroke,
            ));
            // Stand
            painter.line_segment([p(12.0, 18.5), p(12.0, 21.5)], stroke);
            painter.line_segment([p(8.0, 21.5), p(16.0, 21.5)], stroke);
        }

        Icon::Tag => {
            let pts = vec![
                p(20.5, 13.5),
                p(13.5, 20.5),
                p(2.5, 9.5),
                p(2.5, 3.5),
                p(8.5, 3.5),
            ];
            painter.add(Shape::closed_line(pts, stroke));
            painter.circle_filled(p(6.5, 7.5), 1.4 * s, color);
        }

        Icon::Folder => {
            let pts = vec![
                p(2.5, 6.0),
                p(8.5, 6.0),
                p(11.0, 8.5),
                p(21.5, 8.5),
                p(21.5, 19.5),
                p(2.5, 19.5),
            ];
            painter.add(Shape::closed_line(pts, stroke));
        }

        Icon::Clock => {
            painter.circle_stroke(p(12.0, 12.0), 9.5 * s, stroke);
            painter.line_segment([p(12.0, 12.0), p(12.0, 7.0)], stroke);
            painter.line_segment([p(12.0, 12.0), p(16.0, 12.0)], stroke);
        }

        Icon::Fire => {
            // Clean flame vector
            let pts = vec![
                p(12.0, 2.5),
                p(14.5, 7.0),
                p(17.5, 9.5),
                p(18.5, 14.5),
                p(16.0, 19.5),
                p(12.0, 21.5),
                p(8.0, 19.5),
                p(5.5, 14.5),
                p(6.5, 9.5),
                p(9.5, 7.0),
                p(8.5, 11.5),
                p(11.5, 13.5),
                p(13.0, 11.5),
                p(12.0, 8.5),
            ];
            painter.add(Shape::closed_line(pts, stroke));
        }

        Icon::Sparkle => {
            // 4-point star sparkle
            let pts = vec![
                p(12.0, 2.0),
                p(13.5, 9.5),
                p(21.0, 12.0),
                p(13.5, 14.5),
                p(12.0, 22.0),
                p(10.5, 14.5),
                p(3.0, 12.0),
                p(10.5, 9.5),
            ];
            painter.add(Shape::convex_polygon(pts.clone(), color, Stroke::NONE));
            painter.add(Shape::closed_line(pts, stroke));
        }

        Icon::Hourglass => {
            painter.line_segment([p(5.0, 2.5), p(19.0, 2.5)], stroke);
            painter.line_segment([p(5.0, 21.5), p(19.0, 21.5)], stroke);
            painter.line_segment([p(6.0, 2.5), p(12.0, 12.0)], stroke);
            painter.line_segment([p(12.0, 12.0), p(6.0, 21.5)], stroke);
            painter.line_segment([p(18.0, 2.5), p(12.0, 12.0)], stroke);
            painter.line_segment([p(12.0, 12.0), p(18.0, 21.5)], stroke);
        }

        Icon::Dices => {
            let r = Rect::from_min_max(p(3.5, 3.5), p(20.5, 20.5));
            painter.rect_stroke(r, Rounding::same(4.0 * s), stroke);
            painter.circle_filled(p(8.0, 8.0), 1.5 * s, color);
            painter.circle_filled(p(12.0, 12.0), 1.5 * s, color);
            painter.circle_filled(p(16.0, 16.0), 1.5 * s, color);
            painter.circle_filled(p(16.0, 8.0), 1.5 * s, color);
            painter.circle_filled(p(8.0, 16.0), 1.5 * s, color);
        }

        Icon::Playlist => {
            painter.line_segment([p(3.5, 6.0), p(15.5, 6.0)], stroke);
            painter.line_segment([p(3.5, 12.0), p(13.0, 12.0)], stroke);
            painter.line_segment([p(3.5, 18.0), p(10.5, 18.0)], stroke);
            // Single note on right
            painter.circle_filled(p(17.0, 17.0), 2.2 * s, color);
            painter.line_segment([p(19.2, 17.0), p(19.2, 9.0)], stroke);
            painter.line_segment([p(19.2, 9.0), p(21.5, 10.5)], stroke);
        }

        Icon::Settings => {
            // Audio Sliders style (clean, modern audio aesthetics)
            painter.line_segment([p(4.0, 3.5), p(4.0, 20.5)], stroke);
            painter.circle_filled(p(4.0, 9.0), 2.2 * s, color);

            painter.line_segment([p(12.0, 3.5), p(12.0, 20.5)], stroke);
            painter.circle_filled(p(12.0, 15.0), 2.2 * s, color);

            painter.line_segment([p(20.0, 3.5), p(20.0, 20.5)], stroke);
            painter.circle_filled(p(20.0, 8.0), 2.2 * s, color);
        }

        Icon::Library => {
            // Books on a shelf
            let r1 = Rect::from_min_max(p(3.5, 5.0), p(7.5, 19.5));
            painter.rect_stroke(r1, Rounding::same(1.0 * s), stroke);
            let r2 = Rect::from_min_max(p(8.5, 5.0), p(12.5, 19.5));
            painter.rect_stroke(r2, Rounding::same(1.0 * s), stroke);
            // Leaning book
            painter.line_segment([p(14.5, 6.5), p(19.5, 19.5)], stroke);
            painter.line_segment([p(17.5, 5.0), p(21.5, 19.5)], stroke);
            // Shelf
            painter.line_segment([p(2.0, 20.5), p(22.0, 20.5)], stroke);
        }

        Icon::Close => {
            painter.line_segment([p(5.5, 5.5), p(18.5, 18.5)], stroke);
            painter.line_segment([p(18.5, 5.5), p(5.5, 18.5)], stroke);
        }

        Icon::Maximize => {
            let r = Rect::from_min_max(p(4.5, 4.5), p(19.5, 19.5));
            painter.rect_stroke(r, Rounding::same(2.5 * s), stroke);
        }

        Icon::Restore => {
            let r_front = Rect::from_min_max(p(4.0, 8.0), p(16.0, 20.0));
            painter.rect_stroke(r_front, Rounding::same(2.0 * s), stroke);
            // Back window
            painter.add(Shape::line(
                vec![p(8.0, 8.0), p(8.0, 4.0), p(20.0, 4.0), p(20.0, 16.0), p(16.0, 16.0)],
                stroke,
            ));
        }

        Icon::Minimize => {
            painter.line_segment([p(4.0, 12.0), p(20.0, 12.0)], stroke);
        }

        Icon::Trash => {
            // Lid
            painter.line_segment([p(3.0, 6.0), p(21.0, 6.0)], stroke);
            painter.add(Shape::line(
                vec![p(8.5, 6.0), p(8.5, 3.5), p(15.5, 3.5), p(15.5, 6.0)],
                stroke,
            ));
            // Can
            painter.add(Shape::line(
                vec![p(5.5, 6.0), p(6.5, 20.5), p(17.5, 20.5), p(18.5, 6.0)],
                stroke,
            ));
            // Ribs
            painter.line_segment([p(10.0, 10.0), p(10.0, 16.5)], stroke);
            painter.line_segment([p(14.0, 10.0), p(14.0, 16.5)], stroke);
        }

        Icon::Plus => {
            painter.line_segment([p(12.0, 4.5), p(12.0, 19.5)], stroke);
            painter.line_segment([p(4.5, 12.0), p(19.5, 12.0)], stroke);
        }

        Icon::ChevronLeft => {
            painter.add(Shape::line(
                vec![p(15.0, 5.5), p(8.5, 12.0), p(15.0, 18.5)],
                stroke,
            ));
        }

        Icon::ChevronRight => {
            painter.add(Shape::line(
                vec![p(9.0, 5.5), p(15.5, 12.0), p(9.0, 18.5)],
                stroke,
            ));
        }

        Icon::ChevronDown => {
            painter.add(Shape::line(
                vec![p(5.5, 9.0), p(12.0, 15.5), p(18.5, 9.0)],
                stroke,
            ));
        }

        Icon::ChevronUp => {
            painter.add(Shape::line(
                vec![p(5.5, 15.0), p(12.0, 8.5), p(18.5, 15.0)],
                stroke,
            ));
        }

        Icon::Info => {
            painter.circle_stroke(p(12.0, 12.0), 9.5 * s, stroke);
            painter.circle_filled(p(12.0, 7.5), 1.2 * s, color);
            painter.line_segment([p(12.0, 11.5), p(12.0, 16.5)], stroke);
        }

        Icon::Refresh => {
            // Circular arrow
            painter.add(Shape::line(
                vec![
                    p(21.0, 11.0),
                    p(20.0, 15.5),
                    p(17.0, 19.0),
                    p(12.0, 20.5),
                    p(7.0, 19.0),
                    p(4.0, 15.5),
                    p(3.5, 11.0),
                    p(5.0, 6.5),
                    p(8.5, 4.0),
                    p(13.0, 3.5),
                    p(17.5, 5.0),
                    p(20.0, 8.0),
                ],
                stroke,
            ));
            // Arrow head
            painter.line_segment([p(20.0, 3.5), p(20.0, 8.5)], stroke);
            painter.line_segment([p(15.5, 8.5), p(20.0, 8.5)], stroke);
        }

        Icon::Queue => {
            painter.line_segment([p(3.5, 6.0), p(20.5, 6.0)], stroke);
            painter.line_segment([p(3.5, 12.0), p(20.5, 12.0)], stroke);
            painter.line_segment([p(3.5, 18.0), p(20.5, 18.0)], stroke);
        }

        Icon::Expand => {
            // Expand diagonals
            painter.line_segment([p(14.0, 4.0), p(20.0, 4.0)], stroke);
            painter.line_segment([p(20.0, 4.0), p(20.0, 10.0)], stroke);
            painter.line_segment([p(20.0, 4.0), p(13.5, 10.5)], stroke);

            painter.line_segment([p(10.0, 20.0), p(4.0, 20.0)], stroke);
            painter.line_segment([p(4.0, 20.0), p(4.0, 14.0)], stroke);
            painter.line_segment([p(4.0, 20.0), p(10.5, 13.5)], stroke);
        }

        Icon::Spotify => {
            painter.circle_stroke(p(12.0, 12.0), 9.5 * s, stroke);
            painter.add(Shape::line(
                vec![p(6.5, 9.5), p(12.0, 8.0), p(17.5, 9.5)],
                Stroke::new(stroke_w * 1.2, color),
            ));
            painter.add(Shape::line(
                vec![p(7.5, 12.5), p(12.0, 11.5), p(16.5, 12.5)],
                stroke,
            ));
            painter.add(Shape::line(
                vec![p(8.5, 15.5), p(12.0, 14.8), p(15.5, 15.5)],
                Stroke::new((stroke_w * 0.8).max(1.0), color),
            ));
        }

        Icon::YouTube => {
            let r = Rect::from_min_max(p(2.5, 5.5), p(21.5, 18.5));
            painter.rect_stroke(r, Rounding::same(4.0 * s), stroke);
            let tri = vec![p(10.0, 9.0), p(15.5, 12.0), p(10.0, 15.0)];
            painter.add(Shape::convex_polygon(tri, color, Stroke::NONE));
        }

        Icon::Palette => {
            // Paint palette
            let pts = vec![
                p(12.0, 2.5),
                p(17.0, 3.5),
                p(21.5, 8.0),
                p(21.5, 13.5),
                p(18.5, 19.5),
                p(14.0, 21.5),
                p(8.5, 21.0),
                p(3.5, 16.5),
                p(2.5, 11.0),
                p(5.0, 5.0),
                p(8.5, 3.0),
            ];
            painter.add(Shape::closed_line(pts, stroke));
            painter.circle_filled(p(8.0, 8.0), 1.2 * s, color);
            painter.circle_filled(p(13.5, 6.5), 1.2 * s, color);
            painter.circle_filled(p(17.5, 9.5), 1.2 * s, color);
        }

        Icon::BarChart => {
            painter.line_segment([p(4.5, 19.5), p(4.5, 11.0)], stroke);
            painter.line_segment([p(9.5, 19.5), p(9.5, 5.0)], stroke);
            painter.line_segment([p(14.5, 19.5), p(14.5, 9.0)], stroke);
            painter.line_segment([p(19.5, 19.5), p(19.5, 14.0)], stroke);
        }

        Icon::Headphones => {
            painter.add(Shape::line(
                vec![
                    p(4.0, 13.0),
                    p(4.0, 9.0),
                    p(7.0, 4.0),
                    p(17.0, 4.0),
                    p(20.0, 9.0),
                    p(20.0, 13.0),
                ],
                stroke,
            ));
            let r_left = Rect::from_min_max(p(2.5, 12.0), p(5.5, 18.5));
            let r_right = Rect::from_min_max(p(18.5, 12.0), p(21.5, 18.5));
            painter.rect_filled(r_left, Rounding::same(1.5 * s), color);
            painter.rect_filled(r_right, Rounding::same(1.5 * s), color);
        }

        Icon::PictureInPicture => {
            let r_outer = Rect::from_min_max(p(3.0, 5.0), p(21.0, 19.0));
            painter.rect_stroke(r_outer, Rounding::same(2.0 * s), stroke);
            let r_inner = Rect::from_min_max(p(12.0, 11.0), p(19.0, 17.0));
            painter.rect_filled(r_inner, Rounding::same(1.5 * s), color);
        }

        Icon::MoreHorizontal => {
            let r = 1.6 * s;
            painter.circle_filled(p(5.0, 12.0), r, color);
            painter.circle_filled(p(12.0, 12.0), r, color);
            painter.circle_filled(p(19.0, 12.0), r, color);
        }
    }
}

/// Helper generating points for smooth heart
fn heart_points(p: &impl Fn(f32, f32) -> Pos2) -> Vec<Pos2> {
    vec![
        p(12.0, 7.5),
        p(10.0, 4.5),
        p(6.5, 4.0),
        p(3.5, 7.0),
        p(3.5, 11.0),
        p(7.0, 15.0),
        p(12.0, 20.5),
        p(17.0, 15.0),
        p(20.5, 11.0),
        p(20.5, 7.0),
        p(17.5, 4.0),
        p(14.0, 4.5),
    ]
}

/// Helper widget to render a non-interactive or hoverable vector icon in an egui UI.
pub fn render_icon(ui: &mut Ui, icon: Icon, size: f32, color: Color32) -> Response {
    let (rect, resp) = ui.allocate_exact_size(vec2(size, size), Sense::hover());
    if ui.is_rect_visible(rect) {
        paint_icon(ui.painter(), icon, rect, color);
    }
    resp
}

/// Helper widget to render an interactive clickable icon button with hover feedback.
pub fn icon_button(ui: &mut Ui, icon: Icon, size: f32, color: Color32) -> Response {
    let pad = 6.0;
    let (rect, resp) = ui.allocate_exact_size(vec2(size + pad, size + pad), Sense::click());
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    if ui.is_rect_visible(rect) {
        let draw_color = if resp.hovered() {
            // Brighten slightly on hover
            Color32::from_rgba_premultiplied(
                (color.r() as u16 + 30).min(255) as u8,
                (color.g() as u16 + 30).min(255) as u8,
                (color.b() as u16 + 30).min(255) as u8,
                color.a(),
            )
        } else {
            color
        };
        let icon_rect = Rect::from_center_size(rect.center(), vec2(size, size));
        paint_icon(ui.painter(), icon, icon_rect, draw_color);
    }
    resp
}

/// Helper to render an icon inside an allocated bounding box with specified alignment.
pub fn render_icon_in_rect(painter: &Painter, icon: Icon, rect: Rect, color: Color32) {
    paint_icon(painter, icon, rect, color);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_icons_paint_without_panic() {
        let all_icons = [
            Icon::Play,
            Icon::Pause,
            Icon::SkipBack,
            Icon::SkipForward,
            Icon::Shuffle,
            Icon::Repeat,
            Icon::RepeatOne,
            Icon::Heart,
            Icon::HeartOutline,
            Icon::VolumeMute,
            Icon::VolumeLow,
            Icon::VolumeMed,
            Icon::VolumeHigh,
            Icon::Search,
            Icon::Home,
            Icon::Music,
            Icon::Disc,
            Icon::Mic,
            Icon::Tag,
            Icon::Folder,
            Icon::Clock,
            Icon::Fire,
            Icon::Sparkle,
            Icon::Hourglass,
            Icon::Dices,
            Icon::Playlist,
            Icon::Settings,
            Icon::Library,
            Icon::Close,
            Icon::Maximize,
            Icon::Restore,
            Icon::Minimize,
            Icon::Trash,
            Icon::Plus,
            Icon::ChevronLeft,
            Icon::ChevronRight,
            Icon::ChevronDown,
            Icon::ChevronUp,
            Icon::Info,
            Icon::Refresh,
            Icon::Queue,
            Icon::Expand,
            Icon::Spotify,
            Icon::YouTube,
            Icon::Palette,
            Icon::BarChart,
            Icon::Headphones,
            Icon::PictureInPicture,
            Icon::MoreHorizontal,
        ];

        let rect = Rect::from_min_size(pos2(0.0, 0.0), vec2(24.0, 24.0));
        let painter = Painter::new(
            egui::Context::default(),
            egui::LayerId::background(),
            rect,
        );

        for icon in all_icons {
            paint_icon(&painter, icon, rect, Color32::WHITE);
        }
    }

    #[test]
    fn test_icon_helpers_allocate_and_render() {
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let resp1 = render_icon(ui, Icon::Play, 16.0, Color32::WHITE);
                assert_eq!(resp1.rect.width(), 16.0);

                let resp2 = icon_button(ui, Icon::Heart, 16.0, Color32::RED);
                assert!(resp2.rect.width() >= 16.0);
            });
        });
    }

}
