use async_trait::async_trait;
use pumpkin::{
    command::{args::ConsumedArgs, dispatcher::CommandError, tree::CommandTree, CommandExecutor, CommandSender},
    server::Server,
};
use pumpkin_util::text::TextComponent;

pub struct PingExecutor;

const NAMES: [&str; 1] = ["ping"];
const DESCRIPTION: &str = "Test command.";

#[async_trait]
impl CommandExecutor for PingExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        _: &Server,
        _: &ConsumedArgs<'a>
    ) -> Result<(), CommandError> {
        sender.send_message(TextComponent::text("Pong!")).await;
        Ok(())
    }
}

pub fn init_command() -> CommandTree {
    log::info!("Initializing ping command...");
    CommandTree::new(NAMES, DESCRIPTION)
    .execute(PingExecutor)

}