use std::net::SocketAddr;

use serde::Deserialize;

#[derive(Deserialize)]
pub struct Route {
    pub host: String,
    pub action: Action,
}

#[derive(Deserialize)]
pub enum Action {
    Forward {
        target: String,
    },
    Redirect {
        target: String,
        #[serde(default)]
        permanent: bool,
    },
}

#[derive(Deserialize)]
pub struct Config {
    pub listen_on: SocketAddr,
    pub routes: Vec<Route>,
}

pub fn parse() -> eyre::Result<Config> {
    match std::env::var("ROUTER_CONFIG") {
        Ok(raw) => Ok(ron::from_str(&raw)?),
        Err(_) => Ok(ron::from_str(&std::fs::read_to_string("router.ron")?)?),
    }
}
