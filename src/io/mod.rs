use std::time::Duration;

pub mod handler;

#[derive(Debug, Clone)]
pub enum IoEvent {
    Initialize,
    Sleep(Duration),
}
