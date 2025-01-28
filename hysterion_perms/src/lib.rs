mod commands;
mod permissions;
mod utils;
mod db;
mod config;

use std::path::PathBuf;
use pumpkin::plugin::Context;
use pumpkin_api_macros::{plugin_impl, plugin_method};
use pumpkin_util::PermissionLvl;

use commands::perms::PermsCommand;
use commands::Command;

#[plugin_method]
async fn on_load(&mut self, server: &Context) -> Result<(), String> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    
    // Get plugin data directory
    let data_dir = PathBuf::from(server.get_data_folder());
    std::fs::create_dir_all(&data_dir).map_err(|e| format!("Failed to create data directory: {}", e))?;
    
    // Initialize database
    let db_path = data_dir.join("hysterion_perms.db");
    if let Err(e) = db::setup_db(db_path.to_str().unwrap()).await {
        log::error!("Failed to initialize database: {}", e);
        return Err(format!("Failed to initialize database: {}", e));
    }
    
    // Initialize permissions tables
    if let Err(e) = permissions::init_tables().await {
        log::error!("Failed to initialize permission tables: {}", e);
        return Err(format!("Failed to initialize permission tables: {}", e));
    }
    
    // Load config
    let config = config::Config::load(&data_dir)
        .map_err(|e| format!("Failed to load config: {}", e))?;

    // Initialize roles from config
    for (role_name, role_config) in config.roles {
        if let Err(e) = permissions::create_role(&role_name, role_config.level).await {
            log::error!("Failed to create role {}: {}", role_name, e);
            continue;
        }
        
        // Add permissions to role
        for permission in role_config.permissions {
            if let Err(e) = permissions::add_role_permission(&role_name, &permission).await {
                log::warn!("Failed to add permission {} to role {}: {}", permission, role_name, e);
            }
        }
    }

    server
        .register_command(PermsCommand::init_command(), PermissionLvl::Four)
        .await;

    log::info!("[Hysterion (perms)] Commands registered successfully!");
    log::info!("[Hysterion (perms)] Plugin loaded!");
    Ok(())
}

#[plugin_impl(hysterion_perms)]
pub struct MyPlugin;

impl MyPlugin {
    pub fn new() -> Self {
        MyPlugin {}
    }
}

impl Default for MyPlugin {
    fn default() -> Self {
        Self::new()
    }
}