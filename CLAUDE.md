# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Run

```bash
cargo build              # Build
cargo run                # Run (must be inside a tmux session)
```

No tests yet. No linter configured — use `cargo clippy` for ad-hoc checks.

## What This Is

A Rust TUI dashboard for monitoring multiple Claude Code sessions running in tmux. It shows session status (Working / Needs you / Stale), lets you navigate between them, and displays context to help resume work quickly.

Runs in its own tmux pane. Press Enter to `tmux switch-client` + `select-pane` to the selected session's exact pane.

## Architecture

Three-layer design: **data** (discover + correlate sessions) → **model** (unified SessionInfo) → **ui** (ratatui rendering).

### Data Pipeline (`data/correlator.rs`)

The core logic joins three independent data sources into `Vec<SessionInfo>`:

1. **Session registry** (`~/.claude/sessions/{pid}.json`) — one file per running Claude Code process, contains pid, sessionId, cwd, startedAt
2. **Tmux panes** (`tmux list-panes -a`) — maps pane PIDs to session:window.pane targets
3. **Transcript JSONL** (`~/.claude/projects/{encoded-cwd}/{sessionId}.jsonl`) — conversation history, parsed for state detection

**Session-to-tmux correlation**: Claude Code's `pid` → look up its `ppid` via `ps` → match against tmux `pane_pid`. This is indirect because Claude runs as a child process of the shell in the tmux pane.

### CWD Encoding

Claude Code encodes working directory paths for project storage: both `/` and `.` are replaced with `-`. Example: `/Users/foo/.config/bar` → `-Users-foo--config-bar`. This logic lives in `config::encode_cwd()`.

### State Detection (`data/transcript.rs`)

Reads the tail (~32KB) of the JSONL transcript and walks events to determine state:
- **Working**: last meaningful event is `user`, `tool_result`, or `assistant` with `stop_reason=tool_use`
- **WaitingForInput**: last event is `assistant` with `stop_reason=end_turn`, or `system` with `subtype=stop_hook_summary`
- **Stale**: process PID no longer alive

### Event Loop (`main.rs`)

Tokio-based async loop with three event sources merged into a single `mpsc` channel:
- `notify` file watcher on `~/.claude/sessions/` for session appear/disappear
- 5-second tick timer for periodic refresh
- `crossterm` keyboard input via `spawn_blocking`

## Key Constraint

All string truncation must use char-based indexing (`s.chars()`), not byte slicing (`s[..n]`). User prompts and assistant responses frequently contain CJK characters (3 bytes per char in UTF-8).
