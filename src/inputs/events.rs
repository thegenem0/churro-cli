use std::{
    sync::{atomic::AtomicBool, atomic::Ordering, Arc},
    time::Duration,
};

use super::key::Key;
use crossterm::event;

use super::InputEvent;
use log::error;

pub struct Events {
    rx: tokio::sync::mpsc::Receiver<InputEvent>,
    // Keep so doesn't get disposed sender-side
    _tx: tokio::sync::mpsc::Sender<InputEvent>,
    stop_capture: Arc<AtomicBool>,
}

impl Events {
    pub fn new(tick_rate: Duration) -> Events {
        let (tx, rx) = tokio::sync::mpsc::channel(100);
        let stop_capture = Arc::new(AtomicBool::new(false));

        let event_tx = tx.clone();
        let event_stop_capture = stop_capture.clone();
        tokio::spawn(async move {
            loop {
                if crossterm::event::poll(tick_rate).unwrap() {
                    if let event::Event::Key(key) = event::read().unwrap() {
                        let key = Key::from(key);
                        if let Err(err) = event_tx.send(InputEvent::Input(key)).await {
                            error!("Error sending key: {}", err);
                        }
                    }
                }
                if let Err(err) = event_tx.send(InputEvent::Tick).await {
                    error!("Error sending tick: {}", err);
                }
                if event_stop_capture.load(Ordering::Relaxed) {
                    break;
                }
            }
        });

        Events {
            rx,
            _tx: tx,
            stop_capture,
        }
    }

    pub async fn next(&mut self) -> InputEvent {
        self.rx.recv().await.unwrap_or(InputEvent::Tick)
    }

    pub fn close(&mut self) {
        self.stop_capture.store(true, Ordering::Relaxed);
    }
}
