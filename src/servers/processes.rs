use std::process::Stdio;

use anyhow::{anyhow, Context, Result};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::select;
use tokio::signal::unix::SignalKind;
use tokio::sync::mpsc::UnboundedReceiver;
use tracing::{debug, error, info, info_span, Instrument};
use uuid::Uuid;

use crate::servers::server_handle::ServerHandle;
use crate::servers::server_info::ServerInfo;

pub fn spawn_server(id: Uuid) {
    let (sender, receiver) = tokio::sync::mpsc::unbounded_channel::<ProcCommand>();

    tokio::spawn(async move {
        let span = info_span!("handle_server", ?id);
        if let Err(err) = handle_server(id, receiver).instrument(span).await {
            error!(?err, "Error handling server!")
        }
    });
}

async fn handle_server(id: Uuid, mut receiver: UnboundedReceiver<ProcCommand>) -> Result<()> {
    let info = ServerInfo::fetch(id).unwrap();
    let exec = info.get_exec_str();
    let exec_str = exec.as_str();
    let args: Vec<String> = shell_words::split(exec_str).context("Invalid exec string")?;
    let program = args.first().ok_or_else(|| anyhow!("Invalid exec string"))?;

    let mut command = Command::new(program);
    command.args(&args[1..]);
    command.stdin(Stdio::piped());
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());

    let mut child = command.spawn().context("Error spawning server process")?;

    let mut stdout_reader = BufReader::new(
        child
            .stdout
            .take()
            .ok_or_else(|| anyhow!("Server has no stdout handle!"))?,
    );


    loop {
        let mut line = String::new();
        select! {
            res = stdout_reader.read_line(&mut line) => {
                let line = line.trim();
                info!("stdout: {line}, {res:?}");
            },
            Some(cmd) = receiver.recv() => {
                info!("receiver: {cmd:?}");
            },
            exit = child.wait() => {
                info!("exit: {exit:?}");
                break;
            }
        };
    };

    Ok(())
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ProcCommand {
    Signal(SignalKind),
    RunCommand(String),
}
