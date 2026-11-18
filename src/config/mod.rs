use serde::Deserialize;
use std::{env, error::Error, fs, io::ErrorKind, net::SocketAddr, path::PathBuf};

#[derive(Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub bind_addr: SocketAddr,
    pub public_base_url: Option<String>,
    pub llm: Option<LlmConfig>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LlmConfig {
    #[serde(default)]
    pub provider: LlmProvider,
    pub base_url: String,
    pub model: String,
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LlmProvider {
    #[default]
    Deepseek,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            bind_addr: SocketAddr::from(([127, 0, 0, 1], 3210)),
            public_base_url: None,
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

        if let Some(base) = &mut config.public_base_url {
            *base = base.trim_end_matches('/').to_owned();
            let uri: axum::http::Uri = base
                .parse()
                .map_err(|_| "public_base_url 必须是有效的 HTTP(S) 地址")?;
            if !matches!(uri.scheme_str(), Some("http" | "https"))
                || uri.authority().is_none()
                || uri
                    .authority()
                    .is_some_and(|authority| authority.as_str().contains('@'))
                || !matches!(uri.path(), "" | "/")
                || uri.query().is_some()
            {
                return Err("public_base_url 应为访问服务的根地址，例如 https://aice.example.com，不含路径或查询参数".into());
            }
        }
        Ok(config)
    }
}
