use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Federation {
    pub federation_id: Uuid,
    pub name: String,
    pub rulebook_id: Option<Uuid>,
    pub country: Option<String>,
    pub abbreviation: Option<String>,
}
