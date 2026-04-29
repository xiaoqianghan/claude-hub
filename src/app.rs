use crate::data::correlator;
use crate::model::feed::{FeedEvent, FeedEventKind};
use crate::model::session::{SessionInfo, SessionState};
use anyhow::Result;
use chrono::Utc;
use ratatui::widgets::TableState;
use std::collections::{HashMap, VecDeque};

const MAX_FEED_EVENTS: usize = 50;

fn group_by_window(sessions: Vec<SessionInfo>) -> Vec<SessionInfo> {
    let mut groups: Vec<(String, Vec<SessionInfo>)> = Vec::new();
    let mut ungrouped: Vec<SessionInfo> = Vec::new();
    let mut key_index: HashMap<String, usize> = HashMap::new();

    for s in sessions {
        if let Some(key) = s.tmux_window_key() {
            if let Some(&idx) = key_index.get(&key) {
                groups[idx].1.push(s);
            } else {
                key_index.insert(key.clone(), groups.len());
                groups.push((key, vec![s]));
            }
        } else {
            ungrouped.push(s);
        }
    }

    let mut result = Vec::new();
    for (window_key, mut panes) in groups {
        panes.sort_by_key(|s| s.state.sort_priority());
        let primary = panes.remove(0);
        let state = primary.state.clone();
        let last_activity = std::iter::once(&primary)
            .chain(panes.iter())
            .filter_map(|s| s.last_activity)
            .max();
        let last_input_tokens = std::iter::once(&primary)
            .chain(panes.iter())
            .map(|s| s.last_input_tokens)
            .max()
            .unwrap_or(0);

        result.push(SessionInfo {
            state,
            last_activity,
            last_input_tokens,
            tmux_target: Some(window_key),
            ..primary
        });
    }
    result.extend(ungrouped);
    result
}

pub enum SortOrder {
    State,
    Activity,
    Project,
}

pub struct App {
    pub sessions: Vec<SessionInfo>,
    pub table_state: TableState,
    pub show_detail: bool,
    pub in_tmux: bool,
    pub sort_order: SortOrder,
    pub should_quit: bool,
    pub feed: VecDeque<FeedEvent>,
    prev_states: HashMap<String, (SessionState, String)>,
    pending_removals: HashMap<String, String>,
}

impl App {
    pub async fn new() -> Result<Self> {
        let sessions = group_by_window(correlator::build_session_list().await?);
        let mut table_state = TableState::default();
        if !sessions.is_empty() {
            table_state.select(Some(0));
        }

        let prev_states: HashMap<String, _> = sessions
            .iter()
            .map(|s| {
                (
                    s.session_id.clone(),
                    (s.state.clone(), s.project_name.clone()),
                )
            })
            .collect();

        let mut app = Self {
            sessions,
            table_state,
            show_detail: false,
            in_tmux: crate::data::tmux::is_inside_tmux(),
            sort_order: SortOrder::State,
            should_quit: false,
            feed: VecDeque::new(),
            prev_states,
            pending_removals: HashMap::new(),
        };
        app.apply_sort();
        Ok(app)
    }

    pub async fn refresh(&mut self) -> Result<()> {
        let selected_id = self
            .table_state
            .selected()
            .and_then(|i| self.sessions.get(i))
            .map(|s| s.session_id.clone());

        self.sessions = group_by_window(correlator::build_session_list().await?);
        self.detect_changes();
        self.apply_sort();

        if let Some(id) = selected_id {
            let idx = self.sessions.iter().position(|s| s.session_id == id);
            self.table_state.select(idx.or(Some(0)));
        } else if !self.sessions.is_empty() {
            self.table_state.select(Some(0));
        } else {
            self.table_state.select(None);
        }

        Ok(())
    }

    fn detect_changes(&mut self) {
        let now = Utc::now();
        let mut new_states = HashMap::new();
        let current_ids: std::collections::HashSet<&str> = self
            .sessions
            .iter()
            .map(|s| s.session_id.as_str())
            .collect();

        for s in &self.sessions {
            new_states.insert(
                s.session_id.clone(),
                (s.state.clone(), s.project_name.clone()),
            );

            // Session reappeared after being pending removal — transient gap, cancel removal
            if self.pending_removals.remove(&s.session_id).is_some() {
                continue;
            }

            match self.prev_states.get(&s.session_id) {
                Some((prev, _)) if *prev != s.state => {
                    self.feed.push_back(FeedEvent {
                        timestamp: now,
                        project_name: s.project_name.clone(),
                        kind: FeedEventKind::StateChanged {
                            from: prev.clone(),
                            to: s.state.clone(),
                        },
                    });
                }
                None => {
                    self.feed.push_back(FeedEvent {
                        timestamp: now,
                        project_name: s.project_name.clone(),
                        kind: FeedEventKind::SessionStarted,
                    });
                }
                _ => {}
            }
        }

        // Confirm pending removals (missing for 2 consecutive refreshes)
        let confirmed: Vec<_> = self
            .pending_removals
            .drain()
            .filter(|(id, _)| !current_ids.contains(id.as_str()))
            .collect();
        for (_, project) in confirmed {
            self.feed.push_back(FeedEvent {
                timestamp: now,
                project_name: project,
                kind: FeedEventKind::SessionEnded,
            });
        }

        // Mark newly missing sessions as pending (require one more miss to confirm)
        for (id, (_, project)) in &self.prev_states {
            if !current_ids.contains(id.as_str()) {
                self.pending_removals.insert(id.clone(), project.clone());
            }
        }

        self.prev_states = new_states;
        while self.feed.len() > MAX_FEED_EVENTS {
            self.feed.pop_front();
        }
    }

    pub fn next(&mut self) {
        if self.sessions.is_empty() {
            return;
        }
        let i = self.table_state.selected().unwrap_or(0);
        let next = if i >= self.sessions.len() - 1 {
            0
        } else {
            i + 1
        };
        self.table_state.select(Some(next));
    }

    pub fn previous(&mut self) {
        if self.sessions.is_empty() {
            return;
        }
        let i = self.table_state.selected().unwrap_or(0);
        let prev = if i == 0 {
            self.sessions.len() - 1
        } else {
            i - 1
        };
        self.table_state.select(Some(prev));
    }

    pub fn selected_session(&self) -> Option<&SessionInfo> {
        self.table_state
            .selected()
            .and_then(|i| self.sessions.get(i))
    }

    pub fn cycle_sort(&mut self) {
        self.sort_order = match self.sort_order {
            SortOrder::State => SortOrder::Activity,
            SortOrder::Activity => SortOrder::Project,
            SortOrder::Project => SortOrder::State,
        };
        self.apply_sort();
    }

    fn apply_sort(&mut self) {
        match self.sort_order {
            SortOrder::State => {
                self.sessions.sort_by_key(|s| s.state.sort_priority());
            }
            SortOrder::Activity => {
                self.sessions
                    .sort_by_key(|s| std::cmp::Reverse(s.last_activity));
            }
            SortOrder::Project => {
                self.sessions
                    .sort_by(|a, b| a.project_name.cmp(&b.project_name));
            }
        }
    }
}
