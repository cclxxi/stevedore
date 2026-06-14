//! Application state and input handling.
//!
//! [`App`] is the single owner of all UI state. Background tasks never touch it
//! directly; they send [`AppMessage`]s which the main loop applies via
//! [`App::apply_message`]. Keyboard input is handled by [`App::on_key`].

use std::collections::{HashMap, VecDeque};

use crossterm::event::{KeyCode, KeyEvent};
use tokio::sync::mpsc::Sender;
use tokio::task::JoinHandle;

use crate::docker::DockerClient;
use crate::docker::actions::{self, Action};
use crate::docker::logs;
use crate::docker::model::{Container, ContainerStats};
use crate::event::AppMessage;

/// Upper bound on buffered log lines, to cap memory on chatty containers.
const MAX_LOG_LINES: usize = 5000;
/// Lines moved per page-up / page-down keypress.
const PAGE_STEP: usize = 10;

/// Shared handles background tasks need: a Docker client and the message sender.
#[derive(Clone)]
pub struct Context {
    pub docker: DockerClient,
    pub tx: Sender<AppMessage>,
}

/// Which screen is currently shown.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    List,
    Logs,
}

/// State for the log viewer.
#[derive(Default)]
pub struct LogState {
    pub container_id: Option<String>,
    pub container_name: String,
    pub lines: VecDeque<String>,
    /// Lines scrolled up from the bottom; 0 means showing the latest output.
    pub scroll: usize,
    /// When true, the view sticks to the newest line as logs arrive.
    pub follow: bool,
    task: Option<JoinHandle<()>>,
}

impl LogState {
    /// Stop the running log stream, if any, and reset to an empty state.
    fn reset(&mut self) {
        if let Some(task) = self.task.take() {
            task.abort();
        }
        self.container_id = None;
        self.container_name.clear();
        self.lines.clear();
        self.scroll = 0;
        self.follow = true;
    }
}

/// A destructive action staged and waiting for the user to confirm or cancel.
pub struct PendingAction {
    pub action: Action,
    pub container_id: String,
    pub container_name: String,
}

/// The complete application state.
pub struct App {
    pub containers: Vec<Container>,
    pub stats: HashMap<String, ContainerStats>,
    /// Cached indices into `containers` visible under the current filter.
    /// Recomputed only when the list or the filter changes, so per-frame and
    /// per-keypress reads are allocation-free.
    visible: Vec<usize>,
    /// Selection index into the currently visible (filtered) container list.
    pub selected: usize,
    pub view: View,
    pub show_all: bool,
    pub show_help: bool,
    /// A destructive action awaiting confirmation, shown as an overlay.
    pub confirm: Option<PendingAction>,
    pub logs: LogState,
    pub status: Option<String>,
    pub should_quit: bool,
    pub docker_version: String,
}

impl App {
    /// Create a fresh app, showing only running containers by default.
    pub fn new(docker_version: String) -> Self {
        App {
            containers: Vec::new(),
            stats: HashMap::new(),
            visible: Vec::new(),
            selected: 0,
            view: View::List,
            show_all: false,
            show_help: false,
            confirm: None,
            logs: LogState {
                follow: true,
                ..Default::default()
            },
            status: None,
            should_quit: false,
            docker_version,
        }
    }

    /// Cached indices into `containers` visible under the current filter.
    pub fn visible_indices(&self) -> &[usize] {
        &self.visible
    }

    /// Rebuild the visible-index cache from the current containers and filter.
    fn recompute_visible(&mut self) {
        self.visible.clear();
        self.visible.extend(
            self.containers
                .iter()
                .enumerate()
                .filter(|(_, c)| self.show_all || c.state.is_running())
                .map(|(i, _)| i),
        );
    }

    /// The currently selected container, if any.
    pub fn selected_container(&self) -> Option<&Container> {
        self.visible
            .get(self.selected)
            .map(|&i| &self.containers[i])
    }

