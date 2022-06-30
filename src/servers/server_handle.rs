use log_buffer::LogBuffer;
use parking_lot::Mutex;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;
use uuid::Uuid;

#[derive(Debug)]
pub struct ServerHandle {
    pub id: Uuid,
    command: UnboundedSender<String>,
    log_buffer: Arc<Mutex<LogBuffer<Vec<u8>>>>,
}

impl ServerHandle {
    pub fn new(
        id: Uuid,
        command: UnboundedSender<String>,
        log_buffer: Arc<Mutex<LogBuffer<Vec<u8>>>>,
    ) -> Self {
        Self {
            id,
            command,
            log_buffer,
        }
    }

    pub fn command(&self) -> UnboundedSender<String> {
        self.command.clone()
    }

    pub fn read_full_log(&self) -> String {
        let mut buf = self.log_buffer.lock();
        buf.extract().to_string()
    }
}
