use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

/// Response containing basic athlete information
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AthleteResponse {
    pub athlete_id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub slug: String,
    pub gender: String,
    pub nationality: Option<String>,
    pub country: String,
    pub profile_picture_url: Option<String>,
    pub created_at: NaiveDateTime,
}

/// Detailed athlete response with competition history
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AthleteDetailResponse {
    pub athlete_id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub slug: String,
    pub gender: String,
    pub nationality: Option<String>,
    pub country: String,
    pub profile_picture_url: Option<String>,
    pub created_at: NaiveDateTime,
    pub competitions: Vec<AthleteCompetitionSummary>,
    pub personal_records: Vec<PersonalRecord>,
    pub total_competitions: i64,
}

/// Summary of athlete's performance in a competition
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AthleteCompetitionSummary {
    pub competition_id: Uuid,
    pub competition_name: String,
    pub competition_slug: String,
    pub competition_date: Option<chrono::NaiveDate>,
    pub category_name: String,
    pub rank: Option<i32>,
    pub total: rust_decimal::Decimal,
    pub ris_score: Option<rust_decimal::Decimal>,
    pub is_disqualified: bool,
}

/// Personal record for a specific movement
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PersonalRecord {
    pub movement_name: String,
    pub max_weight: rust_decimal::Decimal,
    pub competition_name: String,
    pub competition_slug: String,
    pub date: Option<chrono::NaiveDate>,
}

/// Request payload for creating a new athlete
#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct CreateAthleteRequest {
    #[validate(length(
        min = 1,
        max = 255,
        message = "First name must be between 1 and 255 characters"
    ))]
    pub first_name: String,

    #[validate(length(
        min = 1,
        max = 255,
        message = "Last name must be between 1 and 255 characters"
    ))]
    pub last_name: String,

    #[validate(custom(function = "validate_gender"))]
    pub gender: String,

    #[validate(length(max = 255))]
    pub nationality: Option<String>,

    #[validate(length(min = 1, max = 255, message = "Country is required"))]
    pub country: String,

    #[validate(url)]
    #[validate(length(max = 500))]
    pub profile_picture_url: Option<String>,
}

/// Request payload for updating an existing athlete
#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct UpdateAthleteRequest {
    #[validate(length(min = 1, max = 255))]
    pub first_name: Option<String>,

    #[validate(length(min = 1, max = 255))]
    pub last_name: Option<String>,

    #[validate(custom(function = "validate_gender"))]
    pub gender: Option<String>,

    #[validate(length(max = 255))]
    pub nationality: Option<String>,

    #[validate(length(min = 1, max = 255))]
    pub country: Option<String>,

    #[validate(url)]
    #[validate(length(max = 500))]
    pub profile_picture_url: Option<String>,
}

// Validation helper
fn validate_gender(gender: &str) -> Result<(), validator::ValidationError> {
    const VALID_GENDERS: &[&str] = &["M", "F", "MX"];

    if VALID_GENDERS.contains(&gender) {
        Ok(())
    } else {
        Err(validator::ValidationError::new("invalid_gender"))
    }
}

impl From<crate::models::Athlete> for AthleteResponse {
    fn from(athlete: crate::models::Athlete) -> Self {
        Self {
            athlete_id: athlete.athlete_id,
            first_name: athlete.first_name,
            last_name: athlete.last_name,
            slug: athlete.slug,
            gender: athlete.gender,
            nationality: athlete.nationality,
            country: athlete.country,
            profile_picture_url: athlete.profile_picture_url,
            created_at: athlete.created_at,
        }
    }
}
