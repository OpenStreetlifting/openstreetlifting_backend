use std::collections::HashMap;

use crate::ImporterError;
use chrono::NaiveDate;

/// Metadata for a competition that cannot be inferred from the API
#[derive(Debug, Clone)]
pub struct CompetitionMetadata {
    pub name: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub venue: Option<String>,
    pub city: Option<String>,
    pub country: Option<String>,
    pub number_of_judges: Option<i16>,
    pub federation: FederationInfo,
    pub default_athlete_country: String,
    pub default_athlete_nationality: String,
}

#[derive(Debug, Clone)]
pub struct FederationInfo {
    pub name: String,
    pub abbreviation: String,
    pub country: String,
}

impl CompetitionMetadata {
    pub fn annecy_4lift_2025() -> Self {
        Self {
            name: "Annecy 4 Lift 2025".to_string(),
            start_date: NaiveDate::from_ymd_opt(2025, 11, 1).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2025, 11, 2).unwrap(),
            venue: Some("Oski Crossfit".to_string()),
            city: Some("Annecy".to_string()),
            country: Some("France".to_string()),
            number_of_judges: Some(3),
            federation: FederationInfo {
                name: "4Lift".to_string(),
                abbreviation: "4L".to_string(),
                country: "FR".to_string(),
            },
            default_athlete_country: "FR".to_string(),
            default_athlete_nationality: "French".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LiftControlSpec {
    base_slug: String,
    sub_slugs: Vec<String>,
    metadata: CompetitionMetadata,
}

impl LiftControlSpec {
    pub fn new(
        base_slug: impl Into<String>,
        sub_slugs: Vec<String>,
        metadata: CompetitionMetadata,
    ) -> Self {
        Self {
            base_slug: base_slug.into(),
            sub_slugs,
            metadata,
        }
    }

    pub fn base_slug(&self) -> &str {
        &self.base_slug
    }

    pub fn sub_slugs(&self) -> &[String] {
        &self.sub_slugs
    }

    pub fn metadata(&self) -> &CompetitionMetadata {
        &self.metadata
    }

    pub fn from_config(config: &CompetitionConfig) -> Self {
        Self {
            base_slug: config.base_slug.clone(),
            sub_slugs: config.sub_slugs.clone(),
            metadata: config.metadata.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CompetitionConfig {
    pub id: CompetitionId,
    pub base_slug: String,
    pub sub_slugs: Vec<String>,
    pub metadata: CompetitionMetadata,
}

impl CompetitionConfig {
    pub fn new(
        id: CompetitionId,
        base_slug: impl Into<String>,
        sub_slugs: Vec<String>,
        metadata: CompetitionMetadata,
    ) -> Self {
        Self {
            id,
            base_slug: base_slug.into(),
            sub_slugs,
            metadata,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CompetitionId {
    Annecy4Lift2025,
}

impl CompetitionId {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Annecy4Lift2025 => "annecy-4-lift-2025",
        }
    }

    pub fn all() -> &'static [CompetitionId] {
        &[Self::Annecy4Lift2025]
    }

    fn parse_str(s: &str) -> Result<Self, ImporterError> {
        let normalized = s.to_lowercase().replace('_', "-");
        match normalized.as_str() {
            "annecy-4-lift-2025" | "annecy4lift2025" | "annecy" => Ok(Self::Annecy4Lift2025),
            _ => Err(ImporterError::ImportError(format!(
                "Unknown competition: '{}'. Available: {}",
                s,
                Self::all()
                    .iter()
                    .map(|c| c.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ))),
        }
    }
}

impl TryFrom<&str> for CompetitionId {
    type Error = ImporterError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::parse_str(value)
    }
}

impl std::str::FromStr for CompetitionId {
    type Err = ImporterError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse_str(s)
    }
}

// Implement Display for pretty printing
impl std::fmt::Display for CompetitionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Registry of predefined LiftControl competitions.
/// This provides a central place to define all importable competitions
/// with their configuration (base slug + sub-slugs).
pub struct LiftControlRegistry {
    competitions: HashMap<CompetitionId, CompetitionConfig>,
}

impl LiftControlRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            competitions: HashMap::new(),
        };

        // Register Annecy 4 Lift 2025
        // has two open session (rest of for the moment closed)
        registry.register(CompetitionConfig::new(
            CompetitionId::Annecy4Lift2025,
            "annecy-4-lift-2025",
            vec![
                "annecy-4-lift-2025-dimanche-matin-39".to_string(),
                "annecy-4-lift-2025-dimanche-apres-midi-40".to_string(),
            ],
            CompetitionMetadata::annecy_4lift_2025(),
        ));

        registry
    }

    fn register(&mut self, config: CompetitionConfig) {
        self.competitions.insert(config.id, config);
    }

    pub fn get_config(&self, id: CompetitionId) -> Option<&CompetitionConfig> {
        self.competitions.get(&id)
    }

    pub fn list_competitions(&self) -> Vec<CompetitionId> {
        self.competitions.keys().copied().collect()
    }

    pub fn get_spec(&self, id: CompetitionId) -> Option<LiftControlSpec> {
        self.get_config(id).map(LiftControlSpec::from_config)
    }
}

impl Default for LiftControlRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_competition_id_parsing() {
        use std::str::FromStr;

        let result = CompetitionId::try_from("annecy-4-lift-2025");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), CompetitionId::Annecy4Lift2025);

        let result = CompetitionId::from_str("ANNECY");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), CompetitionId::Annecy4Lift2025);

        let result = "annecy".parse::<CompetitionId>();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), CompetitionId::Annecy4Lift2025);

        assert!(CompetitionId::from_str("annecy4lift2025").is_ok());
        assert!(CompetitionId::try_from("Annecy-4-Lift-2025").is_ok());

        assert!(CompetitionId::from_str("unknown").is_err());
        assert!(CompetitionId::try_from("invalid").is_err());
        assert!("paris".parse::<CompetitionId>().is_err());
    }

    #[test]
    fn test_registry_get_config() {
        let registry = LiftControlRegistry::new();
        let config = registry.get_config(CompetitionId::Annecy4Lift2025).unwrap();

        assert_eq!(config.base_slug, "annecy-4-lift-2025");
        assert_eq!(config.sub_slugs.len(), 2);
    }

    #[test]
    fn test_create_spec_from_registry() {
        let registry = LiftControlRegistry::new();
        let spec = registry.get_spec(CompetitionId::Annecy4Lift2025).unwrap();

        assert_eq!(spec.base_slug(), "annecy-4-lift-2025");
        assert_eq!(spec.sub_slugs().len(), 2);
    }

    #[test]
    fn test_list_competitions() {
        let registry = LiftControlRegistry::new();
        let competitions = registry.list_competitions();

        assert!(!competitions.is_empty());
        assert!(competitions.contains(&CompetitionId::Annecy4Lift2025));
    }
}
