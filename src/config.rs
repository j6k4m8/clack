use std::{fs::File, io::Read};

/// This module contains configuration logic for reading and writing
/// a clack config file.
use dirs::home_dir;
use toml::Value;

const DEFAULT_CONFIG_PATH: &str = ".config/clack/config.toml";

pub(crate) const DEFAULT_RATE_WPM: i64 = 300;

pub fn read_config() -> Value {
    let config_path = home_dir().unwrap().join(DEFAULT_CONFIG_PATH);

    // If the config file doesn't exist, create it with the default settings.
    if !config_path.exists() {
        return Value::from(DEFAULT_CONFIG_PATH);
    }

    let mut file = File::open(config_path).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    contents.parse::<Value>().unwrap()
}

pub struct ConfigManager {
    config: Value,
}

impl ConfigManager {
    pub fn new() -> Self {
        Self {
            config: read_config(),
        }
    }

    fn get(&mut self, key: &str) -> Option<&Value> {
        self.config.get(key)
    }

    pub fn get_rate_wpm(&mut self) -> i64 {
        self.get("rate_wpm")
            .unwrap_or(&Value::Integer(DEFAULT_RATE_WPM))
            .as_integer()
            .unwrap()
    }
}
