use askama::Template;
use poem::{
    get, handler, patch, post,
    web::{Form, Html, Path, Query},
    Error, put,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct User {
    first_name: String,
    last_name: String,
    email: String,
}

#[derive(Clone, Template)]
#[template(path = "settings_page.html")]
struct SettingsTemplate {
    editing: bool,
    user: User,
}

#[derive(Clone, Template)]
#[template(path = "user_settings.html")]
struct UserSettingsTemplate {
    editing: bool,
    user: User,
}

#[derive(Deserialize)]
pub struct GetSettingQuery {
    editing: Option<bool>,
}

#[handler]
pub fn get_settings(query: Query<GetSettingQuery>) -> Result<Html<String>, Error> {
    let user = User {
        first_name: "John".to_string(),
        last_name: "Doe".to_string(),
        email: "something@gmail.com".to_string(),
    };
    match query.editing {
        Some(editing) => {
            let template = UserSettingsTemplate { user, editing };
            Ok(Html(template.render().unwrap()))
        }
        None => {
            let template = SettingsTemplate {
                user,
                editing: false,
            };
            Ok(Html(template.render().unwrap()))
        }
    }
}

#[handler]
pub fn put_user_settings(Form(user): Form<User>) -> Result<Html<String>, Error> {
    println!("User updated to: {:?}", user);
    let template = UserSettingsTemplate {
        user,
        editing: false,
    };
    Ok(Html(template.render().unwrap()))
}

pub fn route() -> poem::Route {
    poem::Route::new().at("", put(put_user_settings).get(get_settings))
}
