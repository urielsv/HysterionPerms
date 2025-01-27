mod ping;

use pumpkin::plugin::Context;
use pumpkin_api_macros::{plugin_impl, plugin_method};
use pumpkin_util::PermissionLvl;

#[plugin_method]
async fn on_load(&mut self, server: &Context) -> Result<(), String> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    
    // server
    //     .register_command(ping::init_command(), PermissionLvl::Zero)
    //     .await;

    Ok(())
    // Never reached
    // log::info!("Ping command registered successfully!");

    // log::info!("Commands registered successfully!");
    // log::info!("Use /perms add to manage permissions");
    // log::info!("Plugin loaded!");

}

#[plugin_impl]
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