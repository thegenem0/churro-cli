use std::{sync::Arc, time::Duration};

use crate::app::App;
use eyre::Result;

use super::IoEvent;
use log::{error, info};

use crate::jira::auth_service;

pub struct IoAsyncHandler {
    app: Arc<tokio::sync::Mutex<App>>,
}

impl IoAsyncHandler {
    pub fn new(app: Arc<tokio::sync::Mutex<App>>) -> Self {
        Self { app }
    }

    pub async fn handle_io_event(&mut self, io_event: IoEvent) {
        let result = match io_event {
            IoEvent::Initialize => self.do_initialize().await,
            IoEvent::Sleep(duration) => self.do_sleep(duration).await,
            IoEvent::Login => self.do_login().await,
        };

        if let Err(err) = result {
            error!("Error handling io event: {}", err);
        }

        let mut app = self.app.lock().await;
        app.loaded();
    }

    async fn do_initialize(&mut self) -> Result<()> {
        info!("🚀 Initialize the application");
        let mut app = self.app.lock().await;
        tokio::time::sleep(Duration::from_secs(1)).await;
        app.initialized(); // we could update the app state
        info!("👍 Application initialized");

        Ok(())
    }

    async fn do_sleep(&mut self, duration: Duration) -> Result<()> {
        info!("😴 Go to sleep for {:?}...", duration);
        tokio::time::sleep(duration).await; // Sleeping
        info!("⏰ Wake up !");
        // Notify the app for having slept
        let mut app = self.app.lock().await;
        app.slept();
        Ok(())
    }

    async fn do_login(&mut self) -> Result<()> {
        info!("🔑 Login...");
        let token = auth_service::authenticate().await;
        // Notify the app for having logged in
        let mut app = self.app.lock().await;
        match token {
            Ok(token) => app.set_token(token),
            Err(err) => error!("Error logging in: {}", err),
        }
        Ok(())
    }
}
