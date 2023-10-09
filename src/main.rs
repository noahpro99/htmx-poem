mod conversation;
mod entity;
mod settings;

use poem::{listener::TcpListener, EndpointExt, Route, Server};
use sea_orm::{Database, DatabaseConnection};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    dotenv::dotenv().ok();
    env_logger::init();

    let openai_key = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set");

    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not set");
    let db: DatabaseConnection = Database::connect(db_url)
        .await
        .expect("Failed to connect to database");

    let app = Route::new()
        .nest("/conversation", conversation::conversation_route())
        .nest("/settings", settings::route())
        .data(db)
        .data(openai_key);

    let port = std::env::var("PORT").unwrap_or("3000".to_string());
    Server::new(TcpListener::bind(format!("0.0.0.0:{}", port)))
        .run(app)
        .await
}
