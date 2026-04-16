use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct WindowConfig {
	pub width: usize,
	pub height: usize,
}

#[derive(Debug, Deserialize)]
pub struct GraphicsConfig {
	pub antialiasing: u8
}

#[derive(Debug, Deserialize)]
pub struct Config {
	pub window: WindowConfig,
	pub graphics: GraphicsConfig
}

impl Config {
	pub fn from_toml(toml: &str) -> Self {
		toml::from_str(toml).unwrap()
	}
}
