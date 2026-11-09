# Agent Protocol & Operating Rules

All subagents participating in the Umbra repair and redesign project MUST strictly adhere to these rules:

1. **Inter-Agent Communication**:
   - The shared files (`PLAN.md`, `TASKS.md`, `BUGS.md`, `DESIGN.md`, and `handoffs/`) are the **only** way agents communicate.
   - Every agent MUST re-read these shared files before beginning any task.

2. **Domain Isolation**:
   - Each agent edits **only** its own designated files and area.
   - If a change is needed in another agent's area, write a request file at `handoffs/REQUEST-<from>-<to>.md` rather than modifying the other agent's code.

3. **Task Lifecycle**:
   - Claim a task in `TASKS.md` (set status to `IN_PROGRESS` with your agent name) before starting.
   - When finished, mark the task as `HANDOFF_READY` in `TASKS.md` and write a detailed handoff document at `handoffs/<task-id>.md` (explaining what changed, how to test, and any open issues).

4. **Build & Quality Gates**:
   - Never commit or declare done any code that does not build.
   - Run `cargo check`, `cargo clippy`, and `cargo test` before marking anything done.
   - Test by **actually running the app** (e.g. `cargo run` or `umbra.exe`), not just compiling.

5. **Handling Uncertainty**:
   - If unsure about design, architecture, or behavior, write `handoffs/QUESTION-<id>.md` and stop. Do not guess or assume.
