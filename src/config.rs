use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct WindowConfig {
    pub width: usize,
    pub height: usize,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub window: WindowConfig,
}

impl Config {
    pub fn from_toml(toml: &str) -> Self {
        toml::from_str(toml).unwrap()
    }
}
