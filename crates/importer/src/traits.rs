use crate::Result;
use sqlx::PgPool;

/// Context passed to all importers, containing shared resources
pub struct ImportContext {
    pub pool: PgPool,
}

/// The main importer trait that all competition importers must implement.
/// Each importer type (LiftControl, Spreadsheet, PDF, etc.) implements this trait
/// with its own specification type that defines the contract for that source.
///
/// # Type Parameters
/// * `Spec` - The specification type that defines what this importer needs to operate
///
/// # Example
/// ```ignore
/// struct LiftControlImporter { ... }
///
/// #[async_trait::async_trait]
/// impl CompetitionImporter for LiftControlImporter {
///     type Spec = LiftControlSpec;
///
///     async fn import(&self, spec: &Self::Spec, context: &ImportContext) -> Result<()> {
///         // Implementation
///     }
/// }
/// ```
#[async_trait::async_trait]
pub trait CompetitionImporter: Send + Sync {
    /// The specification type that defines the contract for this importer
    type Spec: Send + Sync;

    /// Imports competition data according to the provided specification
    async fn import(&self, spec: &Self::Spec, context: &ImportContext) -> Result<()>;

    /// Returns a human-readable name for this importer (e.g., "LiftControl", "Spreadsheet")
    fn name(&self) -> &'static str;
}
