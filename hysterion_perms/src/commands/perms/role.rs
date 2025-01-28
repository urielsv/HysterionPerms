use async_trait::async_trait;
use pumpkin::{
    command::{
        args::{Arg, ConsumedArgs},
        dispatcher::CommandError,
        CommandExecutor, CommandSender,
    },
    server::Server,
};
use pumpkin_util::text::TextComponent;
use tokio::runtime::Handle;

use crate::{permissions, utils::success_colour};

pub struct PermsRoleCommand;

#[async_trait]
impl CommandExecutor for PermsRoleCommand {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        _: &Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let Some(Arg::Simple(role_action)) = args.get("role_action") else {
            return Err(CommandError::InvalidConsumption(Some("role_action".into())));
        };
        let Some(Arg::Players(targets)) = args.get("player") else {
            return Err(CommandError::InvalidConsumption(Some("player".into())));
        };
        let Some(Arg::Simple(role)) = args.get("role") else {
            return Err(CommandError::InvalidConsumption(Some("role".into())));
        };

        let player = &targets[0];
        let player_uuid = player.gameprofile.id.to_string();
        let role_name = role.to_string();

        if *role_action == "add" {
            if let Err(e) = Handle::current().block_on(async move {
                permissions::add_player_to_role(&player_uuid, &role_name).await
            }) {
                log::error!("Failed to add role: {}", e);
                return Ok(());
            }
            sender
                .send_message(TextComponent::text(format!(
                    "Added role {} to {}",
                    role, player.gameprofile.name
                )).color_rgb(success_colour()))
                .await;
        } else {
            sender
                .send_message(TextComponent::text("Invalid role action. Use 'add' or 'remove'"))
                .await;
        }
        Ok(())
    }
} 