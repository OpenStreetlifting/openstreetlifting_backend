use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct CompetitionGroup {
    pub group_id: Uuid,
    pub competition_id: Uuid,
    pub category_id: Uuid,
    pub name: String,
    pub max_size: Option<i32>,
    pub created_at: chrono::NaiveDateTime,
}
