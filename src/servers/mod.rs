pub mod command;
pub mod processes;
pub mod server_handle;
pub mod server_info;

use anyhow::{anyhow, Result};
use std::collections::HashMap;

use crate::servers::command::{handle_server_command, ServerCommand};
use crate::servers::server_handle::ServerHandle;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::OnceCell;
use tracing::{info_span, Instrument};
use uuid::Uuid;

static COMMAND_SENDER: OnceCell<UnboundedSender<ServerCommand>> = OnceCell::const_new();

pub fn init() -> Result<()> {
    let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel::<ServerCommand>();
    COMMAND_SENDER.set(sender)?;

    tokio::spawn(async move {
        let mut running_servers = HashMap::<Uuid, ServerHandle>::new();

        while let Some(command) = receiver.recv().await {
            let span = info_span!("command", ?command);
            handle_server_command(command, &mut running_servers)
                .instrument(span)
                .await;
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
