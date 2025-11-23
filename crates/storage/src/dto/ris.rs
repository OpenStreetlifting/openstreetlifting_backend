use chrono::{NaiveDate, NaiveDateTime};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RisScoreResponse {
    pub formula_year: i32,
    pub ris_score: Decimal,
    pub bodyweight: Decimal,
    pub total_weight: Decimal,
    pub computed_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RisHistoryResponse {
    pub participant_id: Uuid,
    pub athlete_name: String,
    pub current_ris: Decimal,
    pub historical_scores: Vec<RisScoreResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RisFormulaResponse {
    pub formula_id: Uuid,
    pub year: i32,
    pub is_current: bool,
    pub effective_from: NaiveDate,
    pub effective_until: Option<NaiveDate>,
    pub constants: RisConstants,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RisConstants {
    pub men: GenderConstants,
    pub women: GenderConstants,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GenderConstants {
    pub a: Decimal,
    pub k: Decimal,
    pub b: Decimal,
    pub v: Decimal,
    pub q: Decimal,
}

#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
pub struct ComputeRisRequest {
    pub bodyweight: Decimal,
    pub total: Decimal,
    pub gender: String,
    pub formula_year: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ComputeRisResponse {
    pub ris_score: Decimal,
    pub formula_year: i32,
}
