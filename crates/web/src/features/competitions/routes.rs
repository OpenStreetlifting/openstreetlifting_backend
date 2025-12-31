use axum::{
    middleware,
    routing::{delete, get, post, put},
    Router,
};
use storage::Database;

use super::handlers::{
    create_competition, delete_competition, get_competition, get_competition_detailed,
    list_competitions, list_competitions_detailed, update_competition,
};
use crate::middleware::auth::{require_auth, ApiKeys};

pub fn routes(api_keys: ApiKeys) -> Router<Database> {
    let protected = Router::new()
        .route("/", post(create_competition))
        .route("/:slug", put(update_competition))
        .route("/:slug", delete(delete_competition))
        .route_layer(middleware::from_fn_with_state(api_keys, require_auth));

    Router::new()
        .route("/", get(list_competitions))
        .route("/detailed", get(list_competitions_detailed))
        .route("/:slug", get(get_competition))
        .route("/:slug/detailed", get(get_competition_detailed))
        .merge(protected)
}
