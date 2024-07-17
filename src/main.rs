use base64::{engine, DecodeError, Engine};
use bytes::Bytes;
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
use teloxide::{prelude::*, types::InputFile, utils::command::BotCommands};
use uuid::Uuid;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    info!("Starting bot...");

    let bot: Bot = Bot::from_env();

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
        Command::Generate(prompt) => match generate_cad_model(prompt).await {
            Ok(result) => {
                bot.send_message(message.chat.id, result.to_string()).await?;
                // Duck <3
                let sticker_file_id = "CAACAgIAAxkBAAEspCVmmAOyoiYGIgXTWhY8HbJ0XBCcngACTgIAAladvQow_mttgTIDbzUE";
                let sticker_file = InputFile::file_id(sticker_file_id);
                let pending_message = bot.send_animation(message.chat.id, sticker_file).await?;
                match generate(result.id).await {
                    Ok(generate_output) => {
                        match generate_output {
                            Generate::Message(status_message) => {
                                bot.send_message(message.chat.id, status_message).await?
                            }
                            Generate::Data(data) => match data {
                                Some(data_mapping) => match decode_b64(data_mapping) {
                                    Ok(data_bytes) => {
                                        let file_name = "gear-test.stl";
                                        let input_file =
                                            InputFile::memory(Bytes::copy_from_slice(&data_bytes))
                                                .file_name(file_name);
                                        bot.send_document(message.chat.id, input_file).await?
                                    }
                                    Err(e) => {
                                        bot.send_message(
                                            message.chat.id,
                                            format!("Error while decoding: {}", e),
                                        )
                                            .await?
                                    }
                                },
                                None => {
                                    let not_found_message = "Output data not found";
                                    bot.send_message(message.chat.id, not_found_message).await?
                                }
                            },
                        }
                    },
                    Err(e) => bot.send_message(message.chat.id, e.to_string()).await?,
                }
            }
            Err(e) => bot.send_message(message.chat.id, e.to_string()).await?,
        },
    };

    Ok(())
}

fn decode_b64(data_mapping: HashMap<String, Base64Data>) -> anyhow::Result<Vec<u8>, String> {
    if let Some(data) = data_mapping.values().next() {
        Ok(data.clone().into()) // Convert the Base64Data instance to Vec<u8>
    } else {
        Err("No data found".to_string())
    }
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
                    ApiCallStatus::Completed => return Ok(Generate::Data(outputs)),
                    _ => {
                        println!("continue the loop");
                        sleep(Duration::from_secs(5));
                    }
                }
            }
            _ => {
                return Ok(Generate::Message(
                    "Could not parse the response".to_string(),
                ))
            }
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
