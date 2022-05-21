use crate::util::LineCache;
use tokio::sync::mpsc::Sender;
use uuid::Uuid;

#[derive(Debug)]
pub struct ServerHandle {
    pub id: Uuid,
    //stdout: LineCache,
    //stdin: Sender<String>,
}

impl ServerHandle {
    /*fn stdin(&self) -> Sender<String> {
        self.stdin.clone()
    }*/
}
