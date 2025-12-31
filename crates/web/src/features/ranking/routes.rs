use axum::{Router, routing::get};
use storage::Database;

use super::handlers::get_global_ranking;

pub fn routes() -> Router<Database> {
    Router::new().route("/global", get(get_global_ranking))
}
