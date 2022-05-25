use anyhow::Result;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Interest};
use tokio::net::unix::UCred;
use tokio::net::{UnixListener, UnixStream};
use tracing::{debug, error, info, info_span, warn, Instrument};

mod executor;
mod parser;

pub fn init() -> Result<()> {
    let listener = UnixListener::bind("/tmp/whack.sock")?;
    info!("Opened socket at /tmp/whack.sock");

    tokio::spawn(async move {
        loop {
            match listener.accept().await {
                Ok((stream, address)) => {
                    let peer_cred = stream.peer_cred();
                    let span = info_span!("cli_connection", ?peer_cred, ?address);
                    let _e = span.enter();
                    info!("Accepted cli connection!");

                    tokio::spawn(async move {
                        if let Err(err) = prep_client(stream).await {
                            error!(?err, ?address, "Error handling client");
                        }
                    });
                }
                Err(err) => {
                    error!(?err, "Failed to accept connection!");
                }
            }
        }
    });
    Ok(())
}

async fn prep_client(stream: UnixStream) -> Result<()> {
    stream
        .ready(Interest::READABLE | Interest::WRITABLE)
        .await?;
    let peer_cred = stream.peer_cred()?;

    let span = info_span!("handle_client", ?peer_cred);

    handle_client(stream, peer_cred).instrument(span).await?;

    Ok(())
}

async fn handle_client(mut stream: UnixStream, peer_cred: UCred) -> Result<()> {
    let (reader, mut writer) = stream.split();
    let mut reader = BufReader::new(reader);

    loop {
        let mut line = String::new();
        let result = reader.read_line(&mut line).await;
        if result.is_err() {
            warn!("Invalid data received!");
            continue;
        } else if result? < 1 {
            // connection closed
            break;
        }

        match executor::on_command(line.trim().to_string(), peer_cred.into()) {
            Ok(reply) => {
                if let Some(reply) = reply {
                    if let Err(write_err) = writer.write_all(reply.as_bytes()).await {
                        debug!(?write_err, "Error writing to stream!");
                    }
                }
            }
            Err(err) => {
                warn!(?err, "Error handling command!");
                if let Err(write_err) = writer.write_all("err\n".as_bytes()).await {
                    debug!(?write_err, "Error writing to stream!");
                }
            }
        }
    }

    info!("Connection closed");

    Ok(())
}

#[derive(Debug)]
pub struct ClientProperties {
    pub uid: u32,
    pub gid: u32,
}

impl From<UCred> for ClientProperties {
    fn from(cred: UCred) -> Self {
        Self {
            uid: cred.uid(),
            gid: cred.gid(),
        }
    }
}
