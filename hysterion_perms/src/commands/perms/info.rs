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

use crate::{permissions, utils::{self, success_colour, neutral_colour}, get_runtime};

pub struct PermsInfoCommand;

#[async_trait]
impl CommandExecutor for PermsInfoCommand {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        _server: &Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let Some(Arg::Players(targets)) = args.get("player") else {
            return Err(CommandError::InvalidConsumption(Some("player".into())));
        };

        let player = &targets[0];
        let player_uuid = player.gameprofile.id.to_string();

        // Execute database operation in our runtime
        let runtime = get_runtime();
        match runtime.spawn(async move {
            permissions::get_player_permissions(&player_uuid).await
        }).await.unwrap() {
            Ok(perms) => {
                // Send player info
                sender.send_message(
                    TextComponent::text(format!("=== {} Permissions ===", player.gameprofile.name))
                        .color_rgb(success_colour())
                ).await;

                // Show roles
                if perms.roles.is_empty() {
                    sender.send_message(
                        TextComponent::text("Roles: None")
                            .color_rgb(neutral_colour())
                    ).await;
                } else {
                    sender.send_message(
                        TextComponent::text(format!("Roles: {}", perms.roles.join(", ")))
                            .color_rgb(neutral_colour())
                    ).await;
                }

                // Show direct permissions
                if perms.direct_permissions.is_empty() {
                    sender.send_message(
                        TextComponent::text("Direct Permissions: None")
                            .color_rgb(neutral_colour())
                    ).await;
                } else {
                    sender.send_message(
                        TextComponent::text(format!("Direct Permissions: {}", perms.direct_permissions.join(", ")))
                            .color_rgb(neutral_colour())
                    ).await;
                }
            },
            Err(e) => {
                log::error!("Failed to get player permissions: {}", e);
                sender.send_message(
                    TextComponent::text("Failed to get player permissions")
                        .color_rgb(utils::error_colour())
                ).await;
            }
        }

        Ok(())
    }
} 