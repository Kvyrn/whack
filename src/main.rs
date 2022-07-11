#![warn(missing_debug_implementations, rust_2018_idioms)]
#![allow(clippy::redundant_field_names)]

use crate::config::WhackConfig;
use anyhow::{Context, Result};
use tokio::select;
use tokio::signal::unix::{signal, SignalKind};
use tokio::sync::OnceCell;
use tracing::{error, info, Level};
use tracing_subscriber::util::SubscriberInitExt;
use uuid::{uuid, Uuid};

use crate::servers::command::ServerCommand;
use crate::servers::server_info::ServerInfo;

mod cli;
mod config;
mod servers;

pub static CONFIG: OnceCell<WhackConfig> = OnceCell::const_new();
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() -> Result<()> {
    setup_tracing()?;
    info!("Starting whack v{}", VERSION);

    config::load_config().context("Error loading config!")?;

    servers::init().context("Error initialising servers!")?;
    cli::init().context("Error initialising cli!")?;

    let sender = servers::get_command_sender()?;
    sender.send(ServerCommand::StartServer(uuid!(
        "8d7d8cfd-5e77-4cbb-8108-0e36c7201f42"
    )))?;

    let _handle = servers::processes::spawn_server(ServerInfo::new(
        Uuid::new_v4(),
        "epic name".to_string(),
        "ls a .".to_string(),
    ))?;

    // exit on SIGINT or SIGTERM
    let mut sigint = signal(SignalKind::interrupt())?;
    let mut sigterm = signal(SignalKind::terminate())?;
    select! {
        _ = sigint.recv() => {},
        _ = sigterm.recv() => {}
    };

    info!("Deleting socket file");
    if let Err(err) = std::fs::remove_file("/tmp/whack.sock") {
        error!(?err, "Error deleting socket file!");
    } else {
        info!("Socket file deleted");
    }

    info!("Exiting! :)");
    Ok(())
}

fn setup_tracing() -> Result<()> {
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .with_thread_ids(true)
        //.with_thread_names(true)
        //.with_span_events(FmtSpan::FULL)
        .finish();
    subscriber.init();
    Ok(())
}
