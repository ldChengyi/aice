//! 配置读取与校验；环境变量可覆盖配置文件中的监听地址。

use serde::Deserialize;
use std::{env, error::Error, fs, io::ErrorKind, net::SocketAddr, path::PathBuf};

/// 应用配置；缺省配置允许服务在不启用模型和数据库的情况下启动。
#[derive(Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub bind_addr: SocketAddr,
    pub public_base_url: Option<String>,
    pub llm: Option<LlmConfig>,
}

/// 模型供应商与请求地址；密钥通过环境变量注入。
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LlmConfig {
    #[serde(default)]
    pub provider: LlmProvider,
    pub base_url: String,
    pub model: String,
}

/// 当前支持的模型供应商，按小写名称从 YAML 解析。
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
    /// 加载 YAML 并应用环境变量覆盖；显式路径缺失、未知配置项或地址无效均返回错误。
    pub fn load() -> Result<Self, Box<dyn Error>> {
        let configured_path = env::var_os("AICE_CONFIG_PATH");
        let path = configured_path
            .as_ref()
            .map_or_else(|| PathBuf::from("config.yaml"), PathBuf::from);
        let mut config = match fs::read_to_string(&path) {
            Ok(contents) => serde_yaml_ng::from_str::<Self>(&contents)
                .map_err(|error| format!("invalid config file {}: {error}", path.display()))?,
            // 仅默认文件缺失时回退；用户显式指定的文件缺失应暴露配置错误。
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

        // 下载链接在根地址后追加固定路由，因此不接受路径、查询参数或用户信息。
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
