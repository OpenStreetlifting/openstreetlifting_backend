use chrono::{NaiveDate, NaiveDateTime};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

/// RIS (Relative Index for Streetlifting) formula version
///
/// Stores the formula constants for computing RIS scores for a specific year/version.
/// Formula: RIS = Total × 100 / (A + (K - A) / (1 + Q · e^(-B · (BW - v))))
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct RisFormulaVersion {
    pub formula_id: Uuid,
    pub year: i32,
    pub effective_from: NaiveDate,
    pub effective_until: Option<NaiveDate>,
    pub is_current: bool,

    // Men's constants
    pub men_a: Decimal,
    pub men_k: Decimal,
    pub men_b: Decimal,
    pub men_v: Decimal,
    pub men_q: Decimal,

    // Women's constants
    pub women_a: Decimal,
    pub women_k: Decimal,
    pub women_b: Decimal,
    pub women_v: Decimal,
    pub women_q: Decimal,

    pub notes: Option<String>,
    pub created_at: NaiveDateTime,
}

impl RisFormulaVersion {
    /// Get the formula constants for a specific gender
    pub fn constants_for_gender(&self, gender: &str) -> FormulaConstants {
        match gender.to_uppercase().as_str() {
            "M" | "MALE" | "MEN" => FormulaConstants {
                a: self.men_a,
                k: self.men_k,
                b: self.men_b,
                v: self.men_v,
                q: self.men_q,
            },
            "F" | "FEMALE" | "WOMEN" => FormulaConstants {
                a: self.women_a,
                k: self.women_k,
                b: self.women_b,
                v: self.women_v,
                q: self.women_q,
            },
            _ => {
                // Default to men's formula for unknown genders
                FormulaConstants {
                    a: self.men_a,
                    k: self.men_k,
                    b: self.men_b,
                    v: self.men_v,
                    q: self.men_q,
                }
            }
        }
    }
}

/// Formula constants for a specific gender
#[derive(Debug, Clone, Copy)]
pub struct FormulaConstants {
    pub a: Decimal,
    pub k: Decimal,
    pub b: Decimal,
    pub v: Decimal,
    pub q: Decimal,
}
