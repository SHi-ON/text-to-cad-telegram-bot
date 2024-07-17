use bytes::Bytes;
use kittycad::types::base64::Base64Data;
use kittycad::types::{ApiCallStatus, AsyncApiCallOutput, TextToCad};
use log::info;
use std::collections::HashMap;
use std::thread::sleep;
use std::time::Duration;
use teloxide::{prelude::*, types::InputFile, utils::command::BotCommands};
use uuid::Uuid;

const STICKER_FILE_ID: &str =
    "CAACAgIAAxkBAAEspCVmmAOyoiYGIgXTWhY8HbJ0XBCcngACTgIAAladvQow_mttgTIDbzUE"; // Duck <3

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
    #[command(
        description = "Generates a 3D model for the given prompt. e.g. /generate Thor's hammer"
    )]
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
            if prompt.is_empty() {
                bot.send_message(
                    message.chat.id,
                    "Empty prompt, explain your thinking in a few words",
                )
                .await?;
                return Ok(());
            }
            match generate_cad_model(prompt).await {
                Ok(result) => {
                    bot.send_message(message.chat.id, "Generating...").await?;
                    // Send the pending sticker
                    let sticker_file = InputFile::file_id(STICKER_FILE_ID);
                    let pending_message = bot.send_animation(message.chat.id, sticker_file).await?;
                    match generate(result.id).await {
                        Ok(generate_output) => match generate_output {
                            Generate::Message(status_message) => {
                                bot.delete_message(message.chat.id, pending_message.id)
                                    .await?;
                                bot.send_message(message.chat.id, status_message).await?
                            }
                            Generate::Data(data) => match data {
                                Some(data_mapping) => match decode_base64(data_mapping) {
                                    Ok(model_file) => {
                                        bot.delete_message(message.chat.id, pending_message.id)
                                            .await?;
                                        bot.send_document(message.chat.id, model_file).await?
                                    }
                                    Err(e) => {
                                        bot.delete_message(message.chat.id, pending_message.id)
                                            .await?;
                                        bot.send_message(
                                            message.chat.id,
                                            format!("Error while decoding: {}", e),
                                        )
                                        .await?
                                    }
                                },
                                None => {
                                    bot.delete_message(message.chat.id, pending_message.id)
                                        .await?;
                                    let not_found_message = "Output data not found";
                                    bot.send_message(message.chat.id, not_found_message).await?
                                }
                            },
                        },
                        Err(e) => {
                            bot.delete_message(message.chat.id, pending_message.id)
                                .await?;
                            bot.send_message(message.chat.id, e.to_string()).await?
                        }
                    }
                }
                Err(e) => bot.send_message(message.chat.id, e.to_string()).await?,
            }
        }
    };

    Ok(())
}

fn decode_base64(data_mapping: HashMap<String, Base64Data>) -> anyhow::Result<InputFile, String> {
    for (model_file_path, model_data) in data_mapping {
        if let Some(data) = Some(&model_data) {
            let model_bytes = &data.0;
            let model_file =
                InputFile::memory(Bytes::copy_from_slice(model_bytes)).file_name(model_file_path);
            return Ok(model_file);
        }
    }
    Err("No data found".to_string())
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
