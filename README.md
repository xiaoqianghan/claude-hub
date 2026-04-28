# claude-hub

A TUI dashboard for monitoring multiple Claude Code sessions running in tmux. Built for developers who run several Claude Code instances in parallel across tmux sessions and need a single place to see which ones need attention.

## The Problem

When working with Claude Code, I naturally end up with multiple sessions — one per project or branch, each in its own tmux session. This creates a polling problem: I have to manually rotate through sessions to check if any of them need my input.

Most of the time, Claude is either actively working (running tools, writing code) or waiting for me to respond (approve a permission, answer a question, review output). The sessions that are actively working don't need me. The ones waiting for me are blocked until I notice them.

The cost of this is straightforward:

- **Wasted attention**: I check sessions that are running fine and don't need me.
- **Wasted Claude time**: Sessions that need my input sit idle because I'm looking at the wrong tmux pane.
- **Context loading**: Every time I switch to a session, I need to re-read the conversation to remember what's happening.

## The Insight

The underlying need is not "manage multiple sessions" — it's **"know which sessions need me right now, and quickly resume context when I switch to one."**

This is an interrupt-driven problem, not a polling problem. I should be _pulled_ to a session when it needs me, not _push_ my attention across all sessions hoping to find one that's blocked.

## The Solution

claude-hub is a read-only TUI that runs in its own tmux pane. It:

1. **Discovers** all running Claude Code sessions by watching `~/.claude/sessions/`.
2. **Detects state** by reading the tail of each session's JSONL transcript — is Claude working, or waiting for me?
3. **Surfaces what needs me** — sessions waiting for input sort to the top, highlighted in yellow.
4. **Provides context** — the detail panel shows the last question I asked and Claude's last response, so I can resume without re-reading the conversation.
5. **Navigates** — press Enter to `tmux switch-client` directly to that session's pane.

It does not replace Claude Code. It does not talk to Claude. It's a layer of awareness on top of tmux.

## Installation

```bash
cargo install --path .
```

Or build and run directly:

```bash
cargo build --release
./target/release/claude-hub
```

## Usage

Must be run inside a tmux session.

```
claude-hub
```

| Key | Action |
|-----|--------|
| `j`/`↓` | Select next |
| `k`/`↑` | Select previous |
| `Enter` | Switch to that session's tmux pane |
| `r` | Force refresh |
| `s` | Cycle sort order |
| `Tab` | Toggle detail panel |
| `q`/`Esc` | Quit |

## How It Works

```
~/.claude/sessions/{pid}.json     Claude Code writes one file per running session
            │
            ▼
    ┌── correlator ──┐
    │                │
    │  session file  │──→ pid, sessionId, cwd
    │  tmux panes    │──→ which pane runs which Claude (via ppid)
    │  transcript    │──→ Working vs WaitingForInput (via last JSONL event)
    │                │
    └───────┬────────┘
            │
            ▼
    ┌─── TUI (ratatui) ───────────────────────────┐
    │  #  Status      Project        Tmux     Idle │
    │  1  ◆ Needs you nixfiles       nix:1.1  3m   │
    │  2  ● Running   feed-schema    lex:1.1  10s  │
    │  3  ● Running   tensor         lex:1.2  25s  │
    ├──────────────────────────────────────────────┤
    │  You asked: "..."                            │
    │  Claude replied: "..."                       │
    └──────────────────────────────────────────────┘
```

## Alternatives Considered

- **cmux**: Purpose-built AI terminal, but requires abandoning tmux as the multiplexer.
- **Claude Code Agent Teams**: Built-in multi-agent coordination, but experimental and solves a different problem (delegation, not awareness).
- **MCP Server**: Each session connects to a shared MCP for cross-session state. Problem: the state is conversational and dies with the MCP process — no persistent memory across restarts.
- **Hooks + notifications only**: macOS notifications when Claude needs input. Solves the interrupt problem but not the context-loading problem.

claude-hub combines the interrupt signal (state detection) with the context preview (transcript parsing) in a single interface that lives alongside the existing tmux workflow.
