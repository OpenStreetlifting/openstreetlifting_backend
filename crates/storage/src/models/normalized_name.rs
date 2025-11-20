/// A wrapper that normalizes athlete names for consistent database storage.
/// Applies trimming and case normalization to prevent duplicates from inconsistent
/// formatting (e.g., "JOHN SMITH", "john smith", "John Smith" are all normalized the same).
///
/// The actual first name and last name order is preserved as provided.
/// Duplicate prevention is handled by database constraints.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedAthleteName {
    /// The athlete's first name (normalized to title case)
    first_name: String,
    /// The athlete's last name (normalized to title case)
    last_name: String,
}

impl NormalizedAthleteName {
    /// Creates a new normalized athlete name from first and last name.
    /// Applies normalization: trims whitespace and converts to title case.
    ///
    /// # Examples
    ///
    /// ```
    /// use storage::models::NormalizedAthleteName;
    ///
    /// let name1 = NormalizedAthleteName::new("john", "smith");
    /// let name2 = NormalizedAthleteName::new("JOHN", "SMITH");
    /// let name3 = NormalizedAthleteName::new("John", "Smith");
    ///
    /// // All produce the same normalized form: "John" "Smith"
    /// assert_eq!(name1.database_first_name(), "John");
    /// assert_eq!(name1.database_last_name(), "Smith");
    /// assert_eq!(name1, name2);
    /// assert_eq!(name2, name3);
    /// ```
    pub fn new(first_name: impl Into<String>, last_name: impl Into<String>) -> Self {
        let first_name = normalize_name_part(first_name.into());
        let last_name = normalize_name_part(last_name.into());

        Self {
            first_name,
            last_name,
        }
    }

    /// Returns the first name for database storage
    pub fn database_first_name(&self) -> &str {
        &self.first_name
    }

    /// Returns the last name for database storage
    pub fn database_last_name(&self) -> &str {
        &self.last_name
    }

    /// Returns both parts as a tuple (first_name, last_name) for database storage
    pub fn as_database_tuple(&self) -> (&str, &str) {
        (&self.first_name, &self.last_name)
    }
}

/// Normalizes a name part by trimming and converting to title case
fn normalize_name_part(name: String) -> String {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    // Convert to title case: first letter uppercase, rest lowercase
    let mut chars = trimmed.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalization_preserves_order() {
        let name = NormalizedAthleteName::new("John", "Smith");
        assert_eq!(name.database_first_name(), "John");
        assert_eq!(name.database_last_name(), "Smith");
    }

    #[test]
    fn test_normalization_title_case() {
        let name1 = NormalizedAthleteName::new("john", "smith");
        let name2 = NormalizedAthleteName::new("JOHN", "SMITH");
        let name3 = NormalizedAthleteName::new("John", "Smith");

        assert_eq!(name1.database_first_name(), "John");
        assert_eq!(name1.database_last_name(), "Smith");
        assert_eq!(name1, name2);
        assert_eq!(name2, name3);
    }

    #[test]
    fn test_normalization_trims_whitespace() {
        let name = NormalizedAthleteName::new("  John  ", "  Smith  ");
        assert_eq!(name.database_first_name(), "John");
        assert_eq!(name.database_last_name(), "Smith");
    }

    #[test]
    fn test_different_names_not_equal() {
        let name1 = NormalizedAthleteName::new("John", "Smith");
        let name2 = NormalizedAthleteName::new("Jane", "Smith");
        assert_ne!(name1, name2);
    }
}