    /// Apply a message from a background task to the state.
    pub fn apply_message(&mut self, message: AppMessage) {
        match message {
            AppMessage::Containers(list) => {
                self.containers = list;
                self.prune_stats();
                self.recompute_visible();
                self.clamp_selection();
            }
            AppMessage::Stats(stats) => self.stats = stats,
            AppMessage::LogLine { container_id, line } => self.push_log_line(container_id, line),
            AppMessage::LogEnded { container_id } => {
                if self.logs.container_id.as_deref() == Some(container_id.as_str()) {
                    self.push_log_line(container_id, "[log stream ended]".to_string());
                }
            }
            AppMessage::Status(message) => self.status = Some(message),
            AppMessage::Error(message) => self.status = Some(message),
        }
    }

    /// Handle a key press, possibly spawning background work via `ctx`.
    pub fn on_key(&mut self, key: KeyEvent, ctx: &Context) {
        // Help overlay swallows the next keypress.
        if self.show_help {
            self.show_help = false;
            return;
        }
        // A pending confirmation captures input until it is resolved.
        if self.confirm.is_some() {
            self.on_key_confirm(key, ctx);
            return;
        }
        match self.view {
            View::List => self.on_key_list(key, ctx),
            View::Logs => self.on_key_logs(key),
        }
    }

