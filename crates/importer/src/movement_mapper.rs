#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CanonicalMovement {
    MuscleUp,
    PullUp,
    Dips,
    Squat,
}

impl CanonicalMovement {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::MuscleUp => "Muscle-up",
            Self::PullUp => "Pull-up",
            Self::Dips => "Dips",
            Self::Squat => "Squat",
        }
    }
}

pub trait MovementMapper {
    fn map_movement(&self, source_name: &str) -> Option<CanonicalMovement>;
}
