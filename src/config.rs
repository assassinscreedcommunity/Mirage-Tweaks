use log::{info, warn};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use toml::Value;

pub static CONFIG: Lazy<Mutex<Config>> = Lazy::new(|| Mutex::new(Config::load()));

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    #[serde(rename = "module-names")]
    pub module_names: Option<Vec<String>>,
    pub tweaks: Option<HashMap<String, TweakConfig>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TweakConfig {
    pub enabled: bool,
    pub value: Value,
}

impl Config {
    fn load() -> Self {
        match std::fs::read_to_string("mirage-tweaks.toml") {
            Ok(config) => match toml::from_str(config.as_str()) {
                Ok(config) => {
                    info!("Loaded config from mirage-tweaks.toml");
                    config
                }
                Err(error) => {
                    info!("Couldn't parse mirage-tweaks.toml, using default config ({error})");
                    Default::default()
                }
            },
            Err(error) => {
                info!("Couldn't read mirage-tweaks.toml, using default config ({error})");
                Default::default()
            }
        }
    }

    pub fn save(&self) {
        let config = match toml::to_string(self) {
            Ok(config) => config,
            Err(error) => {
                warn!("Couldn't serialize config to toml ({error})");
                return;
            }
        };

        if let Err(error) = std::fs::write("mirage-tweaks.toml", config) {
            warn!("Couldn't write config to mirage-tweaks.toml ({error})");
        }
    }
}
