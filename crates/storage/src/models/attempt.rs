use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Attempt {
    pub attempt_id: i32,
    pub lift_id: i32,
    pub attempt_number: i16,
    pub weight: Decimal,
    pub is_successful: bool,
    pub passing_judges: Option<i16>,
    pub no_rep_reason: Option<String>,
    pub created_at: chrono::NaiveDateTime,
    pub created_by: Option<String>,
}
