use anyhow::Result;
use uuid::{uuid, Uuid};

pub struct ServerInfo {
    id: Uuid,
    name: String,
    exec_str: String,
}

impl ServerInfo {
    pub fn fetch(id: Uuid) -> Result<ServerInfo> {
        Ok(ServerInfo {
            id: uuid!("37e7a7c5-4d57-4b79-b10f-851f18b22d70"),
            name: String::from("dgsdgf"),
            exec_str: String::from("echo hello"),
        })
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
