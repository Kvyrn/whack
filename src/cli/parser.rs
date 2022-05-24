use anyhow::{anyhow, Result};
use nom::bytes::complete::take_till;
use nom::character::complete::multispace0;
use nom::sequence::terminated;
use nom::IResult;

pub fn parse_command(input: &str) -> Result<(&str, &str)> {
    let result: IResult<&str, &str, nom::error::Error<&str>> = terminated(
        take_till(|c: char| c.is_whitespace()),
        multispace0,
    )(input);
    match result {
        Ok((o1, o2)) => Ok((o1, o2)),
        Err(err) => Err(anyhow!("{}", err)),
    }
}
