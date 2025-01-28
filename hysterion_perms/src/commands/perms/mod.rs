mod add;
mod role;

use async_trait::async_trait;
use pumpkin::{
    command::{
        args::{players::PlayersArgumentConsumer, simple::SimpleArgConsumer},
        dispatcher::CommandError,
        tree::CommandTree,
        tree_builder::{argument, literal},
        CommandExecutor, CommandSender,
    },
    server::Server,
};
use pumpkin_util::text::TextComponent;

pub use add::PermsAddCommand;
pub use role::PermsRoleCommand;

use crate::{utils::success_colour, commands::Command};

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
            .send_message(TextComponent::text(
                "Hysterion Permissions Plugin"
            ).color_rgb(success_colour()))
            .await;
        Ok(())
    }
}

impl Command for PermsCommand {
    fn get_name(&self) -> &'static str {
        "perms"
    }

    fn get_description(&self) -> &'static str {
        "Manage permissions"
    }

    fn init_command() -> CommandTree where Self: Sized {
        CommandTree::new([Self.get_name()], Self.get_description())
            .then(literal("add")
                .then(argument("player", PlayersArgumentConsumer)
                    .then(argument("permission", SimpleArgConsumer)
                        .execute(PermsAddCommand))))
            .then(literal("role")
                .then(argument("role_action", SimpleArgConsumer)
                    .then(argument("player", PlayersArgumentConsumer)
                        .then(argument("role", SimpleArgConsumer)
                            .execute(PermsRoleCommand)))))
    }
} 