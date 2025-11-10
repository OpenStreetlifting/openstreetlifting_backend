use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct CompetitionGroup {
    pub group_id: i32,
    pub competition_id: i32,
    pub category_id: i32,
    pub name: String,
    pub max_size: Option<i32>,
    pub created_at: chrono::NaiveDateTime,
}
