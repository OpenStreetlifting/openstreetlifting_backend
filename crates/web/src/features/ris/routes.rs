use axum::{
    Router,
    routing::{get, post},
};
use storage::Database;

use super::handlers::{
    compute_ris, get_current_formula, get_formula_by_year, get_participant_ris_history,
    list_ris_formulas, recompute_all_ris,
};
use crate::middleware::auth::ApiKeys;

pub fn routes(_api_keys: ApiKeys) -> Router<Database> {
    Router::new()
        .route("/formulas", get(list_ris_formulas))
        .route("/formulas/current", get(get_current_formula))
        .route("/formulas/:year", get(get_formula_by_year))
        .route("/compute", post(compute_ris))
}

pub fn participant_routes() -> Router<Database> {
    Router::new().route(
        "/:participant_id/ris-history",
        get(get_participant_ris_history),
    )
}

pub fn admin_routes() -> Router<Database> {
    Router::new().route("/recompute-all", post(recompute_all_ris))
}
