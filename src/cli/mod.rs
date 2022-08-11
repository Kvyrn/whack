use crate::cli::commands::create_dispatcher;
use color_eyre::eyre::eyre;
use color_eyre::Result;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Interest};
use tokio::net::unix::UCred;
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::OnceCell;
use tracing::{debug, error, info, info_span, warn, Instrument};
use yogurt::Dispatcher;

mod commands;

static DISPATCHER: OnceCell<Dispatcher<Option<ClientProperties>, Option<String>, ()>> =
    OnceCell::const_new();

pub fn init() -> Result<()> {
    let _e = info_span!("init_cli").entered();

    DISPATCHER
        .set(create_dispatcher()?)
        .map_err(|_| eyre!("Error setting dispatcher!"))?;

    let listener = UnixListener::bind("/tmp/whack.sock")?;
    info!("Opened socket at /tmp/whack.sock");

    tokio::spawn(listen(listener));
    Ok(())
}

async fn listen(listener: UnixListener) {
    loop {
        match listener.accept().await {
            Ok((stream, address)) => {
                let peer_cred = stream.peer_cred();
                let _e = info_span!("cli_connection", ?peer_cred, ?address).entered();
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
            warn!(err = ?result.unwrap_err(), "Invalid data received!");
            continue;
        } else if result? < 1 {
            // connection closed
            break;
        }

        debug!(command = ?line, "Executing command");

        let dispatcher = DISPATCHER
            .get()
            .ok_or_else(|| eyre!("Missing dispatcher!"))?;

        let client_props: ClientProperties = peer_cred.into();
        let replies = dispatcher
            .run_command_in_context(line.as_str(), Box::new(move |_| Some(client_props.clone())));

        match replies {
            Ok(replies) => {
                for reply in replies.into_iter().flatten() {
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

#[derive(Debug, Clone)]
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
