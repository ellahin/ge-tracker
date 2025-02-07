use axum::extract::State;

use crate::AppState;

pub async fn get(State(state): State<AppState>) -> String {
    let res = state.osrs.get_metrics_string();

    return res;
}
