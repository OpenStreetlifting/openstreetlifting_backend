use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

#[derive(Debug, Deserialize, IntoParams)]
pub struct GlobalRankingFilter {
    #[serde(flatten)]
    pub pagination: super::common::PaginationParams,
    pub gender: Option<String>,
    pub country: Option<String>,
    #[serde(default = "default_movement")]
    pub movement: String,
}

fn default_movement() -> String {
    "total".to_string()
}

impl GlobalRankingFilter {
    pub fn validate(&self) -> Result<(), String> {
        self.pagination.validate()?;

        if let Some(ref gender) = self.gender
            && gender != "M" && gender != "F" {
                return Err("gender must be 'M' or 'F'".to_string());
            }

        let valid_movements = ["muscleup", "pullup", "dips", "squat", "total"];
        if !valid_movements.contains(&self.movement.as_str()) {
            return Err(format!(
                "movement must be one of: {}",
                valid_movements.join(", ")
            ));
        }

        Ok(())
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct GlobalRankingEntry {
    pub rank: i64,
    pub athlete: AthleteInfo,
    pub total: f64,
    pub muscleup: f64,
    pub pullup: f64,
    pub dips: f64,
    pub squat: f64,
    pub competition: CompetitionInfo,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AthleteInfo {
    pub athlete_id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub slug: String,
    pub country: String,
    pub gender: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CompetitionInfo {
    pub competition_id: Uuid,
    pub name: String,
    pub date: Option<NaiveDate>,
}
