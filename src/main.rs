#![warn(missing_debug_implementations, rust_2018_idioms)]
#![allow(clippy::redundant_field_names)]

use crate::config::WhackConfig;
use color_eyre::eyre::WrapErr;
use color_eyre::Result;
use tokio::select;
use tokio::signal::unix::{signal, SignalKind};
use tokio::sync::OnceCell;
use tracing::{error, info, Level};
use tracing_subscriber::util::SubscriberInitExt;
use uuid::{uuid, Uuid};

use crate::servers::command::{ServerCommand, ServerInteraction};
use crate::servers::server_info::ServerInfo;

mod cli;
mod config;
mod servers;

pub static CONFIG: OnceCell<WhackConfig> = OnceCell::const_new();
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    setup_tracing()?;
    info!("Starting whack v{}", VERSION);

    config::load_config().wrap_err("Error loading config!")?;

    cli::init().wrap_err("Error initialising cli!")?;
    servers::init().wrap_err("Error initialising servers!")?;

    let test_uuid = uuid!("8d7d8cfd-5e77-4cbb-8108-0e36c7201f42");
    let result =
        servers::run_command(ServerCommand::new(test_uuid, ServerInteraction::Start)).await;
    info!("Command result: {:?}", result);

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
    info!("Cleaning up...");

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
