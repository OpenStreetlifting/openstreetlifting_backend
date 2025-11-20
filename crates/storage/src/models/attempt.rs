use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Attempt {
    pub group_id: Uuid,
    pub athlete_id: Uuid,
    pub movement_name: String,
    pub attempt_number: i16,
    pub weight: Decimal,
    pub is_successful: bool,
    pub passing_judges: Option<i16>,
    pub no_rep_reason: Option<String>,
    pub created_at: chrono::NaiveDateTime,
    pub created_by: Option<String>,
}
