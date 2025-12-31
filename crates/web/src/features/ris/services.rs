use rust_decimal::Decimal;
use sqlx::PgPool;
use storage::{
    dto::ris::RisFormulaResponse,
    error::Result,
    models::RisScoreHistory,
    repository::ris::RisRepository,
    services::ris_computation,
};
use uuid::Uuid;

use super::handlers::formula_to_response;

/// List all RIS formula versions
pub async fn list_ris_formulas(pool: &PgPool) -> Result<Vec<RisFormulaResponse>> {
    let repo = RisRepository::new(pool);
    let formulas = repo.list_all_formulas().await?;

    Ok(formulas
        .into_iter()
        .map(|f| formula_to_response(&f))
        .collect())
}

/// Get the current active RIS formula
pub async fn get_current_formula(pool: &PgPool) -> Result<RisFormulaResponse> {
    let repo = RisRepository::new(pool);
    let formula = repo.get_current_formula().await?;
    Ok(formula_to_response(&formula))
}

/// Get RIS formula for a specific year
pub async fn get_formula_by_year(pool: &PgPool, year: i32) -> Result<RisFormulaResponse> {
    let repo = RisRepository::new(pool);
    let formula = repo.get_formula_by_year(year).await?;
    Ok(formula_to_response(&formula))
}

/// Get participant RIS history with formula year mapping
pub async fn get_participant_ris_history(
    pool: &PgPool,
    participant_id: Uuid,
) -> Result<(Vec<RisScoreHistory>, std::collections::HashMap<Uuid, i32>)> {
    let repo = RisRepository::new(pool);
    let history = repo.get_participant_ris_history(participant_id).await?;

    let formulas = repo.list_all_formulas().await?;
    let formula_map: std::collections::HashMap<Uuid, i32> =
        formulas.into_iter().map(|f| (f.formula_id, f.year)).collect();

    Ok((history, formula_map))
}

/// Compute RIS score for given parameters
pub async fn compute_ris(
    pool: &PgPool,
    bodyweight: Decimal,
    total: Decimal,
    gender: &str,
    formula_year: Option<i32>,
) -> Result<(Decimal, i32)> {
    let repo = RisRepository::new(pool);
    let formula = if let Some(year) = formula_year {
        repo.get_formula_by_year(year).await?
    } else {
        repo.get_current_formula().await?
    };

    let ris_score = ris_computation::compute_ris(bodyweight, total, gender, &formula).await?;

    Ok((ris_score, formula.year))
}

/// Recompute all RIS scores
pub async fn recompute_all_ris(pool: &PgPool) -> Result<u64> {
    ris_computation::recompute_all_ris(pool, None).await
}
