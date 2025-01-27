use async_trait::async_trait;
use pumpkin::{
    command::{
        dispatcher::CommandError,
        tree::CommandTree,
        CommandExecutor, CommandSender,
    },
    server::Server,
};

#[async_trait]
pub trait Command: CommandExecutor {
    fn get_name(&self) -> &'static str;
    fn get_description(&self) -> &'static str;
    fn init_command() -> CommandTree where Self: Sized;
}

// Re-export commands
pub mod perms;