use crate::data::correlator;
use crate::model::session::SessionInfo;
use anyhow::Result;
use ratatui::widgets::TableState;

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
}

impl App {
    pub async fn new() -> Result<Self> {
        let sessions = correlator::build_session_list().await?;
        let mut table_state = TableState::default();
        if !sessions.is_empty() {
            table_state.select(Some(0));
        }

        let mut app = Self {
            sessions,
            table_state,
            show_detail: true,
            in_tmux: crate::data::tmux::is_inside_tmux(),
            sort_order: SortOrder::State,
            should_quit: false,
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

        self.sessions = correlator::build_session_list().await?;
        self.apply_sort();

        // Restore selection by session_id
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
                    .sort_by(|a, b| b.last_activity.cmp(&a.last_activity));
            }
            SortOrder::Project => {
                self.sessions
                    .sort_by(|a, b| a.project_name.cmp(&b.project_name));
            }
        }
    }
}
