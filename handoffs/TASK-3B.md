# Task 3B Handoff: Zero-Emoji Vector/SVG Icon System & Sidebar Layout Overhaul

## Overview
Purged 100% of Unicode emojis and font-dependent text glyphs across the codebase. Created a dedicated pure-vector icon rendering engine in `src/ui/icons.rs` utilizing `egui::Painter` shapes and geometry for crisp rendering at any DPI and scale. Overhauled the sidebar layout with fixed 240px width, refined scroll area padding, compact status badges for streaming services, high-contrast active navigation pills with 3px left indicator bars, and a full-width footer statistics card.

## Key Changes
1. **Vector Icon Rendering Engine** (`src/ui/icons.rs`):
   - Defined `Icon` enum covering 48 core icons (Play, Pause, SkipBack, SkipForward, Shuffle, Repeat, RepeatOne, Heart, HeartOutline, VolumeMute, VolumeLow, VolumeMed, VolumeHigh, Search, Home, Music, Disc, Mic, Tag, Folder, Clock, Fire, Sparkle, Hourglass, Dices, Playlist, Settings, Library, Close, Maximize, Restore, Minimize, Trash, Plus, ChevronLeft, ChevronRight, ChevronDown, ChevronUp, Info, Refresh, Queue, Expand, Spotify, YouTube, Palette, BarChart, Headphones).
   - Rendered entirely via vector primitives (`Shape::Path`, `Shape::LineSegment`, `Shape::Circle`, `painter.rect_filled`, `painter.arc`) normalized on a 24x24 coordinate grid.
   - Provided helper functions `icon_button`, `icon_toggle_button`, and `paint_icon_with_stroke` for seamless integration into egui UI blocks.
   - STRICT COMPLIANCE: 0 Unicode emojis, 0 fallback text glyphs.

2. **Sidebar Layout & Styling Overhaul** (`src/ui/sidebar.rs`):
   - Fixed sidebar width to 240px with clean scroll area padding (`Margin::symmetric(12.0, 10.0)`).
   - Replaced all raw unicode icons with vector icons (`Icon::Home`, `Icon::Music`, `Icon::Disc`, `Icon::Mic`, `Icon::Tag`, `Icon::Folder`, `Icon::Playlist`, `Icon::Settings`, `Icon::Info`, etc.).
   - Fixed YouTube Music "Disabled" badge collision by using compact pill badges without overflowing.
   - Implemented high-contrast active navigation pill with purple glowing border/background, light lavender text (`#f0edff`), and 3px left accent indicator bar (`#b388ff`).
   - Integrated full-width footer library statistics card with database track counts and vector indicators.

3. **Global Codebase Emoji Purge**:
   - Replaced emojis in navigation tabs, buttons, player controls, lyrics headers, and dialogs with clean vector icons.
