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

use crate::{permissions, utils::success_colour, get_runtime};

pub struct PermsAddCommand;

#[async_trait]
impl CommandExecutor for PermsAddCommand {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        _server: &Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let Some(Arg::Players(targets)) = args.get("player") else {
            return Err(CommandError::InvalidConsumption(Some("player".into())));
        };
        let Some(Arg::Simple(permission)) = args.get("permission") else {
            return Err(CommandError::InvalidConsumption(Some("permission".into())));
        };

        let player = &targets[0];
        let player_uuid = player.gameprofile.id;
        let permission_str = permission.to_string();

        // Execute database operation in our runtime
        let runtime = get_runtime();
        if let Err(e) = runtime.spawn(async move {
            permissions::add_player_permission(&player_uuid, &permission_str).await
        }).await.unwrap() {
            log::error!("Failed to add permission: {}", e);
            return Ok(());
        }

        sender
            .send_message(TextComponent::text(format!(
                "Added permission {} to {}",
                permission, player.gameprofile.name
            )).color_rgb(success_colour()))
            .await;
        Ok(())
    }
} 