    fn on_key_list(&mut self, key: KeyEvent, ctx: &Context) {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
            KeyCode::Char('?') => self.show_help = true,
            KeyCode::Down | KeyCode::Char('j') => self.move_selection(1),
            KeyCode::Up | KeyCode::Char('k') => self.move_selection(-1),
            KeyCode::Char('g') => self.selected = 0,
            KeyCode::Char('G') => self.select_last(),
            KeyCode::Char('a') => {
                self.show_all = !self.show_all;
                self.recompute_visible();
                self.clamp_selection();
            }
            KeyCode::Enter | KeyCode::Char('l') => self.open_logs(ctx),
            KeyCode::Char('S') => self.run_action(Action::Start, ctx),
            KeyCode::Char('s') => self.run_action(Action::Stop, ctx),
            KeyCode::Char('r') => self.run_action(Action::Restart, ctx),
            KeyCode::Char('x') | KeyCode::Char('d') => self.run_action(Action::Remove, ctx),
            _ => {}
        }
    }

    /// Handle a keypress while the confirmation overlay is up.
    fn on_key_confirm(&mut self, key: KeyEvent, ctx: &Context) {
        match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => self.confirm_pending(ctx),
            _ => self.cancel_confirm(),
        }
    }

    fn on_key_logs(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.close_logs(),
            KeyCode::Char('?') => self.show_help = true,
            KeyCode::Up | KeyCode::Char('k') => self.scroll_logs_up(1),
            KeyCode::Down | KeyCode::Char('j') => self.scroll_logs_down(1),
            KeyCode::PageUp => self.scroll_logs_up(PAGE_STEP),
            KeyCode::PageDown => self.scroll_logs_down(PAGE_STEP),
            KeyCode::Char('g') => self.scroll_logs_to_top(),
            KeyCode::Char('G') => self.scroll_logs_to_bottom(),
            KeyCode::Char('f') => self.logs.follow = !self.logs.follow,
            _ => {}
        }
    }

    fn move_selection(&mut self, delta: isize) {
        let len = self.visible.len();
        if len == 0 {
            self.selected = 0;
            return;
        }
        let max = len - 1;
        let next = self.selected as isize + delta;
        self.selected = next.clamp(0, max as isize) as usize;
    }

    fn select_last(&mut self) {
        self.selected = self.visible.len().saturating_sub(1);
    }

    fn clamp_selection(&mut self) {
        let len = self.visible.len();
        if self.selected >= len {
            self.selected = len.saturating_sub(1);
        }
    }

    /// Drop stats for containers that no longer exist (H1: bound the map).
    fn prune_stats(&mut self) {
        let live: std::collections::HashSet<&str> =
            self.containers.iter().map(|c| c.id.as_str()).collect();
        self.stats.retain(|id, _| live.contains(id.as_str()));
    }

    fn open_logs(&mut self, ctx: &Context) {
        let Some(container) = self.selected_container() else {
            self.status = Some("No container selected".to_string());
            return;
        };
        let id = container.id.clone();
        let name = container.name.clone();

        self.logs.reset();
        self.logs.container_id = Some(id.clone());
        self.logs.container_name = name;
        self.logs.follow = true;
        self.logs.task = Some(logs::spawn(ctx.docker.clone(), ctx.tx.clone(), id));
        self.view = View::Logs;
    }

    fn close_logs(&mut self) {
        self.logs.reset();
        self.view = View::List;
    }

    /// Begin a lifecycle action on the selected container. Destructive actions
    /// are staged for confirmation; the rest are dispatched immediately.
    fn run_action(&mut self, action: Action, ctx: &Context) {
        if let Some(pending) = self.request_action(action) {
            Self::dispatch(pending, ctx);
        }
    }

    /// Resolve a requested action against the current selection.
    ///
    /// Returns the action to dispatch now, or `None` when there is nothing to
    /// do — either no container is selected (a status is set) or the action is
    /// destructive and has been staged in `confirm` for the user to confirm.
    fn request_action(&mut self, action: Action) -> Option<PendingAction> {
        let Some(container) = self.selected_container() else {
            self.status = Some("No container selected".to_string());
            return None;
        };
        let pending = PendingAction {
            action,
            container_id: container.id.clone(),
            container_name: container.name.clone(),
        };
        if action.is_destructive() {
            self.confirm = Some(pending);
            None
        } else {
            Some(pending)
        }
    }

    /// Dispatch the staged action and clear the confirmation overlay.
    fn confirm_pending(&mut self, ctx: &Context) {
        if let Some(pending) = self.confirm.take() {
            Self::dispatch(pending, ctx);
        }
    }

    /// Dismiss the confirmation overlay without acting.
    fn cancel_confirm(&mut self) {
        self.confirm = None;
    }

    /// Spawn the background Docker task for a pending action.
    fn dispatch(pending: PendingAction, ctx: &Context) {
        actions::spawn(
            ctx.docker.clone(),
            ctx.tx.clone(),
            pending.action,
            pending.container_id,
            pending.container_name,
        );
    }

    fn push_log_line(&mut self, container_id: String, line: String) {
        if self.logs.container_id.as_deref() != Some(container_id.as_str()) {
            return; // stale line from a previous container
        }
        self.logs.lines.push_back(line);
        while self.logs.lines.len() > MAX_LOG_LINES {
            self.logs.lines.pop_front();
        }
        if self.logs.follow {
            self.logs.scroll = 0;
        }
    }

    fn scroll_logs_up(&mut self, step: usize) {
        let max = self.logs.lines.len().saturating_sub(1);
        self.logs.scroll = (self.logs.scroll + step).min(max);
        self.logs.follow = false;
    }

    fn scroll_logs_down(&mut self, step: usize) {
        self.logs.scroll = self.logs.scroll.saturating_sub(step);
        if self.logs.scroll == 0 {
            self.logs.follow = true;
        }
    }

    fn scroll_logs_to_top(&mut self) {
        self.logs.scroll = self.logs.lines.len().saturating_sub(1);
        self.logs.follow = false;
    }

    fn scroll_logs_to_bottom(&mut self) {
        self.logs.scroll = 0;
        self.logs.follow = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::docker::model::ContainerState;

    fn container(id: &str, state: ContainerState) -> Container {
        Container {
            id: id.to_string(),
            name: id.to_string(),
            image: "img".to_string(),
            state,
            status: "Up".to_string(),
            ports: String::new(),
        }
    }

    fn app_with(containers: Vec<Container>) -> App {
        let mut app = App::new("test".to_string());
        app.apply_message(AppMessage::Containers(containers));
        app
    }

    #[test]
    fn running_filter_hides_stopped_containers() {
        let app = app_with(vec![
            container("a", ContainerState::Running),
            container("b", ContainerState::Exited),
        ]);
        assert_eq!(app.visible_indices().len(), 1);
    }

    #[test]
    fn toggling_show_all_reveals_stopped_containers() {
        let mut app = app_with(vec![
            container("a", ContainerState::Running),
            container("b", ContainerState::Exited),
        ]);
        app.show_all = true;
        app.recompute_visible();
        assert_eq!(app.visible_indices().len(), 2);
    }

    #[test]
    fn selection_is_clamped_when_list_shrinks() {
        let mut app = app_with(vec![
            container("a", ContainerState::Running),
            container("b", ContainerState::Running),
        ]);
        app.selected = 1;
        app.apply_message(AppMessage::Containers(vec![container(
            "a",
            ContainerState::Running,
        )]));
        assert_eq!(app.selected, 0);
    }

    #[test]
    fn log_lines_are_bounded() {
        let mut app = App::new("test".to_string());
        app.logs.container_id = Some("a".to_string());
        for i in 0..(MAX_LOG_LINES + 100) {
            app.push_log_line("a".to_string(), format!("line {i}"));
        }
        assert_eq!(app.logs.lines.len(), MAX_LOG_LINES);
        assert_eq!(app.logs.lines.front().unwrap(), "line 100");
    }

    #[test]
    fn stale_log_lines_are_ignored() {
        let mut app = App::new("test".to_string());
        app.logs.container_id = Some("current".to_string());
        app.push_log_line("other".to_string(), "noise".to_string());
        assert!(app.logs.lines.is_empty());
    }

    #[test]
    fn requesting_remove_stages_a_confirmation() {
        let mut app = app_with(vec![container("web", ContainerState::Running)]);

        let dispatch = app.request_action(Action::Remove);

        assert!(
            dispatch.is_none(),
            "destructive action must not dispatch immediately"
        );
        let pending = app.confirm.as_ref().expect("confirmation should be staged");
        assert_eq!(pending.action, Action::Remove);
        assert_eq!(pending.container_name, "web");
    }

    #[test]
    fn requesting_non_destructive_action_dispatches_without_confirmation() {
        let mut app = app_with(vec![container("web", ContainerState::Running)]);

        let pending = app
            .request_action(Action::Stop)
            .expect("non-destructive action should be dispatched");

        assert_eq!(pending.action, Action::Stop);
        assert_eq!(pending.container_id, "web");
        assert!(app.confirm.is_none());
    }

    #[test]
    fn requesting_action_without_selection_sets_status() {
        let mut app = App::new("test".to_string());

        let pending = app.request_action(Action::Start);

        assert!(pending.is_none());
        assert!(app.confirm.is_none());
        assert_eq!(app.status.as_deref(), Some("No container selected"));
    }

    #[test]
    fn cancelling_confirmation_clears_pending_action() {
        let mut app = app_with(vec![container("web", ContainerState::Running)]);
        app.request_action(Action::Remove);
        assert!(app.confirm.is_some());

        app.cancel_confirm();

        assert!(app.confirm.is_none());
    }

    #[test]
    fn scrolling_up_disables_follow_and_down_to_bottom_reenables() {
        let mut app = App::new("test".to_string());
        app.logs.container_id = Some("a".to_string());
        for i in 0..50 {
            app.push_log_line("a".to_string(), format!("l{i}"));
        }
        app.scroll_logs_up(5);
        assert!(!app.logs.follow);
        assert_eq!(app.logs.scroll, 5);

        app.scroll_logs_down(5);
        assert!(app.logs.follow);
        assert_eq!(app.logs.scroll, 0);
    }
}
