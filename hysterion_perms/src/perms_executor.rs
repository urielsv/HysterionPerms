use pumpkin::{
    command::{args::ConsumedArgs, dispatcher::CommandError, CommandExecutor, CommandSender},
    server::Server,
};
use pumpkin_util::text::TextComponent;

pub struct PermsExecutor;

#[async_trait::async_trait]
impl CommandExecutor for PermsExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        _: &Server,
        _args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        sender.send_message(TextComponent::text("Usage: /perms <role/add/ping>")).await;
        Ok(())
    }
}