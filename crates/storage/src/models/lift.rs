use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Lift {
    pub lift_id: i32,
    pub participant_id: i32,
    pub movement_id: i32,
    pub max_weight: Decimal,
    pub equipment_setting: Option<String>,
    pub updated_at: Option<chrono::NaiveDateTime>,
}
