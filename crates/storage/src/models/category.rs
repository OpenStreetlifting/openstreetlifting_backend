use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Category {
    pub category_id: i32,
    pub name: String,
    pub gender: String,
    pub weight_class_min: Option<Decimal>,
    pub weight_class_max: Option<Decimal>,
}
