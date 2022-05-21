use anyhow::Result;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Interest};
use tokio::net::unix::UCred;
use tokio::net::{UnixListener, UnixStream};
use tracing::{error, info, info_span, warn};

mod executor;

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
                        match handle_client(stream).await {
                            Ok(_) => {}
                            Err(err) => {
                                error!(?err, "Error handling client");
                            }
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

async fn handle_client(mut stream: UnixStream) -> Result<()> {
    stream
        .ready(Interest::READABLE | Interest::WRITABLE)
        .await?;
    let peer_cred = stream.peer_cred()?;

    let span = info_span!("handle_client", ?peer_cred);
    let _e = span.enter();

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

        if let Some(reply) = executor::on_command(line.trim(), peer_cred.into()).await {
            let _ = writer.write_all(reply.as_bytes());
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
