use std::io::Write;

use askama::Template;
use poem::{
    error::InternalServerError,
    get, handler, post,
    web::{Data, Form, Html, Path},
    Error,
};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct Chat {
    id: u32,
    name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Message {
    chat_id: u32,
    message: String,
}

#[derive(Clone, Template)]
#[template(path = "chat_page.html")]
struct ChatTemplate {
    id: u32,
    chats: Vec<Chat>,
    messages: Vec<String>,
}

#[derive(Clone, Template)]
#[template(path = "message.html")]
struct MessageTemplate {
    message: String,
}

#[derive(Clone, Template)]
#[template(path = "chat_window.html")]
struct ChatWindowTemplate {
    id: u32,
    messages: Vec<String>,
}

#[handler]
pub fn get_chat(id: Path<u32>) -> Result<Html<String>, Error> {
    std::fs::create_dir_all("messages").map_err(|e| InternalServerError(e))?;
    let mut chats = vec![];
    if let Ok(entries) = std::fs::read_dir("messages") {
        for entry in entries {
            if let Ok(entry) = entry {
                if !entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
                    continue;
                }
                if let Some(file_name) = entry.file_name().to_str() {
                    if let Ok(id) = file_name.replace(".txt", "").parse::<u32>() {
                        chats.push(Chat {
                            id,
                            name: format!("Chat {}", id),
                        });
                    }
                }
            }
        }
    }

    // read in the messages as json and then append the new message then write new json to file
    let file_name = format!("messages/{}.txt", id.0);
    let mut messages = vec![];
    if let Ok(mut file) = std::fs::File::open(file_name.clone()) {
        if let Ok(mut json) = serde_json::from_reader::<_, Vec<String>>(&mut file) {
            for message in json.drain(..) {
                messages.push(message);
            }
        }
    }

    let html = ChatTemplate {
        id: id.0,
        chats,
        messages,
    }
    .render()
    .unwrap_or_else(|e| format!("Failed to render template: {}", e));
    Ok(Html(html))
}

#[handler]
pub async fn send_message(
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
        messages.iter().map(|m| openai_rust::chat::Message {
            role: "user".to_string(),
            content: m.clone(),
        }).collect(),
    );

    let res = client.create_chat(args).await.unwrap();
    messages.push(res.choices[0].message.content.clone());
    
    let json = serde_json::to_string(&messages).map_err(|e| InternalServerError(e))?;
    file.write_all(json.as_bytes())
        .map_err(|e| InternalServerError(e))?;

    let html = ChatWindowTemplate {
        id: message.0.chat_id,
        messages,
    }
    .render()
    .unwrap_or_else(|e| format!("Failed to render template: {}", e));

    Ok(Html(html))
}

pub fn chat_routes() -> poem::Route {
    poem::Route::new().at("/:id", get(get_chat))
}

pub fn message_routes() -> poem::Route {
    poem::Route::new().at("", post(send_message))
}
