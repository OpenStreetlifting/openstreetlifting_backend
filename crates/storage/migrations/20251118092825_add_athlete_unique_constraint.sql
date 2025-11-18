-- Add unique constraint to prevent duplicate athletes regardless of name order
-- This constraint ensures that we cannot insert "John Smith" and "Smith John" as separate athletes

-- First, we need to ensure the names are always stored in a consistent order
-- We'll use a constraint that requires first_name <= last_name alphabetically
-- combined with unique constraint on the normalized pair

-- Add a unique constraint on (LEAST(first_name, last_name), GREATEST(first_name, last_name), gender, country)
-- This ensures that "John Smith" and "Smith John" are treated as duplicates
CREATE UNIQUE INDEX athletes_unique_normalized_names
ON athletes (
    LEAST(LOWER(first_name), LOWER(last_name)),
    GREATEST(LOWER(first_name), LOWER(last_name)),
    gender,
    country
);

-- Note: This will prevent duplicates at the database level, regardless of which field contains which name
