use std::io::Write;

use crate::entity::{chat, conversation, prelude::*, sea_orm_active_enums::ChatRole};
use askama::Template;
use poem::{
    error::InternalServerError,
    get, handler, post,
    web::{Data, Form, Html, Path},
    Error, Route,
};
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, ModelTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NewMessage {
    conversation_id: Option<u32>,
    message: String,
}

#[derive(Clone, Template)]
#[template(path = "conversation_page.html")]
struct ChatTemplate {
    id: Option<u32>,
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
    id: Option<u32>,
    chats: Vec<chat::Model>,
}

#[handler]
pub async fn get_conversation(
    id: Path<u32>,
    db: Data<&DatabaseConnection>,
) -> Result<Html<String>, Error> {
    let conversation = Conversation::find_by_id(id.0 as i32)
        .one(db.0)
        .await
        .map_err(|e| InternalServerError(e))?;
    let conversations = Conversation::find()
        .all(db.0)
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
                id: Some(id.0),
                conversations,
                chats,
            }
            .render()
            .unwrap_or_else(|e| format!("Failed to render template: {}", e));
            Ok(Html(html))
        }
        None => {
            let html = ChatTemplate {
                id: None,
                conversations,
                chats: vec![],
            }
            .render()
            .unwrap_or_else(|e| format!("Failed to render template: {}", e));
            Ok(Html(html))
        }
    }
}

impl ToString for ChatRole {
    fn to_string(&self) -> String {
        match self {
            ChatRole::Assistant => "assistant".to_string(),
            ChatRole::User => "user".to_string(),
        }
    }
}

async fn new_conv(db: &DatabaseConnection) -> Result<conversation::Model, Error> {
    let new_conversation = conversation::ActiveModel {
        title: Set("".to_string()),
        ..Default::default()
    };
    new_conversation
        .insert(db)
        .await
        .map_err(|e| InternalServerError(e))
}

#[handler]
pub async fn send_message(
    new_message: Form<NewMessage>,
    db: Data<&DatabaseConnection>,
    openai_key: Data<&String>,
) -> Result<Html<String>, Error> {
    let conversation = match new_message.conversation_id {
        Some(id) => Conversation::find_by_id(id as i32)
            .one(db.0)
            .await
            .map_err(|e| InternalServerError(e))?
            .unwrap_or(new_conv(&db.0).await?),
        None => new_conv(&db.0).await?,
    };

    // patch the conversation with an updated title to either the first message or the id of the conversation
    let first_chat = conversation
        .find_related(Chat)
        .one(db.0)
        .await
        .map_err(|e| InternalServerError(e))?;
    let title = match first_chat {
        Some(chat) => chat.content,
        None => conversation.id.to_string(),
    };
    let mut new_conversation: conversation::ActiveModel = conversation.into();
    new_conversation.title = Set(title);
    let conversation = new_conversation
        .update(db.0)
        .await
        .map_err(|e| InternalServerError(e))?;

    let mut chats = conversation
        .find_related(Chat)
        .all(db.0)
        .await
        .map_err(|e| InternalServerError(e))?;

    let new_user_chat = chat::ActiveModel {
        role: Set(Some(ChatRole::User)),
        content: Set(new_message.message.clone()),
        conversation_id: Set(conversation.id),
        ..Default::default()
    };

    let user_chat = new_user_chat
        .insert(db.0)
        .await
        .map_err(|e| InternalServerError(e))?;

    chats.push(user_chat);

    let messages = chats
        .iter()
        .map(|c| openai_rust::chat::Message {
            role: c.clone().role.unwrap().to_string(),
            content: c.clone().content,
        })
        .collect();
    let client = openai_rust::Client::new(&openai_key.clone());
    let args = openai_rust::chat::ChatArguments::new("gpt-3.5-turbo", messages);

    let res = client.create_chat(args).await.unwrap();

    let new_assistant_chat = chat::ActiveModel {
        role: Set(Some(ChatRole::Assistant)),
        content: Set(res.choices[0].message.content.clone()),
        conversation_id: Set(conversation.id),
        ..Default::default()
    };

    let assistant_chat = new_assistant_chat
        .insert(db.0)
        .await
        .map_err(|e| InternalServerError(e))?;

    chats.push(assistant_chat);

    let html = ChatWindowTemplate {
        id: Some(conversation.id as u32),
        chats,
    }
    .render()
    .unwrap_or_else(|e| format!("Failed to render template: {}", e));

    Ok(Html(html))
}

#[handler]
pub async fn get_blank_conversation(db: Data<&DatabaseConnection>) -> Result<Html<String>, Error> {
    let conversations = Conversation::find()
        .all(db.0)
        .await
        .map_err(|e| InternalServerError(e))?;
    let html = ChatTemplate {
        id: None,
        conversations,
        chats: vec![],
    }
    .render()
    .unwrap_or_else(|e| format!("Failed to render template: {}", e));
    Ok(Html(html))
}

pub fn route() -> poem::Route {
    Route::new()
        .at("/", get_blank_conversation)
        .at("/:id", get_conversation)
        .at("/:id/send", post(send_message))
}
