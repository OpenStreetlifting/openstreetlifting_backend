use std::collections::HashMap;

use crate::ImporterError;

/// The import specification for LiftControl competitions.
/// This defines the contract for importing from the LiftControl API platform.
///
/// LiftControl competitions consist of:
/// - A base slug that groups all sessions of a competition together
/// - Multiple sub-slugs representing individual sessions/divisions/time slots
#[derive(Debug, Clone)]
pub struct LiftControlSpec {
    /// The base slug used to group all sessions into one competition
    base_slug: String,
    /// Individual session/division slugs to fetch from the API
    sub_slugs: Vec<String>,
}

impl LiftControlSpec {
    /// Creates a new import specification with a base slug and sub-slugs
    pub fn new(base_slug: impl Into<String>, sub_slugs: Vec<String>) -> Self {
        Self {
            base_slug: base_slug.into(),
            sub_slugs,
        }
    }

    /// Returns the base slug
    pub fn base_slug(&self) -> &str {
        &self.base_slug
    }

    /// Returns the sub-slugs
    pub fn sub_slugs(&self) -> &[String] {
        &self.sub_slugs
    }

    /// Creates a spec from a predefined competition configuration
    pub fn from_config(config: &CompetitionConfig) -> Self {
        Self {
            base_slug: config.base_slug.clone(),
            sub_slugs: config.sub_slugs.clone(),
        }
    }
}

/// Represents a predefined competition configuration for LiftControl.
/// This encapsulates all the information needed to import a specific competition.
#[derive(Debug, Clone)]
pub struct CompetitionConfig {
    /// The competition identifier
    pub id: CompetitionId,
    /// The base slug for grouping all sessions
    pub base_slug: String,
    /// All session/division slugs for this competition
    pub sub_slugs: Vec<String>,
}

impl CompetitionConfig {
    /// Creates a new competition configuration
    pub fn new(id: CompetitionId, base_slug: impl Into<String>, sub_slugs: Vec<String>) -> Self {
        Self {
            id,
            base_slug: base_slug.into(),
            sub_slugs,
        }
    }
}

/// Strongly-typed competition identifiers for LiftControl.
/// This enum represents all predefined competitions that can be imported.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CompetitionId {
    Annecy4Lift2025,
    // Add more competitions here as they become available:
    // Paris4Lift2025,
    // Lyon4Lift2025,
}

impl CompetitionId {
    /// Returns the canonical string identifier
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Annecy4Lift2025 => "annecy-4-lift-2025",
        }
    }

    /// Returns all available competition IDs
    pub fn all() -> &'static [CompetitionId] {
        &[Self::Annecy4Lift2025]
    }

    /// Parse a competition ID from a string (internal helper)
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

// Implement TryFrom<&str> for ergonomic conversion with try_into()
impl TryFrom<&str> for CompetitionId {
    type Error = ImporterError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::parse_str(value)
    }
}

// Implement FromStr to enable .parse() method
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
///
/// # Usage
/// ```ignore
/// let registry = LiftControlRegistry::new();
///
/// // Get a predefined competition
/// let spec = registry.get_spec(CompetitionId::Annecy4Lift2025).unwrap();
///
/// // List all available competitions
/// for id in registry.list_competitions() {
///     println!("{}", id);
/// }
/// ```
pub struct LiftControlRegistry {
    competitions: HashMap<CompetitionId, CompetitionConfig>,
}

impl LiftControlRegistry {
    /// Creates a new registry with all predefined competitions
    pub fn new() -> Self {
        let mut registry = Self {
            competitions: HashMap::new(),
        };

        // Register Annecy 4 Lift 2025
        // This competition has two sessions: Sunday morning and Sunday afternoon
        registry.register(CompetitionConfig::new(
            CompetitionId::Annecy4Lift2025,
            "annecy-4-lift-2025",
            vec![
                "annecy-4-lift-2025-dimanche-matin-39".to_string(),
                "annecy-4-lift-2025-dimanche-apres-midi-40".to_string(),
            ],
        ));

        // Future competitions can be added here:
        // registry.register(CompetitionConfig::new(
        //     CompetitionId::Paris4Lift2025,
        //     "paris-4-lift-2025",
        //     vec![
        //         "paris-4-lift-2025-morning-session".to_string(),
        //         "paris-4-lift-2025-evening-session".to_string(),
        //     ],
        // ));

        registry
    }

    /// Registers a competition configuration
    fn register(&mut self, config: CompetitionConfig) {
        self.competitions.insert(config.id, config);
    }

    /// Retrieves a competition configuration by ID
    pub fn get_config(&self, id: CompetitionId) -> Option<&CompetitionConfig> {
        self.competitions.get(&id)
    }

    /// Lists all available competition IDs
    pub fn list_competitions(&self) -> Vec<CompetitionId> {
        self.competitions.keys().copied().collect()
    }

    /// Creates an import spec from a competition ID
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

        // Test TryFrom with exact slug
        let result = CompetitionId::try_from("annecy-4-lift-2025");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), CompetitionId::Annecy4Lift2025);

        // Test FromStr with uppercase variant
        let result = CompetitionId::from_str("ANNECY");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), CompetitionId::Annecy4Lift2025);

        // Test parse() method with short name
        let result = "annecy".parse::<CompetitionId>();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), CompetitionId::Annecy4Lift2025);

        // Test various valid formats
        assert!(CompetitionId::from_str("annecy4lift2025").is_ok());
        assert!(CompetitionId::try_from("Annecy-4-Lift-2025").is_ok());

        // Test invalid input
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
