use crate::servers::server_handle::ServerHandle;
use crate::servers::server_info::ServerInfo;
use anyhow::{anyhow, Context, Result};
use log_buffer::LogBuffer;
use parking_lot::Mutex;
use std::fmt::Write;
use std::process::Stdio;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{ChildStderr, ChildStdin, ChildStdout, Command};
use tokio::select;
use tokio::sync::mpsc::UnboundedReceiver;
#[cfg(debug_assertions)]
use tracing::debug as line_log;
#[cfg(not(debug_assertions))]
use tracing::trace as line_log;
use tracing::{error, info, info_span, warn, Instrument};

pub fn spawn_server(server_info: ServerInfo) -> Result<ServerHandle> {
    let (input_sender, input_receiver) = tokio::sync::mpsc::unbounded_channel::<String>();
    let output_cache = Arc::new(Mutex::new(LogBuffer::new(vec![0; 4096])));
    let cache_clone = output_cache.clone();
    let id = *server_info.uuid();

    tokio::spawn(async move {
        let span = info_span!("run_server", id = ?server_info.uuid());
        let result = run_server(server_info, input_receiver, cache_clone)
            .instrument(span)
            .await;
        if let Err(err) = result {
            error!(?err, "Error running server!")
        }
    });
    Ok(ServerHandle::new(id, input_sender, output_cache))
}

async fn run_server(
    server_info: ServerInfo,
    input_receiver: UnboundedReceiver<String>,
    output_cache: Arc<Mutex<LogBuffer<Vec<u8>>>>,
) -> Result<()> {
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
        .ok_or_else(|| anyhow!("Missing child stderr!"))?;
    let stdin = child
        .stdin
        .take()
        .ok_or_else(|| anyhow!("Missing child stdin!"))?;
    tokio::spawn(read_stdout(stdout, stderr, output_cache).in_current_span());
    tokio::spawn(write_stdin(stdin, input_receiver).in_current_span());

    child.wait().await?;
    info!("Child exited!");

    Ok(())
}

async fn write_stdin(mut stdin: ChildStdin, mut receiver: UnboundedReceiver<String>) {
    while let Some(command) = receiver.recv().await {
        if let Err(err) = stdin.write_all(command.as_bytes()).await {
            warn!(?err, "Error writing to child stdin!");
        } else {
            line_log!("Stdin: {:?}", command);
        }
    }
}

async fn read_stdout(
    stdout: ChildStdout,
    stderr: ChildStderr,
    output_cache: Arc<Mutex<LogBuffer<Vec<u8>>>>,
) {
    let mut stdout_reader = BufReader::new(stdout);
    let mut stderr_reader = BufReader::new(stderr);

    let write_line = |line: String| {
        let mut buf = output_cache.lock();
        if let Err(err) = buf.write_str(line.as_str()) {
            warn!(?err, "Error writing child output to buffer!");
        }
    };

    let mut stdout_open = true;
    let mut stderr_open = true;
    while stdout_open || stderr_open {
        let mut line = String::new();
        let mut err_line = String::new();
        select! {
            res = stdout_reader.read_line(&mut line) => {
                match res {
                    Ok(n) if n > 0 => {
                        line_log!("Stdout: {:?}", line);
                        write_line(line);
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
                        line_log!("Stderr: {:?}", err_line);
                        write_line(line);
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
