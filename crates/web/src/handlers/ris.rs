use actix_web::{HttpResponse, web};
use storage::{
    Database,
    dto::ris::{
        ComputeRisRequest, ComputeRisResponse, GenderConstants, RisConstants, RisFormulaResponse,
        RisScoreResponse,
    },
    models::RisFormulaVersion,
    repository::ris::RisRepository,
    services::ris_computation,
};
use uuid::Uuid;
use validator::Validate;

use crate::error::WebResult;

#[utoipa::path(
    get,
    path = "/api/ris/formulas",
    responses(
        (status = 200, description = "List all RIS formula versions", body = Vec<RisFormulaResponse>)
    ),
    tag = "ris"
)]
pub async fn list_ris_formulas(db: web::Data<Database>) -> WebResult<HttpResponse> {
    let repo = RisRepository::new(db.pool());
    let formulas = repo.list_all_formulas().await?;

    let response: Vec<RisFormulaResponse> = formulas
        .into_iter()
        .map(|f| formula_to_response(&f))
        .collect();

    Ok(HttpResponse::Ok().json(response))
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
pub async fn get_current_formula(db: web::Data<Database>) -> WebResult<HttpResponse> {
    let repo = RisRepository::new(db.pool());
    let formula = repo.get_current_formula().await?;

    Ok(HttpResponse::Ok().json(formula_to_response(&formula)))
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
    db: web::Data<Database>,
    path: web::Path<i32>,
) -> WebResult<HttpResponse> {
    let year = path.into_inner();
    let repo = RisRepository::new(db.pool());
    let formula = repo.get_formula_by_year(year).await?;

    Ok(HttpResponse::Ok().json(formula_to_response(&formula)))
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
    db: web::Data<Database>,
    path: web::Path<Uuid>,
) -> WebResult<HttpResponse> {
    let participant_id = path.into_inner();
    let repo = RisRepository::new(db.pool());
    let history = repo.get_participant_ris_history(participant_id).await?;

    let formulas = repo.list_all_formulas().await?;
    let formula_map: std::collections::HashMap<Uuid, i32> = formulas
        .into_iter()
        .map(|f| (f.formula_id, f.year))
        .collect();

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

    Ok(HttpResponse::Ok().json(response))
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
    db: web::Data<Database>,
    payload: web::Json<ComputeRisRequest>,
) -> WebResult<HttpResponse> {
    payload.validate()?;

    let repo = RisRepository::new(db.pool());
    let formula = if let Some(year) = payload.formula_year {
        repo.get_formula_by_year(year).await?
    } else {
        repo.get_current_formula().await?
    };

    let ris_score =
        ris_computation::compute_ris(payload.bodyweight, payload.total, &payload.gender, &formula)
            .await?;

    let response = ComputeRisResponse {
        ris_score,
        formula_year: formula.year,
    };

    Ok(HttpResponse::Ok().json(response))
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
pub async fn recompute_all_ris(db: web::Data<Database>) -> WebResult<HttpResponse> {
    let count = ris_computation::recompute_all_ris(db.pool(), None).await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "recomputed_count": count,
        "message": format!("Successfully recomputed RIS for {} participants", count)
    })))
}

fn formula_to_response(formula: &RisFormulaVersion) -> RisFormulaResponse {
    RisFormulaResponse {
        formula_id: formula.formula_id,
        year: formula.year,
        is_current: formula.is_current,
        effective_from: formula.effective_from,
        effective_until: formula.effective_until,
        constants: RisConstants {
            men: GenderConstants {
                a: formula.men_a,
                k: formula.men_k,
                b: formula.men_b,
                v: formula.men_v,
                q: formula.men_q,
            },
            women: GenderConstants {
                a: formula.women_a,
                k: formula.women_k,
                b: formula.women_b,
                v: formula.women_v,
                q: formula.women_q,
            },
        },
    }
}
