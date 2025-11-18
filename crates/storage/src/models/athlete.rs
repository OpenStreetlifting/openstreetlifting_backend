use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Athlete {
    pub athlete_id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub gender: String,
    pub created_at: chrono::NaiveDateTime,
    pub nationality: Option<String>,
    pub country: String,
    pub profile_picture_url: Option<String>,
}
