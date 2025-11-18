use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct AthleteSocial {
    pub athlete_social_id: Uuid,
    pub athlete_id: Uuid,
    pub social_id: Uuid,
    pub handle: String,
}
