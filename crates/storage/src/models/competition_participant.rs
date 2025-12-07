use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct CompetitionParticipant {
    pub competition_id: Uuid,
    pub category_id: Uuid,
    pub athlete_id: Uuid,
    pub bodyweight: Option<Decimal>,
    pub rank: Option<i32>,
    pub is_disqualified: bool,
    pub created_at: Option<chrono::NaiveDateTime>,
    pub disqualified_reason: Option<String>,
    pub ris_score: Option<Decimal>,
}
