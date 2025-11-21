-- Add slug and slug_history columns to athletes table
ALTER TABLE athletes ADD COLUMN slug VARCHAR(255) UNIQUE;
ALTER TABLE athletes ADD COLUMN slug_history JSONB DEFAULT '[]'::jsonb;

-- Generate slugs for existing athletes
-- This will create slugs in format: firstname-lastname
-- For duplicates, it will append -2, -3, etc.
DO $$
DECLARE
    athlete_record RECORD;
    base_slug TEXT;
    final_slug TEXT;
    counter INTEGER;
BEGIN
    FOR athlete_record IN
        SELECT athlete_id, first_name, last_name
        FROM athletes
        WHERE slug IS NULL
        ORDER BY created_at
    LOOP
        -- Create base slug from first and last name
        base_slug := lower(
            regexp_replace(
                regexp_replace(
                    trim(athlete_record.first_name || '-' || athlete_record.last_name),
                    '[^a-zA-Z0-9-]', '', 'g'
                ),
                '-+', '-', 'g'
            )
        );

        -- Handle edge case of empty slug
        IF base_slug = '' OR base_slug = '-' THEN
            base_slug := 'athlete';
        END IF;

        -- Check if slug exists, if so append number
        final_slug := base_slug;
        counter := 2;

        WHILE EXISTS (SELECT 1 FROM athletes WHERE slug = final_slug) LOOP
            final_slug := base_slug || '-' || counter;
            counter := counter + 1;
        END LOOP;

        -- Update the athlete with the unique slug
        UPDATE athletes
        SET slug = final_slug
        WHERE athlete_id = athlete_record.athlete_id;
    END LOOP;
END $$;

-- Make slug NOT NULL after generating values
ALTER TABLE athletes ALTER COLUMN slug SET NOT NULL;

-- Create index on slug for faster lookups
CREATE INDEX athletes_slug_idx ON athletes(slug);

-- Create GIN index on slug_history for faster redirect lookups
CREATE INDEX athletes_slug_history_idx ON athletes USING GIN(slug_history);
