use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Record {
    pub record_id: Uuid,
    pub record_type: String,
    pub category_id: Uuid,
    pub movement_name: String,
    pub athlete_id: Uuid,
    pub competition_id: Uuid,
    pub date_set: chrono::NaiveDate,
    pub weight: Decimal,
    pub gender: Option<String>,
}
