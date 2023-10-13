
use askama::Template;
use poem::{
    error::InternalServerError,
    get, handler, post,
    web::{Data, Form, Html, Path},
    Error, Route,
};
use sea_orm::{DatabaseConnection, EntityTrait, ModelTrait};
use serde::{Deserialize, Serialize};
use crate::entity::{chat, prelude::*};

#[derive(Clone, Template)]
#[template(path = "index.html")]
struct IndexTemplate {

}

#[handler]
pub async fn get_index(db: Data<&DatabaseConnection>) -> Result<Html<String>, Error> {
    let html = IndexTemplate {}
        .render()
        .map_err(|e| InternalServerError(e))?;
    Ok(Html(html))
}

pub fn route() -> poem::Route {
    Route::new()
        .at("/", get_index)
}
