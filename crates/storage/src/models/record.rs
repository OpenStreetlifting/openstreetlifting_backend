use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Record {
    pub record_id: i32,
    pub record_type: String,
    pub category_id: i32,
    pub movement_id: i32,
    pub athlete_id: i32,
    pub competition_id: i32,
    pub date_set: chrono::NaiveDate,
    pub weight: Decimal,
    pub gender: Option<String>,
}
