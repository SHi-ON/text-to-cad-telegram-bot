use base64::{decode, DecodeError};
use kittycad::types::base64::Base64Data;
use kittycad::types::{ApiCallStatus, AsyncApiCallOutput, TextToCad};
use log::info;
use std::collections::HashMap;
use std::mem::size_of;
use std::os::macos::raw::stat;
use std::process::Output;
use std::thread::sleep;
use std::time::Duration;
use teloxide::types::True;
use teloxide::{prelude::*, utils::command::BotCommands};
use uuid::Uuid;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    info!("Starting bot...");

    let bot = Bot::from_env();

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

enum Generate {
    Message(String),
    Data(Option<HashMap<String, Base64Data>>),
}

async fn answer(bot: Bot, message: Message, command: Command) -> ResponseResult<()> {
    match command {
        Command::Help => {
            bot.send_message(message.chat.id, Command::descriptions().to_string())
                .await?
        }
        Command::Generate(prompt) => {
            // let response: String = generate(&prompt).await;
            match generate_cad_model(prompt).await {
                Ok(result) => {
                    bot.send_message(message.chat.id, result.to_string()).await;
                    match generate(result.id).await {
                        Ok(generate_output) => {
                            match generate_output {
                                Generate::Message(status_message) => {
                                    bot.send_message(message.chat.id, status_message)
                                        .await?
                                }
                                Generate::Data(data) => {
                                    bot.send_message(message.chat.id, "base 64 bia")
                                        .await?
                                }
                            }

                        }
                        Err(e) => bot.send_message(message.chat.id, e.to_string()).await?,
                    }
                }
                Err(e) => bot.send_message(message.chat.id, e.to_string()).await?,
            }
        }
    };

    Ok(())
}

async fn generate(prompt: Uuid) -> anyhow::Result<Generate> {
    let client = kittycad::Client::new_from_env();
    loop {
        let response: AsyncApiCallOutput = client.api_calls().get_async_operation(prompt).await?;
        match response {
            AsyncApiCallOutput::TextToCad {
                status, outputs, ..
            } => {
                println!("{:?}", status);
                match status {
                    ApiCallStatus::Failed => {
                        return Ok(Generate::Message("Failed to generate!".to_string()))
                    }
                    ApiCallStatus::Completed =>{
                        return Ok(Generate::Data(outputs))
                    }
                    _ => {
                        println!("continue the loop");
                        sleep(Duration::from_secs(5));
                    }
                }
                // return Ok(status.to_string())
            }
            _ => return Ok(Generate::Message("Could not parse the response".to_string())),
        }
    }
}

async fn generate_cad_model(prompt: String) -> anyhow::Result<TextToCad> {
    let client = kittycad::Client::new_from_env();
    let result: TextToCad = client
        .ai()
        .create_text_to_cad(
            kittycad::types::FileExportFormat::Stl,
            &kittycad::types::TextToCadCreateBody { prompt },
        )
        .await?;

    Ok(result)
}