use askama::Template;
use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};

use crate::repo::data::osrs::CraftingItemProfit;
use crate::AppState;

pub async fn get(State(state): State<AppState>) -> impl IntoResponse {
    let crafting = state.osrs.get_crafting_profit();
    let template = IndexTemplate {
        crafting,
        stringnull: stringnull,
        pretty: pretty_int,
    };
    HtmlTemplate(template)
}

#[derive(Template)]
#[template(path = "crafting.html")]
struct IndexTemplate {
    crafting: Vec<CraftingItemProfit>,
    pretty: fn(i: &i64) -> String,
    stringnull: fn(i: &Option<String>) -> String,
}

fn pretty_int(i: &i64) -> String {
    let mut s = String::new();
    let i_str = i.to_string();
    let a = i_str.chars().rev().enumerate();
    for (idx, val) in a {
        if idx != 0 && idx % 3 == 0 {
            s.insert(0, ',');
        }
        s.insert(0, val);
    }
    s
}

fn stringnull(s: &Option<String>) -> String {
    match s {
        Some(e) => e.to_string(),
        None => "-".to_string(),
    }
}

/// A wrapper type that we'll use to encapsulate HTML parsed by askama into valid HTML for axum to serve.
struct HtmlTemplate<T>(T);

/// Allows us to convert Askama HTML templates into valid HTML for axum to serve in the response.
impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> Response {
        // Attempt to render the template with askama
        match self.0.render() {
            // If we're able to successfully parse and aggregate the template, serve it
            Ok(html) => Html(html).into_response(),
            // If we're not, return an error or some bit of fallback HTML
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render template. Error: {}", err),
            )
                .into_response(),
        }
    }
}
