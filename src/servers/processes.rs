use crate::servers::server_info::ServerInfo;
use anyhow::{anyhow, Context, Result};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{ChildStderr, ChildStdout, Command};
use tokio::select;
use tracing::{error, info, info_span, trace, warn, Instrument};

pub fn spawn_server(server_info: ServerInfo) -> Result<()> {
    tokio::spawn(async move {
        let span = info_span!("run_server", id = ?server_info.uuid());
        let result = run_server(server_info).instrument(span).await;
        if let Err(err) = result {
            error!(?err, "Error running server!")
        }
    });
    Ok(())
}

async fn run_server(server_info: ServerInfo) -> Result<()> {
    let args: Vec<_> =
        shell_words::split(server_info.get_exec_str()).context("Invalid exec string")?;
    let mut command = Command::new(args.first().ok_or_else(|| anyhow!("Invalid exec string"))?);
    command
        .args(&args[1..])
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .stdin(Stdio::piped());

    let mut child = command.spawn().context("Error spawning child process")?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| anyhow!("Missing child stdout!"))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| anyhow!("Missing child stdout!"))?;
    tokio::spawn(read_stdout(stdout, stderr).in_current_span());

    child.wait().await?;
    info!("Child exited!");

    Ok(())
}

async fn read_stdout(stdout: ChildStdout, stderr: ChildStderr) {
    let mut stdout_reader = BufReader::new(stdout);
    let mut stderr_reader = BufReader::new(stderr);

    let mut stdout_open = true;
    let mut stderr_open = true;
    while stdout_open || stderr_open {
        let mut line = String::new();
        let mut err_line = String::new();
        select! {
            res = stdout_reader.read_line(&mut line) => {
                match res {
                    Ok(n) if n > 0 => {
                        trace!("Stdout: {:?}", line);
                    },
                    Err(err) => {
                        warn!(?err, "Invalid data received from child stdout")
                    },
                    _ => {
                        stdout_open = false;
                    }
                }
            },
            res = stderr_reader.read_line(&mut err_line) => {
                match res {
                    Ok(n) if n > 0 => {
                        trace!("Stderr: {:?}", err_line);
                    },
                    Err(err) => {
                        warn!(?err, "Invalid data received from child stdout")
                    },
                    _ => {
                        stderr_open = false;
                    }
                }
            }
        }
    }
}
