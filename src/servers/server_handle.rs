use crate::util::LineCache;
use tokio::sync::mpsc::Sender;
use uuid::Uuid;

#[derive(Debug)]
pub struct ServerHandle {
    pub id: Uuid,
    command: Sender<String>,
}

impl ServerHandle {
    fn command(&self) -> Sender<String> {
        self.command.clone()
    }
}
