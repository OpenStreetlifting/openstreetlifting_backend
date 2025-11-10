use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Federation {
    pub federation_id: i32,
    pub name: String,
    pub rulebook_id: Option<i32>,
    pub country: Option<String>,
    pub abbreviation: Option<String>,
}
