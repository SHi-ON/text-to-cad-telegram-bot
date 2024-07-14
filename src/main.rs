use dotenv::dotenv;
use log::info;
use std::env;
use teloxide::prelude::*;
use tokio::main;

#[main]
async fn main() {
    dotenv().ok();

    pretty_env_logger::init();
    info!("Starting bot...");

    let bot_token =
        env::var("TELEGRAM_BOT_TOKEN").expect("environment variable TELEGRAM_BOT_TOKEN not found");
    let bot = Bot::new(bot_token);

    // Define the bot's behavior: reply to any received message with dice roll
    teloxide::repl(bot, |bot: Bot, message: Message| async move {
        bot.send_dice(message.chat.id).await?;
        Ok(())
    })
    .await;
}
