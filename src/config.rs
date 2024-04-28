use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    // database: DatabaseSettings,
    pub api_path: String,
    pub databases: Vec<String>,
    pub db_path: String,
    pub app_name: String,
    pub bakery_trigger_cooldown: i64,
    pub default_item_per_feed: i64,
    pub default_main_sleep: u64,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let mut s = Config::new();

        // Start off by merging with the "default" configuration file
        s.merge(File::with_name("config/default"))?;

        // Add in the current environment file
        // Default to 'development' env
        // Note: This will look for a file named `config/{environment}.json` or a similar format
        let env = std::env::var("RUN_MODE").unwrap_or_else(|_| "development".into());
        s.merge(File::with_name(&format!("config/{}", env)).required(false))?;

        // Add in settings from environment variables (with a prefix of APP)
        // Eg.. `APP_DEBUG=1 would set the `debug` key
        s.merge(Environment::with_prefix("APP").separator("__"))?;

        // Now that we're done, let's access our configuration
        s.try_into()
    }
}
