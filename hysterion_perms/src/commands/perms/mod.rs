mod add;
mod role;
mod info;

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
pub use info::PermsInfoCommand;

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
        let tree = CommandTree::new([Self.get_name()], Self.get_description())
            .execute(PermsCommand)
            .then(literal("add")
                .requires_permission()
                .then(argument("player", PlayersArgumentConsumer)
                    .then(argument("permission", SimpleArgConsumer)
                        .execute(PermsAddCommand))))
            .then(literal("role")
                .requires_permission()
                .then(argument("role_action", SimpleArgConsumer)
                    .then(argument("player", PlayersArgumentConsumer)
                        .then(argument("role", SimpleArgConsumer)
                            .execute(PermsRoleCommand)))))
            .then(literal("info")
                .requires_permission()
                .then(argument("player", PlayersArgumentConsumer)
                    .execute(PermsInfoCommand)));

        // Set requires_permission on the root node
        let mut nodes = tree.nodes;
        if let Some(first_node) = nodes.first_mut() {
            first_node.requires_permission = true;
        }
        
        CommandTree {
            nodes,
            children: tree.children,
            names: tree.names,
            description: tree.description,
        }
    }
} 