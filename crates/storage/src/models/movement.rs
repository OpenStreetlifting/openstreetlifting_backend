use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Movement {
    pub movement_id: i32,
    pub name: String,
    pub display_order: i32,
}
