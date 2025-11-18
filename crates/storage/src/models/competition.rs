use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Competition {
    pub competition_id: Uuid,
    pub name: String,
    pub created_at: chrono::NaiveDateTime,
    pub slug: String,
    pub status: String,
    pub federation_id: Uuid,
    pub venue: Option<String>,
    pub city: Option<String>,
    pub country: Option<String>,
    pub start_date: chrono::NaiveDate,
    pub end_date: chrono::NaiveDate,
    pub number_of_judge: Option<i16>,
}
