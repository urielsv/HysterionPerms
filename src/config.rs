use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::OnceCell;

#[derive(Debug, Serialize, Deserialize)]
pub struct RoleConfig {
    pub level: i32,
    pub permissions: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigValue {
    pub roles: HashMap<String, RoleConfig>,
}

#[derive(Debug)]
pub struct Config {
    pub value: ConfigValue,
    path: String,
}

static CONFIG_INSTANCE: OnceCell<Arc<Config>> = OnceCell::const_new();

impl Config {
    #[allow(dead_code)]
    pub async fn init(path: &str) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let config_path = format!("{}/config.toml", path);
        
        if Path::new(&config_path).exists() {
            let content = tokio::fs::read_to_string(&config_path).await?;
            let value: ConfigValue = toml::from_str(&content)?;
            Ok(Config {
                value,
                path: config_path,
            })
        } else {
            Ok(Config::new(&config_path).await)
        }
    }

    #[allow(dead_code)]
    pub async fn new(path: &str) -> Self {
        let default_config = r#"
[roles.default]
level = 0
permissions = []

[roles.admin]
level = 4
permissions = ["*"]
"#;

        tokio::fs::write(path, default_config).await.unwrap();

        let value: ConfigValue = toml::from_str(default_config).unwrap();

        Config {
            value,
            path: path.to_owned(),
        }
    }

    #[allow(dead_code)]
    pub async fn save(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let data = toml::to_string_pretty(&self.value)?;
        tokio::fs::write(&self.path, data).await?;
        Ok(())
    }
}

#[allow(dead_code)]
pub async fn setup_config(path: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config = Config::init(path).await?;
    if let Err(e) = CONFIG_INSTANCE.set(Arc::new(config)) {
        return Err(format!("Failed to set Config instance: {}", e).into());
    }
    Ok(())
}

#[allow(dead_code)]
pub async fn get_config() -> Arc<Config> {
    CONFIG_INSTANCE.get().expect("Config not initialized").clone()
} 