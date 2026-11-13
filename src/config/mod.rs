use serde::Deserialize;
use std::{env, error::Error, fs, io::ErrorKind, net::SocketAddr, path::PathBuf};

#[derive(Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub bind_addr: SocketAddr,
    pub llm: Option<LlmConfig>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LlmConfig {
    pub base_url: String,
    pub model: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            bind_addr: SocketAddr::from(([127, 0, 0, 1], 3210)),
            llm: None,
        }
    }
}

impl Config {
    pub fn load() -> Result<Self, Box<dyn Error>> {
        let configured_path = env::var_os("AICE_CONFIG_PATH");
        let path = configured_path
            .as_ref()
            .map_or_else(|| PathBuf::from("config.yaml"), PathBuf::from);
        let mut config = match fs::read_to_string(&path) {
            Ok(contents) => serde_yaml_ng::from_str::<Self>(&contents)
                .map_err(|error| format!("invalid config file {}: {error}", path.display()))?,
            Err(error) if error.kind() == ErrorKind::NotFound && configured_path.is_none() => {
                Self::default()
            }
            Err(error) => {
                return Err(format!("cannot read config file {}: {error}", path.display()).into());
            }
        };

        match env::var("AICE_BIND_ADDR") {
            Ok(value) => {
                config.bind_addr = value
                    .parse()
                    .map_err(|error| format!("invalid AICE_BIND_ADDR {value:?}: {error}"))?;
            }
            Err(env::VarError::NotPresent) => {}
            Err(error) => return Err(format!("cannot read AICE_BIND_ADDR: {error}").into()),
        }

        Ok(config)
    }
}
