use yogurt::{Command, Dispatcher, ExecContext};
use crate::cli::ClientProperties;
use yogurt::Result;

type Ctx = ExecContext<Option<ClientProperties>>;

pub fn create_dispatcher() -> Result<Dispatcher<Option<ClientProperties>, Option<String>, ()>> {
    Dispatcher::builder()
        .base_context(())
        .context_factory(|_| None)
        .child(Command::literal("version").exec(version))
        .build()
}

fn version(_ctx: &mut Ctx) -> Result<Option<String>> {
    let version = concat!(env!("CARGO_PKG_VERSION"), "\n");
    Ok(Some(version.to_string()))
}
