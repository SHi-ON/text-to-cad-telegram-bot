use dotenv::dotenv;
use log::info;
use std::env;
use teloxide::{prelude::*, utils::command::BotCommands};

#[tokio::main]
async fn main() {
    dotenv().ok();

    pretty_env_logger::init();
    info!("Starting bot...");

    let bot_token =
        env::var("TELEGRAM_BOT_TOKEN").expect("Environment variable TELEGRAM_BOT_TOKEN not found");
    let bot = Bot::new(bot_token);

    Command::repl(bot, answer).await;
}

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "List of the supported commands:"
)]
enum Command {
    #[command(description = "Displays this text")]
    Help,
    #[command(description = "Generates a model for the given prompt")]
    Generate(String),
}

async fn answer(bot: Bot, message: Message, command: Command) -> ResponseResult<()> {
    match command {
        Command::Help => {
            bot.send_message(message.chat.id, Command::descriptions().to_string())
                .await?
        }
        Command::Generate(prompt) => {
            bot.send_message(message.chat.id, format!("This is your prompt: {prompt}"))
                .await?
        }
    };

    Ok(())
}
