use anyhow::Result;
use churro_cli::{
    app::App,
    io::{handler::IoAsyncHandler, IoEvent},
    start_ui,
};
use log::LevelFilter;
use std::sync::Arc;
extern crate dotenv;

use dotenv::dotenv;

extern crate dotenv_codegen;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let (sync_io_tx, mut sync_io_rx) = tokio::sync::mpsc::channel::<IoEvent>(100);

    let app = Arc::new(tokio::sync::Mutex::new(App::new(sync_io_tx.clone())));
    let app_ui = Arc::clone(&app);

    tui_logger::init_logger(LevelFilter::Debug).unwrap();
    tui_logger::set_default_level(LevelFilter::Debug);

    // Handle I/O

    tokio::spawn(async move {
        let mut handler = IoAsyncHandler::new(app);
        while let Some(io_event) = sync_io_rx.recv().await {
            handler.handle_io_event(io_event).await;
        }
    });

    start_ui(&app_ui).await?;
    Ok(())
}
