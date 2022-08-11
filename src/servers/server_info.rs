use anyhow::Result;
use uuid::Uuid;

#[allow(dead_code)]
pub struct ServerInfo {
    id: Uuid,
    name: String,
    exec_str: String,
}

impl ServerInfo {
    pub fn fetch(_id: Uuid) -> Result<ServerInfo> {
        todo!()
    }

    pub fn new(id: Uuid, name: String, exec_str: String) -> Self {
        ServerInfo { id, name, exec_str }
    }

    pub fn get_exec_str(&self) -> &str {
        self.exec_str.as_str()
    }

    pub fn uuid(&self) -> &Uuid {
        &self.id
    }
}
