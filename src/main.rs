mod templates;

use askama::Template;
use poem::{
    get, handler,
    listener::TcpListener,
    post,
    web::{Form, Html, Path},
    Route, Server,
};

#[derive(Clone)]
pub struct NeededReview {
    id: u32,
    title: String,
}

// age post
#[derive(serde::Deserialize)]
struct AgePost {
    age: u8,
}

#[handler]
fn age(data: Form<AgePost>) -> String {
    // do a sleep to simulate a long running task
    std::thread::sleep(std::time::Duration::from_secs(2));
    format!("Your age is {}", data.age)
}

#[handler]
fn hello(Path(name): Path<String>) -> Html<String> {
    let html = templates::HelloTemplate { name }
        .render()
        .unwrap_or_else(|e| format!("Failed to render template: {}", e));
    Html(html)
}

#[handler]
fn needed_reviews() -> Html<String> {
    let needed_review_example = NeededReview {
        title: "title".to_string(),
        id: 55,
    };

    let needed_reviews = vec![needed_review_example.clone(), needed_review_example.clone()];
    let html = templates::NeededReviewsTemplate { needed_reviews }
        .render()
        .unwrap_or_else(|e| format!("Failed to render template: {}", e));
    Html(html)
}

pub struct Employee {
    id: u32,
    name: String,
}

#[handler]
fn needed_review(id: Path<u32>) -> Html<String> {
    let employees = vec![Employee { id: 1, name: "name".to_string() }];
    let html = templates::NeededReviewTemplate { id: id.0, employees }
        .render()
        .unwrap_or_else(|e| format!("Failed to render template: {}", e));
    Html(html)
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let app = Route::new()
        .at("/age", post(age))
        .at("/needed_reviews", get(needed_reviews))

        .at("/hello/:name", get(hello));

    Server::new(TcpListener::bind("127.0.0.1:3000"))
        .run(app)
        .await
}
