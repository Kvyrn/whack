use crate::cli::parser::parse_command;
use crate::cli::ClientProperties;
use anyhow::Result;
use tracing::{info, info_span};

/*
Example commands:
stop 6a16edc7-02f6-4f9c-b7df-07ae4a9db1c2   ->  ok
create-server                               ->  ok 6a16edc7-02f6-4f9c-b7df-07ae4a9db1c2
 */

pub fn on_command(input: String, _props: ClientProperties) -> Result<Option<String>> {
    let span = info_span!("on_command", data = ?input.clone());
    let _e = span.enter();

    let input_str = input.as_str();
    let (payload, command) = parse_command(input_str)?;
    info!("{:?}, {:?}", command, payload);
    Ok(None)
}
