use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct CompetitionParticipant {
    pub participant_id: i32,
    pub group_id: i32,
    pub athlete_id: i32,
    pub bodyweight: Option<Decimal>,
    pub rank: Option<i32>,
    pub is_disqualified: bool,
    pub created_at: chrono::NaiveDateTime,
    pub disqualified_reason: Option<String>,
    pub ris_score: Option<Decimal>,
}
