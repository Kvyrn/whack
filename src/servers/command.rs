use crate::servers::server_handle::ServerHandle;
use std::collections::HashMap;
use tokio::signal::unix::SignalKind;
use tokio::sync::oneshot::Sender;
use tracing::{debug, warn};
use uuid::Uuid;

pub async fn handle_server_command(
    reply_sender: Sender<InteractionResult>,
    command: ServerCommand,
    _running_servers: &mut HashMap<Uuid, ServerHandle>,
) {
    debug!("Handling server command");

    match &command.interaction {
        /*ServerInteraction::StartServer(id) => {
            if running_servers.contains_key(&id) {
                info!("Server already running!");
                return;
            }
            info!("Starting server");
        }
        ServerInteraction::RestartServer(_id) => {
            info!("Restarting server");
        }
        ServerInteraction::StopServer(_id) => {
            info!("Stopping server");
        }*/
        ServerInteraction::Start => {}
        ServerInteraction::Restart => {}
        ServerInteraction::Stop => {}
        ServerInteraction::Signal(_) => {}
        ServerInteraction::Command(_) => {}
    }
    if let Err(err) = reply_sender.send(InteractionResult::Success) {
        warn!(?err, "Error sending command reply!");
    }
}

#[derive(Debug, Clone)]
pub struct ServerCommand {
    pub id: Uuid,
    pub interaction: ServerInteraction,
}

impl ServerCommand {
    pub fn new(id: Uuid, interaction: ServerInteraction) -> Self {
       ServerCommand {
           id,
           interaction
       }
    }
}

#[derive(Debug, Clone)]
pub enum InteractionResult {
    Success
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum ServerInteraction {
    Start,
    Restart,
    Stop,
    Signal(SignalKind),
    Command(String),
}
