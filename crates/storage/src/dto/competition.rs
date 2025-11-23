use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum RankingScope {
    Group,
    Category,
    Competition,
}

impl Default for RankingScope {
    fn default() -> Self {
        Self::Group
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct RankingScopeQuery {
    #[serde(default)]
    pub ranking_scope: RankingScope,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct CreateCompetitionRequest {
    #[validate(length(
        min = 1,
        max = 255,
        message = "Name must be between 1 and 255 characters"
    ))]
    pub name: String,

    #[validate(length(
        min = 1,
        max = 255,
        message = "Slug must be between 1 and 255 characters"
    ))]
    #[validate(custom(function = "validate_slug"))]
    pub slug: String,

    #[validate(custom(function = "validate_status"))]
    #[serde(default = "default_status")]
    pub status: String,

    pub federation_id: Uuid,

    #[validate(length(max = 255))]
    pub venue: Option<String>,

    #[validate(length(max = 255))]
    pub city: Option<String>,

    #[validate(length(max = 255))]
    pub country: Option<String>,

    pub start_date: Option<NaiveDate>,

    pub end_date: Option<NaiveDate>,

    pub number_of_judge: Option<i16>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct UpdateCompetitionRequest {
    #[validate(length(min = 1, max = 255))]
    pub name: Option<String>,

    #[validate(length(min = 1, max = 255))]
    #[validate(custom(function = "validate_slug"))]
    pub slug: Option<String>,

    #[validate(custom(function = "validate_status"))]
    pub status: Option<String>,

    pub federation_id: Option<Uuid>,

    #[validate(length(max = 255))]
    pub venue: Option<String>,

    #[validate(length(max = 255))]
    pub city: Option<String>,

    #[validate(length(max = 255))]
    pub country: Option<String>,

    pub start_date: Option<NaiveDate>,

    pub end_date: Option<NaiveDate>,

    pub number_of_judge: Option<i16>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CompetitionResponse {
    pub competition_id: Uuid,
    pub name: String,
    pub created_at: chrono::NaiveDateTime,
    pub slug: String,
    pub status: String,
    pub federation_id: Uuid,
    pub venue: Option<String>,
    pub city: Option<String>,
    pub country: Option<String>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub number_of_judge: Option<i16>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CompetitionListResponse {
    pub competition_id: Uuid,
    pub name: String,
    pub created_at: chrono::NaiveDateTime,
    pub slug: String,
    pub status: String,
    pub venue: Option<String>,
    pub city: Option<String>,
    pub country: Option<String>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub federation: FederationInfo,
    pub movements: Vec<MovementInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct FederationInfo {
    pub federation_id: Uuid,
    pub name: String,
    pub abbreviation: Option<String>,
    pub country: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MovementInfo {
    pub movement_name: String,
    pub is_required: bool,
    pub display_order: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CompetitionDetailResponse {
    pub competition_id: uuid::Uuid,
    pub name: String,
    pub slug: String,
    pub status: String,
    pub venue: Option<String>,
    pub city: Option<String>,
    pub country: Option<String>,
    pub start_date: Option<chrono::NaiveDate>,
    pub end_date: Option<chrono::NaiveDate>,
    pub federation: FederationInfo,
    pub categories: Vec<CategoryDetail>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CategoryDetail {
    pub category: CategoryInfo,
    pub participants: Vec<ParticipantDetail>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CategoryInfo {
    pub category_id: uuid::Uuid,
    pub name: String,
    pub gender: String,
    pub weight_class_min: Option<rust_decimal::Decimal>,
    pub weight_class_max: Option<rust_decimal::Decimal>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ParticipantDetail {
    pub athlete: AthleteInfo,
    pub bodyweight: Option<rust_decimal::Decimal>,
    pub rank: Option<i32>,
    pub category_rank: Option<i32>,
    pub competition_rank: Option<i32>,
    pub ris_score: Option<rust_decimal::Decimal>,
    pub is_disqualified: bool,
    pub disqualified_reason: Option<String>,
    pub lifts: Vec<LiftDetail>,
    pub total: rust_decimal::Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AthleteInfo {
    pub athlete_id: uuid::Uuid,
    pub first_name: String,
    pub last_name: String,
    pub gender: String,
    pub nationality: Option<String>,
    pub country: String,
    pub slug: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct LiftDetail {
    pub movement_name: String,
    pub best_weight: rust_decimal::Decimal,
    pub attempts: Vec<AttemptInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AttemptInfo {
    pub attempt_number: i16,
    pub weight: rust_decimal::Decimal,
    pub is_successful: bool,
    pub passing_judges: Option<i16>,
    pub no_rep_reason: Option<String>,
}

fn default_status() -> String {
    "draft".to_string()
}

fn validate_slug(slug: &str) -> Result<(), validator::ValidationError> {
    let is_valid = slug
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        && !slug.starts_with('-')
        && !slug.ends_with('-')
        && !slug.contains("--");

    if is_valid {
        Ok(())
    } else {
        Err(validator::ValidationError::new("invalid_slug"))
    }
}

fn validate_status(status: &str) -> Result<(), validator::ValidationError> {
    const VALID_STATUSES: &[&str] = &["draft", "upcoming", "live", "completed", "cancelled"];

    if VALID_STATUSES.contains(&status) {
        Ok(())
    } else {
        Err(validator::ValidationError::new("invalid_status"))
    }
}

impl CreateCompetitionRequest {
    pub fn validate_dates(&self) -> Result<(), &'static str> {
        if let (Some(end), Some(start)) = (self.end_date, self.start_date)
            && end < start
        {
            return Err("End date must be on or after start date");
        }

        if let Some(judges) = self.number_of_judge
            && judges != 1
            && judges != 3
        {
            return Err("Number of judges must be 1 or 3");
        }

        Ok(())
    }
}

impl From<crate::models::Competition> for CompetitionResponse {
    fn from(comp: crate::models::Competition) -> Self {
        Self {
            competition_id: comp.competition_id,
            name: comp.name,
            created_at: comp.created_at,
            slug: comp.slug,
            status: comp.status,
            federation_id: comp.federation_id,
            venue: comp.venue,
            city: comp.city,
            country: comp.country,
            start_date: comp.start_date,
            end_date: comp.end_date,
            number_of_judge: comp.number_of_judge,
        }
    }
}
