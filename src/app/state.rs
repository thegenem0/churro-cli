use std::time::Duration;

use oauth2::{basic::BasicTokenType, EmptyExtraTokenFields, StandardTokenResponse};

#[derive(Clone)]
pub enum AppState {
    Init,
    LoggedOut,
    Initialized {
        duration: Duration,
        counter_sleep: u32,
        counter_tick: u64,
        auth_token: Option<StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>>,
    },
}

impl AppState {
    pub fn initialized() -> Self {
        let duration = Duration::from_secs(1);
        let counter_sleep = 0;
        let counter_tick = 0;
        Self::Initialized {
            duration,
            counter_sleep,
            counter_tick,
            auth_token: None,
        }
    }

    pub fn is_initialized(&self) -> bool {
        matches!(self, Self::Initialized { .. })
    }

    pub fn incr_sleep(&mut self) {
        if let Self::Initialized { counter_sleep, .. } = self {
            *counter_sleep += 1;
        }
    }

    pub fn incr_tick(&mut self) {
        if let Self::Initialized { counter_tick, .. } = self {
            *counter_tick += 1;
        }
    }

    pub fn count_sleep(&self) -> Option<u32> {
        if let Self::Initialized { counter_sleep, .. } = self {
            Some(*counter_sleep)
        } else {
            None
        }
    }

    pub fn count_tick(&self) -> Option<u64> {
        if let Self::Initialized { counter_tick, .. } = self {
            Some(*counter_tick)
        } else {
            None
        }
    }

    pub fn duration(&self) -> Option<&Duration> {
        if let Self::Initialized { duration, .. } = self {
            Some(duration)
        } else {
            None
        }
    }

    pub fn increment_delay(&mut self) {
        if let Self::Initialized { duration, .. } = self {
            let secs = (duration.as_secs() + 1).clamp(1, 10);
            *duration = Duration::from_secs(secs);
        }
    }

    pub fn decrement_delay(&mut self) {
        if let Self::Initialized { duration, .. } = self {
            let secs = (duration.as_secs() + 1).clamp(1, 10);
            *duration = Duration::from_secs(secs);
        }
    }

    pub fn set_token(
        &mut self,
        token: StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>,
    ) {
        if let Self::Initialized { auth_token, .. } = self {
            *auth_token = Some(token);
        }
    }

    pub fn is_authenticated(&self) -> bool {
        if let Self::Initialized { auth_token, .. } = self {
            auth_token.is_some()
        } else {
            false
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::Init
    }
}
