# Project Agents & Roles

This repository uses dedicated specialist agents for development and quality assurance:

- **`orchestrator`** ([`agents/orchestrator/agent.md`](agents/orchestrator/agent.md)): Master task coordinator, roadmap planner, and handoff reviewer.
- **`interaction`** ([`agents/interaction/agent.md`](agents/interaction/agent.md)): Handles clicks, window dragging, minimize/maximize/close, timeline seek and volume sliders, scrolling, and keyboard shortcuts.
- **`fonts-rendering`** ([`agents/fonts-rendering/agent.md`](agents/fonts-rendering/agent.md)): Typography, CJK fallback glyphs (Japanese/Chinese characters), crisp icons, text overlap, and viewport layout scaling.
- **`design`** ([`agents/design/agent.md`](agents/design/agent.md)): Purple glass aesthetic matching `docs/reference/target-umbra.png`, frosted cards, translucent borders, and central `Theme` module (`src/ui/theme.rs`).
- **`qa-reviewer`** ([`agents/qa-reviewer/agent.md`](agents/qa-reviewer/agent.md)): Verifies handoffs, compiles (`cargo check`, `cargo clippy`), runs live tests, and logs defects in `BUGS.md`.

All agents adhere to the rules in [`.agent/rules/protocol.md`](.agent/rules/protocol.md).
