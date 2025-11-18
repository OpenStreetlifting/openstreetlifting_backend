use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Rulebook {
    pub rulebook_id: Uuid,
    pub name: Option<String>,
    pub url: Option<String>,
}
