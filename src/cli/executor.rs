use crate::cli::parser::get_command_name;
use crate::cli::ClientProperties;
use anyhow::Result;
use tracing::{info, info_span};

/*
Example commands:
stop 6a16edc7-02f6-4f9c-b7df-07ae4a9db1c2   ->  ok
create-server                               ->  ok 6a16edc7-02f6-4f9c-b7df-07ae4a9db1c2
 */

pub fn on_command(input: String, _props: ClientProperties) -> Result<Option<String>> {
    let _e = info_span!("on_command", ?input).entered();

    let input_str = input.as_str();
    let (payload, command) = get_command_name(input_str)?;
    info!("{:?}, {:?}", command, payload);
    Ok(None)
}
