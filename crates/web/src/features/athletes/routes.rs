use axum::{
    middleware,
    routing::{delete, get, post, put},
    Router,
};
use storage::Database;

use super::handlers::{
    create_athlete, delete_athlete, get_athlete, get_athlete_detailed, list_athletes,
    update_athlete,
};
use crate::middleware::auth::{require_auth, ApiKeys};

pub fn routes(api_keys: ApiKeys) -> Router<Database> {
    let protected = Router::new()
        .route("/", post(create_athlete))
        .route("/:slug", put(update_athlete))
        .route("/:slug", delete(delete_athlete))
        .route_layer(middleware::from_fn_with_state(api_keys, require_auth));

    Router::new()
        .route("/", get(list_athletes))
        .route("/:slug", get(get_athlete))
        .route("/:slug/detailed", get(get_athlete_detailed))
        .merge(protected)
}
