use pumpkin_api_macros::{plugin_impl, plugin_method};
use pumpkin::{
    command::{
        args::ConsumedArgs, dispatcher::CommandError, tree::CommandTree, tree_builder::literal,
        CommandExecutor, CommandSender,
    },
    plugin::Context,
    server::Server,
};
use pumpkin_util::{text::TextComponent, PermissionLvl};

mod ping;
use ping::PingExecutor;

mod perms_executor;
use perms_executor::PermsExecutor;

const PERMS_COMMAND: &str = "perms";
const DESCRIPTION: &str = "Manage server permissions";

struct RoleExecutor;

#[async_trait::async_trait]
impl CommandExecutor for RoleExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        _: &Server,
        _args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        sender.send_message(TextComponent::text("Role command executed")).await;
        Ok(())
    }
}

pub async fn initialize_commands(context: &Context) -> Result<(), String> {
    let mut command = CommandTree::new(vec![PERMS_COMMAND.to_string()], DESCRIPTION.to_string());
    
    // Add role command with add/remove subcommands
    let mut role_node = literal("role");
    role_node = role_node.then(literal("add").execute(RoleExecutor));
    role_node = role_node.then(literal("remove").execute(RoleExecutor));
    command = command.then(role_node);

    // Add the add command
    command = command.then(literal("add").execute(PermsExecutor));

    // Add the ping command
    command = command.then(literal("ping").execute(PingExecutor));

    // Register the command
    context.register_command(command, PermissionLvl::Four).await;

    Ok(())
}

#[plugin_method]
async fn on_load(&mut self, context: &Context) -> Result<(), String> {
    initialize_commands(context).await?;
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init(); 

    log::info!("[Hysterion (perms)] Plugin loaded!");
    Ok(())
}

#[plugin_impl]
pub struct MyPlugin {}

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