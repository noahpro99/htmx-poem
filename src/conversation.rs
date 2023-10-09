use std::io::Write;

use askama::Template;
use poem::{
    error::InternalServerError,
    get, handler, post,
    web::{Data, Form, Html, Path},
    Error, Route,
};
use sea_orm::{DatabaseConnection, EntityTrait, ModelTrait};
use serde::{Deserialize, Serialize};

use crate::entity::{chat, conversation, prelude::*};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Message {
    chat_id: u32,
    message: String,
}

#[derive(Clone, Template)]
#[template(path = "conversation_page.html")]
struct ChatTemplate {
    id: u32,
    conversations: Vec<conversation::Model>,
    chats: Vec<chat::Model>,
}

#[derive(Clone, Template)]
#[template(path = "chat.html")]
struct MessageTemplate {
    chat: chat::Model,
}

#[derive(Clone, Template)]
#[template(path = "conversation_window.html")]
struct ChatWindowTemplate {
    id: u32,
    chats: Vec<chat::Model>,
}

#[handler]
pub async fn get_conversation(id: Path<u32>, db: Data<&DatabaseConnection>) -> Result<Html<String>, Error> {
    dbg!(id.0);
    let conversation = Conversation::find_by_id(id.0 as i32)
        .one(db.0)
        .await
        .map_err(|e| InternalServerError(e))?;

    match conversation {
        Some(conversation) => {
            let chats = conversation
                .find_related(Chat)
                .all(db.0)
                .await
                .map_err(|e| InternalServerError(e))?;
            let html = ChatTemplate {
                id: id.0,
                conversations: vec![conversation],
                chats,
            }
            .render()
            .unwrap_or_else(|e| format!("Failed to render template: {}", e));
            Ok(Html(html))
        }
        None => {
            return Ok(Html("Conversation not found".to_string()));
        }
    }
}

#[handler]
pub fn send_message(
    message: Form<Message>,
    openai_key: Data<&String>,
) -> Result<Html<String>, Error> {
    std::thread::sleep(std::time::Duration::from_secs(1));
    let file_name = format!("messages/{}.txt", message.0.chat_id);
    std::fs::create_dir_all("messages").map_err(|e| InternalServerError(e))?;
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .open(file_name.clone())
        .map_err(|e| InternalServerError(e))?;

    // read in the messages as json and then append the new message then write new json to file
    let mut messages = vec![];
    if let Ok(mut file) = std::fs::File::open(file_name.clone()) {
        if let Ok(mut json) = serde_json::from_reader::<_, Vec<String>>(&mut file) {
            for message in json.drain(..) {
                messages.push(message);
            }
        }
    }
    messages.push(message.0.message.clone());

    let client = openai_rust::Client::new(&openai_key.clone());
    let args = openai_rust::chat::ChatArguments::new(
        "gpt-3.5-turbo",
        messages
            .iter()
            .map(|m| openai_rust::chat::Message {
                role: "user".to_string(),
                content: m.clone(),
            })
            .collect(),
    );

    // let res = client.create_chat(args).await.unwrap();
    // messages.push(res.choices[0].message.content.clone());

    let json = serde_json::to_string(&messages).map_err(|e| InternalServerError(e))?;
    file.write_all(json.as_bytes())
        .map_err(|e| InternalServerError(e))?;

    // let html = ChatWindowTemplate {
    // id: message.0.chat_id,
    // messages,
    // }
    // .render()
    // .unwrap_or_else(|e| format!("Failed to render template: {}", e));

    // Ok(Html(html))
    Ok(Html("".to_string()))
}

pub fn conversation_route() -> poem::Route {
    Route::new()
        .at("/:id", get_conversation)
        .at("/:id/send", post(send_message))
}
