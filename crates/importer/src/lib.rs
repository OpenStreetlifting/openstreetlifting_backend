pub mod error;
pub mod movement_mapper;
pub mod sources;
pub mod traits;

pub use error::{ImporterError, Result};
pub use movement_mapper::CanonicalMovement;
pub use traits::{CompetitionImporter, ImportContext};

// Re-export LiftControl types
pub use sources::liftcontrol::{
    CompetitionConfig as LiftControlConfig, CompetitionId as LiftControlCompetitionId,
    LiftControlImporter, LiftControlRegistry, LiftControlSpec,
};
