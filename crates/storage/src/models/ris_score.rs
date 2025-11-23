use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

/// Historical RIS score for a competition participant
///
/// Stores computed RIS scores with version history, allowing us to track
/// how a performance would be rated under different formula versions.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct RisScoreHistory {
    pub ris_score_id: Uuid,
    pub participant_id: Uuid,
    pub formula_id: Uuid,
    pub ris_score: Decimal,
    pub bodyweight: Decimal,
    pub total_weight: Decimal,
    pub computed_at: NaiveDateTime,
}
