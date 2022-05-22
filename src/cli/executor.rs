use nom::bytes::complete::{tag, take_till};

use nom::error::ErrorKind;
use nom::sequence::terminated;
use nom::{Err, IResult};
use tracing::info;

use crate::cli::ClientProperties;

/*
Example commands:
stop 6a16edc7-02f6-4f9c-b7df-07ae4a9db1c2   ->  ok
create-server                               ->  ok 6a16edc7-02f6-4f9c-b7df-07ae4a9db1c2
 */

#[tracing::instrument]
pub async fn on_command(
    data: &str,
    client_props: ClientProperties,
) -> anyhow::Result<Option<String>> {
    let (payload, command) = parse(data)?;
    info!("{}, {}", command, payload);
    Ok(None)
}

fn parse(input: &str) -> Result<(&str, &str), nom::error::Error<&str>> {
    let result: IResult<&str, &str, nom::error::Error<&str>> =
        terminated(take_till(|c: char| c.is_whitespace()), tag(" "))(input);

    return match result {
        Ok((o1, o2)) => Ok((o1, o2)),
        Err(err) => {
            return match err {
                Err::Incomplete(needed) => {
                    Err(nom::error::Error::new(input, ErrorKind::LengthValue))
                }
                Err::Error(err) => Err(err),
                Err::Failure(err) => Err(err),
            };
        }
    };
}
