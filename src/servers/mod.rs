pub mod command;
pub mod processes;
pub mod server_handle;
pub mod server_info;
use crate::servers::command::{handle_server_command, InteractionResult, ServerCommand};
use crate::servers::server_handle::ServerHandle;
use color_eyre::eyre::bail;
use color_eyre::Result;
use std::collections::HashMap;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::sync::oneshot::Sender;
use tokio::sync::OnceCell;
use tracing::{info_span, Instrument};
use uuid::Uuid;

static COMMAND_SENDER: OnceCell<UnboundedSender<(Sender<InteractionResult>, ServerCommand)>> =
    OnceCell::const_new();

pub fn init() -> Result<()> {
    let (sender, receiver) =
        tokio::sync::mpsc::unbounded_channel::<(Sender<InteractionResult>, ServerCommand)>();
    COMMAND_SENDER.set(sender)?;
    tokio::spawn(run_servers(receiver));
    Ok(())
}

async fn run_servers(mut receiver: UnboundedReceiver<(Sender<InteractionResult>, ServerCommand)>) {
    let mut running_servers = HashMap::<Uuid, ServerHandle>::new();

    while let Some((reply_sender, command)) = receiver.recv().await {
        let span = info_span!("command", ?command);
        handle_server_command(reply_sender, command, &mut running_servers)
            .instrument(span)
            .await;
    }
}

pub async fn run_command(command: ServerCommand) -> Result<InteractionResult> {
    if let Some(sender) = COMMAND_SENDER.get() {
        let (reply_sender, reply_receiver) = tokio::sync::oneshot::channel::<InteractionResult>();
        sender.send((reply_sender, command))?;
        Ok(reply_receiver.await?)
    } else {
        bail!("Missing command sender");
    }
}
