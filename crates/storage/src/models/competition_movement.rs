use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct CompetitionMovement {
    pub competition_id: Uuid,
    pub movement_name: String,
    pub is_required: bool,
    pub display_order: Option<i32>,
}
