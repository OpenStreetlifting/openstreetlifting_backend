use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum Movement {
    Muscleup,
    Pullup,
    Dips,
    Squat,
    #[default]
    Total,
}

impl Movement {
    pub fn as_column(&self) -> &'static str {
        match self {
            Self::Muscleup => "muscleup",
            Self::Pullup => "pullup",
            Self::Dips => "dips",
            Self::Squat => "squat",
            Self::Total => "total",
        }
    }
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct GlobalRankingFilter {
    #[serde(flatten)]
    pub pagination: super::common::PaginationParams,
    pub gender: Option<String>,
    pub country: Option<String>,
    #[serde(default)]
    pub movement: Movement,
}

impl GlobalRankingFilter {
    pub fn validate(&self) -> Result<(), String> {
        self.pagination.validate()?;

        if let Some(ref gender) = self.gender
            && gender != "M"
            && gender != "F"
        {
            return Err("gender must be 'M' or 'F'".to_string());
        }

        Ok(())
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct GlobalRankingEntry {
    pub rank: i64,
    pub athlete: AthleteInfo,
    pub ris: f64,
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
    pub bodyweight: Option<f64>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CompetitionInfo {
    pub competition_id: Uuid,
    pub name: String,
    pub date: Option<NaiveDate>,
}
