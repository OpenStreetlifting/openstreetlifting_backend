use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Lift {
    pub lift_id: Uuid,
    pub participant_id: Uuid,
    pub movement_id: Uuid,
    pub max_weight: Decimal,
    pub equipment_setting: Option<String>,
    pub updated_at: Option<chrono::NaiveDateTime>,
}
