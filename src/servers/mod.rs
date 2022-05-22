pub mod server_handle;
pub mod server_info;

use anyhow::{anyhow, Result};
use std::collections::HashMap;

use crate::servers::server_handle::ServerHandle;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::OnceCell;
use tracing::{info, info_span};
use uuid::Uuid;

static COMMAND_SENDER: OnceCell<UnboundedSender<ServerCommand>> = OnceCell::const_new();

pub fn init() -> Result<()> {
    let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel::<ServerCommand>();
    COMMAND_SENDER.set(sender)?;

    tokio::spawn(async move {
        let mut running_servers = HashMap::<Uuid, ServerHandle>::new();

        while let Some(command) = receiver.recv().await {
            let span = info_span!("command", ?command);
            let _e = span.enter();
            match command {
                ServerCommand::StartServer(id) => {
                    if running_servers.contains_key(&id) {
                        info!("Server already running!");
                        return;
                    }
                    info!("Starting server");
                }
                ServerCommand::RestartServer(id) => {
                    info!("Restarting server");
                }
                ServerCommand::StopServer(id) => {
                    info!("Stopping server");
                }
            }
        }
    });
    Ok(())
}

pub fn get_command_sender() -> Result<UnboundedSender<ServerCommand>> {
    COMMAND_SENDER
        .get()
        .ok_or_else(|| anyhow!("Server not initialized!"))
        .cloned()
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum ServerCommand {
    StartServer(Uuid),
    RestartServer(Uuid),
    StopServer(Uuid),
}
