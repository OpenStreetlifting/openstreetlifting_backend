use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Rulebook {
    pub rulebook_id: i32,
    pub name: Option<String>,
    pub url: Option<String>,
}
