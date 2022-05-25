use crate::servers::server_handle::ServerHandle;
use crate::ServerCommand;
use std::collections::HashMap;
use tracing::info;
use uuid::Uuid;

pub async fn handle_server_command(
    command: ServerCommand,
    running_servers: &mut HashMap<Uuid, ServerHandle>,
) {
    info!("Handling server command");

    match command {
        ServerCommand::StartServer(id) => {
            if running_servers.contains_key(&id) {
                info!("Server already running!");
                return;
            }
            info!("Starting server");
        }
        ServerCommand::RestartServer(_id) => {
            info!("Restarting server");
        }
        ServerCommand::StopServer(_id) => {
            info!("Stopping server");
        }
    }
}
