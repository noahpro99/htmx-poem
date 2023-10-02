use askama::Template;

use crate::{NeededReview, Employee};

#[derive(Template)]
#[template(path = "hello.html")]
pub struct HelloTemplate {
    pub name: String,
}

#[derive(Template)]
#[template(path = "needed_reviews.html")]
pub struct NeededReviewsTemplate {
    pub needed_reviews: Vec<NeededReview>,
}

#[derive(Template)]
#[template(path = "needed_review.html")]
pub struct NeededReviewTemplate {
    pub id: u32,
    pub employees: Vec<Employee>,
}
