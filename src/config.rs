use crate::CONFIG;
use color_eyre::Result;
use figment::providers::{Env, Format, Serialized, Toml};
use figment::Figment;
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhackConfig {
    pub admin_gid: Option<u32>,
}

impl Default for WhackConfig {
    fn default() -> Self {
        WhackConfig { admin_gid: None }
    }
}

pub fn load_config() -> Result<()> {
    let config: WhackConfig = Figment::from(Serialized::defaults(WhackConfig::default()))
        .merge(Toml::file("whack.toml"))
        .merge(Env::prefixed("WHACK_"))
        .extract()?;

    // log config for debugging
    if cfg!(debug_assertions) {
        info!("{:#?}", config);
    }

    CONFIG.set(config)?;
    Ok(())
}
