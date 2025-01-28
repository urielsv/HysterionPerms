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
        let player_uuid = player.gameprofile.id.to_string();
        let permission_str = permission.to_string();

        // Get the current runtime handle
        let handle = Handle::current();

        // Execute database operation in the runtime
        if let Err(e) = handle.block_on(async {
            permissions::add_player_permission(&player_uuid, &permission_str).await
        }) {
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