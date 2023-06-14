use std::time::Duration;

pub mod handler;

#[derive(Debug, Clone)]
pub enum IoEvent {
    Login,
    Initialize,
    Sleep(Duration),
}
