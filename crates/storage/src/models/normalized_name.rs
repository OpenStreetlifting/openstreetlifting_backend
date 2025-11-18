/// A newtype that ensures athlete names are stored in a normalized, consistent order
/// to prevent duplicates like "John Smith" and "Smith John" from being created.
///
/// The type enforces at compile-time that you must use the normalized form when
/// interacting with athlete names in the database.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedAthleteName {
    /// The first part of the name (alphabetically sorted)
    first: String,
    /// The second part of the name (alphabetically sorted)
    second: String,
}

impl NormalizedAthleteName {
    /// Creates a new normalized athlete name from two name parts.
    /// The names will be automatically sorted alphabetically (case-insensitive)
    /// to ensure consistent ordering regardless of input order.
    ///
    /// # Examples
    ///
    /// ```
    /// use storage::models::NormalizedAthleteName;
    ///
    /// let name1 = NormalizedAthleteName::new("John", "Smith");
    /// let name2 = NormalizedAthleteName::new("Smith", "John");
    ///
    /// // Both produce the same normalized form
    /// assert_eq!(name1, name2);
    /// ```
    pub fn new(name1: impl Into<String>, name2: impl Into<String>) -> Self {
        let name1 = name1.into();
        let name2 = name2.into();

        // Sort alphabetically (case-insensitive) to ensure consistent ordering
        if name1.to_lowercase() <= name2.to_lowercase() {
            Self {
                first: name1,
                second: name2,
            }
        } else {
            Self {
                first: name2,
                second: name1,
            }
        }
    }

    /// Returns the first name part (which might be either first or last name)
    /// This is stored in `first_name` column in the database
    pub fn database_first_name(&self) -> &str {
        &self.first
    }

    /// Returns the second name part (which might be either first or last name)
    /// This is stored in `last_name` column in the database
    pub fn database_last_name(&self) -> &str {
        &self.second
    }

    /// Returns both parts as a tuple (first_name, last_name) for database storage
    pub fn as_database_tuple(&self) -> (&str, &str) {
        (&self.first, &self.second)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalization_same_order() {
        let name = NormalizedAthleteName::new("Alice", "Smith");
        assert_eq!(name.database_first_name(), "Alice");
        assert_eq!(name.database_last_name(), "Smith");
    }

    #[test]
    fn test_normalization_reversed_order() {
        let name = NormalizedAthleteName::new("Smith", "Alice");
        assert_eq!(name.database_first_name(), "Alice");
        assert_eq!(name.database_last_name(), "Smith");
    }

    #[test]
    fn test_normalization_case_insensitive() {
        let name1 = NormalizedAthleteName::new("SMITH", "alice");
        let name2 = NormalizedAthleteName::new("alice", "SMITH");
        assert_eq!(name1, name2);
    }

    #[test]
    fn test_equality_regardless_of_input_order() {
        let name1 = NormalizedAthleteName::new("John", "Doe");
        let name2 = NormalizedAthleteName::new("Doe", "John");
        assert_eq!(name1, name2);
    }
}
