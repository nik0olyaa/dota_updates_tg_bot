use crate::{json_part, LINK};
use log::{error, info};
use regex::Regex;
use teloxide::prelude::*;
use teloxide::types::ParseMode;
use teloxide::Bot;

/// Sends the first update.
///
/// This function initializes a Telegram bot from environment variables and sets up a REPL
/// (Read-Eval-Print Loop) using `teloxide::repl`. Incoming messages are handled asynchronously
/// by the `handle_message()` function. If there's an error during message handling, it logs the error message.
pub async fn send_first_upd() {
    let bot = Bot::from_env();
    teloxide::repl(bot, |bot: Bot, msg: Message| async move {
        if let Err(e) = handle_message(&bot, &msg).await {
            error!("Failed to send message: {}", e);
        }
        Ok(())
    })
    .await;
}

/// Handles an incoming message.
///
/// This asynchronous function handles incoming messages. It first retrieves events using
/// `json_part::read_page_to_json_str_events()` and processes the first event's body. It constructs
/// a message body containing the event headline and processed body. The constructed message is then
/// sent using `send_chunks()`. If there's an error during message handling, it returns an error message.
async fn handle_message(bot: &Bot, msg: &Message) -> Result<(), String> {
    let mut msg_msg = String::new();
    info!("Handling incoming message...");
    if let Ok(events) = json_part::read_page_to_json_str_events(LINK).await {
        info!("Retrieved events successfully.");
        if let Some(event) = events.first() {
            if let Some(body_str) = event.announcement_body.body.as_str() {
                let processed_body = process_body(body_str);
                msg_msg += &format!(
                    "_*To see more updates and news follow this [link](https://www.dota2.com/news?l=english)*_\n\n*{}*\n{}\n\n",
                    event.announcement_body.headline, processed_body
                );
                info!("Prepared message body for sending.");
            }
        }
    } else {
        error!("Failed to retrieve events.");
        return Ok(());
    }
    send_chunks(bot, msg.chat.id, &msg_msg)
        .await
        .map_err(|err| {
            error!("Failed to send message: {}", err);
            err.to_string()
        })?;
    info!("Message sent successfully.");
    Ok(())
}

/// Processes the body of an event announcement.
///
/// This function removes certain elements like tables, images, and YouTube video previews using
/// regular expressions. It replaces URLs with placeholders and modifies some formatting elements.
/// It returns the processed body text along with a vector containing the found URL fragments.
fn process_body(body_str: &str) -> String {
    let body = body_str.to_owned();

    let re_url = Regex::new(r"\[url=([^]]+)]([^\[]+)\[/url]").unwrap();
    let re_table = Regex::new(r"\[table\].*?\[\/table\]").unwrap();
    let re_img = Regex::new(r"\[img\].*?\[\\/img\]").unwrap();
    let re_preview = Regex::new(r"\[previewyoutube.*?\]").unwrap();

    let mut found_fragments = Vec::new();

    let removed_tables = re_table.replace_all(&body, "").as_ref().to_owned();
    let removed_img = re_img.replace_all(&removed_tables, "").as_ref().to_owned();
    let removed_preview = re_preview
        .replace_all(
            &removed_img,
            "(This update contains video. To watch the video, go to the official website.)",
        )
        .as_ref()
        .to_owned();

    let replaced_body = re_url.replace_all(&removed_preview, |captures: &regex::Captures| {
        let found_fragment = captures.get(0).unwrap().as_str();
        found_fragments.push(found_fragment.to_string());
        format!("SomeReplacement{}", found_fragments.len() - 1)
    });

    let mut modified_body = replaced_body.to_string();

    modified_body = modified_body.replace("[/h1]", "*");
    modified_body = modified_body.replace("[\\/h1]", "*");
    modified_body = modified_body.replace("[/h2]", "*");
    modified_body = modified_body.replace("[\\/h2]", "*");
    modified_body = modified_body.replace("[/h3]", "*");
    modified_body = modified_body.replace("[\\/h3]", "*");
    modified_body = modified_body.replace("[/h5]", "*");
    modified_body = modified_body.replace("[\\/h5]", "*");

    modified_body = modified_body.replace("[list]", "");
    modified_body = modified_body.replace("[/list]", "");
    modified_body = modified_body.replace("[*][b]", "🔸*");
    modified_body = modified_body.replace("[\\/b]", "*");
    modified_body = modified_body.replace("[*]", "📌");
    modified_body = modified_body.replace("[strike]", "~");
    modified_body = modified_body.replace("[\\/strike]", "~");
    modified_body = modified_body.replace("[\\/previewyoutube]", "");

    let special_chars = "_*()~`>#-|{}.!";
    let replaced_string: String = modified_body
        .chars()
        .map(|c| {
            if special_chars.contains(c) {
                format!("\\{}", c)
            } else {
                c.to_string()
            }
        })
        .collect();

    let body_with_links = restore_links(&replaced_string, &found_fragments);

    re_url.replace_all(&body_with_links, "[$2]($1)").to_string()
}

/// Restores the replaced URLs back into the processed text.
///
/// This function replaces the placeholders with the original URL fragments found during processing.
fn restore_links(replaced_text: &str, found_fragments: &[String]) -> String {
    let mut restored_text = replaced_text.to_string();
    found_fragments
        .iter()
        .enumerate()
        .for_each(|(index, fragment)| {
            let replacement = format!("SomeReplacement{}", index);
            restored_text = restored_text.replace(&replacement, fragment);
        });
    restored_text
}

/// Sends the message in chunks to avoid Telegram message size limitations.
///
/// This asynchronous function splits the message into chunks of 4000 characters each and sends
/// them individually. It uses Markdown V2 formatting for the messages. If there's an error during
/// message sending, it returns an error.
async fn send_chunks(
    bot: &Bot,
    chat_id: ChatId,
    msg: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let chunks = msg
        .chars()
        .collect::<Vec<_>>()
        .chunks(4000)
        .map(|chunk| chunk.iter().collect::<String>())
        .collect::<Vec<_>>();
    for chunk in chunks {
        bot.send_message(chat_id, &chunk)
            .parse_mode(ParseMode::MarkdownV2)
            .await?;
        info!("Chunk sent successfully.");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_body() {
        let input = "[url=https://www.dota2.com]Dota 2[/url]";
        let processed_body = process_body(input);

        assert_eq!(processed_body, "[Dota 2](https://www.dota2.com)");
    }

    #[test]
    fn test_restore_links() {
        let replaced_text = "SomeReplacement0";
        let found_fragments = vec!["[url=https://www.dota2.com]Dota 2[/url]".to_string()];
        let restored_text = restore_links(replaced_text, &found_fragments);

        assert_eq!(restored_text, found_fragments[0]);
    }
}
