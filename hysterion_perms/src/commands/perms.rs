use async_trait::async_trait;
use pumpkin::{
    command::{
        dispatcher::CommandError,
        tree::CommandTree,
        CommandExecutor, CommandSender,
    },
    server::Server,
};
use pumpkin_util::text::TextComponent;

use super::super::utils::success_colour;
use super::Command;

pub struct PermsCommand;

#[async_trait]
impl CommandExecutor for PermsCommand {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        _: &Server,
        _: &pumpkin::command::args::ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        sender
            .send_message(TextComponent::text("
            Usage:\n
            /perms add <player> <permission>\n
            /perms remove <player> <permission>\n
            /perms role <add/remove> <player> <role>\n
            /perms list <player>\n
            ").color_rgb(success_colour()))
            .await;
        Ok(())
    }
}

#[async_trait]
impl Command for PermsCommand {
    fn get_name(&self) -> &'static str {
        "perms"
    }

    fn get_description(&self) -> &'static str {
        "Manage permissions"
    }

    fn init_command() -> CommandTree {
        CommandTree::new([Self.get_name()], Self.get_description()).execute(Self)
    }
}