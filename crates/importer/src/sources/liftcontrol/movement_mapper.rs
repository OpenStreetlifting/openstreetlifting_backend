use crate::movement_mapper::{CanonicalMovement, MovementMapper};

pub struct LiftControlMovementMapper;

impl MovementMapper for LiftControlMovementMapper {
    fn map_movement(&self, name: &str) -> Option<CanonicalMovement> {
        match name.to_lowercase().as_str() {
            "traction" => Some(CanonicalMovement::PullUp),
            "dips" => Some(CanonicalMovement::Dips),
            "muscle-up" | "muscle up" | "muscleup" => Some(CanonicalMovement::MuscleUp),
            "squat" => Some(CanonicalMovement::Squat),
            _ => None,
        }
    }
}
