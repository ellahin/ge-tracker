use askama::Template;
use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};

use crate::repo::data::osrs::HighAlchProfit;
use crate::AppState;

pub async fn get(State(state): State<AppState>) -> impl IntoResponse {
    let nr_price = match state.osrs.get_ge_one(&561_i64).await {
        Some(e) => match e.high {
            Some(e) => e,
            None => panic!("no nature ruin price"),
        },

        None => panic!("no nature ruin price"),
    };

    let profits = state.osrs.get_high_alch_profit().await;
    let template = IndexTemplate { profits, nr_price };
    HtmlTemplate(template)
}

#[derive(Template)]
#[template(path = "highalch.html")]
struct IndexTemplate {
    profits: Vec<HighAlchProfit>,
    nr_price: u128,
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
