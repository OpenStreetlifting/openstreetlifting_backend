use axum::{
    Json,
    extract::{Path, State},
    response::{IntoResponse, Response},
};
use storage::{
    Database,
    dto::ris::{ComputeRisRequest, ComputeRisResponse, RisFormulaResponse, RisScoreResponse},
};
use uuid::Uuid;
use validator::Validate;

use crate::error::WebError;

use super::services;

#[utoipa::path(
    get,
    path = "/api/ris/formulas",
    responses(
        (status = 200, description = "List all RIS formula versions", body = Vec<RisFormulaResponse>)
    ),
    tag = "ris"
)]
pub async fn list_ris_formulas(State(db): State<Database>) -> Result<Response, WebError> {
    let response = services::list_ris_formulas(db.pool()).await?;

    Ok(Json(response).into_response())
}

#[utoipa::path(
    get,
    path = "/api/ris/formulas/current",
    responses(
        (status = 200, description = "Get the current active RIS formula", body = RisFormulaResponse),
        (status = 404, description = "No current formula found")
    ),
    tag = "ris"
)]
pub async fn get_current_formula(State(db): State<Database>) -> Result<Response, WebError> {
    let formula = services::get_current_formula(db.pool()).await?;

    Ok(Json(formula).into_response())
}

#[utoipa::path(
    get,
    path = "/api/ris/formulas/{year}",
    params(
        ("year" = i32, Path, description = "Formula year")
    ),
    responses(
        (status = 200, description = "RIS formula for specified year", body = RisFormulaResponse),
        (status = 404, description = "Formula not found for this year")
    ),
    tag = "ris"
)]
pub async fn get_formula_by_year(
    State(db): State<Database>,
    Path(year): Path<i32>,
) -> Result<Response, WebError> {
    let formula = services::get_formula_by_year(db.pool(), year).await?;

    Ok(Json(formula).into_response())
}

#[utoipa::path(
    get,
    path = "/api/participants/{participant_id}/ris-history",
    params(
        ("participant_id" = Uuid, Path, description = "Participant ID")
    ),
    responses(
        (status = 200, description = "RIS score history for participant", body = Vec<RisScoreResponse>)
    ),
    tag = "ris"
)]
pub async fn get_participant_ris_history(
    State(db): State<Database>,
    Path(participant_id): Path<Uuid>,
) -> Result<Response, WebError> {
    let (history, formula_map) =
        services::get_participant_ris_history(db.pool(), participant_id).await?;

    let response: Vec<RisScoreResponse> = history
        .into_iter()
        .map(|h| RisScoreResponse {
            formula_year: *formula_map.get(&h.formula_id).unwrap_or(&2025),
            ris_score: h.ris_score,
            bodyweight: h.bodyweight,
            total_weight: h.total_weight,
            computed_at: h.computed_at,
        })
        .collect();

    Ok(Json(response).into_response())
}

#[utoipa::path(
    post,
    path = "/api/ris/compute",
    request_body = ComputeRisRequest,
    responses(
        (status = 200, description = "RIS computed successfully", body = ComputeRisResponse),
        (status = 400, description = "Invalid request")
    ),
    tag = "ris"
)]
pub async fn compute_ris(
    State(db): State<Database>,
    Json(payload): Json<ComputeRisRequest>,
) -> Result<Response, WebError> {
    payload.validate()?;

    let (ris_score, formula_year) = services::compute_ris(
        db.pool(),
        payload.bodyweight,
        payload.total,
        &payload.gender,
        payload.formula_year,
    )
    .await?;

    let response = ComputeRisResponse {
        ris_score,
        formula_year,
    };

    Ok(Json(response).into_response())
}

#[utoipa::path(
    post,
    path = "/api/admin/ris/recompute-all",
    responses(
        (status = 200, description = "RIS scores recomputed successfully"),
        (status = 500, description = "Recomputation failed")
    ),
    tag = "ris"
)]
pub async fn recompute_all_ris(State(db): State<Database>) -> Result<Response, WebError> {
    let count = services::recompute_all_ris(db.pool()).await?;

    Ok(Json(serde_json::json!({
        "recomputed_count": count,
        "message": format!("Successfully recomputed RIS for {} participants", count)
    }))
    .into_response())
}
