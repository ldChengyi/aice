use std::{env, error::Error, net::SocketAddr};

pub struct Config {
    pub bind_addr: SocketAddr,
}

impl Config {
    pub fn from_env() -> Result<Self, Box<dyn Error>> {
        let value = match env::var("AICE_BIND_ADDR") {
            Ok(value) => value,
            Err(env::VarError::NotPresent) => "127.0.0.1:3210".to_owned(),
            Err(error) => return Err(format!("cannot read AICE_BIND_ADDR: {error}").into()),
        };
        let bind_addr = value
            .parse()
            .map_err(|error| format!("invalid AICE_BIND_ADDR {value:?}: {error}"))?;

        Ok(Self { bind_addr })
    }
}
