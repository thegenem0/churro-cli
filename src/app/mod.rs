use crate::{inputs::key::Key, io::IoEvent};

use self::{
    actions::{Action, Actions},
    state::AppState,
};

pub mod actions;
pub mod state;
pub mod ui;

use log::{debug, error, warn};

#[derive(Debug, PartialEq, Eq)]
pub enum AppReturn {
    Exit,
    Continue,
}

pub struct App {
    io_tx: tokio::sync::mpsc::Sender<IoEvent>,
    actions: Actions,
    state: AppState,
    is_loading: bool,
}

impl App {
    pub fn new(io_tx: tokio::sync::mpsc::Sender<IoEvent>) -> Self {
        let actions = vec![Action::Quit].into();
        let is_loading = false;
        let state = AppState::default();

        Self {
            io_tx,
            actions,
            state,
            is_loading,
        }
    }

    pub async fn dispatch(&mut self, action: IoEvent) {
        self.is_loading = true;
        if let Err(e) = self.io_tx.send(action).await {
            self.is_loading = false;
            error!("Error sending IoEvent: {}", e);
        }
    }

    pub async fn do_action(&mut self, key: Key) -> AppReturn {
        if let Some(action) = self.actions.find(key) {
            debug!("Doing action: {:?}", action);
            match action {
                Action::Quit => return AppReturn::Exit,
                Action::Sleep => {
                    if let Some(duration) = self.state.duration().cloned() {
                        self.dispatch(IoEvent::Sleep(duration)).await
                    }
                    AppReturn::Continue
                }
                Action::IncrementDelay => {
                    self.state.increment_delay();
                    AppReturn::Continue
                }
                Action::DecrementDelay => {
                    self.state.decrement_delay();
                    AppReturn::Continue
                }
            }
        } else {
            warn!("No action found for key: {:?}", key);
            AppReturn::Continue
        }
    }

    pub fn update_on_tick(&mut self) -> AppReturn {
        self.state.incr_tick();
        AppReturn::Continue
    }

    pub fn actions(&self) -> &Actions {
        &self.actions
    }
    pub fn state(&self) -> &AppState {
        &self.state
    }

    pub fn is_loading(&self) -> bool {
        self.is_loading
    }

    pub fn initialized(&mut self) {
        // Update contextual actions
        self.actions = vec![
            Action::Quit,
            Action::Sleep,
            Action::IncrementDelay,
            Action::DecrementDelay,
        ]
        .into();
        self.state = AppState::initialized()
    }

    pub fn loaded(&mut self) {
        self.is_loading = false;
    }

    pub fn slept(&mut self) {
        self.state.incr_sleep();
    }
}
