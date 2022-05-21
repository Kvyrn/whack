#![warn(missing_debug_implementations, rust_2018_idioms)]
#![allow(clippy::redundant_field_names)]

mod cli;
mod servers;
mod util;

use anyhow::Result;
use tokio::select;
use tokio::signal::unix::{signal, SignalKind};

use tracing::{info, Level};

use crate::servers::ServerCommand;
use tracing_subscriber::util::SubscriberInitExt;
use uuid::uuid;

#[tokio::main]
async fn main() -> Result<()> {
    setup_tracing()?;

    servers::init()?;

    let sender = servers::get_command_sender()?;
    sender.send(ServerCommand::StartServer(uuid!(
        "8d7d8cfd-5e77-4cbb-8108-0e36c7201f42"
    )))?;

    // exit on SIGINT or SIGTERM
    let mut sigint = signal(SignalKind::interrupt())?;
    let mut sigterm = signal(SignalKind::terminate())?;
    select! {
        _ = sigint.recv() => {},
        _ = sigterm.recv() => {}
    };
    info!("Exiting!");
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
