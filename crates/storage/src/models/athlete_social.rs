use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct AthleteSocial {
    pub athlete_social_id: i32,
    pub athlete_id: i32,
    pub social_id: i32,
    pub handle: String,
}
