mod chats;

use poem::{
    listener::TcpListener, Route, Server, EndpointExt,
};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    dotenv::dotenv().ok();
    env_logger::init();

    let openai_key = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set");

    let app = Route::new()
        .nest("/chat", chats::chat_routes())
        .nest("/send-message", chats::message_routes())
        .data(openai_key);

    Server::new(TcpListener::bind("127.0.0.1:3000"))
        .run(app)
        .await
}
