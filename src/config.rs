use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct WindowConfig {
	pub width: usize,
	pub height: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GraphicsConfig {
	pub antialiasing: u8,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
	pub window: WindowConfig,
	pub graphics: GraphicsConfig,
}

use log::{error, warn};

const CONFIG_PATH: &str = "config.toml";

impl Config {
	pub fn load() -> Self {
		match std::fs::read_to_string(CONFIG_PATH) {
			Ok(val) => {
				Self::from_toml(&val)
			},
			Err(err) => {
				warn!("Unable to load config file: {:?} Creating a new one.", err);
				let config = Self::default();

				match toml::to_string(&config) {
					Ok(val) => {
						if let Err(err) = std::fs::write(CONFIG_PATH, &val) {
							error!("Error saving default config to disk: {:?}", err);
						}
					},
					Err(err) => {
						error!("Error serializing default config: {:?} Something is definitely wrong here.", err);
					}
				}

				config
			}
		}
	}

	pub fn from_toml(toml: &str) -> Self {
		match toml::from_str(toml) {
			Ok(val) => val,
			Err(err) => {
				error!("Error deserializing loaded config: {:?}", err);
				Self::default()
			}
		}
	}
}

impl Default for Config {
	fn default() -> Self {
	   Self {
			window: WindowConfig {
				width: 1024,
				height: 768
			},
			graphics: GraphicsConfig {
				antialiasing: 4
			}
		}
	}
}
