use super::models::CanonicalFormat;
use crate::{ImporterError, Result};
use std::collections::HashSet;
use tracing::warn;

pub struct CanonicalValidator;

impl CanonicalValidator {
    pub fn validate(canonical: &CanonicalFormat) -> Result<ValidationReport> {
        let mut report = ValidationReport::default();

        if canonical.format_version != "1.0.0" {
            report.errors.push(format!(
                "Unsupported format version: {}. Expected 1.0.0",
                canonical.format_version
            ));
        }

        if canonical.competition.name.is_empty() {
            report
                .errors
                .push("Competition name is required".to_string());
        }
        if canonical.competition.slug.is_empty() {
            report
                .errors
                .push("Competition slug is required".to_string());
        }
        if canonical.competition.country.is_empty() {
            report
                .errors
                .push("Competition country is required".to_string());
        }
        if canonical.competition.end_date < canonical.competition.start_date {
            report
                .errors
                .push("Competition end_date must be >= start_date".to_string());
        }

        if canonical.competition.federation.name.is_empty() {
            report
                .errors
                .push("Federation name is required".to_string());
        }

        if canonical.competition.venue.is_none() {
            report
                .warnings
                .push("Competition venue is not specified".to_string());
        }
        if canonical.competition.city.is_none() {
            report
                .warnings
                .push("Competition city is not specified".to_string());
        }
        if canonical.competition.number_of_judges.is_none() {
            report
                .warnings
                .push("Number of judges is not specified".to_string());
        }

        if canonical.movements.is_empty() {
            report
                .errors
                .push("At least one movement is required".to_string());
        }

        let mut movement_names = HashSet::new();
        for movement in &canonical.movements {
            if movement.name.is_empty() {
                report
                    .errors
                    .push("Movement name cannot be empty".to_string());
            }
            if movement.order < 1 {
                report.errors.push(format!(
                    "Movement '{}' has invalid order: {}. Order must be >= 1",
                    movement.name, movement.order
                ));
            }
            if !movement_names.insert(&movement.name) {
                report
                    .errors
                    .push(format!("Duplicate movement name: '{}'", movement.name));
            }
        }

        if canonical.categories.is_empty() {
            report
                .errors
                .push("At least one category is required".to_string());
        }

        for category in &canonical.categories {
            if category.name.is_empty() {
                report
                    .errors
                    .push("Category name cannot be empty".to_string());
            }
            if category.gender != "M" && category.gender != "F" {
                report.errors.push(format!(
                    "Invalid gender in category '{}': '{}'. Must be 'M' or 'F'",
                    category.name, category.gender
                ));
            }

            if category.athletes.is_empty() {
                report
                    .warnings
                    .push(format!("Category '{}' has no athletes", category.name));
            }

            for (idx, athlete) in category.athletes.iter().enumerate() {
                let athlete_label =
                    format!("{}. {} {}", idx + 1, athlete.first_name, athlete.last_name);

                if athlete.first_name.is_empty() {
                    report.errors.push(format!(
                        "Athlete in category '{}' has empty first_name",
                        category.name
                    ));
                }
                if athlete.last_name.is_empty() {
                    report.errors.push(format!(
                        "Athlete in category '{}' has empty last_name",
                        category.name
                    ));
                }
                if athlete.country.is_empty() {
                    report
                        .errors
                        .push(format!("Athlete '{}' has empty country", athlete_label));
                }

                if athlete.bodyweight.is_none() {
                    report
                        .warnings
                        .push(format!("Athlete '{}' is missing bodyweight", athlete_label));
                }

                if athlete.lifts.is_empty() {
                    report
                        .warnings
                        .push(format!("Athlete '{}' has no lifts", athlete_label));
                }

                for lift in &athlete.lifts {
                    if !movement_names.contains(&lift.movement) {
                        report.errors.push(format!(
                            "Athlete '{}' has lift for unknown movement: '{}'",
                            athlete_label, lift.movement
                        ));
                    }

                    if lift.attempts.is_empty() {
                        report.errors.push(format!(
                            "Athlete '{}' has lift '{}' with no attempts",
                            athlete_label, lift.movement
                        ));
                    }

                    for attempt in &lift.attempts {
                        if attempt.attempt_number < 1 || attempt.attempt_number > 3 {
                            report.errors.push(format!(
                                "Athlete '{}', movement '{}': invalid attempt_number {}. Must be 1-3",
                                athlete_label, lift.movement, attempt.attempt_number
                            ));
                        }
                        if attempt.weight.is_sign_negative() {
                            report.errors.push(format!(
                                "Athlete '{}', movement '{}', attempt {}: negative weight",
                                athlete_label, lift.movement, attempt.attempt_number
                            ));
                        }
                    }
                }
            }
        }

        if !report.errors.is_empty() {
            Err(ImporterError::ValidationError(format!(
                "Validation failed with {} error(s): {}",
                report.errors.len(),
                report.errors.join("; ")
            )))
        } else {
            Ok(report)
        }
    }
}

#[derive(Debug, Default)]
pub struct ValidationReport {
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl ValidationReport {
    pub fn log_warnings(&self) {
        for warning in &self.warnings {
            warn!("{}", warning);
        }
    }
}
