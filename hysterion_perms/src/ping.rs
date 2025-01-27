use pumpkin::{
    command::{args::ConsumedArgs, dispatcher::CommandError, CommandExecutor, CommandSender},
    server::Server,
};
use pumpkin_util::text::TextComponent;

pub struct PingExecutor;

#[async_trait::async_trait]
impl CommandExecutor for PingExecutor {
    async fn execute<'a>(&self, sender: &mut CommandSender<'a>, _: &Server, _: &ConsumedArgs<'a>) -> Result<(), CommandError> {
        sender.send_message(TextComponent::text("Pong!")).await;
        Ok(())
    }
}