use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalFormat {
    pub format_version: String,
    pub source: SourceMetadata,
    pub competition: CompetitionData,
    pub movements: Vec<MovementData>,
    pub categories: Vec<CategoryData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub liftcontrol_metadata: Option<LiftControlMetadata>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pdf_metadata: Option<PdfMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceMetadata {
    #[serde(rename = "type")]
    pub r#type: SourceType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    pub extracted_at: DateTime<Utc>,
    pub extractor: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original_filename: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SourceType {
    LiftControl,
    Pdf,
    Html,
    Csv,
    Manual,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompetitionData {
    pub name: String,
    pub slug: String,
    pub federation: FederationData,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub venue: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub city: Option<String>,
    pub country: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number_of_judges: Option<i16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationData {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slug: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub abbreviation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MovementData {
    pub name: String,
    pub order: i16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_required: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryData {
    pub name: String,
    pub gender: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight_class_min: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight_class_max: Option<Decimal>,
    pub athletes: Vec<AthleteData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AthleteData {
    pub first_name: String,
    pub last_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gender: Option<String>,
    pub country: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nationality: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bodyweight: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_disqualified: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disqualified_reason: Option<String>,
    pub lifts: Vec<LiftData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub liftcontrol_athlete_metadata: Option<LiftControlAthleteMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiftData {
    pub movement: String,
    pub attempts: Vec<AttemptData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttemptData {
    pub attempt_number: i16,
    pub weight: Decimal,
    pub is_successful: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_rep_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiftControlMetadata {
    pub contest_id: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiftControlAthleteMetadata {
    pub athlete_id: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reglage_dips: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reglage_squat: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extraction_confidence: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pages_processed: Option<Vec<i32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warnings: Option<Vec<String>>,
}
