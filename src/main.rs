#![warn(missing_debug_implementations, rust_2018_idioms)]

use anyhow::Result;

use tracing::Level;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::util::SubscriberInitExt;

#[tokio::main]
async fn main() -> Result<()> {
    setup_tracing()?;
    Ok(())
}

fn setup_tracing() -> Result<()> {
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .with_thread_names(true)
        .with_span_events(FmtSpan::FULL)
        .finish();
    subscriber.init();
    Ok(())
}
